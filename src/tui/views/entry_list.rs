use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};

use crate::tui::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let header = Row::new(vec!["", "ID", "Description", "Active", "File Path"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app
        .entries
        .iter()
        .map(|entry| {
            let mut marker = String::new();
            if app.boot_current == Some(entry.id) {
                marker.push('*');
            }
            if app.boot_next == Some(entry.id) {
                marker.push('N');
            }

            let active_style = if entry.active {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            Row::new(vec![
                Cell::from(marker),
                Cell::from(format!("{:04X}", entry.id)),
                Cell::from(entry.description.as_str()),
                Cell::from(if entry.active { "Yes" } else { "No" }).style(active_style),
                Cell::from(entry.file_path.as_deref().unwrap_or("-")),
            ])
        })
        .collect();

    let mode_hint = if app.reorder_mode {
        " [REORDER] "
    } else {
        ""
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Length(6),
            Constraint::Percentage(35),
            Constraint::Length(8),
            Constraint::Percentage(40),
        ],
    )
    .header(header)
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ")
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Boot Entries ({}) {mode_hint}", app.entries.len()))
            .border_style(if app.reorder_mode {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Cyan)
            }),
    );

    let mut state = TableState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(table, area, &mut state);
}
