use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::rcast::RcastStation;

// Function to render the RCast stations pane
pub fn render_rcast_stations(
    f: &mut Frame,
    stations: &[RcastStation],
    list_state: &mut ListState,
    area: Rect,
    loading: bool,
) {
    // Create a block for the stations list
    let rcast_block = Block::default()
        .borders(Borders::ALL)
        .title("RCast Radio Stations");

    if loading {
        // Show loading message if we're waiting for stations to load
        let loading_text = Paragraph::new("Loading stations from RCast.net...")
            .style(Style::default().fg(Color::Yellow))
            .block(rcast_block);
        f.render_widget(loading_text, area);
        return;
    }

    // If we have stations, show them in a list
    if !stations.is_empty() {
        // Create list items
        let items: Vec<ListItem> = stations
            .iter()
            .map(|s| {
                let name = if let Some(bitrate) = &s.bitrate {
                    format!("{} ({})", s.name, bitrate)
                } else {
                    s.name.clone()
                };

                ListItem::new(Span::styled(name, Style::default().fg(Color::Cyan)))
            })
            .collect();

        // Create the list widget
        let list = List::new(items)
            .block(rcast_block)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        // Render the list
        f.render_stateful_widget(list, area, list_state);
    } else {
        // No stations available
        let no_stations_text = Paragraph::new("No stations found. Press 'r' to refresh.")
            .style(Style::default().fg(Color::Red))
            .block(rcast_block);
        f.render_widget(no_stations_text, area);
    }
}
