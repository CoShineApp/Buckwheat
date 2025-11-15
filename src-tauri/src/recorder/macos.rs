#![cfg_attr(
    all(target_os = "macos", feature = "real-recording"),
    allow(unexpected_cfgs)
)]

#[cfg(all(target_os = "macos", feature = "real-recording"))]
use super::{Error, Recorder};

#[cfg(all(target_os = "macos", feature = "real-recording"))]
use core_foundation::{
    base::{CFType, TCFType},
    dictionary::CFDictionary,
    number::CFNumber,
    string::{CFString, CFStringRef},
    url::CFURL,
};
#[cfg(all(target_os = "macos", feature = "real-recording"))]
use core_media_rs::{
    cm_sample_buffer::{CMSampleBuffer, CMSampleBufferRef},
    cm_time::CMTime,
};
#[cfg(all(target_os = "macos", feature = "real-recording"))]
use core_video_rs::cv_pixel_buffer::CVPixelBuffer;
#[cfg(all(target_os = "macos", feature = "real-recording"))]
use objc::{
    class, msg_send,
    rc::StrongPtr,
    runtime::{Object, Sel},
    sel, sel_impl, MessageArguments,
};
#[cfg(all(target_os = "macos", feature = "real-recording"))]
use screencapturekit::{
    shareable_content::{SCShareableContent, SCWindow},
    stream::{
        configuration::SCStreamConfiguration, content_filter::SCContentFilter,
        output_trait::SCStreamOutputTrait, output_type::SCStreamOutputType, SCStream,
    },
};
#[cfg(all(target_os = "macos", feature = "real-recording"))]
use std::any::Any;
#[cfg(all(target_os = "macos", feature = "real-recording"))]
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

#[cfg(all(target_os = "macos", feature = "real-recording"))]
const FALLBACK_WIDTH: i32 = 1280;
#[cfg(all(target_os = "macos", feature = "real-recording"))]
const FALLBACK_HEIGHT: i32 = 720;
#[cfg(all(target_os = "macos", feature = "real-recording"))]
const MIN_DIMENSION: i32 = 320;
#[cfg(all(target_os = "macos", feature = "real-recording"))]
const PIXEL_FORMAT_BGRA: i32 = 0x4247_5241; // 'BGRA'

#[cfg(all(target_os = "macos", feature = "real-recording"))]
type RawStreamOutput = *mut Object;

#[cfg(all(target_os = "macos", feature = "real-recording"))]
#[link(name = "CoreMedia", kind = "framework")]
extern "C" {
    fn CMSampleBufferGetPresentationTimeStamp(buffer: CMSampleBufferRef) -> CMTime;
}

#[cfg(all(target_os = "macos", feature = "real-recording"))]
pub struct MacOSRecorder {
    is_recording: bool,
    stream: Option<Arc<Mutex<SCStream>>>,
    output_handle: Option<RawStreamOutput>,
    writer: Option<Arc<Mutex<VideoWriter>>>,
    output_path: Option<String>,
}

#[cfg(all(target_os = "macos", feature = "real-recording"))]
unsafe impl Send for MacOSRecorder {}

#[cfg(all(target_os = "macos", feature = "real-recording"))]
impl MacOSRecorder {
    pub fn new() -> Self {
        Self {
            is_recording: false,
            stream: None,
            output_handle: None,
            writer: None,
            output_path: None,
        }
    }

    fn find_dolphin_window(&self) -> Result<SCWindow, Error> {
        let content = SCShareableContent::get().map_err(|e| {
            Error::InitializationError(format!("Failed to enumerate windows: {}", e))
        })?;

        let dolphin_window = content.windows().into_iter().find(|window| {
            let title = window.title();
            (title.contains("Slippi Dolphin")
                || title.contains("Melee")
                || (title.contains("Dolphin") && title.contains("FPS")))
                && !title.contains("Configuration")
                && !title.contains("Settings")
                && !title.contains("Graphics")
        });

        dolphin_window.ok_or(Error::WindowNotFound)
    }

