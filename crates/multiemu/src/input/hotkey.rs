use super::{gamepad::GamepadInput, keyboard::KeyboardInput, Input};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, sync::LazyLock};
use strum::EnumIter;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum Hotkey {
    ToggleMenu,
    FastForward,
    LoadSnapshot,
    SaveSnapshot,
}

pub static DEFAULT_HOTKEYS: LazyLock<IndexMap<BTreeSet<Input>, Hotkey>> = LazyLock::new(|| {
    [
        (
            [
                Input::Gamepad(GamepadInput::Mode),
                Input::Gamepad(GamepadInput::Start),
            ]
            .into(),
            Hotkey::ToggleMenu,
        ),
        (
            [Input::Keyboard(KeyboardInput::F1)].into(),
            Hotkey::ToggleMenu,
        ),
        (
            [
                Input::Gamepad(GamepadInput::Mode),
                Input::Gamepad(GamepadInput::Select),
            ]
            .into(),
            Hotkey::FastForward,
        ),
        (
            [Input::Keyboard(KeyboardInput::F2)].into(),
            Hotkey::FastForward,
        ),
        (
            [
                Input::Gamepad(GamepadInput::Mode),
                Input::Gamepad(GamepadInput::FPadUp),
            ]
            .into(),
            Hotkey::SaveSnapshot,
        ),
        (
            [Input::Keyboard(KeyboardInput::F3)].into(),
            Hotkey::SaveSnapshot,
        ),
        (
            [
                Input::Gamepad(GamepadInput::Mode),
                Input::Gamepad(GamepadInput::FPadLeft),
            ]
            .into(),
            Hotkey::LoadSnapshot,
        ),
        (
            [Input::Keyboard(KeyboardInput::F4)].into(),
            Hotkey::LoadSnapshot,
        ),
    ]
    .into()
});
