use super::{audio::Chip8Audio, display::Chip8Display, timer::Chip8Timer, Chip8Kind};
use crate::{
    component::{
        input::{EmulatedGamepadMetadata, InputComponent},
        schedulable::SchedulableComponent,
        Component, ComponentId, FromConfig,
    },
    input::{
        gamepad::GamepadInput,
        keyboard::KeyboardInput,
        manager::{GamepadState, InputManager},
        GamepadPort, Input,
    },
    machine::ComponentBuilder,
    memory::MemoryTranslationTable,
};
use arrayvec::ArrayVec;
use decode::decode_instruction;
use input::Chip8KeyCode;
use instruction::Register;
use num::rational::Ratio;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::HashSet,
    sync::{Arc, Mutex, OnceLock},
};

mod decode;
mod input;
mod instruction;
mod interpret;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum ExecutionState {
    Normal,
    AwaitingKeyPress {
        register: Register,
    },
    // KeyQuery does not return on key press but on key release, contrary to some documentation
    AwaitingKeyRelease {
        register: Register,
        keys: Vec<Chip8KeyCode>,
    },
}

// This is extremely complex because the chip8 cpu has a lot of non cpu machinery

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Chip8ProcessorRegisters {
    work_registers: [u8; 16],
    index: u16,
    program: u16,
}

impl Default for Chip8ProcessorRegisters {
    fn default() -> Self {
        Self {
            work_registers: [0; 16],
            index: 0,
            program: 0x200,
        }
    }
}

#[derive(Debug)]
pub struct Chip8ProcessorConfig {
    pub frequency: Ratio<u64>,
    pub kind: Chip8Kind,
    pub display: ComponentId,
    pub audio: ComponentId,
    pub timer: ComponentId,
}

pub struct ProcessorState {
    stack: ArrayVec<u16, 16>,
    registers: Chip8ProcessorRegisters,
    execution_state: ExecutionState,
}

/// FIXME: This complexity is insane
pub struct Chip8Processor {
    /// Configuration this processor was created with
    config: Chip8ProcessorConfig,
    /// chip8 display component
    display: Arc<Chip8Display>,
    /// chip8 audio component
    audio: Arc<Chip8Audio>,
    /// chip8 timer component
    timer: Arc<Chip8Timer>,
    /// parts of the cpu that actually change over execution
    state: Mutex<ProcessorState>,
    /// memory translation table
    memory_translation_table: OnceLock<Arc<MemoryTranslationTable>>,
    /// input manager + port for our keypad
    input_manager: OnceLock<(Arc<InputManager>, GamepadPort)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chip8ProcessorSnapshot {
    registers: Chip8ProcessorRegisters,
    stack: ArrayVec<u16, 16>,
    execution_state: ExecutionState,
}

impl Component for Chip8Processor {
    fn reset(&self) {
        let mut state = self.state.lock().unwrap();

        state.stack.clear();
        state.registers = Chip8ProcessorRegisters::default();
        state.execution_state = ExecutionState::Normal;
    }

    fn save_snapshot(&self) -> rmpv::Value {
        let state = self.state.lock().unwrap();

        rmpv::ext::to_value(&Chip8ProcessorSnapshot {
            registers: state.registers.clone(),
            stack: state.stack.clone(),
            execution_state: state.execution_state.clone(),
        })
        .unwrap()
    }

    fn load_snapshot(&self, state: rmpv::Value) {
        let snapshot: Chip8ProcessorSnapshot = rmpv::ext::from_value(state).unwrap();
        let mut state = self.state.lock().unwrap();

        state.registers = snapshot.registers;
        state.stack = snapshot.stack;
        state.execution_state = snapshot.execution_state;
    }

    fn set_memory_translation_table(&self, memory_translation_table: Arc<MemoryTranslationTable>) {
        let _ = self.memory_translation_table.set(memory_translation_table);
    }
}

impl FromConfig for Chip8Processor {
    type Config = Chip8ProcessorConfig;