    fn desired_dimensions(window: &SCWindow) -> (i32, i32) {
        let frame = window.get_frame();
        let width = frame.size.width.round() as i32;
        let height = frame.size.height.round() as i32;

        let width = width.clamp(MIN_DIMENSION, i32::MAX);
        let height = height.clamp(MIN_DIMENSION, i32::MAX);

        if width > 0 && height > 0 {
            (width, height)
        } else {
            (FALLBACK_WIDTH, FALLBACK_HEIGHT)
        }
    }

    fn initialize_stream(
        &mut self,
        output_path: &str,
        _quality: super::RecordingQuality,
    ) -> Result<(), Error> {
        // Note: macOS implementation uses VideoWriter which doesn't currently support
        // configurable bitrate, but we accept the parameter for API consistency
        let window = self.find_dolphin_window()?;
        log::info!("🎮 Found game window: {}", window.title());

        let (width, height) = Self::desired_dimensions(&window);
        log::info!("🖥️  Capturing window at {}x{}", width, height);

        let filter = SCContentFilter::new().with_desktop_independent_window(&window);
        let config = SCStreamConfiguration::new()
            .set_width(width as u32)
            .map_err(|e| Error::InitializationError(format!("Failed to set width: {e}")))?
            .set_height(height as u32)
            .map_err(|e| Error::InitializationError(format!("Failed to set height: {e}")))?
            .set_shows_cursor(true)
            .map_err(|e| Error::InitializationError(format!("Failed to set cursor flag: {e}")))?
            .set_captures_audio(false)
            .map_err(|e| Error::InitializationError(format!("Failed to disable audio: {e}")))?;

        let writer = VideoWriter::new(output_path, width, height)?;
        let writer_arc = Arc::new(Mutex::new(writer));

        let mut stream = SCStream::new(&filter, &config);
        let handler = StreamFrameHandler::new(writer_arc.clone());
        let handler_id = stream
            .add_output_handler(handler, SCStreamOutputType::Screen)
            .ok_or_else(|| {
                Error::InitializationError("Failed to attach screen output handler".into())
            })?;

        self.stream = Some(Arc::new(Mutex::new(stream)));
        self.output_handle = Some(handler_id);
        self.writer = Some(writer_arc);
        self.output_path = Some(output_path.to_string());

        Ok(())
    }
}

#[cfg(all(target_os = "macos", feature = "real-recording"))]
impl Recorder for MacOSRecorder {
    fn start_recording(
        &mut self,
        output_path: &str,
        quality: super::RecordingQuality,
    ) -> Result<(), Error> {
        if self.is_recording {
            return Err(Error::RecordingFailed("Already recording".into()));
        }

        log::info!(
            "🎥 [macOS] Starting recording to {} with {:?} quality (bitrate: {} Mbps)",
            output_path,
            quality,
            quality.bitrate() / 1_000_000
        );
        self.initialize_stream(output_path, quality)?;

        if let Some(stream_arc) = &self.stream {
            let stream_guard = stream_arc
                .lock()
                .map_err(|e| Error::InitializationError(format!("Failed to lock stream: {e}")))?;
            stream_guard
                .start_capture()
                .map_err(|e| Error::RecordingFailed(format!("Failed to start capture: {e}")))?;
        } else {
            return Err(Error::InitializationError(
                "Stream was not initialized".into(),
            ));
        }

        self.is_recording = true;
        Ok(())
    }

