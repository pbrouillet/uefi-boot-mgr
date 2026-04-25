use std::fs;

use dialoguer::Confirm;

use crate::core::BootManager;
use crate::core::backup::{self, BackupData};
use crate::error::AppError;

pub fn run(mgr: &mut BootManager, file: &str, force: bool) -> Result<(), AppError> {
    let contents = fs::read_to_string(file).map_err(|e| AppError::Backup {
        message: format!("Failed to read backup file '{file}': {e}"),
    })?;

    let data: BackupData = serde_json::from_str(&contents)?;

    // Validate before anything else
    backup::validate(&data)?;

    println!("Backup file: {file}");
    println!("  Version:    {}", data.version);
    println!("  Timestamp:  {}", data.timestamp);
    println!("  Entries:    {}", data.entries.len());
    println!("  Boot order: {:?}", data.boot_order);
    println!();

    for entry in &data.entries {
        let desc = entry
            .decoded
            .as_ref()
            .map(|d| d.description.as_str())
            .unwrap_or("(unknown)");
        println!("  {} — {desc}", entry.variable_name);
    }
    println!();

    if !force {
        if !Confirm::new()
            .with_prompt("Restore these boot entries? This will overwrite existing entries.")
            .default(false)
            .interact()?
        {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Auto-backup current state before restore
    let auto_backup_path = format!("{file}.pre-restore.json");
    if let Ok(current) = backup::export(mgr) {
        if let Ok(json) = serde_json::to_string_pretty(&current) {
            if fs::write(&auto_backup_path, &json).is_ok() {
                println!("Auto-backup of current state saved to: {auto_backup_path}");
            }
        }
    }

    backup::restore(mgr, &data)?;
    println!("Restore complete.");

    Ok(())
}
