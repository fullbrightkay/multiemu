use crate::{
    gui::menu::MenuState,
    rom::{id::RomId, manager::RomManager, system::GameSystem},
    runtime::{launch::Runtime, rendering_backend::RenderingBackendState},
};
use ::winit::{event_loop::EventLoop, window::Window};
use std::sync::Arc;
use winit::{MachineContext, WindowingContext};

pub mod renderer;
mod winit;

pub struct PlatformRuntime<RS: RenderingBackendState> {
    menu: MenuState,
    windowing_context: Option<WindowingContext<RS>>,
    machine_context: Option<MachineContext>,
    rom_manager: Arc<RomManager>,
}

impl<RS: RenderingBackendState<DisplayApiHandle = Arc<Window>>> Runtime for PlatformRuntime<RS> {
    fn launch_gui(rom_manager: Arc<RomManager>) {
        let mut me = Self {
            menu: MenuState::default(),
            windowing_context: None,
            machine_context: None,
            rom_manager,
        };

        let event_loop = EventLoop::new().unwrap();
        event_loop.run_app(&mut me).unwrap();
    }

    fn launch_game(
        user_specified_roms: Vec<RomId>,
        forced_system: Option<GameSystem>,
        rom_manager: Arc<RomManager>,
    ) {
        let mut me = Self {
            menu: MenuState::default(),
            windowing_context: None,
            machine_context: Some(MachineContext::Pending {
                user_specified_roms,
                forced_system,
            }),
            rom_manager,
        };

        let event_loop = EventLoop::new().unwrap();
        event_loop.run_app(&mut me).unwrap();
    }
}
