pub mod list;
pub mod info;
pub mod create;
pub mod delete;
pub mod edit;
pub mod order;
pub mod next;
pub mod toggle;
pub mod backup;
pub mod restore;
pub mod esp;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "uefibootmgrrs",
    about = "UEFI Boot Manager — manage boot entries, boot order, and ESP",
    long_about = "UEFI Boot Manager — a CLI tool to list, create, edit, delete, \
        enable/disable boot entries, set boot order and BootNext, \
        and backup/restore EFI NVRAM variables.\n\n\
        Requires administrator/root privileges to access EFI variables.",
    version,
    author
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose/debug output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Output in JSON format (where supported)
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all boot entries in boot order
    List,

    /// Show detailed information about a boot entry
    Info {
        /// Boot entry ID in hex (e.g., 0001)
        id: String,
    },

    /// Create a new boot entry
    Create {
        /// Human-readable description for the entry
        #[arg(short, long)]
        description: String,

        /// Path to the EFI executable (e.g., \EFI\ubuntu\shimx64.efi)
        #[arg(short, long)]
        loader: String,

        /// GPT partition GUID of the ESP
        #[arg(short, long)]
        partition: Option<String>,

        /// Boot entry ID in hex (auto-assigned if omitted)
        #[arg(long)]
        id: Option<String>,
    },

    /// Delete a boot entry
    Delete {
        /// Boot entry ID in hex (e.g., 0001)
        id: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Edit an existing boot entry
    Edit {
        /// Boot entry ID in hex (e.g., 0001)
        id: String,

        /// New description
        #[arg(short, long)]
        description: Option<String>,

        /// New EFI loader path
        #[arg(short, long)]
        loader: Option<String>,
    },

    /// Set the boot order
    Order {
        /// Comma-separated boot entry IDs in desired order (e.g., 0001,0000,0002)
        ids: String,
    },

    /// Set BootNext for a one-time boot override
    Next {
        /// Boot entry ID in hex (e.g., 0001)
        id: String,
    },

    /// Enable a boot entry (set LOAD_OPTION_ACTIVE)
    Enable {
        /// Boot entry ID in hex
        id: String,
    },

    /// Disable a boot entry (clear LOAD_OPTION_ACTIVE)
    Disable {
        /// Boot entry ID in hex
        id: String,
    },

    /// Export all boot entries to a JSON backup file
    Backup {
        /// Output file path
        file: String,
    },

    /// Restore boot entries from a JSON backup file
    Restore {
        /// Input backup file path
        file: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Launch interactive TUI mode
    Tui,

    /// Manage ESP partition flags
    Esp {
        #[command(subcommand)]
        action: EspAction,
    },
}

#[derive(Subcommand)]
pub enum EspAction {
    /// List all GPT partitions
    List,

    /// Set a partition's type GUID to ESP
    Set {
        /// Disk identifier (e.g., "Disk 0" on Windows, "/dev/sda" on Linux)
        disk: String,

        /// Partition number
        partition: u32,
    },

    /// Clear the ESP flag (revert to Basic Data)
    Clear {
        /// Disk identifier (e.g., "Disk 0" on Windows, "/dev/sda" on Linux)
        disk: String,

        /// Partition number
        partition: u32,
    },

    /// Show bootloaders found on the ESP
    Bootloader,
}
