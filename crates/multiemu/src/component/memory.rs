use super::Component;
use crate::memory::{PreviewMemoryRecord, ReadMemoryRecord, WriteMemoryRecord};
use rangemap::RangeMap;
use std::ops::Range;
use thiserror::Error;

pub trait MemoryComponent: Component {
    fn read_memory(
        &self,
        address: usize,
        buffer: &mut [u8],
        errors: &mut RangeMap<usize, ReadMemoryRecord>,
    );

    fn write_memory(
        &self,
        address: usize,
        buffer: &[u8],
        errors: &mut RangeMap<usize, WriteMemoryRecord>,
    );

    // Its like read_memory but without the restriction on the size of the buffer and it cannot cause a state change
    fn preview_memory(
        &self,
        address: usize,
        buffer: &mut [u8],
        errors: &mut RangeMap<usize, PreviewMemoryRecord>,
    ) {
        errors.insert(
            address..address + buffer.len(),
            PreviewMemoryRecord::PreviewImpossible,
        );
    }
}

#[derive(Error, Debug)]
pub enum MemoryOperationError {
    #[error("Memory could not be read/written/previewed")]
    Denied(Range<usize>),
    #[error("Memory access is out of bounds")]
    OutOfBounds(Range<usize>),
}
