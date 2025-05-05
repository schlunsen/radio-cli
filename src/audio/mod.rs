use rand::Rng;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::io::{BufRead, BufReader};
use std::thread;

#[derive(Clone)]
pub struct StreamInfo {
    pub bitrate: String,
    pub format: String,
    pub station_name: String,
    pub current_song: Option<String>,
}

#[derive(Clone)]
pub struct AudioState {
    pub bars: Vec<u64>,
    pub is_playing: bool,
    pub stream_info: Option<StreamInfo>,
}

impl AudioState {
    pub fn new() -> Self {
        AudioState {
            bars: vec![0; 20],
            is_playing: false,
            stream_info: None,
        }
    }
    
    pub fn update_bars(&mut self) {
        if self.is_playing {
            let mut rng = rand::thread_rng();
            for bar in &mut self.bars {
                *bar = rng.gen_range(0..20);
            }
        } else {
            for bar in &mut self.bars {
                *bar = 0;
            }
        }
    }
}

pub struct AudioVisualizer {
    pub state: Arc<Mutex<AudioState>>,
}

impl AudioVisualizer {
    pub fn new() -> Self {
        AudioVisualizer {
            state: Arc::new(Mutex::new(AudioState::new())),
        }
    }

    pub fn update(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.update_bars();
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

impl Player {
    pub fn new() -> Self {
        Player {
            current_player: None,
        }
    }
    
    pub fn play_station(&mut self, station_name: String, url: String, visualizer: &AudioVisualizer) -> Result<(), String> {
        // Kill any currently playing process
        self.stop();
        
        // Get the shared state handle for the background thread
        let state_handle = visualizer.get_state_handle();
        
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
                    for line in reader.lines() {
                        if let Ok(line) = line {
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
        if let Some(mut player) = self.current_player.take() {
            let _ = player.kill();
        }
    }
}