    fn stop_recording(&mut self) -> Result<String, Error> {
        if !self.is_recording {
            return Err(Error::RecordingFailed("Not recording".into()));
        }

        log::info!("⏹️  [macOS] Stopping recording");

        let stop_result = (|| -> Result<(), Error> {
            if let Some(stream_arc) = &self.stream {
                let mut stream = stream_arc
                    .lock()
                    .map_err(|e| Error::RecordingFailed(format!("Failed to lock stream: {e}")))?;
                stream
                    .stop_capture()
                    .map_err(|e| Error::RecordingFailed(format!("Failed to stop capture: {e}")))?;

                if let Some(handle) = self.output_handle.take() {
                    stream.remove_output_handler(handle, SCStreamOutputType::Screen);
                }
            }

            if let Some(writer) = &self.writer {
                let mut writer = writer
                    .lock()
                    .map_err(|e| Error::RecordingFailed(format!("Writer lock poisoned: {e}")))?;
                writer.finish()?;
            }

            Ok(())
        })();

        let output_path = self
            .output_path
            .clone()
            .unwrap_or_else(|| "recording.mp4".into());

        self.stream = None;
        self.writer = None;
        self.output_path = None;
        self.output_handle = None;
        self.is_recording = false;

        stop_result?;

        log::info!("✅ [macOS] Recording saved to {}", output_path);
        Ok(output_path)
    }

    fn is_recording(&self) -> bool {
        self.is_recording
    }
}

#[cfg(all(target_os = "macos", feature = "real-recording"))]
impl Default for MacOSRecorder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(target_os = "macos", feature = "real-recording"))]
struct StreamFrameHandler {
    writer: Arc<Mutex<VideoWriter>>,
}

#[cfg(all(target_os = "macos", feature = "real-recording"))]
impl StreamFrameHandler {
    fn new(writer: Arc<Mutex<VideoWriter>>) -> Self {
        Self { writer }
    }
}

#[cfg(all(target_os = "macos", feature = "real-recording"))]
impl SCStreamOutputTrait for StreamFrameHandler {
    fn did_output_sample_buffer(&self, sample_buffer: CMSampleBuffer, of_type: SCStreamOutputType) {
        if of_type != SCStreamOutputType::Screen {
            return;
        }

        if let Ok(mut writer) = self.writer.lock() {
            if let Err(err) = writer.append_sample_buffer(sample_buffer) {
                log::error!("Failed to append frame: {err}");
            }
        }
    }
}

#[cfg(all(target_os = "macos", feature = "real-recording"))]
struct VideoWriter {
    writer: StrongPtr,
    input: StrongPtr,
    adaptor: StrongPtr,
    started: bool,
    dropped_frames: usize,
}

#[cfg(all(target_os = "macos", feature = "real-recording"))]
unsafe impl Send for VideoWriter {}

