use crate::{
    config::GlobalConfig,
    rom::{GameSystem, RomId, RomManager},
    runtime::{
        desktop::display::vulkan::VulkanRendering, launch_gui, InitialGuiState, SoftwareRendering,
    },
};
use std::{
    fs::create_dir_all,
    sync::{Arc, Mutex},
};

pub fn run(
    user_specified_roms: Vec<RomId>,
    forced_game_system: Option<GameSystem>,
    global_config: GlobalConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rom_manager = RomManager::default();

    create_dir_all(&global_config.roms_directory)?;

    // Load rom database
    rom_manager.load_database(&global_config.database_file)?;
    // Load on disk roms
    rom_manager.load_roms(&global_config.roms_directory)?;

    let hardware_acceleration = global_config.hardware_acceleration;
    let global_config = Arc::new(Mutex::new(global_config));

    if hardware_acceleration {
        launch_gui::<VulkanRendering>(
            rom_manager,
            InitialGuiState::OpenGame {
                user_specified_roms,
                forced_game_system,
            },
            global_config,
        );
    } else {
        launch_gui::<SoftwareRendering>(
            rom_manager,
            InitialGuiState::OpenGame {
                user_specified_roms,
                forced_game_system,
            },
            global_config,
        );
    }

    Ok(())
}
