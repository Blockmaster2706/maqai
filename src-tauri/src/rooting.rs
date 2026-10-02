use crate::{adb, compatibility, state::DeviceState};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    sync::Mutex,
    time::Duration,
};
use tauri::{ipc::Channel, Manager};

const RELEASE_URL: &str = "https://api.github.com/repos/Lumince/singularity/releases/latest";
const MAX_APK_SIZE: u64 = 512 * 1024 * 1024;

#[derive(Default)]
pub struct Installer(pub Mutex<()>);

#[derive(Clone, Serialize)]
pub struct InstallProgress {
    pub message: String,
    pub downloaded: u64,
    pub total: u64,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    size: u64,
    browser_download_url: String,
    digest: Option<String>,
}

fn select_apk(release: &Release) -> Result<&Asset, String> {
    if release.draft || release.prerelease {
        return Err("GitHub did not return a stable Singularity release. Try again later.".into());
    }
    let assets: Vec<_> = release
        .assets
        .iter()
        .filter(|asset| asset.name.eq_ignore_ascii_case("Singularity.apk"))
        .collect();
    if assets.len() != 1 {
        return Err("The latest release does not contain exactly one Singularity.apk. Check the release page and try again later.".into());
    }
    let asset = assets[0];
    if asset.size == 0 || asset.size > MAX_APK_SIZE {
        return Err("The release reports an invalid APK size. Nothing was installed.".into());
    }
    if !asset
        .browser_download_url
        .starts_with("https://github.com/Lumince/singularity/releases/download/")
    {
        return Err(
            "The APK download URL is not from the Singularity project. Nothing was installed."
                .into(),
        );
    }
    let digest = asset
        .digest
        .as_deref()
        .and_then(|s| s.strip_prefix("sha256:"));
    if !digest.is_some_and(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())) {
        return Err(
            "GitHub did not provide a valid APK checksum. Try again later; nothing was installed."
                .into(),
        );
    }
    Ok(asset)
}

fn transfer(
    mut source: impl Read,
    destination: &mut impl Write,
    asset: &Asset,
    mut progress: impl FnMut(u64),
) -> Result<(), String> {
    let mut hash = Sha256::new();
    let mut downloaded = 0u64;
    let mut header = Vec::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = source.read(&mut buffer).map_err(|e| {
            format!(
                "The APK download was interrupted: {e}. Check your internet connection and retry."
            )
        })?;
        if count == 0 {
            break;
        }
        downloaded += count as u64;
        if downloaded > asset.size || downloaded > MAX_APK_SIZE {
            return Err("The download is larger than GitHub reported. Nothing was installed; retry the download.".into());
        }
        header.extend(buffer[..count].iter().take(4 - header.len()).copied());
        hash.update(&buffer[..count]);
        destination.write_all(&buffer[..count]).map_err(|e| {
            format!("Could not save the APK: {e}. Check free disk space and retry.")
        })?;
        progress(downloaded);
    }
    if downloaded != asset.size {
        return Err(
            "The APK download is incomplete. Check your internet connection and retry.".into(),
        );
    }
    if header != b"PK\x03\x04" {
        return Err("The downloaded file is not an APK archive. Nothing was installed.".into());
    }
    let actual = format!("sha256:{:x}", hash.finalize());
    if asset
        .digest
        .as_deref()
        .map(str::to_ascii_lowercase)
        .as_deref()
        != Some(actual.as_str())
    {
        return Err("The APK checksum does not match GitHub's release. Nothing was installed; retry the download.".into());
    }
    destination
        .flush()
        .map_err(|e| format!("Could not save the APK: {e}"))?;
    Ok(())
}

fn check_device(
    app: &tauri::AppHandle,
    serial: &str,
    model: &str,
    build: &str,
) -> Result<(), String> {
    let current = adb::inspect(app).map_err(|e| format!("Could not read the headset: {e}"))?;
    match current.state {
        DeviceState::Disconnected => return Err("No headset is connected. Reconnect a USB data cable, wake the Quest, and retry.".into()),
        DeviceState::Unauthorized => return Err("USB debugging is not authorized. Put on the Quest and select Always allow from this computer, then Allow.".into()),
        DeviceState::Offline => return Err("The Quest is offline. Wake it, reconnect USB, and restart ADB if needed.".into()),
        DeviceState::Connected => {},
        _ => return Err("The Quest is in bootloader or sideload mode. Start Horizon OS normally before installing.".into()),
    }
    if serial.is_empty()
        || current.serial != serial
        || current.product != model
        || current.buildnumber != build
    {
        return Err("The connected headset or firmware changed. Return to the connection step and try again.".into());
    }
    if !matches!(
        current.product.as_str(),
        "Quest 2" | "Quest Pro" | "Quest 3" | "Quest 3S"
    ) {
        return Err(format!("{} detected. Disconnect this device and connect a Meta Quest 2, Quest Pro, Quest 3, or Quest 3S. Nothing was installed.", current.product));
    }
    if !compatibility::supported(&current.product, &current.buildnumber) {
        return Err(format!(
            "{} firmware {} is outside the supported Singularity range. Nothing was installed.",
            current.product, current.buildnumber
        ));
    }
    if adb::bundled(app, &["-d", "get-serialno"])? != serial {
        return Err("Use a wired USB connection to the selected Quest. Disconnect other USB Android devices.".into());
    }
    Ok(())
}

