//! Clip creation commands
//!
//! Commands for marking clip timestamps and processing clips.

use crate::app_state::AppState;
use crate::commands::errors::Error;
use crate::events::clips as clip_events;
use crate::library;
use std::path::Path;
use tauri::{Emitter, State};
use tauri_plugin_store::StoreExt;

/// Mark a timestamp for clip creation
#[tauri::command]
pub fn mark_clip_timestamp(
    recording_file: String,
    timestamp: f64,
    state: State<'_, AppState>,
) -> Result<(), Error> {
    let mut markers = state
        .clip_markers
        .lock()
        .map_err(|e| Error::InitializationError(format!("Failed to lock clip markers: {}", e)))?;
    
    markers.push(crate::app_state::ClipMarker {
        recording_file,
        timestamp_seconds: timestamp,
    });
    
    log::info!("📍 Clip marker added at {}s", timestamp);
    Ok(())
}

/// Process all clip markers for a recording file
#[tauri::command]
pub async fn process_clip_markers(
    recording_file: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<String>, Error> {
    // Ensure FFmpeg is available
    log::info!("Ensuring FFmpeg is available...");
    match crate::clip_processor::ensure_ffmpeg() {
        Ok(_) => log::info!("✅ FFmpeg is ready"),
        Err(e) => {
            log::error!("❌ FFmpeg not available: {:?}", e);
            return Err(e);
        }
    }
    
    // Get clip duration from settings
    let clip_duration = {
        let store = app.store("settings.json").map_err(|e| {
            Error::InitializationError(format!("Failed to open settings store: {}", e))
        })?;
        
        store
            .get("clipDuration")
            .and_then(|v| v.as_f64())
            .unwrap_or(30.0)
    };
    
    log::info!("⏱ Clip duration: {}s", clip_duration);
    
    // Get markers for this recording (match by base filename)
    let recording_base = Path::new(&recording_file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&recording_file);
    
    log::debug!("Looking for clip markers matching base: {}", recording_base);
    
    let markers = {
        let mut markers_lock = state.clip_markers.lock().map_err(|e| {
            Error::InitializationError(format!("Failed to lock clip markers: {}", e))
        })?;
        
        // Match by base filename
        let recording_markers: Vec<_> = markers_lock
            .iter()
            .filter(|m| {
                let marker_base = Path::new(&m.recording_file)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&m.recording_file);
                marker_base == recording_base
            })
            .cloned()
            .collect();
        
        // Remove processed markers
        markers_lock.retain(|m| {
            let marker_base = Path::new(&m.recording_file)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&m.recording_file);
            marker_base != recording_base
        });
        
        recording_markers
    };
    
    if markers.is_empty() {
        log::info!("ℹ No clip markers found for this recording");
        return Ok(Vec::new());
    }
    
    log::info!("Found {} clip marker(s) to process", markers.len());
    
    // Get recording directory
    let recording_dir = library::get_recording_directory(&app).await?;
    
    // Determine video path
    let video_path = if recording_file.ends_with(".mp4") {
        recording_file.clone()
    } else {
        format!("{}.mp4", recording_file.trim_end_matches(".slp"))
    };
    
    let input_path = if Path::new(&video_path).is_absolute() {
        video_path.clone()
    } else {
        format!("{}/{}", recording_dir, video_path)
    };
    
    // Verify input file exists
    if !Path::new(&input_path).exists() {
        log::error!("Recording file not found: {}", input_path);
        return Err(Error::InvalidPath(format!("Recording file not found: {}", input_path)));
    }
    
    // Create clips directory
    let recording_dir_path = Path::new(&recording_dir);
    let clips_parent_dir = recording_dir_path.parent().unwrap_or(recording_dir_path);
    let clips_dir_path = clips_parent_dir.join("Clips");
    
    std::fs::create_dir_all(&clips_dir_path).map_err(|e| {
        log::error!("Failed to create clips directory: {}", e);
        Error::RecordingFailed(format!("Failed to create clips directory: {}", e))
    })?;
    
    let mut created_clips = Vec::new();
    
    // Process each marker
    for (idx, marker) in markers.iter().enumerate() {
        let start_time = (marker.timestamp_seconds - clip_duration).max(0.0);
        
        // Generate clip filename
        let timestamp = Path::new(&recording_file)
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.strip_prefix("Game_").unwrap_or(s))
            .unwrap_or("unknown");
        
        let clip_filename = format!("Clip_{}_{:03}.mp4", timestamp, idx + 1);
        let output_path = clips_dir_path.join(&clip_filename);
        let output_path_str = output_path
            .to_str()
            .ok_or_else(|| Error::InvalidPath("Failed to build clip output path".to_string()))?
            .to_string();
        
        // Extract clip
        match crate::clip_processor::extract_clip(&input_path, &output_path_str, start_time, clip_duration) {
            Ok(_) => {
                log::info!(
                    "✅ Clip created ({}/{}): {} (start {}s, duration {}s)",
                    idx + 1,
                    markers.len(),
                    clip_filename,
                    start_time,
                    clip_duration
                );
                created_clips.push(output_path_str);
            }
            Err(e) => {
                log::error!("Failed to create clip: {:?}", e);
                return Err(e);
            }
        }
    }
    
    log::info!("✅ Created {} clip(s)", created_clips.len());
    
    // Emit event to frontend
    if !created_clips.is_empty() {
        if let Err(e) = app.emit(clip_events::CREATED, created_clips.clone()) {
            log::error!("Failed to emit {} event: {:?}", clip_events::CREATED, e);
        }
    }
    
    Ok(created_clips)
}

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

