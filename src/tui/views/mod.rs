pub mod entry_list;
pub mod entry_detail;
pub mod entry_form;
pub mod backup;
pub mod help;
pub mod wizard;

use ratatui::prelude::*;

use super::app::{App, View};
use super::widgets;

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Min(0),   // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

    // Title bar
    widgets::draw_title_bar(frame, chunks[0], app);

    // Main content depends on current view
    match app.view {
        View::EntryList => entry_list::draw(frame, chunks[1], app),
        View::EntryDetail => {
            entry_list::draw(frame, chunks[1], app);
            entry_detail::draw(frame, chunks[1], app);
        }
        View::EntryForm => {
            entry_list::draw(frame, chunks[1], app);
            entry_form::draw(frame, chunks[1], app);
        }
        View::BackupRestore => {
            entry_list::draw(frame, chunks[1], app);
            backup::draw(frame, chunks[1], app);
        }
        View::Help => {
            entry_list::draw(frame, chunks[1], app);
            help::draw(frame, chunks[1]);
        }
        View::Confirm => {
            entry_list::draw(frame, chunks[1], app);
            widgets::draw_confirm(frame, chunks[1], app);
        }
        View::Wizard => {
            entry_list::draw(frame, chunks[1], app);
            wizard::draw(frame, chunks[1], app);
        }
    }

    // Status bar
    widgets::draw_status_bar(frame, chunks[2], app);
}
