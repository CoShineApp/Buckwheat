//! Windows screen recorder using windows-capture 2.0 + cpal for audio.
//!
//! - windows-capture: Hardware-accelerated video capture and encoding
//! - cpal: WASAPI loopback audio capture
//! - Audio sent to encoder via send_audio_buffer()

#![cfg_attr(
    all(target_os = "windows", feature = "real-recording"),
    allow(unexpected_cfgs)
)]

#[cfg(all(target_os = "windows", feature = "real-recording"))]
use super::{Error, Recorder};

#[cfg(all(target_os = "windows", feature = "real-recording"))]
use log::{debug, error, info, warn};
#[cfg(all(target_os = "windows", feature = "real-recording"))]
use std::env;
#[cfg(all(target_os = "windows", feature = "real-recording"))]
use std::path::Path;
#[cfg(all(target_os = "windows", feature = "real-recording"))]
use std::sync::mpsc;
#[cfg(all(target_os = "windows", feature = "real-recording"))]
use std::sync::{Arc, Mutex};
#[cfg(all(target_os = "windows", feature = "real-recording"))]
use std::time::Instant;

#[cfg(all(target_os = "windows", feature = "real-recording"))]
use windows_capture::{
    capture::{CaptureControl, Context, GraphicsCaptureApiHandler},
    encoder::{AudioSettingsBuilder, ContainerSettingsBuilder, VideoEncoder, VideoSettingsBuilder},
    frame::Frame,
    graphics_capture_api::InternalCaptureControl,
    monitor::Monitor,
    settings::{
        ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
        MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
    },
    window::Window,
};


/// Audio settings for the encoder
#[cfg(all(target_os = "windows", feature = "real-recording"))]
const AUDIO_SAMPLE_RATE: u32 = 48000;
#[cfg(all(target_os = "windows", feature = "real-recording"))]
const AUDIO_CHANNELS: u32 = 2;
#[cfg(all(target_os = "windows", feature = "real-recording"))]
const AUDIO_BITS_PER_SAMPLE: u32 = 16;

/// Shared state for capture coordination
#[cfg(all(target_os = "windows", feature = "real-recording"))]
struct CaptureState {
    stop_requested: bool,
    frame_count: u64,
    start_time: Option<Instant>,
    audio_receiver: Option<mpsc::Receiver<Vec<u8>>>,
}

/// Frame handler with VideoEncoder
#[cfg(all(target_os = "windows", feature = "real-recording"))]
struct FrameHandler {
    encoder: Option<VideoEncoder>,
    state: Arc<Mutex<CaptureState>>,
}

/// Flags passed to the frame handler
#[cfg(all(target_os = "windows", feature = "real-recording"))]
struct CaptureFlags {
    width: u32,
    height: u32,
    output_path: String,
    enable_audio: bool,
    bitrate: u32,
    state: Arc<Mutex<CaptureState>>,
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
impl GraphicsCaptureApiHandler for FrameHandler {
    type Flags = CaptureFlags;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        let flags = ctx.flags;
        
        info!(
            "Initializing VideoEncoder: {}x{}, audio: {}, bitrate: {} Mbps",
            flags.width, flags.height,
            if flags.enable_audio { "enabled (cpal)" } else { "disabled" },
            flags.bitrate / 1_000_000
        );

        // Build video settings
        let video_settings = VideoSettingsBuilder::new(flags.width, flags.height)
            .bitrate(flags.bitrate);

        // Build audio settings - we'll provide audio via send_audio_buffer()
        let audio_settings = if flags.enable_audio {
            AudioSettingsBuilder::default()
                .sample_rate(AUDIO_SAMPLE_RATE)
                .channel_count(AUDIO_CHANNELS)
                .bit_per_sample(AUDIO_BITS_PER_SAMPLE)
                .disabled(false)
        } else {
            AudioSettingsBuilder::default().disabled(true)
        };

        // Create the encoder
        let encoder = VideoEncoder::new(
            video_settings,
            audio_settings,
            ContainerSettingsBuilder::default(),
            &flags.output_path,
        )?;

        info!("VideoEncoder initialized successfully");

