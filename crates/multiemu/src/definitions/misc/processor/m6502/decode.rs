use crate::scheduler::query::memory::MemoryMut;

use super::instruction::M6502InstructionSet;
use bitvec::{
    field::BitField,
    prelude::{BitSlice, Msb0},
    view::BitView,
};
use std::ops::Range;

const INSTRUCTION_IDENTIFIER: Range<usize> = 6..8;
const SECONDARY_INSTRUCTION_IDENTIFIER: Range<usize> = 0..3;
const ARGUMENT: Range<usize> = 3..6;

pub fn decode_instruction(
    cursor: usize,
    memory: &MemoryMut,
) -> Result<(M6502InstructionSet, u8), Box<dyn std::error::Error>> {
    let mut instruction_first_byte = 0;
    memory.read(cursor, bytemuck::bytes_of_mut(&mut instruction_first_byte));
    let instruction_first_byte = instruction_first_byte.view_bits::<Msb0>();
    let instruction_identifier = instruction_first_byte[INSTRUCTION_IDENTIFIER].load::<u8>();

    match instruction_identifier {
        0b00 => {
            let instruction_identifier =
                instruction_first_byte[SECONDARY_INSTRUCTION_IDENTIFIER].load::<u8>();

            decode_group3_instruction(
                cursor,
                memory,
                instruction_identifier,
                instruction_first_byte,
            )
        }
        0b01 => {
            let instruction_identifier =
                instruction_first_byte[SECONDARY_INSTRUCTION_IDENTIFIER].load::<u8>();

            decode_group1_space_instruction(
                cursor,
                memory,
                instruction_identifier,
                instruction_first_byte,
            )
        }
        0b10 => {
            let instruction_identifier =
                instruction_first_byte[SECONDARY_INSTRUCTION_IDENTIFIER].load::<u8>();

            decode_group2_space_instruction(
                cursor,
                memory,
                instruction_identifier,
                instruction_first_byte,
            )
        }
        0b11 => {
            let instruction_identifier =
                instruction_first_byte[SECONDARY_INSTRUCTION_IDENTIFIER].load::<u8>();

            decode_undocumented_space_instruction(
                cursor,
                memory,
                instruction_identifier,
                instruction_first_byte,
            )
        }
        _ => {
            unreachable!()
        }
    }
}

#[inline]
pub fn decode_group1_space_instruction(
    cursor: usize,
    memory: &MemoryMut,
    instruction_identifier: u8,
    instruction_first_byte: &BitSlice<u8, Msb0>,
) -> Result<(M6502InstructionSet, u8), Box<dyn std::error::Error>> {
    let addressing_mode = instruction_first_byte[ARGUMENT].load::<u8>();

    match instruction_identifier {
        0b000 => {
            todo!()
        }
        0b001 => {
            todo!()
        }
        0b010 => {
            todo!()
        }
        0b011 => {
            todo!()
        }
        0b100 => {
            todo!()
        }
        0b101 => {
            todo!()
        }
        0b110 => {
            todo!()
        }
        0b111 => {
            todo!()
        }
        _ => {
            unreachable!()
        }
    }
}

#[inline]
pub fn decode_group2_space_instruction(
    cursor: usize,
    memory: &MemoryMut,
    instruction_identifier: u8,
    instruction_first_byte: &BitSlice<u8, Msb0>,
) -> Result<(M6502InstructionSet, u8), Box<dyn std::error::Error>> {
    todo!()
}

#[inline]
pub fn decode_undocumented_space_instruction(
    cursor: usize,
    memory: &MemoryMut,
    instruction_identifier: u8,
    instruction_first_byte: &BitSlice<u8, Msb0>,
) -> Result<(M6502InstructionSet, u8), Box<dyn std::error::Error>> {
    match instruction_identifier {
        0b000 => {
            todo!()
        }
        0b001 => {
            todo!()
        }
        0b010 => {
            todo!()
        }
        0b011 => {
            todo!()
        }
        0b100 => {
            todo!()
        }
        0b101 => {
            todo!()
        }
        0b110 => {
            todo!()
        }
        0b111 => {
            todo!()
        }
        _ => {
            unreachable!()
        }
    }
}

fn decode_group3_instruction(
    cursor: usize,
    memory: &MemoryMut,
    instruction_identifier: u8,
    instruction_first_byte: &BitSlice<u8, Msb0>,
) -> Result<(M6502InstructionSet, u8), Box<dyn std::error::Error>> {
    match instruction_identifier {
        0b000 => {
            todo!()
        }
        0b001 => {
            todo!()
        }
        0b010 => todo!(),
        0b011 => todo!(),
        0b100 => todo!(),
        0b101 => todo!(),
        0b110 => todo!(),
        0b111 => todo!(),
        _ => {
            unreachable!()
        }
    }
}
