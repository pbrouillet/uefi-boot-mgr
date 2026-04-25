use std::io;

/// Application-level errors
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("EFI variable error: {message}")]
    Efi { message: String },

    #[error("Insufficient privileges: {message}")]
    Privilege { message: String },

    #[error("Boot entry not found: Boot{id:04X}")]
    EntryNotFound { id: u16 },

    #[error("Invalid boot entry ID: {input}")]
    InvalidEntryId { input: String },

    #[error("Backup/restore error: {message}")]
    Backup { message: String },

    #[error("Parse error: {message}")]
    Parse { message: String },

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl From<efivar::Error> for AppError {
    fn from(e: efivar::Error) -> Self {
        AppError::Efi {
            message: e.to_string(),
        }
    }
}

impl From<dialoguer::Error> for AppError {
    fn from(e: dialoguer::Error) -> Self {
        AppError::Io(match e {
            dialoguer::Error::IO(io_err) => io_err,
        })
    }
}



/// Parse a hex boot entry ID like "0001" or "1" into a u16
pub fn parse_boot_id(input: &str) -> Result<u16, AppError> {
    u16::from_str_radix(input.trim_start_matches("0x").trim_start_matches("0X"), 16).map_err(
        |_| AppError::InvalidEntryId {
            input: input.to_string(),
        },
    )
}
