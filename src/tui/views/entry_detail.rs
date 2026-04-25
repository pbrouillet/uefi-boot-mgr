use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::tui::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let entry = match app.selected_entry() {
        Some(e) => e,
        None => return,
    };

    let popup = centered_popup(area, 70, 60);
    frame.render_widget(Clear, popup);

    let hex_preview: String = entry
        .raw_bytes
        .iter()
        .take(64)
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ");
    let truncated = if entry.raw_bytes.len() > 64 { " ..." } else { "" };

    let text = vec![
        Line::from(vec![
            Span::styled("ID:           ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{:04X}", entry.id)),
        ]),
        Line::from(vec![
            Span::styled("Description:  ", Style::default().fg(Color::Yellow)),
            Span::raw(&entry.description),
        ]),
        Line::from(vec![
            Span::styled("Active:       ", Style::default().fg(Color::Yellow)),
            Span::styled(
                if entry.active { "Yes" } else { "No" },
                if entry.active {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                },
            ),
        ]),
        Line::from(vec![
            Span::styled("File Path:    ", Style::default().fg(Color::Yellow)),
            Span::raw(entry.file_path.as_deref().unwrap_or("(none)")),
        ]),
        Line::from(vec![
            Span::styled("Partition:    ", Style::default().fg(Color::Yellow)),
            Span::raw(entry.partition_guid.as_deref().unwrap_or("(none)")),
        ]),
        Line::from(vec![
            Span::styled("Device Path:  ", Style::default().fg(Color::Yellow)),
            Span::raw(&entry.device_path_display),
        ]),
        Line::from(vec![
            Span::styled("Raw Size:     ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{} bytes", entry.raw_bytes.len())),
        ]),
        Line::from(""),
        Line::from(Span::styled("Raw (hex):", Style::default().fg(Color::Yellow))),
        Line::from(format!("{hex_preview}{truncated}")),
        Line::from(""),
        Line::from(Span::styled(
            "Press Esc or Enter to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Boot{:04X} Detail ", entry.id))
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(paragraph, popup);
}

fn centered_popup(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
