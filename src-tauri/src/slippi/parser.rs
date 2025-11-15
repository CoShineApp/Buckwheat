// Core .slp file parsing logic

use crate::commands::errors::Error;
use std::fs::File;
use std::io::BufReader;

/// Parse a .slp file and return the game data
pub fn parse_slp_file(slp_path: &str) -> Result<peppi::game::immutable::Game, Error> {
    log::info!("📊 Parsing .slp file: {}", slp_path);

    let file = File::open(slp_path)
        .map_err(|e| Error::InvalidPath(format!("Failed to open .slp file: {}", e)))?;

    let mut reader = BufReader::new(file);

    let game = peppi::io::slippi::read(&mut reader, None)
        .map_err(|e| Error::RecordingFailed(format!("Failed to parse .slp file: {:?}", e)))?;

    log::info!("✅ Successfully parsed .slp file");
    Ok(game)
}
