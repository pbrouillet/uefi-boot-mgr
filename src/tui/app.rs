use crate::core::BootManager;
use crate::core::BootEntryInfo;

/// Which view is currently active
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    EntryList,
    EntryDetail,
    EntryForm,
    BackupRestore,
    Help,
    Confirm,
    Wizard,
}

/// Sub-mode for entry form
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormMode {
    Create,
    Edit,
}

/// Which field is focused in the entry form
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormField {
    Description,
    Loader,
    Partition,
}

impl FormField {
    pub fn next(self) -> Self {
        match self {
            FormField::Description => FormField::Loader,
            FormField::Loader => FormField::Partition,
            FormField::Partition => FormField::Description,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            FormField::Description => FormField::Partition,
            FormField::Loader => FormField::Description,
            FormField::Partition => FormField::Loader,
        }
    }
}

/// Sub-mode for backup/restore
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupMode {
    Backup,
    Restore,
}

/// Confirmation dialog context
#[derive(Debug, Clone)]
pub struct ConfirmState {
    pub message: String,
    pub action: ConfirmAction,
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeleteEntry(u16),
    RestoreBackup(String),
}

/// Pre-defined boot entry templates for the wizard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardTemplate {
    Windows,
    Ubuntu,
    Fedora,
    Arch,
    Debian,
    OpenSuse,
    GenericGrub,
}

impl WizardTemplate {
    pub const ALL: &[WizardTemplate] = &[
        WizardTemplate::Windows,
        WizardTemplate::Ubuntu,
        WizardTemplate::Fedora,
        WizardTemplate::Arch,
        WizardTemplate::Debian,
        WizardTemplate::OpenSuse,
        WizardTemplate::GenericGrub,
    ];

    pub fn label(self) -> &'static str {
        match self {
            WizardTemplate::Windows => "Windows Boot Manager",
            WizardTemplate::Ubuntu => "Ubuntu (shim)",
            WizardTemplate::Fedora => "Fedora (shim)",
            WizardTemplate::Arch => "Arch Linux",
            WizardTemplate::Debian => "Debian (shim)",
            WizardTemplate::OpenSuse => "openSUSE (shim)",
            WizardTemplate::GenericGrub => "Generic GRUB",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            WizardTemplate::Windows => "Windows Boot Manager",
            WizardTemplate::Ubuntu => "Ubuntu",
            WizardTemplate::Fedora => "Fedora",
            WizardTemplate::Arch => "Arch Linux",
            WizardTemplate::Debian => "Debian",
            WizardTemplate::OpenSuse => "openSUSE",
            WizardTemplate::GenericGrub => "GRUB",
        }
    }

    pub fn loader(self) -> &'static str {
        match self {
            WizardTemplate::Windows => r"\EFI\Microsoft\Boot\bootmgfw.efi",
            WizardTemplate::Ubuntu => r"\EFI\ubuntu\shimx64.efi",
            WizardTemplate::Fedora => r"\EFI\fedora\shimx64.efi",
            WizardTemplate::Arch => r"\EFI\arch\grubx64.efi",
            WizardTemplate::Debian => r"\EFI\debian\shimx64.efi",
            WizardTemplate::OpenSuse => r"\EFI\opensuse\shimx64.efi",
            WizardTemplate::GenericGrub => r"\EFI\BOOT\grubx64.efi",
        }
    }
}

/// Main application state
pub struct App {
    pub mgr: BootManager,
    pub entries: Vec<BootEntryInfo>,
    pub boot_order: Vec<u16>,
    pub boot_current: Option<u16>,
    pub boot_next: Option<u16>,

    // Navigation
    pub view: View,
    pub selected: usize,
    pub should_quit: bool,

    // Status bar
    pub status_message: Option<String>,
    pub status_is_error: bool,

    // Entry form state
    pub form_mode: FormMode,
    pub form_field: FormField,
    pub form_description: String,
    pub form_loader: String,
    pub form_partition: String,
    pub form_edit_id: Option<u16>,

    // Backup/restore state
    pub backup_mode: BackupMode,
    pub backup_path: String,

    // Reorder mode
    pub reorder_mode: bool,

    // Confirm dialog
    pub confirm: Option<ConfirmState>,

    // Wizard state
    pub wizard_selected: usize,
}

impl App {
    pub fn new(mgr: BootManager) -> Self {
        Self {
            mgr,
            entries: Vec::new(),
            boot_order: Vec::new(),
            boot_current: None,
            boot_next: None,
            view: View::EntryList,
            selected: 0,
            should_quit: false,
            status_message: None,
            status_is_error: false,
            form_mode: FormMode::Create,
            form_field: FormField::Description,
            form_description: String::new(),
            form_loader: String::new(),
            form_partition: String::new(),
            form_edit_id: None,
            backup_mode: BackupMode::Backup,
            backup_path: String::new(),
            reorder_mode: false,
            confirm: None,
            wizard_selected: 0,
        }
    }

