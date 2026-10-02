// Copyright (c) 2026 Blockmaster2706 <wynter@breedable.men>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use crate::{
    compatibility,
    state::{self, DeviceInfo, DeviceState},
};
use std::{
    io::{Error, ErrorKind, Read},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use tauri::{path::BaseDirectory, Manager};

pub fn run_adb_command(command: &str) -> Result<String, Error> {
    run(
        Command::new("adb").args(command.split_whitespace()),
        Duration::from_secs(15),
    )
}

pub(crate) fn bundled(app: &tauri::AppHandle, args: &[&str]) -> Result<String, String> {
    let executable = app
        .path()
        .resolve(
            format!("adb/adb{}", std::env::consts::EXE_SUFFIX),
            BaseDirectory::Resource,
        )
        .map_err(|e| e.to_string())?;
    let timeout = if args.contains(&"install") { 180 } else { 15 };
    run(
        Command::new(executable).args(args),
        Duration::from_secs(timeout),
    )
    .map_err(|e| e.to_string())
}

fn run(command: &mut Command, timeout: Duration) -> Result<String, Error> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let out = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).map(|_| bytes)
    });
    let err = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).map(|_| bytes)
    });
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(50)),
            result => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(match result {
                    Err(error) => error,
                    _ => Error::new(
                        ErrorKind::TimedOut,
                        "ADB timed out. Wake the headset, check the USB cable, and retry.",
                    ),
                });
            }
        }
    };
    let stdout = out
        .join()
        .map_err(|_| Error::other("Could not read ADB output"))??;
    let stderr = err
        .join()
        .map_err(|_| Error::other("Could not read ADB error"))??;
    if !status.success() {
        return Err(Error::other(format!(
            "{} {}",
            String::from_utf8_lossy(&stdout).trim(),
            String::from_utf8_lossy(&stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&stdout).trim().to_owned())
}

fn parse_devices(output: &str) -> Result<Option<(String, DeviceState)>, String> {
    let rows: Vec<_> = output.lines().filter_map(|line| {
        let mut fields = line.split_whitespace();
        let serial = fields.next()?;
        let status = fields.next()?;
        if serial == "List" || serial.starts_with('*') || serial.contains(':') || serial.starts_with("emulator-") {
            return None;
        }
        let state = match status {
            "device" => DeviceState::Connected,
            "unauthorized" => DeviceState::Unauthorized,
            "offline" => DeviceState::Offline,
            "bootloader" => DeviceState::Bootloader,
            "sideload" => DeviceState::Sideload,
            "no" if line.contains("no permissions") => return Some(Err("USB permission denied. Install your distribution's Android udev rules, log out and back in, then reconnect the headset.".into())),
            _ => return None,
        };
        Some(Ok((serial.to_owned(), state)))
    }).collect::<Result<_, String>>()?;
    if rows.len() > 1 {
        return Err("More than one USB device is connected. Disconnect extra phones or headsets so only your Quest remains.".into());
    }
    Ok(rows.into_iter().next())
}

pub(crate) fn inspect(app: &tauri::AppHandle) -> Result<DeviceInfo, String> {
    let output = bundled(app, &["devices", "-l"])?;
    let Some((serial, device_state)) = parse_devices(&output)? else {
        return Ok(DeviceInfo::default());
    };
    let mut info = DeviceInfo {
        serial,
        state: device_state,
        ..Default::default()
    };
    if matches!(info.state, DeviceState::Connected) {
        info.product = bundled(
            app,
            &["-s", &info.serial, "shell", "getprop", "ro.product.model"],
        )?;
        info.buildnumber = bundled(
            app,
            &[
                "-s",
                &info.serial,
                "shell",
                "getprop",
                "ro.build.version.incremental",
            ],
        )?;
        info.firmware_compatible = compatibility::supported(&info.product, &info.buildnumber);
    }
    Ok(info)
}

pub fn listener(app: &tauri::AppHandle) {
    loop {
        let mut device = match inspect(app) {
            Ok(device) => device,
            Err(error) => DeviceInfo {
                error,
                ..Default::default()
            },
        };
        device.platform = std::env::consts::OS.to_owned();
        if let Err(error) = state::update(app, |s| *s = device) {
            eprintln!("Failed to update device state: {error}");
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_exact_statuses_and_windows_line_endings() {
        let output = "List of devices attached\r\ndevice123 unauthorized usb:1-2\r\n";
        let (serial, state) = parse_devices(output).unwrap().unwrap();
        assert_eq!(serial, "device123");
        assert!(matches!(state, DeviceState::Unauthorized));
        assert!(parse_devices("List of devices attached\n\n")
            .unwrap()
            .is_none());
        assert!(matches!(
            parse_devices("quest offline").unwrap().unwrap().1,
            DeviceState::Offline
        ));
    }

    #[test]
    fn rejects_multiple_usb_devices_and_reports_permissions() {
        assert!(parse_devices("quest device\nphone device")
            .unwrap_err()
            .contains("More than one"));
        assert!(
            parse_devices("quest no permissions (user in plugdev group)")
                .unwrap_err()
                .contains("permission")
        );
        assert!(parse_devices("192.168.0.2:5555 device\nquest device")
            .unwrap()
            .is_some());
    }
}
