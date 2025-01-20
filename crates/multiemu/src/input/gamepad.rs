use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(
    Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter,
)]
pub enum GamepadInput {
    // These ones are almost always digital

    // Face pad buttons
    /// Aliases:
    ///
    /// Microsoft: Y
    ///
    /// Nintendo: X
    ///
    /// Sony: △
    FPadUp,
    /// Aliases:
    ///
    /// Microsoft: A
    ///
    /// Nintendo: B
    ///
    /// Sony: ×
    FPadDown,
    /// Aliases:
    ///
    /// Microsoft: X
    ///
    /// Nintendo: Y
    ///
    /// Sony: □
    FPadLeft,
    /// Aliases:
    ///
    /// Microsoft: B
    ///
    /// Nintendo: B
    ///
    /// Sony: ○
    FPadRight,
    // N64 specific C pad buttons, essentially a second 2 pad
    CPadUp,
    CPadDown,
    CPadLeft,
    CPadRight,
    /// Aliases:
    ///
    /// Nintendo: +
    ///
    /// Microsoft: Menu
    ///
    /// Sony: Options
    Select,
    /// Aliases:
    ///
    /// Nintendo: -
    ///
    /// Microsoft: View
    ///
    /// Sony: Share/Create
    Start,
    /// Usually called the (brandname) button
    ///
    /// I have seen analog versions of this button on unusual controllers
    Mode,
    // Pushing down on the joystick gives these two
    LeftThumb,
    RightThumb,
    // Standard digital pad
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    LeftTrigger,
    RightTrigger,
    /// Gamecube specific
    ZTrigger,
    // These ones are usually analog
    LeftSecondaryTrigger,
    RightSecondaryTrigger,
    /// Standard analog sticks
    LeftStickUp,
    LeftStickDown,
    LeftStickLeft,
    LeftStickRight,
    RightStickUp,
    RightStickDown,
    RightStickLeft,
    RightStickRight,
}
