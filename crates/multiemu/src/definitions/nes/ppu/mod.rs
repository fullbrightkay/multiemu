use crate::{
    component::{memory::MemoryComponent, Component, FromConfig},
    machine::ComponentBuilder,
    memory::{AddressSpaceId, MemoryTranslationTable, ReadMemoryRecord, WriteMemoryRecord},
};
use std::sync::Arc;

use super::{NES_CPU_ADDRESS_SPACE_ID, NES_PPU_ADDRESS_SPACE_ID};

// We store ppu state registers in normal struct sizes for easier gpu access

const PPUCTRL_ADDRESS: usize = 0x2000;
const PPUMASK_ADDRESS: usize = 0x2001;
const PPUSTATUS_ADDRESS: usize = 0x2002;
const OAMADDR_ADDRESS: usize = 0x2003;

pub struct OamData {}

impl OamData {
    const ADDRESS: usize = 0x2004;
}

const PPUSCROLL_ADDRESS: usize = 0x2005;
const PPUADDR_ADDRESS: usize = 0x2006;
const PPUDATA_ADDRESS: usize = 0x2007;
const OAMDMA_ADDRESS: usize = 0x4014;

struct State {
    oamdata: u8,
}

#[derive(Debug)]
pub(super) struct NesPPU {}

impl Component for NesPPU {
    fn set_memory_translation_table(&self, _memory_translation_table: Arc<MemoryTranslationTable>) {
    }
}

impl FromConfig for NesPPU {
    type Config = ();

    fn from_config(component_builder: &mut ComponentBuilder<Self>, config: Self::Config) {
        component_builder
            .set_component(Self {})
            // Claim our registers
            .set_memory([
                (NES_CPU_ADDRESS_SPACE_ID, 0x2000..0x2008),
                (NES_CPU_ADDRESS_SPACE_ID, 0x4014..0x4015),
            ]);
    }
}

impl MemoryComponent for NesPPU {
    fn read_memory(
        &self,
        address: usize,
        buffer: &mut [u8],
        _address_space: AddressSpaceId,
        errors: &mut rangemap::RangeMap<usize, ReadMemoryRecord>,
    ) {
        match address {
            PPUCTRL_ADDRESS => {}
            PPUMASK_ADDRESS => {}
            PPUSTATUS_ADDRESS => {}
            OAMADDR_ADDRESS => {}
            OamData::ADDRESS => {}
            PPUSCROLL_ADDRESS => {}
            PPUADDR_ADDRESS => {}
            PPUDATA_ADDRESS => {}
            OAMDMA_ADDRESS => {}
            _ => {
                unreachable!()
            }
        }
    }

    fn write_memory(
        &self,
        address: usize,
        buffer: &[u8],
        _address_space: AddressSpaceId,
        errors: &mut rangemap::RangeMap<usize, WriteMemoryRecord>,
    ) {
        match address {
            PPUCTRL_ADDRESS => {}
            PPUMASK_ADDRESS => {}
            PPUSTATUS_ADDRESS => {}
            OAMADDR_ADDRESS => {}
            OamData::ADDRESS => {}
            PPUSCROLL_ADDRESS => {}
            PPUADDR_ADDRESS => {}
            PPUDATA_ADDRESS => {}
            OAMDMA_ADDRESS => {}
            _ => {
                unreachable!()
            }
        }
    }
}
