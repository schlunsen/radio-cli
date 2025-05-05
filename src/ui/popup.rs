use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Line as TextLine},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

// Function to render the add station popup
pub fn render_add_station_popup(
    f: &mut Frame,
    name: &str,
    url: &str,
    description: &str,
    input_field: usize,
    input_cursor: usize,
) {
    let size = f.size();
    
    // Create a centered popup area
    let popup_width = 60.min(size.width - 4);
    let popup_height = 10.min(size.height - 4);
    
    let popup_area = Rect {
        x: (size.width - popup_width) / 2,
        y: (size.height - popup_height) / 2,
        width: popup_width,
        height: popup_height,
    };
    
    // Clear the area behind the popup
    f.render_widget(Clear, popup_area);
    
    // Draw the popup frame
    let popup_block = Block::default()
        .title("Add New Station")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .style(Style::default().bg(Color::Black));
    
    f.render_widget(popup_block, popup_area);
    
    // Create inner layout for fields
    let inner_area = Rect {
        x: popup_area.x + 2,
        y: popup_area.y + 1,
        width: popup_area.width - 4,
        height: popup_area.height - 2,
    };
    
    let input_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(2), // Name
                Constraint::Length(2), // URL
                Constraint::Length(2), // Description
            ]
            .as_ref(),
        )
        .split(inner_area);
    
    // Render each field
    render_input_field(f, input_chunks[0], "Name:", name, input_field == 0, input_cursor);
    render_input_field(f, input_chunks[1], "URL:", url, input_field == 1, input_cursor);
    render_input_field(f, input_chunks[2], "Description:", description, input_field == 2, input_cursor);
}

// Helper function to render an input field
fn render_input_field(f: &mut Frame, area: Rect, label: &str, value: &str, is_focused: bool, cursor_pos: usize) {
    // Calculate lengths
    let label_width = label.len() as u16 + 1; // +1 for the space
    
    // Create label area
    let label_area = Rect {
        x: area.x,
        y: area.y,
        width: label_width,
        height: 1,
    };
    
    // Create input area
    let input_area = Rect {
        x: area.x + label_width,
        y: area.y,
        width: area.width - label_width,
        height: 1,
    };
    
    // Render label
    let label_text = Paragraph::new(label).style(Style::default().fg(Color::Gray));
    f.render_widget(label_text, label_area);
    
    // Determine style based on focus
    let input_style = if is_focused {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Gray)
    };
    
    // Handle cursor display - add a visible cursor marker if this field is focused
    let text = if is_focused {
        let left = value.chars().take(cursor_pos).collect::<String>();
        let cursor_char = value.chars().nth(cursor_pos).unwrap_or(' ');
        let right = value.chars().skip(cursor_pos + 1).collect::<String>();
        
        let mut spans = vec![Span::styled(left, input_style)];
        
        // Add the cursor character with inverted colors
        spans.push(Span::styled(
            cursor_char.to_string(),
            Style::default().fg(Color::Black).bg(Color::White),
        ));
        
        spans.push(Span::styled(right, input_style));
        
        TextLine::from(spans)
    } else {
        // Just display the value without cursor
        TextLine::from(Span::styled(value, input_style))
    };
    
    let input_text = Paragraph::new(text);
    f.render_widget(input_text, input_area);
}