use crate::{
    component::{memory::MemoryComponent, Component, FromConfig},
    machine::ComponentBuilder,
    memory::{AddressSpaceId, ReadMemoryRecord, WriteMemoryRecord, VALID_ACCESS_SIZES},
};
use rangemap::RangeMap;

#[derive(Debug)]
pub struct MirrorMemoryConfig {
    pub readable: bool,
    pub writable: bool,
    pub assigned_ranges: RangeMap<usize, usize>,
    /// Address space this exists on
    pub assigned_address_space: AddressSpaceId,
}

#[derive(Debug)]
pub struct MirrorMemory {
    config: MirrorMemoryConfig,
}

impl Component for MirrorMemory {}

impl FromConfig for MirrorMemory {
    type Config = MirrorMemoryConfig;

    fn from_config(component_builder: &mut ComponentBuilder<Self>, config: Self::Config) {
        let assigned_address_space = config.assigned_address_space;
        let assigned_ranges = config.assigned_ranges.clone();

        component_builder.set_component(Self { config }).set_memory(
            assigned_ranges
                .into_iter()
                .map(|(assignment, _)| (assigned_address_space, assignment)),
        );
    }
}

impl MemoryComponent for MirrorMemory {
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

        if !self.config.readable {
            errors.insert(affected_range.clone(), ReadMemoryRecord::Denied);
        }

        let redirect_base_address = self
            .config
            .assigned_ranges
            .get(&affected_range.start)
            .unwrap();
        let adjusted_redirect_base_address =
            redirect_base_address + (address - affected_range.start);

        errors.insert(
            affected_range,
            ReadMemoryRecord::Redirect {
                address: adjusted_redirect_base_address,
            },
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

        let affected_range = address..address + buffer.len();

        if !self.config.writable {
            errors.insert(affected_range.clone(), WriteMemoryRecord::Denied);
        }

        let redirect_base_address = self
            .config
            .assigned_ranges
            .get(&affected_range.start)
            .unwrap();
        let adjusted_redirect_base_address =
            redirect_base_address + (address - affected_range.start);

        errors.insert(
            affected_range,
            WriteMemoryRecord::Redirect {
                address: adjusted_redirect_base_address,
            },
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        definitions::misc::memory::standard::{
            StandardMemory, StandardMemoryConfig, StandardMemoryInitialContents,
        },
        machine::Machine,
        rom::{manager::RomManager, system::GameSystem},
    };
    use std::sync::Arc;

    const ADDRESS_SPACE: AddressSpaceId = 0;

    #[test]
    fn basic_read() {
        let rom_manager = Arc::new(RomManager::new(None).unwrap());
        let machine = Machine::build(GameSystem::Unknown, rom_manager)
            .build_component::<StandardMemory>(StandardMemoryConfig {
                max_word_size: 8,
                readable: true,
                writable: true,
                assigned_range: 0..0x10000,
                assigned_address_space: ADDRESS_SPACE,
                initial_contents: StandardMemoryInitialContents::Value { value: 0xff },
            })
            .0
            .build_component::<MirrorMemory>(MirrorMemoryConfig {
                readable: true,
                writable: true,
                assigned_ranges: RangeMap::from_iter([(0x10000..0x20000, 0x0000)]),
                assigned_address_space: ADDRESS_SPACE,
            })
            .0
            .build();
        let mut buffer = [0; 8];

        machine
            .memory_translation_table
            .read(0x10000, &mut buffer, ADDRESS_SPACE)
            .unwrap();
        assert_eq!(buffer, [0xff; 8]);
    }

    #[test]
    fn basic_write() {
        let rom_manager = Arc::new(RomManager::new(None).unwrap());
        let machine = Machine::build(GameSystem::Unknown, rom_manager)
            .build_component::<StandardMemory>(StandardMemoryConfig {
                max_word_size: 8,
                readable: true,
                writable: true,
                assigned_range: 0..0x10000,
                assigned_address_space: ADDRESS_SPACE,
                initial_contents: StandardMemoryInitialContents::Value { value: 0xff },
            })
            .0
            .build_component::<MirrorMemory>(MirrorMemoryConfig {
                readable: true,
                writable: true,
                assigned_ranges: RangeMap::from_iter([(0x10000..0x20000, 0x0000)]),
                assigned_address_space: ADDRESS_SPACE,
            })
            .0
            .build();
        let buffer = [0; 8];

        machine
            .memory_translation_table
            .write(0x10000, &buffer, ADDRESS_SPACE)
            .unwrap();
    }
}
