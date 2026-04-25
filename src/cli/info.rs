use crate::core::BootManager;
use crate::error::{parse_boot_id, AppError};

pub fn run(mgr: &BootManager, id_str: &str, json: bool) -> Result<(), AppError> {
    let id = parse_boot_id(id_str)?;
    let entry = mgr.get_entry(id)?;

    if json {
        let output = serde_json::to_string_pretty(&entry)?;
        println!("{output}");
        return Ok(());
    }

    println!("Boot Entry {:04X}", entry.id);
    println!("  Description:  {}", entry.description);
    println!("  Active:       {}", if entry.active { "Yes" } else { "No" });
    println!(
        "  File Path:    {}",
        entry.file_path.as_deref().unwrap_or("(none)")
    );
    println!(
        "  Partition:    {}",
        entry.partition_guid.as_deref().unwrap_or("(none)")
    );
    println!("  Device Path:  {}", entry.device_path_display);
    println!("  Raw Size:     {} bytes", entry.raw_bytes.len());

    // Hex dump of first 64 bytes
    println!("  Raw (hex):    {}", hex_preview(&entry.raw_bytes, 64));

    Ok(())
}

fn hex_preview(data: &[u8], max: usize) -> String {
    let truncated = data.len() > max;
    let bytes = &data[..data.len().min(max)];
    let hex: Vec<String> = bytes.iter().map(|b| format!("{b:02x}")).collect();
    let s = hex.join(" ");
    if truncated {
        format!("{s} ...")
    } else {
        s
    }
}