    fn from_config(component_builder: &mut ComponentBuilder<Self>, config: Self::Config)
    where
        Self: Sized,
    {
        let frequency = config.frequency;

        component_builder
            .set_component(Self {
                state: Mutex::new(ProcessorState {
                    stack: ArrayVec::default(),
                    registers: Chip8ProcessorRegisters::default(),
                    execution_state: ExecutionState::Normal,
                }),
                display: component_builder
                    .machine()
                    .get_component(config.display)
                    .expect("Display component not found"),
                audio: component_builder
                    .machine()
                    .get_component(config.audio)
                    .expect("Audio component not found"),
                timer: component_builder
                    .machine()
                    .get_component(config.timer)
                    .expect("Timer component not found"),
                config,
                memory_translation_table: OnceLock::default(),
                input_manager: OnceLock::default(),
            })
            .set_schedulable(frequency, [], [])
            .set_input([Cow::Owned(EmulatedGamepadMetadata {
                name: "Chip8 Keypad".into(),
                inputs: HashSet::from_iter([
                    Input::Keyboard(KeyboardInput::Numpad1),
                    Input::Keyboard(KeyboardInput::Numpad2),
                    Input::Keyboard(KeyboardInput::Numpad3),
                    Input::Keyboard(KeyboardInput::KeyC),
                    Input::Keyboard(KeyboardInput::Numpad4),
                    Input::Keyboard(KeyboardInput::Numpad5),
                    Input::Keyboard(KeyboardInput::Numpad6),
                    Input::Keyboard(KeyboardInput::KeyD),
                    Input::Keyboard(KeyboardInput::Numpad7),
                    Input::Keyboard(KeyboardInput::Numpad8),
                    Input::Keyboard(KeyboardInput::Numpad9),
                    Input::Keyboard(KeyboardInput::KeyE),
                    Input::Keyboard(KeyboardInput::KeyA),
                    Input::Keyboard(KeyboardInput::Numpad0),
                    Input::Keyboard(KeyboardInput::KeyB),
                    Input::Keyboard(KeyboardInput::KeyF),
                ]),
            })]);
    }
}

impl InputComponent for Chip8Processor {
    fn set_input_manager(&self, input_manager: Arc<InputManager>, gamepad_ports: &[GamepadPort]) {
        self.input_manager
            .set((input_manager, gamepad_ports[0]))
            .expect("Input manager set multiple times");
    }
}

impl SchedulableComponent for Chip8Processor {
    fn run(&self, period: u64) {
        let mut state = self.state.lock().unwrap();

        for _ in 0..period {
            match &state.execution_state {
                ExecutionState::Normal => {
                    let mut instruction = [0; 2];
                    self.memory_translation_table
                        .get()
                        .unwrap()
                        .read(state.registers.program as usize, &mut instruction);
                    let decompiled_instruction = decode_instruction(instruction).unwrap();
                    state.registers.program = state.registers.program.wrapping_add(2);

                    tracing::trace!(
                        "Decoded instruction {:?} from {:#04x}",
                        instruction,
                        state.registers.program
                    );

                    self.interpret_instruction(&mut state, decompiled_instruction);
                }
                ExecutionState::AwaitingKeyPress { register } => {
                    // FIXME: A allocation every cycle isn't a good idea
                    let mut pressed = Vec::new();
                    let (input_manager, gamepad_port) = self.input_manager.get().unwrap();

                    // Go through every chip8 key
                    for key in 0x0..0xf {
                        let keycode = Chip8KeyCode(key);

                        if input_manager
                            .get_input(*gamepad_port, keycode.try_into().unwrap())
                            .as_digital()
                        {
                            pressed.push(keycode);
                        }
                    }

                    if !pressed.is_empty() {
                        state.execution_state = ExecutionState::AwaitingKeyRelease {
                            register: *register,
                            keys: pressed,
                        }
                    }
                }
                ExecutionState::AwaitingKeyRelease { register, keys } => {
                    let (input_manager, gamepad_port) = self.input_manager.get().unwrap();

                    for key_code in keys {
                        if !input_manager
                            .get_input(*gamepad_port, (*key_code).try_into().unwrap())
                            .as_digital()
                        {
                            let register = *register;
                            state.registers.work_registers[register as usize] = key_code.0;
                            state.execution_state = ExecutionState::Normal;
                            break;
                        }
                    }
                }
            }
        }
    }
}
