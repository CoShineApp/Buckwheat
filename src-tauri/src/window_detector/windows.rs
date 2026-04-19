//! Windows-specific window enumeration and detection

use super::types::GameWindow;
use std::collections::{HashMap, HashSet};
use sysinfo::System;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::Graphics::Gdi::{
    CreatePen, CreateSolidBrush, DeleteObject, GetDC, Rectangle, ReleaseDC, SelectObject,
    SetROP2, PS_SOLID, R2_NOTXORPEN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetWindow, GetWindowRect, GetWindowTextW,
    GetWindowThreadProcessId, SetForegroundWindow, GW_OWNER,
};

/// Context for child window enumeration
struct ChildEnumContext {
    windows: Vec<GameWindow>,
    parent_pid: u32,
}

/// Enumerate all visible top-level windows with a title, regardless of app.
/// Used by the capture window picker so users can select any app to record.
pub fn find_all_windows() -> Vec<GameWindow> {
    let mut sys = System::new_all();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

    let mut pid_to_name: HashMap<u32, String> = HashMap::new();
    for (pid, process) in sys.processes() {
        pid_to_name.insert(pid.as_u32(), process.name().to_string_lossy().to_string());
    }

    let mut windows: Vec<GameWindow> = Vec::new();

    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut windows as *mut Vec<GameWindow> as isize),
        );
    }

    for w in &mut windows {
        if let Some(name) = pid_to_name.get(&w.process_id) {
            w.process_name = name.clone();
        }
    }

    // Keep only real windows: has title, visible, reasonable size
    windows.retain(|w| {
        !w.window_title.trim().is_empty()
            && !w.is_cloaked
            && w.width >= 200
            && w.height >= 100
    });

    // De-duplicate
    let mut seen: HashSet<String> = HashSet::new();
    windows.retain(|w| {
        let key = format!("{}:{}:{}", w.process_id, w.class_name, w.window_title);
        seen.insert(key)
    });

    // Sort game candidates first, then alphabetical by title
    windows.sort_by(|a, b| {
        let a_game = a.matches_game_keywords();
        let b_game = b.matches_game_keywords();
        match (a_game, b_game) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.window_title.to_lowercase().cmp(&b.window_title.to_lowercase()),
        }
    });

    windows
}

/// Check if any visible window exists for a process matching the given name.
/// Case-insensitive substring match on the process executable name.
pub fn check_window_by_process(process_name: &str) -> bool {
    let needle = process_name.to_lowercase();

    let mut sys = System::new_all();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

    let matching_pids: HashSet<u32> = sys
        .processes()
        .iter()
        .filter(|(_, p)| p.name().to_string_lossy().to_lowercase().contains(&needle))
        .map(|(pid, _)| pid.as_u32())
        .collect();

    if matching_pids.is_empty() {
        return false;
    }

    let mut windows: Vec<GameWindow> = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut windows as *mut Vec<GameWindow> as isize),
        );
    }

    windows.iter().any(|w| {
        matching_pids.contains(&w.process_id)
            && !w.is_cloaked
            && w.width >= 200
            && w.height >= 100
    })
}

/// Find all potential game windows (Slippi/Dolphin)
pub fn find_game_windows() -> Vec<GameWindow> {
    // Get all processes
    let mut sys = System::new_all();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);
    
    // Map PIDs to process names
    let mut pid_to_name: HashMap<u32, String> = HashMap::new();
    for (pid, process) in sys.processes() {
        pid_to_name.insert(pid.as_u32(), process.name().to_string_lossy().to_string());
    }
    
    let mut windows: Vec<GameWindow> = Vec::new();
    
    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut windows as *mut Vec<GameWindow> as isize),
        );
    }
    
    // Attach process names
    for w in &mut windows {
        if let Some(name) = pid_to_name.get(&w.process_id) {
            w.process_name = name.clone();
        }
    }
    
    // Pre-filter to likely candidates
    let prefiltered: Vec<GameWindow> = windows
        .clone()
        .into_iter()
        .filter(|w| w.matches_game_keywords() && w.is_valid_candidate())
        .collect();
    
    // Score and sort
    let mut scored: Vec<GameWindow> = prefiltered
        .into_iter()
        .filter(|w| w.score() >= 2)
        .collect();
    scored.sort_by_key(|w| -w.score());
    
    // Use scored results or fall back to basic filter
    let mut game_windows: Vec<GameWindow> = if !scored.is_empty() {
        scored
    } else {
        windows
            .into_iter()
            .filter(|w| {
                let title_lower = w.window_title.to_lowercase();
                (title_lower.contains("slippi")
                    || title_lower.contains("melee")
                    || title_lower.contains("dolphin"))
                    && !title_lower.contains("launcher")
                    && !title_lower.contains("settings")
                    && !title_lower.contains("configuration")
                    && w.is_valid_candidate()
            })
            .collect()
    };
    
    // De-duplicate
    let mut seen: HashSet<String> = HashSet::new();
    game_windows.retain(|w| {
        let key = format!(
            "{}:{}x{}:{}:{}",
            w.process_id, w.width, w.height, w.class_name, w.window_title
        );
        seen.insert(key)
    });
    
    for window in &game_windows {
        log::info!(
            "  - PID: {} | Title: {} | Size: {}x{} | Class: {} | Cloaked: {} | Child: {} | HasOwner: {}",
            window.process_id,
            window.window_title,
            window.width,
            window.height,
            window.class_name,
            window.is_cloaked,
            window.is_child,
            window.has_owner
        );
    }
    
    game_windows
}

