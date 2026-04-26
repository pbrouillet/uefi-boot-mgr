use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Clear, Row, Table};

use crate::tui::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let popup = centered_popup(area, 80, 70);
    frame.render_widget(Clear, popup);

    let esp_path = app
        .bootloader_esp_path
        .as_deref()
        .unwrap_or("unknown");

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // ESP path info
            Constraint::Length(1), // Spacer
            Constraint::Min(0),   // Table
            Constraint::Length(2), // Help text
        ])
        .split(popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" ESP Bootloaders ")
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, popup);

    // ESP path line
    let path_line = ratatui::widgets::Paragraph::new(Line::from(vec![
        Span::styled("ESP: ", Style::default().fg(Color::DarkGray)),
        Span::styled(esp_path, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]));
    frame.render_widget(path_line, inner[0]);

    if app.bootloader_entries.is_empty() {
        let empty = ratatui::widgets::Paragraph::new(
            "No known bootloaders found on the ESP."
        )
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
        frame.render_widget(empty, inner[2]);
    } else {
        let header = Row::new(vec!["", "Identity", "Path", "Size", "Modified"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

        let rows: Vec<Row> = app
            .bootloader_entries
            .iter()
            .map(|loader| {
                let marker = if loader.is_default { "★" } else { "" };
                let style = if loader.is_default {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default()
                };
                Row::new(vec![
                    Cell::from(marker).style(Style::default().fg(Color::Yellow)),
                    Cell::from(loader.identity.as_str()).style(style),
                    Cell::from(loader.path.as_str()),
                    Cell::from(loader.size.as_deref().unwrap_or("-")),
                    Cell::from(loader.modified.as_deref().unwrap_or("-")),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(2),
                Constraint::Percentage(25),
                Constraint::Percentage(35),
                Constraint::Length(10),
                Constraint::Length(20),
            ],
        )
        .header(header);

        frame.render_widget(table, inner[2]);
    }

    // Help text
    let help = ratatui::widgets::Paragraph::new(Line::from(vec![
        Span::styled("Esc", Style::default().fg(Color::Yellow)),
        Span::raw(" close  "),
        Span::styled("★", Style::default().fg(Color::Yellow)),
        Span::raw(" = UEFI default fallback"),
    ]));
    frame.render_widget(help, inner[3]);
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