/// Apply video edits (trim and/or crop) to a video file
/// Optionally replaces the original file with the edited version
#[tauri::command]
pub async fn apply_video_edit(
    input_path: String,
    trim_start: Option<f64>,
    trim_end: Option<f64>,
    crop_x: Option<u32>,
    crop_y: Option<u32>,
    crop_width: Option<u32>,
    crop_height: Option<u32>,
    replace_original: bool,
) -> Result<String, Error> {
    log::info!(
        "🎬 Applying video edit: input={}, trim={:?}-{:?}, crop=({:?},{:?},{:?},{:?}), replace={}",
        input_path,
        trim_start,
        trim_end,
        crop_x,
        crop_y,
        crop_width,
        crop_height,
        replace_original
    );

    // Ensure FFmpeg is available
    crate::clip_processor::ensure_ffmpeg()?;

    // Verify input file exists
    if !Path::new(&input_path).exists() {
        return Err(Error::InvalidPath(format!(
            "Input file does not exist: {}",
            input_path
        )));
    }

    // Build crop region if all crop parameters are provided
    let crop = if let (Some(x), Some(y), Some(w), Some(h)) =
        (crop_x, crop_y, crop_width, crop_height)
    {
        Some(crate::clip_processor::CropRegion {
            x,
            y,
            width: w,
            height: h,
        })
    } else {
        None
    };

    // Check if there's actually an edit to make
    if trim_start.is_none() && trim_end.is_none() && crop.is_none() {
        log::warn!("No edits specified, returning original path");
        return Ok(input_path);
    }

    // Generate temp output path
    let input_file = Path::new(&input_path);
    let file_stem = input_file
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| Error::InvalidPath("Invalid input path".into()))?;

    let temp_dir = std::env::temp_dir();
    let temp_output_path = temp_dir.join(format!("{}_edited.mp4", file_stem));
    let temp_output_str = temp_output_path
        .to_str()
        .ok_or_else(|| Error::InvalidPath("Invalid temp output path".into()))?
        .to_string();

    // Process video edits
    crate::clip_processor::process_video_edit(
        &input_path,
        &temp_output_str,
        trim_start,
        trim_end,
        crop,
    )?;

    // Handle file replacement
    let final_path = if replace_original {
        // Delete original file
        std::fs::remove_file(&input_path).map_err(|e| {
            Error::RecordingFailed(format!("Failed to delete original file: {}", e))
        })?;

        // Move temp file to original location
        std::fs::rename(&temp_output_str, &input_path).map_err(|e| {
            Error::RecordingFailed(format!("Failed to rename temp file: {}", e))
        })?;

        log::info!("✅ Video edit applied, original replaced: {}", input_path);

        // Regenerate thumbnail for the edited video
        let thumbnail_path = input_file.with_extension("jpg");
        if let Some(thumb_str) = thumbnail_path.to_str() {
            if let Err(e) = crate::clip_processor::generate_thumbnail(&input_path, thumb_str, None)
            {
                log::warn!("Failed to regenerate thumbnail: {:?}", e);
            }
        }

        input_path
    } else {
        log::info!("✅ Video edit applied: {}", temp_output_str);
        temp_output_str
    };

    Ok(final_path)
}

/// Create a clip from a video with specified start and end times
/// This is used by the clip editor to create a new clip from a selection
#[tauri::command]
pub async fn create_clip_from_range(
    input_path: String,
    start_time: f64,
    end_time: f64,
    output_dir: Option<String>,
    app: tauri::AppHandle,
) -> Result<String, Error> {
    log::info!(
        "✂️ Creating clip from range: input={}, start={}s, end={}s",
        input_path,
        start_time,
        end_time
    );

    // Ensure FFmpeg is available
    crate::clip_processor::ensure_ffmpeg()?;

    // Verify input file exists
    if !Path::new(&input_path).exists() {
        return Err(Error::InvalidPath(format!(
            "Input file does not exist: {}",
            input_path
        )));
    }

    // Validate time range
    if start_time >= end_time {
        return Err(Error::RecordingFailed(
            "Start time must be less than end time".into(),
        ));
    }

    let duration = end_time - start_time;

    // Determine output directory
    let clips_dir = if let Some(dir) = output_dir {
        std::path::PathBuf::from(dir)
    } else {
        // Use default clips directory
        let recording_dir = library::get_recording_directory(&app).await?;
        let recording_dir_path = Path::new(&recording_dir);
        let clips_parent_dir = recording_dir_path.parent().unwrap_or(recording_dir_path);
        clips_parent_dir.join("Clips")
    };

    // Ensure clips directory exists
    std::fs::create_dir_all(&clips_dir).map_err(|e| {
        Error::RecordingFailed(format!("Failed to create clips directory: {}", e))
    })?;

    // Generate clip filename with timestamp
    let input_file = Path::new(&input_path);
    let source_stem = input_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("video");

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let clip_filename = format!("Clip_{}_{}.mp4", source_stem, timestamp);
    let output_path = clips_dir.join(&clip_filename);
    let output_str = output_path
        .to_str()
        .ok_or_else(|| Error::InvalidPath("Invalid output path".into()))?
        .to_string();

    // Extract clip using existing function
    crate::clip_processor::extract_clip(&input_path, &output_str, start_time, duration)?;

    // Generate thumbnail
    let thumbnail_path = output_path.with_extension("jpg");
    if let Some(thumb_str) = thumbnail_path.to_str() {
        if let Err(e) = crate::clip_processor::generate_thumbnail(&output_str, thumb_str, None) {
            log::warn!("Failed to generate thumbnail: {:?}", e);
        }
    }

    log::info!("✅ Clip created: {}", output_str);

    // Emit clip created event
    if let Err(e) = app.emit(clip_events::CREATED, vec![output_str.clone()]) {
        log::error!("Failed to emit {} event: {:?}", clip_events::CREATED, e);
    }

    Ok(output_str)
}
