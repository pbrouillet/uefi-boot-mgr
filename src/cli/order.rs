use crate::core::BootManager;
use crate::error::{parse_boot_id, AppError};

pub fn run(mgr: &mut BootManager, ids_str: &str) -> Result<(), AppError> {
    let ids: Result<Vec<u16>, _> = ids_str.split(',').map(|s| parse_boot_id(s.trim())).collect();
    let ids = ids?;

    mgr.set_boot_order(ids.clone())?;

    let formatted: Vec<String> = ids.iter().map(|id| format!("{id:04X}")).collect();
    println!("Boot order set to: {}", formatted.join(", "));

    Ok(())
}
