// Type definitions for game events

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GameEvent {
    #[serde(rename = "death")]
    Death(DeathEvent),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeathEvent {
    pub frame: i32,
    pub timestamp: f64,
    pub port: u8,
    pub player_tag: String,
}
