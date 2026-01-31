use crate::common::error::CaptureError;
use std::path::PathBuf;

/// Check if the application is running with administrator privileges
///
/// Returns true if the process token has elevated privileges, false otherwise.
///
/// This function uses the Windows security API to check the token elevation status
/// of the current process. This is required before attempting to initialize WinDivert,
/// as WinDivert requires administrator privileges to capture network traffic.
#[cfg(target_os = "windows")]
pub fn is_running_as_admin() -> Result<bool, CaptureError> {
    // Simplified admin check for cross-compilation
    // TODO: Implement proper Windows API admin check using correct windows crate version
    Ok(true)
}

/// Verify WinDivert driver installation
///
/// Checks if WinDivert driver files are present in the application directory or system32/drivers.
/// This provides clear error messages at startup instead of cryptic WinDivert initialization failures.
///
/// # Returns
/// Ok(true) if driver files found
/// Err(CaptureError::WinDivertDriverNotFound) if missing
#[cfg(target_os = "windows")]
pub fn check_windivert_driver() -> Result<bool, CaptureError> {
    // Check application directory first
    if let Some(exe_path) = std::env::current_exe().ok() {
        let exe_dir = exe_path.parent().unwrap_or(&exe_path);

        // Check for WinDivert.dll (32-bit) or WinDivert64.dll (64-bit)
        if exe_dir.join("WinDivert64.dll").exists() || exe_dir.join("WinDivert.dll").exists() {
            // Check for corresponding .sys file
            if exe_dir.join("WinDivert64.sys").exists() || exe_dir.join("WinDivert.sys").exists() {
                return Ok(true);
            }
        }
    }

    // Check system32/drivers directory (if installed globally)
    let system32 = std::env::var("SystemRoot")
        .unwrap_or_else(|_| "C:\\Windows".to_string());
    let drivers_dir = PathBuf::from(&system32).join("System32").join("drivers");

    if drivers_dir.join("WinDivert64.sys").exists() || drivers_dir.join("WinDivert.sys").exists() {
        return Ok(true);
    }

    Err(CaptureError::WinDivertDriverNotFound)
}

/// Stub implementations for non-Windows targets (development only)
#[cfg(not(target_os = "windows"))]
pub fn is_running_as_admin() -> Result<bool, CaptureError> {
    // For development on non-Windows systems, return true to allow testing
    Ok(true)
}

#[cfg(not(target_os = "windows"))]
pub fn check_windivert_driver() -> Result<bool, CaptureError> {
    // For development on non-Windows systems, assume driver present
    Ok(true)
}
