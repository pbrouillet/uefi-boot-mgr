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

// --- ESP bootloader scanning ---

/// Information about a bootloader found on the ESP.
#[derive(Debug, Clone, Serialize)]
pub struct BootloaderInfo {
    /// Path relative to ESP root (e.g., `\EFI\BOOT\BOOTX64.EFI`)
    pub path: String,
    /// Identified OS or role (e.g., "UEFI Default Fallback", "Windows", "Ubuntu")
    pub identity: String,
    /// Whether this is the UEFI default fallback bootloader
    pub is_default: bool,
    /// File size in human-readable form
    pub size: Option<String>,
    /// Last modified timestamp
    pub modified: Option<String>,
}

/// Well-known EFI bootloader paths and their identities.
const KNOWN_LOADERS: &[(&str, &str, bool)] = &[
    (r"EFI\BOOT\BOOTX64.EFI", "UEFI Default Fallback (x64)", true),
    (r"EFI\BOOT\BOOTIA32.EFI", "UEFI Default Fallback (IA-32)", true),
    (r"EFI\BOOT\BOOTAA64.EFI", "UEFI Default Fallback (ARM64)", true),
    (r"EFI\Microsoft\Boot\bootmgfw.efi", "Windows Boot Manager", false),
    (r"EFI\ubuntu\shimx64.efi", "Ubuntu (shim)", false),
    (r"EFI\ubuntu\grubx64.efi", "Ubuntu (GRUB)", false),
    (r"EFI\fedora\shimx64.efi", "Fedora (shim)", false),
    (r"EFI\fedora\grubx64.efi", "Fedora (GRUB)", false),
    (r"EFI\debian\shimx64.efi", "Debian (shim)", false),
    (r"EFI\debian\grubx64.efi", "Debian (GRUB)", false),
    (r"EFI\arch\grubx64.efi", "Arch Linux (GRUB)", false),
    (r"EFI\opensuse\shimx64.efi", "openSUSE (shim)", false),
    (r"EFI\opensuse\grubx64.efi", "openSUSE (GRUB)", false),
    (r"EFI\centos\shimx64.efi", "CentOS (shim)", false),
    (r"EFI\rocky\shimx64.efi", "Rocky Linux (shim)", false),
    (r"EFI\systemd\systemd-bootx64.efi", "systemd-boot", false),
    (r"EFI\refind\refind_x64.efi", "rEFInd", false),
];

/// Find the ESP mount point on the current system.
pub fn find_esp_mount() -> Result<std::path::PathBuf, AppError> {
    #[cfg(target_os = "windows")]
    {
        find_esp_mount_windows()
    }

    #[cfg(target_os = "linux")]
    {
        find_esp_mount_linux()
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err(AppError::Efi {
            message: "ESP bootloader scanning is not supported on this platform".to_string(),
        })
    }
}

#[cfg(target_os = "windows")]
fn find_esp_mount_windows() -> Result<std::path::PathBuf, AppError> {
    // Try to find an already-mounted ESP partition via Get-Partition
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            r#"Get-Partition | Where-Object { $_.GptType -eq '{c12a7328-f81f-11d2-ba4b-00a0c93ec93b}' } | Select-Object -ExpandProperty AccessPaths | ConvertTo-Json -Compress"#,
        ])
        .output()
        .map_err(|e| AppError::Efi {
            message: format!("Failed to run PowerShell: {e}"),
        })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let trimmed = stdout.trim();
        if !trimmed.is_empty() {
            // Could be a JSON array of strings or a single string
            if let Ok(paths) = serde_json::from_str::<Vec<String>>(trimmed) {
                for path in &paths {
                    let p = std::path::Path::new(path);
                    if p.join("EFI").exists() {
                        return Ok(p.to_path_buf());
                    }
                }
                // If none has EFI dir visible, return the first with a drive letter
                for path in &paths {
                    if path.len() >= 2 && path.as_bytes()[1] == b':' {
                        return Ok(std::path::PathBuf::from(path));
                    }
                }
            } else if let Ok(path) = serde_json::from_str::<String>(trimmed) {
                return Ok(std::path::PathBuf::from(path));
            }
        }
    }

    // Fallback: try mountvol to mount ESP temporarily
    let output = Command::new("mountvol")
        .args(["S:", "/S"])
        .output()
        .map_err(|e| AppError::Efi {
            message: format!("Failed to run mountvol: {e}"),
        })?;

    if output.status.success() {
        let path = std::path::PathBuf::from("S:\\");
        if path.join("EFI").exists() {
            return Ok(path);
        }
    }

    Err(AppError::Efi {
        message: "Could not find or mount the ESP partition. Try running as administrator.".to_string(),
    })
}

