use ratatui::widgets::canvas::Context;
use std::fmt;

use crate::audio::AudioState;

// Trait for visualizations to implement
pub trait Visualization {
    fn render(&self, ctx: &mut Context, state: &AudioState);
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

// Visualization type enum for selecting visualization
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VisualizationType {
    Starfield,
    BarSpectrum,
    WaveForms,
}

impl fmt::Display for VisualizationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VisualizationType::Starfield => write!(f, "Starfield"),
            VisualizationType::BarSpectrum => write!(f, "Bar Spectrum"),
            VisualizationType::WaveForms => write!(f, "Wave Forms"),
        }
    }
}

// Module imports
mod bar_spectrum;
mod starfield;
mod waveforms;

// Re-exports
pub use bar_spectrum::BarSpectrumVisualization;
pub use starfield::StarfieldVisualization;
pub use waveforms::WaveFormsVisualization;

// Manager for handling visualizations
pub struct VisualizationManager {
    current_type: VisualizationType,
    starfield: StarfieldVisualization,
    bar_spectrum: BarSpectrumVisualization,
    waveforms: WaveFormsVisualization,
}

impl Default for VisualizationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl VisualizationManager {
    pub fn new() -> Self {
        VisualizationManager {
            current_type: VisualizationType::Starfield, // Default visualization
            starfield: StarfieldVisualization::new(),
            bar_spectrum: BarSpectrumVisualization::new(),
            waveforms: WaveFormsVisualization::new(),
        }
    }

    pub fn current_visualization(&self) -> &dyn Visualization {
        match self.current_type {
            VisualizationType::Starfield => &self.starfield,
            VisualizationType::BarSpectrum => &self.bar_spectrum,
            VisualizationType::WaveForms => &self.waveforms,
        }
    }

    pub fn set_visualization_type(&mut self, vis_type: VisualizationType) {
        self.current_type = vis_type;
    }

    pub fn current_type(&self) -> VisualizationType {
        self.current_type
    }

    pub fn get_available_visualizations(&self) -> Vec<(VisualizationType, &str, &str)> {
        vec![
            (
                VisualizationType::Starfield,
                self.starfield.name(),
                self.starfield.description(),
            ),
            (
                VisualizationType::BarSpectrum,
                self.bar_spectrum.name(),
                self.bar_spectrum.description(),
            ),
            (
                VisualizationType::WaveForms,
                self.waveforms.name(),
                self.waveforms.description(),
            ),
        ]
    }

    // These methods are commented out as they're not currently used,
    // but might be useful for keyboard shortcuts in the future
    /*
    pub fn next_visualization(&mut self) {
        self.current_type = match self.current_type {
            VisualizationType::Starfield => VisualizationType::BarSpectrum,
            VisualizationType::BarSpectrum => VisualizationType::WaveForms,
            VisualizationType::WaveForms => VisualizationType::Starfield,
        };
    }

    pub fn previous_visualization(&mut self) {
        self.current_type = match self.current_type {
            VisualizationType::Starfield => VisualizationType::WaveForms,
            VisualizationType::BarSpectrum => VisualizationType::Starfield,
            VisualizationType::WaveForms => VisualizationType::BarSpectrum,
        };
    }
    */

    pub fn render(&self, ctx: &mut Context, state: &AudioState) {
        self.current_visualization().render(ctx, state);
    }
}
