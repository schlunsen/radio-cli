use std::error::Error;
use std::io::{self, BufRead, BufReader};
use std::process::{Command, Child, Stdio};
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::thread;
use rand::Rng;

use rusqlite::{params, Connection};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, BarChart},
    Frame, Terminal,
};

struct Station {
    id: i32,
    name: String,
    url: String,
    favorite: bool,
}

#[derive(Clone)]
struct StreamInfo {
    bitrate: String,
    format: String,
    station_name: String,
    current_song: Option<String>,
}

#[derive(Clone)]
// Thread-safe shared audio visualizer state
struct AudioState {
    bars: Vec<u64>,
    is_playing: bool,
    stream_info: Option<StreamInfo>,
}

impl AudioState {
    fn new() -> Self {
        AudioState {
            bars: vec![0; 20],
            is_playing: false,
            stream_info: None,
        }
    }
    
    fn update_bars(&mut self) {
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

// Main audio visualizer that owns the shared state
struct AudioVisualizer {
    state: Arc<Mutex<AudioState>>,
}

impl AudioVisualizer {
    fn new() -> Self {
        AudioVisualizer {
            state: Arc::new(Mutex::new(AudioState::new())),
        }
    }

    fn update(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.update_bars();
        }
    }

    fn set_playing(&self, playing: bool) {
        if let Ok(mut state) = self.state.lock() {
            state.is_playing = playing;
            if !playing {
                state.stream_info = None;
            }
        }
    }
    
    fn set_stream_info(&self, station_name: String, bitrate: String, format: String) {
        if let Ok(mut state) = self.state.lock() {
            state.stream_info = Some(StreamInfo {
                bitrate,
                format,
                station_name,
                current_song: None,
            });
        }
    }
    
    fn update_current_song(&self, song: Option<String>) {
        if let Ok(mut state) = self.state.lock() {
            if let Some(info) = &mut state.stream_info {
                info.current_song = song;
            }
        }
    }
    
    // Get a clone of the state for threads to use
    fn get_state_handle(&self) -> Arc<Mutex<AudioState>> {
        Arc::clone(&self.state)
    }
}

fn init_db(conn: &Connection) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS stations (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            favorite INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )?;
    let count: i32 = conn.query_row("SELECT COUNT(*) FROM stations", [], |row| row.get(0))?;
    if count == 0 {
        let stations = vec![
            ("Groove Salad (SomaFM)", "http://ice1.somafm.com/groovesalad-128-mp3"),
            ("Secret Agent (SomaFM)", "http://ice4.somafm.com/secretagent-128-mp3"),
            ("BBC Radio 1", "http://icecast.omroep.nl/radio1-bb-mp3"),
        ];
        for (name, url) in stations {
            conn.execute(
                "INSERT INTO stations (name, url) VALUES (?1, ?2)",
                params![name, url],
            )?;
        }
    }
    Ok(())
}

fn load_stations(conn: &Connection) -> Result<Vec<Station>, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT id, name, url, favorite FROM stations")?;
    let station_iter = stmt.query_map([], |row| {
        Ok(Station {
            id: row.get(0)?,
            name: row.get(1)?,
            url: row.get(2)?,
            favorite: row.get::<_, i32>(3)? != 0,
        })
    })?;
    let mut stations = Vec::new();
    for station in station_iter {
        stations.push(station?);
    }
    Ok(stations)
}

