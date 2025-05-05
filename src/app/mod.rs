use std::error::Error;
use std::io;
use std::time::Duration;

use crate::audio::{AudioVisualizer, Player};
use crate::db::{Station, toggle_favorite};
use crate::ui;

use crossterm::{
    event::{self, Event, KeyCode, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};
use rusqlite::Connection;

pub struct App {
    pub terminal: Terminal<CrosstermBackend<io::Stdout>>,
    pub stations: Vec<Station>,
    pub list_state: ListState,
    pub visualizer: AudioVisualizer,
    pub player: Player,
    pub conn: Connection,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // Set up database
        let conn = Connection::open("stations.db")?;
        crate::db::init_db(&conn)?;
        let stations = crate::db::load_stations(&conn)?;

        // Set up terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Initialize list state
        let mut list_state = ListState::default();
        if !stations.is_empty() {
            list_state.select(Some(0));
        }
        
        // Initialize audio components
        let visualizer = AudioVisualizer::new();
        let player = Player::new();

        Ok(App {
            terminal,
            stations,
            list_state,
            visualizer,
            player,
            conn,
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            // Update visualization
            self.visualizer.update();
            
            // Draw UI
            self.terminal.draw(|f| ui::ui(f, &self.stations, &mut self.list_state, &self.visualizer))?;

            // Handle events
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => {
                            // Quit application
                            break;
                        },
                        KeyCode::Down => {
                            // Navigate down
                            if let Some(i) = self.list_state.selected() {
                                let next = if i + 1 < self.stations.len() { i + 1 } else { i };
                                self.list_state.select(Some(next));
                            }
                        }
                        KeyCode::Up => {
                            // Navigate up
                            if let Some(i) = self.list_state.selected() {
                                let prev = if i > 0 { i - 1 } else { 0 };
                                self.list_state.select(Some(prev));
                            }
                        }
                        KeyCode::Char('f') => {
                            // Toggle favorite
                            if let Some(i) = self.list_state.selected() {
                                let station = &self.stations[i];
                                let new_fav = !station.favorite;
                                toggle_favorite(&self.conn, station.id, new_fav)?;
                                self.stations[i].favorite = new_fav;
                            }
                        }
                        KeyCode::Enter => {
                            // Play selected station
                            if let Some(i) = self.list_state.selected() {
                                let station = &self.stations[i];
                                let _ = self.player.play_station(
                                    station.name.clone(), 
                                    station.url.clone(), 
                                    &self.visualizer
                                );
                            }
                        },
                        KeyCode::Char('s') => {
                            // Stop playback
                            self.player.stop();
                            self.visualizer.set_playing(false);
                        }
                        _ => {}
                    }
                }
            }
        }

        self.cleanup()?;
        Ok(())
    }

    pub fn cleanup(&mut self) -> Result<(), Box<dyn Error>> {
        // Stop any playing audio
        self.player.stop();
        self.visualizer.set_playing(false);

        // Restore terminal
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
        self.terminal.show_cursor()?;
        
        Ok(())
    }
}