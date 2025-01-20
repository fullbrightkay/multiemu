use crate::{config::GLOBAL_CONFIG, rom::manager::RomManager};
use std::path::PathBuf;

pub fn run(directory: Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let global_config_guard = GLOBAL_CONFIG.try_read()?;
    let rom_manager = RomManager::new(Some(&global_config_guard.database_file))?;

    for path in &directory {
        rom_manager.load_database(path)?;
    }

    Ok(())
}
