---
phase: 01-capture-infrastructure-proof-of-concept
plan: 04A
type: execute
wave: 2
depends_on: ["01-01", "01-02"]
files_modified: [src-tauri/src/ui/commands.rs, src-tauri/src/main.rs]
autonomous: true
user_setup: []

must_haves:
  truths:
    - "UI can start and stop capture"
    - "Frontend can query capture status"
    - "Admin status checks respond correctly"
  artifacts:
    - path: "src-tauri/src/ui/commands.rs"
      provides: "Tauri commands for capture control"
      contains: "CaptureState|CaptureStatus|start_capture|stop_capture|get_capture_status"
    - path: "src-tauri/src/main.rs"
      provides: "Tauri application entry point with command registration"
      contains: "invoke_handler|manage"
  key_links:
    - from: "src-tauri/src/ui/commands.rs"
      to: "src-tauri/src/capture/admin.rs"
      via: "admin privilege check"
      pattern: "is_running_as_admin"
    - from: "src-tauri/src/main.rs"
      to: "src-tauri/src/ui/commands.rs"
      via: "command registration"
      pattern: "invoke_handler"
---

<objective>
Define capture status types and Tauri commands for capture control.

Purpose: Provide backend command structure for capture status management before implementing frontend UI.
Output: Capture status types, Tauri commands for start/stop/status, and command registration in main.rs.
</objective>

<execution_context>
@~/.config/opencode/get-shit-done/workflows/execute-plan.md
@~/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/REQUIREMENTS.md

# Prior Plan 01-01: Tauri project initialized with vanilla TypeScript template
# Prior Plan 01-02: Admin privilege and driver check Tauri command created (check_admin_privileges)
# This plan creates the capture status types and Tauri commands (backend only)
</context>

<tasks>

<task type="auto">
  <name>Task 1: Define capture status types and Tauri commands</name>
  <files>src-tauri/src/ui/commands.rs</files>
  <action>
Update src-tauri/src/ui/commands.rs with capture status types and commands:

```rust
use crate::capture::admin::{is_running_as_admin, check_windivert_driver};
use crate::common::error::CaptureError;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Capture state (managed via Arc<Mutex<>> for thread-safe access)
pub struct CaptureState {
    is_capturing: bool,
    packet_count: u64,
    bytes_per_second: f64,
    last_packet_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for CaptureState {
    fn default() -> Self {
        Self {
            is_capturing: false,
            packet_count: 0,
            bytes_per_second: 0.0,
            last_packet_time: None,
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
    let is_admin = is_running_as_admin().map_err(|e| e.to_string())?;
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

/// Start packet capture (placeholder - full implementation in Plan 05)
#[tauri::command]
pub async fn start_capture(
    state: tauri::State<'_, Arc<Mutex<CaptureState>>>,
) -> Result<CaptureStatus, String> {
    // Check admin privileges first
    if !is_running_as_admin().map_err(|e| e.to_string())? {
        return Err("Application requires Administrator privileges to capture network traffic. Please restart as Administrator.".to_string());
    }

    // Check WinDivert driver
    if !check_windivert_driver().map_err(|e| e.to_string())? {
        return Err("WinDivert driver not found. Please download WinDivert 2.2.2-A from https://reqrypt.org/windivert.html and place WinDivert.dll and WinDivert64.sys in the application directory.".to_string());
    }

    // Placeholder: Full capture loop implementation in Plan 05
    // For now, just mark as capturing
    let mut state_guard = state.lock().await;
    state_guard.is_capturing = true;
    state_guard.packet_count = 0;

    Ok(CaptureStatus {
        is_running: true,
        packet_count: 0,
        bytes_per_second: 0.0,
        last_packet_time: None,
    })
}

/// Stop packet capture (placeholder - full implementation in Plan 05)
#[tauri::command]
pub async fn stop_capture(
    state: tauri::State<'_, Arc<Mutex<CaptureState>>>,
) -> Result<CaptureStatus, String> {
    // Placeholder: Full shutdown implementation in Plan 05
    let mut state_guard = state.lock().await;
    state_guard.is_capturing = false;

    Ok(CaptureStatus {
        is_running: false,
        packet_count: state_guard.packet_count,
        bytes_per_second: state_guard.bytes_per_second,
        last_packet_time: state_guard.last_packet_time.map(|dt| dt.to_rfc3339()),
    })
}
```

Add chrono dependency for timestamp handling:
```bash
cargo add chrono --features "serde"
```

DO NOT implement full capture loop here - that's Plan 05. This plan creates the command signatures and state management.
  </action>
  <verify>grep -E "(CaptureState|CaptureStatus|start_capture|stop_capture|get_capture_status)" src-tauri/src/ui/commands.rs</verify>
  <done>Capture state types and Tauri commands defined with placeholder implementations</done>
</task>

<task type="auto">
  <name>Task 2: Register commands in Tauri main.rs</name>
  <files>src-tauri/src/main.rs</files>
  <action>
Update src-tauri/src/main.rs to register capture commands and initialize shared state:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::ui::commands::{CaptureState, check_admin_privileges, get_capture_status, start_capture, stop_capture};
use std::sync::Arc;
use tokio::sync::Mutex;

mod capture;
mod common;
mod ui;

fn main() {
    // Initialize shared capture state
    let capture_state = Arc::new(Mutex::new(CaptureState::default()));

    tauri::Builder::default()
        .manage(capture_state)
        .invoke_handler(tauri::generate_handler![
            check_admin_privileges,
            get_capture_status,
            start_capture,
            stop_capture
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

DO NOT forget to register all commands in invoke_handler or they won't be available to the frontend.
  </action>
  <verify>grep -E "(invoke_handler|check_admin_privileges|get_capture_status|start_capture|stop_capture)" src-tauri/src/main.rs</verify>
  <done>All capture commands registered in Tauri main function with shared state management</done>
</task>

</tasks>

<verification>
- Capture state types defined with shared Arc<Mutex<CaptureState>>
- Tauri commands registered (start_capture, stop_capture, get_capture_status)
- Admin and driver checks integrated into start_capture command
- Commands accessible from frontend via invoke_handler
- Project compiles without errors
</verification>

<success_criteria>
Capture status types and Tauri commands implemented with placeholder implementations, ready for frontend integration in Plan 04B.
</success_criteria>

<output>
After completion, create `.planning/phases/01-capture-infrastructure-proof-of-concept/01-04A-SUMMARY.md`
</output>
