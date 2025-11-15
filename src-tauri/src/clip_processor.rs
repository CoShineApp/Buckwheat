use crate::commands::errors::Error;
use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::download::auto_download;
use std::path::Path;

/// Ensures FFmpeg is available, downloading if necessary
pub fn ensure_ffmpeg() -> Result<(), Error> {
    auto_download()
        .map_err(|e| Error::RecordingFailed(format!("Failed to download FFmpeg: {}", e)))?;
    Ok(())
}

/// Extract a clip from a video file
pub fn extract_clip(
    input_path: &str,
    output_path: &str,
    start_time: f64,
    duration: f64,
) -> Result<(), Error> {
    log::info!(
        "🎬 Extracting clip: input={}, output={}, start={}s, duration={}s",
        input_path,
        output_path,
        start_time,
        duration
    );

    // Ensure input file exists
    if !Path::new(input_path).exists() {
        return Err(Error::InvalidPath(format!(
            "Input file does not exist: {}",
            input_path
        )));
    }

    // Ensure output directory exists
    if let Some(parent) = Path::new(output_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            Error::RecordingFailed(format!("Failed to create output directory: {}", e))
        })?;
    }

    // Build FFmpeg command
    let result = FfmpegCommand::new()
        .arg("-ss")
        .arg(start_time.to_string())
        .arg("-i")
        .arg(input_path)
        .arg("-t")
        .arg(duration.to_string())
        .arg("-c")
        .arg("copy")
        .arg("-avoid_negative_ts")
        .arg("1")
        .arg("-y") // Overwrite output file
        .arg(output_path)
        .spawn();

    match result {
        Ok(mut child) => {
            let status = child
                .wait()
                .map_err(|e| Error::RecordingFailed(format!("FFmpeg process error: {}", e)))?;

            if status.success() {
                log::info!("✅ Clip extracted successfully: {}", output_path);
                Ok(())
            } else {
                Err(Error::RecordingFailed(format!(
                    "FFmpeg failed with status: {:?}",
                    status
                )))
            }
        }
        Err(e) => Err(Error::RecordingFailed(format!(
            "Failed to spawn FFmpeg: {}",
            e
        ))),
    }
}

/// Generate a thumbnail image from a video file
/// Extracts a frame at the specified time (default: 1 second) and saves as JPEG
pub fn generate_thumbnail(
    video_path: &str,
    thumbnail_path: &str,
    time_offset: Option<f64>,
) -> Result<(), Error> {
    let offset = time_offset.unwrap_or(1.0); // Default to 1 second into video
    
    log::debug!(
        "🖼️  Generating thumbnail: video={}, output={}, offset={}s",
        video_path,
        thumbnail_path,
        offset
    );

    // Ensure input file exists
    if !Path::new(video_path).exists() {
        return Err(Error::InvalidPath(format!(
            "Video file does not exist: {}",
            video_path
        )));
    }

    // Ensure output directory exists
    if let Some(parent) = Path::new(thumbnail_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            Error::RecordingFailed(format!("Failed to create thumbnail directory: {}", e))
        })?;
    }

    // Build FFmpeg command to extract frame as JPEG
    // -ss: seek to time offset
    // -i: input file
    // -vframes 1: extract only 1 frame
    // -vf scale=320:-1: scale to 320px width, maintain aspect ratio
    // -q:v 2: high quality JPEG (lower = better quality, 2-5 is good)
    let result = FfmpegCommand::new()
        .arg("-ss")
        .arg(offset.to_string())
        .arg("-i")
        .arg(video_path)
        .arg("-vframes")
        .arg("1")
        .arg("-vf")
        .arg("scale=320:-1")
        .arg("-q:v")
        .arg("2")
        .arg("-y") // Overwrite output file
        .arg(thumbnail_path)
        .spawn();

    match result {
        Ok(mut child) => {
            let status = child
                .wait()
                .map_err(|e| Error::RecordingFailed(format!("FFmpeg process error: {}", e)))?;

            if status.success() {
                log::debug!("✅ Thumbnail generated successfully: {}", thumbnail_path);
                Ok(())
            } else {
                Err(Error::RecordingFailed(format!(
                    "FFmpeg failed with status: {:?}",
                    status
                )))
            }
        }
        Err(e) => Err(Error::RecordingFailed(format!(
            "Failed to spawn FFmpeg: {}",
            e
        ))),
    }
}
