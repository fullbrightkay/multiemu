use crate::{
    component::input::{EmulatedGamepadMetadata, EmulatedGamepadTypeId},
    config::GLOBAL_CONFIG,
    rom::system::GameSystem,
};

use super::{EmulatedGamepadId, GamepadId, Input, InputState};
use dashmap::DashMap;
use std::collections::HashMap;

#[derive(Debug)]
/// Stores what each gamepad is cached to be at right now
struct EmulatedGamepadState {
    kind: EmulatedGamepadTypeId,
    state: HashMap<Input, InputState>,
}

#[derive(Debug, Default)]
pub struct InputManager {
    pub gamepad_types: HashMap<EmulatedGamepadTypeId, EmulatedGamepadMetadata>,
    emulated_gamepads: DashMap<EmulatedGamepadId, EmulatedGamepadState>,
    real_to_emulated_gamepad_mappings: DashMap<GamepadId, EmulatedGamepadId>,
}

impl InputManager {
    pub fn get_input(&self, port: EmulatedGamepadId, input: Input) -> InputState {
        self.emulated_gamepads
            .get(&port)
            .unwrap()
            .state
            .get(&input)
            .cloned()
            .unwrap_or_default()
    }

    pub fn insert_input(&self, system: GameSystem, id: GamepadId, input: Input, state: InputState) {
        let global_config = GLOBAL_CONFIG.read().unwrap();

        // Find out which real controller is hooked up to which emulated one
        if let Some(mut emulated_gamepad_state) = self
            .real_to_emulated_gamepad_mappings
            .get(&id)
            .and_then(|entry| self.emulated_gamepads.get_mut(entry.key()))
        {
            let metadata = self
                .gamepad_types
                .get(&emulated_gamepad_state.kind)
                .unwrap();

            // Translate the input according to the global config
            let Some(translated_input) = global_config
                .gamepad_configs
                .get(&system)
                .and_then(|emulated_gamepad_infos| {
                    emulated_gamepad_infos.get(&emulated_gamepad_state.kind)
                })
                .and_then(|gamepad_specific_mappings| gamepad_specific_mappings.get(&input))
            else {
                tracing::warn!("Unbound input {:?}", input);
                return;
            };

            if metadata.present_inputs.contains(translated_input) {
                emulated_gamepad_state
                    .state
                    .insert(*translated_input, state);
            } else {
                tracing::warn!("We have a bound from {:?} to {:?}, but emulated gamepad doesn't support this input", input, translated_input);
            }
        }
    }

    pub fn set_real_to_emulated_mapping(&self, gamepad_id: GamepadId, index: EmulatedGamepadId) {
        self.real_to_emulated_gamepad_mappings
            .insert(gamepad_id, index);
    }

    pub fn register_emulated_gamepad(
        &mut self,
        port: EmulatedGamepadId,
        kind: EmulatedGamepadTypeId,
    ) {
        self.emulated_gamepads.insert(
            port,
            EmulatedGamepadState {
                state: HashMap::default(),
                kind,
            },
        );
    }

    pub fn register_emulated_gamepad_type(
        &mut self,
        kind: EmulatedGamepadTypeId,
        metadata: EmulatedGamepadMetadata,
    ) {
        self.gamepad_types.insert(kind, metadata);
    }
}
