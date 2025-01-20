use crate::{
    config::GLOBAL_CONFIG,
    rom::{id::RomId, info::RomInfo, manager::RomManager, system::GameSystem},
    runtime::{
        launch::Runtime,
        platform::{PlatformRuntime, SoftwareRenderingRuntime},
    },
};
use std::{
    fs::{create_dir_all, File},
    path::PathBuf,
    sync::Arc,
};

pub fn run(
    roms: Vec<PathBuf>,
    forced_game_system: Option<GameSystem>,
) -> Result<(), Box<dyn std::error::Error>> {
    let global_config_guard = GLOBAL_CONFIG.read().unwrap();
    let rom_manager = RomManager::new(Some(&global_config_guard.database_file))?;

    create_dir_all(&global_config_guard.roms_directory)?;

    let mut user_specified_roms = Vec::new();

    let transaction = rom_manager.rom_information.rw_transaction()?;

    for rom in roms {
        let Some(system) = GameSystem::guess(&rom) else {
            return Err(format!("{} is not a valid rom", rom.display()).into());
        };

        let mut rom_file = File::open(&rom)?;
        let rom_id = RomId::from_read(&mut rom_file);

        let rom_info = RomInfo {
            name: Some(rom.to_string_lossy().to_string()),
            hash: rom_id,
            system,
            region: None,
        };

        user_specified_roms.push(rom_id);
        if let Err(e) = transaction.insert(rom_info) {
            if let native_db::db_type::Error::DuplicateKey { key_name: _ } = e {
                tracing::warn!("Skipping inserting duplicate information into the database");
            } else {
                return Err(e.into());
            }
        }

        rom_manager.rom_paths.insert(rom_id, rom);
    }

    transaction.commit()?;

    let hardware_acceleration = global_config_guard.hardware_acceleration;
    drop(global_config_guard);
    let rom_manager = Arc::new(rom_manager);

    if hardware_acceleration {
        #[cfg(desktop)]
        {
            use crate::runtime::platform::desktop::renderer::vulkan::VulkanRenderingRuntime;

            PlatformRuntime::<VulkanRenderingRuntime>::launch_game(
                user_specified_roms,
                forced_game_system,
                rom_manager,
            );
        }
    } else {
        PlatformRuntime::<SoftwareRenderingRuntime>::launch_game(
            user_specified_roms,
            forced_game_system,
            rom_manager,
        );
    }
    Ok(())
}
