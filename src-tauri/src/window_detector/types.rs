//! Type definitions for window detection

use serde::{Deserialize, Serialize};

/// Represents a detected game window
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameWindow {
    pub process_name: String,
    pub window_title: String,
    pub width: i32,
    pub height: i32,
    pub process_id: u32,
    pub class_name: String,
    pub is_cloaked: bool,
    pub is_child: bool,
    pub has_owner: bool,
}

impl GameWindow {
    /// Score a window to determine if it's likely the actual game render window
    /// Higher scores = more likely to be the game window
    pub fn score(&self) -> i32 {
        let mut s = 0;
        let title = self.window_title.to_lowercase();
        
        // Positive signals
        if title.contains("slippi") || title.contains("melee") || title.contains("dolphin") {
            s += 3;
        }
        
        // Negative signals (launcher, settings, etc.)
        if title.contains("launcher")
            || title.contains("settings")
            || title.contains("configuration")
        {
            s -= 3;
        }
        
        // Good size and not hidden
        if self.width >= 640 && self.height >= 480 && !self.is_cloaked {
            s += 3;
        }
        
        // Aspect ratio check (4:3 or 16:9)
        if self.height > 0 {
            let ar = (self.width as f32) / (self.height as f32);
            let d43 = (ar - (4.0 / 3.0)).abs();
            let d169 = (ar - (16.0 / 9.0)).abs();
            if d43 < 0.08 || d169 < 0.08 {
                s += 2;
            }
        }
        
        // Class name signals
        let class = self.class_name.to_lowercase();
        if class.contains("dolphin") || class.contains("wxwindownr") {
            s += 3;
        }
        if class.starts_with("#32770") || class.contains("tooltips") {
            s -= 4;
        }
        
        s
    }
    
    /// Check if this window matches Slippi/Dolphin/Melee keywords
    pub fn matches_game_keywords(&self) -> bool {
        let pn = self.process_name.to_lowercase();
        let tl = self.window_title.to_lowercase();
        let cn = self.class_name.to_lowercase();
        
        pn.contains("dolphin")
            || pn.contains("slippi")
            || pn.contains("melee")
            || tl.contains("slippi")
            || tl.contains("melee")
            || tl.contains("dolphin")
            || cn.contains("wxwindownr")
    }
    
    /// Check if window is a valid game window candidate (right size, not hidden)
    pub fn is_valid_candidate(&self) -> bool {
        self.width >= 640 && self.height >= 480 && !self.is_cloaked
    }
}