        Ok(Self {
            encoder: Some(encoder),
            state: flags.state,
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        let mut state = self.state.lock().map_err(|e| format!("Lock poisoned: {}", e))?;

        // Check if stop requested
        if state.stop_requested {
            if let Some(encoder) = self.encoder.take() {
                info!("Finishing encoder...");
                encoder.finish()?;
                info!("Encoder finished successfully");
            }
            capture_control.stop();
            return Ok(());
        }

        // Initialize start time on first frame
        if state.start_time.is_none() {
            state.start_time = Some(Instant::now());
            info!("First frame received, recording started");
        }

        state.frame_count += 1;
        let frame_count = state.frame_count;
        
        // Collect audio data from cpal
        let mut audio_data = Vec::new();
        if let Some(ref receiver) = state.audio_receiver {
            while let Ok(buffer) = receiver.try_recv() {
                audio_data.extend(buffer);
            }
        }
        
        drop(state); // Release lock before encoding

        // Send frame and audio to encoder
        if let Some(ref mut encoder) = self.encoder {
            encoder.send_frame(frame)?;
            
            // Send audio if we have any
            if !audio_data.is_empty() {
                if let Err(e) = encoder.send_audio_buffer(&audio_data, 0) {
                    if frame_count == 1 {
                        warn!("Audio send error: {}", e);
                    }
                }
            }
        }

        // Log progress
        if frame_count == 1 {
            info!("First frame encoded (audio buffer: {} bytes)", audio_data.len());
        } else if frame_count % 300 == 0 {
            debug!("Encoded {} frames", frame_count);
        }

        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        info!("Capture session closed");
        if let Some(encoder) = self.encoder.take() {
            encoder.finish()?;
        }
        Ok(())
    }
}

/// Audio capture using cpal - runs in a dedicated thread to be Send-safe
#[cfg(all(target_os = "windows", feature = "real-recording"))]
struct AudioCapture {
    stop_flag: Arc<Mutex<bool>>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
impl AudioCapture {
    fn start() -> Result<(Self, mpsc::Receiver<Vec<u8>>), String> {
        let (sender, receiver) = mpsc::channel();
        let stop_flag = Arc::new(Mutex::new(false));
        let stop_flag_clone = stop_flag.clone();

        // Spawn thread to own the stream (cpal::Stream is not Send)
        let thread_handle = std::thread::spawn(move || {
            if let Err(e) = Self::run_audio_capture(sender, stop_flag_clone) {
                error!("Audio capture thread error: {}", e);
            }
        });

        // Give the thread a moment to initialize
        std::thread::sleep(std::time::Duration::from_millis(100));

        Ok((
            Self {
                stop_flag,
                thread_handle: Some(thread_handle),
            },
            receiver,
        ))
    }

    fn run_audio_capture(
        sender: mpsc::Sender<Vec<u8>>,
        stop_flag: Arc<Mutex<bool>>,
    ) -> Result<(), String> {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

        let host = cpal::default_host();
        
        // Get default output device for loopback capture
        let device = host.default_output_device()
            .ok_or_else(|| "No output device available".to_string())?;
        
        let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        info!("Audio capture device: {}", device_name);

        // Configure stream
        let config = cpal::StreamConfig {
            channels: AUDIO_CHANNELS as u16,
            sample_rate: cpal::SampleRate(AUDIO_SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Default,
        };

        info!("Audio config: {} Hz, {} channels", AUDIO_SAMPLE_RATE, AUDIO_CHANNELS);

        // Build input stream for loopback
        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let pcm_data = convert_f32_to_i16_pcm(data);
                let _ = sender.send(pcm_data);
            },
            |err| {
                error!("Audio stream error: {}", err);
            },
            None,
        ).map_err(|e| format!("Failed to build audio stream: {}", e))?;

        stream.play().map_err(|e| format!("Failed to start audio stream: {}", e))?;
        info!("Audio capture started");

        // Keep thread alive until stop is requested
        loop {
            if *stop_flag.lock().unwrap() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        drop(stream);
        info!("Audio capture stopped");
        Ok(())
    }

