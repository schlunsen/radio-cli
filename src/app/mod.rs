use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use crate::audio::{AudioVisualizer, Player};
use crate::db::{toggle_favorite, Station};
use crate::ui;
use crate::visualizations::VisualizationManager;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use lazy_static::lazy_static;
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};
use rusqlite::Connection;

// Global application state for UI components to access
lazy_static! {
    pub static ref APP_STATE: Mutex<Option<AppState>> = Mutex::new(None);
}

// A simplified version of App for UI access
pub struct AppState {
    pub edit_station_name: String,
    pub edit_station_url: String,
    pub edit_station_desc: String,
}

// Add an enum for app modes
#[derive(PartialEq)]
pub enum AppMode {
    Normal,
    AddingStation,
    EditingStation,
    VisualizationMenu,
    DeletingStation,
    RcastStations,
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
    pub vis_manager: VisualizationManager,
    pub vis_menu_state: ListState, // State for visualization menu selection
    pub edit_station_id: i32,      // ID of the station being edited
    pub edit_station_name: String,
    pub edit_station_url: String,
    pub edit_station_desc: String,
    pub confirm_delete: bool, // Whether the user has confirmed deletion
    pub rcast_stations: Vec<crate::rcast::RcastStation>, // List of stations from RCast.net
    pub rcast_list_state: ListState, // State for RCast stations list
    pub rcast_loading: bool,  // Whether we're currently loading RCast stations
}

impl App {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // Get the database path
        let db_path = get_database_path()?;

        // Ensure the directory exists
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Set up database
        let conn = Connection::open(&db_path)?;
        crate::db::init_db(&conn)?;
        let stations = crate::db::load_stations(&conn)?;

        // Set up terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Create app state
        let mut list_state = ListState::default();
        if !stations.is_empty() {
            list_state.select(Some(0)); // Start with the first station selected
        }

        // Create visualization and player components
        let visualizer = AudioVisualizer::new();
        let player = Player::new();
        let vis_manager = VisualizationManager::new();

        // Create visualization menu state
        let mut vis_menu_state = ListState::default();
        vis_menu_state.select(Some(0)); // Select first visualization by default

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
            vis_manager,
            vis_menu_state,
            edit_station_id: 0,
            edit_station_name: String::new(),
            edit_station_url: String::new(),
            edit_station_desc: String::new(),
            confirm_delete: false,
            rcast_stations: Vec::new(),
            rcast_list_state: ListState::default(),
            rcast_loading: false,
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // Update global app state for UI components
        {
            let mut app_state = APP_STATE.lock().unwrap();
            *app_state = Some(AppState {
                edit_station_name: self.edit_station_name.clone(),
                edit_station_url: self.edit_station_url.clone(),
                edit_station_desc: self.edit_station_desc.clone(),
            });
        }

