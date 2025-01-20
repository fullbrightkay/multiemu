use crate::{
    config::{GlobalConfig, GLOBAL_CONFIG},
    rom::{id::RomId, info::RomInfo, manager::RomManager},
};
use rayon::prelude::{ParallelBridge, ParallelIterator};
use std::{
    fs::{self, File},
    ops::Deref,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;
use zip::ZipArchive;

pub fn run(paths: Vec<PathBuf>, symlink: bool) -> Result<(), Box<dyn std::error::Error>> {
    let global_config_guard = GLOBAL_CONFIG.try_read()?;
    let rom_manager = RomManager::new(Some(&global_config_guard.database_file))?;
    fs::create_dir_all(&global_config_guard.roms_directory)?;

    for path in paths {
        tracing::info!("Inspecting {} for known ROMs", path.display());

        if path.is_dir() {
            let walkdir = WalkDir::new(path);

            walkdir
                .into_iter()
                .par_bridge()
                .flatten_iter()
                .try_for_each(|entry| {
                    process_file(
                        symlink,
                        entry.path(),
                        global_config_guard.deref(),
                        &rom_manager,
                    )
                })
                .map_err(|e| e as Box<dyn std::error::Error>)?;
        } else {
            process_file(symlink, path, global_config_guard.deref(), &rom_manager)
                .map_err(|e| e as Box<dyn std::error::Error>)?;
        }
    }

    Ok(())
}

fn process_file(
    symlink: bool,
    path: impl AsRef<Path>,
    global_config: &GlobalConfig,
    database: &RomManager,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let path = path.as_ref();
    let database_transaction = database.rom_information.r_transaction()?;

    if path.is_dir() {
        return Ok(());
    }

    let mut file = File::open(path)?;

    // First attempt to open as a zip file
    if let Ok(mut zip_file) = ZipArchive::new(&mut file) {
        for file_index in 0..zip_file.len() {
            let mut zip_entry = zip_file.by_index(file_index)?;

            if zip_entry.is_file() {
                let hash = RomId::from_read(&mut zip_entry);
                drop(zip_entry);
                // We simply reopen it since seeking isn't supported
                let mut zip_entry = zip_file.by_index(file_index)?;

                if let Some(rom) = database_transaction.get().primary::<RomInfo>(hash)? {
                    let hash_string = hash.to_string();

                    tracing::info!(
                        "Identified ROM inside zip archive {} at {} as \"{:?}\" for the system {} with hash {}",
                        path.display(),
                        zip_entry.name(),
                        rom.name,
                        rom.system,
                        hash_string
                    );
                    let internal_store_path = global_config.roms_directory.join(hash_string);
                    let mut file = File::create(internal_store_path)?;

                    std::io::copy(&mut zip_entry, &mut file)?;
                } else {
                    tracing::warn!(
                        "Could not identify ROM inside zip archive {} at {} with hash {}",
                        path.display(),
                        zip_entry.name(),
                        hash
                    );
                }
            }
        }
    }

    let mut file = File::open(path)?;
    let hash = RomId::from_read(&mut file);

    if let Some(rom) = database_transaction.get().primary::<RomInfo>(hash)? {
        let hash_string = hash.to_string();

        tracing::info!(
            "Identified ROM at {} as \"{:?}\" for the system {} with hash {}",
            path.display(),
            rom.name,
            rom.system,
            hash_string
        );
        let internal_store_path = global_config.roms_directory.join(hash_string);
        let _ = fs::remove_file(&internal_store_path);

        #[cfg(unix)]
        if symlink {
            #[cfg(unix)]
            std::os::unix::fs::symlink(path, internal_store_path)?;

            #[cfg(windows)]
            std::os::windows::fs::symlink_file(path, internal_store_path)?;

            #[cfg(not(any(unix, windows)))]
            panic!("Unsupported platform for symlinking");
        } else {
            fs::copy(path, internal_store_path)?;
        }
    } else {
        tracing::warn!(
            "Could not identify ROM at {} with hash {}",
            path.display(),
            hash
        );
    }

    Ok(())
}
