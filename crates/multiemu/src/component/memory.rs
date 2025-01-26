use super::Component;
use crate::memory::{AddressSpaceId, PreviewMemoryRecord, ReadMemoryRecord, WriteMemoryRecord};
use rangemap::RangeMap;
use std::ops::Range;
use thiserror::Error;

pub trait MemoryComponent: Component {
    fn read_memory(
        &self,
        address: usize,
        buffer: &mut [u8],
        address_space: AddressSpaceId,
        errors: &mut RangeMap<usize, ReadMemoryRecord>,
    );

    fn write_memory(
        &self,
        address: usize,
        buffer: &[u8],
        address_space: AddressSpaceId,
        errors: &mut RangeMap<usize, WriteMemoryRecord>,
    );

    // Its like read_memory but without the restriction on the size of the buffer and it cannot cause a state change
    fn preview_memory(
        &self,
        address: usize,
        buffer: &mut [u8],
        address_space: AddressSpaceId,
        errors: &mut RangeMap<usize, PreviewMemoryRecord>,
    ) {
        let mut read_errors = RangeMap::default();
        self.read_memory(address, buffer, address_space, &mut read_errors);

        // Translate read errors to preview errors
        for (range, error) in read_errors {
            match error {
                ReadMemoryRecord::Denied => errors.insert(range, PreviewMemoryRecord::Denied),
                ReadMemoryRecord::Redirect { address } => {
                    errors.insert(range, PreviewMemoryRecord::Redirect { address })
                }
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum MemoryOperationError {
    #[error("Memory could not be read/written/previewed")]
    Denied(Range<usize>),
    #[error("Memory access is out of bounds")]
    OutOfBounds(Range<usize>),
}
