use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use crate::radiobrowser::RadioBrowserStation;

pub fn render_radiobrowser_stations(
    f: &mut Frame,
    stations: &[RadioBrowserStation],
    list_state: &mut ListState,
    area: Rect,
    loading: bool,
    filter_active: bool,
    filter_input: &str,
) {
    let title = if filter_active {
        format!("Radio Browser - Filter: {}_", filter_input)
    } else {
        "Radio Browser  [Enter:Play  a:Save  g:Genre  r:Reset  Esc:Back]".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(Style::default().fg(Color::Green));

    if loading {
        let msg = Paragraph::new("Fetching stations from Radio Browser API...")
            .style(Style::default().fg(Color::Yellow))
            .block(block);
        f.render_widget(msg, area);
        return;
    }

    if stations.is_empty() {
        let msg = Paragraph::new("No stations. Press 'r' to refresh.")
            .style(Style::default().fg(Color::Red))
            .block(block);
        f.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = stations
        .iter()
        .map(|s| {
            let mut label = s.name.clone();
            if let Some(br) = s.bitrate {
                if br > 0 {
                    label = format!("{} [{}kbps]", label, br);
                }
            }
            if let Some(ref c) = s.country {
                if !c.is_empty() {
                    label = format!("{} ({})", label, c);
                }
            }
            ListItem::new(Span::styled(label, Style::default().fg(Color::Green)))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, list_state);
}

pub fn render_radiobrowser_detail(
    f: &mut Frame,
    stations: &[RadioBrowserStation],
    list_state: &ListState,
    area: Rect,
    loading: bool,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Station Info")
        .title_style(Style::default().fg(Color::Green));

    if loading {
        f.render_widget(Paragraph::new("").block(block), area);
        return;
    }

    let text = if let Some(i) = list_state.selected() {
        if i < stations.len() {
            let s = &stations[i];
            let mut t = format!("Name:    {}\nURL:     {}", s.name, s.url);
            if let Some(ref tags) = s.tags {
                if !tags.is_empty() {
                    t.push_str(&format!("\nTags:    {}", tags));
                }
            }
            if let Some(ref country) = s.country {
                if !country.is_empty() {
                    t.push_str(&format!("\nCountry: {}", country));
                }
            }
            if let Some(ref codec) = s.codec {
                if !codec.is_empty() {
                    t.push_str(&format!("\nCodec:   {}", codec));
                }
            }
            if let Some(br) = s.bitrate {
                if br > 0 {
                    t.push_str(&format!("\nBitrate: {}kbps", br));
                }
            }
            if let Some(cc) = s.clickcount {
                t.push_str(&format!("\nClicks:  {}", cc));
            }
            if let Some(v) = s.votes {
                t.push_str(&format!("\nVotes:   {}", v));
            }
            t
        } else {
            String::new()
        }
    } else {
        "Select a station with j/k to see details".to_string()
    };

    f.render_widget(
        Paragraph::new(text)
            .style(Style::default().fg(Color::White))
            .block(block),
        area,
    );
}
