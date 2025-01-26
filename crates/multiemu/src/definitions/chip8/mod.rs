use super::misc::memory::standard::{
    StandardMemory, StandardMemoryConfig, StandardMemoryInitialContents,
};
use crate::{
    machine::Machine, memory::AddressSpaceId, rom::{
        id::RomId,
        manager::RomManager,
        system::{GameSystem, OtherSystem},
    }
};
use audio::Chip8Audio;
use display::{Chip8Display, Chip8DisplayConfig};
use num::rational::Ratio;
use processor::{Chip8Processor, Chip8ProcessorConfig};
use std::{borrow::Cow, sync::Arc};
use timer::Chip8Timer;

pub mod audio;
pub mod display;
pub mod processor;
pub mod timer;

pub const CHIP8_ADDRESS_SPACE_ID: AddressSpaceId = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Chip8Kind {
    Chip8,
    Chip8x,
    Chip48,
    SuperChip8,
    XoChip,
}

#[rustfmt::skip]
const CHIP8_FONT: [[u8; 5]; 16] = [
    [
        0b11110000,
        0b10010000,
        0b10010000,
        0b10010000,
        0b11110000,
    ],
    [
        0b00100000,
        0b01100000,
        0b00100000,
        0b00100000,
        0b01110000,
    ],
    [
        0b11110000,
        0b00010000,
        0b11110000,
        0b10000000,
        0b11110000,
    ],
    [
        0b11100000,
        0b00100000,
        0b11100000,
        0b00100000,
        0b11100000,
    ],
    [
        0b10010000,
        0b10010000,
        0b11110000,
        0b00010000,
        0b00010000,
    ],
    [
        0b11110000,
        0b10000000,
        0b11110000,
        0b00010000,
        0b11110000,
    ],
    [
        0b11110000,
        0b10000000,
        0b11110000,
        0b10010000,
        0b11110000,
    ],
    [
        0b11110000,
        0b00010000,
        0b00010000,
        0b00010000,
        0b00010000,
    ],
    [
        0b11110000,
        0b10010000,
        0b11110000,
        0b10010000,
        0b11110000,
    ],
    [
        0b11110000,
        0b10010000,
        0b11110000,
        0b00010000,
        0b11110000,
    ],
    [
        0b11110000,
        0b10010000,
        0b11110000,
        0b10010000,
        0b10010000,
    ],
    [
        0b11110000,
        0b10010000,
        0b11100000,
        0b10010000,
        0b11110000,
    ],
    [
        0b11110000,
        0b10000000,
        0b10000000,
        0b10000000,
        0b11110000,
    ],
    [
        0b11100000,
        0b10010000,
        0b10010000,
        0b10010000,
        0b11100000,
    ],
    [
        0b11110000,
        0b10000000,
        0b11110000,
        0b10000000,
        0b11110000,
    ],
    [
        0b11110000,
        0b10000000,
        0b11110000,
        0b10000000, 
        0b10000000,
    ],
];

pub fn chip8_machine(user_specified_roms: Vec<RomId>, rom_manager: Arc<RomManager>) -> Machine {
    let machine = Machine::build(GameSystem::Other(OtherSystem::Chip8), rom_manager);

    let (machine, audio_component_id) = machine.default_component::<Chip8Audio>();
    let (machine, timer_component_id) = machine.default_component::<Chip8Timer>();
    let (machine, display_component_id) =
        machine.build_component::<Chip8Display>(Chip8DisplayConfig {
            kind: Chip8Kind::Chip8,
        });

    let (machine, _) = machine.build_component::<Chip8Processor>(Chip8ProcessorConfig {
        frequency: Ratio::from_integer(700),
        kind: Chip8Kind::Chip8,
        display: display_component_id,
        audio: audio_component_id,
        timer: timer_component_id,
    });

    let (machine, _) = machine.build_component::<StandardMemory>(StandardMemoryConfig {
        readable: true,
        writable: true,
        max_word_size: 2,
        assigned_range: 0x000..0x200,
        assigned_address_space: CHIP8_ADDRESS_SPACE_ID,
        initial_contents: StandardMemoryInitialContents::Array {
            value: Cow::Borrowed(bytemuck::cast_slice(&CHIP8_FONT)),
            offset: 0x000,
        },
    });

    let (machine, _) = machine.build_component::<StandardMemory>(StandardMemoryConfig {
        readable: true,
        writable: true,
        max_word_size: 2,
        assigned_range: 0x200..0x1000,
        assigned_address_space: CHIP8_ADDRESS_SPACE_ID,
        initial_contents: StandardMemoryInitialContents::Rom {
            rom_id: user_specified_roms[0],
            offset: 0x200,
        },
    });

    machine.build()
}
