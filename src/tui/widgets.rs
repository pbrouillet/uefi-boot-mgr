use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use super::app::App;

pub fn draw_title_bar(frame: &mut Frame, area: Rect, app: &App) {
    let mode = if app.reorder_mode {
        " REORDER "
    } else {
        match app.view {
            super::app::View::EntryList => "",
            super::app::View::EntryDetail => " DETAIL ",
            super::app::View::EntryForm => " FORM ",
            super::app::View::BackupRestore => " BACKUP ",
            super::app::View::Help => " HELP ",
            super::app::View::Confirm => " CONFIRM ",
            super::app::View::Wizard => " WIZARD ",
        }
    };

    let title = Line::from(vec![
        Span::styled(
            " uefibootmgrrs ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(mode, Style::default().fg(Color::Yellow)),
        Span::raw("  "),
        Span::styled("? for help", Style::default().fg(Color::DarkGray)),
    ]);

    frame.render_widget(Paragraph::new(title).style(Style::default().bg(Color::Black)), area);
}

pub fn draw_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let content = match &app.status_message {
        Some(msg) => {
            let style = if app.status_is_error {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Green)
            };
            Line::from(Span::styled(format!(" {msg}"), style))
        }
        None => {
            let entry_count = app.entries.len();
            Line::from(Span::styled(
                format!(" {entry_count} entries | q quit | ? help"),
                Style::default().fg(Color::DarkGray),
            ))
        }
    };

    frame.render_widget(
        Paragraph::new(content).style(Style::default().bg(Color::Black)),
        area,
    );
}

pub fn draw_confirm(frame: &mut Frame, area: Rect, app: &App) {
    if let Some(ref confirm) = app.confirm {
        let popup = centered_popup(area, 50, 25);
        frame.render_widget(Clear, popup);

        let text = vec![
            Line::from(""),
            Line::from(Span::raw(&confirm.message)),
            Line::from(""),
            Line::from(vec![
                Span::styled("y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" confirm  "),
                Span::styled("n", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(" cancel"),
            ]),
        ];

        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Confirm ")
                    .border_style(Style::default().fg(Color::Red)),
            );

        frame.render_widget(paragraph, popup);
    }
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
