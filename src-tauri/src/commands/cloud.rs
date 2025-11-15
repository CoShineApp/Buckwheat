use tauri::AppHandle;
use uuid::Uuid;

/// Get or create device ID for anonymous clip identification
#[tauri::command]
pub async fn get_device_id(app: AppHandle) -> Result<String, String> {
    use tauri_plugin_store::StoreExt;

    let store = app
        .store("settings.json")
        .map_err(|e| format!("Failed to open store: {}", e))?;

    // Check if device_id already exists
    if let Some(value) = store.get("device_id") {
        if let Some(device_id) = value.as_str() {
            return Ok(device_id.to_string());
        }
    }

    // Generate new device_id
    let device_id = Uuid::new_v4().to_string();
    store.set("device_id", serde_json::json!(device_id));

    store
        .save()
        .map_err(|e| format!("Failed to save store: {}", e))?;

    log::info!("📱 Generated new device ID: {}", device_id);
    Ok(device_id)
}
