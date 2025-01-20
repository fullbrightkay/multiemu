use crate::{
    config::GLOBAL_CONFIG,
    rom::{id::RomId, info::RomInfo, manager::RomManager, system::GameSystem},
};
use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

pub fn run(
    path: PathBuf,
    system: GameSystem,
    name: Option<String>,
    symlink: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let global_config_guard = GLOBAL_CONFIG.try_read()?;
    let rom_manager = RomManager::new(Some(&global_config_guard.database_file))?;
    let database_transaction = rom_manager.rom_information.rw_transaction()?;
    let mut rom = BufReader::new(File::open(&path)?);
    let hash = RomId::from_read(&mut rom);

    tracing::info!("Imported ROM {:?} with hash {}", path, hash);

    database_transaction.upsert(RomInfo {
        name,
        hash,
        system,
        region: None,
    })?;

    database_transaction.commit()?;

    let internal_store_path = global_config_guard.roms_directory.join(hash.to_string());
    let _ = fs::remove_file(&internal_store_path);

    if symlink {
        #[cfg(unix)]
        std::os::unix::fs::symlink(path, internal_store_path)?;

        #[cfg(windows)]
        std::os::windows::fs::symlink_file(path, internal_store_path)?;

        #[cfg(not(any(unix, windows)))]
        panic!("Unsupported platform for symlinking");
    } else {
        fs::copy(path, internal_store_path).unwrap();
    }

    Ok(())
}
