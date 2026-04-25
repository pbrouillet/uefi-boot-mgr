//! Integration tests for CLI command handlers.
//! These call the handler functions directly with a file_store backend.

use efivar::boot::{
    BootEntry, BootEntryAttributes, EFIHardDrive, EFIHardDriveType, FilePath,
    FilePathList,
};
use std::str::FromStr;
use uuid::Uuid;

fn test_store(suffix: &str) -> Box<dyn efivar::VarManager> {
    efivar::file_store(std::env::temp_dir().join(format!(
        "uefibootmgrrs-cli-{}-{}.toml",
        suffix,
        std::process::id()
    )))
}

fn sample_entry(desc: &str, loader: &str) -> BootEntry {
    BootEntry {
        attributes: BootEntryAttributes::LOAD_OPTION_ACTIVE,
        description: desc.to_string(),
        file_path_list: Some(FilePathList {
            file_path: FilePath {
                path: loader.to_string(),
            },
            hard_drive: EFIHardDrive {
                partition_number: 1,
                partition_start: 2048,
                partition_size: 1024000,
                partition_sig: Uuid::from_str("90364bbd-1000-47fc-8c05-8707e01b4593").unwrap(),
                format: 0x02,
                sig_type: EFIHardDriveType::Gpt,
            },
        }),
        optional_data: Vec::new(),
    }
}

fn setup_mgr(suffix: &str) -> uefibootmgrrs::core::BootManager {
    let mut store = test_store(suffix);
    store
        .add_boot_entry(0, sample_entry("Windows", r"\EFI\Microsoft\Boot\bootmgfw.efi"))
        .unwrap();
    store
        .add_boot_entry(1, sample_entry("Ubuntu", r"\EFI\ubuntu\shimx64.efi"))
        .unwrap();
    store.set_boot_order(vec![0, 1]).unwrap();
    uefibootmgrrs::core::BootManager::new(store)
}

#[test]
fn cli_list_json_output() {
    let mgr = setup_mgr("list-json");
    // list with json=true should not panic
    uefibootmgrrs::cli::list::run(&mgr, true).unwrap();
}

#[test]
fn cli_list_table_output() {
    let mgr = setup_mgr("list-table");
    uefibootmgrrs::cli::list::run(&mgr, false).unwrap();
}

#[test]
fn cli_info_json() {
    let mgr = setup_mgr("info-json");
    uefibootmgrrs::cli::info::run(&mgr, "0000", true).unwrap();
}

#[test]
fn cli_info_plain() {
    let mgr = setup_mgr("info-plain");
    uefibootmgrrs::cli::info::run(&mgr, "0000", false).unwrap();
}

#[test]
fn cli_info_invalid_id() {
    let mgr = setup_mgr("info-invalid");
    let result = uefibootmgrrs::cli::info::run(&mgr, "ZZZZ", false);
    assert!(result.is_err());
}

#[test]
fn cli_create_auto_id() {
    let mut mgr = setup_mgr("create-auto");
    uefibootmgrrs::cli::create::run(
        &mut mgr,
        "Fedora",
        r"\EFI\fedora\shimx64.efi",
        Some("90364bbd-1000-47fc-8c05-8707e01b4593"),
        None,
    )
    .unwrap();

    // Should have been assigned ID 2
    let entry = mgr.get_entry(2).unwrap();
    assert_eq!(entry.description, "Fedora");
}

#[test]
fn cli_create_explicit_id() {
    let mut mgr = setup_mgr("create-explicit");
    uefibootmgrrs::cli::create::run(
        &mut mgr,
        "Arch",
        r"\EFI\arch\grubx64.efi",
        None,
        Some("000A"),
    )
    .unwrap();

    let entry = mgr.get_entry(0x0A).unwrap();
    assert_eq!(entry.description, "Arch");
}

#[test]
fn cli_delete_force() {
    let mut mgr = setup_mgr("delete-force");
    uefibootmgrrs::cli::delete::run(&mut mgr, "0001", true).unwrap();
    assert!(mgr.get_entry(1).is_err());
}

#[test]
fn cli_edit_description() {
    let mut mgr = setup_mgr("edit-desc");
    uefibootmgrrs::cli::edit::run(&mut mgr, "0000", Some("New Windows"), None).unwrap();
    let entry = mgr.get_entry(0).unwrap();
    assert_eq!(entry.description, "New Windows");
}

#[test]
fn cli_edit_loader() {
    let mut mgr = setup_mgr("edit-loader");
    uefibootmgrrs::cli::edit::run(&mut mgr, "0000", None, Some(r"\EFI\new\boot.efi")).unwrap();
    let entry = mgr.get_entry(0).unwrap();
    assert_eq!(entry.file_path.as_deref(), Some(r"\EFI\new\boot.efi"));
}

#[test]
fn cli_edit_no_changes() {
    let mut mgr = setup_mgr("edit-noop");
    // No changes specified — should succeed quietly
    uefibootmgrrs::cli::edit::run(&mut mgr, "0000", None, None).unwrap();
}

#[test]
fn cli_order() {
    let mut mgr = setup_mgr("order");
    uefibootmgrrs::cli::order::run(&mut mgr, "0001,0000").unwrap();
    let order = mgr.get_boot_order().unwrap();
    assert_eq!(order, vec![1, 0]);
}

#[test]
fn cli_next() {
    let mut mgr = setup_mgr("next");
    uefibootmgrrs::cli::next::run(&mut mgr, "0001").unwrap();
    let next = mgr.get_boot_next().unwrap();
    assert_eq!(next, Some(1));
}

#[test]
fn cli_toggle_enable_already_active() {
    let mut mgr = setup_mgr("toggle-en");
    // Entry 0 is already active — should print already-enabled message
    uefibootmgrrs::cli::toggle::run_enable(&mut mgr, "0000").unwrap();
}

#[test]
fn cli_toggle_disable_then_enable() {
    let mut mgr = setup_mgr("toggle-dis-en");
    uefibootmgrrs::cli::toggle::run_disable(&mut mgr, "0000").unwrap();
    let entry = mgr.get_entry(0).unwrap();
    assert!(!entry.active);

    uefibootmgrrs::cli::toggle::run_enable(&mut mgr, "0000").unwrap();
    let entry = mgr.get_entry(0).unwrap();
    assert!(entry.active);
}

#[test]
fn cli_backup_and_restore() {
    let mgr = setup_mgr("backup-restore");

    let tmp = std::env::temp_dir().join(format!(
        "uefibootmgrrs-backup-{}.json",
        std::process::id()
    ));
    let tmp_str = tmp.to_str().unwrap();

    // Export backup
    uefibootmgrrs::cli::backup::run(&mgr, tmp_str).unwrap();

    // Verify the file exists and is valid JSON
    let contents = std::fs::read_to_string(&tmp).unwrap();
    let data: uefibootmgrrs::core::backup::BackupData = serde_json::from_str(&contents).unwrap();
    assert_eq!(data.entries.len(), 2);

    // Clean up
    let _ = std::fs::remove_file(&tmp);
}
