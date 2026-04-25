use serde::Serialize;
use std::process::Command;

use crate::error::AppError;

/// ESP partition type GUID
pub const ESP_TYPE_GUID: &str = "c12a7328-f81f-11d2-ba4b-00a0c93ec93b";
/// Basic data partition type GUID (used when clearing ESP flag)
pub const BASIC_DATA_TYPE_GUID: &str = "ebd0a0a2-b9e5-4433-87c0-68b6b72699c7";

/// Information about a disk partition
#[derive(Debug, Clone, Serialize)]
pub struct PartitionInfo {
    pub disk: String,
    pub partition_number: u32,
    pub type_guid: Option<String>,
    pub is_esp: bool,
    pub label: Option<String>,
    pub size: Option<String>,
    pub mountpoint: Option<String>,
}

/// List all GPT partitions on the system.
pub fn list_partitions() -> Result<Vec<PartitionInfo>, AppError> {
    #[cfg(target_os = "windows")]
    {
        list_partitions_windows()
    }

    #[cfg(target_os = "linux")]
    {
        list_partitions_linux()
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err(AppError::Efi {
            message: "Partition listing is not supported on this platform".to_string(),
        })
    }
}

/// Set a partition's type GUID to ESP.
pub fn set_esp_flag(disk: &str, partition_number: u32) -> Result<(), AppError> {
    #[cfg(target_os = "windows")]
    {
        set_partition_type_windows(disk, partition_number, ESP_TYPE_GUID)
    }

    #[cfg(target_os = "linux")]
    {
        set_partition_type_linux(disk, partition_number, "EF00")
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err(AppError::Efi {
            message: "ESP flag management is not supported on this platform".to_string(),
        })
    }
}

/// Clear the ESP flag (revert to Basic Data partition).
pub fn clear_esp_flag(disk: &str, partition_number: u32) -> Result<(), AppError> {
    #[cfg(target_os = "windows")]
    {
        set_partition_type_windows(disk, partition_number, BASIC_DATA_TYPE_GUID)
    }

    #[cfg(target_os = "linux")]
    {
        set_partition_type_linux(disk, partition_number, "0700")
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err(AppError::Efi {
            message: "ESP flag management is not supported on this platform".to_string(),
        })
    }
}

// --- Windows implementation ---

