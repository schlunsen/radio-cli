use crate::app::AppMode;
use crate::audio::AudioVisualizer;
use crate::db::{format_play_time, get_station_stats, get_top_stations, Station};
use crate::visualizations::VisualizationManager;
use rusqlite::{params, Connection};
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
    show_top_stations: bool,
    conn: &Connection,
    current_station_id: Option<i32>,
    search_query: &str,
    search_results: &[Station],
    search_list_state: &mut ListState,
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
        AppMode::Normal => "↑/↓: Navigate  ⏎: Play  s: Stop  m: Mute/Unmute  +/-: Volume  f: Favorite  a: Add  e: Edit  d: Delete  t: Toggle Top Stations  v: Visualizations  /: Search  Tab: RCast  q: Quit",
        AppMode::AddingStation => "Tab: Next Field  Enter: Confirm  Esc: Cancel",
        AppMode::EditingStation => "Tab: Next Field  Enter: Save  Esc: Cancel",
        AppMode::DeletingStation => "y: Confirm Delete  n/Esc: Cancel",
        AppMode::VisualizationMenu => "↑/↓: Navigate  Enter: Select  Esc: Cancel",
        AppMode::RcastStations => "↑/↓: Navigate  ⏎: Play  m: Mute/Unmute  +/-: Volume  r: Refresh  t: Toggle Top Stations  /: Search  Tab: Main View  q: Quit",
        AppMode::Searching => "↑/↓: Navigate  ⏎: Play Selected  Esc: Cancel  Type to search...",
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
                    if state.is_muted {
                        "Muted"
                    } else {
                        "Playing"
                    }
                } else {
                    "Paused"
                };
                (status, state.clone())
            } else {
                ("Locked", crate::audio::AudioState::new()) // Fallback if we can't acquire the lock
            };

            // Make the mute status more prominent by adding a symbol
            let status_with_symbol = if state.is_muted {
                format!("Visualization - {} 🔇", status_text)
            } else {
                format!("Visualization - {} 🔊", status_text)
            };

            let vis_block = Block::default()
                .borders(Borders::ALL)
                .title(status_with_symbol);

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

            // Display stream metadata or top stations
            let metadata_text = if show_top_stations {
                // Show top 5 stations by play time
                match get_top_stations(conn, 5) {
                    Ok(top_stations) => {
                        if top_stations.is_empty() {
                            "No station play history yet.\nListen to some stations to build your stats!".to_string()
                        } else {
                            let mut text = "Top 5 Stations by Play Time:\n\n".to_string();
                            for (i, (station, play_time)) in top_stations.iter().enumerate() {
                                text.push_str(&format!(
                                    "{}. {} - {}\n",
                                    i + 1,
                                    station.name,
                                    format_play_time(*play_time)
                                ));
                            }
                            text
                        }
                    }
                    Err(_) => "Error loading top stations stats.".to_string(),
                }
            } else if let Some(info) = &state.stream_info {
                let unknown = "Unknown".to_string();
                let song = info.current_song.as_ref().unwrap_or(&unknown);

                // Start with basic stream info
                let mut text = format!(
                    "Station: {}\nFormat: {}\nBitrate: {}\nCurrent Song: {}\nMuted: {}",
                    info.station_name,
                    info.format,
                    info.bitrate,
                    song,
                    if state.is_muted { "Yes" } else { "No" }
                );

                // If we have a current station ID, add the stats
                if let Some(station_id) = current_station_id {
                    if let Ok(Some(stats)) = get_station_stats(conn, station_id) {
                        text.push_str(&format!(
                            "\n\nTotal Play Time: {}",
                            format_play_time(stats.total_play_time)
                        ));

                        if let Some(last_played) = stats.last_played {
                            // Convert timestamp to local date/time
                            let datetime = chrono::DateTime::from_timestamp(last_played, 0)
                                .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
                            let local_time = datetime.format("%Y-%m-%d %H:%M:%S");
                            text.push_str(&format!("\nLast Played: {}", local_time));
                        }
                    }
                }

                text
            } else if let Some(selected) = list_state.selected() {
                // When no stream is playing, show the description of the selected station
                if selected < stations.len() {
                    let mut text = format!("Selected: {}", stations[selected].name);

                    // Add station stats if available
                    if let Ok(Some(stats)) = get_station_stats(conn, stations[selected].id) {
                        text.push_str(&format!(
                            "\nTotal Play Time: {}",
                            format_play_time(stats.total_play_time)
                        ));

                        if let Some(last_played) = stats.last_played {
                            let datetime = chrono::DateTime::from_timestamp(last_played, 0)
                                .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
                            let local_time = datetime.format("%Y-%m-%d %H:%M:%S");
                            text.push_str(&format!("\nLast Played: {}", local_time));
                        }
                    }

                    // Add description if available
                    if let Some(desc) = &stations[selected].description {
                        text.push_str(&format!("\n\nDescription: {}", desc));
                    }

                    text
                } else {
                    "No station selected".to_string()
                }
            } else {
                "No stream playing".to_string()
            };

            let block_title = if show_top_stations {
                "Top Stations"
            } else {
                "Stream Info"
            };

            let metadata = Paragraph::new(metadata_text)
                .block(Block::default().borders(Borders::ALL).title(block_title));

            f.render_widget(metadata, vis_chunks[1]);
        }
        AppMode::RcastStations => {
            // Split the right pane for stations list and either stats or loading indicator
            let rcast_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                .split(main_chunks[1]);

            // Show RCast stations in the top part of the right pane
            render_rcast_stations(
                f,
                rcast_stations,
                rcast_list_state,
                rcast_chunks[0],
                rcast_loading,
            );

            // Show either stats or loading indicator in the bottom part
            if show_top_stations {
                // Show top 5 stations by play time
                let metadata_text = match get_top_stations(conn, 5) {
                    Ok(top_stations) => {
                        if top_stations.is_empty() {
                            "No station play history yet.\nListen to some stations to build your stats!".to_string()
                        } else {
                            let mut text = "Top 5 Stations by Play Time:\n\n".to_string();
                            for (i, (station, play_time)) in top_stations.iter().enumerate() {
                                text.push_str(&format!(
                                    "{}. {} - {}\n",
                                    i + 1,
                                    station.name,
                                    format_play_time(*play_time)
                                ));
                            }
                            text
                        }
                    }
                    Err(_) => "Error loading top stations stats.".to_string(),
                };

                let metadata = Paragraph::new(metadata_text)
                    .block(Block::default().borders(Borders::ALL).title("Top Stations"));

                f.render_widget(metadata, rcast_chunks[1]);
            } else if rcast_loading {
                // Show loading indicator
                let loading = Paragraph::new("Loading stations from RCast.net...")
                    .block(Block::default().borders(Borders::ALL).title("Loading"));

                f.render_widget(loading, rcast_chunks[1]);
            } else if let Some(selected) = rcast_list_state.selected() {
                // Show selected station info
                if selected < rcast_stations.len() {
                    let station = &rcast_stations[selected];
                    let mut text = format!("Selected: {}", station.name);

                    // Add description if available
                    if let Some(desc) = &station.description {
                        text.push_str(&format!("\n\nDescription: {}", desc));
                    }

                    // Add other available info
                    if let Some(bitrate) = &station.bitrate {
                        text.push_str(&format!("\nBitrate: {}", bitrate));
                    }

                    if let Some(genre) = &station.genre {
                        text.push_str(&format!("\nGenre: {}", genre));
                    }

                    if let Some(listeners) = &station.listeners {
                        text.push_str(&format!("\nListeners: {}", listeners));
                    }

                    // Try to find station stats in our database (by URL)
                    if let Ok(mut stmt) = conn.prepare("SELECT id FROM stations WHERE url = ?1") {
                        if let Ok(id_result) =
                            stmt.query_map(params![&station.url], |row| row.get::<_, i32>(0))
                        {
                            if let Some(id) = id_result.flatten().next() {
                                if let Ok(Some(stats)) = get_station_stats(conn, id) {
                                    text.push_str(&format!(
                                        "\n\nTotal Play Time: {}",
                                        format_play_time(stats.total_play_time)
                                    ));

                                    if let Some(last_played) = stats.last_played {
                                        let datetime =
                                            chrono::DateTime::from_timestamp(last_played, 0)
                                                .unwrap_or_else(|| {
                                                    chrono::DateTime::from_timestamp(0, 0).unwrap()
                                                });
                                        let local_time = datetime.format("%Y-%m-%d %H:%M:%S");
                                        text.push_str(&format!("\nLast Played: {}", local_time));
                                    }
                                }
                            }
                        }
                    }

                    let metadata = Paragraph::new(text)
                        .block(Block::default().borders(Borders::ALL).title("Station Info"));

                    f.render_widget(metadata, rcast_chunks[1]);
                }
            }
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
        AppMode::Searching => {
            // Split the main area into search input and search results
            let search_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(10)].as_ref())
                .split(main_chunks[0]);

            // Render search input
            let search_input = Paragraph::new(search_query.to_string())
                .block(Block::default().borders(Borders::ALL).title("Search"))
                .style(Style::default().fg(Color::Yellow));

            f.render_widget(search_input, search_chunks[0]);

            // Render search results
            let items: Vec<ListItem> = search_results
                .iter()
                .map(|s| {
                    let mut content = s.name.clone();
                    if s.favorite {
                        content = format!("★ {}", content);
                    }
                    ListItem::new(Span::styled(content, Style::default().fg(Color::Cyan)))
                })
                .collect();

            let results_list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Results"))
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            f.render_stateful_widget(results_list, search_chunks[1], search_list_state);

            // Render station details or visualization in the right pane
            if let Some(selected) = search_list_state.selected() {
                if selected < search_results.len() {
                    // Show details of the selected station
                    let station = &search_results[selected];
                    let mut details = String::new();

                    details.push_str(&format!("Name: {}\n", station.name));
                    details.push_str(&format!("URL: {}\n", station.url));

                    if let Some(desc) = &station.description {
                        details.push_str(&format!("\nDescription: {}\n", desc));
                    }

                    // Try to get station stats if available
                    if station.id > 0 {
                        if let Ok(Some(stats)) = get_station_stats(conn, station.id) {
                            details.push_str(&format!(
                                "\nTotal Play Time: {}",
                                format_play_time(stats.total_play_time)
                            ));

                            if let Some(last_played) = stats.last_played {
                                let datetime = chrono::DateTime::from_timestamp(last_played, 0)
                                    .unwrap_or_else(|| {
                                        chrono::DateTime::from_timestamp(0, 0).unwrap()
                                    });
                                let local_time = datetime.format("%Y-%m-%d %H:%M:%S");
                                details.push_str(&format!("\nLast Played: {}", local_time));
                            }
                        }
                    }

                    let details_widget = Paragraph::new(details).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Station Details"),
                    );

                    f.render_widget(details_widget, main_chunks[1]);
                }
            } else {
                // No station selected, show help
                let help_widget =
                    Paragraph::new("Type to search for stations. Results will appear here.")
                        .block(Block::default().borders(Borders::ALL).title("Search Help"));

                f.render_widget(help_widget, main_chunks[1]);
            }
        }
    }
}
