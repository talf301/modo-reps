use thiserror::Error;

/// Capture-related errors
#[derive(Error, Debug)]
pub enum CaptureError {
    #[error("Application requires Administrator privileges to capture network traffic. Please restart the application as Administrator.")]
    RequiresAdminPrivileges,

    #[error("Failed to detect administrator privileges: {0}")]
    PrivilegeDetectionFailed(String),

    #[error("WinDivert driver not found. Please download WinDivert 2.2.2-A from https://reqrypt.org/windivert.html and place WinDivert.dll and WinDivert64.sys in the application directory.")]
    WinDivertDriverNotFound,

    #[error("WinDivert driver installation blocked: WinDivert64.sys may be blocked by antivirus. Please add it to the antivirus allowlist.")]
    DriverBlocked,

    #[error("Failed to initialize WinDivert handle: {0}")]
    WinDivertInitFailed(#[from] windivert::error::Error),

    #[error("Packet capture channel error: {0}")]
    ChannelError(String),

    #[error("Capture loop error: {0}")]
    CaptureLoopError(String),
}

/// Implement Into<String> for Tauri command compatibility
impl From<CaptureError> for String {
    fn from(error: CaptureError) -> Self {
        error.to_string()
    }
}
