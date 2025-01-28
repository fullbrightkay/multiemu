use super::misc::memory::{
    mirror::{MirrorMemory, MirrorMemoryConfig},
    standard::{StandardMemory, StandardMemoryConfig, StandardMemoryInitialContents},
};
use crate::{
    machine::Machine,
    memory::AddressSpaceId,
    rom::{
        id::RomId,
        manager::RomManager,
        system::{GameSystem, NintendoSystem},
    },
};
use ppu::NesPPU;
use rangemap::RangeMap;
use std::sync::Arc;

pub const NES_CPU_ADDRESS_SPACE_ID: AddressSpaceId = 0;
pub const NES_PPU_ADDRESS_SPACE_ID: AddressSpaceId = 1;

mod ppu;

pub fn nes_machine(user_specified_roms: Vec<RomId>, rom_manager: Arc<RomManager>) -> Machine {
    let machine = Machine::build(
        GameSystem::Nintendo(NintendoSystem::NintendoEntertainmentSystem),
        rom_manager,
    );
    // TODO: This is guesswork
    let machine = machine.insert_bus(NES_CPU_ADDRESS_SPACE_ID, 16);
    let machine = machine.insert_bus(NES_PPU_ADDRESS_SPACE_ID, 16);

    // Set up the NES workram
    let (machine, _) = machine.build_component::<StandardMemory>(StandardMemoryConfig {
        readable: true,
        writable: true,
        max_word_size: 2,
        assigned_range: 0x0000..0x0800,
        assigned_address_space: NES_CPU_ADDRESS_SPACE_ID,
        initial_contents: StandardMemoryInitialContents::Random,
    });
    let (machine, _) = machine.build_component::<MirrorMemory>(MirrorMemoryConfig {
        readable: true,
        writable: true,
        assigned_ranges: RangeMap::from_iter([
            (0x0800..0x1000, 0x0000),
            (0x1000..0x1800, 0x0000),
            (0x1800..0x2000, 0x0000),
        ]),
        assigned_address_space: NES_CPU_ADDRESS_SPACE_ID,
    });

    // Set up the PPU
    let (machine, _) = machine.default_component::<NesPPU>();
    let (machine, _) = machine.build_component::<MirrorMemory>(MirrorMemoryConfig {
        readable: true,
        writable: true,
        // Repeats every 8 bytes, not writing it out manually
        assigned_ranges: RangeMap::from_iter(
            (0x2008..0x4000)
                .step_by(8)
                .map(|base| (base..base + 8, 0x2000)),
        ),
        assigned_address_space: NES_CPU_ADDRESS_SPACE_ID,
    });
    // Set up the PPU address space
    // Pattern tables
    let (machine, _) = machine.build_component::<StandardMemory>(StandardMemoryConfig {
        readable: true,
        writable: true,
        max_word_size: 2,
        assigned_range: 0x0000..0x1000,
        assigned_address_space: NES_PPU_ADDRESS_SPACE_ID,
        initial_contents: StandardMemoryInitialContents::Random,
    });
    let (machine, _) = machine.build_component::<StandardMemory>(StandardMemoryConfig {
        readable: true,
        writable: true,
        max_word_size: 2,
        assigned_range: 0x1000..0x2000,
        assigned_address_space: NES_PPU_ADDRESS_SPACE_ID,
        initial_contents: StandardMemoryInitialContents::Random,
    });
    // Name tables
    let (machine, _) = machine.build_component::<StandardMemory>(StandardMemoryConfig {
        readable: true,
        writable: true,
        max_word_size: 2,
        assigned_range: 0x2000..0x2400,
        assigned_address_space: NES_PPU_ADDRESS_SPACE_ID,
        initial_contents: StandardMemoryInitialContents::Random,
    });
    let (machine, _) = machine.build_component::<StandardMemory>(StandardMemoryConfig {
        readable: true,
        writable: true,
        max_word_size: 2,
        assigned_range: 0x2400..0x2800,
        assigned_address_space: NES_PPU_ADDRESS_SPACE_ID,
        initial_contents: StandardMemoryInitialContents::Random,
    });
    let (machine, _) = machine.build_component::<StandardMemory>(StandardMemoryConfig {
        readable: true,
        writable: true,
        max_word_size: 2,
        assigned_range: 0x2800..0x2c00,
        assigned_address_space: NES_PPU_ADDRESS_SPACE_ID,
        initial_contents: StandardMemoryInitialContents::Random,
    });
    let (machine, _) = machine.build_component::<StandardMemory>(StandardMemoryConfig {
        readable: true,
        writable: true,
        max_word_size: 2,
        assigned_range: 0x2c00..0x3000,
        assigned_address_space: NES_PPU_ADDRESS_SPACE_ID,
        initial_contents: StandardMemoryInitialContents::Random,
    });

    machine.build()
}
