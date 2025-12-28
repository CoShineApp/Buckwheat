mod app_state;
mod clip_processor;
mod commands;
mod database;
mod game_detector;
mod recorder;
mod slippi;
use commands::cloud::get_device_id;
use commands::default::{read, write};
use commands::settings::{
    get_recording_directory, get_setting, get_settings_path, open_settings_folder,
};
use commands::slippi::{
    capture_window_preview, check_game_window, compress_video_for_upload, delete_recording,
    delete_temp_file, get_clips, get_default_slippi_path, get_game_process_name,
    get_last_replay_path, get_recordings, list_game_windows, mark_clip_timestamp,
    open_file_location, open_recording_folder, open_video, parse_slp_events, process_clip_markers,
    set_game_process_name, start_generic_recording, start_recording, start_watching,
    stop_recording, stop_watching,
};
use commands::stats::{
    calculate_game_stats, get_aggregate_stats, get_player_stats, get_recording_stats,
    sync_stats_to_cloud,
};
use tauri::Manager;

#[allow(clippy::missing_panics_doc)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Initialize app state
            let state = app_state::AppState::new();
            
            // Initialize stats database
            let app_dir = app.path().app_data_dir()
                .expect("Failed to get app data directory");
            let db_path = app_dir.join("stats.db");
            
            match database::StatsDatabase::new(db_path) {
                Ok(db) => {
                    *state.stats_db.lock().unwrap() = Some(db);
                    log::info!("✅ Stats database initialized");
                }
                Err(e) => {
                    log::error!("❌ Failed to initialize stats database: {:?}", e);
                    // Continue without stats database - non-critical feature
                }
            }
            
            app.manage(state);

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        // Filter out verbose peppi library logs
                        .filter(|metadata| !metadata.target().starts_with("peppi::"))
                        .build(),
                )?;
            }
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
            get_recordings,
            delete_recording,
            open_video,
            open_recording_folder,
            check_game_window,
            capture_window_preview,
            list_game_windows,
            get_game_process_name,
            set_game_process_name,
            get_settings_path,
            open_settings_folder,
            get_setting,
            get_recording_directory,
            open_file_location,
            get_last_replay_path,
            parse_slp_events,
            // Clip commands
            mark_clip_timestamp,
            process_clip_markers,
            get_clips,
            // Cloud commands
            compress_video_for_upload,
            delete_temp_file,
            get_device_id,
            // Stats commands
            calculate_game_stats,
            get_recording_stats,
            get_player_stats,
            get_aggregate_stats,
            sync_stats_to_cloud,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
