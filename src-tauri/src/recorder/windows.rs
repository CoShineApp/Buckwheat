#![cfg_attr(
    all(target_os = "windows", feature = "real-recording"),
    allow(unexpected_cfgs)
)]

#[cfg(all(target_os = "windows", feature = "real-recording"))]
use super::{Error, Recorder};

#[cfg(all(target_os = "windows", feature = "real-recording"))]
use std::collections::VecDeque;
#[cfg(all(target_os = "windows", feature = "real-recording"))]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(all(target_os = "windows", feature = "real-recording"))]
use std::sync::{mpsc, Arc, Mutex};
#[cfg(all(target_os = "windows", feature = "real-recording"))]
use std::thread::{self, JoinHandle};

#[cfg(all(target_os = "windows", feature = "real-recording"))]
use log::{info, warn};
#[cfg(all(target_os = "windows", feature = "real-recording"))]
use wasapi::{
    get_default_device, initialize_mta, AudioClient, Direction, SampleType, StreamMode, WaveFormat,
};
#[cfg(all(target_os = "windows", feature = "real-recording"))]
use windows::Win32::Foundation::HWND;
#[cfg(all(target_os = "windows", feature = "real-recording"))]
use windows::Win32::UI::WindowsAndMessaging::GetClassNameW;
#[cfg(all(target_os = "windows", feature = "real-recording"))]
use windows_capture::capture::Context;
#[cfg(all(target_os = "windows", feature = "real-recording"))]
use windows_capture::capture::{CaptureControl, GraphicsCaptureApiHandler};
#[cfg(all(target_os = "windows", feature = "real-recording"))]
use windows_capture::encoder::{
    AudioSettingsBuilder, ContainerSettingsBuilder, VideoEncoder, VideoSettingsBuilder,
    VideoSettingsSubType,
};
#[cfg(all(target_os = "windows", feature = "real-recording"))]
use windows_capture::frame::Frame;
#[cfg(all(target_os = "windows", feature = "real-recording"))]
use windows_capture::graphics_capture_api::InternalCaptureControl;
#[cfg(all(target_os = "windows", feature = "real-recording"))]
use windows_capture::settings::{
    ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
    MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
};
#[cfg(all(target_os = "windows", feature = "real-recording"))]
use windows_capture::window::Window as CaptureWindow;

#[cfg(all(target_os = "windows", feature = "real-recording"))]
const TARGET_FPS: u32 = 60;
#[cfg(all(target_os = "windows", feature = "real-recording"))]
const AUDIO_SAMPLE_RATE: u32 = 48_000;
#[cfg(all(target_os = "windows", feature = "real-recording"))]
const AUDIO_CHANNELS: u32 = 2;
#[cfg(all(target_os = "windows", feature = "real-recording"))]
const AUDIO_CHUNK_FRAMES: usize = 2048;

#[cfg(all(target_os = "windows", feature = "real-recording"))]
pub struct WindowsRecorder {
    is_recording: bool,
    capture_control: Option<CaptureControl<WindowCaptureHandler, HandlerError>>,
    audio_thread: Option<JoinHandle<()>>,
    shared_state: Option<Arc<SharedRecorderState>>,
    output_path: Option<String>,
    target_process_id: Option<u32>,
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
impl WindowsRecorder {
    pub fn new() -> Self {
        Self {
            is_recording: false,
            capture_control: None,
            audio_thread: None,
            shared_state: None,
            output_path: None,
            target_process_id: None,
        }
    }

