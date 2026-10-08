use std::{
    io::{Error, ErrorKind, Read},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use tauri::{path::BaseDirectory, Manager};

pub(crate) fn bundled(
    app: &tauri::AppHandle,
    tool: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<String, String> {
    let executable = app
        .path()
        .resolve(
            format!("adb/{tool}{}", std::env::consts::EXE_SUFFIX),
            BaseDirectory::Resource,
        )
        .map_err(|e| e.to_string())?;
    run(Command::new(executable).args(args), timeout, tool).map_err(|e| e.to_string())
}

pub(crate) fn run(command: &mut Command, timeout: Duration, tool: &str) -> Result<String, Error> {
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
                    _ => Error::new(ErrorKind::TimedOut, format!("{tool} timed out")),
                });
            }
        }
    };
    let stdout = out
        .join()
        .map_err(|_| Error::other(format!("Could not read {tool} output")))??;
    let stderr = err
        .join()
        .map_err(|_| Error::other(format!("Could not read {tool} error")))??;
    if !status.success() {
        return Err(Error::other(format!(
            "{} {}",
            String::from_utf8_lossy(&stdout).trim(),
            String::from_utf8_lossy(&stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&stdout).trim().to_owned())
}
