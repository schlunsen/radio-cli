use crate::visualizations::VisualizationManager;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn render_visualization_menu(
    f: &mut Frame,
    vis_manager: &VisualizationManager,
    vis_menu_state: &mut ListState,
    area: Rect,
) {
    // Create a centered popup
    let popup_width = 50;
    let popup_height = 15;
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_rect = Rect::new(
        area.x + popup_x,
        area.y + popup_y,
        popup_width.min(area.width),
        popup_height.min(area.height),
    );

    // Render clear behind the popup
    f.render_widget(Clear, popup_rect);

    // Create a block for the popup
    let popup_block = Block::default()
        .title("Select Visualization")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));

    // Get visualizations
    let vis_list = vis_manager.get_available_visualizations();

    // Render the popup with a margin inside
    let inner_popup = popup_block.inner(popup_rect);
    f.render_widget(popup_block, popup_rect);

    // Set up layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(8), // Visualization list
                Constraint::Length(3), // Description
            ]
            .as_ref(),
        )
        .split(inner_popup);

    // Create visualization list items
    let items: Vec<ListItem> = vis_list
        .iter()
        .map(|(_, name, _)| ListItem::new(Span::styled(*name, Style::default().fg(Color::White))))
        .collect();

    // Create list widget with highlighting
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Available Visualizations"),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // Render the list with the state
    f.render_stateful_widget(list, chunks[0], vis_menu_state);

    // Show description of selected visualization
    let description = if let Some(selected) = vis_menu_state.selected() {
        if selected < vis_list.len() {
            let (_, name, desc) = vis_list[selected];
            format!("{}: {}", name, desc)
        } else {
            "Select a visualization".to_string()
        }
    } else {
        "Select a visualization".to_string()
    };

    // Create and render the description paragraph
    let desc_para = Paragraph::new(description)
        .block(Block::default().borders(Borders::ALL).title("Description"))
        .style(Style::default().fg(Color::White));

    f.render_widget(desc_para, chunks[1]);
}
