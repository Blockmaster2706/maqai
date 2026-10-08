use crate::platform_tools;
use std::time::Duration;

pub fn bundled(app: &tauri::AppHandle, args: &[&str], timeout: Duration) -> Result<String, String> {
    platform_tools::bundled(app, "fastboot", args, timeout)
}

pub fn devices(app: &tauri::AppHandle) -> Result<Vec<String>, String> {
    let output = bundled(app, &["devices"], Duration::from_secs(15))?;
    Ok(output
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let serial = fields.next()?;
            (fields.next() == Some("fastboot")).then(|| serial.to_owned())
        })
        .collect())
}
