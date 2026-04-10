use std::path::PathBuf;

#[cfg(target_os = "windows")]
const OBS_INSTALLER_URL: &str = "https://github.com/obsproject/obs-studio/releases/download/32.1.0/OBS-Studio-32.1.0-Windows-x64-Installer.exe";

#[cfg(target_os = "macos")]
const OBS_DMG_URL: &str = "https://github.com/obsproject/obs-studio/releases/download/32.1.0/obs-studio-32.1.0-macos-arm64.dmg";

/// Check the Windows registry and common paths for an OBS installation.
#[cfg(target_os = "windows")]
pub fn find_obs_installation() -> Option<PathBuf> {
    // Check registry
    if let Ok(hklm) = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
        .open_subkey("SOFTWARE\\OBS Studio")
    {
        if let Ok(path) = hklm.get_value::<String, _>("") {
            let obs_exe = PathBuf::from(&path).join("bin").join("64bit").join("obs64.exe");
            if obs_exe.exists() {
                return Some(obs_exe);
            }
        }
    }

    // Check common install paths
    let common_paths = [
        r"C:\Program Files\obs-studio\bin\64bit\obs64.exe",
        r"C:\Program Files (x86)\obs-studio\bin\64bit\obs64.exe",
    ];
    for path in &common_paths {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    None
}

/// Check common macOS paths for an OBS installation.
#[cfg(target_os = "macos")]
pub fn find_obs_installation() -> Option<PathBuf> {
    // Standard .app bundle location
    let app_binary = PathBuf::from("/Applications/OBS.app/Contents/MacOS/OBS");
    if app_binary.exists() {
        return Some(app_binary);
    }

    // Homebrew cask installs to /Applications too, but check the Cellar just in case
    if let Ok(output) = std::process::Command::new("brew")
        .args(["--prefix", "obs-studio"])
        .output()
    {
        if output.status.success() {
            let prefix = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let brew_binary = PathBuf::from(&prefix).join("bin").join("obs");
            if brew_binary.exists() {
                return Some(brew_binary);
            }
        }
    }

    None
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn find_obs_installation() -> Option<PathBuf> {
    None
}

/// Download the OBS installer/DMG to the given directory.
pub async fn download_obs(target_dir: &std::path::Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(target_dir).map_err(|e| format!("Failed to create dir: {e}"))?;

    #[cfg(target_os = "windows")]
    let (url, filename) = (OBS_INSTALLER_URL, "OBS-Studio-Installer.exe");

    #[cfg(target_os = "macos")]
    let (url, filename) = (OBS_DMG_URL, "OBS-Studio.dmg");

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let (url, filename) = ("", "obs");

    let dest = target_dir.join(filename);
    log::info!("Downloading OBS to {:?}", dest);

    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Download failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("Download returned status {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?;

    std::fs::write(&dest, &bytes).map_err(|e| format!("Failed to write installer: {e}"))?;

    log::info!("OBS downloaded ({} bytes)", bytes.len());
    Ok(dest)
}

/// Run the OBS NSIS installer silently (Windows).
#[cfg(target_os = "windows")]
pub fn install_obs(installer_path: &std::path::Path) -> Result<PathBuf, String> {
    log::info!("Running OBS silent install from {:?}", installer_path);

    let status = std::process::Command::new(installer_path)
        .arg("/S") // NSIS silent install (case-sensitive)
        .status()
        .map_err(|e| format!("Failed to run installer: {e}"))?;

    if !status.success() {
        return Err(format!("Installer exited with status {status}"));
    }

    find_obs_installation().ok_or_else(|| "OBS not found after installation".to_string())
}

/// Mount the DMG and copy OBS.app to /Applications (macOS).
#[cfg(target_os = "macos")]
pub fn install_obs(dmg_path: &std::path::Path) -> Result<PathBuf, String> {
    log::info!("Installing OBS from DMG: {:?}", dmg_path);

    // Mount the DMG
    let mount_output = std::process::Command::new("hdiutil")
        .args(["attach", "-nobrowse", "-quiet"])
        .arg(dmg_path)
        .output()
        .map_err(|e| format!("Failed to mount DMG: {e}"))?;

    if !mount_output.status.success() {
        return Err(format!(
            "Failed to mount DMG: {}",
            String::from_utf8_lossy(&mount_output.stderr)
        ));
    }

    // Find the mount point — OBS DMGs mount to /Volumes/OBS*
    let mount_point = find_obs_mount_point()?;

    // Copy OBS.app to /Applications
    let src = mount_point.join("OBS.app");
    if !src.exists() {
        let _ = unmount_dmg(&mount_point);
        return Err(format!("OBS.app not found in mounted DMG at {:?}", src));
    }

    let dest = PathBuf::from("/Applications/OBS.app");
    // Remove existing installation if present
    if dest.exists() {
        let _ = std::fs::remove_dir_all(&dest);
    }

    let cp_status = std::process::Command::new("cp")
        .args(["-R"])
        .arg(&src)
        .arg("/Applications/")
        .status()
        .map_err(|e| format!("Failed to copy OBS.app: {e}"))?;

    // Unmount regardless of copy result
    let _ = unmount_dmg(&mount_point);

    if !cp_status.success() {
        return Err("Failed to copy OBS.app to /Applications".to_string());
    }

    find_obs_installation().ok_or_else(|| "OBS not found after installation".to_string())
}

#[cfg(target_os = "macos")]
fn find_obs_mount_point() -> Result<PathBuf, String> {
    let entries = std::fs::read_dir("/Volumes")
        .map_err(|e| format!("Failed to read /Volumes: {e}"))?;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("OBS") {
            return Ok(entry.path());
        }
    }

    Err("Could not find OBS mount point in /Volumes".to_string())
}

#[cfg(target_os = "macos")]
fn unmount_dmg(mount_point: &std::path::Path) -> Result<(), String> {
    std::process::Command::new("hdiutil")
        .args(["detach", "-quiet"])
        .arg(mount_point)
        .status()
        .map_err(|e| format!("Failed to unmount DMG: {e}"))?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn install_obs(_path: &std::path::Path) -> Result<PathBuf, String> {
    Err("OBS installation is not supported on this platform".to_string())
}

/// Find OBS, or download and install it if not present.
pub async fn ensure_obs_available() -> Result<PathBuf, String> {
    if let Some(path) = find_obs_installation() {
        log::info!("Found existing OBS installation: {:?}", path);
        return Ok(path);
    }

    log::info!("OBS not found, downloading...");
    let temp_dir = std::env::temp_dir().join("peppi-obs-install");
    let download_path = download_obs(&temp_dir).await?;
    let obs_path = install_obs(&download_path)?;

    // Clean up installer/DMG
    let _ = std::fs::remove_file(&download_path);
    let _ = std::fs::remove_dir(&temp_dir);

    Ok(obs_path)
}
