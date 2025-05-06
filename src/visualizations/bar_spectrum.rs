use super::Visualization;
use crate::audio::AudioState;
use ratatui::style::Color;
use ratatui::widgets::canvas::{Context, Rectangle};

pub struct BarSpectrumVisualization;

impl BarSpectrumVisualization {
    pub fn new() -> Self {
        BarSpectrumVisualization
    }
}

impl Visualization for BarSpectrumVisualization {
    fn name(&self) -> &str {
        "Bar Spectrum"
    }

    fn description(&self) -> &str {
        "Audio spectrum visualization with vertical bars"
    }

    fn render(&self, ctx: &mut Context, state: &AudioState) {
        // Background - dark background
        ctx.draw(&Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            color: Color::Rgb(10, 10, 20),
        });

        if state.is_playing {
            let num_bars = 30;
            let bar_width = 100.0 / num_bars as f64;
            let spacing = 1.0;
            let effective_width = bar_width - spacing;

            // Generate pseudo-random bar heights based on bass impact and frame count
            for i in 0..num_bars {
                // Create a pseudo-random height using various parameters
                let x_pos = (i as f64 / num_bars as f64) * 2.0 - 1.0; // Position from -1 to 1

                // Use a combination of sine waves with different phases
                // Based on frame count, position, and bass impact
                let t = state.frame_count as f64 * 0.02;

                // Height will be influenced by position, time, and bass impact
                let phase1 = t * 0.5 + x_pos * 3.0;
                let phase2 = t * 0.7 - x_pos * 2.0;
                let phase3 = t * 0.3 + x_pos * 4.0;

                // Combine multiple sine waves with different frequencies
                let base_height =
                    ((phase1.sin() * 0.5 + phase2.sin() * 0.3 + phase3.sin() * 0.2) + 1.0) / 2.0;

                // Apply bass impact to make it more dynamic
                let height = base_height * (0.3 + state.bass_impact * 0.7) * 70.0;

                // Determine color based on height and bass impact
                let intensity = (height / 70.0).min(1.0);
                let color = Color::Rgb(
                    ((0.2 + 0.8 * intensity) * 255.0) as u8,
                    ((0.5 - 0.3 * intensity + state.bass_impact * 0.3) * 255.0) as u8,
                    ((0.8 - 0.3 * intensity) * 255.0) as u8,
                );

                // Draw the bar
                ctx.draw(&Rectangle {
                    x: i as f64 * bar_width + spacing / 2.0,
                    y: 100.0 - height,
                    width: effective_width,
                    height,
                    color,
                });
            }

            // Draw bass impact indicator at the bottom
            let indicator_width = 50.0;
            let indicator_height = 3.0;
            let x = 50.0 - indicator_width / 2.0;
            let y = 95.0;

            // Background
            ctx.draw(&Rectangle {
                x,
                y,
                width: indicator_width,
                height: indicator_height,
                color: Color::Rgb(30, 30, 50),
            });

            // Filled portion based on bass impact
            ctx.draw(&Rectangle {
                x,
                y,
                width: indicator_width * state.bass_impact,
                height: indicator_height,
                color: Color::Rgb(
                    100 + (155.0 * state.bass_impact) as u8,
                    50 + (100.0 * (1.0 - state.bass_impact)) as u8,
                    200,
                ),
            });
        } else {
            // Draw a static pattern when not playing
            for i in 0..15 {
                let height = 5.0 + (i as f64 % 5.0) * 3.0;
                let x = 10.0 + i as f64 * 6.0;

                ctx.draw(&Rectangle {
                    x,
                    y: 50.0 - height / 2.0,
                    width: 3.0,
                    height,
                    color: Color::Rgb(50, 50, 100),
                });
            }
        }
    }
}
