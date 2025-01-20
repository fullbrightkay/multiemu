use config::GLOBAL_CONFIG;
use rom::manager::RomManager;
use runtime::{
    launch::Runtime,
    platform::{PlatformRuntime, SoftwareRenderingRuntime},
};
use std::sync::Arc;

#[cfg(desktop)]
pub mod cli;
pub mod component;
pub mod config;
pub mod definitions;
pub mod gui;
pub mod input;
pub mod machine;
pub mod memory;
pub mod processor;
pub mod rom;
pub mod runtime;
pub mod scheduler;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("MultiEMU v{}", env!("CARGO_PKG_VERSION"));

    #[cfg(desktop)]
    {
        use clap::Parser;
        use cli::handle_cli;
        use cli::Cli;

        let cli = Cli::parse();

        if let Some(action) = cli.action {
            handle_cli(action).unwrap();
            return;
        }
    }

    let global_config_guard = GLOBAL_CONFIG.try_read().unwrap();
    let rom_manager = Arc::new(RomManager::new(Some(&global_config_guard.database_file)).unwrap());
    let hardware_acceleration = global_config_guard.hardware_acceleration;
    drop(global_config_guard);

    if hardware_acceleration {
        #[cfg(desktop)]
        {
            use runtime::platform::desktop::renderer::vulkan::VulkanRenderingRuntime;

            PlatformRuntime::<VulkanRenderingRuntime>::launch_gui(rom_manager);
        }
    } else {
        PlatformRuntime::<SoftwareRenderingRuntime>::launch_gui(rom_manager);
    }

    GLOBAL_CONFIG
        .read()
        .unwrap()
        .save()
        .expect("Failed to save config");
}
