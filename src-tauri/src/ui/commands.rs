use crate::capture::admin::{is_running_as_admin, check_windivert_driver};
use crate::common::error::CaptureError;
use serde::Serialize;

/// Admin privilege check response
#[derive(Serialize, Clone)]
pub struct AdminStatus {
    pub is_admin: bool,
    pub can_capture: bool,
    pub windivert_driver_found: bool,
}

/// Check if the application is running with administrator privileges and WinDivert driver is installed
///
/// This command allows the frontend to display appropriate UI (e.g., UAC shield icon, driver status)
/// and guide users to restart as Administrator or download WinDivert if needed.
#[tauri::command]
pub async fn check_admin_privileges() -> Result<AdminStatus, String> {
    let is_admin = is_running_as_admin().map_err(|e| e.to_string())?;
    let driver_found = check_windivert_driver().map_err(|e| e.to_string())?;

    Ok(AdminStatus {
        is_admin,
        can_capture: is_admin && driver_found,
        windivert_driver_found: driver_found,
    })
}
