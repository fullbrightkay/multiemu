use super::Machine;
use crate::{
    definitions::chip8::chip8_machine,
    rom::{
        id::RomId,
        manager::RomManager,
        system::{GameSystem, NintendoSystem, OtherSystem},
    },
};
use std::sync::Arc;

impl Machine {
    pub fn from_system(
        user_specified_roms: Vec<RomId>,
        rom_manager: Arc<RomManager>,
        system: GameSystem,
    ) -> Machine {
        match system {
            GameSystem::Nintendo(NintendoSystem::GameBoy) => todo!(),
            GameSystem::Nintendo(NintendoSystem::GameBoyColor) => todo!(),
            GameSystem::Nintendo(NintendoSystem::GameBoyAdvance) => todo!(),
            GameSystem::Nintendo(NintendoSystem::NintendoEntertainmentSystem) => todo!(),
            GameSystem::Nintendo(NintendoSystem::SuperNintendoEntertainmentSystem) => todo!(),
            GameSystem::Sega(sega_system) => todo!(),
            GameSystem::Sony(sony_system) => todo!(),
            GameSystem::Atari(atari_system) => todo!(),
            GameSystem::Other(OtherSystem::Chip8) => {
                chip8_machine(user_specified_roms, rom_manager)
            }
            GameSystem::Unknown => todo!(),
            _ => {
                unimplemented!("This system is not supported by this emulator");
            }
        }
    }
}
