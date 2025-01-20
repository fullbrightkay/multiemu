use super::{gamepad::GamepadInput, Input};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, sync::LazyLock};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Hotkey {
    OpenMenu,
}

pub static DEFAULT_HOTKEYS: LazyLock<IndexMap<BTreeSet<Input>, Hotkey>> = LazyLock::new(|| {
    [(
        [Input::Gamepad(GamepadInput::Mode)].into(),
        Hotkey::OpenMenu,
    )]
    .into()
});
