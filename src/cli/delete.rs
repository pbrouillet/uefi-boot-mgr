use dialoguer::Confirm;

use crate::core::BootManager;
use crate::error::{parse_boot_id, AppError};

pub fn run(mgr: &mut BootManager, id_str: &str, force: bool) -> Result<(), AppError> {
    let id = parse_boot_id(id_str)?;
    let entry = mgr.get_entry(id)?;

    if !force {
        let prompt = format!(
            "Delete Boot{:04X} ({})? This cannot be undone.",
            id, entry.description
        );
        if !Confirm::new().with_prompt(&prompt).default(false).interact()? {
            println!("Cancelled.");
            return Ok(());
        }
    }

    mgr.delete_entry(id)?;
    println!("Deleted Boot{id:04X} ({})", entry.description);

    Ok(())
}
