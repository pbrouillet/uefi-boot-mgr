use efivar::boot::BootEntry;

use crate::core::BootManager;
use crate::error::{parse_boot_id, AppError};

pub fn run(
    mgr: &mut BootManager,
    id_str: &str,
    description: Option<&str>,
    loader: Option<&str>,
) -> Result<(), AppError> {
    let id = parse_boot_id(id_str)?;
    let info = mgr.get_entry(id)?;

    let mut entry = BootEntry::parse(info.raw_bytes).map_err(|e| AppError::Parse {
        message: format!("Failed to parse Boot{id:04X}: {e}"),
    })?;

    let mut changed = false;

    if let Some(desc) = description {
        entry.description = desc.to_string();
        changed = true;
    }

    if let Some(path) = loader {
        if let Some(ref mut fpl) = entry.file_path_list {
            fpl.file_path.path = path.to_string();
            changed = true;
        }
    }

    if !changed {
        println!("No changes specified.");
        return Ok(());
    }

    mgr.update_entry(id, entry)?;
    println!("Updated Boot{id:04X}");

    Ok(())
}
