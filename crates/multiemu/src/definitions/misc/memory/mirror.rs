use crate::{
    component::{
        memory::{MemoryComponent, PreviewMemoryRecord, ReadMemoryRecord, WriteMemoryRecord},
        Component, FromConfig,
    },
    machine::ComponentBuilder,
    rom::manager::RomManager,
};
use arrayvec::ArrayVec;
use rangemap::RangeMap;
use std::{ops::Range, sync::Arc};

#[derive(Debug)]
pub enum MirrorMemoryOverflowMode {
    // Deny if it goes outside the assigned range if the assigned range is larger than the target
    Deny,
    // Wrap X times
    Wrap(usize),
}

#[derive(Debug)]
pub struct MirrorMemoryConfig {
    pub readable: bool,
    pub writable: bool,
    pub assigned_range: Range<usize>,
    pub targets: RangeMap<usize, usize>,
    pub overflow_mode: MirrorMemoryOverflowMode,
}

pub struct MirrorMemory {
    config: MirrorMemoryConfig,
    rom_manager: Arc<RomManager>,
}

impl Component for MirrorMemory {}

impl FromConfig for MirrorMemory {
    type Config = MirrorMemoryConfig;

    fn from_config(component_builder: &mut ComponentBuilder<Self>, config: Self::Config) {
        component_builder.set_component(Self {
            config,
            rom_manager: component_builder.machine().rom_manager(),
        });
    }
}

impl MemoryComponent for MirrorMemory {
    fn assigned_memory_range(&self) -> Range<usize> {
        self.config.assigned_range.clone()
    }

    fn read_memory(
        &self,
        address: usize,
        buffer: &mut [u8],
        errors: &mut RangeMap<usize, ReadMemoryRecord>,
    ) {
        debug_assert!([1, 2, 4, 8].contains(&buffer.len()));

        let affected_range = address..address + buffer.len();

        if !self.config.readable {
            errors.insert(affected_range, ReadMemoryRecord::Denied);
        }

        let assigned_range_size = self.config.assigned_range.clone().count();
        let target_range_size = self.config.target.clone().count();

        let offset = (address - self.config.assigned_range.start) + self.config.target.start;

        if assigned_range_size > target_range_size && offset >= self.config.target.end {
            match self.config.overflow_mode {
                MirrorMemoryOverflowMode::Deny => {
                    records.push((affected_range.clone(), ReadMemoryRecord::Denied));
                    return (self.config.read_cycle_penalty_calculator)(affected_range, true);
                }
                MirrorMemoryOverflowMode::Wrap(n) => {
                    if offset / target_range_size >= n {
                        records.push((affected_range.clone(), ReadMemoryRecord::Denied));

                        return (self.config.read_cycle_penalty_calculator)(affected_range, true);
                    }

                    let real_offset = offset % target_range_size;

                    records.push((
                        affected_range.clone(),
                        ReadMemoryRecord::Redirect {
                            offset: real_offset,
                        },
                    ));

                    return (self.config.read_cycle_penalty_calculator)(affected_range, false);
                }
            }
        }

        records.push((
            affected_range.clone(),
            ReadMemoryRecord::Redirect { offset },
        ));

        (self.config.read_cycle_penalty_calculator)(affected_range, false)
    }

    fn write_memory(
        &mut self,
        address: usize,
        buffer: &[u8],
        records: &mut ArrayVec<(Range<usize>, WriteMemoryRecord), 8>,
    ) -> u64 {
        debug_assert!([1, 2, 4, 8].contains(&buffer.len()));

        let affected_range = address..address + buffer.len();

        if !self.config.writable {
            records.push((affected_range.clone(), WriteMemoryRecord::Denied));

            return (self.config.write_cycle_penalty_calculator)(affected_range, true);
        }

        let assigned_range_size = self.config.assigned_range.clone().count();
        let target_range_size = self.config.target.clone().count();

        let offset = (address - self.config.assigned_range.start) + self.config.target.start;

        if assigned_range_size > target_range_size && offset >= self.config.target.end {
            match self.config.overflow_mode {
                MirrorMemoryOverflowMode::Deny => {
                    records.push((affected_range.clone(), WriteMemoryRecord::Denied));

                    return (self.config.write_cycle_penalty_calculator)(affected_range, true);
                }
                MirrorMemoryOverflowMode::Wrap(n) => {
                    if offset / target_range_size >= n {
                        records.push((affected_range.clone(), WriteMemoryRecord::Denied));

                        return (self.config.write_cycle_penalty_calculator)(affected_range, true);
                    }

                    let real_offset = offset % target_range_size;

                    records.push((
                        affected_range.clone(),
                        WriteMemoryRecord::Redirect {
                            offset: real_offset,
                        },
                    ));

                    return (self.config.write_cycle_penalty_calculator)(affected_range, false);
                }
            }
        }

        records.push((
            affected_range.clone(),
            WriteMemoryRecord::Redirect { offset },
        ));

        (self.config.write_cycle_penalty_calculator)(affected_range, false)
    }

    fn preview_memory(
        &mut self,
        address: usize,
        buffer: &mut [u8],
        records: &mut ArrayVec<(Range<usize>, PreviewMemoryRecord), 8>,
    ) {
        let affected_range = address..address + buffer.len();

        if !self.config.readable {
            records.push((affected_range.clone(), PreviewMemoryRecord::Denied));
            return;
        }

        let assigned_range_size = self.config.assigned_range.clone().count();
        let target_range_size = self.config.target.clone().count();

        let offset = (address - self.config.assigned_range.start) + self.config.target.start;

        if assigned_range_size > target_range_size && offset >= self.config.target.end {
            match self.config.overflow_mode {
                MirrorMemoryOverflowMode::Deny => {
                    records.push((affected_range.clone(), PreviewMemoryRecord::Denied));
                }
                MirrorMemoryOverflowMode::Wrap(n) => {
                    if offset / target_range_size >= n {
                        records.push((affected_range.clone(), PreviewMemoryRecord::Denied));
                    }

                    let real_offset = offset % target_range_size;

                    records.push((
                        affected_range.clone(),
                        PreviewMemoryRecord::Redirect {
                            offset: real_offset,
                        },
                    ));
                }
            }
        }

        records.push((
            affected_range.clone(),
            PreviewMemoryRecord::Redirect { offset },
        ));
    }
}
