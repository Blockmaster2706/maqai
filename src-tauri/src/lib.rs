// Copyright (c) 2026 Blockmaster2706 <wynter@breedable.men>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

pub mod adb;
mod compatibility;
pub mod fastboot;
mod firmware_updater;
mod platform_tools;
mod rooting;
pub mod state;
mod updates;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(state::AppState::default())
        .manage(rooting::Installer::default())
        .manage(updates::PendingUpdate::default())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let mut updater = tauri_plugin_updater::Builder::new();
            if let Some(key) = updates::public_key() {
                updater = updater.pubkey(key);
            }
            app.handle().plugin(updater.build())?;
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                adb::listener(&handle);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            state::get_state,
            rooting::install_singularity,
            rooting::restart_adb,
            rooting::verify_root,
            updates::check_app_update,
            updates::install_app_update,
            firmware_updater::reboot_sideload_from_os,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