#[cfg(target_os = "windows")]
fn list_partitions_windows() -> Result<Vec<PartitionInfo>, AppError> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            r#"Get-Partition | Select-Object DiskNumber, PartitionNumber, GptType, Size, DriveLetter, Type | ConvertTo-Json -Compress"#,
        ])
        .output()
        .map_err(|e| AppError::Efi {
            message: format!("Failed to run PowerShell: {e}"),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Efi {
            message: format!("PowerShell Get-Partition failed: {stderr}"),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_windows_partitions(&stdout)
}

#[cfg(target_os = "windows")]
fn parse_windows_partitions(json_str: &str) -> Result<Vec<PartitionInfo>, AppError> {
    let trimmed = json_str.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    // PowerShell returns a single object (not array) if there's only one partition
    let raw: serde_json::Value = serde_json::from_str(trimmed).map_err(|e| AppError::Parse {
        message: format!("Failed to parse partition JSON: {e}"),
    })?;

    let items = match raw {
        serde_json::Value::Array(arr) => arr,
        obj @ serde_json::Value::Object(_) => vec![obj],
        _ => return Ok(Vec::new()),
    };

    let mut partitions = Vec::new();
    for item in items {
        let disk_num = item["DiskNumber"].as_u64().unwrap_or(0);
        let part_num = item["PartitionNumber"].as_u64().unwrap_or(0) as u32;
        let gpt_type = item["GptType"]
            .as_str()
            .map(|s| s.trim_matches('{').trim_matches('}').to_lowercase());
        let size = item["Size"].as_u64().map(|s| format_size(s));
        let drive_letter = item["DriveLetter"]
            .as_str()
            .and_then(|s| if s.is_empty() || s == "\u{0}" { None } else { Some(format!("{s}:")) });
        let part_type = item["Type"].as_str().map(String::from);

        let is_esp = gpt_type
            .as_ref()
            .is_some_and(|g| g == ESP_TYPE_GUID);

        partitions.push(PartitionInfo {
            disk: format!("Disk {disk_num}"),
            partition_number: part_num,
            type_guid: gpt_type,
            is_esp,
            label: part_type,
            size,
            mountpoint: drive_letter,
        });
    }

    Ok(partitions)
}

#[cfg(target_os = "windows")]
fn set_partition_type_windows(
    disk: &str,
    partition_number: u32,
    type_guid: &str,
) -> Result<(), AppError> {
    // Extract disk number from "Disk N" format
    let disk_num = disk
        .trim_start_matches("Disk ")
        .trim_start_matches("disk ")
        .parse::<u32>()
        .map_err(|_| AppError::Parse {
            message: format!("Invalid disk identifier: {disk}. Expected format: 'Disk N'"),
        })?;

    let cmd = format!(
        "Set-Partition -DiskNumber {disk_num} -PartitionNumber {partition_number} -GptType '{{{type_guid}}}'"
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &cmd])
        .output()
        .map_err(|e| AppError::Efi {
            message: format!("Failed to run PowerShell: {e}"),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Efi {
            message: format!("Set-Partition failed: {stderr}"),
        });
    }

    Ok(())
}

// --- Linux implementation ---

#[cfg(target_os = "linux")]
fn list_partitions_linux() -> Result<Vec<PartitionInfo>, AppError> {
    let output = Command::new("lsblk")
        .args(["--json", "-o", "NAME,PARTTYPE,SIZE,MOUNTPOINT,LABEL", "-p"])
        .output()
        .map_err(|e| AppError::Efi {
            message: format!("Failed to run lsblk: {e}"),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Efi {
            message: format!("lsblk failed: {stderr}"),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_linux_partitions(&stdout)
}

#[cfg(target_os = "linux")]
fn parse_linux_partitions(json_str: &str) -> Result<Vec<PartitionInfo>, AppError> {
    let raw: serde_json::Value = serde_json::from_str(json_str).map_err(|e| AppError::Parse {
        message: format!("Failed to parse lsblk JSON: {e}"),
    })?;

    let mut partitions = Vec::new();
    if let Some(devices) = raw["blockdevices"].as_array() {
        for device in devices {
            let parent_name = device["name"].as_str().unwrap_or("");
            // Process child partitions
            if let Some(children) = device["children"].as_array() {
                for (idx, child) in children.iter().enumerate() {
                    let name = child["name"].as_str().unwrap_or("");
                    let parttype = child["parttype"]
                        .as_str()
                        .map(|s| s.to_lowercase());
                    let size = child["size"].as_str().map(String::from);
                    let mountpoint = child["mountpoint"]
                        .as_str()
                        .and_then(|s| if s.is_empty() { None } else { Some(s.to_string()) });
                    let label = child["label"]
                        .as_str()
                        .and_then(|s| if s.is_empty() { None } else { Some(s.to_string()) });

                    let is_esp = parttype
                        .as_ref()
                        .is_some_and(|g| g == ESP_TYPE_GUID);

                    partitions.push(PartitionInfo {
                        disk: parent_name.to_string(),
                        partition_number: (idx + 1) as u32,
                        type_guid: parttype,
                        is_esp,
                        label,
                        size,
                        mountpoint,
                    });
                }
            }
        }
    }

    Ok(partitions)
}

#[cfg(target_os = "linux")]
fn set_partition_type_linux(
    disk: &str,
    partition_number: u32,
    type_code: &str,
) -> Result<(), AppError> {
    let output = Command::new("sgdisk")
        .args([
            &format!("--typecode={partition_number}:{type_code}"),
            disk,
        ])
        .output()
        .map_err(|e| AppError::Efi {
            message: format!("Failed to run sgdisk: {e}"),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Efi {
            message: format!("sgdisk failed: {stderr}"),
        });
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1023), "1023 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_parse_windows_single_partition() {
        let json = r#"{"DiskNumber":0,"PartitionNumber":1,"GptType":"{c12a7328-f81f-11d2-ba4b-00a0c93ec93b}","Size":104857600,"DriveLetter":"","Type":"System"}"#;
        let parts = parse_windows_partitions(json).unwrap();
        assert_eq!(parts.len(), 1);
        assert!(parts[0].is_esp);
        assert_eq!(parts[0].partition_number, 1);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_parse_windows_multiple_partitions() {
        let json = r#"[
            {"DiskNumber":0,"PartitionNumber":1,"GptType":"{c12a7328-f81f-11d2-ba4b-00a0c93ec93b}","Size":104857600,"DriveLetter":"","Type":"System"},
            {"DiskNumber":0,"PartitionNumber":2,"GptType":"{ebd0a0a2-b9e5-4433-87c0-68b6b72699c7}","Size":536870912000,"DriveLetter":"C","Type":"Basic"}
        ]"#;
        let parts = parse_windows_partitions(json).unwrap();
        assert_eq!(parts.len(), 2);
        assert!(parts[0].is_esp);
        assert!(!parts[1].is_esp);
        assert_eq!(parts[1].mountpoint.as_deref(), Some("C:"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_parse_windows_empty() {
        let parts = parse_windows_partitions("").unwrap();
        assert!(parts.is_empty());
    }
}
