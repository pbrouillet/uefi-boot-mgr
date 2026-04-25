use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::tui::app::{App, FormField, FormMode};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let popup = centered_popup(area, 60, 50);
    frame.render_widget(Clear, popup);

    let title = match app.form_mode {
        FormMode::Create => " New Boot Entry ",
        FormMode::Edit => " Edit Boot Entry ",
    };

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Description
            Constraint::Length(3), // Loader
            Constraint::Length(3), // Partition
            Constraint::Length(2), // Help text
        ])
        .split(popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Magenta));
    frame.render_widget(block, popup);

    draw_input_field(frame, inner[0], "Description", &app.form_description, app.form_field == FormField::Description);
    draw_input_field(frame, inner[1], "Loader Path", &app.form_loader, app.form_field == FormField::Loader);
    draw_input_field(frame, inner[2], "Partition GUID (optional)", &app.form_partition, app.form_field == FormField::Partition);

    let help = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(" next field  "),
        Span::styled("Ctrl+Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" save  "),
        Span::styled("Esc", Style::default().fg(Color::Yellow)),
        Span::raw(" cancel"),
    ]));
    frame.render_widget(help, inner[3]);
}

fn draw_input_field(frame: &mut Frame, area: Rect, label: &str, value: &str, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let display_value = if value.is_empty() && !focused {
        "(empty)"
    } else {
        value
    };

    let cursor_suffix = if focused { "▏" } else { "" };

    let paragraph = Paragraph::new(format!("{display_value}{cursor_suffix}")).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {label} "))
            .border_style(border_style),
    );
    frame.render_widget(paragraph, area);
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
