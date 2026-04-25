#[cfg(test)]
mod tests {
    use efivar::boot::{
        BootEntry, BootEntryAttributes, EFIHardDrive, EFIHardDriveType, FilePath,
        FilePathList,
    };
    use std::str::FromStr;
    use uuid::Uuid;

    fn test_store() -> Box<dyn efivar::VarManager> {
        efivar::file_store(std::env::temp_dir().join(format!(
            "uefibootmgrrs-test-{}.toml",
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

    fn setup_store_with_entries() -> Box<dyn efivar::VarManager> {
        let mut store = test_store();
        let e0 = sample_entry("Windows Boot Manager", r"\EFI\Microsoft\Boot\bootmgfw.efi");
        let e1 = sample_entry("Ubuntu", r"\EFI\ubuntu\shimx64.efi");

        store.add_boot_entry(0, e0).unwrap();
        store.add_boot_entry(1, e1).unwrap();
        store.set_boot_order(vec![0, 1]).unwrap();

        store
    }

    #[test]
    fn test_list_entries() {
        let store = setup_store_with_entries();
        let mgr = uefibootmgrrs::core::BootManager::new(store);

        let entries = mgr.list_entries().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].description, "Windows Boot Manager");
        assert_eq!(entries[1].description, "Ubuntu");
        assert!(entries[0].active);
        assert!(entries[1].active);
    }

    #[test]
    fn test_get_entry() {
        let store = setup_store_with_entries();
        let mgr = uefibootmgrrs::core::BootManager::new(store);

        let entry = mgr.get_entry(0).unwrap();
        assert_eq!(entry.id, 0);
        assert_eq!(entry.description, "Windows Boot Manager");
        assert_eq!(
            entry.file_path.as_deref(),
            Some(r"\EFI\Microsoft\Boot\bootmgfw.efi")
        );
        assert!(entry.partition_guid.is_some());
    }

    #[test]
    fn test_get_entry_not_found() {
        let store = setup_store_with_entries();
        let mgr = uefibootmgrrs::core::BootManager::new(store);

        let result = mgr.get_entry(99);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_entry() {
        let store = setup_store_with_entries();
        let mut mgr = uefibootmgrrs::core::BootManager::new(store);

        let new_entry = sample_entry("Fedora", r"\EFI\fedora\shimx64.efi");
        mgr.create_entry(2, new_entry).unwrap();

        let order = mgr.get_boot_order().unwrap();
        assert!(order.contains(&2));

        let entry = mgr.get_entry(2).unwrap();
        assert_eq!(entry.description, "Fedora");
    }

    #[test]
    fn test_delete_entry() {
        let store = setup_store_with_entries();
        let mut mgr = uefibootmgrrs::core::BootManager::new(store);

        mgr.delete_entry(1).unwrap();

        let order = mgr.get_boot_order().unwrap();
        assert!(!order.contains(&1));
        assert!(mgr.get_entry(1).is_err());
    }

    #[test]
    fn test_set_boot_order() {
        let store = setup_store_with_entries();
        let mut mgr = uefibootmgrrs::core::BootManager::new(store);

        mgr.set_boot_order(vec![1, 0]).unwrap();
        let order = mgr.get_boot_order().unwrap();
        assert_eq!(order, vec![1, 0]);
    }

    #[test]
    fn test_set_boot_next() {
        let store = setup_store_with_entries();
        let mut mgr = uefibootmgrrs::core::BootManager::new(store);

        mgr.set_boot_next(1).unwrap();
        let next = mgr.get_boot_next().unwrap();
        assert_eq!(next, Some(1));
    }

    #[test]
    fn test_toggle_active() {
        let store = setup_store_with_entries();
        let mut mgr = uefibootmgrrs::core::BootManager::new(store);

        // Should be active initially
        let entry = mgr.get_entry(0).unwrap();
        assert!(entry.active);

        // Toggle off
        let new_active = mgr.toggle_active(0).unwrap();
        assert!(!new_active);

        let entry = mgr.get_entry(0).unwrap();
        assert!(!entry.active);

        // Toggle back on
        let new_active = mgr.toggle_active(0).unwrap();
        assert!(new_active);
    }

    #[test]
    fn test_next_free_id() {
        let store = setup_store_with_entries();
        let mgr = uefibootmgrrs::core::BootManager::new(store);

        let free_id = mgr.next_free_id().unwrap();
        assert_eq!(free_id, 2);
    }

    #[test]
    fn test_update_entry() {
        let store = setup_store_with_entries();
        let mut mgr = uefibootmgrrs::core::BootManager::new(store);

        let info = mgr.get_entry(0).unwrap();
        let mut entry = BootEntry::parse(info.raw_bytes).unwrap();
        entry.description = "Modified Windows".to_string();
        mgr.update_entry(0, entry).unwrap();

        let updated = mgr.get_entry(0).unwrap();
        assert_eq!(updated.description, "Modified Windows");
    }

    #[test]
    fn test_backup_export_roundtrip() {
        let store = setup_store_with_entries();
        let mgr = uefibootmgrrs::core::BootManager::new(store);

        let backup = uefibootmgrrs::core::backup::export(&mgr).unwrap();

        assert_eq!(backup.version, 1);
        assert_eq!(backup.boot_order, vec![0, 1]);
        assert_eq!(backup.entries.len(), 2);
        assert_eq!(backup.entries[0].variable_name, "Boot0000");
        assert_eq!(backup.entries[1].variable_name, "Boot0001");

        // Decoded metadata should be present
        let decoded = backup.entries[0].decoded.as_ref().unwrap();
        assert_eq!(decoded.description, "Windows Boot Manager");
        assert!(decoded.active);

        // Validate the backup
        uefibootmgrrs::core::backup::validate(&backup).unwrap();

        // JSON roundtrip
        let json = serde_json::to_string_pretty(&backup).unwrap();
        let restored: uefibootmgrrs::core::backup::BackupData =
            serde_json::from_str(&json).unwrap();
        uefibootmgrrs::core::backup::validate(&restored).unwrap();
    }

    #[test]
    fn test_backup_restore() {
        // Create source store and export backup
        let src_store = setup_store_with_entries();
        let src_mgr = uefibootmgrrs::core::BootManager::new(src_store);
        let backup = uefibootmgrrs::core::backup::export(&src_mgr).unwrap();

        // Create fresh destination store and restore
        let dst_store = test_store();
        let mut dst_mgr = uefibootmgrrs::core::BootManager::new(dst_store);

        uefibootmgrrs::core::backup::restore(&mut dst_mgr, &backup).unwrap();

        // Verify restored entries
        let order = dst_mgr.get_boot_order().unwrap();
        assert_eq!(order, vec![0, 1]);

        let entry0 = dst_mgr.get_entry(0).unwrap();
        assert_eq!(entry0.description, "Windows Boot Manager");

        let entry1 = dst_mgr.get_entry(1).unwrap();
        assert_eq!(entry1.description, "Ubuntu");
    }

    #[test]
    fn test_backup_validate_bad_version() {
        let backup = uefibootmgrrs::core::backup::BackupData {
            version: 99,
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            boot_order: vec![],
            boot_next: None,
            entries: vec![],
        };

        let result = uefibootmgrrs::core::backup::validate(&backup);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_boot_id() {
        assert_eq!(uefibootmgrrs::error::parse_boot_id("0001").unwrap(), 1);
        assert_eq!(uefibootmgrrs::error::parse_boot_id("FFFF").unwrap(), 0xFFFF);
        assert_eq!(uefibootmgrrs::error::parse_boot_id("0x0A").unwrap(), 10);
        assert!(uefibootmgrrs::error::parse_boot_id("ZZZZ").is_err());
        assert!(uefibootmgrrs::error::parse_boot_id("").is_err());
    }
}
