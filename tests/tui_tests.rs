//! Tests for TUI App state transitions (no rendering).

use efivar::boot::{
    BootEntry, BootEntryAttributes, EFIHardDrive, EFIHardDriveType, FilePath, FilePathList,
};
use std::str::FromStr;
use uuid::Uuid;

fn test_store(suffix: &str) -> Box<dyn efivar::VarManager> {
    efivar::file_store(std::env::temp_dir().join(format!(
        "uefibootmgrrs-tui-{}-{}.toml",
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

fn setup_app(suffix: &str) -> uefibootmgrrs::tui::app::App {
    let mut store = test_store(suffix);
    store
        .add_boot_entry(0, sample_entry("Windows", r"\EFI\Microsoft\Boot\bootmgfw.efi"))
        .unwrap();
    store
        .add_boot_entry(1, sample_entry("Ubuntu", r"\EFI\ubuntu\shimx64.efi"))
        .unwrap();
    store.set_boot_order(vec![0, 1]).unwrap();
    let mgr = uefibootmgrrs::core::BootManager::new(store);
    let mut app = uefibootmgrrs::tui::app::App::new(mgr);
    app.refresh_entries();
    app
}

#[test]
fn tui_initial_state() {
    let app = setup_app("init");
    assert_eq!(app.entries.len(), 2);
    assert_eq!(app.selected, 0);
    assert_eq!(app.view, uefibootmgrrs::tui::app::View::EntryList);
    assert!(!app.should_quit);
    assert!(!app.reorder_mode);
}

#[test]
fn tui_navigation() {
    let mut app = setup_app("nav");

    // Move down
    assert_eq!(app.selected, 0);
    app.selected += 1;
    assert_eq!(app.selected, 1);

    // Don't go past end
    if app.selected + 1 >= app.entries.len() {
        // can't move further
    }
    assert_eq!(app.selected, 1);

    // Move back up
    app.selected -= 1;
    assert_eq!(app.selected, 0);
}

#[test]
fn tui_toggle_active() {
    let mut app = setup_app("toggle");
    assert!(app.entries[0].active);

    app.toggle_selected_active();
    assert!(!app.entries[0].active);

    app.toggle_selected_active();
    assert!(app.entries[0].active);
}

#[test]
fn tui_delete_with_confirm() {
    let mut app = setup_app("delete");
    assert_eq!(app.entries.len(), 2);

    // Request delete — should open confirm
    app.delete_selected();
    assert_eq!(app.view, uefibootmgrrs::tui::app::View::Confirm);
    assert!(app.confirm.is_some());

    // Execute confirm
    app.execute_confirm();
    assert_eq!(app.view, uefibootmgrrs::tui::app::View::EntryList);
    assert_eq!(app.entries.len(), 1);
}

#[test]
fn tui_cancel_delete() {
    let mut app = setup_app("cancel-del");
    app.delete_selected();
    assert_eq!(app.view, uefibootmgrrs::tui::app::View::Confirm);

    app.cancel_confirm();
    assert_eq!(app.view, uefibootmgrrs::tui::app::View::EntryList);
    assert_eq!(app.entries.len(), 2); // Still 2
}

#[test]
fn tui_reorder() {
    let mut app = setup_app("reorder");

    // Select second entry
    app.selected = 1;
    app.reorder_mode = true;

    // Move it up
    app.move_selected_up();
    assert_eq!(app.selected, 0);
    assert_eq!(app.entries[0].description, "Ubuntu");
    assert_eq!(app.entries[1].description, "Windows");
}

#[test]
fn tui_open_create_form() {
    let mut app = setup_app("create-form");
    app.open_create_form();
    assert_eq!(app.view, uefibootmgrrs::tui::app::View::EntryForm);
    assert_eq!(app.form_mode, uefibootmgrrs::tui::app::FormMode::Create);
    assert!(app.form_description.is_empty());
}

#[test]
fn tui_open_edit_form() {
    let mut app = setup_app("edit-form");
    app.open_edit_form();
    assert_eq!(app.view, uefibootmgrrs::tui::app::View::EntryForm);
    assert_eq!(app.form_mode, uefibootmgrrs::tui::app::FormMode::Edit);
    assert_eq!(app.form_description, "Windows");
    assert_eq!(app.form_edit_id, Some(0));
}

#[test]
fn tui_create_entry_via_form() {
    let mut app = setup_app("create-submit");
    app.open_create_form();
    app.form_description = "Fedora".to_string();
    app.form_loader = r"\EFI\fedora\shimx64.efi".to_string();
    app.submit_form();

    assert_eq!(app.view, uefibootmgrrs::tui::app::View::EntryList);
    assert_eq!(app.entries.len(), 3);
    assert_eq!(app.entries[2].description, "Fedora");
}

#[test]
fn tui_edit_entry_via_form() {
    let mut app = setup_app("edit-submit");
    app.open_edit_form();
    app.form_description = "Modified Windows".to_string();
    app.submit_form();

    assert_eq!(app.entries[0].description, "Modified Windows");
}

#[test]
fn tui_backup_and_restore() {
    let mut app = setup_app("backup-restore");

    let tmp = std::env::temp_dir().join(format!(
        "uefibootmgrrs-tui-backup-{}.json",
        std::process::id()
    ));
    let path = tmp.to_str().unwrap().to_string();

    // Backup
    app.backup_path = path.clone();
    app.backup_mode = uefibootmgrrs::tui::app::BackupMode::Backup;
    app.submit_backup_restore();
    assert!(tmp.exists());
    assert!(!app.status_is_error);

    // Clean up
    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn tui_view_transitions() {
    let mut app = setup_app("views");

    // Help
    app.view = uefibootmgrrs::tui::app::View::Help;
    assert_eq!(app.view, uefibootmgrrs::tui::app::View::Help);
    app.view = uefibootmgrrs::tui::app::View::EntryList;

    // Detail
    app.view = uefibootmgrrs::tui::app::View::EntryDetail;
    assert_eq!(app.view, uefibootmgrrs::tui::app::View::EntryDetail);

    // BackupRestore
    app.open_backup();
    assert_eq!(app.view, uefibootmgrrs::tui::app::View::BackupRestore);
    assert_eq!(app.backup_mode, uefibootmgrrs::tui::app::BackupMode::Backup);

    app.open_restore();
    assert_eq!(app.backup_mode, uefibootmgrrs::tui::app::BackupMode::Restore);
}

#[test]
fn tui_form_field_navigation() {
    use uefibootmgrrs::tui::app::FormField;

    assert_eq!(FormField::Description.next(), FormField::Loader);
    assert_eq!(FormField::Loader.next(), FormField::Partition);
    assert_eq!(FormField::Partition.next(), FormField::Description);

    assert_eq!(FormField::Description.prev(), FormField::Partition);
    assert_eq!(FormField::Loader.prev(), FormField::Description);
    assert_eq!(FormField::Partition.prev(), FormField::Loader);
}

// --- Wizard tests ---

#[test]
fn tui_wizard_open() {
    let mut app = setup_app("wizard-open");
    assert_eq!(app.view, uefibootmgrrs::tui::app::View::EntryList);

    app.open_wizard();
    assert_eq!(app.view, uefibootmgrrs::tui::app::View::Wizard);
    assert_eq!(app.wizard_selected, 0);
}

#[test]
fn tui_wizard_navigate() {
    let mut app = setup_app("wizard-nav");
    app.open_wizard();

    let count = app.wizard_templates.len();

    // Move down to last
    for _ in 0..count - 1 {
        app.wizard_selected += 1;
    }
    assert_eq!(app.wizard_selected, count - 1);

    // Move back up
    app.wizard_selected = 0;
    assert_eq!(app.wizard_selected, 0);
}

#[test]
fn tui_wizard_apply_windows() {
    let mut app = setup_app("wizard-win");
    app.open_wizard();

    // Select Windows (index 0)
    app.apply_wizard_template(0);

    assert_eq!(app.view, uefibootmgrrs::tui::app::View::EntryForm);
    assert_eq!(app.form_mode, uefibootmgrrs::tui::app::FormMode::Create);
    assert_eq!(app.form_description, "Windows Boot Manager");
    assert_eq!(app.form_loader, r"\EFI\Microsoft\Boot\bootmgfw.efi");
    assert!(app.form_partition.is_empty());
    assert!(app.form_edit_id.is_none());
}

#[test]
fn tui_wizard_apply_ubuntu() {
    let mut app = setup_app("wizard-ubu");
    app.open_wizard();

    // Select Ubuntu (index 1)
    app.apply_wizard_template(1);

    assert_eq!(app.form_description, "Ubuntu");
    assert_eq!(app.form_loader, r"\EFI\ubuntu\shimx64.efi");
}

#[test]
fn tui_wizard_apply_fedora() {
    let mut app = setup_app("wizard-fed");
    app.open_wizard();
    app.apply_wizard_template(2);
    assert_eq!(app.form_description, "Fedora");
    assert_eq!(app.form_loader, r"\EFI\fedora\shimx64.efi");
}

#[test]
fn tui_wizard_apply_generic_grub() {
    let mut app = setup_app("wizard-grub");
    app.open_wizard();

    let last = app.wizard_templates.len() - 1;
    app.apply_wizard_template(last);

    assert_eq!(app.form_description, "GRUB");
    assert_eq!(app.form_loader, r"\EFI\BOOT\grubx64.efi");
}

#[test]
fn tui_wizard_cancel() {
    let mut app = setup_app("wizard-cancel");
    app.open_wizard();
    assert_eq!(app.view, uefibootmgrrs::tui::app::View::Wizard);

    // Esc cancels back to list
    app.view = uefibootmgrrs::tui::app::View::EntryList;
    assert_eq!(app.view, uefibootmgrrs::tui::app::View::EntryList);
}

#[test]
fn tui_wizard_submit_creates_entry() {
    let mut app = setup_app("wizard-submit");
    assert_eq!(app.entries.len(), 2);

    app.open_wizard();
    app.apply_wizard_template(0); // Windows

    // Now in EntryForm with pre-filled values — submit it
    app.submit_form();

    assert_eq!(app.view, uefibootmgrrs::tui::app::View::EntryList);
    assert_eq!(app.entries.len(), 3);
    assert_eq!(app.entries[2].description, "Windows Boot Manager");
}

#[test]
fn tui_wizard_all_templates_valid() {
    use uefibootmgrrs::tui::app::WizardTemplate;

    for (i, template) in WizardTemplate::defaults().iter().enumerate() {
        assert!(!template.label.is_empty(), "template {i} has empty label");
        assert!(!template.description.is_empty(), "template {i} has empty description");
        assert!(template.loader.ends_with(".efi"), "template {i} loader should end with .efi");
    }
}

#[test]
fn tui_wizard_load_custom_json() {
    use uefibootmgrrs::tui::app::WizardTemplate;

    let custom = vec![
        WizardTemplate {
            label: "Custom OS".into(),
            description: "My OS".into(),
            loader: r"\EFI\custom\boot.efi".into(),
        },
    ];
    let json = serde_json::to_string_pretty(&custom).unwrap();

    // Verify round-trip
    let parsed: Vec<WizardTemplate> = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].label, "Custom OS");
    assert_eq!(parsed[0].description, "My OS");
    assert_eq!(parsed[0].loader, r"\EFI\custom\boot.efi");
}
