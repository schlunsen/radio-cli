use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::assets;
use rodio::{OutputStream, Sink};

// SoundEffectManager handles playback of sound effects used during station switching
pub struct SoundEffectManager {
    tuning_sink: Option<Arc<Mutex<Sink>>>,
    _stream: Option<OutputStream>,
    _stream_handle: Option<rodio::OutputStreamHandle>,
}

impl SoundEffectManager {
    pub fn new() -> Self {
        // Initialize audio output stream
        let (stream, stream_handle) = match OutputStream::try_default() {
            Ok((stream, handle)) => (Some(stream), Some(handle)),
            Err(e) => {
                eprintln!(
                    "Warning: Failed to initialize audio for sound effects: {}",
                    e
                );
                (None, None)
            }
        };

        SoundEffectManager {
            tuning_sink: None,
            _stream: stream,
            _stream_handle: stream_handle,
        }
    }

    // Play the radio tuning sound effect
    pub fn play_tuning_sound(&mut self) -> Result<(), String> {
        if self._stream_handle.is_none() {
            return Err("Audio output not available".to_string());
        }

        // Create a new sink for this sound effect
        let sink = match Sink::try_new(self._stream_handle.as_ref().unwrap()) {
            Ok(sink) => sink,
            Err(e) => return Err(format!("Failed to create audio sink: {}", e)),
        };

        // Set volume for the tuning sound (higher volume to be more noticeable)
        sink.set_volume(1.0);

        // Load the embedded tuning sound directly from the binary
        let source = assets::get_radio_static_sound()?;

        // Play the sound
        sink.append(source);

        // Store the sink for later control
        self.tuning_sink = Some(Arc::new(Mutex::new(sink)));

        Ok(())
    }

    // Stop the tuning sound if it's playing
    pub fn stop_tuning_sound(&mut self) {
        if let Some(sink) = &self.tuning_sink {
            if let Ok(sink) = sink.lock() {
                sink.stop();
            }
        }
        self.tuning_sink = None;
    }

    // Fade out the tuning sound gradually
    pub fn fade_out_tuning_sound(&self, duration_ms: u64) {
        if let Some(sink_arc) = &self.tuning_sink {
            let sink_clone = Arc::clone(sink_arc);

            // Create a thread to handle the fade out
            thread::spawn(move || {
                // For longer fades, we want more steps for smoother transition
                let steps = if duration_ms > 3000 { 60 } else { 20 };
                let step_duration = duration_ms / steps as u64;

                // For longer durations (like 6 seconds), we want to:
                // 1. Keep full volume for a while
                // 2. Then gradually decrease

                // If it's a longer effect (> 3 seconds), hold at full volume for 1/3 of the time
                let hold_steps = if duration_ms > 3000 { steps / 3 } else { 0 };

                // Hold at full volume for the first part
                for _ in 0..hold_steps {
                    if let Ok(sink) = sink_clone.lock() {
                        sink.set_volume(1.0);
                    }
                    thread::sleep(Duration::from_millis(step_duration));
                }

                // Then fade out over the remaining time
                for i in 0..(steps - hold_steps) {
                    if let Ok(sink) = sink_clone.lock() {
                        // Non-linear fade for more natural sound (slower at first, then quicker)
                        let progress = i as f32 / (steps - hold_steps) as f32;
                        let volume = 1.0 * (1.0 - (progress * progress));
                        sink.set_volume(volume);
                    }
                    thread::sleep(Duration::from_millis(step_duration));
                }

                // Finally, stop the sound
                if let Ok(sink) = sink_clone.lock() {
                    sink.stop();
                }
            });
        }
    }
}
