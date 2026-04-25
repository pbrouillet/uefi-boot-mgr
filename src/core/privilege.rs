use crate::error::AppError;

/// Check whether the current process has sufficient privileges to access EFI variables.
pub fn check_privileges() -> Result<(), AppError> {
    #[cfg(target_os = "windows")]
    {
        check_windows_admin()?;
    }

    #[cfg(target_os = "linux")]
    {
        check_linux_efivarfs()?;
    }

    Ok(())
}

/// Returns a human-readable hint for how to get the required privileges.
pub fn privilege_hint() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "Run this program as Administrator (right-click → Run as administrator)"
    }

    #[cfg(target_os = "linux")]
    {
        "Run this program as root, or ensure /sys/firmware/efi/efivars is mounted"
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        "EFI variable access is not supported on this platform"
    }
}

#[cfg(target_os = "windows")]
fn check_windows_admin() -> Result<(), AppError> {
    #[link(name = "shell32")]
    unsafe extern "system" {
        fn IsUserAnAdmin() -> i32;
    }

    // SAFETY: IsUserAnAdmin is a standard Win32 API with no preconditions.
    if unsafe { IsUserAnAdmin() } == 0 {
        return Err(AppError::Privilege {
            message: "This program must be run as Administrator to access EFI variables"
                .to_string(),
        });
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn check_linux_efivarfs() -> Result<(), AppError> {
    use std::path::Path;

    let efivarfs = Path::new("/sys/firmware/efi/efivars");
    if !efivarfs.exists() {
        return Err(AppError::Privilege {
            message:
                "EFI variables not available. Ensure you booted in UEFI mode and efivarfs is mounted"
                    .to_string(),
        });
    }

    // Check read access
    if std::fs::read_dir(efivarfs).is_err() {
        return Err(AppError::Privilege {
            message: "Cannot read /sys/firmware/efi/efivars — insufficient permissions".to_string(),
        });
    }

    Ok(())
}