    fn ensure_output_dir(&self, output_path: &str) -> Result<(), Error> {
        if let Some(parent) = std::path::Path::new(output_path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|err| {
                    Error::RecordingFailed(format!("Failed to create output directory: {err}"))
                })?;
            }
        }
        Ok(())
    }

    fn resolve_target_window(&self) -> Result<TargetWindow, Error> {
        let selection = TargetSelection::from_env();
        find_best_window(&selection)
    }

    fn build_encoder(
        &self,
        width: u32,
        height: u32,
        output_path: &str,
        quality: super::RecordingQuality,
    ) -> Result<VideoEncoder, Error> {
        let bitrate = quality.bitrate();
        log::info!(
            "🎬 Building encoder with {:?} quality (bitrate: {} Mbps)",
            quality,
            bitrate / 1_000_000
        );

        let video_settings = VideoSettingsBuilder::new(width, height)
            .sub_type(VideoSettingsSubType::H264)
            .frame_rate(TARGET_FPS)
            .bitrate(bitrate);

        let audio_settings = AudioSettingsBuilder::new().disabled(false);
        let container_settings = ContainerSettingsBuilder::new();

        VideoEncoder::new(
            video_settings,
            audio_settings,
            container_settings,
            output_path,
        )
        .map_err(|err| Error::InitializationError(format!("Failed to create encoder: {err:?}")))
    }
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
impl Recorder for WindowsRecorder {
    fn start_recording(
        &mut self,
        output_path: &str,
        quality: super::RecordingQuality,
    ) -> Result<(), Error> {
        if self.is_recording {
            return Err(Error::RecordingFailed("Already recording".into()));
        }

        self.ensure_output_dir(output_path)?;
        let target = self.resolve_target_window()?;
        info!(
            "Starting Windows capture for '{}' (pid {}, {}x{}) with {:?} quality",
            target.title, target.pid, target.width, target.height, quality
        );

        let encoder = self.build_encoder(target.width, target.height, output_path, quality)?;
        let shared = Arc::new(SharedRecorderState::new(encoder));
        let capture_settings = Settings::new(
            target.window,
            CursorCaptureSettings::WithoutCursor,
            DrawBorderSettings::Default,
            SecondaryWindowSettings::Default,
            MinimumUpdateIntervalSettings::Default,
            DirtyRegionSettings::Default,
            ColorFormat::Rgba8,
            shared.clone(),
        );

        let capture_control =
            WindowCaptureHandler::start_free_threaded(capture_settings).map_err(|err| {
                Error::InitializationError(format!("Failed to start graphics capture: {err}"))
            })?;

        let audio_thread = spawn_audio_thread(shared.clone(), Some(target.pid))?;

        self.capture_control = Some(capture_control);
        self.audio_thread = Some(audio_thread);
        self.shared_state = Some(shared);
        self.output_path = Some(output_path.to_string());
        self.target_process_id = Some(target.pid);
        self.is_recording = true;

        Ok(())
    }

    fn stop_recording(&mut self) -> Result<String, Error> {
        if !self.is_recording {
            return Err(Error::RecordingFailed("Not recording".into()));
        }

        if let Some(shared) = &self.shared_state {
            shared.request_stop();
        }

        if let Some(control) = self.capture_control.take() {
            control
                .stop()
                .map_err(|err| Error::RecordingFailed(format!("Failed to stop capture: {err}")))?;
        }

        if let Some(handle) = self.audio_thread.take() {
            let _ = handle.join();
        }

        let output = self
            .output_path
            .clone()
            .unwrap_or_else(|| "recording.mp4".into());

        if let Some(shared) = self.shared_state.take() {
            if let Some(encoder) = shared.take_encoder() {
                encoder.finish().map_err(|err| {
                    Error::RecordingFailed(format!("Failed to finalize recording: {err:?}"))
                })?;
            }

            if let Some(err) = shared.take_error() {
                self.is_recording = false;
                return Err(Error::RecordingFailed(err));
            }
        }

        self.output_path = None;
        self.is_recording = false;
        info!("Recording saved to {output}");
        Ok(output)
    }

    fn is_recording(&self) -> bool {
        self.is_recording
    }
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
struct SharedRecorderState {
    encoder: Mutex<Option<VideoEncoder>>,
    stop_flag: AtomicBool,
    last_error: Mutex<Option<String>>,
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
impl SharedRecorderState {
    fn new(encoder: VideoEncoder) -> Self {
        Self {
            encoder: Mutex::new(Some(encoder)),
            stop_flag: AtomicBool::new(false),
            last_error: Mutex::new(None),
        }
    }

    fn request_stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    fn should_stop(&self) -> bool {
        self.stop_flag.load(Ordering::Relaxed)
    }

    fn take_encoder(&self) -> Option<VideoEncoder> {
        self.encoder.lock().unwrap().take()
    }

    fn record_error(&self, message: impl Into<String>) {
        *self.last_error.lock().unwrap() = Some(message.into());
        self.request_stop();
    }

    fn take_error(&self) -> Option<String> {
        self.last_error.lock().unwrap().take()
    }
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
struct WindowCaptureHandler {
    shared: Arc<SharedRecorderState>,
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
#[derive(Debug, thiserror::Error)]
enum HandlerError {
    #[error("{0}")]
    Encoder(String),
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
impl GraphicsCaptureApiHandler for WindowCaptureHandler {
    type Flags = Arc<SharedRecorderState>;
    type Error = HandlerError;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self { shared: ctx.flags })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        if self.shared.should_stop() {
            capture_control.stop();
            return Ok(());
        }

