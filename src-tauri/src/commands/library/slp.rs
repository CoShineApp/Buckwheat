//! SLP file operations

use crate::app_state::AppState;
use crate::commands::errors::Error;
use crate::database;
use tauri::State;

/// List all .slp files in a directory (recursive, up to 5 levels deep)
#[tauri::command]
pub async fn list_slp_files(directory: String) -> Result<Vec<String>, Error> {
    use walkdir::WalkDir;

    let dir_path = std::path::Path::new(&directory);
    if !dir_path.exists() {
        return Err(Error::InvalidPath(format!("Directory does not exist: {}", directory)));
    }

    let mut slp_files = Vec::new();

    for entry in WalkDir::new(&directory)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("slp") {
            slp_files.push(path.to_string_lossy().to_string());
        }
    }

    log::info!("Found {} .slp files in {}", slp_files.len(), directory);
    Ok(slp_files)
}

/// Check if a game with the given slp_path already exists in the database
#[tauri::command]
pub async fn check_slp_synced(
    slp_path: String,
    state: State<'_, AppState>,
) -> Result<bool, Error> {
    let db = state.database.clone();
    let conn = db.connection();

    database::game_stats_exists_by_slp_path(&conn, &slp_path)
        .map_err(|e| Error::RecordingFailed(format!("Failed to check slp sync status: {}", e)))
}
