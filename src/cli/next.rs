use crate::core::BootManager;
use crate::error::{parse_boot_id, AppError};

pub fn run(mgr: &mut BootManager, id_str: &str) -> Result<(), AppError> {
    let id = parse_boot_id(id_str)?;

    // Verify the entry exists
    let entry = mgr.get_entry(id)?;

    mgr.set_boot_next(id)?;
    println!(
        "BootNext set to Boot{:04X} ({})",
        id, entry.description
    );
    println!("The system will boot this entry on next reboot only.");

    Ok(())
}
