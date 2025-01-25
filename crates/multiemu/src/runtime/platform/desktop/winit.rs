use super::PlatformRuntime;
use crate::{
    config::GLOBAL_CONFIG,
    definitions::chip8::chip8_machine,
    gui::menu::UiOutput,
    input::{GamepadId, InputState},
    machine::Machine,
    rom::{
        id::RomId,
        info::RomInfo,
        system::{GameSystem, OtherSystem},
    },
    runtime::rendering_backend::RenderingBackendState,
};
use indexmap::IndexMap;
use std::{fs::File, sync::Arc};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

// FIXME: Duplicated hack code is present here

const KEYBOARD_GAMEPAD_ID: GamepadId = 0;

pub enum MachineContext {
    /// Machine is waiting for graphics context to be ready
    Pending {
        user_specified_roms: Vec<RomId>,
        forced_system: Option<GameSystem>,
    },
    /// Machine is currently running
    Running(Machine),
}

pub struct WindowingContext<RS: RenderingBackendState> {
    window: Arc<Window>,
    egui_winit_context: egui_winit::State,
    runtime_state: RS,
}

impl<RS: RenderingBackendState<DisplayApiHandle = Arc<Window>>> ApplicationHandler
    for PlatformRuntime<RS>
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // HACK: This will cause frequent crashes on mobile platforms
        if self.windowing_context.is_some() {
            panic!("Window already created");
        }

        let window = setup_window(event_loop);
        let egui_winit_context = egui_winit::State::new(
            self.menu.egui_context.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        let mut runtime_state = RS::new(window.clone());

        match self.machine_context.take() {
            Some(MachineContext::Pending {
                user_specified_roms,
                forced_system,
            }) => {
                let system = forced_system
                    .or_else(|| {
                        self.rom_manager
                            .rom_information
                            .r_transaction()
                            .unwrap()
                            .get()
                            .primary::<RomInfo>(user_specified_roms[0])
                            .unwrap()
                            .map(|info| info.system)
                    })
                    .expect("Could not figure out system");

                let machine =
                    Machine::from_system(user_specified_roms, self.rom_manager.clone(), system);
                runtime_state.initialize_machine(&machine);

                // HACK: Wire the keyboard to port 0
                machine
                    .input_manager
                    .set_real_to_emulated_mapping(KEYBOARD_GAMEPAD_ID, 0);

                // Make sure the system being run has a default mapping
                let mut global_config_guard = GLOBAL_CONFIG.write().unwrap();

                for (gamepad_type, metadata) in machine.input_manager.gamepad_types.iter() {
                    global_config_guard
                        .gamepad_configs
                        .entry(machine.system)
                        .or_default()
                        .entry(gamepad_type.clone())
                        .or_insert_with(|| IndexMap::from_iter(metadata.default_bindings.clone()));
                }

                self.menu.active = false;
                self.machine_context = Some(MachineContext::Running(machine));
            }
            Some(MachineContext::Running(_)) => {
                panic!("Window resume while machine is running");
            }
            None => {}
        }

        self.windowing_context = Some(WindowingContext {
            window,
            egui_winit_context,
            runtime_state,
        })
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // This helps the user not stare at a black screen
        if !matches!(self.machine_context, Some(MachineContext::Running { .. })) {
            self.menu.active = true;
        }

        let window_context = self
            .windowing_context
            .as_mut()
            .expect("Window was not initialized");

        // Ensure a resize happens before drawing occurs
        if matches!(event, WindowEvent::Resized(_)) {
            window_context.runtime_state.surface_resized();
            return;
        }

        if self.menu.active {
            let egui_winit::EventResponse { consumed, repaint } = window_context
                .egui_winit_context
                .on_window_event(&window_context.window, &event);

            if consumed {
                return;
            }

            if repaint {
                window_context.window.request_redraw();
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Window close requested");

                // Save the config on exit
                GLOBAL_CONFIG
                    .read()
                    .unwrap()
                    .save()
                    .expect("Failed to save config");

                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic,
            } => {
                if !is_synthetic {
                    return;
                }

                if let PhysicalKey::Code(key_code) = event.physical_key {
                    let state = event.state.is_pressed();

                    if !self.menu.active {
                        if let Some(MachineContext::Running(machine)) = &mut self.machine_context {
                            machine.input_manager.insert_input(
                                machine.system,
                                KEYBOARD_GAMEPAD_ID,
                                key_code.try_into().unwrap(),
                                InputState::Digital(state),
                            );
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if self.menu.active {
                    // We put the ui output like this so multipassing egui gui building works
                    let mut ui_output = None;
                    let full_output = self.menu.egui_context.clone().run(
                        window_context
                            .egui_winit_context
                            .take_egui_input(&window_context.window),
                        |context| {
                            ui_output = ui_output.take().or(self.menu.run_menu(context));
                        },
                    );

                    match ui_output {
                        None => {}
                        Some(UiOutput::OpenGame { path }) => {
                            tracing::info!("Opening rom at {}", path.display());

                            let mut rom_file = File::open(&path).unwrap();
                            let rom_id = RomId::from_read(&mut rom_file);

                            // Check if we know about the game from the manager
                            if let Some(system) = self
                                .rom_manager
                                .rom_information
                                .r_transaction()
                                .unwrap()
                                .get()
                                .primary::<RomInfo>(rom_id)
                                .unwrap()
                                .map(|info| info.system)
                                .or_else(|| GameSystem::guess(&path))
                            {
                                self.rom_manager.rom_paths.insert(rom_id, path.clone());

                                let machine = match system {
                                    GameSystem::Other(OtherSystem::Chip8) => {
                                        chip8_machine(vec![rom_id], self.rom_manager.clone())
                                    }
                                    _ => {
                                        unimplemented!()
                                    }
                                };

                                // HACK: Wire the keyboard to port 0
                                machine
                                    .input_manager
                                    .set_real_to_emulated_mapping(KEYBOARD_GAMEPAD_ID, 0);

                                // Make sure the system being run has a default mapping
                                let mut global_config_guard = GLOBAL_CONFIG.write().unwrap();

                                for (gamepad_type, metadata) in
                                    machine.input_manager.gamepad_types.iter()
                                {
                                    global_config_guard
                                        .gamepad_configs
                                        .entry(machine.system)
                                        .or_default()
                                        .entry(gamepad_type.clone())
                                        .or_insert_with(|| {
                                            IndexMap::from_iter(metadata.default_bindings.clone())
                                        });
                                }

                                // Initialize graphics components
                                window_context.runtime_state.initialize_machine(&machine);
                                self.machine_context = Some(MachineContext::Running(machine));
                                // Close the menu
                                self.menu.active = false;
                            } else {
                                tracing::error!("Could not identify rom at {}", path.display());
                            }
                        }
                    }

                    window_context
                        .runtime_state
                        .redraw_menu(&self.menu.egui_context, full_output);
                } else if let Some(MachineContext::Running(machine)) = &mut self.machine_context {
                    machine.run();
                    window_context.runtime_state.redraw(machine);
                    window_context.window.request_redraw();
                } else {
                    tracing::warn!("Machine not running when redraw requested");
                }
            }
            _ => {}
        }
    }
}

fn setup_window(event_loop: &ActiveEventLoop) -> Arc<Window> {
    let window_attributes = Window::default_attributes()
        .with_title("MultiEMU")
        .with_resizable(true)
        .with_transparent(false);
    Arc::new(event_loop.create_window(window_attributes).unwrap())
}
