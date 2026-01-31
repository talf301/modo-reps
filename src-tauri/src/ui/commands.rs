use crate::capture::admin::{is_running_as_admin, check_windivert_driver};
use crate::capture::handle::CaptureHandle;
use crate::capture::loop_::capture_loop;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

/// Capture task handle for shutdown control
struct CaptureTask {
    abort_handle: tokio::task::AbortHandle,
    shutdown_tx: broadcast::Sender<()>,
}

impl CaptureTask {
    fn shutdown(&self) {
        // Send shutdown signal via broadcast channel
        let _ = self.shutdown_tx.send(());
        // Also abort the task
        self.abort_handle.abort();
    }
}

/// Capture state (managed via Arc<Mutex<>> for thread-safe access)
pub struct CaptureState {
    is_capturing: bool,
    packet_count: u64,
    bytes_per_second: f64,
    last_packet_time: Option<chrono::DateTime<chrono::Utc>>,
    capture_task: Option<CaptureTask>,
}

impl Default for CaptureState {
    fn default() -> Self {
        Self {
            is_capturing: false,
            packet_count: 0,
            bytes_per_second: 0.0,
            last_packet_time: None,
            capture_task: None,
        }
    }
}

/// Admin privilege check response
#[derive(Serialize, Clone)]
pub struct AdminStatus {
    pub is_admin: bool,
    pub can_capture: bool,
    pub windivert_driver_found: bool,
}

/// Capture status response
#[derive(Serialize, Clone)]
pub struct CaptureStatus {
    pub is_running: bool,
    pub packet_count: u64,
    pub bytes_per_second: f64,
    pub last_packet_time: Option<String>, // ISO 8601 formatted
}

/// Check if the application is running with administrator privileges and WinDivert driver is installed
#[tauri::command]
pub async fn check_admin_privileges() -> Result<AdminStatus, String> {
    use crate::capture::admin::check_windivert_driver;

    let is_admin = is_running_as_admin().map_err(|e| format!("Failed to check admin privileges: {}", e))?;
    let driver_found = check_windivert_driver().map_err(|e| e.to_string())?;

    Ok(AdminStatus {
        is_admin,
        can_capture: is_admin && driver_found,
        windivert_driver_found: driver_found,
    })
}

/// Get current capture status
#[tauri::command]
pub async fn get_capture_status(
    state: tauri::State<'_, Arc<Mutex<CaptureState>>>,
) -> Result<CaptureStatus, String> {
    let state_guard = state.lock().await;

    Ok(CaptureStatus {
        is_running: state_guard.is_capturing,
        packet_count: state_guard.packet_count,
        bytes_per_second: state_guard.bytes_per_second,
        last_packet_time: state_guard.last_packet_time.map(|dt| dt.to_rfc3339()),
    })
}

/// Start packet capture
#[tauri::command]
pub async fn start_capture(
    state: tauri::State<'_, Arc<Mutex<CaptureState>>>,
) -> Result<CaptureStatus, String> {
    // Check if already capturing
    {
        let state_guard = state.lock().await;
        if state_guard.is_capturing {
            return Err("Capture is already running".to_string());
        }
    }

    // Check admin privileges first
    if !is_running_as_admin().map_err(|e| e.to_string())? {
        return Err("Application requires Administrator privileges to capture network traffic. Please restart as Administrator.".to_string());
    }

    // Create WinDivert handle
    let handle = CaptureHandle::new()
        .map_err(|e| format!("Failed to initialize WinDivert: {}", e))?;

    // Create shutdown channel
    let (shutdown_tx, _shutdown_rx) = broadcast::channel(1);
    let shutdown_tx_for_capture = shutdown_tx.clone();
    let shutdown_tx = shutdown_tx.clone();

    // Start capture loop
    let (_packet_rx, abort_handle) = capture_loop(
        handle.clone_handle(),
        shutdown_tx_for_capture,
    );

    // Update state
    let mut state_guard = state.lock().await;
    state_guard.is_capturing = true;
    state_guard.packet_count = 0;
    state_guard.bytes_per_second = 0.0;
    state_guard.last_packet_time = None;
    state_guard.capture_task = Some(CaptureTask {
        abort_handle,
        shutdown_tx,
    });

    Ok(CaptureStatus {
        is_running: true,
        packet_count: 0,
        bytes_per_second: 0.0,
        last_packet_time: None,
    })
}

/// Stop packet capture
#[tauri::command]
pub async fn stop_capture(
    state: tauri::State<'_, Arc<Mutex<CaptureState>>>,
) -> Result<CaptureStatus, String> {
    // Get capture task and shut down
    let capture_task_opt = {
        let mut state_guard = state.lock().await;
        if !state_guard.is_capturing {
            return Err("Capture is not running".to_string());
        }
        state_guard.is_capturing = false;
        state_guard.capture_task.take()
    };

    if let Some(capture_task) = capture_task_opt {
        // Send shutdown signal
        let _ = capture_task.shutdown_tx.send(());

        // Give task time to shutdown gracefully
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Abort if still running
        capture_task.shutdown();
    }

    let state_guard = state.lock().await;
    Ok(CaptureStatus {
        is_running: false,
        packet_count: state_guard.packet_count,
        bytes_per_second: state_guard.bytes_per_second,
        last_packet_time: state_guard.last_packet_time.map(|dt| dt.to_rfc3339()),
    })
}