/// Check if the game window is currently open
/// Optionally narrow search using stored identifier (window title or PID)
pub fn check_game_window_open(stored_id: Option<&str>) -> bool {
    let mut windows: Vec<GameWindow> = Vec::new();
    
    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut windows as *mut Vec<GameWindow> as isize),
        );
        
        // Also check child windows
        let copy = windows.clone();
        for parent in copy {
            let mut ctx = ChildEnumContext {
                windows: Vec::new(),
                parent_pid: parent.process_id,
            };
            let _ = EnumWindows(
                Some(enum_child_windows_callback),
                LPARAM(&mut ctx as *mut ChildEnumContext as isize),
            );
            windows.extend(ctx.windows);
        }
    }
    
    // Attach process names
    let mut sys = System::new_all();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);
    for w in &mut windows {
        if let Some(p) = sys.process(sysinfo::Pid::from_u32(w.process_id)) {
            w.process_name = p.name().to_string_lossy().to_string();
        }
    }
    
    // Parse stored identifier for filtering
    let (pid_filter, title_filter) = parse_stored_identifier(stored_id);
    
    // Filter candidates
    let mut candidates: Vec<&GameWindow> = windows.iter().collect();
    
    if let Some(pid) = pid_filter {
        candidates.retain(|w| w.process_id == pid);
    }
    if let Some(ref tf) = title_filter {
        candidates.retain(|w| w.window_title.to_lowercase().contains(tf));
    }
    if pid_filter.is_none() && title_filter.is_none() {
        candidates.retain(|w| {
            let pn = w.process_name.to_lowercase();
            pn.contains("dolphin") || pn.contains("slippi") || pn.contains("melee")
        });
    }
    
    // Find best candidate
    let best = candidates.into_iter().max_by_key(|w| w.score());
    if let Some(w) = best {
        return w.score() >= 4;
    }
    
    false
}

/// Parse a stored identifier string into PID and/or title filter
fn parse_stored_identifier(stored_id: Option<&str>) -> (Option<u32>, Option<String>) {
    let Some(id) = stored_id else {
        return (None, None);
    };
    
    if let Some(pos) = id.find("PID:") {
        let after = id[pos + 4..].trim_start();
        let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        let pid = digits.parse::<u32>().ok();
        (pid, None)
    } else {
        (None, Some(id.to_lowercase()))
    }
}

unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<GameWindow>);
    
    // Get window title
    let mut title: [u16; 512] = [0; 512];
    let len = GetWindowTextW(hwnd, &mut title);
    let window_title = if len > 0 {
        String::from_utf16_lossy(&title[..len as usize])
    } else {
        "(No Title)".to_string()
    };
    
    // Get window dimensions
    let mut rect = RECT::default();
    if GetWindowRect(hwnd, &mut rect).is_ok() {
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        
        // Get process ID
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
        
        // Get window class name
        let mut class_name: [u16; 256] = [0; 256];
        let class_len = GetClassNameW(hwnd, &mut class_name);
        let class_name_str = if class_len > 0 {
            String::from_utf16_lossy(&class_name[..class_len as usize])
        } else {
            "Unknown".to_string()
        };
        
        // Check if window is cloaked
        let mut is_cloaked = 0u32;
        let cloaked = DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut is_cloaked as *mut _ as *mut _,
            std::mem::size_of::<u32>() as u32,
        )
        .is_ok()
            && is_cloaked != 0;
        
        // Check if window has an owner
        let has_owner = GetWindow(hwnd, GW_OWNER)
            .map(|h| !h.is_invalid())
            .unwrap_or(false);
        
        windows.push(GameWindow {
            process_name: format!("PID: {}", process_id),
            window_title: window_title.clone(),
            width,
            height,
            process_id,
            class_name: class_name_str,
            is_cloaked: cloaked,
            is_child: false,
            has_owner,
        });
    }
    
    BOOL::from(true) // Continue enumeration
}

