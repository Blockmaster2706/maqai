// Copyright (c) 2026 Blockmaster2706 <wynter@breedable.men>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use std::{sync::Mutex, time::Duration};
use tauri::Manager;
use tauri_plugin_updater::{Update, UpdaterExt};

#[derive(Default)]
pub struct PendingUpdate(pub Mutex<Option<Update>>);

#[derive(serde::Serialize)]
pub struct UpdateInfo {
    version: String,
    notes: String,
}

pub fn public_key() -> Option<&'static str> {
    option_env!("TAURI_UPDATER_PUBLIC_KEY")
        .map(str::trim)
        .filter(|key| !key.is_empty())
}

#[tauri::command]
pub async fn check_app_update(app: tauri::AppHandle) -> Result<Option<UpdateInfo>, String> {
    if public_key().is_none() {
        return Err("Updates are not configured in this development build.".into());
    }
    let update = app
        .updater_builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| {
            format!(
                "Could not check for updates: {e}. Check your internet connection and try again."
            )
        })?;
    let info = update.as_ref().map(|update| UpdateInfo {
        version: update.version.clone(),
        notes: update.body.clone().unwrap_or_default(),
    });
    *app.state::<PendingUpdate>()
        .0
        .lock()
        .map_err(|e| e.to_string())? = update;
    Ok(info)
}

#[tauri::command]
pub async fn install_app_update(app: tauri::AppHandle) -> Result<(), String> {
    let update = app
        .state::<PendingUpdate>()
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .take()
        .ok_or("Check for updates before installing.")?;
    let bytes = match update.download(|_, _| {}, || {}).await {
        Ok(bytes) => bytes,
        Err(error) => {
            *app.state::<PendingUpdate>()
                .0
                .lock()
                .map_err(|e| e.to_string())? = Some(update);
            return Err(format!("Could not download or verify the update: {error}. Nothing was installed; retry when online."));
        }
    };
    tauri::async_runtime::spawn_blocking(move || {
        let installer = app.state::<crate::rooting::Installer>();
        let _guard = match installer.0.try_lock() {
            Ok(guard) => guard,
            Err(_) => {
                *app.state::<PendingUpdate>()
                    .0
                    .lock()
                    .map_err(|e| e.to_string())? = Some(update);
                return Err("An ADB operation is running. Finish it before updating Maqai.".into());
            }
        };
        if let Err(error) = update.install(bytes) {
            *app.state::<PendingUpdate>()
                .0
                .lock()
                .map_err(|e| e.to_string())? = Some(update);
            return Err(format!("Could not install the update: {error}"));
        }
        app.restart();
    })
    .await
    .map_err(|e| e.to_string())?
}
