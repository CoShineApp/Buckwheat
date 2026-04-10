mod app_state;
mod clip_processor;
mod commands;
mod database;
mod events;
mod game_detector;
mod library;
mod obs;
mod recorder;
mod slippi;
mod window_detector;

// Clips commands
use commands::clips::{
    apply_video_edit, compress_video_for_upload, create_clip_from_range, delete_temp_file,
    mark_clip_timestamp, process_clip_markers,
};
// Cloud commands
use commands::cloud::get_device_id;
// Default commands
use commands::default::{read, write};
// Library commands
use commands::library::{
    delete_recording, get_clips, get_game_notes, get_player_stats, get_recordings, get_storage_usage,
    set_game_notes, get_total_player_stats, get_available_filter_options, get_player_stats_timeseries,
    open_file_location, open_recording_folder, open_video, refresh_recordings_cache,
    save_computed_stats, list_slp_files, check_slp_synced,
};
// Recording commands
use commands::recording::{start_generic_recording, start_recording, stop_recording, ensure_obs_ready, install_obs};
// Settings commands
use commands::settings::{
    get_recording_directory, get_setting, get_settings_path, open_settings_folder,
};
// Slippi commands
use commands::slippi::{
    get_default_slippi_path, get_last_replay_path, start_watching, stop_watching,
};
// Window commands
use commands::window::{
    capture_window_preview, check_game_window, get_game_process_name, highlight_game_window,
    list_game_windows, set_game_process_name,
};

use tauri::Manager;

#[allow(clippy::missing_panics_doc)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::default().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // Initialize logging first (so we can see database init logs)
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Initialize SQLite database
            let db_path = database::get_database_path(app.handle());
            log::info!("Initializing database at: {:?}", db_path);

            let db = database::Database::open(&db_path)?;
            db.init()?;

            log::info!("Database initialized");

            // Initialize app state with database
            app.manage(app_state::AppState::with_database(db));

            // Trigger background sync of recordings cache
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // Small delay to let the app finish initializing
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                if let Err(e) = library::sync_recordings_cache(&app_handle).await {
                    log::error!("Failed to sync recordings cache: {:?}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            read,
            write,
            get_default_slippi_path,
            start_watching,
            stop_watching,
            start_recording,
            start_generic_recording,
            stop_recording,
            ensure_obs_ready,
            install_obs,
            get_recordings,
            delete_recording,
            open_video,
            open_recording_folder,
            check_game_window,
            capture_window_preview,
            list_game_windows,
            get_game_process_name,
            set_game_process_name,
            highlight_game_window,
            get_settings_path,
            open_settings_folder,
            get_setting,
            get_recording_directory,
            open_file_location,
            get_last_replay_path,
            refresh_recordings_cache,
            get_storage_usage,
            // Clip commands
            mark_clip_timestamp,
            process_clip_markers,
            get_clips,
            apply_video_edit,
            create_clip_from_range,
            // Cloud commands
            compress_video_for_upload,
            delete_temp_file,
            get_device_id,
            // Stats commands
            save_computed_stats,
            get_player_stats,
            get_game_notes,
            set_game_notes,
            get_total_player_stats,
            get_available_filter_options,
            get_player_stats_timeseries,
            // Historical sync commands
            list_slp_files,
            check_slp_synced,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                // Drop the recorder to kill managed OBS process
                if let Some(state) = app.try_state::<app_state::AppState>() {
                    if let Ok(mut recorder_lock) = state.recorder.lock() {
                        let _ = recorder_lock.take();
                    }
                }
            }
        });
}
