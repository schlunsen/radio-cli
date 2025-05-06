use crate::app::AppMode;
use crate::audio::AudioVisualizer;
use crate::db::Station;
use crate::visualizations::VisualizationManager;
mod popup;
mod vis_menu;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{canvas::Canvas, Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

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
) {
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
        if let Some(desc) = &stations[selected].description {
            format!(
                "Selected: {}\n\nDescription: {}",
                stations[selected].name, desc
            )
        } else {
            format!("Selected: {}", stations[selected].name)
        }
    } else {
        "No stream playing".to_string()
    };

    let metadata = Paragraph::new(metadata_text)
        .block(Block::default().borders(Borders::ALL).title("Stream Info"));

    f.render_widget(metadata, vis_chunks[1]);

    // Render help area
    let help_text = match mode {
        AppMode::Normal => "↑/↓: Navigate  ⏎: Play  s: Stop  f: Favorite  a: Add Station  v: Visualizations  q: Quit",
        AppMode::AddingStation => "Tab: Next Field  Enter: Confirm  Esc: Cancel",
        AppMode::VisualizationMenu => "↑/↓: Navigate  Enter: Select  Esc: Cancel",
    };

    let help =
        Paragraph::new(help_text).block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help, main_help_chunks[1]);

    // Render the appropriate popup based on mode
    match mode {
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
        AppMode::VisualizationMenu => {
            vis_menu::render_visualization_menu(f, vis_manager, vis_menu_state, size);
        }
        _ => {}
    }
}
