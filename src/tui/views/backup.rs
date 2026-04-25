use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::tui::app::{App, BackupMode};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let popup = centered_popup(area, 55, 30);
    frame.render_widget(Clear, popup);

    let title = match app.backup_mode {
        BackupMode::Backup => " Backup to File ",
        BackupMode::Restore => " Restore from File ",
    };

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Path input
            Constraint::Length(2), // Help text
        ])
        .split(popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, popup);

    let path_display = format!("{}▏", app.backup_path);
    let path_input = Paragraph::new(path_display).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" File Path ")
            .border_style(Style::default().fg(Color::Yellow)),
    );
    frame.render_widget(path_input, inner[0]);

    let action = match app.backup_mode {
        BackupMode::Backup => "export",
        BackupMode::Restore => "restore",
    };
    let help = Paragraph::new(Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(format!(" {action}  ")),
        Span::styled("Esc", Style::default().fg(Color::Yellow)),
        Span::raw(" cancel"),
    ]));
    frame.render_widget(help, inner[1]);
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
