use crate::component::ComponentId;
use dashmap::DashMap;
use gamepad::GamepadInput;
use keyboard::KeyboardInput;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use strum::IntoEnumIterator;

pub mod gamepad;
pub mod hotkey;
pub mod keyboard;
pub mod manager;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Input {
    Gamepad(GamepadInput),
    Keyboard(KeyboardInput),
}

impl Input {
    pub fn iter() -> impl Iterator<Item = Self> {
        GamepadInput::iter()
            .map(Input::Gamepad)
            .chain(KeyboardInput::iter().map(Input::Keyboard))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InputState {
    /// 0 or 1
    Digital(bool),
    /// Clamped from 0.0 to 1.0
    Analog(f32),
}

impl Default for InputState {
    fn default() -> Self {
        Self::Digital(false)
    }
}

impl InputState {
    const PRESSED: Self = Self::Digital(true);
    const RELEASED: Self = Self::Digital(false);

    pub fn as_digital(&self) -> bool {
        match self {
            InputState::Digital(value) => *value,
            InputState::Analog(value) => *value >= 0.5,
        }
    }

    pub fn as_analog(&self) -> f32 {
        match self {
            InputState::Digital(value) => {
                if *value {
                    1.0
                } else {
                    0.0
                }
            }
            InputState::Analog(value) => *value,
        }
    }
}

/// Id of a gamepad currently recognized by the emulator
pub type GamepadId = u8;

/// Id for a emulated gamepad on the emulator
pub type GamepadPort = u8;
