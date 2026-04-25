use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

pub fn draw(frame: &mut Frame, area: Rect) {
    let popup = centered_popup(area, 55, 70);
    frame.render_widget(Clear, popup);

    let bindings = vec![
        ("↑/k, ↓/j", "Navigate entries"),
        ("Enter", "View entry details"),
        ("n", "New boot entry"),
        ("e", "Edit selected entry"),
        ("d", "Delete selected entry"),
        ("Space", "Toggle active/inactive"),
        ("o", "Reorder mode (↑↓ to move)"),
        ("b", "Backup entries to file"),
        ("r", "Restore entries from file"),
        ("R", "Refresh entries"),
        ("?", "Show this help"),
        ("q / Esc", "Quit"),
        ("Ctrl+C", "Force quit"),
    ];

    let text: Vec<Line> = bindings
        .iter()
        .map(|(key, desc)| {
            Line::from(vec![
                Span::styled(
                    format!("{key:>14}"),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::raw(*desc),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Keybindings ")
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
