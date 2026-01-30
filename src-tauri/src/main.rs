// Prevents additional console window on Windows in release, DO NOT REMOVE!!
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
