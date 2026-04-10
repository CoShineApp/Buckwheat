use std::path::Path;
use std::process::{Child, Command};

/// Spawn OBS in portable mode, minimized to tray.
pub fn spawn_obs(obs_exe: &Path, config_dir: &Path) -> Result<Child, String> {
    log::info!("Launching OBS: {:?} with config {:?}", obs_exe, config_dir);

    let mut cmd = Command::new(obs_exe);
    cmd.arg("--portable")
        .arg("--minimize-to-tray")
        .arg("--disable-shutdown-check")
        .arg("--multi")
        .current_dir(config_dir);

    // macOS: disable the dock icon so OBS runs as a background process
    #[cfg(target_os = "macos")]
    {
        cmd.env("LSUIElement", "1");
    }

    let child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn OBS: {e}"))?;

    log::info!("OBS launched with PID {}", child.id());
    Ok(child)
}

/// Gracefully kill the OBS process.
pub fn kill_obs(child: &mut Child) -> Result<(), String> {
    log::info!("Stopping OBS process (PID {})", child.id());
    child
        .kill()
        .map_err(|e| format!("Failed to kill OBS: {e}"))?;
    let _ = child.wait();
    log::info!("OBS process stopped");
    Ok(())
}

/// Check if the OBS process is still running.
pub fn is_obs_running(child: &mut Child) -> bool {
    matches!(child.try_wait(), Ok(None))
}