#[cfg(target_os = "linux")]
fn find_esp_mount_linux() -> Result<std::path::PathBuf, AppError> {
    for candidate in &["/boot/efi", "/efi", "/boot"] {
        let path = std::path::Path::new(candidate);
        if path.join("EFI").exists() || path.join("efi").exists() {
            return Ok(path.to_path_buf());
        }
    }

    // Try findmnt as fallback
    let output = Command::new("findmnt")
        .args(["-n", "-o", "TARGET", "-t", "vfat"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let path = std::path::Path::new(line.trim());
                if path.join("EFI").exists() || path.join("efi").exists() {
                    return Ok(path.to_path_buf());
                }
            }
        }
    }

    Err(AppError::Efi {
        message: "Could not find mounted ESP. Check that it is mounted at /boot/efi or /efi.".to_string(),
    })
}

/// Scan the ESP for known bootloaders.
pub fn scan_esp_bootloaders(esp_root: &std::path::Path) -> Vec<BootloaderInfo> {
    let mut found = Vec::new();

    for &(rel_path, identity, is_default) in KNOWN_LOADERS {
        // Normalize path separators for the current OS
        let native_path = rel_path.replace('\\', &std::path::MAIN_SEPARATOR.to_string());
        let full = esp_root.join(&native_path);

        // Try case-insensitive match on case-sensitive filesystems
        let file_path = if full.exists() {
            Some(full)
        } else {
            find_case_insensitive(&esp_root, rel_path)
        };

        if let Some(path) = file_path {
            let meta = std::fs::metadata(&path).ok();
            let size = meta.as_ref().map(|m| format_size(m.len()));
            let modified = meta
                .as_ref()
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    let dt: chrono::DateTime<chrono::Local> = t.into();
                    dt.format("%Y-%m-%d %H:%M:%S").to_string()
                });

            found.push(BootloaderInfo {
                path: format!("\\{}", rel_path.replace('/', "\\")),
                identity: identity.to_string(),
                is_default,
                size,
                modified,
            });
        }
    }

    found
}

/// Try to find a file with case-insensitive path matching.
/// This handles Linux ESP partitions where EFI directory casing varies.
fn find_case_insensitive(root: &std::path::Path, rel_path: &str) -> Option<std::path::PathBuf> {
    let components: Vec<&str> = rel_path.split('\\').collect();
    let mut current = root.to_path_buf();

    for component in &components {
        let target_lower = component.to_lowercase();
        let mut matched = false;

        if let Ok(entries) = std::fs::read_dir(&current) {
            for entry in entries.flatten() {
                if entry.file_name().to_string_lossy().to_lowercase() == target_lower {
                    current = entry.path();
                    matched = true;
                    break;
                }
            }
        }

        if !matched {
            return None;
        }
    }

    if current.exists() { Some(current) } else { None }
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

    #[test]
    fn test_scan_esp_bootloaders_mock() {
        let tmp = std::env::temp_dir().join(format!("esp-scan-test-{}", std::process::id()));
        std::fs::create_dir_all(tmp.join("EFI").join("BOOT")).unwrap();
        std::fs::create_dir_all(tmp.join("EFI").join("Microsoft").join("Boot")).unwrap();

        // Create fake bootloader files
        std::fs::write(
            tmp.join("EFI").join("BOOT").join("BOOTX64.EFI"),
            b"fake-efi-binary",
        ).unwrap();
        std::fs::write(
            tmp.join("EFI").join("Microsoft").join("Boot").join("bootmgfw.efi"),
            b"fake-windows-bootmgr",
        ).unwrap();

        let results = scan_esp_bootloaders(&tmp);

        assert!(results.len() >= 2, "expected at least 2 loaders, got {}", results.len());

        let default = results.iter().find(|r| r.identity.contains("Default Fallback"));
        assert!(default.is_some(), "should find UEFI default fallback");
        let default = default.unwrap();
        assert!(default.is_default);
        assert!(default.size.is_some());
        assert!(default.modified.is_some());

        let windows = results.iter().find(|r| r.identity.contains("Windows"));
        assert!(windows.is_some(), "should find Windows boot manager");
        assert!(!windows.unwrap().is_default);

        // Cleanup
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_scan_esp_empty_dir() {
        let tmp = std::env::temp_dir().join(format!("esp-scan-empty-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let results = scan_esp_bootloaders(&tmp);
        assert!(results.is_empty());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_known_loaders_exhaustive() {
        // Verify all KNOWN_LOADERS have non-empty fields
        for &(path, identity, _) in KNOWN_LOADERS {
            assert!(!path.is_empty(), "loader path should not be empty");
            assert!(!identity.is_empty(), "identity should not be empty");
            assert!(
                path.to_lowercase().ends_with(".efi"),
                "{path} should end with .efi"
            );
        }
    }
}