        // Main event loop
        loop {
            // Update global app state with latest values
            {
                if let Ok(mut app_state) = APP_STATE.lock() {
                    if let Some(state) = app_state.as_mut() {
                        state.edit_station_name = self.edit_station_name.clone();
                        state.edit_station_url = self.edit_station_url.clone();
                        state.edit_station_desc = self.edit_station_desc.clone();
                    }
                }
            }

            // Draw the UI
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
                    &self.vis_manager,
                    &mut self.vis_menu_state,
                    &self.rcast_stations,
                    &mut self.rcast_list_state,
                    self.rcast_loading,
                )
            })?;

            // Update the visualization
            self.visualizer.update();

            // Handle input
            if crossterm::event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    match self.mode {
                        AppMode::Normal => {
                            if self.handle_normal_mode(key)? {
                                break; // User requested exit
                            }
                        }
                        AppMode::AddingStation => {
                            self.handle_adding_mode(key)?;
                        }
                        AppMode::EditingStation => {
                            self.handle_editing_mode(key)?;
                        }
                        AppMode::DeletingStation => {
                            self.handle_deleting_mode(key)?;
                        }
                        AppMode::VisualizationMenu => {
                            self.handle_vis_menu_mode(key)?;
                        }
                        AppMode::RcastStations => {
                            if self.handle_rcast_stations_mode(key)? {
                                break; // User requested exit
                            }
                        }
                    }
                }
            }
        }

        // Clean up
        self.player.stop();
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;

        Ok(())
    }

    fn handle_normal_mode(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<bool, Box<dyn Error>> {
        match key.code {
            KeyCode::Char('q') => {
                return Ok(true); // Signal to exit the program
            }
            KeyCode::Tab => {
                // Toggle to RcastStations mode
                self.mode = AppMode::RcastStations;
                // Initialize rcast stations list if empty
                if self.rcast_stations.is_empty() {
                    // Only refresh if there are no stations
                    self.refresh_rcast_stations()?;
                }
                // Ensure a station is selected in the list
                if !self.rcast_stations.is_empty() && self.rcast_list_state.selected().is_none() {
                    self.rcast_list_state.select(Some(0));
                }
            }
            KeyCode::Char('a') => {
                self.mode = AppMode::AddingStation;
                self.add_station_name.clear();
                self.add_station_url.clear();
                self.add_station_desc.clear();
                self.input_cursor = 0;
                self.input_field = 0;
            }
            KeyCode::Char('e') => {
                // Edit selected station
                if let Some(i) = self.list_state.selected() {
                    if i < self.stations.len() {
                        let station = &self.stations[i];
                        self.mode = AppMode::EditingStation;
                        self.edit_station_id = station.id;
                        self.edit_station_name = station.name.clone();
                        self.edit_station_url = station.url.clone();
                        self.edit_station_desc = station.description.clone().unwrap_or_default();
                        self.input_cursor = 0;
                        self.input_field = 0;
                    }
                }
            }
            KeyCode::Char('d') => {
                // Delete selected station
                if let Some(i) = self.list_state.selected() {
                    if i < self.stations.len() {
                        self.mode = AppMode::DeletingStation;
                        self.confirm_delete = false;
                    }
                }
            }
            KeyCode::Char('v') => {
                self.mode = AppMode::VisualizationMenu;

                // Select the current visualization in the menu
                let current_vis_type = self.vis_manager.current_type();
                let visualizations = self.vis_manager.get_available_visualizations();

                // Find the index of the current visualization
                for (i, (vis_type, _, _)) in visualizations.iter().enumerate() {
                    if *vis_type == current_vis_type {
                        self.vis_menu_state.select(Some(i));
                        break;
                    }
                }
            }
            KeyCode::Down => {
                if !self.stations.is_empty() {
                    let i = match self.list_state.selected() {
                        Some(i) => {
                            if i >= self.stations.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(i));
                }
            }
            KeyCode::Up => {
                if !self.stations.is_empty() {
                    let i = match self.list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                self.stations.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(i));
                }
            }
            KeyCode::Enter => {
                if let Some(i) = self.list_state.selected() {
                    if i < self.stations.len() {
                        let station = &self.stations[i];
                        self.player.play_station(
                            station.name.clone(),
                            station.url.clone(),
                            &self.visualizer,
                        )?;
                    }
                }
            }
            KeyCode::Char('s') => {
                self.player.stop();
                self.visualizer.set_playing(false);
            }
            KeyCode::Char('f') => {
                if let Some(i) = self.list_state.selected() {
                    if i < self.stations.len() {
                        let station = &self.stations[i];
                        let new_favorite = !station.favorite;
                        toggle_favorite(&self.conn, station.id, new_favorite)?;
                        // Update the local stations list
                        self.stations = crate::db::load_stations(&self.conn)?;
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_vis_menu_mode(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<(), Box<dyn Error>> {
        let visualizations = self.vis_manager.get_available_visualizations();

        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
            }
            KeyCode::Enter => {
                if let Some(selected) = self.vis_menu_state.selected() {
                    // Apply the selected visualization
                    if selected < visualizations.len() {
                        let (vis_type, _, _) = visualizations[selected];
                        self.vis_manager.set_visualization_type(vis_type);
                    }
                }
                self.mode = AppMode::Normal;
            }
            KeyCode::Down => {
                if !visualizations.is_empty() {
                    let i = match self.vis_menu_state.selected() {
                        Some(i) => {
                            if i >= visualizations.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    self.vis_menu_state.select(Some(i));
                }
            }
            KeyCode::Up => {
                if !visualizations.is_empty() {
                    let i = match self.vis_menu_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                visualizations.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.vis_menu_state.select(Some(i));
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_adding_mode(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<(), Box<dyn Error>> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
            }
            KeyCode::Tab => {
                // Cycle through fields
                self.input_field = (self.input_field + 1) % 3;
                // Adjust cursor position
                match self.input_field {
                    0 => self.input_cursor = self.add_station_name.len(),
                    1 => self.input_cursor = self.add_station_url.len(),
                    2 => self.input_cursor = self.add_station_desc.len(),
                    _ => {}
                }
            }
            KeyCode::Enter => {
                // Submit form if URL and name are not empty
                if !self.add_station_name.is_empty() && !self.add_station_url.is_empty() {
                    let desc = if self.add_station_desc.is_empty() {
                        None
                    } else {
                        Some(self.add_station_desc.as_str())
                    };

                    crate::db::add_station(
                        &self.conn,
                        &self.add_station_name,
                        &self.add_station_url,
                        desc,
                    )?;

                    // Reload stations and return to normal mode
                    self.stations = crate::db::load_stations(&self.conn)?;
                    self.mode = AppMode::Normal;
                }
            }
            KeyCode::Char(c) => {
                // Add character to current field
                match self.input_field {
                    0 => {
                        if self.input_cursor < self.add_station_name.len() {
                            self.add_station_name.insert(self.input_cursor, c);
                        } else {
                            self.add_station_name.push(c);
                        }
                        self.input_cursor += 1;
                    }
                    1 => {
                        if self.input_cursor < self.add_station_url.len() {
                            self.add_station_url.insert(self.input_cursor, c);
                        } else {
                            self.add_station_url.push(c);
                        }
                        self.input_cursor += 1;
                    }
                    2 => {
                        if self.input_cursor < self.add_station_desc.len() {
                            self.add_station_desc.insert(self.input_cursor, c);
                        } else {
                            self.add_station_desc.push(c);
                        }
                        self.input_cursor += 1;
                    }
                    _ => {}
                }
            }
            KeyCode::Backspace => {
                // Remove character from current field
                match self.input_field {
                    0 => {
                        if self.input_cursor > 0 {
                            self.add_station_name.remove(self.input_cursor - 1);
                            self.input_cursor -= 1;
                        }
                    }
                    1 => {
                        if self.input_cursor > 0 {
                            self.add_station_url.remove(self.input_cursor - 1);
                            self.input_cursor -= 1;
                        }
                    }
                    2 => {
                        if self.input_cursor > 0 {
                            self.add_station_desc.remove(self.input_cursor - 1);
                            self.input_cursor -= 1;
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Left => {
                if self.input_cursor > 0 {
                    self.input_cursor -= 1;
                }
            }
            KeyCode::Right => {
                let max_cursor = match self.input_field {
                    0 => self.add_station_name.len(),
                    1 => self.add_station_url.len(),
                    2 => self.add_station_desc.len(),
                    _ => 0,
                };
                if self.input_cursor < max_cursor {
                    self.input_cursor += 1;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_deleting_mode(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<(), Box<dyn Error>> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
            }
            KeyCode::Char('y') => {
                if let Some(i) = self.list_state.selected() {
                    if i < self.stations.len() {
                        // Store the station ID to delete
                        let station_id = self.stations[i].id;

                        // Delete the station from the database
                        crate::db::delete_station(&self.conn, station_id)?;

                        // Reload stations and return to normal mode
                        self.stations = crate::db::load_stations(&self.conn)?;
                        self.mode = AppMode::Normal;

                        // If the deleted station was the last one, select the previous one
                        if !self.stations.is_empty() {
                            if i >= self.stations.len() {
                                self.list_state.select(Some(self.stations.len() - 1));
                            }
                        } else {
                            self.list_state.select(None);
                        }
                    }
                }
            }
            KeyCode::Char('n') => {
                self.mode = AppMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_editing_mode(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<(), Box<dyn Error>> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
            }
            KeyCode::Tab => {
                // Cycle through fields
                self.input_field = (self.input_field + 1) % 3;
                // Adjust cursor position
                match self.input_field {
                    0 => self.input_cursor = self.edit_station_name.len(),
                    1 => self.input_cursor = self.edit_station_url.len(),
                    2 => self.input_cursor = self.edit_station_desc.len(),
                    _ => {}
                }
            }
            KeyCode::Enter => {
                // Submit form if URL and name are not empty
                if !self.edit_station_name.is_empty() && !self.edit_station_url.is_empty() {
                    let desc = if self.edit_station_desc.is_empty() {
                        None
                    } else {
                        Some(self.edit_station_desc.as_str())
                    };

                    crate::db::update_station(
                        &self.conn,
                        self.edit_station_id,
                        &self.edit_station_name,
                        &self.edit_station_url,
                        desc,
                    )?;

                    // Reload stations and return to normal mode
                    self.stations = crate::db::load_stations(&self.conn)?;
                    self.mode = AppMode::Normal;
                }
            }
            KeyCode::Char(c) => {
                // Add character to current field
                match self.input_field {
                    0 => {
                        if self.input_cursor < self.edit_station_name.len() {
                            self.edit_station_name.insert(self.input_cursor, c);
                        } else {
                            self.edit_station_name.push(c);
                        }
                        self.input_cursor += 1;
                    }
                    1 => {
                        if self.input_cursor < self.edit_station_url.len() {
                            self.edit_station_url.insert(self.input_cursor, c);
                        } else {
                            self.edit_station_url.push(c);
                        }
                        self.input_cursor += 1;
                    }
                    2 => {
                        if self.input_cursor < self.edit_station_desc.len() {
                            self.edit_station_desc.insert(self.input_cursor, c);
                        } else {
                            self.edit_station_desc.push(c);
                        }
                        self.input_cursor += 1;
                    }
                    _ => {}
                }
            }
            KeyCode::Backspace => {
                // Remove character from current field
                match self.input_field {
                    0 => {
                        if self.input_cursor > 0 {
                            self.edit_station_name.remove(self.input_cursor - 1);
                            self.input_cursor -= 1;
                        }
                    }
                    1 => {
                        if self.input_cursor > 0 {
                            self.edit_station_url.remove(self.input_cursor - 1);
                            self.input_cursor -= 1;
                        }
                    }
                    2 => {
                        if self.input_cursor > 0 {
                            self.edit_station_desc.remove(self.input_cursor - 1);
                            self.input_cursor -= 1;
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Left => {
                if self.input_cursor > 0 {
                    self.input_cursor -= 1;
                }
            }
            KeyCode::Right => {
                let max_cursor = match self.input_field {
                    0 => self.edit_station_name.len(),
                    1 => self.edit_station_url.len(),
                    2 => self.edit_station_desc.len(),
                    _ => 0,
                };
                if self.input_cursor < max_cursor {
                    self.input_cursor += 1;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_rcast_stations_mode(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<bool, Box<dyn Error>> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                // Ensure a station is selected in the normal list
                if !self.stations.is_empty() && self.list_state.selected().is_none() {
                    self.list_state.select(Some(0));
                }
            }
            KeyCode::Tab => {
                // Toggle back to normal mode
                self.mode = AppMode::Normal;
                // Ensure a station is selected in the normal list
                if !self.stations.is_empty() && self.list_state.selected().is_none() {
                    self.list_state.select(Some(0));
                }
            }
            KeyCode::Down => {
                if !self.rcast_stations.is_empty() {
                    let i = match self.rcast_list_state.selected() {
                        Some(i) => {
                            if i >= self.rcast_stations.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    self.rcast_list_state.select(Some(i));
                }
            }
            KeyCode::Up => {
                if !self.rcast_stations.is_empty() {
                    let i = match self.rcast_list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                self.rcast_stations.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.rcast_list_state.select(Some(i));
                }
            }
            KeyCode::Enter => {
                if let Some(i) = self.rcast_list_state.selected() {
                    if i < self.rcast_stations.len() {
                        let station = &self.rcast_stations[i];
                        self.player.play_station(
                            station.name.clone(),
                            station.url.clone(),
                            &self.visualizer,
                        )?;
                    }
                }
            }
            KeyCode::Char('r') => {
                // Refresh the station list
                self.refresh_rcast_stations()?;
            }
            KeyCode::Char('a') => {
                // Add current station to saved stations
                if let Some(i) = self.rcast_list_state.selected() {
                    if i < self.rcast_stations.len() {
                        let station = &self.rcast_stations[i];
                        crate::db::add_station(
                            &self.conn,
                            &station.name,
                            &station.url,
                            station.description.as_deref(),
                        )?;

                        // Reload stations
                        self.stations = crate::db::load_stations(&self.conn)?;
                    }
                }
            }
            KeyCode::Char('q') => {
                return Ok(true);
            }
            _ => {}
        }

        Ok(false)
    }

    // Function to refresh the RCast stations list
    fn refresh_rcast_stations(&mut self) -> Result<(), Box<dyn Error>> {
        // Set the loading flag and clear current stations
        self.rcast_loading = true;
        self.rcast_stations.clear();

        // Create a new runtime for async operations
        match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                // Block on the async fetch operation
                match rt.block_on(crate::rcast::fetch_stations()) {
                    Ok(stations) => {
                        // Update stations with fetched data
                        self.rcast_stations = stations;

                        // If no stations fetched, add a message station
                        if self.rcast_stations.is_empty() {
                            self.rcast_stations.push(crate::rcast::RcastStation {
                                name: "No stations found".to_string(),
                                url: "".to_string(),
                                description: Some("Try refreshing the list with 'r'".to_string()),
                                bitrate: None,
                                genre: None,
                                listeners: None,
                            });
                        }
                    }
                    Err(e) => {
                        // Add an error message station
                        self.rcast_stations.push(crate::rcast::RcastStation {
                            name: "Error fetching stations".to_string(),
                            url: "".to_string(),
                            description: Some(format!("Error: {}. Try refreshing with 'r'", e)),
                            bitrate: None,
                            genre: None,
                            listeners: None,
                        });
                    }
                }
            }
            Err(e) => {
                // Add an error message station
                self.rcast_stations.push(crate::rcast::RcastStation {
                    name: "Error initializing fetcher".to_string(),
                    url: "".to_string(),
                    description: Some(format!("Runtime error: {}. Try refreshing with 'r'", e)),
                    bitrate: None,
                    genre: None,
                    listeners: None,
                });
            }
        }

        // Select the first station if available
        if !self.rcast_stations.is_empty() {
            self.rcast_list_state.select(Some(0));
        } else {
            self.rcast_list_state.select(None);
        }

        // Reset loading flag
        self.rcast_loading = false;
        Ok(())
    }
}

// Function to get the database path
pub fn get_database_path() -> Result<PathBuf, Box<dyn Error>> {
    // First, check if stations.db exists in the current directory
    let local_db = PathBuf::from("stations.db");
    if local_db.exists() {
        return Ok(local_db);
    }

    // Next, check if we have an XDG_DATA_HOME environment variable
    let data_dir = match std::env::var_os("XDG_DATA_HOME") {
        Some(dir) => {
            let mut path = PathBuf::from(dir);
            path.push("radio_cli");
            path
        }
        None => {
            // If not, use the platform-specific data directory
            #[cfg(target_os = "macos")]
            {
                let mut path = dirs_next::home_dir().ok_or("Could not find home directory")?;
                path.push("Library");
                path.push("Application Support");
                path.push("radio_cli");
                path
            }
            #[cfg(target_os = "linux")]
            {
                let mut path = dirs_next::home_dir().ok_or("Could not find home directory")?;
                path.push(".local");
                path.push("share");
                path.push("radio_cli");
                path
            }
            #[cfg(target_os = "windows")]
            {
                let mut path = dirs_next::data_dir().ok_or("Could not find data directory")?;
                path.push("radio_cli");
                path
            }
            #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
            {
                let mut path = dirs_next::home_dir().ok_or("Could not find home directory")?;
                path.push(".radio_cli");
                path
            }
        }
    };

    // Create the directory if it doesn't exist
    fs::create_dir_all(&data_dir)?;

    // Return the path to the database file
    let db_path = data_dir.join("stations.db");
    Ok(db_path)
}
