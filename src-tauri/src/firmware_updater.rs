use crate::adb;

#[tauri::command]
pub async fn reboot_sideload_from_os(app: tauri::AppHandle) -> Result<(), String> {
    let result = adb::bundled(&app, &["reboot", "bootloader"]);

    if let Err(e) = result {
        reboot_from_os_error(e.clone());
        return Err(e);
    }

    Ok(())
}

fn reboot_from_os_error(op: String) {
    eprintln!("Failed to reboot to sideload mode: {}", op);
}
