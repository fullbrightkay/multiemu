//! A multisystem hardware emulator

use config::{GraphicsSettings, GLOBAL_CONFIG};
use rom::manager::RomManager;
use runtime::{
    launch::Runtime,
    platform::{PlatformRuntime, SoftwareRenderingRuntime},
};
use std::sync::Arc;

// Cli tools are designed only to operate on desktop
#[cfg(platform_desktop)]
mod cli;
mod component;
mod config;
mod definitions;
mod gui;
mod input;
mod machine;
mod memory;
mod processor;
mod rom;
mod runtime;
mod scheduler;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("MultiEMU v{}", env!("CARGO_PKG_VERSION"));

    #[cfg(platform_desktop)]
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
    let graphics_setting = global_config_guard.graphics_setting;
    drop(global_config_guard);

    match graphics_setting {
        GraphicsSettings::Software => {
            PlatformRuntime::<SoftwareRenderingRuntime>::launch_gui(rom_manager);
        }
        #[cfg(graphics_vulkan)]
        GraphicsSettings::Vulkan => {
            use runtime::platform::desktop::renderer::vulkan::VulkanRenderingRuntime;

            PlatformRuntime::<VulkanRenderingRuntime>::launch_gui(rom_manager);
        }
    }
}