    fn stop(&mut self) {
        // Signal stop
        if let Ok(mut flag) = self.stop_flag.lock() {
            *flag = true;
        }
        // Wait for thread to finish
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
impl Drop for AudioCapture {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Convert f32 audio samples to 16-bit signed integer PCM bytes
#[cfg(all(target_os = "windows", feature = "real-recording"))]
fn convert_f32_to_i16_pcm(samples: &[f32]) -> Vec<u8> {
    let mut output = Vec::with_capacity(samples.len() * 2);
    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let i16_sample = (clamped * 32767.0) as i16;
        output.extend_from_slice(&i16_sample.to_le_bytes());
    }
    output
}

/// Capture target enum
#[cfg(all(target_os = "windows", feature = "real-recording"))]
enum CaptureTarget {
    Window(Window),
    Monitor(Monitor),
}

/// Type alias for capture control
#[cfg(all(target_os = "windows", feature = "real-recording"))]
type WindowCaptureControl = CaptureControl<FrameHandler, Box<dyn std::error::Error + Send + Sync>>;

#[cfg(all(target_os = "windows", feature = "real-recording"))]
pub struct WindowsRecorder {
    capture_control: Option<WindowCaptureControl>,
    capture_state: Option<Arc<Mutex<CaptureState>>>,
    audio_capture: Option<AudioCapture>,
    output_path: Option<String>,
    is_recording: bool,
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
impl WindowsRecorder {
    pub fn new() -> Self {
        Self {
            capture_control: None,
            capture_state: None,
            audio_capture: None,
            output_path: None,
            is_recording: false,
        }
    }

    fn ensure_output_dir(&self, output_path: &str) -> Result<(), Error> {
        if let Some(parent) = Path::new(output_path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|err| {
                    Error::RecordingFailed(format!("Failed to create output directory: {err}"))
                })?;
            }
        }
        Ok(())
    }

    fn find_target(&self) -> Result<CaptureTarget, Error> {
        let selection = TargetSelection::from_env();

        let windows = Window::enumerate()
            .map_err(|e| Error::RecordingFailed(format!("Failed to enumerate windows: {}", e)))?;

        let best_match = if selection.pid.is_some() || selection.title.is_some() {
            let hint = selection.title.as_deref();
            windows
                .into_iter()
                .filter(|w| {
                    w.title()
                        .map(|t| {
                            let lower = t.to_lowercase();
                            if let Some(h) = hint {
                                lower.contains(&h.to_lowercase())
                            } else {
                                lower.contains("slippi")
                                    || lower.contains("dolphin")
                                    || lower.contains("melee")
                            }
                        })
                        .unwrap_or(false)
                })
                .max_by_key(|w| score_window(w, hint))
        } else {
            windows
                .into_iter()
                .filter(|w| {
                    w.title()
                        .map(|t| {
                            let lower = t.to_lowercase();
                            lower.contains("slippi")
                                || lower.contains("dolphin")
                                || lower.contains("melee")
                        })
                        .unwrap_or(false)
                })
                .max_by_key(|w| score_window(w, Some("slippi")))
        };

        if let Some(window) = best_match {
            if let Ok(title) = window.title() {
                info!("Selected capture target: '{}'", title);
            }
            Ok(CaptureTarget::Window(window))
        } else {
            info!("No matching window found, capturing primary monitor");
            let monitor = Monitor::primary()
                .map_err(|e| Error::RecordingFailed(format!("Failed to get primary monitor: {}", e)))?;
            Ok(CaptureTarget::Monitor(monitor))
        }
    }

    fn get_target_size(&self, target: &CaptureTarget) -> Result<(u32, u32), Error> {
        match target {
            CaptureTarget::Window(window) => {
                let rect = window.rect()
                    .map_err(|e| Error::RecordingFailed(format!("Failed to get window rect: {}", e)))?;
                let w = (rect.right - rect.left).max(640) as u32;
                let h = (rect.bottom - rect.top).max(480) as u32;
                Ok((w, h))
            }
            CaptureTarget::Monitor(monitor) => {
                let w = monitor.width().unwrap_or(1920);
                let h = monitor.height().unwrap_or(1080);
                Ok((w, h))
            }
        }
    }

    fn start_window_capture(
        &self,
        window: Window,
        flags: CaptureFlags,
    ) -> Result<WindowCaptureControl, Error> {
        let settings = Settings::new(
            window,
            CursorCaptureSettings::Default,
            DrawBorderSettings::Default,
            SecondaryWindowSettings::Default,
            MinimumUpdateIntervalSettings::Default,
            DirtyRegionSettings::Default,
            ColorFormat::Bgra8,
            flags,
        );

        FrameHandler::start_free_threaded(settings)
            .map_err(|e| Error::RecordingFailed(format!("Failed to start window capture: {}", e)))
    }

    fn start_monitor_capture(
        &self,
        monitor: Monitor,
        flags: CaptureFlags,
    ) -> Result<WindowCaptureControl, Error> {
        let settings = Settings::new(
            monitor,
            CursorCaptureSettings::Default,
            DrawBorderSettings::Default,
            SecondaryWindowSettings::Default,
            MinimumUpdateIntervalSettings::Default,
            DirtyRegionSettings::Default,
            ColorFormat::Bgra8,
            flags,
        );

        FrameHandler::start_free_threaded(settings)
            .map_err(|e| Error::RecordingFailed(format!("Failed to start monitor capture: {}", e)))
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

        let target = self.find_target()?;
        let (width, height) = self.get_target_size(&target)?;
        
        // Ensure even dimensions for H.264
        let width = (width / 2) * 2;
        let height = (height / 2) * 2;

        info!("Capture dimensions: {}x{}", width, height);

        // Check if audio should be enabled
        let enable_audio = resolve_audio_enabled();
        
        // Start audio capture with cpal
        let audio_receiver = if enable_audio {
            match AudioCapture::start() {
                Ok((audio_capture, receiver)) => {
                    self.audio_capture = Some(audio_capture);
                    Some(receiver)
                }
                Err(e) => {
                    warn!("Failed to start audio capture: {}, continuing without audio", e);
                    None
                }
            }
        } else {
            info!("Audio capture disabled");
            None
        };

        // Create shared state
        let capture_state = Arc::new(Mutex::new(CaptureState {
            stop_requested: false,
            frame_count: 0,
            start_time: None,
            audio_receiver,
        }));

        // Create flags for the capture handler
        let flags = CaptureFlags {
            width,
            height,
            output_path: output_path.to_string(),
            enable_audio: self.audio_capture.is_some(),
            bitrate: quality.bitrate(),
            state: capture_state.clone(),
        };

        // Start capture
        let capture_control = match target {
            CaptureTarget::Window(window) => self.start_window_capture(window, flags)?,
            CaptureTarget::Monitor(monitor) => self.start_monitor_capture(monitor, flags)?,
        };

        self.capture_control = Some(capture_control);
        self.capture_state = Some(capture_state);
        self.output_path = Some(output_path.to_string());
        self.is_recording = true;

        info!("Recording started: {}", output_path);
        Ok(())
    }

    fn stop_recording(&mut self) -> Result<String, Error> {
        if !self.is_recording {
            return Err(Error::RecordingFailed("Not recording".into()));
        }

        info!("Stopping recording...");

        // Stop audio first
        if let Some(mut audio) = self.audio_capture.take() {
            audio.stop();
        }

        // Signal stop
        if let Some(ref state) = self.capture_state {
            if let Ok(mut s) = state.lock() {
                s.stop_requested = true;
                info!("Recorded {} frames", s.frame_count);
            }
        }

        // Stop capture
        if let Some(control) = self.capture_control.take() {
            let _ = control.stop();
        }

        let output = self.output_path.take().unwrap_or_default();
        self.capture_state = None;
        self.is_recording = false;

        info!("Recording saved to {}", output);
        Ok(output)
    }

    fn is_recording(&self) -> bool {
        self.is_recording
    }
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
impl WindowsRecorder {
    pub fn set_target_window(&mut self, name: &str) {
        env::set_var("PEPPI_TARGET_WINDOW", name);
        info!("Set target window hint: {}", name);
    }
}

// ============================================================================
// Helper functions
// ============================================================================

#[cfg(all(target_os = "windows", feature = "real-recording"))]
fn resolve_audio_enabled() -> bool {
    match env::var("PEPPI_AUDIO") {
        Ok(val) => {
            let val = val.to_lowercase();
            !matches!(val.as_str(), "false" | "0" | "none" | "disabled")
        }
        Err(_) => true,
    }
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
fn score_window(window: &Window, hint: Option<&str>) -> i32 {
    let mut score = 0;

    if let Ok(title) = window.title() {
        let lower = title.to_lowercase();

        if let Some(h) = hint {
            if lower.contains(&h.to_lowercase()) { score += 100; }
        }

        if lower.contains("slippi") { score += 50; }
        if lower.contains("dolphin") { score += 30; }
        if lower.contains("melee") { score += 40; }
        if lower.contains("faster") { score += 20; }

        if lower.contains("discord") || lower.contains("chrome") || lower.contains("firefox") {
            score -= 50;
        }
    }

    if let Ok(rect) = window.rect() {
        let area = (rect.right - rect.left) * (rect.bottom - rect.top);
        if area > 800 * 600 { score += 10; }
    }

    score
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
        let mut title = env::var("PEPPI_TARGET_WINDOW")
            .ok()
            .map(|value| value.trim().to_string());
        let mut pid = env::var("PEPPI_TARGET_PID")
            .ok()
            .and_then(|value| value.parse::<u32>().ok());

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

// ============================================================================
// Stub for non-Windows builds
// ============================================================================

#[cfg(not(all(target_os = "windows", feature = "real-recording")))]
pub struct WindowsRecorder;

#[cfg(not(all(target_os = "windows", feature = "real-recording")))]
impl WindowsRecorder {
    pub fn new() -> Self {
        Self
    }
}
