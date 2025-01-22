use crate::component::input::EmulatedGamepadMetadata;

use super::{GamepadId, GamepadPort, Input, InputState};
use dashmap::DashMap;
use std::{
    borrow::Cow, collections::{HashMap, HashSet}, sync::Arc
};

#[derive(Debug, PartialEq)]
pub struct GamepadState(pub HashMap<Input, InputState>);

impl GamepadState {
    pub fn diff<'a>(
        &'a self,
        other: &'a Self,
    ) -> impl Iterator<Item = (Input, InputState)> + use<'a> {
        self.0
            .iter()
            .filter_map(move |(input, state)| match other.0.get(input) {
                Some(other_state) if state != other_state => Some((*input, *state)),
                _ => None,
            })
    }
}

#[derive(Debug)]
struct PortInfo {
    state: HashMap<Input, InputState>,
    metadata: Cow<'static, EmulatedGamepadMetadata>,
}

#[derive(Debug, Default)]
pub struct InputManager {
    ports: DashMap<GamepadPort, PortInfo>,
    real_to_emulated_mapping: DashMap<GamepadId, GamepadPort>,
}

impl InputManager {
    pub fn get_input(&self, port: GamepadPort, input: Input) -> InputState {
        self.ports
            .get(&port)
            .unwrap()
            .state
            .get(&input)
            .cloned()
            .unwrap_or_default()
    }

    pub fn set_input(&self, id: GamepadId, input: Input, state: InputState) {
        if let Some(mut port_info) = self
            .real_to_emulated_mapping
            .get(&id)
            .and_then(|entry| self.ports.get_mut(entry.key()))
        {
            if port_info.metadata.inputs.contains(&input) {
                port_info.state.insert(input, state);
            }
        }
    }

    pub fn get_full_state(&self, port: GamepadPort) -> GamepadState {
        GamepadState(self.ports.get(&port).unwrap().state.clone())
    }

    /// Components do not call this!!!!!!!!!!1
    pub fn register_gamepad_port(&self, port: GamepadPort, metadata: Cow<'static, EmulatedGamepadMetadata>) {
        self.ports.insert(
            port,
            PortInfo {
                state: HashMap::default(),
                metadata,
            },
        );
    }
}
