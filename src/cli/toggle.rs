use crate::core::BootManager;
use crate::error::{parse_boot_id, AppError};

pub fn run_enable(mgr: &mut BootManager, id_str: &str) -> Result<(), AppError> {
    let id = parse_boot_id(id_str)?;
    let entry = mgr.get_entry(id)?;

    if entry.active {
        println!("Boot{id:04X} ({}) is already enabled.", entry.description);
        return Ok(());
    }

    mgr.toggle_active(id)?;
    println!("Enabled Boot{id:04X} ({})", entry.description);

    Ok(())
}

pub fn run_disable(mgr: &mut BootManager, id_str: &str) -> Result<(), AppError> {
    let id = parse_boot_id(id_str)?;
    let entry = mgr.get_entry(id)?;

    if !entry.active {
        println!("Boot{id:04X} ({}) is already disabled.", entry.description);
        return Ok(());
    }

    mgr.toggle_active(id)?;
    println!("Disabled Boot{id:04X} ({})", entry.description);

    Ok(())
}
