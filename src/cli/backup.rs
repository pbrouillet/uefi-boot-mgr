use std::fs;

use crate::core::BootManager;
use crate::core::backup;
use crate::error::AppError;

pub fn run(mgr: &BootManager, file: &str) -> Result<(), AppError> {
    let data = backup::export(mgr)?;
    let json = serde_json::to_string_pretty(&data)?;

    fs::write(file, &json)?;

    println!("Backup saved to: {file}");
    println!("  Entries: {}", data.entries.len());
    println!("  Boot order: {:?}", data.boot_order);

    Ok(())
}