        let mut guard = self.shared.encoder.lock().unwrap();
        if let Some(encoder) = guard.as_mut() {
            encoder
                .send_frame(frame)
                .map_err(|err| HandlerError::Encoder(format!("Failed to encode frame: {err:?}")))?;
        }

        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        self.shared.request_stop();
        Ok(())
    }
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
fn spawn_audio_thread(
    shared: Arc<SharedRecorderState>,
    process_id: Option<u32>,
) -> Result<JoinHandle<()>, Error> {
    let (ready_tx, ready_rx) = mpsc::channel();
    let handle = thread::Builder::new()
        .name("windows-audio-capture".into())
        .spawn(move || {
            if let Err(err) = capture_audio_loop(shared.clone(), process_id, ready_tx) {
                shared.record_error(err);
            }
        })
        .map_err(|err| {
            Error::InitializationError(format!("Failed to spawn audio thread: {err}"))
        })?;

    match ready_rx.recv() {
        Ok(Ok(())) => Ok(handle),
        Ok(Err(message)) => {
            let _ = handle.join();
            Err(Error::InitializationError(format!(
                "Audio capture init failed: {message}"
            )))
        }
        Err(_) => {
            let _ = handle.join();
            Err(Error::InitializationError(
                "Audio capture thread did not report initialization status".into(),
            ))
        }
    }
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
fn capture_audio_loop(
    shared: Arc<SharedRecorderState>,
    process_id: Option<u32>,
    ready_tx: mpsc::Sender<Result<(), String>>,
) -> Result<(), String> {
    let _ = initialize_mta();

    let desired_format = WaveFormat::new(
        16,
        16,
        &SampleType::Int,
        AUDIO_SAMPLE_RATE as usize,
        AUDIO_CHANNELS as usize,
        None,
    );
    let blockalign = desired_format.get_blockalign();
    let chunk_bytes = (blockalign as usize) * AUDIO_CHUNK_FRAMES;

    let mut audio_client = match create_loopback_client(process_id) {
        Ok(client) => client,
        Err(err) => {
            let message = format!("Failed to create loopback client: {err:?}");
            let _ = ready_tx.send(Err(message.clone()));
            return Err(message);
        }
    };

    let mode = StreamMode::EventsShared {
        autoconvert: true,
        buffer_duration_hns: 0,
    };
    audio_client
        .initialize_client(&desired_format, &Direction::Capture, &mode)
        .map_err(|err| format!("Failed to initialize audio client: {err:?}"))?;

    let event_handle = audio_client
        .set_get_eventhandle()
        .map_err(|err| format!("Failed to create audio event handle: {err:?}"))?;
    let capture_client = audio_client
        .get_audiocaptureclient()
        .map_err(|err| format!("Failed to create audio capture client: {err:?}"))?;

    audio_client
        .start_stream()
        .map_err(|err| format!("Failed to start audio stream: {err:?}"))?;

    let _ = ready_tx.send(Ok(()));

    let mut sample_queue: VecDeque<u8> = VecDeque::with_capacity(chunk_bytes * 4);

    while !shared.should_stop() {
        while sample_queue.len() >= chunk_bytes {
            let mut chunk = vec![0u8; chunk_bytes];
            for byte in chunk.iter_mut() {
                *byte = sample_queue.pop_front().unwrap_or(0);
            }

            let mut guard = shared.encoder.lock().unwrap();
            if let Some(encoder) = guard.as_mut() {
                if let Err(err) = encoder.send_audio_buffer(&chunk, 0) {
                    return Err(format!("Failed to encode audio buffer: {err:?}"));
                }
            } else {
                return Ok(());
            }
        }

        capture_client
            .read_from_device_to_deque(&mut sample_queue)
            .map_err(|err| format!("Failed to capture audio: {err:?}"))?;

        if event_handle.wait_for_event(100_000).is_err() {
            break;
        }
    }

    audio_client
        .stop_stream()
        .map_err(|err| format!("Failed to stop audio stream: {err:?}"))?;

    Ok(())
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
fn create_loopback_client(
    process_id: Option<u32>,
) -> Result<AudioClient, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(pid) = process_id {
        match AudioClient::new_application_loopback_client(pid, true) {
            Ok(client) => return Ok(client),
            Err(err) => warn!("Falling back to system loopback capture: {err:?}"),
        }
    }

    let device = get_default_device(&Direction::Render)?;
    Ok(device.get_iaudioclient()?)
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
#[derive(Clone)]
struct TargetSelection {
    title: Option<String>,
    pid: Option<u32>,
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
impl TargetSelection {
    fn from_env() -> Self {
        let mut title = std::env::var("PEPPI_TARGET_WINDOW")
            .ok()
            .map(|s| s.trim().to_string());
        let mut pid = std::env::var("PEPPI_TARGET_PID")
            .ok()
            .and_then(|raw| raw.parse::<u32>().ok());

        if let Some(t) = &title {
            if let Some(idx) = t.rfind("(PID:") {
                if pid.is_none() {
                    let digits: String = t[idx + 5..]
                        .chars()
                        .filter(|ch| ch.is_ascii_digit())
                        .collect();
                    pid = digits.parse::<u32>().ok();
                }
                title = Some(t[..idx].trim().to_string());
            }
        }

        Self {
            title: title.filter(|s| !s.is_empty()),
            pid,
        }
    }
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
struct TargetWindow {
    window: CaptureWindow,
    title: String,
    pid: u32,
    width: u32,
    height: u32,
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
fn find_best_window(selection: &TargetSelection) -> Result<TargetWindow, Error> {
    let candidate = if let Some(pid) = selection.pid {
        pick_window_by_pid(pid, selection.title.as_deref())?
    } else if let Some(title) = &selection.title {
        pick_window_by_title(title)?
    } else {
        pick_default_window()?
    };

    let rect = candidate
        .rect()
        .map_err(|err| Error::RecordingFailed(format!("Failed to query window bounds: {err}")))?;
    let mut width = (rect.right - rect.left).max(0) as u32;
    let mut height = (rect.bottom - rect.top).max(0) as u32;
    width = even_dimension(width);
    height = even_dimension(height);

    if width < 2 || height < 2 {
        return Err(Error::RecordingFailed("Target window is too small".into()));
    }

    let pid = candidate
        .process_id()
        .map_err(|err| Error::RecordingFailed(format!("Failed to read process id: {err}")))?;
    let title = candidate
        .title()
        .unwrap_or_else(|_| "Unnamed Window".into());

    Ok(TargetWindow {
        window: candidate,
        title,
        pid,
        width,
        height,
    })
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
fn pick_window_by_pid(pid: u32, hint: Option<&str>) -> Result<CaptureWindow, Error> {
    let windows = CaptureWindow::enumerate()
        .map_err(|err| Error::RecordingFailed(format!("Failed to enumerate windows: {err}")))?;

    windows
        .into_iter()
        .filter_map(|window| match window.process_id() {
            Ok(id) if id == pid => Some(window),
            _ => None,
        })
        .max_by_key(|window| score_window(window, hint))
        .ok_or_else(|| Error::WindowNotFound)
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
fn pick_window_by_title(title: &str) -> Result<CaptureWindow, Error> {
    match CaptureWindow::from_name(title) {
        Ok(window) => Ok(window),
        Err(_) => CaptureWindow::from_contains_name(title).map_err(|_| Error::WindowNotFound),
    }
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
fn pick_default_window() -> Result<CaptureWindow, Error> {
    let windows = CaptureWindow::enumerate()
        .map_err(|err| Error::RecordingFailed(format!("Failed to enumerate windows: {err}")))?;
    windows
        .into_iter()
        .max_by_key(|window| score_window(window, Some("slippi")))
        .ok_or_else(|| Error::WindowNotFound)
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
fn score_window(window: &CaptureWindow, hint: Option<&str>) -> i64 {
    let width = window.width().unwrap_or(0) as i64;
    let height = window.height().unwrap_or(0) as i64;
    if width < 200 || height < 200 {
        return -1;
    }

    let mut score = width * height;
    let lc_title = window.title().unwrap_or_default().to_lowercase();
    let class_name = window_class_name(window);

    if let Some(h) = hint {
        if lc_title.contains(&h.to_lowercase()) {
            score += 1_000_000_000;
        }
    }

    if lc_title.contains("slippi") || lc_title.contains("melee") || lc_title.contains("dolphin") {
        score += 500_000_000;
    }

    if let Some(name) = class_name {
        let ln = name.to_lowercase();
        if ln.contains("d3dproxy") {
            score += 5_000_000_000;
        }
        if ln.contains("wxwindownr") {
            score += 1_000_000_000;
        }
    }

    score
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
fn window_class_name(window: &CaptureWindow) -> Option<String> {
    let mut buf = [0u16; 256];
    let len = unsafe { GetClassNameW(HWND(window.as_raw_hwnd()), &mut buf) };
    if len > 0 {
        Some(String::from_utf16_lossy(&buf[..len as usize]))
    } else {
        None
    }
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
#[inline]
fn even_dimension(value: u32) -> u32 {
    if value % 2 == 0 {
        value
    } else {
        value - 1
    }
}
