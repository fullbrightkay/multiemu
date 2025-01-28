use super::RomSpecification;
use crate::{
    config::{GraphicsSettings, GLOBAL_CONFIG},
    rom::{id::RomId, info::RomInfo, manager::RomManager, system::GameSystem},
    runtime::{
        launch::Runtime,
        platform::{PlatformRuntime, SoftwareRenderingRuntime},
    },
};
use std::{
    error::Error,
    fs::{create_dir_all, File},
    sync::Arc,
};

pub fn rom_run(
    roms: Vec<RomSpecification>,
    forced_system: Option<GameSystem>,
) -> Result<(), Box<dyn Error>> {
    let global_config_guard = GLOBAL_CONFIG.read().unwrap();
    let rom_manager = RomManager::new(Some(&global_config_guard.database_file))?;

    create_dir_all(&global_config_guard.roms_directory)?;

    let mut user_specified_roms = Vec::new();

    let transaction = rom_manager.rom_information.rw_transaction()?;

    for rom in roms {
        match rom {
            RomSpecification::Id(rom_id) => user_specified_roms.push(rom_id),
            RomSpecification::Path(rom_path) => {
                let Some(system) = GameSystem::guess(&rom_path) else {
                    return Err(format!("{} is not a valid rom", rom_path.display()).into());
                };

                let mut rom_file = File::open(&rom_path)?;
                let rom_id = RomId::from_read(&mut rom_file);

                let rom_info = RomInfo {
                    name: Some(rom_path.to_string_lossy().to_string()),
                    id: rom_id,
                    system,
                    region: None,
                };

                user_specified_roms.push(rom_id);
                if let Err(e) = transaction.insert(rom_info) {
                    if let native_db::db_type::Error::DuplicateKey { key_name: _ } = e {
                        tracing::warn!(
                            "Skipping inserting duplicate information into the database"
                        );
                    } else {
                        return Err(e.into());
                    }
                }

                rom_manager.rom_paths.insert(rom_id, rom_path);
            }
        }
    }

    transaction.commit()?;

    let graphics_setting = global_config_guard.graphics_setting;
    drop(global_config_guard);
    let rom_manager = Arc::new(rom_manager);

    match graphics_setting {
        GraphicsSettings::Software => {
            PlatformRuntime::<SoftwareRenderingRuntime>::launch_game(
                user_specified_roms,
                forced_system,
                rom_manager,
            );
        }
        #[cfg(graphics_vulkan)]
        GraphicsSettings::Vulkan => {
            use crate::runtime::platform::desktop::renderer::vulkan::VulkanRenderingRuntime;

            PlatformRuntime::<VulkanRenderingRuntime>::launch_game(
                user_specified_roms,
                forced_system,
                rom_manager,
            );
        }
    }

    Ok(())
}
