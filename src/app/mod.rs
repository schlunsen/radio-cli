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

// Add an enum for app modes
#[derive(PartialEq)]
pub enum AppMode {
    Normal,
    AddingStation,
}

pub struct App {
    pub terminal: Terminal<CrosstermBackend<io::Stdout>>,
    pub stations: Vec<Station>,
    pub list_state: ListState,
    pub visualizer: AudioVisualizer,
    pub player: Player,
    pub conn: Connection,
    pub mode: AppMode,
    pub add_station_name: String,
    pub add_station_url: String,
    pub add_station_desc: String,
    pub input_cursor: usize,
    pub input_field: usize, // 0 = name, 1 = url, 2 = description
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
            mode: AppMode::Normal,
            add_station_name: String::new(),
            add_station_url: String::new(),
            add_station_desc: String::new(),
            input_cursor: 0,
            input_field: 0,
        })
    }

    // Add a method to save the station
    pub fn save_new_station(&mut self) -> Result<(), Box<dyn Error>> {
        // Validate inputs
        if self.add_station_name.trim().is_empty() || self.add_station_url.trim().is_empty() {
            return Ok(()); // Silently ignore empty inputs
        }
        
        // Add to database
        let desc = if self.add_station_desc.is_empty() { None } else { Some(&self.add_station_desc[..]) };
        let id = crate::db::add_station(&self.conn, &self.add_station_name, &self.add_station_url, desc)?;
        
        // Create new station object
        let new_station = Station {
            id,
            name: self.add_station_name.clone(),
            url: self.add_station_url.clone(),
            favorite: false,
            description: if self.add_station_desc.is_empty() { None } else { Some(self.add_station_desc.clone()) },
        };
        
        // Add to the list
        self.stations.push(new_station);
        
        // Reset input fields
        self.add_station_name.clear();
        self.add_station_url.clear();
        self.add_station_desc.clear();
        self.input_cursor = 0;
        self.input_field = 0;
        
        // Return to normal mode
        self.mode = AppMode::Normal;
        
        Ok(())
    }
    
    // Handle input for the add station form
    pub fn handle_add_station_input(&mut self, key: KeyCode) -> Result<(), Box<dyn Error>> {
        match key {
            KeyCode::Esc => {
                // Cancel adding station
                self.mode = AppMode::Normal;
                self.add_station_name.clear();
                self.add_station_url.clear();
                self.add_station_desc.clear();
            },
            KeyCode::Enter => {
                // Move to next field or save
                match self.input_field {
                    0 => {
                        self.input_field = 1;
                        self.input_cursor = 0;
                    },
                    1 => {
                        self.input_field = 2;
                        self.input_cursor = 0;
                    },
                    2 => {
                        self.save_new_station()?;
                    },
                    _ => unreachable!(),
                }
            },
            KeyCode::Tab => {
                // Move to next field
                self.input_field = (self.input_field + 1) % 3;
                self.input_cursor = match self.input_field {
                    0 => self.add_station_name.len(),
                    1 => self.add_station_url.len(),
                    2 => self.add_station_desc.len(),
                    _ => unreachable!(),
                };
            },
            KeyCode::BackTab => {
                // Move to previous field
                self.input_field = if self.input_field == 0 { 2 } else { self.input_field - 1 };
                self.input_cursor = match self.input_field {
                    0 => self.add_station_name.len(),
                    1 => self.add_station_url.len(),
                    2 => self.add_station_desc.len(),
                    _ => unreachable!(),
                };
            },
            KeyCode::Backspace => {
                // Delete character before cursor
                if self.input_cursor > 0 {
                    match self.input_field {
                        0 => {
                            self.add_station_name.remove(self.input_cursor - 1);
                            self.input_cursor -= 1;
                        },
                        1 => {
                            self.add_station_url.remove(self.input_cursor - 1);
                            self.input_cursor -= 1;
                        },
                        2 => {
                            self.add_station_desc.remove(self.input_cursor - 1);
                            self.input_cursor -= 1;
                        },
                        _ => unreachable!(),
                    }
                }
            },
            KeyCode::Delete => {
                // Delete character at cursor
                match self.input_field {
                    0 => {
                        if self.input_cursor < self.add_station_name.len() {
                            self.add_station_name.remove(self.input_cursor);
                        }
                    },
                    1 => {
                        if self.input_cursor < self.add_station_url.len() {
                            self.add_station_url.remove(self.input_cursor);
                        }
                    },
                    2 => {
                        if self.input_cursor < self.add_station_desc.len() {
                            self.add_station_desc.remove(self.input_cursor);
                        }
                    },
                    _ => unreachable!(),
                }
            },
            KeyCode::Left => {
                // Move cursor left
                if self.input_cursor > 0 {
                    self.input_cursor -= 1;
                }
            },
            KeyCode::Right => {
                // Move cursor right
                let max_cursor = match self.input_field {
                    0 => self.add_station_name.len(),
                    1 => self.add_station_url.len(),
                    2 => self.add_station_desc.len(),
                    _ => unreachable!(),
                };
                if self.input_cursor < max_cursor {
                    self.input_cursor += 1;
                }
            },
            KeyCode::Char(c) => {
                // Add character at cursor
                match self.input_field {
                    0 => {
                        self.add_station_name.insert(self.input_cursor, c);
                        self.input_cursor += 1;
                    },
                    1 => {
                        self.add_station_url.insert(self.input_cursor, c);
                        self.input_cursor += 1;
                    },
                    2 => {
                        self.add_station_desc.insert(self.input_cursor, c);
                        self.input_cursor += 1;
                    },
                    _ => unreachable!(),
                }
            },
            _ => {},
        }
        
        Ok(())
    }
    
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            // Update visualization
            self.visualizer.update();
            
            // Draw UI - pass individual fields to avoid borrow checker issues
            self.terminal.draw(|f| {
                ui::ui(
                    f, 
                    &self.stations, 
                    &mut self.list_state, 
                    &self.visualizer,
                    &self.mode,
                    &self.add_station_name,
                    &self.add_station_url,
                    &self.add_station_desc,
                    self.input_field,
                    self.input_cursor,
                )
            })?;

            // Handle events
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match self.mode {
                        AppMode::Normal => match key.code {
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
                            },
                            KeyCode::Char('a') => {
                                // Switch to add station mode
                                self.mode = AppMode::AddingStation;
                                self.add_station_name.clear();
                                self.add_station_url.clear();
                                self.add_station_desc.clear();
                                self.input_cursor = 0;
                                self.input_field = 0;
                            },
                            _ => {}
                        },
                        AppMode::AddingStation => {
                            self.handle_add_station_input(key.code)?;
                        },
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