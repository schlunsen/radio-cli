use rand::Rng;
use std::io::{BufRead, BufReader, Write as IoWrite};
use std::os::unix::net::UnixStream;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

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
    pub is_muted: bool,
    pub volume: u8, // Volume level (0-100)
    pub stream_info: Option<StreamInfo>,
    pub frame_count: u64,              // Count frames for animations
    pub warp_speed: f64,               // Speed factor for the starfield (0.5-3.0)
    pub error_message: Option<String>, // Error message to display in UI
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
            is_muted: false,
            volume: 50, // Default volume at 50%
            stream_info: None,
            frame_count: 0,
            warp_speed: 1.0,
            error_message: None,
        }
    }

    pub fn update_visualization(&mut self) {
        // Increment frame counter
        self.frame_count += 1;
        let mut rng = rand::thread_rng();

        // Clear error messages after a few seconds (roughly 5 seconds at 60fps)
        if self.error_message.is_some() && self.frame_count % 300 == 0 {
            self.error_message = None;
        }

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

    pub fn set_muted(&self, muted: bool) {
        if let Ok(mut state) = self.state.lock() {
            state.is_muted = muted;
        }
    }

    pub fn set_error(&self, message: String) {
        if let Ok(mut state) = self.state.lock() {
            state.error_message = Some(message);
        }
    }

    // Increase volume
    pub fn increase_volume(&self) {
        if let Ok(mut state) = self.state.lock() {
            // Don't go above 100%
            if state.volume < 100 {
                state.volume = state.volume.saturating_add(5);
            }
        }
    }

    // Decrease volume
    pub fn decrease_volume(&self) {
        if let Ok(mut state) = self.state.lock() {
            // Don't go below 0%
            if state.volume > 0 {
                state.volume = state.volume.saturating_sub(5);
            }
        }
    }

    // Get the current volume
    #[allow(dead_code)]
    pub fn get_volume(&self) -> u8 {
        if let Ok(state) = self.state.lock() {
            state.volume
        } else {
            50 // Default volume if can't acquire lock
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

/// Send a JSON command to mpv via its IPC socket
fn send_mpv_command(socket_path: &str, command: &str) -> Result<(), String> {
    let mut stream = UnixStream::connect(socket_path)
        .map_err(|e| format!("Failed to connect to mpv socket: {}", e))?;
    // mpv IPC protocol: send JSON command followed by newline
    let cmd = format!("{{ \"command\": [{}] }}\n", command);
    stream
        .write_all(cmd.as_bytes())
        .map_err(|e| format!("Failed to send command to mpv: {}", e))?;
    Ok(())
}

pub struct Player {
    pub current_player: Option<Child>,
    pub is_muted: bool,
    socket_path: Option<String>, // Path to the mpv IPC socket
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
            is_muted: false,
            socket_path: None,
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
        {
            // Generate a unique socket path using the current process ID and a timestamp
            // to avoid conflicts between successive spawns
            let socket = format!(
                "/tmp/mpvsocket_{}_{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            );

            match Command::new("mpv")
                .arg("--term-status-msg=STATUS: ${metadata/StreamTitle:} FORMAT: ${audio-codec} BITRATE: ${audio-bitrate}")
                .arg(format!("--input-ipc-server={}", socket))
                .arg("--no-video")
                .arg(&url)
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(mut child) => {
                    // Get the stdout to read from it
                    let stdout = child.stdout.take().expect("Failed to get stdout");

                    // Set initial stream info
                    visualizer.set_stream_info(
                        station_name.clone(),
                        "Detecting...".to_string(),
                        "Detecting...".to_string(),
                    );
                    visualizer.set_playing(true);

                    // Clear any previous error
                    if let Ok(mut state) = visualizer.state.lock() {
                        state.error_message = None;
                    }

                    // Store the socket path
                    self.socket_path = Some(socket);

                    // Spawn a thread to read mpv output
                    let vis_state = Arc::clone(&state_handle);
                    thread::spawn(move || {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines().map_while(Result::ok) {
                            // Parse the line for stream metadata
                            if line.starts_with("STATUS:") {
                                if let Ok(mut state) = vis_state.lock() {
                                    // More robust metadata extraction
                                    let line_str = line.trim_start_matches("STATUS: ");

                                    // Find FORMAT: and BITRATE: sections more reliably
                                    let mut format = "Unknown".to_string();
                                    let mut bitrate = "Unknown".to_string();
                                    let mut song = None;

                                    // Extract format
                                    if let Some(format_idx) = line_str.find("FORMAT:") {
                                        // Find the end of the format value (next keyword or end of string)
                                        let format_start = format_idx + "FORMAT:".len();
                                        let format_end = line_str[format_start..]
                                            .find("BITRATE:")
                                            .map_or(line_str.len(), |pos| format_start + pos);

                                        // Extract and trim the format value
                                        format =
                                            line_str[format_start..format_end].trim().to_string();
                                    }

                                    // Extract bitrate
                                    if let Some(bitrate_idx) = line_str.find("BITRATE:") {
                                        // Get the rest of the line after BITRATE:
                                        let bitrate_start = bitrate_idx + "BITRATE:".len();
                                        let bitrate_value = line_str[bitrate_start..].trim();

                                        // Check if the bitrate value is not empty
                                        if !bitrate_value.is_empty() {
                                            bitrate = format!("{} kbps", bitrate_value);
                                        }
                                    }

                                    // Extract song
                                    // The song title is everything before FORMAT: or BITRATE:, whichever comes first
                                    let first_keyword = std::cmp::min(
                                        line_str.find("FORMAT:").unwrap_or(line_str.len()),
                                        line_str.find("BITRATE:").unwrap_or(line_str.len()),
                                    );
                                    let potential_song = line_str[..first_keyword].trim();
                                    if !potential_song.is_empty() {
                                        song = Some(potential_song.to_string());
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
                }
                Err(e) => {
                    let error_msg = if e.kind() == std::io::ErrorKind::NotFound {
                        "mpv not found. Please install mpv to play radio stations.".to_string()
                    } else {
                        format!("Failed to start player: {}", e)
                    };
                    visualizer.set_error(error_msg.clone());
                    Err(error_msg)
                }
            }
        }
    }

    pub fn stop(&mut self) {
        #[cfg(not(feature = "skip_mpv"))]
        if let Some(mut player) = self.current_player.take() {
            // Kill the player process and wait to reap it (prevent zombies)
            let _ = player.kill();
            let _ = player.wait();
        }

        #[cfg(feature = "skip_mpv")]
        {
            // Nothing to stop in simulation mode
            self.current_player = None;
        }

        // Clean up the socket file
        if let Some(ref socket) = self.socket_path {
            let _ = std::fs::remove_file(socket);
            self.socket_path = None;
        }

        // Reset the mute state when stopping
        self.is_muted = false;
    }

    #[allow(unused_variables)]
    pub fn toggle_mute(&mut self, visualizer: &AudioVisualizer) -> Result<(), String> {
        // Toggle the mute state
        self.is_muted = !self.is_muted;

        // Update the mute state in the visualizer
        visualizer.set_muted(self.is_muted);

        #[cfg(feature = "skip_mpv")]
        {
            // Nothing to do in simulation mode
            return Ok(());
        }

        #[cfg(not(feature = "skip_mpv"))]
        if self.current_player.is_some() {
            // Send mute toggle command via IPC socket
            if let Some(ref socket) = self.socket_path {
                if let Err(e) = send_mpv_command(socket, "\"cycle\", \"mute\"") {
                    // Log but don't fail - mute state is still tracked visually
                    eprintln!("Warning: Could not send mute command to mpv: {}", e);
                }
            }
            Ok(())
        } else {
            Err("No player is currently running".to_string())
        }
    }

    // Update metadata - called periodically by the App
    #[allow(dead_code)]
    pub fn update_metadata(&mut self, _visualizer: &AudioVisualizer) {
        // Nothing to do here with the command-line approach,
        // since metadata updates are handled in the background thread
        // that reads from mpv's stdout in the play_station function.
        // This function is included for API compatibility.
    }

    // Increase volume
    #[allow(unused_variables)]
    pub fn volume_up(&mut self, visualizer: &AudioVisualizer) -> Result<(), String> {
        #[cfg(feature = "skip_mpv")]
        {
            // Update volume in the visualizer even in simulation mode
            visualizer.increase_volume();
            return Ok(());
        }

        #[cfg(not(feature = "skip_mpv"))]
        if self.current_player.is_some() {
            // Send volume up command via IPC socket
            if let Some(ref socket) = self.socket_path {
                if let Err(e) = send_mpv_command(socket, "\"add\", \"volume\", \"5\"") {
                    eprintln!("Warning: Could not send volume command to mpv: {}", e);
                }
            }

            // Update volume in the visualizer state
            visualizer.increase_volume();
            Ok(())
        } else {
            Err("No player is currently running".to_string())
        }
    }

    // Decrease volume
    #[allow(unused_variables)]
    pub fn volume_down(&mut self, visualizer: &AudioVisualizer) -> Result<(), String> {
        #[cfg(feature = "skip_mpv")]
        {
            // Update volume in the visualizer even in simulation mode
            visualizer.decrease_volume();
            return Ok(());
        }

        #[cfg(not(feature = "skip_mpv"))]
        if self.current_player.is_some() {
            // Send volume down command via IPC socket
            if let Some(ref socket) = self.socket_path {
                if let Err(e) = send_mpv_command(socket, "\"add\", \"volume\", \"-5\"") {
                    eprintln!("Warning: Could not send volume command to mpv: {}", e);
                }
            }

            // Update volume in the visualizer state
            visualizer.decrease_volume();
            Ok(())
        } else {
            Err("No player is currently running".to_string())
        }
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
    }
}
