use rand::Rng;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
// No need for PI constant in this version

#[derive(Clone)]
pub struct StreamInfo {
    pub bitrate: String,
    pub format: String,
    pub station_name: String,
    pub current_song: Option<String>,
}

#[derive(Clone)]
pub struct Star {
    pub x: f64,          // X position (-1.0 to 1.0 from center)
    pub y: f64,          // Y position (-1.0 to 1.0 from center)
    pub z: f64,          // Z position (depth, smaller = further away)
    pub brightness: f64, // How bright the star is (0.0-1.0)
    pub speed: f64,      // How fast the star moves
    pub color: u8,       // Color index
}

#[derive(Clone)]
pub struct AudioState {
    pub stars: Vec<Star>, // Stars for the starfield effect
    pub bass_impact: f64, // Bass impact value (0.0-1.0) for animations
    pub is_playing: bool,
    pub stream_info: Option<StreamInfo>,
    pub frame_count: u64, // Count frames for animations
    pub warp_speed: f64,  // Speed factor for the starfield (0.5-3.0)
}

impl Default for AudioState {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioState {
    pub fn new() -> Self {
        // Initialize stars for the starfield
        let mut stars = Vec::with_capacity(200); // 200 stars in the field
        let mut rng = rand::thread_rng();

        for _ in 0..200 {
            stars.push(Star {
                // Random position in 3D space
                x: rng.gen_range(-1.0..1.0), // X position (-1 to 1, center = 0)
                y: rng.gen_range(-1.0..1.0), // Y position (-1 to 1, center = 0)
                z: rng.gen_range(0.01..1.0), // Z position (depth, 0 = furthest)
                brightness: rng.gen_range(0.2..1.0), // Random brightness
                speed: rng.gen_range(0.005..0.02), // Speed factor
                color: rng.gen_range(0..5),  // Random color (0-4)
            });
        }

        AudioState {
            stars,
            bass_impact: 0.0,
            is_playing: false,
            stream_info: None,
            frame_count: 0,
            warp_speed: 1.0,
        }
    }

    pub fn update_visualization(&mut self) {
        // Increment frame counter
        self.frame_count += 1;
        let mut rng = rand::thread_rng();

        if self.is_playing {
            // 1. Update bass impact - affects starfield speed
            let bass_target = if rng.gen_bool(0.05) {
                // Occasional bass drop
                rng.gen_range(0.8..1.0)
            } else {
                rng.gen_range(0.2..0.6)
            };

            // Smooth bass impact changes
            self.bass_impact = self.bass_impact * 0.9 + bass_target * 0.1;

            // 2. Update warp speed based on bass impact
            self.warp_speed = 1.0 + self.bass_impact * 2.0; // 1.0 to 3.0

            // 3. Update each star position
            for star in &mut self.stars {
                // Move star closer (increase z)
                star.z += star.speed * self.warp_speed;

                // If star has passed the viewport (z > 1), reset it
                if star.z > 1.0 {
                    // Reset star to a new random position far away
                    star.x = rng.gen_range(-1.0..1.0);
                    star.y = rng.gen_range(-1.0..1.0);
                    star.z = 0.01; // Far away
                    star.brightness = rng.gen_range(0.2..1.0);
                    star.speed = rng.gen_range(0.005..0.02);

                    // Occasionally change color
                    if rng.gen_bool(0.3) {
                        star.color = rng.gen_range(0..5);
                    }
                }
            }

            // 4. Occasionally add new stars for visual variety
            if rng.gen_bool(0.05) && self.stars.len() < 250 {
                self.stars.push(Star {
                    x: rng.gen_range(-1.0..1.0),
                    y: rng.gen_range(-1.0..1.0),
                    z: 0.01, // Start far away
                    brightness: rng.gen_range(0.2..1.0),
                    speed: rng.gen_range(0.005..0.02),
                    color: rng.gen_range(0..5),
                });
            }
        } else {
            // When not playing, gradually slow down the starfield
            self.warp_speed = (self.warp_speed - 0.5) * 0.95 + 0.5;
            self.bass_impact *= 0.95;

            // Still update stars but at a much slower pace
            for star in &mut self.stars {
                star.z += star.speed * self.warp_speed * 0.2;

                if star.z > 1.0 {
                    star.x = rng.gen_range(-1.0..1.0);
                    star.y = rng.gen_range(-1.0..1.0);
                    star.z = 0.01;
                }
            }

            // Gradually reduce star count when not playing
            if rng.gen_bool(0.01) && self.stars.len() > 50 {
                self.stars.pop();
            }
        }
    }
}

pub struct AudioVisualizer {
    pub state: Arc<Mutex<AudioState>>,
}

impl Default for AudioVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioVisualizer {
    pub fn new() -> Self {
        AudioVisualizer {
            state: Arc::new(Mutex::new(AudioState::new())),
        }
    }

