use comfy_table::{Cell, Color, Table};

use crate::core::BootManager;
use crate::error::AppError;

pub fn run(mgr: &BootManager, json: bool) -> Result<(), AppError> {
    let entries = mgr.list_entries()?;
    let boot_current = mgr.get_boot_current()?;
    let boot_next = mgr.get_boot_next()?;

    if json {
        let output = serde_json::to_string_pretty(&entries)?;
        println!("{output}");
        return Ok(());
    }

    if entries.is_empty() {
        println!("No boot entries found.");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec!["", "ID", "Description", "Active", "File Path"]);

    for entry in &entries {
        let mut marker = String::new();
        if boot_current == Some(entry.id) {
            marker.push('*');
        }
        if boot_next == Some(entry.id) {
            marker.push('N');
        }

        let active_str = if entry.active { "Yes" } else { "No" };
        let path = entry.file_path.as_deref().unwrap_or("-");

        table.add_row(vec![
            Cell::new(&marker),
            Cell::new(format!("{:04X}", entry.id)),
            Cell::new(&entry.description),
            Cell::new(active_str).fg(if entry.active {
                Color::Green
            } else {
                Color::DarkGrey
            }),
            Cell::new(path),
        ]);
    }

    println!("{table}");
    println!();
    println!("  * = BootCurrent    N = BootNext");

    Ok(())
}
