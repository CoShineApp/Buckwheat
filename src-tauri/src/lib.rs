mod app_state;
mod commands;
mod game_detector;
mod recorder;
use commands::default::{read, write};
use commands::settings::{get_recording_directory, get_setting, get_settings_path, open_settings_folder};
use commands::slippi::{
    get_default_slippi_path, get_recordings, start_recording, start_watching, stop_recording,
    stop_watching,
};
use tauri::Manager;

#[allow(clippy::missing_panics_doc)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Initialize app state
            app.manage(app_state::AppState::new());
            
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
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
            stop_recording,
            get_recordings,
            get_settings_path,
            open_settings_folder,
            get_setting,
            get_recording_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
