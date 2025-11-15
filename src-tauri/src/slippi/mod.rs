// Slippi replay file parsing and event extraction module

pub mod events;
pub mod parser;
pub mod types;

// Re-export commonly used items
pub use events::extract_death_events;
pub use parser::parse_slp_file;
pub use types::GameEvent;
