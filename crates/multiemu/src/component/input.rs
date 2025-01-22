use super::Component;
use crate::input::{manager::InputManager, GamepadId, GamepadPort, Input};
use std::{
    borrow::Cow,
    collections::HashSet,
    sync::{mpsc::Receiver, Arc},
};

#[derive(Debug, Clone)]
pub struct EmulatedGamepadMetadata {
    pub name: Cow<'static, str>,
    pub inputs: HashSet<Input>,
}

pub trait InputComponent: Component {
    /// Sets the input manager and what gamepad ids this thing obeys
    fn set_input_manager(&self, input_manager: Arc<InputManager>, gamepad_ports: &[GamepadPort]) {}
}
