use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::audio::{AudioVisualizer, Player};
use crate::db::{toggle_favorite, update_station_stats, Station};
use crate::ui;
use crate::visualizations::VisualizationManager;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
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
    EditingStation,
    VisualizationMenu,
    DeletingStation,
    RcastStations,
    RadioBrowser,
    Searching,
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
    pub rb_stations: Vec<crate::radiobrowser::RadioBrowserStation>,
    pub rb_list_state: ListState,
    pub rb_loading: bool,
    pub rb_filter: crate::radiobrowser::RadioBrowserFilter,
    pub rb_filter_input: String,
    pub rb_entering_filter: bool,
    pub stats_last_update: Instant, // Last time stats were updated
    pub metadata_last_update: Instant, // Last time metadata was updated
    pub current_station_id: Option<i32>, // Currently playing station ID
    pub show_top_stations: bool, // Whether to show top stations in Stream info
    pub search_query: String, // Current search query
    pub search_results: Vec<Station>, // Filtered search results
    pub search_list_state: ListState, // State for search results list pane
    pub show_visualizations: bool, // Whether to show visualizations (false = show stats instead)
    pub dirty: bool,          // Whether the UI needs redrawing
    pub tokio_runtime: Option<tokio::runtime::Runtime>, // Reusable tokio runtime
}

