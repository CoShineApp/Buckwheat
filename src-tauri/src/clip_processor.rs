use crate::commands::errors::Error;
use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::download::auto_download;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Represents a crop region with position and dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CropRegion {
    pub x: u32,      // Left offset in pixels
    pub y: u32,      // Top offset in pixels
    pub width: u32,  // Crop width in pixels
    pub height: u32, // Crop height in pixels
}

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

/// Crop a video to a specified region
/// Uses FFmpeg's crop filter: crop=width:height:x:y
pub fn crop_video(
    input_path: &str,
    output_path: &str,
    crop: &CropRegion,
) -> Result<(), Error> {
    log::info!(
        "✂️ Cropping video: input={}, output={}, crop={}x{}+{}+{}",
        input_path,
        output_path,
        crop.width,
        crop.height,
        crop.x,
        crop.y
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

    // Build crop filter string: crop=width:height:x:y
    let crop_filter = format!("crop={}:{}:{}:{}", crop.width, crop.height, crop.x, crop.y);

    // Build FFmpeg command with crop filter
    let result = FfmpegCommand::new()
        .arg("-i")
        .arg(input_path)
        .arg("-vf")
        .arg(&crop_filter)
        .arg("-c:a")
        .arg("copy") // Copy audio without re-encoding
        .arg("-y") // Overwrite output file
        .arg(output_path)
        .spawn();

    match result {
        Ok(mut child) => {
            let status = child
                .wait()
                .map_err(|e| Error::RecordingFailed(format!("FFmpeg process error: {}", e)))?;

            if status.success() {
                log::info!("✅ Video cropped successfully: {}", output_path);
                Ok(())
            } else {
                Err(Error::RecordingFailed(format!(
                    "FFmpeg crop failed with status: {:?}",
                    status
                )))
            }
        }
        Err(e) => Err(Error::RecordingFailed(format!(
            "Failed to spawn FFmpeg for crop: {}",
            e
        ))),
    }
}

/// Process video with combined trim and/or crop operations in a single FFmpeg pass
/// This is more efficient than running separate trim and crop operations
pub fn process_video_edit(
    input_path: &str,
    output_path: &str,
    trim_start: Option<f64>,
    trim_end: Option<f64>,
    crop: Option<CropRegion>,
) -> Result<(), Error> {
    log::info!(
        "🎬 Processing video edit: input={}, output={}, trim_start={:?}, trim_end={:?}, crop={:?}",
        input_path,
        output_path,
        trim_start,
        trim_end,
        crop
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

    let mut cmd = FfmpegCommand::new();

    // Add trim start if specified (seeking before input is faster)
    if let Some(start) = trim_start {
        cmd.arg("-ss").arg(start.to_string());
    }

    // Input file
    cmd.arg("-i").arg(input_path);

    // Add trim end if specified
    if let Some(end) = trim_end {
        let duration = if let Some(start) = trim_start {
            end - start
        } else {
            end
        };
        cmd.arg("-t").arg(duration.to_string());
    }

    // Add crop filter if specified
    if let Some(ref crop_region) = crop {
        let crop_filter = format!(
            "crop={}:{}:{}:{}",
            crop_region.width, crop_region.height, crop_region.x, crop_region.y
        );
        cmd.arg("-vf").arg(&crop_filter);
        // When using video filter, we need to re-encode video
        cmd.arg("-c:a").arg("copy"); // But copy audio
    } else {
        // No crop, can use stream copy for both video and audio (fastest)
        cmd.arg("-c").arg("copy");
    }

    // Avoid negative timestamps issue
    cmd.arg("-avoid_negative_ts").arg("1");
    
    // Overwrite output file
    cmd.arg("-y").arg(output_path);

    let result = cmd.spawn();

    match result {
        Ok(mut child) => {
            let status = child
                .wait()
                .map_err(|e| Error::RecordingFailed(format!("FFmpeg process error: {}", e)))?;

            if status.success() {
                log::info!("✅ Video edit processed successfully: {}", output_path);
                Ok(())
            } else {
                Err(Error::RecordingFailed(format!(
                    "FFmpeg edit failed with status: {:?}",
                    status
                )))
            }
        }
        Err(e) => Err(Error::RecordingFailed(format!(
            "Failed to spawn FFmpeg for edit: {}",
            e
        ))),
    }
}
