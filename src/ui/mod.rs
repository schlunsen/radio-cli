use crate::app::AppMode;
use crate::audio::AudioVisualizer;
use crate::db::Station;
use crate::visualizations::VisualizationManager;
mod popup;
mod rcast_stations;
mod vis_menu;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{canvas::Canvas, Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
pub use rcast_stations::render_rcast_stations;

#[allow(clippy::too_many_arguments)]
pub fn ui(
    f: &mut Frame,
    stations: &[Station],
    list_state: &mut ListState,
    visualizer: &AudioVisualizer,
    mode: &AppMode,
    add_station_name: &str,
    add_station_url: &str,
    add_station_desc: &str,
    input_field: usize,
    input_cursor: usize,
    vis_manager: &VisualizationManager,
    vis_menu_state: &mut ListState,
    rcast_stations: &[crate::rcast::RcastStation],
    rcast_list_state: &mut ListState,
    rcast_loading: bool,
) {
    let size = f.size();

    // First split into main area and help area
    let main_help_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(85), Constraint::Percentage(15)].as_ref())
        .split(size);

    // Split main area into stations list (35%) and right panel (65%)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
        .split(main_help_chunks[0]);

    // Render help area
    let help_text = match mode {
        AppMode::Normal => "↑/↓: Navigate  ⏎: Play  s: Stop  f: Favorite  a: Add  e: Edit  d: Delete  v: Visualizations  Tab: RCast  q: Quit",
        AppMode::AddingStation => "Tab: Next Field  Enter: Confirm  Esc: Cancel",
        AppMode::EditingStation => "Tab: Next Field  Enter: Save  Esc: Cancel",
        AppMode::DeletingStation => "y: Confirm Delete  n/Esc: Cancel",
        AppMode::VisualizationMenu => "↑/↓: Navigate  Enter: Select  Esc: Cancel",
        AppMode::RcastStations => "↑/↓: Navigate  ⏎: Play  r: Refresh  Tab: Main View  q: Quit",
    };

    let help =
        Paragraph::new(help_text).block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help, main_help_chunks[1]);

    // The main UI always shows, regardless of the mode
    // We'll change what appears in the right pane based on the mode

    // Render stations list (always visible in left pane)
    let items: Vec<ListItem> = stations
        .iter()
        .map(|s| {
            let mut content = s.name.clone();
            if s.favorite {
                content = format!("★ {}", content);
            }
            ListItem::new(Span::styled(content, Style::default().fg(Color::Cyan)))
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

    // Render right pane content based on mode
    match mode {
        AppMode::Normal => {
            // Get status from the visualizer state
            let (status_text, state) = if let Ok(state) = visualizer.state.lock() {
                let status = if state.is_playing {
                    "Playing"
                } else {
                    "Paused"
                };
                (status, state.clone())
            } else {
                ("Locked", crate::audio::AudioState::new()) // Fallback if we can't acquire the lock
            };

            let vis_block = Block::default()
                .borders(Borders::ALL)
                .title(format!("Visualization - {}", status_text));

            // Split visualization area into visualization and metadata
            let vis_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                .split(main_chunks[1]);

            // Create a canvas with the active visualization
            let canvas = Canvas::default()
                .block(vis_block)
                .x_bounds([0.0, 100.0])
                .y_bounds([0.0, 100.0])
                .paint(|ctx| {
                    // Use the current visualization from the manager
                    vis_manager.render(ctx, &state);
                });

            f.render_widget(canvas, vis_chunks[0]);

            // Display stream metadata
            let metadata_text = if let Some(info) = &state.stream_info {
                let unknown = "Unknown".to_string();
                let song = info.current_song.as_ref().unwrap_or(&unknown);
                format!(
                    "Station: {}\nFormat: {}\nBitrate: {}\nCurrent Song: {}",
                    info.station_name, info.format, info.bitrate, song
                )
            } else if let Some(selected) = list_state.selected() {
                // When no stream is playing, show the description of the selected station
                if selected < stations.len() {
                    if let Some(desc) = &stations[selected].description {
                        format!(
                            "Selected: {}\n\nDescription: {}",
                            stations[selected].name, desc
                        )
                    } else {
                        format!("Selected: {}", stations[selected].name)
                    }
                } else {
                    "No station selected".to_string()
                }
            } else {
                "No stream playing".to_string()
            };

            let metadata = Paragraph::new(metadata_text)
                .block(Block::default().borders(Borders::ALL).title("Stream Info"));

            f.render_widget(metadata, vis_chunks[1]);
        }
        AppMode::RcastStations => {
            // Show RCast stations in the right pane instead of visualization
            render_rcast_stations(
                f,
                rcast_stations,
                rcast_list_state,
                main_chunks[1],
                rcast_loading,
            );
        }
        AppMode::AddingStation => {
            popup::render_add_station_popup(
                f,
                add_station_name,
                add_station_url,
                add_station_desc,
                input_field,
                input_cursor,
            );
        }
        AppMode::EditingStation => {
            if let Ok(app_guard) = crate::app::APP_STATE.lock() {
                if let Some(app) = app_guard.as_ref() {
                    popup::render_edit_station_popup(
                        f,
                        &app.edit_station_name,
                        &app.edit_station_url,
                        &app.edit_station_desc,
                        input_field,
                        input_cursor,
                    );
                }
            }
        }
        AppMode::DeletingStation => {
            if let Some(selected) = list_state.selected() {
                if selected < stations.len() {
                    popup::render_delete_station_popup(f, &stations[selected].name);
                }
            }
        }
        AppMode::VisualizationMenu => {
            vis_menu::render_visualization_menu(f, vis_manager, vis_menu_state, size);
        }
    }
}
