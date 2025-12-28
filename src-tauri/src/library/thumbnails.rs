//! Thumbnail generation for recordings

use std::path::Path;

/// Generate a thumbnail for a video if one doesn't already exist
/// Returns the thumbnail path if successful
pub fn generate_thumbnail_if_missing(video_path: &Path, id: &str) -> Option<String> {
    let Some(parent) = video_path.parent() else {
        return None;
    };
    
    let thumbnails_dir = parent.join("Thumbnails");
    if let Err(e) = std::fs::create_dir_all(&thumbnails_dir) {
        log::warn!("Failed to create thumbnails directory: {}", e);
    }
    
    let thumbnail_filename = format!("{}.jpg", id);
    let thumbnail_path = thumbnails_dir.join(&thumbnail_filename);
    
    // Generate thumbnail if it doesn't exist
    if !thumbnail_path.exists() {
        // Ensure FFmpeg is available
        if crate::clip_processor::ensure_ffmpeg().is_err() {
            return None;
        }
        
        let video_path_str = video_path.to_string_lossy();
        let thumbnail_path_str = thumbnail_path.to_string_lossy();
        
        if let Err(e) = crate::clip_processor::generate_thumbnail(
            &video_path_str,
            &thumbnail_path_str,
            None,
        ) {
            log::warn!("Failed to generate thumbnail: {}", e);
            return None;
        }
    }
    
    thumbnail_path.to_str().map(|s| s.to_string())
}

