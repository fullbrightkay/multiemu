use super::Component;
use crate::input::{GamepadId, Input};
use std::{borrow::Cow, collections::HashSet, sync::Arc};

pub struct ControllerKind {
    pub name: Cow<'static, str>,
    pub inputs: HashSet<Input>,
}

pub trait InputComponent: Component {
    /// Feeds in input changes to the component
    fn feed_input(&self, gamepad_id: GamepadId, input: Input) {}
}
