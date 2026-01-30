// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod common;
mod capture;
mod ui;

use ui::commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::check_admin_privileges])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
