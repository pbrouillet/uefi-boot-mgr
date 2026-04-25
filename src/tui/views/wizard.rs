use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState};

use crate::tui::app::{App, WizardTemplate};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let popup = centered_popup(area, 55, 60);
    frame.render_widget(Clear, popup);

    let items: Vec<ListItem> = WizardTemplate::ALL
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let marker = if i == app.wizard_selected { "▸ " } else { "  " };
            let style = if i == app.wizard_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(marker, style),
                Span::styled(t.label(), style),
            ]))
        })
        .collect();

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(0),    // Template list
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Preview
            Constraint::Length(2), // Help text
        ])
        .split(popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Boot Entry Wizard ")
        .border_style(Style::default().fg(Color::Magenta));
    frame.render_widget(block, popup);

    let mut list_state = ListState::default().with_selected(Some(app.wizard_selected));
    let list = List::new(items).highlight_style(Style::default());
    frame.render_stateful_widget(list, inner[0], &mut list_state);

    // Preview of selected template
    if let Some(template) = WizardTemplate::ALL.get(app.wizard_selected) {
        let preview = ratatui::widgets::Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Desc:   ", Style::default().fg(Color::DarkGray)),
                Span::raw(template.description()),
            ]),
            Line::from(vec![
                Span::styled("Loader: ", Style::default().fg(Color::DarkGray)),
                Span::raw(template.loader()),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(preview, inner[2]);
    }

    let help = ratatui::widgets::Paragraph::new(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(" select  "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" use template  "),
        Span::styled("Esc", Style::default().fg(Color::Yellow)),
        Span::raw(" cancel"),
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
