use efivar::boot::{BootEntry, BootEntryAttributes, BootVarName};
use efivar::efi::{Variable, VariableFlags};
use efivar::VarManager;

use crate::core::BootEntryInfo;
use crate::error::AppError;

/// High-level boot entry manager wrapping a `VarManager` implementation.
///
/// All read and write operations go through this struct to ensure
/// consistent behavior and error handling.
pub struct BootManager {
    manager: Box<dyn VarManager>,
}

impl BootManager {
    pub fn new(manager: Box<dyn VarManager>) -> Self {
        Self { manager }
    }

    /// List all boot entries in BootOrder, with parsed info.
    /// Returns entries in boot-order priority.
    pub fn list_entries(&self) -> Result<Vec<BootEntryInfo>, AppError> {
        let order = self.get_boot_order()?;
        let mut entries = Vec::new();

        for id in &order {
            match self.get_entry(*id) {
                Ok(info) => entries.push(info),
                Err(e) => {
                    tracing::warn!("Failed to read Boot{:04X}: {}", id, e);
                }
            }
        }

        Ok(entries)
    }

    /// Get the current BootOrder as a list of IDs.
    pub fn get_boot_order(&self) -> Result<Vec<u16>, AppError> {
        self.manager.get_boot_order().map_err(Into::into)
    }

    /// Get a single boot entry by ID.
    pub fn get_entry(&self, id: u16) -> Result<BootEntryInfo, AppError> {
        let var = Variable::new(&id.boot_var_name());
        let (data, _flags) = self
            .manager
            .read(&var)
            .map_err(|_| AppError::EntryNotFound { id })?;
        BootEntryInfo::from_raw(id, data)
    }

    /// Get the BootCurrent variable (which entry was booted).
    pub fn get_boot_current(&self) -> Result<Option<u16>, AppError> {
        self.read_u16_var("BootCurrent")
    }

    /// Get the BootNext variable (one-shot next boot override).
    pub fn get_boot_next(&self) -> Result<Option<u16>, AppError> {
        self.read_u16_var("BootNext")
    }

    /// Create a new boot entry and append it to BootOrder.
    pub fn create_entry(&mut self, id: u16, entry: BootEntry) -> Result<(), AppError> {
        self.manager.add_boot_entry(id, entry)?;

        // Append to BootOrder
        let mut order = self.get_boot_order().unwrap_or_default();
        if !order.contains(&id) {
            order.push(id);
            self.manager.set_boot_order(order)?;
        }

        Ok(())
    }

    /// Delete a boot entry and remove it from BootOrder.
    pub fn delete_entry(&mut self, id: u16) -> Result<(), AppError> {
        let var = Variable::new(&id.boot_var_name());
        self.manager
            .delete(&var)
            .map_err(|_| AppError::EntryNotFound { id })?;

        // Remove from BootOrder
        let mut order = self.get_boot_order().unwrap_or_default();
        order.retain(|&x| x != id);
        self.manager.set_boot_order(order)?;

        Ok(())
    }

    /// Update an existing boot entry's raw variable data.
    pub fn update_entry(&mut self, id: u16, entry: BootEntry) -> Result<(), AppError> {
        let var = Variable::new(&id.boot_var_name());
        let bytes = entry.to_bytes();
        self.manager
            .write(&var, VariableFlags::default(), &bytes)?;
        Ok(())
    }

    /// Set the BootOrder to a specific list of IDs.
    pub fn set_boot_order(&mut self, ids: Vec<u16>) -> Result<(), AppError> {
        self.manager.set_boot_order(ids)?;
        Ok(())
    }

    /// Set the BootNext variable for a one-shot boot override.
    pub fn set_boot_next(&mut self, id: u16) -> Result<(), AppError> {
        let var = Variable::new("BootNext");
        let bytes = id.to_le_bytes().to_vec();
        self.manager
            .write(&var, VariableFlags::default(), &bytes)?;
        Ok(())
    }

    /// Toggle the LOAD_OPTION_ACTIVE flag on an entry.
    pub fn toggle_active(&mut self, id: u16) -> Result<bool, AppError> {
        let var = Variable::new(&id.boot_var_name());
        let (data, _flags) = self
            .manager
            .read(&var)
            .map_err(|_| AppError::EntryNotFound { id })?;

        let mut entry = BootEntry::parse(data).map_err(|e| AppError::Parse {
            message: format!("Failed to parse Boot{id:04X}: {e}"),
        })?;

        let new_active = !entry
            .attributes
            .contains(BootEntryAttributes::LOAD_OPTION_ACTIVE);
        entry
            .attributes
            .set(BootEntryAttributes::LOAD_OPTION_ACTIVE, new_active);

        let bytes = entry.to_bytes();
        self.manager
            .write(&var, VariableFlags::default(), &bytes)?;

        Ok(new_active)
    }

    /// Find the next available boot entry ID.
    pub fn next_free_id(&self) -> Result<u16, AppError> {
        let order = self.get_boot_order().unwrap_or_default();
        for id in 0..=0xFFFFu16 {
            if !order.contains(&id) {
                // Also check if the variable exists even if not in BootOrder
                let var = Variable::new(&id.boot_var_name());
                if self.manager.read(&var).is_err() {
                    return Ok(id);
                }
            }
        }
        Err(AppError::Efi {
            message: "No free boot entry IDs available".to_string(),
        })
    }

    /// Read raw variable bytes for a given variable name.
    pub fn read_raw(&self, name: &str) -> Result<Option<Vec<u8>>, AppError> {
        let var = Variable::new(name);
        match self.manager.read(&var) {
            Ok((data, _)) => Ok(Some(data)),
            Err(_) => Ok(None),
        }
    }

    /// Write raw variable bytes.
    pub fn write_raw(&mut self, name: &str, data: &[u8]) -> Result<(), AppError> {
        let var = Variable::new(name);
        self.manager
            .write(&var, VariableFlags::default(), data)?;
        Ok(())
    }

    fn read_u16_var(&self, name: &str) -> Result<Option<u16>, AppError> {
        let var = Variable::new(name);
        match self.manager.read(&var) {
            Ok((data, _)) if data.len() >= 2 => {
                Ok(Some(u16::from_le_bytes([data[0], data[1]])))
            }
            Ok(_) => Ok(None),
            Err(_) => Ok(None),
        }
    }
}