    /// Refresh entry cache from the boot manager
    pub fn refresh_entries(&mut self) {
        self.boot_order = self.mgr.get_boot_order().unwrap_or_default();
        self.boot_current = self.mgr.get_boot_current().unwrap_or(None);
        self.boot_next = self.mgr.get_boot_next().unwrap_or(None);

        match self.mgr.list_entries() {
            Ok(entries) => {
                self.entries = entries;
                if self.selected >= self.entries.len() && !self.entries.is_empty() {
                    self.selected = self.entries.len() - 1;
                }
            }
            Err(e) => {
                self.set_error(format!("Failed to load entries: {e}"));
            }
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
        self.status_is_error = false;
    }

    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
        self.status_is_error = true;
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    pub fn selected_entry(&self) -> Option<&BootEntryInfo> {
        self.entries.get(self.selected)
    }

    // --- Actions ---

    pub fn delete_selected(&mut self) {
        if let Some(entry) = self.selected_entry() {
            let id = entry.id;
            let desc = entry.description.clone();
            self.confirm = Some(ConfirmState {
                message: format!("Delete Boot{id:04X} ({desc})?"),
                action: ConfirmAction::DeleteEntry(id),
            });
            self.view = View::Confirm;
        }
    }

    pub fn execute_confirm(&mut self) {
        if let Some(confirm) = self.confirm.take() {
            match confirm.action {
                ConfirmAction::DeleteEntry(id) => {
                    match self.mgr.delete_entry(id) {
                        Ok(()) => {
                            self.set_status(format!("Deleted Boot{id:04X}"));
                            self.refresh_entries();
                        }
                        Err(e) => self.set_error(format!("Delete failed: {e}")),
                    }
                }
                ConfirmAction::RestoreBackup(path) => {
                    self.do_restore(&path);
                }
            }
        }
        self.view = View::EntryList;
    }

    pub fn cancel_confirm(&mut self) {
        self.confirm = None;
        self.view = View::EntryList;
    }

    pub fn toggle_selected_active(&mut self) {
        if let Some(entry) = self.selected_entry() {
            let id = entry.id;
            match self.mgr.toggle_active(id) {
                Ok(active) => {
                    let state = if active { "enabled" } else { "disabled" };
                    self.set_status(format!("Boot{id:04X} {state}"));
                    self.refresh_entries();
                }
                Err(e) => self.set_error(format!("Toggle failed: {e}")),
            }
        }
    }

    pub fn move_selected_up(&mut self) {
        if self.selected > 0 && self.entries.len() > 1 {
            let mut order = self.boot_order.clone();
            if self.selected < order.len() {
                order.swap(self.selected, self.selected - 1);
                match self.mgr.set_boot_order(order) {
                    Ok(()) => {
                        self.selected -= 1;
                        self.refresh_entries();
                        self.set_status("Boot order updated");
                    }
                    Err(e) => self.set_error(format!("Reorder failed: {e}")),
                }
            }
        }
    }

    pub fn move_selected_down(&mut self) {
        if self.selected + 1 < self.entries.len() {
            let mut order = self.boot_order.clone();
            if self.selected + 1 < order.len() {
                order.swap(self.selected, self.selected + 1);
                match self.mgr.set_boot_order(order) {
                    Ok(()) => {
                        self.selected += 1;
                        self.refresh_entries();
                        self.set_status("Boot order updated");
                    }
                    Err(e) => self.set_error(format!("Reorder failed: {e}")),
                }
            }
        }
    }

    pub fn open_create_form(&mut self) {
        self.form_mode = FormMode::Create;
        self.form_field = FormField::Description;
        self.form_description.clear();
        self.form_loader.clear();
        self.form_partition.clear();
        self.form_edit_id = None;
        self.view = View::EntryForm;
    }

    pub fn open_edit_form(&mut self) {
        let entry_data = self.selected_entry().map(|e| {
            (e.id, e.description.clone(), e.file_path.clone(), e.partition_guid.clone())
        });
        if let Some((id, desc, path, guid)) = entry_data {
            self.form_mode = FormMode::Edit;
            self.form_field = FormField::Description;
            self.form_description = desc;
            self.form_loader = path.unwrap_or_default();
            self.form_partition = guid.unwrap_or_default();
            self.form_edit_id = Some(id);
            self.view = View::EntryForm;
        }
    }

    pub fn submit_form(&mut self) {
        match self.form_mode {
            FormMode::Create => self.do_create(),
            FormMode::Edit => self.do_edit(),
        }
        self.view = View::EntryList;
    }

    fn do_create(&mut self) {
        if self.form_description.is_empty() || self.form_loader.is_empty() {
            self.set_error("Description and loader path are required");
            return;
        }

        use efivar::boot::*;
        let partition = if self.form_partition.is_empty() {
            None
        } else {
            match uuid::Uuid::parse_str(&self.form_partition) {
                Ok(guid) => Some(guid),
                Err(e) => {
                    self.set_error(format!("Invalid partition GUID: {e}"));
                    return;
                }
            }
        };

        let id = match self.mgr.next_free_id() {
            Ok(id) => id,
            Err(e) => {
                self.set_error(format!("No free ID: {e}"));
                return;
            }
        };

        let hard_drive = EFIHardDrive {
            partition_number: 1,
            partition_start: 0,
            partition_size: 0,
            partition_sig: partition.unwrap_or_default(),
            format: 0x02,
            sig_type: EFIHardDriveType::Gpt,
        };

        let entry = BootEntry {
            attributes: BootEntryAttributes::LOAD_OPTION_ACTIVE,
            description: self.form_description.clone(),
            file_path_list: Some(FilePathList {
                file_path: FilePath {
                    path: self.form_loader.clone(),
                },
                hard_drive,
            }),
            optional_data: Vec::new(),
        };

        match self.mgr.create_entry(id, entry) {
            Ok(()) => {
                self.set_status(format!("Created Boot{id:04X}: {}", self.form_description));
                self.refresh_entries();
            }
            Err(e) => self.set_error(format!("Create failed: {e}")),
        }
    }

    fn do_edit(&mut self) {
        let id = match self.form_edit_id {
            Some(id) => id,
            None => return,
        };

        let info = match self.mgr.get_entry(id) {
            Ok(info) => info,
            Err(e) => {
                self.set_error(format!("Failed to read Boot{id:04X}: {e}"));
                return;
            }
        };

        let mut entry = match efivar::boot::BootEntry::parse(info.raw_bytes) {
            Ok(e) => e,
            Err(e) => {
                self.set_error(format!("Parse error: {e}"));
                return;
            }
        };

        let mut changed = false;
        if entry.description != self.form_description {
            entry.description = self.form_description.clone();
            changed = true;
        }
        if let Some(ref mut fpl) = entry.file_path_list {
            if fpl.file_path.path != self.form_loader {
                fpl.file_path.path = self.form_loader.clone();
                changed = true;
            }
        }

        if !changed {
            self.set_status("No changes");
            return;
        }

        match self.mgr.update_entry(id, entry) {
            Ok(()) => {
                self.set_status(format!("Updated Boot{id:04X}"));
                self.refresh_entries();
            }
            Err(e) => self.set_error(format!("Update failed: {e}")),
        }
    }

    pub fn open_wizard(&mut self) {
        self.wizard_selected = 0;
        self.view = View::Wizard;
    }

    pub fn apply_wizard_template(&mut self, idx: usize) {
        if let Some(template) = WizardTemplate::ALL.get(idx) {
            self.form_mode = FormMode::Create;
            self.form_field = FormField::Description;
            self.form_description = template.description().to_string();
            self.form_loader = template.loader().to_string();
            self.form_partition.clear();
            self.form_edit_id = None;
            self.view = View::EntryForm;
        }
    }

    pub fn open_backup(&mut self) {
        self.backup_mode = BackupMode::Backup;
        self.backup_path = "backup.json".to_string();
        self.view = View::BackupRestore;
    }

    pub fn open_restore(&mut self) {
        self.backup_mode = BackupMode::Restore;
        self.backup_path = "backup.json".to_string();
        self.view = View::BackupRestore;
    }

    pub fn submit_backup_restore(&mut self) {
        match self.backup_mode {
            BackupMode::Backup => self.do_backup(),
            BackupMode::Restore => {
                let path = self.backup_path.clone();
                self.confirm = Some(ConfirmState {
                    message: format!("Restore from '{path}'? This will overwrite existing entries."),
                    action: ConfirmAction::RestoreBackup(path),
                });
                self.view = View::Confirm;
                return;
            }
        }
        self.view = View::EntryList;
    }

    fn do_backup(&mut self) {
        use crate::core::backup;
        match backup::export(&self.mgr) {
            Ok(data) => match serde_json::to_string_pretty(&data) {
                Ok(json) => match std::fs::write(&self.backup_path, &json) {
                    Ok(()) => {
                        self.set_status(format!(
                            "Backup saved to '{}' ({} entries)",
                            self.backup_path,
                            data.entries.len()
                        ));
                    }
                    Err(e) => self.set_error(format!("Write failed: {e}")),
                },
                Err(e) => self.set_error(format!("Serialize failed: {e}")),
            },
            Err(e) => self.set_error(format!("Export failed: {e}")),
        }
    }

    fn do_restore(&mut self, path: &str) {
        use crate::core::backup;
        let contents = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                self.set_error(format!("Read failed: {e}"));
                return;
            }
        };
        let data: crate::core::backup::BackupData = match serde_json::from_str(&contents) {
            Ok(d) => d,
            Err(e) => {
                self.set_error(format!("Parse failed: {e}"));
                return;
            }
        };
        if let Err(e) = backup::validate(&data) {
            self.set_error(format!("Validation failed: {e}"));
            return;
        }
        match backup::restore(&mut self.mgr, &data) {
            Ok(()) => {
                self.set_status(format!("Restored {} entries from '{path}'", data.entries.len()));
                self.refresh_entries();
            }
            Err(e) => self.set_error(format!("Restore failed: {e}")),
        }
    }
}
