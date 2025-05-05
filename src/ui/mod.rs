use crate::app::AppMode;
use crate::audio::AudioVisualizer;
use crate::db::Station;
mod popup;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, canvas::{Canvas, Rectangle, Line}},
    Frame,
};

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

    // Create 90s starfield visualization using canvas
    let canvas = Canvas::default()
        .block(vis_block)
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 100.0])
        .paint(|ctx| {
            // Background - dark space
            ctx.draw(&Rectangle {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
                color: Color::Rgb(0, 0, 20), // Very dark blue
            });
            
            // Star color palette
            let colors = [
                Color::Rgb(255, 255, 255), // White
                Color::Rgb(200, 200, 255), // Light blue
                Color::Rgb(255, 230, 200), // Light yellow
                Color::Rgb(255, 200, 200), // Light red
                Color::Rgb(200, 255, 200), // Light green
            ];
            
            // Draw each star in the starfield
            for star in &state.stars {
                // Calculate projected position based on perspective
                // As z gets closer to 1.0, the star gets closer to the center
                // and appears larger
                
                // Center of screen
                let center_x = 50.0;
                let center_y = 50.0;
                
                // Calculate projected position (perspective projection)
                // Higher z = closer to viewer = further from center
                let scale = 1.0 / (1.01 - star.z.min(0.99)); // Avoid division by zero
                let projected_x = center_x + star.x * scale * 40.0; // Scale factor for width
                let projected_y = center_y + star.y * scale * 40.0; // Scale factor for height
                
                // Calculate size based on z (closer = larger)
                // Use non-linear scaling for more dramatic effect
                let size = star.z.powf(2.0) * 3.0 * (state.warp_speed * 0.5 + 0.5);
                
                // Calculate brightness based on z and intrinsic brightness
                let brightness = star.brightness * star.z.powf(1.5);
                
                // Skip if star is out of bounds
                if projected_x < 0.0 || projected_x > 100.0 || 
                   projected_y < 0.0 || projected_y > 100.0 {
                    continue;
                }
                
                // Get color for this star
                let base_color = colors[star.color as usize % colors.len()];
                
                // Adjust color based on brightness
                let color = match base_color {
                    Color::Rgb(r, g, b) => {
                        Color::Rgb(
                            (r as f64 * brightness) as u8,
                            (g as f64 * brightness) as u8,
                            (b as f64 * brightness) as u8,
                        )
                    },
                    _ => base_color,
                };
                
                // Draw the star
                ctx.draw(&Rectangle {
                    x: projected_x - size / 2.0,
                    y: projected_y - size / 2.0,
                    width: size,
                    height: size,
                    color,
                });
                
                // For closer stars, add a trail/streak effect when at high warp speed
                if star.z > 0.7 && state.warp_speed > 1.5 {
                    // Calculate trail length based on speed and z
                    let trail_length = star.z * state.warp_speed * 5.0;
                    
                    // Direction from center
                    let dx = projected_x - center_x;
                    let dy = projected_y - center_y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    
                    if dist > 0.0 {
                        // Normalized direction
                        let nx = dx / dist;
                        let ny = dy / dist;
                        
                        // Draw trail pointing outward from center
                        let trail_x = projected_x - nx * trail_length;
                        let trail_y = projected_y - ny * trail_length;
                        
                        // Make trail fade out
                        let trail_color = match color {
                            Color::Rgb(r, g, b) => {
                                Color::Rgb(
                                    (r as f64 * 0.5) as u8,
                                    (g as f64 * 0.5) as u8,
                                    (b as f64 * 0.5) as u8,
                                )
                            },
                            _ => color,
                        };
                        
                        ctx.draw(&Line {
                            x1: trail_x,
                            y1: trail_y,
                            x2: projected_x,
                            y2: projected_y,
                            color: trail_color,
                        });
                    }
                }
            }
            
            // Draw warp speed indicator
            let warp = state.warp_speed;
            if state.is_playing {
                let indicator_width = 20.0;
                let indicator_height = 5.0;
                let x = 50.0 - indicator_width / 2.0;
                let y = 90.0;
                
                // Draw background
                ctx.draw(&Rectangle {
                    x,
                    y,
                    width: indicator_width,
                    height: indicator_height,
                    color: Color::Rgb(30, 30, 50),
                });
                
                // Draw filled portion based on warp speed (1.0 to 3.0 mapped to 0-100%)
                let fill_width = (warp - 1.0) / 2.0 * indicator_width;
                ctx.draw(&Rectangle {
                    x,
                    y,
                    width: fill_width.min(indicator_width),
                    height: indicator_height,
                    color: Color::Rgb(
                        (100.0 + (155.0 * (warp - 1.0) / 2.0)) as u8,
                        (200.0 - (150.0 * (warp - 1.0) / 2.0)) as u8,
                        255,
                    ),
                });
            }
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
        AppMode::Normal => "↑/↓: Navigate  ⏎: Play  s: Stop  f: Favorite  a: Add Station  q: Quit",
        AppMode::AddingStation => "Tab: Next Field  Enter: Confirm  Esc: Cancel",
    };
    
    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help, main_help_chunks[1]);
    
    // If in add station mode, render the popup
    if *mode == AppMode::AddingStation {
        popup::render_add_station_popup(
            f, 
            add_station_name,
            add_station_url,
            add_station_desc,
            input_field,
            input_cursor
        );
    }
}