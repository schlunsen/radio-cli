use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
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

    // Split the area into stations list (70%) and station details (30%)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(area);

    if loading {
        // Show loading message if we're waiting for stations to load
        let loading_text = Paragraph::new("Loading stations from RCast.net...")
            .style(Style::default().fg(Color::Yellow))
            .block(rcast_block);
        f.render_widget(loading_text, chunks[0]);

        // Show instructions
        let help_text =
            Paragraph::new("Press 'r' to refresh stations • Press Tab to return to visualization")
                .block(Block::default().borders(Borders::ALL).title("Help"));
        f.render_widget(help_text, chunks[1]);
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
        f.render_stateful_widget(list, chunks[0], list_state);

        // Show details of the selected station
        if let Some(selected) = list_state.selected() {
            if selected < stations.len() {
                let station = &stations[selected];
                let mut details = String::new();

                details.push_str(&format!("Name: {}\n", station.name));
                details.push_str(&format!("URL: {}\n", station.url));

                if let Some(genre) = &station.genre {
                    details.push_str(&format!("Genre: {}\n", genre));
                }

                if let Some(bitrate) = &station.bitrate {
                    details.push_str(&format!("Bitrate: {}\n", bitrate));
                }

                if let Some(listeners) = station.listeners {
                    details.push_str(&format!("Listeners: {}\n", listeners));
                }

                let details_widget = Paragraph::new(details).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Station Details"),
                );
                f.render_widget(details_widget, chunks[1]);
            }
        } else {
            // No station selected
            let help_text = Paragraph::new("Use Up/Down to navigate • Enter to play • r to refresh • a to add • Tab to switch views")
                .block(Block::default().borders(Borders::ALL).title("Help"));
            f.render_widget(help_text, chunks[1]);
        }
    } else {
        // No stations available
        let no_stations_text = Paragraph::new("No stations found. Press 'r' to refresh.")
            .style(Style::default().fg(Color::Red))
            .block(rcast_block);
        f.render_widget(no_stations_text, chunks[0]);

        // Show help
        let help_text =
            Paragraph::new("Press 'r' to refresh stations • Press Tab to return to visualization")
                .block(Block::default().borders(Borders::ALL).title("Help"));
        f.render_widget(help_text, chunks[1]);
    }
}
