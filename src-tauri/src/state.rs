// Copyright (c) 2026 Blockmaster2706 <wynter@breedable.men>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use std::sync::Mutex;
use tauri::{Emitter, Manager};

#[derive(Clone, Default, serde::Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DeviceState {
    Connected,
    Unauthorized,
    Offline,
    Bootloader,
    Sideload,
    #[default]
    Disconnected,
}

#[derive(Clone, Default, serde::Serialize, Debug)]
pub struct DeviceInfo {
    pub revision: u64,
    pub buildnumber: String,
    pub serial: String,
    pub product: String,
    pub state: DeviceState,
    pub firmware_compatible: bool,
    pub error: String,
    pub platform: String,
}

#[derive(Default)]
pub struct AppState(Mutex<DeviceInfo>);

#[tauri::command]
pub fn get_state(state: tauri::State<'_, AppState>) -> Result<DeviceInfo, String> {
    state
        .0
        .lock()
        .map(|state| state.clone())
        .map_err(|e| e.to_string())
}

pub fn update(app: &tauri::AppHandle, change: impl FnOnce(&mut DeviceInfo)) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut snapshot = state.0.lock().map_err(|e| e.to_string())?;
    let revision = snapshot.revision + 1;
    change(&mut snapshot);
    snapshot.revision = revision;
    app.emit("state-changed", snapshot.clone())
        .map_err(|e| e.to_string())
}
