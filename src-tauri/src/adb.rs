// Copyright (c) 2026 Blockmaster2706 <wynter@breedable.men>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use crate::state::{self, DeviceInfo, DeviceState};

pub fn run_adb_command(command: &str) -> Result<String, std::io::Error> {
    use std::process::Command;

    let output = Command::new("adb")
        .args(command.split_whitespace())
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

pub fn listener(app: &tauri::AppHandle) {
    loop {
        let devices = run_adb_command("devices");
        if let Err(error) = devices {
            eprintln!("ADB command failed: {error}");
            continue;
        }
        if let Ok(output) = devices {
            let output_clean = output.replace("List of devices attached\n", "");

            println!("ADB devices:\n{output_clean}");

            let device_state = if output_clean.contains("device") {
                DeviceState::Connected
            } else if output.contains("unauthorized") {
                DeviceState::Unauthorized
            } else if output.contains("bootloader") {
                DeviceState::Bootloader
            } else if output.contains("sideload") {
                DeviceState::Sideload
            } else {
                DeviceState::Disconnected
            };

            let serial = output_clean
                .lines()
                .find_map(|line| {
                    let mut fields = line.split_whitespace();
                    let serial = fields.next()?;
                    let status = fields.next()?;
                    matches!(status, "device" | "unauthorized" | "bootloader" | "sideload")
                        .then(|| serial.to_owned())
                })
                .unwrap_or_default();

            let mut product = String::new();
            let mut buildnumber = String::new();

            if matches!(device_state, DeviceState::Connected) {
                println!("Device is connected, fetching product and build number...");
                product = run_adb_command("shell getprop ro.product.model").unwrap_or_default().trim().to_string();
                buildnumber = run_adb_command("shell getprop ro.build.version.incremental").unwrap_or_default().trim().to_string();
                println!("Product: {product}, Build Number: {buildnumber}");
            };
            
            let device_info = DeviceInfo {
                revision: 0,
                buildnumber,
                serial,
                product,
                state: device_state,
            };

            println!("Device Info: {:?}", device_info);

            state::update(app, |s| *s = device_info).unwrap_or_else(|e| eprintln!("Failed to update state: {e}"));
        }
        
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