fn network_error(context: &str, error: reqwest::Error) -> String {
    if error.status() == Some(reqwest::StatusCode::FORBIDDEN)
        || error.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS)
    {
        return "GitHub temporarily refused the request or its rate limit was reached. Wait a few minutes and retry.".into();
    }
    format!("{context}: {error}. Check your internet connection, proxy or firewall, then retry.")
}

fn install_error(message: String) -> String {
    let help = if message.contains("INSTALL_FAILED_ALREADY_EXISTS") {
        "Singularity is already installed. Continue with the existing app, or uninstall it in the headset before retrying."
    } else if message.contains("INSTALL_FAILED_UPDATE_INCOMPATIBLE") {
        "An existing Singularity app has a different signature. Uninstall it in the headset before retrying."
    } else if message.contains("INSTALL_FAILED_INSUFFICIENT_STORAGE") {
        "Free some storage on the headset and retry."
    } else if message.contains("unauthorized") {
        "Approve USB debugging inside the headset and retry."
    } else if message.contains("no devices")
        || message.contains("device offline")
        || message.contains("device not found")
    {
        "Reconnect the USB data cable, wake and authorize the headset, then retry."
    } else {
        "Keep the headset awake and connected, check the troubleshooting steps, and retry."
    };
    format!("Singularity installation failed. {help}\nADB details: {message}")
}

