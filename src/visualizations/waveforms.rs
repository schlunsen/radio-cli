use super::Visualization;
use crate::audio::AudioState;
use ratatui::style::Color;
use ratatui::widgets::canvas::{Context, Line};
use std::f64::consts::PI;

pub struct WaveFormsVisualization;

impl WaveFormsVisualization {
    pub fn new() -> Self {
        WaveFormsVisualization
    }
}

impl Visualization for WaveFormsVisualization {
    fn name(&self) -> &str {
        "Wave Forms"
    }

    fn description(&self) -> &str {
        "Oscilloscope-style wave form visualization"
    }

    fn render(&self, ctx: &mut Context, state: &AudioState) {
        // Background - gradient from dark blue to black
        for y in 0..100 {
            let color_intensity = (100 - y) as f64 * 0.2;
            let color = Color::Rgb(0, 0, (color_intensity as u8).max(5));

            ctx.draw(&Line {
                x1: 0.0,
                y1: y as f64,
                x2: 100.0,
                y2: y as f64,
                color,
            });
        }

        if state.is_playing {
            // Draw horizontal grid lines
            for y in (10..=90).step_by(20) {
                let y = y as f64;
                ctx.draw(&Line {
                    x1: 0.0,
                    y1: y,
                    x2: 100.0,
                    y2: y,
                    color: Color::Rgb(30, 30, 50),
                });
            }

            // Draw vertical grid lines
            for x in (10..=90).step_by(10) {
                let x = x as f64;
                ctx.draw(&Line {
                    x1: x,
                    y1: 0.0,
                    x2: x,
                    y2: 100.0,
                    color: Color::Rgb(30, 30, 50),
                });
            }

            // Generate sine wave visualization based on frame count and bass impact
            let t = state.frame_count as f64 * 0.02;
            let num_points = 100;
            let mut prev_x = 0.0;
            let mut prev_y = 50.0;

            for i in 1..=num_points {
                let x = i as f64 / num_points as f64 * 100.0;

                // Generate a more complex waveform using multiple frequencies
                let freq1 = 1.0 + state.bass_impact * 2.0; // Base frequency affected by bass
                let freq2 = 2.0 + state.bass_impact; // Second harmonic
                let freq3 = 4.0; // Higher harmonic

                // Combine different frequencies with varying amplitudes
                let amp1 = 15.0 + state.bass_impact * 10.0; // Main amplitude
                let amp2 = 5.0 * state.bass_impact; // Second amplitude affected strongly by bass
                let amp3 = 3.0; // Small high-frequency component

                // Calculate the waveform value
                let phase = x / 100.0 * 2.0 * PI + t;
                let wave = amp1 * (phase * freq1).sin()
                    + amp2 * (phase * freq2).sin()
                    + amp3 * (phase * freq3).sin() * state.bass_impact;

                // Center the wave in the display and apply scaling
                let y = 50.0 - wave;

                // Only draw lines inside the canvas boundaries
                if (0.0..=100.0).contains(&y) && (0.0..=100.0).contains(&prev_y) {
                    // Determine color based on bass impact and position
                    let intensity = 0.5 + state.bass_impact * 0.5;
                    let color = Color::Rgb(
                        ((0.2 + intensity * 0.8) * 255.0) as u8,
                        ((0.8 - intensity * 0.3) * 255.0) as u8,
                        ((0.7 + intensity * 0.3) * 255.0) as u8,
                    );

                    // Draw line segment
                    ctx.draw(&Line {
                        x1: prev_x,
                        y1: prev_y,
                        x2: x,
                        y2: y,
                        color,
                    });
                }

                prev_x = x;
                prev_y = y;
            }

            // Draw second wave (phase shifted) for more interesting effect
            prev_x = 0.0;
            prev_y = 50.0;

            for i in 1..=num_points {
                let x = i as f64 / num_points as f64 * 100.0;

                // Similar to first wave but with phase shift and different parameters
                let freq1 = 0.8 + state.bass_impact;
                let freq2 = 3.0 - state.bass_impact;
                let freq3 = 5.0;

                let amp1 = 10.0;
                let amp2 = 7.0 * state.bass_impact;
                let amp3 = 2.0;

                let phase = x / 100.0 * 2.0 * PI + t + PI / 2.0; // Phase shifted
                let wave = amp1 * (phase * freq1).cos()
                    + amp2 * (phase * freq2).sin()
                    + amp3 * (phase * freq3).cos() * state.bass_impact;

                let y = 50.0 - wave;

                if (0.0..=100.0).contains(&y) && (0.0..=100.0).contains(&prev_y) {
                    // Different color for second wave
                    let color = Color::Rgb(
                        ((0.7 - state.bass_impact * 0.2) * 255.0) as u8,
                        ((0.2 + state.bass_impact * 0.6) * 255.0) as u8,
                        ((0.8) * 255.0) as u8,
                    );

                    ctx.draw(&Line {
                        x1: prev_x,
                        y1: prev_y,
                        x2: x,
                        y2: y,
                        color,
                    });
                }

                prev_x = x;
                prev_y = y;
            }
        } else {
            // Draw a static pattern when not playing
            // Central horizontal line
            ctx.draw(&Line {
                x1: 0.0,
                y1: 50.0,
                x2: 100.0,
                y2: 50.0,
                color: Color::Rgb(50, 50, 80),
            });

            // Small pulses
            for i in 0..5 {
                let x = 10.0 + i as f64 * 20.0;

                ctx.draw(&Line {
                    x1: x - 5.0,
                    y1: 50.0,
                    x2: x,
                    y2: 45.0,
                    color: Color::Rgb(60, 60, 100),
                });

                ctx.draw(&Line {
                    x1: x,
                    y1: 45.0,
                    x2: x + 5.0,
                    y2: 50.0,
                    color: Color::Rgb(60, 60, 100),
                });
            }
        }
    }
}
