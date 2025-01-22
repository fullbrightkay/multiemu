use crate::{
    component::{memory::MemoryComponent, Component, FromConfig},
    machine::ComponentBuilder,
    memory::{ReadMemoryRecord, WriteMemoryRecord, VALID_ACCESS_SIZES},
    rom::{
        id::RomId,
        manager::{RomManager, RomRequirement},
    },
};
use rand::{thread_rng, RngCore};
use rangemap::RangeMap;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    io::{Read, Write},
    ops::Range,
    sync::{Arc, Mutex},
};

const CHUNK_SIZE: usize = 4096;

#[derive(Debug)]
pub enum StandardMemoryInitialContents {
    Value {
        value: u8,
    },
    Array {
        value: Cow<'static, [u8]>,
        offset: usize,
    },
    Rom {
        rom_id: RomId,
        offset: usize,
    },
    Random,
}

#[derive(Debug)]
pub struct StandardMemoryConfig {
    // If the buffer is readable
    pub readable: bool,
    // If the buffer is writable
    pub writable: bool,
    // The maximum word size
    pub max_word_size: usize,
    // Memory region this buffer will be mapped to
    pub assigned_range: Range<usize>,
    // Initial contents
    pub initial_contents: StandardMemoryInitialContents,
}

impl Default for StandardMemoryConfig {
    fn default() -> Self {
        Self {
            readable: true,
            writable: true,
            max_word_size: 8,
            assigned_range: 0..0,
            initial_contents: StandardMemoryInitialContents::Value { value: 0 },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StandardMemorySnapshot {
    pub memory: Vec<u8>,
}

pub struct StandardMemory {
    config: StandardMemoryConfig,
    buffer: Vec<Mutex<[u8; CHUNK_SIZE]>>,
    rom_manager: Arc<RomManager>,
}

impl Component for StandardMemory {
    fn reset(&self) {
        self.initialize_buffer();
    }

    fn save_snapshot(&self) -> rmpv::Value {
        let mut memory = Vec::new();

        for chunk in self.buffer.iter() {
            let chunk_guard = chunk.lock().unwrap();
            memory.write_all(chunk_guard.as_slice()).unwrap();
        }

        let state = StandardMemorySnapshot { memory };

        rmpv::ext::to_value(&state).unwrap()
    }

    fn load_snapshot(&self, state: rmpv::Value) {
        let state = rmpv::ext::from_value::<StandardMemorySnapshot>(state).unwrap();

        assert_eq!(state.memory.len(), self.config.assigned_range.len());

        // This also does size validation
        for (src, dest) in state.memory.chunks(4096).zip(self.buffer.iter()) {
            let mut dest_guard = dest.lock().unwrap();
            dest_guard[..src.len()].copy_from_slice(src);
        }
    }
}

impl FromConfig for StandardMemory {
    type Config = StandardMemoryConfig;

    fn from_config(component_builder: &mut ComponentBuilder<Self>, config: Self::Config) {
        assert!(
            VALID_ACCESS_SIZES.contains(&config.max_word_size),
            "Invalid word size"
        );
        assert!(
            !config.assigned_range.is_empty(),
            "Memory assigned must be non-empty"
        );

        let buffer_size = config.assigned_range.len();
        let chunks_needed = buffer_size.div_ceil(CHUNK_SIZE);
        let buffer = Vec::from_iter(
            std::iter::repeat([0; CHUNK_SIZE])
                .take(chunks_needed)
                .map(Mutex::new),
        );
        let assigned_range = config.assigned_range.clone();

        let me = Self {
            config,
            buffer: buffer.into_iter().collect(),
            rom_manager: component_builder.machine().rom_manager.clone(),
        };
        me.initialize_buffer();

        component_builder
            .set_component(me)
            .set_memory([assigned_range]);
    }
}

impl MemoryComponent for StandardMemory {
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

        if !self.config.readable {
            errors.insert(address..address + buffer.len(), ReadMemoryRecord::Denied);
        }

        let requested_range = address - self.config.assigned_range.start
            ..address - self.config.assigned_range.start + buffer.len();
        let invalid_before_range = address..self.config.assigned_range.start;
        let invalid_after_range = self.config.assigned_range.end..address + buffer.len();

        if !invalid_after_range.is_empty() || !invalid_before_range.is_empty() {
            errors.extend(
                [invalid_after_range, invalid_before_range]
                    .into_iter()
                    .filter_map(|range| {
                        if !range.is_empty() {
                            Some((range, ReadMemoryRecord::Denied))
                        } else {
                            None
                        }
                    }),
            );
        }

        if !errors.is_empty() {
            return;
        }

        let start_chunk = requested_range.start / CHUNK_SIZE;
        let end_chunk = requested_range.end.div_ceil(CHUNK_SIZE);

        let mut buffer_offset = 0;

        for chunk_index in start_chunk..end_chunk {
            let chunk = &self.buffer[chunk_index];

            let chunk_start = if chunk_index == start_chunk {
                requested_range.start % CHUNK_SIZE
            } else {
                0
            };

            let chunk_end = if chunk_index == end_chunk - 1 {
                requested_range.end % CHUNK_SIZE
            } else {
                CHUNK_SIZE
            };

            // Lock the chunk and read the relevant part
            let locked_chunk = chunk.lock().unwrap();
            buffer[buffer_offset..buffer_offset + chunk_end - chunk_start]
                .copy_from_slice(&locked_chunk[chunk_start..chunk_end]);

            buffer_offset += chunk_end - chunk_start;

            if buffer_offset >= buffer.len() {
                break;
            }
        }
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

        if !self.config.writable {
            errors.insert(address..address + buffer.len(), WriteMemoryRecord::Denied);
        }

        let requested_range = address - self.config.assigned_range.start
            ..address - self.config.assigned_range.start + buffer.len();
        let invalid_before_range = address..self.config.assigned_range.start;
        let invalid_after_range = self.config.assigned_range.end..address + buffer.len();

        if !invalid_after_range.is_empty() || !invalid_before_range.is_empty() {
            errors.extend(
                [invalid_after_range, invalid_before_range]
                    .into_iter()
                    .filter_map(|range| {
                        if !range.is_empty() {
                            Some((range, WriteMemoryRecord::Denied))
                        } else {
                            None
                        }
                    }),
            );
        }

        if !errors.is_empty() {
            return;
        }

        let start_chunk = requested_range.start / CHUNK_SIZE;
        let end_chunk = requested_range.end.div_ceil(CHUNK_SIZE);

        let mut buffer_offset = 0;

        for chunk_index in start_chunk..end_chunk {
            let chunk = &self.buffer[chunk_index];

            let chunk_start = if chunk_index == start_chunk {
                requested_range.start % CHUNK_SIZE
            } else {
                0
            };

            let chunk_end = if chunk_index == end_chunk - 1 {
                requested_range.end % CHUNK_SIZE
            } else {
                CHUNK_SIZE
            };

            let mut locked_chunk = chunk.lock().unwrap();
            locked_chunk[chunk_start..chunk_end]
                .copy_from_slice(&buffer[buffer_offset..buffer_offset + chunk_end - chunk_start]);

            buffer_offset += chunk_end - chunk_start;

            if buffer_offset >= buffer.len() {
                break;
            }
        }
    }
}

impl StandardMemory {
    fn initialize_buffer(&self) {
        let buffer_size = self.config.assigned_range.len();

        // HACK: This overfills the buffer for ease of programming, but its ok because the actual mmu doesn't allow accesses out at runtime
        match &self.config.initial_contents {
            StandardMemoryInitialContents::Value { value } => {
                self.buffer
                    .par_iter()
                    .for_each(|chunk| chunk.lock().unwrap().fill(*value));
            }
            StandardMemoryInitialContents::Random => {
                self.buffer.par_iter().for_each(|chunk| {
                    thread_rng().fill_bytes(chunk.lock().unwrap().as_mut_slice())
                });
            }
            StandardMemoryInitialContents::Array { value, offset } => {
                // Adjust the offset relatively to the local buffer
                let adjusted_offset = offset - self.config.assigned_range.start;
                // Get the start chunk
                let start_chunk = adjusted_offset / CHUNK_SIZE;
                let end_chunk = (adjusted_offset + value.len()).min(buffer_size) / CHUNK_SIZE;

                // Shut up
                #[allow(clippy::needless_range_loop)]
                for chunk_index in start_chunk..end_chunk {
                    let chunk_start = if chunk_index == start_chunk {
                        adjusted_offset % CHUNK_SIZE
                    } else {
                        0
                    };

                    let chunk_end = if chunk_index == end_chunk {
                        (adjusted_offset + value.len()) % CHUNK_SIZE
                    } else {
                        CHUNK_SIZE
                    };

                    let copy_start = chunk_index * CHUNK_SIZE;
                    let copy_end = (chunk_index + 1) * CHUNK_SIZE;

                    let value_start = copy_start - adjusted_offset;
                    let value_end = (copy_end - adjusted_offset).min(value.len());

                    let chunk = &self.buffer[chunk_index];
                    chunk.lock().unwrap()[chunk_start..chunk_end]
                        .copy_from_slice(&value[value_start..value_end]);
                }
            }
            StandardMemoryInitialContents::Rom { rom_id, offset } => {
                let mut rom_file = self
                    .rom_manager
                    .open(*rom_id, RomRequirement::Required)
                    .unwrap();

                let adjusted_offset = offset - self.config.assigned_range.start;
                let mut rom_index = adjusted_offset;

                for chunk in self.buffer.iter() {
                    let chunk_start = rom_index % CHUNK_SIZE;
                    let chunk_end = (rom_index + CHUNK_SIZE).min(buffer_size);

                    let read_len = chunk_end - rom_index;
                    let mut temp_buffer = vec![0; read_len];
                    let bytes_read = rom_file.read(&mut temp_buffer).unwrap();
                    if bytes_read == 0 {
                        break;
                    }

                    chunk.lock().unwrap()[chunk_start..chunk_start + bytes_read]
                        .copy_from_slice(&temp_buffer[..bytes_read]);

                    rom_index += bytes_read;
                    if rom_index >= buffer_size {
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::machine::Machine;

    use super::*;

    #[test]
    fn basic_read() {
        let rom_manager = Arc::new(RomManager::new(None).unwrap());
        let machine = Machine::build(rom_manager)
            .build_component::<StandardMemory>(StandardMemoryConfig {
                max_word_size: 8,
                readable: true,
                writable: true,
                assigned_range: 0..0x10000,
                initial_contents: StandardMemoryInitialContents::Value { value: 0xff },
            })
            .0
            .build();
        let mut buffer = [0; 8];

        machine.memory_translation_table.read(0, &mut buffer);
        assert_eq!(buffer, [0xff; 8]);
    }

    #[test]
    fn basic_write() {
        let rom_manager = Arc::new(RomManager::new(None).unwrap());
        let machine = Machine::build(rom_manager)
            .build_component::<StandardMemory>(StandardMemoryConfig {
                max_word_size: 8,
                readable: true,
                writable: true,
                assigned_range: 0..0x10000,
                initial_contents: StandardMemoryInitialContents::Value { value: 0xff },
            })
            .0
            .build();
        let buffer = [0; 8];

        machine.memory_translation_table.write(0, &buffer);
    }

    #[test]
    fn basic_read_write() {
        let rom_manager = Arc::new(RomManager::new(None).unwrap());
        let machine = Machine::build(rom_manager)
            .build_component::<StandardMemory>(StandardMemoryConfig {
                max_word_size: 8,
                readable: true,
                writable: true,
                assigned_range: 0..0x10000,
                initial_contents: StandardMemoryInitialContents::Value { value: 0xff },
            })
            .0
            .build();
        let mut buffer = [0xff; 8];

        machine.memory_translation_table.write(0, &buffer);
        buffer.fill(0);
        machine.memory_translation_table.read(0, &mut buffer);
        assert_eq!(buffer, [0xff; 8]);
    }

    #[test]
    fn extensive() {
        let rom_manager = Arc::new(RomManager::new(None).unwrap());
        let machine = Machine::build(rom_manager)
            .build_component::<StandardMemory>(StandardMemoryConfig {
                max_word_size: 8,
                readable: true,
                writable: true,
                assigned_range: 0..0x10000,
                initial_contents: StandardMemoryInitialContents::Value { value: 0xff },
            })
            .0
            .build();
        let mut buffer = [0xff; 1];

        for i in 0..0x10000 {
            machine.memory_translation_table.write(i, &buffer);
            buffer.fill(0x00);
            machine.memory_translation_table.read(i, &mut buffer);
            assert_eq!(buffer, [0xff; 1]);
        }
    }
}
