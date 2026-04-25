use comfy_table::Table;

use crate::core::esp;
use crate::error::AppError;

pub fn run_list(json: bool) -> Result<(), AppError> {
    let partitions = esp::list_partitions()?;

    if json {
        let output = serde_json::to_string_pretty(&partitions)?;
        println!("{output}");
        return Ok(());
    }

    if partitions.is_empty() {
        println!("No partitions found.");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec!["Disk", "Part#", "Type GUID", "ESP", "Label", "Size", "Mount"]);

    for p in &partitions {
        let esp_marker = if p.is_esp { "✓" } else { "" };
        table.add_row(vec![
            &p.disk,
            &p.partition_number.to_string(),
            p.type_guid.as_deref().unwrap_or("-"),
            esp_marker,
            p.label.as_deref().unwrap_or("-"),
            p.size.as_deref().unwrap_or("-"),
            p.mountpoint.as_deref().unwrap_or("-"),
        ]);
    }

    println!("{table}");
    Ok(())
}

pub fn run_set(disk: &str, partition: u32) -> Result<(), AppError> {
    esp::set_esp_flag(disk, partition)?;
    println!("Set ESP flag on {disk} partition {partition}");
    println!("Type GUID → {}", esp::ESP_TYPE_GUID);
    Ok(())
}

pub fn run_clear(disk: &str, partition: u32) -> Result<(), AppError> {
    esp::clear_esp_flag(disk, partition)?;
    println!("Cleared ESP flag on {disk} partition {partition}");
    println!("Type GUID → {}", esp::BASIC_DATA_TYPE_GUID);
    Ok(())
}
