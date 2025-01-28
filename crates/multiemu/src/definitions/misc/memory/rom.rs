use crate::{
    component::{memory::MemoryComponent, Component, FromConfig},
    machine::ComponentBuilder,
    memory::{
        AddressSpaceId, PreviewMemoryRecord, ReadMemoryRecord, WriteMemoryRecord,
        VALID_ACCESS_SIZES,
    },
    rom::{id::RomId, manager::RomRequirement},
};
use memmap2::{Mmap, MmapOptions};
use rangemap::RangeMap;
use std::ops::Range;

#[derive(Debug)]
pub struct RomMemoryConfig {
    pub rom: RomId,
    // The maximum word size
    pub max_word_size: u8,
    // Memory region this buffer will be mapped to
    pub assigned_range: Range<usize>,
    /// Address space this exists on
    pub assigned_address_space: AddressSpaceId,
}

#[derive(Debug)]
pub struct RomMemory {
    config: RomMemoryConfig,
    // FIXME: Create a fallback for platforms without mmap
    rom: Mmap,
}

impl Component for RomMemory {
    fn reset(&self) {
        // This is basically a stateless component so there isn't any need to reset
    }
}

impl FromConfig for RomMemory {
    type Config = RomMemoryConfig;

    fn from_config(component_builder: &mut ComponentBuilder<Self>, config: Self::Config) {
        let rom_file = component_builder
            .machine()
            .rom_manager
            .open(config.rom, RomRequirement::Required)
            .unwrap();

        let assigned_range = config.assigned_range.clone();
        let assigned_address_space = config.assigned_address_space;
        let rom = unsafe { MmapOptions::new().map(&rom_file).unwrap() };

        component_builder
            .set_component(Self { config, rom })
            .set_memory([(assigned_address_space, assigned_range)]);
    }
}

impl MemoryComponent for RomMemory {
    fn read_memory(
        &self,
        address: usize,
        buffer: &mut [u8],
        _address_space: AddressSpaceId,
        errors: &mut RangeMap<usize, ReadMemoryRecord>,
    ) {
        debug_assert!(
            VALID_ACCESS_SIZES.contains(&buffer.len()),
            "Invalid memory access size {}",
            buffer.len()
        );

        let affected_range = address..address + buffer.len();

        if buffer.len() > self.config.max_word_size as usize {
            errors.insert(affected_range.clone(), ReadMemoryRecord::Denied);
        }

        let adjusted_offset = address - self.config.assigned_range.start;
        buffer.copy_from_slice(
            &self.rom[adjusted_offset..(adjusted_offset + buffer.len()).min(self.rom.len())],
        );
    }

    fn write_memory(
        &self,
        address: usize,
        buffer: &[u8],
        _address_space: AddressSpaceId,
        errors: &mut RangeMap<usize, WriteMemoryRecord>,
    ) {
        debug_assert!(
            VALID_ACCESS_SIZES.contains(&buffer.len()),
            "Invalid memory access size {}",
            buffer.len()
        );
        errors.insert(address..address + buffer.len(), WriteMemoryRecord::Denied);
    }

    fn preview_memory(
        &self,
        address: usize,
        buffer: &mut [u8],
        _address_space: AddressSpaceId,
        _errors: &mut RangeMap<usize, PreviewMemoryRecord>,
    ) {
        let adjusted_offset = address - self.config.assigned_range.start;
        buffer.copy_from_slice(
            &self.rom[adjusted_offset..(adjusted_offset + buffer.len()).min(self.rom.len())],
        );
    }
}
