use efivar::boot::{BootEntry, BootEntryAttributes};

/// Display-friendly representation of a parsed UEFI boot entry
#[derive(Debug, Clone, serde::Serialize)]
pub struct BootEntryInfo {
    /// Numeric boot entry ID (e.g., 0x0001)
    pub id: u16,
    /// Human-readable description (e.g., "Windows Boot Manager")
    pub description: String,
    /// Whether LOAD_OPTION_ACTIVE is set
    pub active: bool,
    /// File path from the device path list (e.g., `\EFI\Microsoft\Boot\bootmgfw.efi`)
    pub file_path: Option<String>,
    /// Partition GUID from the hard drive device path node
    pub partition_guid: Option<String>,
    /// Human-readable device path summary
    pub device_path_display: String,
    /// Raw variable bytes (for backup purposes)
    #[serde(skip)]
    pub raw_bytes: Vec<u8>,
}

impl BootEntryInfo {
    /// Parse a raw Boot#### variable value into a display-friendly struct
    pub fn from_raw(id: u16, raw_bytes: Vec<u8>) -> Result<Self, crate::error::AppError> {
        let entry = BootEntry::parse(raw_bytes.clone()).map_err(|e| crate::error::AppError::Parse {
            message: format!("Failed to parse Boot{id:04X}: {e}"),
        })?;

        let mut file_path: Option<String> = None;
        let mut partition_guid: Option<String> = None;
        let mut dp_parts: Vec<String> = Vec::new();

        if let Some(ref fpl) = entry.file_path_list {
            dp_parts.push(format!("{}", fpl.hard_drive));
            partition_guid = Some(fpl.hard_drive.partition_sig.to_string());

            dp_parts.push(fpl.file_path.path.clone());
            file_path = Some(fpl.file_path.path.clone());
        }

        let device_path_display = if dp_parts.is_empty() {
            "(none)".to_string()
        } else {
            dp_parts.join("/")
        };

        Ok(BootEntryInfo {
            id,
            description: entry.description.clone(),
            active: entry.attributes.contains(BootEntryAttributes::LOAD_OPTION_ACTIVE),
            file_path,
            partition_guid,
            device_path_display,
            raw_bytes,
        })
    }
}