fn ui(f: &mut Frame, stations: &[Station], list_state: &mut ListState, visualizer: &AudioVisualizer) {
    let size = f.size();
    
    // First split into main area and help area
    let main_help_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(85), Constraint::Percentage(15)].as_ref())
        .split(size);
        
    // Split main area into stations list (35%) and visualization (65%)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
        .split(main_help_chunks[0]);
    
    // Render stations list
    let items: Vec<ListItem> = stations
        .iter()
        .map(|s| {
            let mut content = s.name.clone();
            if s.favorite {
                content = format!("★ {}", content);
            }
            ListItem::new(Span::styled(
                content,
                Style::default().fg(Color::Cyan),
            ))
        })
        .collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Stations"))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, main_chunks[0], list_state);
    
    // Get status from the visualizer state
    let (status_text, state) = if let Ok(state) = visualizer.state.lock() {
        let status = if state.is_playing { "Playing" } else { "Paused" };
        (status, state.clone())
    } else {
        ("Locked", AudioState::new()) // Fallback if we can't acquire the lock
    };
    
    let vis_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Visualization - {}", status_text));
    
    // Split visualization area into visualization and metadata
    let vis_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(main_chunks[1]);

    // Create visualization with bars
    let mut bar_data: Vec<(&str, u64)> = Vec::new();
    for &value in &state.bars {
        bar_data.push(("", value));
    }
    
    let chart = BarChart::default()
        .block(vis_block)
        .bar_width(4)
        .bar_gap(1)
        .bar_style(Style::default().fg(Color::Green))
        .value_style(Style::default().fg(Color::Black).bg(Color::Green))
        .data(&bar_data)
        .max(25);
    
    f.render_widget(chart, vis_chunks[0]);
    
    // Display stream metadata
    let metadata_text = if let Some(info) = &state.stream_info {
        let unknown = "Unknown".to_string();
        let song = info.current_song.as_ref().unwrap_or(&unknown);
        format!(
            "Station: {}\nFormat: {}\nBitrate: {}\nCurrent Song: {}", 
            info.station_name, info.format, info.bitrate, song
        )
    } else {
        "No stream playing".to_string()
    };
    
    let metadata = Paragraph::new(metadata_text)
        .block(Block::default().borders(Borders::ALL).title("Stream Info"));
    
    f.render_widget(metadata, vis_chunks[1]);
    
    // Render help area
    let help = Paragraph::new("↑/↓: Navigate  ⏎: Play  s: Stop  f: Favorite  q: Quit")
        .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help, main_help_chunks[1]);
}

fn main() -> Result<(), Box<dyn Error>> {
    // Print the current working directory for debugging
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("unknown"));
    println!("Current directory: {:?}", current_dir);
    
    let conn = Connection::open("stations.db")?;
    init_db(&conn)?;
    let mut stations = load_stations(&conn)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut list_state = ListState::default();
    if !stations.is_empty() {
        list_state.select(Some(0));
    }
    
    // Track the currently playing station process
    let mut current_player: Option<Child> = None;
    
    // Initialize audio visualizer
    let visualizer = AudioVisualizer::new();

    loop {
        // Update visualization
        visualizer.update();
        
        terminal.draw(|f| ui(f, &stations, &mut list_state, &visualizer))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        // Kill any playing processes before exiting
                        if let Some(mut player) = current_player.take() {
                            let _ = player.kill();
                            visualizer.set_playing(false);
                        }
                        break;
                    },
                    KeyCode::Down => {
                        if let Some(i) = list_state.selected() {
                            let next = if i + 1 < stations.len() { i + 1 } else { i };
                            list_state.select(Some(next));
                        }
                    }
                    KeyCode::Up => {
                        if let Some(i) = list_state.selected() {
                            let prev = if i > 0 { i - 1 } else { 0 };
                            list_state.select(Some(prev));
                        }
                    }
                    KeyCode::Char('f') => {
                        if let Some(i) = list_state.selected() {
                            let station = &stations[i];
                            let new_fav = !station.favorite;
                            conn.execute(
                                "UPDATE stations SET favorite = ?1 WHERE id = ?2",
                                params![new_fav as i32, station.id],
                            )?;
                            stations[i].favorite = new_fav;
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(i) = list_state.selected() {
                            // Kill any currently playing process
                            if let Some(mut player) = current_player.take() {
                                let _ = player.kill();
                                visualizer.set_playing(false);
                            }
                            
                            // Start new player process
                            let url = stations[i].url.clone();
                            let station_name = stations[i].name.clone();
                            
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
                                        station_name, 
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
                                    
                                    current_player = Some(child);
                                },
                                Err(e) => {
                                    eprintln!("Failed to start player: {} (make sure mpv is installed)", e);
                                    visualizer.set_stream_info(
                                        station_name,
                                        "Error".to_string(),
                                        format!("Failed to start: {}", e)
                                    );
                                },
                            }
                        }
                    },
                    KeyCode::Char('s') => {
                        // Stop the current player
                        if let Some(mut player) = current_player.take() {
                            let _ = player.kill();
                            visualizer.set_playing(false);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Clean up any running player process
    if let Some(mut player) = current_player {
        let _ = player.kill();
        visualizer.set_playing(false);
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}