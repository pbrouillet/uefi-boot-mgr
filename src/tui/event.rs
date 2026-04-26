use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use std::time::Duration;

use super::app::{App, FormField, View};
use super::views;

pub fn run_event_loop(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|frame| views::draw(frame, app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != event::KeyEventKind::Press {
                    continue;
                }
                handle_key(app, key);
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn handle_key(app: &mut App, key: KeyEvent) {
    // Ctrl+C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.should_quit = true;
        return;
    }

    match app.view {
        View::EntryList => handle_entry_list_key(app, key),
        View::EntryDetail => handle_detail_key(app, key),
        View::EntryForm => handle_form_key(app, key),
        View::BackupRestore => handle_backup_key(app, key),
        View::Help => handle_help_key(app, key),
        View::Confirm => handle_confirm_key(app, key),
        View::Wizard => handle_wizard_key(app, key),
    }
}

fn handle_entry_list_key(app: &mut App, key: KeyEvent) {
    if app.reorder_mode {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => app.move_selected_up(),
            KeyCode::Down | KeyCode::Char('j') => app.move_selected_down(),
            KeyCode::Esc | KeyCode::Char('o') => {
                app.reorder_mode = false;
                app.set_status("Reorder mode off");
            }
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected > 0 {
                app.selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.selected + 1 < app.entries.len() {
                app.selected += 1;
            }
        }
        KeyCode::Home => app.selected = 0,
        KeyCode::End => {
            if !app.entries.is_empty() {
                app.selected = app.entries.len() - 1;
            }
        }
        KeyCode::Enter => {
            if app.selected_entry().is_some() {
                app.view = View::EntryDetail;
            }
        }
        KeyCode::Char('n') => app.open_create_form(),
        KeyCode::Char('w') => app.open_wizard(),
        KeyCode::Char('e') => app.open_edit_form(),
        KeyCode::Char('d') => app.delete_selected(),
        KeyCode::Char(' ') => app.toggle_selected_active(),
        KeyCode::Char('o') => {
            app.reorder_mode = true;
            app.set_status("Reorder mode: ↑↓ to move, Esc to finish");
        }
        KeyCode::Char('b') => app.open_backup(),
        KeyCode::Char('r') => app.open_restore(),
        KeyCode::Char('R') => {
            app.refresh_entries();
            app.set_status("Refreshed");
        }
        KeyCode::Char('?') => app.view = View::Help,
        _ => {}
    }
}

fn handle_detail_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => app.view = View::EntryList,
        _ => {}
    }
}

fn handle_form_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.view = View::EntryList,
        KeyCode::Tab => app.form_field = app.form_field.next(),
        KeyCode::BackTab => app.form_field = app.form_field.prev(),
        KeyCode::Enter => {
            if key.modifiers.contains(KeyModifiers::CONTROL) || app.form_field == FormField::Partition {
                app.submit_form();
            } else {
                app.form_field = app.form_field.next();
            }
        }
        KeyCode::Backspace => {
            let field = active_field_mut(app);
            field.pop();
        }
        KeyCode::Char(c) => {
            let field = active_field_mut(app);
            field.push(c);
        }
        _ => {}
    }
}

fn active_field_mut(app: &mut App) -> &mut String {
    match app.form_field {
        FormField::Description => &mut app.form_description,
        FormField::Loader => &mut app.form_loader,
        FormField::Partition => &mut app.form_partition,
    }
}

fn handle_backup_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.view = View::EntryList,
        KeyCode::Enter => app.submit_backup_restore(),
        KeyCode::Backspace => {
            app.backup_path.pop();
        }
        KeyCode::Char(c) => {
            app.backup_path.push(c);
        }
        _ => {}
    }
}

fn handle_help_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => app.view = View::EntryList,
        _ => {}
    }
}

fn handle_confirm_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => app.execute_confirm(),
        KeyCode::Char('n') | KeyCode::Esc => app.cancel_confirm(),
        _ => {}
    }
}

fn handle_wizard_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => app.view = View::EntryList,
        KeyCode::Up | KeyCode::Char('k') => {
            if app.wizard_selected > 0 {
                app.wizard_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.wizard_selected + 1 < app.wizard_templates.len() {
                app.wizard_selected += 1;
            }
        }
        KeyCode::Enter => app.apply_wizard_template(app.wizard_selected),
        _ => {}
    }
}
