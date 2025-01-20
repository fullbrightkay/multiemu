use crate::{
    component::{memory::MemoryComponent, Component, FromConfig},
    machine::ComponentBuilder,
    memory::{PreviewMemoryRecord, ReadMemoryRecord, WriteMemoryRecord, VALID_ACCESS_SIZES},
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
}

impl Component for MirrorMemory {}

impl FromConfig for MirrorMemory {
    type Config = MirrorMemoryConfig;

    fn from_config(component_builder: &mut ComponentBuilder<Self>, config: Self::Config) {
        component_builder.set_component(Self { config });
    }
}

impl MemoryComponent for MirrorMemory {
    fn read_memory(
        &self,
        address: usize,
        buffer: &mut [u8],
        errors: &mut RangeMap<usize, ReadMemoryRecord>,
    ) {
        debug_assert!(
            VALID_ACCESS_SIZES.contains(&buffer.len()),
            "Invalid memory access size {}",
            buffer.len()
        );

        let affected_range = address..address + buffer.len();

        if !self.config.readable {
            errors.insert(affected_range, ReadMemoryRecord::Denied);
        }

        let assigned_range_size = self.config.assigned_range.len();
        let target_range_size = self.config.target.clone().count();

        let offset = (address - self.config.assigned_range.start) + self.config.target.start;

        if assigned_range_size > target_range_size && offset >= self.config.target.end {
            match self.config.overflow_mode {
                MirrorMemoryOverflowMode::Deny => {
                    errors.insert(affected_range.clone(), ReadMemoryRecord::Denied);
                }
                MirrorMemoryOverflowMode::Wrap(n) => {
                    if offset / target_range_size >= n {
                        errors.push((affected_range.clone(), ReadMemoryRecord::Denied));
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

    }

    fn write_memory(
        &self,
        address: usize,
        buffer: &[u8],
        errors: &mut RangeMap<usize, WriteMemoryRecord>,
    ) {
        debug_assert!(
            VALID_ACCESS_SIZES.contains(&buffer.len()),
            "Invalid memory access size {}",
            buffer.len()
        );

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
