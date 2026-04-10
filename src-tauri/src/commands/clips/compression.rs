//! Compression and cleanup

use crate::commands::errors::Error;
use std::path::Path;

/// Compress video for cloud upload
#[tauri::command]
pub async fn compress_video_for_upload(input_path: String) -> Result<String, Error> {
    log::info!("Compressing video for upload: {}", input_path);

    crate::clip_processor::ensure_ffmpeg()?;

    // Generate output path in temp directory
    let input_file = Path::new(&input_path);
    let file_stem = input_file
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| Error::InvalidPath("Invalid input path".into()))?;

    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join(format!("{}_compressed.mp4", file_stem));
    let output_path_str = output_path
        .to_str()
        .ok_or_else(|| Error::InvalidPath("Invalid output path".into()))?
        .to_string();

    // Compress video
    use ffmpeg_sidecar::command::FfmpegCommand;

    let mut command = FfmpegCommand::new();
    command
        .input(&input_path)
        .args([
            "-c:v", "libx264",
            "-preset", "fast",
            "-crf", "28",
            "-vf", "scale=-2:720",
            "-c:a", "aac",
            "-b:a", "128k",
        ])
        .output(&output_path_str)
        .overwrite();

    let output = command
        .spawn()
        .map_err(|e| Error::RecordingFailed(format!("Failed to start FFmpeg: {}", e)))?
        .wait()
        .map_err(|e| Error::RecordingFailed(format!("FFmpeg failed: {}", e)))?;

    if !output.success() {
        return Err(Error::RecordingFailed(format!("FFmpeg exited with error: {:?}", output)));
    }

    log::info!("✅ Video compressed successfully");
    Ok(output_path_str)
}

/// Delete a temporary file
#[tauri::command]
pub async fn delete_temp_file(path: String) -> Result<(), Error> {
    std::fs::remove_file(&path)
        .map_err(|e| Error::RecordingFailed(format!("Failed to delete temp file: {}", e)))?;
    log::debug!("🗑️ Deleted temp file: {}", path);
    Ok(())
}
