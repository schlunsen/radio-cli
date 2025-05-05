use crate::audio::AudioVisualizer;
use crate::db::Station;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, BarChart},
    Frame,
};

pub fn ui(f: &mut Frame, stations: &[Station], list_state: &mut ListState, visualizer: &AudioVisualizer) {
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
    let help = Paragraph::new("↑/↓: Navigate  ⏎: Play  s: Stop  f: Favorite  q: Quit")
        .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help, main_help_chunks[1]);
}