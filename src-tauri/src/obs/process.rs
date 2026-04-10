use std::path::Path;
use std::process::{Child, Command};

/// Spawn OBS minimized to tray, using a Peppi-specific profile and scene collection.
///
/// OBS 32 does not support `--config-dir` or relocating the portable config.
/// Instead we write our profile/scenes into the standard OBS config directory
/// (`%APPDATA%/obs-studio`) under a "Peppi" profile and scene collection,
/// then launch OBS with `--profile` and `--collection` to select them.
/// This keeps Peppi's config isolated from the user's own OBS setup.
pub fn spawn_obs(obs_exe: &Path, _config_dir: &Path) -> Result<Child, String> {
    log::info!("Launching OBS: {:?}", obs_exe);

    let mut cmd = Command::new(obs_exe);

    // OBS resolves locale and data files relative to cwd, not the exe path.
    // https://github.com/obsproject/obs-studio/issues/2966
    let obs_bin_dir = obs_exe.parent().unwrap_or(Path::new("."));
    cmd.current_dir(obs_bin_dir);

    cmd.arg("--minimize-to-tray")
        .arg("--disable-shutdown-check")
        .arg("--multi")
        .arg("--profile").arg("Peppi")
        .arg("--collection").arg("Peppi");

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