unsafe extern "system" fn enum_child_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let context = &mut *(lparam.0 as *mut ChildEnumContext);
    
    // Get process ID
    let mut process_id: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut process_id));
    
    // Only process windows from the same process as parent
    if process_id == context.parent_pid {
        let mut title: [u16; 512] = [0; 512];
        let len = GetWindowTextW(hwnd, &mut title);
        let window_title = if len > 0 {
            String::from_utf16_lossy(&title[..len as usize])
        } else {
            "(No Title - Child)".to_string()
        };
        
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            
            let mut class_name: [u16; 256] = [0; 256];
            let class_len = GetClassNameW(hwnd, &mut class_name);
            let class_name_str = if class_len > 0 {
                String::from_utf16_lossy(&class_name[..class_len as usize])
            } else {
                "Unknown".to_string()
            };
            
            let mut is_cloaked = 0u32;
            let cloaked = DwmGetWindowAttribute(
                hwnd,
                DWMWA_CLOAKED,
                &mut is_cloaked as *mut _ as *mut _,
                std::mem::size_of::<u32>() as u32,
            )
            .is_ok()
                && is_cloaked != 0;
            
            let has_owner = GetWindow(hwnd, GW_OWNER)
                .map(|h| !h.is_invalid())
                .unwrap_or(false);
            
            // Only add if it has reasonable dimensions
            if width > 100 && height > 100 {
                context.windows.push(GameWindow {
                    process_name: format!("PID: {} (Child)", process_id),
                    window_title,
                    width,
                    height,
                    process_id,
                    class_name: class_name_str,
                    is_cloaked: cloaked,
                    is_child: true,
                    has_owner,
                });
            }
        }
    }
    
    BOOL::from(true) // Continue enumeration
}

/// Context for finding a window by process_id
struct FindWindowContext {
    target_pid: u32,
    found_hwnd: Option<HWND>,
}

unsafe extern "system" fn find_window_by_pid_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let ctx = &mut *(lparam.0 as *mut FindWindowContext);
    
    let mut process_id: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut process_id));
    
    if process_id == ctx.target_pid {
        ctx.found_hwnd = Some(hwnd);
        return BOOL::from(false); // Stop enumeration
    }
    
    BOOL::from(true) // Continue enumeration
}

/// Highlight a window by bringing it to the foreground and drawing a yellow border
/// The border is drawn using XOR so it can be erased by drawing again
pub fn highlight_window(process_id: u32) -> Result<(), String> {
    unsafe {
        // Find the window handle by process ID
        let mut ctx = FindWindowContext {
            target_pid: process_id,
            found_hwnd: None,
        };
        
        let _ = EnumWindows(
            Some(find_window_by_pid_callback),
            LPARAM(&mut ctx as *mut FindWindowContext as isize),
        );
        
        let hwnd = ctx.found_hwnd.ok_or_else(|| {
            format!("No window found for process ID {}", process_id)
        })?;
        
        // Bring window to foreground
        let _ = SetForegroundWindow(hwnd);
        
        // Get window rectangle
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return Err("Failed to get window rect".to_string());
        }
        
        // Get screen DC to draw on
        let hdc = GetDC(None);
        if hdc.is_invalid() {
            return Err("Failed to get DC".to_string());
        }
        
        // Create a thick yellow pen (5 pixels)
        let yellow = 0x00FFFF_u32; // BGR format: Yellow
        let pen = CreatePen(PS_SOLID, 5, windows::Win32::Foundation::COLORREF(yellow));
        let brush = CreateSolidBrush(windows::Win32::Foundation::COLORREF(0)); // Hollow brush
        
        let old_pen = SelectObject(hdc, pen);
        let old_brush = SelectObject(hdc, brush);
        let _old_rop = SetROP2(hdc, R2_NOTXORPEN);
        
        // Draw rectangle border
        let _ = Rectangle(hdc, rect.left, rect.top, rect.right, rect.bottom);
        
        // Wait a bit
        std::thread::sleep(std::time::Duration::from_millis(1500));
        
        // Erase by drawing again (XOR)
        let _ = Rectangle(hdc, rect.left, rect.top, rect.right, rect.bottom);
        
        // Cleanup
        SelectObject(hdc, old_brush);
        SelectObject(hdc, old_pen);
        let _ = DeleteObject(pen);
        let _ = DeleteObject(brush);
        ReleaseDC(None, hdc);
        
        Ok(())
    }
}

