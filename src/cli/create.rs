use efivar::boot::{
    BootEntry, BootEntryAttributes, EFIHardDrive, EFIHardDriveType, FilePath, FilePathList,
};

use crate::core::BootManager;
use crate::error::{parse_boot_id, AppError};

pub fn run(
    mgr: &mut BootManager,
    description: &str,
    loader: &str,
    partition: Option<&str>,
    id: Option<&str>,
) -> Result<(), AppError> {
    let entry_id = match id {
        Some(id_str) => parse_boot_id(id_str)?,
        None => mgr.next_free_id()?,
    };

    let part_guid_str = partition.unwrap_or("00000000-0000-0000-0000-000000000000");
    let guid = uuid::Uuid::parse_str(part_guid_str).map_err(|e| AppError::Parse {
        message: format!("Invalid partition GUID: {e}"),
    })?;

    let file_path_list = FilePathList {
        file_path: FilePath {
            path: loader.to_string(),
        },
        hard_drive: EFIHardDrive {
            partition_number: 1,
            partition_start: 0,
            partition_size: 0,
            partition_sig: guid,
            format: 0x02, // GPT
            sig_type: EFIHardDriveType::Gpt,
        },
    };

    let entry = BootEntry {
        attributes: BootEntryAttributes::LOAD_OPTION_ACTIVE,
        description: description.to_string(),
        file_path_list: Some(file_path_list),
        optional_data: Vec::new(),
    };

    mgr.create_entry(entry_id, entry)?;

    println!("Created boot entry Boot{entry_id:04X}: {description}");
    println!("  Loader: {loader}");
    if let Some(p) = partition {
        println!("  Partition: {p}");
    }

    Ok(())
}
