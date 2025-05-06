use super::Visualization;
use crate::audio::AudioState;
use ratatui::style::Color;
use ratatui::widgets::canvas::{Context, Line, Rectangle};

pub struct StarfieldVisualization;

impl Default for StarfieldVisualization {
    fn default() -> Self {
        Self::new()
    }
}

impl StarfieldVisualization {
    pub fn new() -> Self {
        StarfieldVisualization
    }
}

impl Visualization for StarfieldVisualization {
    fn name(&self) -> &str {
        "Starfield"
    }

    fn description(&self) -> &str {
        "3D starfield with warp effect"
    }

    fn render(&self, ctx: &mut Context, state: &AudioState) {
        // Background - dark space
        ctx.draw(&Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            color: Color::Rgb(0, 0, 20), // Very dark blue
        });

        // Star color palette
        let colors = [
            Color::Rgb(255, 255, 255), // White
            Color::Rgb(200, 200, 255), // Light blue
            Color::Rgb(255, 230, 200), // Light yellow
            Color::Rgb(255, 200, 200), // Light red
            Color::Rgb(200, 255, 200), // Light green
        ];

        // Draw each star in the starfield
        for star in &state.stars {
            // Calculate projected position based on perspective
            // As z gets closer to 1.0, the star gets closer to the center
            // and appears larger

            // Center of screen
            let center_x = 50.0;
            let center_y = 50.0;

            // Calculate projected position (perspective projection)
            // Higher z = closer to viewer = further from center
            let scale = 1.0 / (1.01 - star.z.min(0.99)); // Avoid division by zero
            let projected_x = center_x + star.x * scale * 40.0; // Scale factor for width
            let projected_y = center_y + star.y * scale * 40.0; // Scale factor for height

            // Calculate size based on z (closer = larger)
            // Use non-linear scaling for more dramatic effect
            let size = star.z.powf(2.0) * 3.0 * (state.warp_speed * 0.5 + 0.5);

            // Calculate brightness based on z and intrinsic brightness
            let brightness = star.brightness * star.z.powf(1.5);

            // Skip if star is out of bounds
            if !(0.0..=100.0).contains(&projected_x) || !(0.0..=100.0).contains(&projected_y) {
                continue;
            }

            // Get color for this star
            let base_color = colors[star.color as usize % colors.len()];

            // Adjust color based on brightness
            let color = match base_color {
                Color::Rgb(r, g, b) => Color::Rgb(
                    (r as f64 * brightness) as u8,
                    (g as f64 * brightness) as u8,
                    (b as f64 * brightness) as u8,
                ),
                _ => base_color,
            };

            // Draw the star
            ctx.draw(&Rectangle {
                x: projected_x - size / 2.0,
                y: projected_y - size / 2.0,
                width: size,
                height: size,
                color,
            });

            // For closer stars, add a trail/streak effect when at high warp speed
            if star.z > 0.7 && state.warp_speed > 1.5 {
                // Calculate trail length based on speed and z
                let trail_length = star.z * state.warp_speed * 5.0;

                // Direction from center
                let dx = projected_x - center_x;
                let dy = projected_y - center_y;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist > 0.0 {
                    // Normalized direction
                    let nx = dx / dist;
                    let ny = dy / dist;

                    // Draw trail pointing outward from center
                    let trail_x = projected_x - nx * trail_length;
                    let trail_y = projected_y - ny * trail_length;

                    // Make trail fade out
                    let trail_color = match color {
                        Color::Rgb(r, g, b) => Color::Rgb(
                            (r as f64 * 0.5) as u8,
                            (g as f64 * 0.5) as u8,
                            (b as f64 * 0.5) as u8,
                        ),
                        _ => color,
                    };

                    ctx.draw(&Line {
                        x1: trail_x,
                        y1: trail_y,
                        x2: projected_x,
                        y2: projected_y,
                        color: trail_color,
                    });
                }
            }
        }

        // Draw warp speed indicator
        let warp = state.warp_speed;
        if state.is_playing {
            let indicator_width = 20.0;
            let indicator_height = 5.0;
            let x = 50.0 - indicator_width / 2.0;
            let y = 90.0;

            // Draw background
            ctx.draw(&Rectangle {
                x,
                y,
                width: indicator_width,
                height: indicator_height,
                color: Color::Rgb(30, 30, 50),
            });

            // Draw filled portion based on warp speed (1.0 to 3.0 mapped to 0-100%)
            let fill_width = (warp - 1.0) / 2.0 * indicator_width;
            ctx.draw(&Rectangle {
                x,
                y,
                width: fill_width.min(indicator_width),
                height: indicator_height,
                color: Color::Rgb(
                    (100.0 + (155.0 * (warp - 1.0) / 2.0)) as u8,
                    (200.0 - (150.0 * (warp - 1.0) / 2.0)) as u8,
                    255,
                ),
            });
        }
    }
}