#[tauri::command]
pub async fn install_singularity(
    app: tauri::AppHandle,
    serial: String,
    model: String,
    build: String,
    on_progress: Channel<InstallProgress>,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let installer = app.state::<Installer>();
        let _guard = installer
            .0
            .try_lock()
            .map_err(|_| "An installation or ADB restart is already in progress.")?;
        let report = |message: String, downloaded, total| {
            let _ = on_progress.send(InstallProgress {
                message,
                downloaded,
                total,
            });
        };
        report("Checking the headset and firmware…".into(), 0, 0);
        check_device(&app, &serial, &model, &build)?;
        let client = Client::builder()
            .user_agent("Maqai/0.1 (Singularity installer)")
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(600))
            .build()
            .map_err(|e| format!("Could not start the download client: {e}"))?;
        report("Finding the latest Singularity release…".into(), 0, 0);
        let release = client
            .get(RELEASE_URL)
            .timeout(Duration::from_secs(30))
            .send()
            .and_then(|r| r.error_for_status())
            .map_err(|e| network_error("Could not fetch the latest release", e))?
            .json::<Release>()
            .map_err(|e| network_error("Could not read GitHub's release information", e))?;
        let asset = select_apk(&release)?;
        let cache = app.path().app_cache_dir().map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&cache)
            .map_err(|e| format!("Could not create the download folder: {e}"))?;
        let mut file = tempfile::Builder::new()
            .prefix("singularity-")
            .suffix(".apk")
            .tempfile_in(cache)
            .map_err(|e| format!("Could not save the download: {e}"))?;
        report(
            format!("Downloading Singularity {}…", release.tag_name),
            0,
            asset.size,
        );
        let response = client
            .get(&asset.browser_download_url)
            .send()
            .and_then(|r| r.error_for_status())
            .map_err(|e| network_error("Could not download Singularity", e))?;
        let mut last_percent = 0;
        transfer(response, file.as_file_mut(), asset, |downloaded| {
            let percent = downloaded * 100 / asset.size;
            if percent != last_percent {
                last_percent = percent;
                report(
                    format!("Downloading Singularity {}…", release.tag_name),
                    downloaded,
                    asset.size,
                );
            }
        })?;
        report(
            "Download verified. Checking the USB connection…".into(),
            asset.size,
            asset.size,
        );
        check_device(&app, &serial, &model, &build)?;
        let path = file
            .path()
            .to_str()
            .ok_or("The download path could not be read.")?;
        report(
            "Installing Singularity on your Quest. Keep USB connected…".into(),
            0,
            0,
        );
        let output = adb::bundled(&app, &["-s", &serial, "install", "-r", "-g", path])
            .map_err(install_error)?;
        if !output.lines().any(|line| line.trim() == "Success") {
            return Err(install_error(output));
        }
        report("Singularity installed successfully.".into(), 0, 0);
        Ok(release.tag_name)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn restart_adb(app: tauri::AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let installer = app.state::<Installer>();
        let _guard = installer
            .0
            .try_lock()
            .map_err(|_| "Wait for the installation to finish before restarting ADB.")?;
        adb::bundled(&app, &["kill-server"])?;
        adb::bundled(&app, &["start-server"])?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

fn confirms_root(output: &str) -> bool {
    output.lines().any(|line| {
        matches!(
            line.split_whitespace().next(),
            Some("uid=0(root)" | "uid=0")
        )
    })
}

#[tauri::command]
pub async fn verify_root(app: tauri::AppHandle, serial: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let installer = app.state::<Installer>();
        let _guard = installer.0.try_lock().map_err(|_| "Wait for the current ADB operation to finish, then retry.")?;
        let current = adb::inspect(&app).map_err(|e| format!("Could not read the headset: {e}. Reconnect USB and retry."))?;
        match current.state {
            DeviceState::Unauthorized => return Err("Approve USB debugging inside the Quest, then retry verification.".into()),
            DeviceState::Offline | DeviceState::Disconnected => return Err("The Quest is not ready after reboot. Wait for it to start, reconnect USB, and retry.".into()),
            DeviceState::Connected => {},
            _ => return Err("Start the Quest normally before verifying root.".into()),
        }
        if serial.is_empty() || current.serial != serial {
            return Err("Reconnect the same Quest used for this rooting session. No root check was run on the other device.".into());
        }
        if !matches!(current.product.as_str(), "Quest 2" | "Quest Pro" | "Quest 3" | "Quest 3S") {
            return Err("Root verification is available for Meta Quest headsets only.".into());
        }
        let output = adb::bundled(&app, &["-s", &serial, "shell", "su", "-c", "id"])
            .map_err(|e| format!("Root access is not verified. If a Shell or ADB permission prompt appears in the headset, select Grant and retry. Check Singularity-Magisk permissions if no prompt appears.\nADB details: {e}"))?;
        if !confirms_root(&output) {
            return Err(format!("The shell did not report UID 0, so root access is not verified. Grant Shell or ADB access in Singularity-Magisk, then retry.\nADB output: {output}"));
        }
        Ok(output)
    }).await.map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_verification_requires_a_root_uid() {
        assert!(confirms_root("uid=0(root) gid=0(root) groups=0(root)"));
        assert!(confirms_root("uid=0 gid=0"));
        assert!(!confirms_root("uid=2000(shell) gid=2000(shell)"));
        assert!(!confirms_root("error: expected uid=0(root)"));
        assert!(!confirms_root("uid=0(rooted) gid=0"));
        assert!(!confirms_root(""));
    }

    fn fixture(bytes: &[u8]) -> Asset {
        Asset {
            name: "Singularity.apk".into(),
            size: bytes.len() as u64,
            browser_download_url:
                "https://github.com/Lumince/singularity/releases/download/v1/Singularity.apk".into(),
            digest: Some(format!("sha256:{:x}", Sha256::digest(bytes))),
        }
    }

    #[test]
    fn verifies_complete_download_and_checksum() {
        let bytes = b"PK\x03\x04test APK";
        let mut asset = fixture(bytes);
        let mut output = Vec::new();
        transfer(&bytes[..], &mut output, &asset, |_| {}).unwrap();
        assert_eq!(output, bytes);
        assert!(transfer(&bytes[..3], &mut Vec::new(), &asset, |_| {}).is_err());
        asset.digest = Some(format!("sha256:{}", "0".repeat(64)));
        assert!(transfer(&bytes[..], &mut Vec::new(), &asset, |_| {}).is_err());
    }

    #[test]
    fn rejects_missing_ambiguous_and_untrusted_assets() {
        let mut release = Release {
            tag_name: "v1".into(),
            draft: false,
            prerelease: false,
            assets: vec![fixture(b"PK\x03\x04test")],
        };
        assert!(select_apk(&release).is_ok());
        release.assets.push(fixture(b"PK\x03\x04test"));
        assert!(select_apk(&release).is_err());
        release.assets.pop();
        release.assets[0].browser_download_url = "https://example.com/Singularity.apk".into();
        assert!(select_apk(&release).is_err());
        release.assets.clear();
        assert!(select_apk(&release).is_err());
    }
}
