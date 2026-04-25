use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::core::BootManager;
use crate::error::AppError;

/// Top-level backup structure serialized to JSON
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupData {
    /// Backup format version
    pub version: u32,
    /// ISO 8601 timestamp of when the backup was created
    pub timestamp: String,
    /// BootOrder as a list of entry IDs
    pub boot_order: Vec<u16>,
    /// BootNext value, if set
    pub boot_next: Option<u16>,
    /// All boot entries
    pub entries: Vec<BackupEntry>,
}

/// A single boot entry in the backup
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupEntry {
    /// Hex ID (e.g., "0001")
    pub id: String,
    /// Full variable name (e.g., "Boot0001")
    pub variable_name: String,
    /// Vendor GUID
    pub vendor_guid: String,
    /// Variable attribute flags
    pub attributes: u32,
    /// Raw variable value as base64 — canonical for restore
    pub raw_value_base64: String,
    /// Optional decoded metadata (informational only, ignored on restore)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decoded: Option<DecodedMetadata>,
}

/// Human-readable decoded fields (informational only)
#[derive(Debug, Serialize, Deserialize)]
pub struct DecodedMetadata {
    pub description: String,
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition_guid: Option<String>,
}

const EFI_GLOBAL_VARIABLE_GUID: &str = "8be4df61-93ca-11d2-aa0d-00e098032b8c";
const BACKUP_VERSION: u32 = 1;

/// Export all boot entries and BootOrder to a BackupData struct.
pub fn export(mgr: &BootManager) -> Result<BackupData, AppError> {
    let boot_order = mgr.get_boot_order()?;
    let boot_next = mgr.get_boot_next()?;

    let mut entries = Vec::new();
    for &id in &boot_order {
        let info = mgr.get_entry(id)?;

        entries.push(BackupEntry {
            id: format!("{id:04X}"),
            variable_name: format!("Boot{id:04X}"),
            vendor_guid: EFI_GLOBAL_VARIABLE_GUID.to_string(),
            attributes: 0x07, // NV|BS|RT default
            raw_value_base64: BASE64.encode(&info.raw_bytes),
            decoded: Some(DecodedMetadata {
                description: info.description.clone(),
                active: info.active,
                file_path: info.file_path.clone(),
                partition_guid: info.partition_guid.clone(),
            }),
        });
    }

    Ok(BackupData {
        version: BACKUP_VERSION,
        timestamp: Utc::now().to_rfc3339(),
        boot_order,
        boot_next,
        entries,
    })
}

/// Validate a BackupData for correctness before applying.
pub fn validate(backup: &BackupData) -> Result<(), AppError> {
    if backup.version != BACKUP_VERSION {
        return Err(AppError::Backup {
            message: format!(
                "Unsupported backup version {} (expected {})",
                backup.version, BACKUP_VERSION
            ),
        });
    }

    for entry in &backup.entries {
        // Validate ID is parseable
        u16::from_str_radix(&entry.id, 16).map_err(|_| AppError::Backup {
            message: format!("Invalid entry ID: {}", entry.id),
        })?;

        // Validate base64 decodes
        BASE64
            .decode(&entry.raw_value_base64)
            .map_err(|e| AppError::Backup {
                message: format!("Invalid base64 for {}: {e}", entry.variable_name),
            })?;
    }

    // Validate BootOrder entries all have corresponding entry data
    for &id in &backup.boot_order {
        let id_str = format!("{id:04X}");
        if !backup.entries.iter().any(|e| e.id == id_str) {
            return Err(AppError::Backup {
                message: format!("BootOrder references Boot{id_str} but no entry data found"),
            });
        }
    }

    Ok(())
}

/// Restore boot entries from a BackupData.
/// Writes all Boot#### variables first, then BootOrder last.
pub fn restore(mgr: &mut BootManager, backup: &BackupData) -> Result<(), AppError> {
    // Validate first
    validate(backup)?;

    // Write each Boot#### variable from raw bytes
    for entry in &backup.entries {
        let raw = BASE64
            .decode(&entry.raw_value_base64)
            .map_err(|e| AppError::Backup {
                message: format!("Failed to decode base64 for {}: {e}", entry.variable_name),
            })?;

        mgr.write_raw(&entry.variable_name, &raw)?;
        tracing::info!("Restored {}", entry.variable_name);
    }

    // Write BootOrder last
    mgr.set_boot_order(backup.boot_order.clone())?;
    tracing::info!("Restored BootOrder: {:?}", backup.boot_order);

    // Restore BootNext if present
    if let Some(next) = backup.boot_next {
        mgr.set_boot_next(next)?;
        tracing::info!("Restored BootNext: {next:04X}");
    }

    Ok(())
}
