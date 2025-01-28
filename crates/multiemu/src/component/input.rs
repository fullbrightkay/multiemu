use super::Component;
use crate::input::{manager::InputManager, EmulatedGamepadId, Input};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::Display,
    sync::Arc,
};

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct EmulatedGamepadTypeId(Cow<'static, str>);

impl EmulatedGamepadTypeId {
    pub const fn new(id: &'static str) -> Self {
        Self(Cow::Borrowed(id))
    }
}

impl AsRef<str> for EmulatedGamepadTypeId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for EmulatedGamepadTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub struct EmulatedGamepadMetadata {
    pub present_inputs: HashSet<Input>,
    pub default_bindings: HashMap<Input, Input>,
}

pub trait InputComponent: Component {
    /// Sets the input manager and what gamepad ids this thing obeys
    fn set_input_manager(
        &self,
        _input_manager: Arc<InputManager>,
        _gamepad_ids: &[EmulatedGamepadId],
    ) {
    }
}
