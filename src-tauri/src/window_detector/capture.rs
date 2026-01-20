//! Windows-specific window capture for preview screenshots

use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS,
    HGDIOBJ, SRCCOPY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClientRect, GetWindowTextW, GetWindowThreadProcessId,
};

/// Context for window search enumeration
struct WindowSearchContext {
    pid: Option<u32>,
    needle: String,
    hwnd: Option<HWND>,
}

/// Capture a preview screenshot of a window identified by title/PID string
/// Returns PNG bytes on success
pub fn capture_window_preview(identifier: &str) -> Result<Vec<u8>, String> {
    let hwnd = find_window_handle(identifier).ok_or_else(|| {
        format!(
            "No window found matching identifier '{}'",
            identifier.trim()
        )
    })?;
    capture_hwnd_png(hwnd)
}

/// Parse identifier string to extract title and optional PID
fn parse_identifier(identifier: &str) -> (String, Option<u32>) {
    let trimmed = identifier.trim();
    if trimmed.is_empty() {
        return (String::new(), None);
    }
    if let Some(idx) = trimmed.rfind("(PID:") {
        let title = trimmed[..idx].trim_end().to_string();
        let digits: String = trimmed[idx + 5..]
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();
        let pid = digits.parse::<u32>().ok();
        (title, pid)
    } else {
        (trimmed.to_string(), None)
    }
}

/// Find window handle by identifier string
fn find_window_handle(identifier: &str) -> Option<HWND> {
    let (title, pid) = parse_identifier(identifier);
    if title.is_empty() && pid.is_none() {
        return None;
    }
    
    let mut ctx = WindowSearchContext {
        pid,
        needle: title.to_lowercase(),
        hwnd: None,
    };
    
    unsafe {
        let _ = EnumWindows(
            Some(find_window_enum_callback),
            LPARAM(&mut ctx as *mut WindowSearchContext as isize),
        );
    }
    
    // Fallback: if we had a PID but didn't find it, try title-only search
    if ctx.hwnd.is_none() && pid.is_some() {
        let mut fallback = WindowSearchContext {
            pid: None,
            needle: title.to_lowercase(),
            hwnd: None,
        };
        unsafe {
            let _ = EnumWindows(
                Some(find_window_enum_callback),
                LPARAM(&mut fallback as *mut WindowSearchContext as isize),
            );
        }
        return fallback.hwnd;
    }
    
    ctx.hwnd
}

unsafe extern "system" fn find_window_enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let ctx = &mut *(lparam.0 as *mut WindowSearchContext);
    
    if let Some(pid) = ctx.pid {
        let mut window_pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut window_pid));
        if window_pid != pid {
            return BOOL(1);
        }
    }
    
    let mut buf: [u16; 512] = [0; 512];
    let len = GetWindowTextW(hwnd, &mut buf);
    if len == 0 {
        return BOOL(1);
    }
    
    let title = String::from_utf16_lossy(&buf[..len as usize]).to_lowercase();
    if ctx.needle.is_empty() || title.contains(&ctx.needle) {
        ctx.hwnd = Some(hwnd);
        BOOL(0) // Stop enumeration
    } else {
        BOOL(1) // Continue
    }
}

/// Capture a window to PNG bytes
fn capture_hwnd_png(hwnd: HWND) -> Result<Vec<u8>, String> {
    unsafe {
        let mut rect = RECT::default();
        if GetClientRect(hwnd, &mut rect).is_err() {
            return Err("Failed to get window bounds".into());
        }
        
        let width = (rect.right - rect.left) as i32;
        let height = (rect.bottom - rect.top) as i32;
        if width <= 0 || height <= 0 {
            return Err("Window has invalid dimensions".into());
        }
        
        let hdc_window = GetDC(hwnd);
        if hdc_window.is_invalid() {
            return Err("Failed to acquire window device context".into());
        }
        
        let hdc_mem = CreateCompatibleDC(hdc_window);
        if hdc_mem.is_invalid() {
            ReleaseDC(hwnd, hdc_window);
            return Err("Failed to create memory device context".into());
        }
        
        let hbitmap = CreateCompatibleBitmap(hdc_window, width, height);
        if hbitmap.is_invalid() {
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(hwnd, hdc_window);
            return Err("Failed to create compatible bitmap".into());
        }
        
        let old_obj = SelectObject(hdc_mem, HGDIOBJ(hbitmap.0));
        if old_obj.is_invalid() {
            let _ = DeleteObject(HGDIOBJ(hbitmap.0));
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(hwnd, hdc_window);
            return Err("Failed to select bitmap into memory DC".into());
        }
        
        let blt_result = BitBlt(
            hdc_mem,
            0,
            0,
            width,
            height,
            hdc_window,
            0,
            0,
            SRCCOPY | CAPTUREBLT,
        );
        
        if let Err(err) = blt_result {
            let _ = SelectObject(hdc_mem, old_obj);
            let _ = DeleteObject(HGDIOBJ(hbitmap.0));
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(hwnd, hdc_window);
            return Err(format!("BitBlt failed while copying window content: {}", err));
        }
        
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [Default::default(); 1],
        };
        
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        let dib_res = GetDIBits(
            hdc_mem,
            hbitmap,
            0,
            height as u32,
            Some(pixels.as_mut_ptr().cast()),
            &mut info,
            DIB_RGB_COLORS,
        );
        
        let _ = SelectObject(hdc_mem, old_obj);
        let _ = DeleteObject(HGDIOBJ(hbitmap.0));
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(hwnd, hdc_window);
        
        if dib_res == 0 {
            return Err("Failed to read bitmap pixels".into());
        }
        
        // Convert BGRA -> RGBA
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }
        
        // Encode to PNG
        let mut png_data = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png_data, width as u32, height as u32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder
                .write_header()
                .map_err(|e| format!("Failed to write PNG header: {}", e))?;
            writer
                .write_image_data(&pixels)
                .map_err(|e| format!("Failed to encode PNG: {}", e))?;
        }
        
        Ok(png_data)
    }
}