#[cfg(all(target_os = "macos", feature = "real-recording"))]
impl VideoWriter {
    fn new(output_path: &str, width: i32, height: i32) -> Result<Self, Error> {
        let path = Path::new(output_path);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|err| {
                    Error::RecordingFailed(format!("Failed to create output directory: {err}"))
                })?;
            }
        }
        if path.exists() {
            std::fs::remove_file(path).map_err(|err| {
                Error::RecordingFailed(format!("Failed to overwrite existing file: {err}"))
            })?;
        }

        let file_url = CFURL::from_path(path, false)
            .ok_or_else(|| Error::RecordingFailed("Invalid output path".into()))?;

        let video_settings = video_output_settings(width as u32, height as u32)?;
        let pixel_attrs = pixel_buffer_attributes(width as u32, height as u32)?;

        unsafe {
            let mut error: *mut Object = std::ptr::null_mut();
            let writer_alloc: *mut Object = msg_send![class!(AVAssetWriter), alloc];
            let file_type = CFString::new("com.apple.mpeg-4");
            let writer_ptr: *mut Object = send_objc_message::<*mut Object, _>(
                writer_alloc,
                sel!(initWithURL:fileType:error:),
                (cf_to_obj(&file_url), cf_to_obj(&file_type), &mut error),
                "AVAssetWriter initWithURL:fileType:error:",
            )?;

            if writer_ptr.is_null() {
                return Err(Error::RecordingFailed(writer_error_string(error)));
            }

            let input_alloc: *mut Object = msg_send![class!(AVAssetWriterInput), alloc];
            let media_type = CFString::new("vide");
            let input_ptr: *mut Object = send_objc_message::<*mut Object, _>(
                input_alloc,
                sel!(initWithMediaType:outputSettings:),
                (cf_to_obj(&media_type), cf_to_obj(&video_settings)),
                "AVAssetWriterInput initWithMediaType:outputSettings:",
            )?;

            if input_ptr.is_null() {
                return Err(Error::RecordingFailed(
                    "Failed to create AVAssetWriterInput".into(),
                ));
            }

            let adaptor_alloc: *mut Object =
                msg_send![class!(AVAssetWriterInputPixelBufferAdaptor), alloc];
            let adaptor_ptr: *mut Object = send_objc_message::<*mut Object, _>(
                adaptor_alloc,
                sel!(initWithAssetWriterInput:sourcePixelBufferAttributes:),
                (input_ptr, cf_to_obj(&pixel_attrs)),
                "AVAssetWriterInputPixelBufferAdaptor init",
            )?;

            if adaptor_ptr.is_null() {
                return Err(Error::RecordingFailed(
                    "Failed to create AVAssetWriterInputPixelBufferAdaptor".into(),
                ));
            }

            let writer = StrongPtr::new(writer_ptr);
            let input = StrongPtr::new(input_ptr);
            let adaptor = StrongPtr::new(adaptor_ptr);

            let _: () = msg_send![*input, setExpectsMediaDataInRealTime: true];
            let can_add: bool = msg_send![*writer, canAddInput: *input];
            if !can_add {
                return Err(Error::RecordingFailed(
                    "AVAssetWriter rejected the video input".into(),
                ));
            }
            let _: () = msg_send![*writer, addInput: *input];

            Ok(Self {
                writer,
                input,
                adaptor,
                started: false,
                dropped_frames: 0,
            })
        }
    }

    fn append_sample_buffer(&mut self, sample_buffer: CMSampleBuffer) -> Result<(), Error> {
        sample_buffer
            .make_data_ready()
            .map_err(|e| Error::RecordingFailed(format!("Buffer not ready: {e:?}")))?;

        let timestamp =
            unsafe { CMSampleBufferGetPresentationTimeStamp(sample_buffer.as_concrete_TypeRef()) };
        let pixel_buffer = sample_buffer
            .get_pixel_buffer()
            .map_err(|e| Error::RecordingFailed(format!("Failed to get pixel buffer: {e:?}")))?;

        self.ensure_started(timestamp)?;
        self.append_pixel_buffer(pixel_buffer, timestamp)
    }

    fn ensure_started(&mut self, timestamp: CMTime) -> Result<(), Error> {
        if self.started {
            return Ok(());
        }

        unsafe {
            let started: bool = msg_send![*self.writer, startWriting];
            if !started {
                return Err(Error::RecordingFailed(
                    "AVAssetWriter could not start writing".into(),
                ));
            }

            let _: () = msg_send![*self.writer, startSessionAtSourceTime: timestamp];
        }

        self.started = true;
        Ok(())
    }

    fn append_pixel_buffer(
        &mut self,
        pixel_buffer: CVPixelBuffer,
        timestamp: CMTime,
    ) -> Result<(), Error> {
        unsafe {
            let ready: bool = msg_send![*self.input, isReadyForMoreMediaData];
            if !ready {
                self.dropped_frames += 1;
                if self.dropped_frames % 60 == 0 {
                    log::warn!("Dropped {} frames while encoding", self.dropped_frames);
                }
                return Ok(());
            }

            let appended: bool = msg_send![
                *self.adaptor,
                appendPixelBuffer: pixel_buffer.as_concrete_TypeRef() as *mut _
                withPresentationTime: timestamp
            ];

            if !appended {
                return Err(Error::RecordingFailed(format!(
                    "Failed to append pixel buffer: {}",
                    self.describe_writer_error()
                )));
            }
        }

        Ok(())
    }

    fn finish(&mut self) -> Result<(), Error> {
        if !self.started {
            unsafe {
                let _: () = msg_send![*self.writer, cancelWriting];
            }
            return Err(Error::RecordingFailed(
                "No frames captured before stopping".into(),
            ));
        }

        unsafe {
            let _: () = msg_send![*self.input, markAsFinished];
            let _: () = msg_send![*self.writer, finishWriting];
            let status: i32 = msg_send![*self.writer, status];
            if status == 3 {
                return Err(Error::RecordingFailed(format!(
                    "Writer failed during finalize: {}",
                    self.describe_writer_error()
                )));
            }
        }

        Ok(())
    }

    fn describe_writer_error(&self) -> String {
        unsafe {
            let error: *mut Object = msg_send![*self.writer, error];
            writer_error_string(error)
        }
    }
}