    pub fn update(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.update_visualization();
        }
    }

    pub fn set_playing(&self, playing: bool) {
        if let Ok(mut state) = self.state.lock() {
            state.is_playing = playing;
            if !playing {
                state.stream_info = None;
            }
        }
    }

    pub fn set_stream_info(&self, station_name: String, bitrate: String, format: String) {
        if let Ok(mut state) = self.state.lock() {
            state.stream_info = Some(StreamInfo {
                bitrate,
                format,
                station_name,
                current_song: None,
            });
        }
    }

    #[allow(dead_code)]
    pub fn update_current_song(&self, song: Option<String>) {
        if let Ok(mut state) = self.state.lock() {
            if let Some(info) = &mut state.stream_info {
                info.current_song = song;
            }
        }
    }

    pub fn get_state_handle(&self) -> Arc<Mutex<AudioState>> {
        Arc::clone(&self.state)
    }
}

pub struct Player {
    pub current_player: Option<Child>,
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl Player {
    pub fn new() -> Self {
        Player {
            current_player: None,
        }
    }

    pub fn play_station(
        &mut self,
        station_name: String,
        url: String,
        visualizer: &AudioVisualizer,
    ) -> Result<(), String> {
        // Kill any currently playing process
        self.stop();

        // Get the shared state handle for the background thread
        let state_handle = visualizer.get_state_handle();

        #[cfg(feature = "skip_mpv")]
        {
            // Simulation mode for Windows builds without MPV
            visualizer.set_stream_info(
                station_name.clone(),
                "Simulated".to_string(),
                "Demo Mode".to_string(),
            );
            visualizer.set_playing(true);

            // No actual player process in simulation mode
            // Just simulate playing
            return Ok(());
        }

        #[cfg(not(feature = "skip_mpv"))]
        match Command::new("mpv")
            .arg("--term-status-msg=STATUS: ${metadata/StreamTitle:} FORMAT: ${audio-codec} BITRATE: ${audio-bitrate}")
            .arg(url)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn() {
            Ok(mut child) => {
                // Get the stdout to read from it
                let stdout = child.stdout.take().expect("Failed to get stdout");

                // Set initial stream info
                visualizer.set_stream_info(
                    station_name.clone(),
                    "Detecting...".to_string(),
                    "Detecting...".to_string()
                );
                visualizer.set_playing(true);

                // Spawn a thread to read mpv output
                let vis_state = Arc::clone(&state_handle);
                thread::spawn(move || {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines().map_while(Result::ok) {
                        // Parse the line for stream metadata
                        if line.starts_with("STATUS:") {
                            if let Ok(mut state) = vis_state.lock() {
                                // Extract metadata from the line
                                let parts: Vec<&str> = line.split_whitespace().collect();
                                let mut song = None;
                                let mut format = "Unknown".to_string();
                                let mut bitrate = "Unknown".to_string();

                                // Parse the parts
                                let mut i = 1; // Start after "STATUS:"
                                while i < parts.len() {
                                    if parts[i] == "FORMAT:" && i + 1 < parts.len() {
                                        format = parts[i+1].to_string();
                                        i += 2;
                                    } else if parts[i] == "BITRATE:" && i + 1 < parts.len() {
                                        bitrate = format!("{} kbps", parts[i+1]);
                                        i += 2;
                                    } else {
                                        // Assume it's part of the song title
                                        if song.is_none() {
                                            song = Some(parts[i].to_string());
                                        } else {
                                            song = Some(format!("{} {}", song.unwrap(), parts[i]));
                                        }
                                        i += 1;
                                    }
                                }

                                // Update the stream info
                                if let Some(info) = &mut state.stream_info {
                                    info.format = format;
                                    info.bitrate = bitrate;
                                    info.current_song = song;
                                }
                            }
                        }
                    }
                });

                self.current_player = Some(child);
                Ok(())
            },
            Err(e) => {
                eprintln!("Failed to start player: {} (make sure mpv is installed)", e);
                visualizer.set_stream_info(
                    station_name,
                    "Error".to_string(),
                    format!("Failed to start: {}", e)
                );
                Err(e.to_string())
            },
        }
    }

    pub fn stop(&mut self) {
        #[cfg(not(feature = "skip_mpv"))]
        if let Some(mut player) = self.current_player.take() {
            let _ = player.kill();
        }

        #[cfg(feature = "skip_mpv")]
        {
            // Nothing to stop in simulation mode
            self.current_player = None;
        }
    }
}