impl App {
    pub fn new(show_visualizations: bool) -> Result<Self, Box<dyn Error>> {
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

        // Create a reusable tokio runtime
        let tokio_runtime = tokio::runtime::Runtime::new().ok();

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
            rb_stations: Vec::new(),
            rb_list_state: ListState::default(),
            rb_loading: false,
            rb_filter: crate::radiobrowser::RadioBrowserFilter::default(),
            rb_filter_input: String::new(),
            rb_entering_filter: false,
            stats_last_update: Instant::now(),
            metadata_last_update: Instant::now(),
            current_station_id: None,
            show_top_stations: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_list_state: ListState::default(),
            show_visualizations,
            dirty: true,
            tokio_runtime,
        })
    }

    // Helper method to update station stats
    fn update_station_stats(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(station_id) = self.current_station_id {
            // Update stats for the current station (add 10 seconds of play time)
            update_station_stats(&self.conn, station_id, 10)?;
        }
        self.stats_last_update = Instant::now();
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // Main event loop
        loop {
            // Check if we need to update stats (every 10 seconds)
            if self.current_station_id.is_some()
                && self.stats_last_update.elapsed() >= Duration::from_secs(10)
            {
                if let Err(e) = self.update_station_stats() {
                    eprintln!("Failed to update station stats: {}", e);
                }
            }

            // We don't need to explicitly update metadata as it's handled by
            // the background thread in the player. Leaving this timer for potential
            // future use or other periodic tasks.
            if self.metadata_last_update.elapsed() >= Duration::from_secs(1) {
                self.metadata_last_update = Instant::now();
            }

            // Only redraw when dirty or when visualizations are active (they animate)
            let needs_draw = self.dirty || self.show_visualizations;

            if needs_draw {
                // Capture edit fields for the UI before drawing
                let edit_name = self.edit_station_name.clone();
                let edit_url = self.edit_station_url.clone();
                let edit_desc = self.edit_station_desc.clone();

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
                        &self.rb_stations,
                        &mut self.rb_list_state,
                        self.rb_loading,
                        self.rb_entering_filter,
                        &self.rb_filter_input,
                        self.show_top_stations,
                        &self.conn,
                        self.current_station_id,
                        &self.search_query,
                        &self.search_results,
                        &mut self.search_list_state,
                        self.show_visualizations,
                        &edit_name,
                        &edit_url,
                        &edit_desc,
                    )
                })?;

                self.dirty = false;
            }

            // Update the visualization
            self.visualizer.update();

            // Handle input
            if crossterm::event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    self.dirty = true; // Any key press means we need to redraw
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
                        AppMode::RadioBrowser => {
                            if self.handle_radiobrowser_stations_mode(key)? {
                                break;
                            }
                        }
                        AppMode::Searching => {
                            self.handle_search_mode(key)?;
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

    /// Navigate down in a list with wrapping
    fn navigate_down(list_state: &mut ListState, len: usize) {
        if len == 0 {
            return;
        }
        let i = match list_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        list_state.select(Some(i));
    }

    /// Navigate up in a list with wrapping
    fn navigate_up(list_state: &mut ListState, len: usize) {
        if len == 0 {
            return;
        }
        let i = match list_state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        list_state.select(Some(i));
    }

    /// Navigate by page (jump multiple items)
    fn navigate_page(list_state: &mut ListState, len: usize, forward: bool) {
        if len == 0 {
            return;
        }
        let page_size = 10; // Jump 10 items at a time
        let current = list_state.selected().unwrap_or(0);
        let new_pos = if forward {
            (current + page_size).min(len - 1)
        } else {
            current.saturating_sub(page_size)
        };
        list_state.select(Some(new_pos));
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
            KeyCode::Char('b') => {
                self.mode = AppMode::RadioBrowser;
                if self.rb_stations.is_empty() {
                    self.refresh_radiobrowser_stations()?;
                }
                if !self.rb_stations.is_empty() && self.rb_list_state.selected().is_none() {
                    self.rb_list_state.select(Some(0));
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
            KeyCode::Char('/') => {
                // Enter search mode
                self.mode = AppMode::Searching;
                self.search_query.clear();
                self.search_results.clear();
                self.search_list_state.select(None);
            }
            // Vim-style and enhanced navigation
            KeyCode::Down | KeyCode::Char('j') => {
                let len = self.stations.len();
                Self::navigate_down(&mut self.list_state, len);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let len = self.stations.len();
                Self::navigate_up(&mut self.list_state, len);
            }
            KeyCode::PageDown => {
                let len = self.stations.len();
                Self::navigate_page(&mut self.list_state, len, true);
            }
            KeyCode::PageUp => {
                let len = self.stations.len();
                Self::navigate_page(&mut self.list_state, len, false);
            }
            KeyCode::Home => {
                if !self.stations.is_empty() {
                    self.list_state.select(Some(0));
                }
            }
            KeyCode::End => {
                if !self.stations.is_empty() {
                    self.list_state.select(Some(self.stations.len() - 1));
                }
            }
            KeyCode::Enter => {
                if let Some(i) = self.list_state.selected() {
                    if i < self.stations.len() {
                        // Clone the values to avoid borrowing issues
                        let name = self.stations[i].name.clone();
                        let url = self.stations[i].url.clone();
                        let description = self.stations[i].description.clone();

                        self.play_station(&name, &url, description.as_deref())?;
                    }
                }
            }
            KeyCode::Char('s') => {
                self.player.stop();
                self.visualizer.set_playing(false);
                // Clear current station ID when stopping
                self.current_station_id = None;
            }
            KeyCode::Char('m') => {
                // Toggle mute
                if let Err(e) = self.player.toggle_mute(&self.visualizer) {
                    self.visualizer.set_error(format!("Mute failed: {}", e));
                }
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                // Increase volume
                if let Err(e) = self.player.volume_up(&self.visualizer) {
                    self.visualizer.set_error(format!("Volume failed: {}", e));
                }
            }
            KeyCode::Char('-') => {
                // Decrease volume
                if let Err(e) = self.player.volume_down(&self.visualizer) {
                    self.visualizer.set_error(format!("Volume failed: {}", e));
                }
            }
            KeyCode::Char('t') => {
                // Toggle showing top stations in Stream info
                self.show_top_stations = !self.show_top_stations;
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
            KeyCode::Char('V') => {
                // Toggle visualization mode
                self.show_visualizations = !self.show_visualizations;
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
            KeyCode::Down | KeyCode::Char('j') => {
                let len = visualizations.len();
                Self::navigate_down(&mut self.vis_menu_state, len);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let len = visualizations.len();
                Self::navigate_up(&mut self.vis_menu_state, len);
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
            KeyCode::Down | KeyCode::Char('j') => {
                let len = self.rcast_stations.len();
                Self::navigate_down(&mut self.rcast_list_state, len);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let len = self.rcast_stations.len();
                Self::navigate_up(&mut self.rcast_list_state, len);
            }
            KeyCode::PageDown => {
                let len = self.rcast_stations.len();
                Self::navigate_page(&mut self.rcast_list_state, len, true);
            }
            KeyCode::PageUp => {
                let len = self.rcast_stations.len();
                Self::navigate_page(&mut self.rcast_list_state, len, false);
            }
            KeyCode::Home => {
                if !self.rcast_stations.is_empty() {
                    self.rcast_list_state.select(Some(0));
                }
            }
            KeyCode::End => {
                if !self.rcast_stations.is_empty() {
                    self.rcast_list_state
                        .select(Some(self.rcast_stations.len() - 1));
                }
            }
            KeyCode::Enter => {
                if let Some(i) = self.rcast_list_state.selected() {
                    if i < self.rcast_stations.len() {
                        // Clone the values to avoid borrowing issues
                        let name = self.rcast_stations[i].name.clone();
                        let url = self.rcast_stations[i].url.clone();
                        let description = self.rcast_stations[i].description.clone();

                        self.play_station(&name, &url, description.as_deref())?;
                    }
                }
            }
            KeyCode::Char('r') => {
                // Refresh the station list
                self.refresh_rcast_stations()?;
            }
            KeyCode::Char('m') => {
                // Toggle mute
                if let Err(e) = self.player.toggle_mute(&self.visualizer) {
                    self.visualizer.set_error(format!("Mute failed: {}", e));
                }
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                // Increase volume
                if let Err(e) = self.player.volume_up(&self.visualizer) {
                    self.visualizer.set_error(format!("Volume failed: {}", e));
                }
            }
            KeyCode::Char('-') => {
                // Decrease volume
                if let Err(e) = self.player.volume_down(&self.visualizer) {
                    self.visualizer.set_error(format!("Volume failed: {}", e));
                }
            }
            KeyCode::Char('t') => {
                // Toggle showing top stations in Stream info
                self.show_top_stations = !self.show_top_stations;
            }
            KeyCode::Char('V') => {
                // Toggle visualization mode
                self.show_visualizations = !self.show_visualizations;
            }
            KeyCode::Char('a') => {
                // Add current station to saved stations
                if let Some(i) = self.rcast_list_state.selected() {
                    if i < self.rcast_stations.len() {
                        let station = &self.rcast_stations[i];

                        // Check if this URL already exists using db function
                        if let Ok(None) = crate::db::find_station_by_url(&self.conn, &station.url) {
                            // Only add the station if the URL doesn't exist yet
                            crate::db::add_station(
                                &self.conn,
                                &station.name,
                                &station.url,
                                station.description.as_deref(),
                            )?;
                        }

                        // Reload stations (this will also remove any duplicates)
                        self.stations = crate::db::load_stations(&self.conn)?;
                    }
                }
            }
            KeyCode::Char('q') => {
                return Ok(true);
            }
            KeyCode::Char('/') => {
                // Enter search mode
                self.mode = AppMode::Searching;
                self.search_query.clear();
                self.search_results.clear();
                self.search_list_state.select(None);
            }
            _ => {}
        }

        Ok(false)
    }

    // Helper method to play a station and track stats
    fn play_station(
        &mut self,
        name: &str,
        url: &str,
        description: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        // First play the station
        match self
            .player
            .play_station(name.to_string(), url.to_string(), &self.visualizer)
        {
            Ok(()) => {
                // Make sure the visualizer is marked as playing
                self.visualizer.set_playing(true);
            }
            Err(e) => {
                // Surface error to UI instead of silently failing
                self.visualizer.set_error(format!("Playback failed: {}", e));
                self.current_station_id = None;
                return Ok(()); // Don't propagate - we showed the error in UI
            }
        }

        // Then handle the station ID for stats tracking
        // Use the db module function instead of raw SQL
        if let Ok(Some(id)) = crate::db::find_station_by_url(&self.conn, url) {
            // URL already exists, use the existing station ID
            self.current_station_id = Some(id);
        } else {
            // URL doesn't exist, add the new station
            match crate::db::add_station(&self.conn, name, url, description) {
                Ok(id) => {
                    self.current_station_id = Some(id);
                    // Update the local stations list to include the new station
                    self.stations = crate::db::load_stations(&self.conn)?;
                }
                Err(_) => {
                    // If adding fails for any reason, set current station ID to None
                    self.current_station_id = None;
                }
            }
        }

        // Reset the stats timer if we have a valid station ID
        if self.current_station_id.is_some() {
            self.stats_last_update = Instant::now();
        }

        Ok(())
    }

    // Handle search mode input events
    fn handle_search_mode(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<(), Box<dyn Error>> {
        match key.code {
            KeyCode::Esc => {
                // Exit search mode and return to normal mode
                self.mode = AppMode::Normal;
            }
            KeyCode::Enter => {
                // If we have a selected search result and we hit Enter, play that station
                if let Some(i) = self.search_list_state.selected() {
                    if i < self.search_results.len() {
                        // Clone the values to avoid borrowing issues
                        let name = self.search_results[i].name.clone();
                        let url = self.search_results[i].url.clone();
                        let description = self.search_results[i].description.clone();

                        self.play_station(&name, &url, description.as_deref())?;

                        // Exit search mode
                        self.mode = AppMode::Normal;
                    }
                }
            }
            KeyCode::Char(c) => {
                // Add character to search query
                self.search_query.push(c);
                self.update_search_results();
            }
            KeyCode::Backspace => {
                // Remove character from search query
                self.search_query.pop();
                self.update_search_results();
            }
            KeyCode::Down => {
                let len = self.search_results.len();
                Self::navigate_down(&mut self.search_list_state, len);
            }
            KeyCode::Up => {
                let len = self.search_results.len();
                Self::navigate_up(&mut self.search_list_state, len);
            }
            _ => {}
        }
        Ok(())
    }

    // Update search results based on current search query
    fn update_search_results(&mut self) {
        self.search_results.clear();

        if self.search_query.is_empty() {
            // If query is empty, don't show any results
            return;
        }

        // Convert query to lowercase for case-insensitive search
        let query = self.search_query.to_lowercase();

        // Track URLs we've already added to prevent duplicates
        let mut added_urls = std::collections::HashSet::new();

        // Search for stations matching the query in both regular and RCast stations
        // First check in regular stations
        for station in &self.stations {
            // Only add each URL once
            if added_urls.contains(&station.url) {
                continue;
            }

            if station.name.to_lowercase().contains(&query) {
                self.search_results.push(station.clone());
                added_urls.insert(station.url.clone());
            } else if let Some(desc) = &station.description {
                if desc.to_lowercase().contains(&query) {
                    self.search_results.push(station.clone());
                    added_urls.insert(station.url.clone());
                }
            }
        }

        // Then check in RCast stations
        for rcast_station in &self.rcast_stations {
            // Skip if we already have this URL from local stations
            if added_urls.contains(&rcast_station.url) {
                continue;
            }

            if rcast_station.name.to_lowercase().contains(&query) {
                // Convert RCast station to regular station
                let station = Station {
                    id: 0, // This will be assigned by the database if needed
                    name: rcast_station.name.clone(),
                    url: rcast_station.url.clone(),
                    favorite: false,
                    description: rcast_station.description.clone(),
                };

                self.search_results.push(station);
                added_urls.insert(rcast_station.url.clone());
            } else if let Some(desc) = &rcast_station.description {
                if desc.to_lowercase().contains(&query) && !added_urls.contains(&rcast_station.url)
                {
                    // Convert RCast station to regular station
                    let station = Station {
                        id: 0,
                        name: rcast_station.name.clone(),
                        url: rcast_station.url.clone(),
                        favorite: false,
                        description: rcast_station.description.clone(),
                    };

                    self.search_results.push(station);
                    added_urls.insert(rcast_station.url.clone());
                }
            }
        }

        // Search in Radio Browser stations
        for rb_station in &self.rb_stations {
            if added_urls.contains(&rb_station.url) {
                continue;
            }
            let query_lower = self.search_query.to_lowercase();
            let name_match = rb_station.name.to_lowercase().contains(&query_lower);
            let tags_match = rb_station.tags.as_deref()
                .map(|t| t.to_lowercase().contains(&query_lower))
                .unwrap_or(false);
            let country_match = rb_station.country.as_deref()
                .map(|c| c.to_lowercase().contains(&query_lower))
                .unwrap_or(false);

            if name_match || tags_match || country_match {
                let station = crate::db::Station {
                    id: 0,
                    name: rb_station.name.clone(),
                    url: rb_station.url.clone(),
                    favorite: false,
                    description: rb_station.tags.clone(),
                };
                if !rb_station.url.is_empty() {
                    self.search_results.push(station);
                    added_urls.insert(rb_station.url.clone());
                }
            }
        }

        // If we have search results, select the first one
        if !self.search_results.is_empty() {
            self.search_list_state.select(Some(0));
        } else {
            self.search_list_state.select(None);
        }
    }

    // Function to refresh the RCast stations list
    fn refresh_rcast_stations(&mut self) -> Result<(), Box<dyn Error>> {
        // Set the loading flag and clear current stations
        self.rcast_loading = true;
        self.rcast_stations.clear();

        // Use the reusable runtime, or create a new one if needed
        let result = if let Some(ref rt) = self.tokio_runtime {
            rt.block_on(crate::rcast::fetch_stations())
        } else {
            match tokio::runtime::Runtime::new() {
                Ok(rt) => rt.block_on(crate::rcast::fetch_stations()),
                Err(e) => {
                    self.rcast_stations.push(crate::rcast::RcastStation {
                        name: "Error initializing fetcher".to_string(),
                        url: "".to_string(),
                        description: Some(format!("Runtime error: {}. Try refreshing with 'r'", e)),
                        bitrate: None,
                        genre: None,
                        listeners: None,
                    });
                    self.rcast_loading = false;
                    return Ok(());
                }
            }
        };

        match result {
            Ok(stations) => {
                self.rcast_stations = stations;
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

    fn handle_radiobrowser_stations_mode(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        use crossterm::event::KeyCode;

        // Sub-mode: typing a genre/tag filter
        if self.rb_entering_filter {
            match key.code {
                KeyCode::Esc => {
                    self.rb_entering_filter = false;
                    self.rb_filter_input.clear();
                }
                KeyCode::Enter => {
                    let tag = self.rb_filter_input.trim().to_string();
                    self.rb_filter.tag = if tag.is_empty() { None } else { Some(tag) };
                    self.rb_entering_filter = false;
                    self.rb_filter_input.clear();
                    self.refresh_radiobrowser_stations()?;
                }
                KeyCode::Backspace => {
                    self.rb_filter_input.pop();
                }
                KeyCode::Char(c) => {
                    self.rb_filter_input.push(c);
                }
                _ => {}
            }
            return Ok(false);
        }

        match key.code {
            KeyCode::Esc | KeyCode::Tab => {
                self.mode = AppMode::Normal;
                if !self.stations.is_empty() && self.list_state.selected().is_none() {
                    self.list_state.select(Some(0));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let len = self.rb_stations.len();
                if len > 0 {
                    let i = match self.rb_list_state.selected() {
                        Some(i) => (i + 1) % len,
                        None => 0,
                    };
                    self.rb_list_state.select(Some(i));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let len = self.rb_stations.len();
                if len > 0 {
                    let i = match self.rb_list_state.selected() {
                        Some(i) => {
                            if i == 0 { len - 1 } else { i - 1 }
                        }
                        None => 0,
                    };
                    self.rb_list_state.select(Some(i));
                }
            }
            KeyCode::PageDown => {
                let len = self.rb_stations.len();
                if len > 0 {
                    let i = match self.rb_list_state.selected() {
                        Some(i) => std::cmp::min(i + 10, len - 1),
                        None => 0,
                    };
                    self.rb_list_state.select(Some(i));
                }
            }
            KeyCode::PageUp => {
                if !self.rb_stations.is_empty() {
                    let i = match self.rb_list_state.selected() {
                        Some(i) => i.saturating_sub(10),
                        None => 0,
                    };
                    self.rb_list_state.select(Some(i));
                }
            }
            KeyCode::Home => {
                if !self.rb_stations.is_empty() {
                    self.rb_list_state.select(Some(0));
                }
            }
            KeyCode::End => {
                if !self.rb_stations.is_empty() {
                    self.rb_list_state.select(Some(self.rb_stations.len() - 1));
                }
            }
            KeyCode::Enter => {
                if let Some(i) = self.rb_list_state.selected() {
                    if i < self.rb_stations.len() {
                        let name = self.rb_stations[i].name.clone();
                        let url = self.rb_stations[i].url.clone();
                        let desc = crate::radiobrowser::build_rb_description(&self.rb_stations[i]);
                        if !url.is_empty() {
                            self.play_station(&name, &url, Some(&desc))?;
                        }
                    }
                }
            }
            KeyCode::Char('a') => {
                if let Some(i) = self.rb_list_state.selected() {
                    if i < self.rb_stations.len() {
                        let url = self.rb_stations[i].url.clone();
                        let name = self.rb_stations[i].name.clone();
                        let desc = crate::radiobrowser::build_rb_description(&self.rb_stations[i]);
                        if !url.is_empty() {
                            if let Ok(None) = crate::db::find_station_by_url(&self.conn, &url) {
                                let desc_opt = if desc.is_empty() { None } else { Some(desc.as_str()) };
                                crate::db::add_station(&self.conn, &name, &url, desc_opt)?;
                            }
                            self.stations = crate::db::load_stations(&self.conn)?;
                        }
                    }
                }
            }
            KeyCode::Char('r') => {
                self.rb_filter = crate::radiobrowser::RadioBrowserFilter::default();
                self.refresh_radiobrowser_stations()?;
            }
            KeyCode::Char('g') => {
                self.rb_entering_filter = true;
                self.rb_filter_input.clear();
            }
            KeyCode::Char('q') => {
                return Ok(true);
            }
            _ => {}
        }
        Ok(false)
    }

    fn refresh_radiobrowser_stations(&mut self) -> Result<(), Box<dyn Error>> {
        self.rb_loading = true;
        self.rb_stations.clear();

        let filter = self.rb_filter.clone();

        let result = if let Some(ref rt) = self.tokio_runtime {
            rt.block_on(crate::radiobrowser::fetch_stations(&filter))
        } else {
            match tokio::runtime::Runtime::new() {
                Ok(rt) => rt.block_on(crate::radiobrowser::fetch_stations(&filter)),
                Err(_e) => {
                    self.rb_stations.push(crate::radiobrowser::RadioBrowserStation {
                        name: "Error initializing async runtime".to_string(),
                        url: String::new(),
                        codec: None,
                        bitrate: None,
                        tags: None,
                        country: None,
                        clickcount: None,
                        votes: None,
                    });
                    self.rb_loading = false;
                    return Ok(());
                }
            }
        };

        match result {
            Ok(stations) => {
                self.rb_stations = stations;
                if self.rb_stations.is_empty() {
                    self.rb_stations.push(crate::radiobrowser::RadioBrowserStation {
                        name: "No stations found. Try a different filter ('g') or refresh ('r').".to_string(),
                        url: String::new(),
                        codec: None,
                        bitrate: None,
                        tags: None,
                        country: None,
                        clickcount: None,
                        votes: None,
                    });
                }
            }
            Err(e) => {
                self.rb_stations.push(crate::radiobrowser::RadioBrowserStation {
                    name: format!("Error: {}. Press 'r' to retry.", e),
                    url: String::new(),
                    codec: None,
                    bitrate: None,
                    tags: None,
                    country: None,
                    clickcount: None,
                    votes: None,
                });
            }
        }

        if !self.rb_stations.is_empty() {
            self.rb_list_state.select(Some(0));
        }
        self.rb_loading = false;
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
                let mut path = dirs::home_dir().ok_or("Could not find home directory")?;
                path.push("Library");
                path.push("Application Support");
                path.push("radio_cli");
                path
            }
            #[cfg(target_os = "linux")]
            {
                let mut path = dirs::home_dir().ok_or("Could not find home directory")?;
                path.push(".local");
                path.push("share");
                path.push("radio_cli");
                path
            }
            #[cfg(target_os = "windows")]
            {
                let mut path = dirs::data_dir().ok_or("Could not find data directory")?;
                path.push("radio_cli");
                path
            }
            #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
            {
                let mut path = dirs::home_dir().ok_or("Could not find home directory")?;
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
