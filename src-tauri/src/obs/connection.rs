use std::time::Duration;

/// Wrapper around an OBS websocket connection.
pub struct ObsConnection {
    client: obws::Client,
}

impl ObsConnection {
    /// Connect to OBS websocket.
    pub async fn connect(port: u16, password: &str) -> Result<Self, String> {
        let client = obws::Client::connect("127.0.0.1", port, Some(password))
            .await
            .map_err(|e| format!("Failed to connect to OBS: {e}"))?;
        Ok(Self { client })
    }

    /// Set the directory OBS will write recordings to.
    pub async fn set_record_directory(&self, directory: &str) -> Result<(), String> {
        self.client
            .config()
            .set_record_directory(directory)
            .await
            .map_err(|e| format!("Failed to set OBS record directory: {e}"))
    }

    /// Start recording.
    pub async fn start_recording(&self) -> Result<(), String> {
        self.client
            .recording()
            .start()
            .await
            .map_err(|e| format!("Failed to start OBS recording: {e}"))
    }

    /// Stop recording and return the output file path.
    pub async fn stop_recording(&self) -> Result<String, String> {
        self.client
            .recording()
            .stop()
            .await
            .map_err(|e| format!("Failed to stop OBS recording: {e}"))
    }

    /// Check if OBS is currently recording.
    pub async fn is_recording(&self) -> Result<bool, String> {
        let status = self
            .client
            .recording()
            .status()
            .await
            .map_err(|e| format!("Failed to get OBS recording status: {e}"))?;
        Ok(status.active)
    }
}

/// Poll until OBS websocket is ready, with timeout.
pub async fn wait_for_obs_ready(
    port: u16,
    password: &str,
    timeout: Duration,
) -> Result<ObsConnection, String> {
    let start = std::time::Instant::now();
    let poll_interval = Duration::from_millis(500);

    loop {
        match ObsConnection::connect(port, password).await {
            Ok(conn) => {
                log::info!("Connected to OBS websocket on port {}", port);
                return Ok(conn);
            }
            Err(e) => {
                if start.elapsed() > timeout {
                    return Err(format!(
                        "Timed out waiting for OBS websocket after {:?}: {e}",
                        timeout
                    ));
                }
                log::debug!("OBS not ready yet, retrying in {:?}...", poll_interval);
                tokio::time::sleep(poll_interval).await;
            }
        }
    }
}