#[cfg(all(target_os = "macos", feature = "real-recording"))]
fn video_output_settings(width: u32, height: u32) -> Result<CFDictionary<CFString, CFType>, Error> {
    let codec_key = CFString::new("AVVideoCodecKey");
    let codec_value = CFString::new("avc1").as_CFType(); // H.264
    let width_key = CFString::new("AVVideoWidthKey");
    let height_key = CFString::new("AVVideoHeightKey");
    let width_value = CFNumber::from(width as i64).as_CFType();
    let height_value = CFNumber::from(height as i64).as_CFType();

    Ok(CFDictionary::<CFString, CFType>::from_CFType_pairs(&[
        (codec_key, codec_value),
        (width_key, width_value),
        (height_key, height_value),
    ]))
}

#[cfg(all(target_os = "macos", feature = "real-recording"))]
fn pixel_buffer_attributes(
    width: u32,
    height: u32,
) -> Result<CFDictionary<CFString, CFType>, Error> {
    let format_key = CFString::new("kCVPixelBufferPixelFormatTypeKey");
    let width_key = CFString::new("kCVPixelBufferWidthKey");
    let height_key = CFString::new("kCVPixelBufferHeightKey");

    let format_value = CFNumber::from(PIXEL_FORMAT_BGRA).as_CFType();
    let width_value = CFNumber::from(width as i64).as_CFType();
    let height_value = CFNumber::from(height as i64).as_CFType();

    Ok(CFDictionary::<CFString, CFType>::from_CFType_pairs(&[
        (format_key, format_value),
        (width_key, width_value),
        (height_key, height_value),
    ]))
}

#[cfg(all(target_os = "macos", feature = "real-recording"))]
fn writer_error_string(error: *mut Object) -> String {
    if error.is_null() {
        return "Unknown error".into();
    }

    unsafe {
        let description: *mut Object = msg_send![error, localizedDescription];
        nsstring_to_string(description)
    }
}

#[cfg(all(target_os = "macos", feature = "real-recording"))]
fn nsstring_to_string(ns_string: *mut Object) -> String {
    if ns_string.is_null() {
        return "Unknown error".into();
    }

    unsafe {
        let cf_ref = ns_string as CFStringRef;
        let cf_string = CFString::wrap_under_get_rule(cf_ref);
        cf_string.to_string()
    }
}

#[cfg(all(target_os = "macos", feature = "real-recording"))]
fn cf_to_obj<T: TCFType>(value: &T) -> *mut Object {
    value.as_CFTypeRef() as *mut _
}

#[cfg(all(target_os = "macos", feature = "real-recording"))]
unsafe fn send_objc_message<R, A>(
    target: *mut Object,
    selector: Sel,
    args: A,
    context: &str,
) -> Result<R, Error>
where
    A: MessageArguments,
    R: Any,
{
    objc::__send_message(target, selector, args)
        .map_err(|err| Error::RecordingFailed(format!("{context}: {err}")))
}
