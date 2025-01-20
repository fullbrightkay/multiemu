use super::instruction::{Chip8InstructionSet, InstructionSetChip8, Register};
use bitvec::{field::BitField, prelude::Msb0, view::BitView};
use nalgebra::Point2;

pub fn decode_instruction(
    instruction: [u8; 2],
) -> Result<Chip8InstructionSet, Box<dyn std::error::Error>> {
    let instruction_view = instruction.view_bits::<Msb0>();

    match instruction_view[0..4].load::<u8>() {
        0x0 => {
            let syscall = instruction_view[4..16].load_be::<u16>();

            Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Sys {
                syscall,
            }))
        }
        0x1 => {
            let address = instruction_view[4..16].load_be::<u16>();

            Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Jump {
                address,
            }))
        }
        0x2 => {
            let address = instruction_view[4..16].load_be::<u16>();

            Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Call {
                address,
            }))
        }
        0x3 => {
            let register = instruction_view[4..8].load::<u8>();
            let immediate = instruction_view[8..16].load::<u8>();

            Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Ske {
                register: Register::try_from(register).unwrap(),
                immediate,
            }))
        }
        0x4 => {
            let register = instruction_view[4..8].load::<u8>();
            let immediate = instruction_view[8..16].load::<u8>();

            Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Skne {
                register: Register::try_from(register).unwrap(),
                immediate,
            }))
        }
        0x5 => {
            let param_register_1 = instruction_view[4..8].load::<u8>();
            let param_register_2 = instruction_view[8..12].load::<u8>();

            Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Skre {
                param_register_1: Register::try_from(param_register_1).unwrap(),
                param_register_2: Register::try_from(param_register_2).unwrap(),
            }))
        }
        0x6 => {
            let register = instruction_view[4..8].load::<u8>();
            let immediate = instruction_view[8..16].load::<u8>();

            Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Load {
                register: Register::try_from(register).unwrap(),
                immediate,
            }))
        }
        0x7 => {
            let register = instruction_view[4..8].load::<u8>();
            let immediate = instruction_view[8..16].load::<u8>();

            Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Add {
                register: Register::try_from(register).unwrap(),
                immediate,
            }))
        }
        0x8 => {
            let param_register_1 = instruction_view[4..8].load::<u8>();
            let param_register_2 = instruction_view[8..12].load::<u8>();

            let specifier = instruction_view[12..16].load::<u8>();

            match specifier {
                0x0 => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Move {
                    param_register_1: Register::try_from(param_register_1).unwrap(),
                    param_register_2: Register::try_from(param_register_2).unwrap(),
                })),
                0x1 => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Or {
                    destination: Register::try_from(param_register_1).unwrap(),
                    source: Register::try_from(param_register_2).unwrap(),
                })),
                0x2 => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::And {
                    destination: Register::try_from(param_register_1).unwrap(),
                    source: Register::try_from(param_register_2).unwrap(),
                })),
                0x3 => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Xor {
                    destination: Register::try_from(param_register_1).unwrap(),
                    source: Register::try_from(param_register_2).unwrap(),
                })),
                0x4 => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Addr {
                    destination: Register::try_from(param_register_1).unwrap(),
                    source: Register::try_from(param_register_2).unwrap(),
                })),
                0x5 => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Sub {
                    destination: Register::try_from(param_register_1).unwrap(),
                    source: Register::try_from(param_register_2).unwrap(),
                })),
                0x6 => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Shr {
                    register: Register::try_from(param_register_1).unwrap(),
                    value: Register::try_from(param_register_2).unwrap(),
                })),
                0x7 => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Subn {
                    destination: Register::try_from(param_register_1).unwrap(),
                    source: Register::try_from(param_register_2).unwrap(),
                })),
                0xe => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Shl {
                    register: Register::try_from(param_register_1).unwrap(),
                    value: Register::try_from(param_register_2).unwrap(),
                })),
                _ => {
                    unimplemented!()
                }
            }
        }
        0x9 => {
            let param_register_1 = instruction_view[4..8].load::<u8>();
            let param_register_2 = instruction_view[8..12].load::<u8>();

            match instruction_view[12..16].load::<u8>() {
                0x0 => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Skrne {
                    param_register_1: Register::try_from(param_register_1).unwrap(),
                    param_register_2: Register::try_from(param_register_2).unwrap(),
                })),
                _ => {
                    unimplemented!()
                }
            }
        }
        0xa => {
            let value = instruction_view[4..16].load_be::<u16>();

            Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Loadi {
                value,
            }))
        }
        0xb => {
            let address = instruction_view[4..16].load_be::<u16>();

            Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Jumpi {
                address,
            }))
        }
        0xc => {
            let register = instruction_view[4..8].load::<u8>();
            let immediate = instruction_view[8..16].load::<u8>();

            Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Rand {
                register: Register::try_from(register).unwrap(),
                immediate,
            }))
        }
        0xd => {
            let x_register = instruction_view[4..8].load::<u8>();
            let y_register = instruction_view[8..12].load::<u8>();
            let height = instruction_view[12..16].load::<u8>();

            Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Draw {
                coordinate_registers: Point2::new(
                    Register::try_from(x_register).unwrap(),
                    Register::try_from(y_register).unwrap(),
                ),
                height,
            }))
        }
        0xe => {
            let register = instruction_view[4..8].load::<u8>();

            match instruction_view[8..16].load::<u8>() {
                0x9e => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Skpr {
                    key: Register::try_from(register).unwrap(),
                })),
                0xa1 => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Skup {
                    key: Register::try_from(register).unwrap(),
                })),
                _ => {
                    unimplemented!()
                }
            }
        }
        0xf => {
            let register = instruction_view[4..8].load::<u8>();

            match instruction_view[8..16].load::<u8>() {
                0x07 => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Moved {
                    register: Register::try_from(register).unwrap(),
                })),
                0x0a => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Keyd {
                    key: Register::try_from(register).unwrap(),
                })),
                0x15 => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Loadd {
                    register: Register::try_from(register).unwrap(),
                })),
                0x18 => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Loads {
                    register: Register::try_from(register).unwrap(),
                })),
                0x1e => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Addi {
                    register: Register::try_from(register).unwrap(),
                })),
                0x29 => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Font {
                    register: Register::try_from(register).unwrap(),
                })),
                0x33 => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Bcd {
                    register: Register::try_from(register).unwrap(),
                })),
                0x55 => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Save {
                    count: register,
                })),
                0x65 => Ok(Chip8InstructionSet::Chip8(InstructionSetChip8::Restore {
                    count: register,
                })),
                _ => {
                    unimplemented!("{:#04x?}", instruction);
                }
            }
        }
        _ => {
            unreachable!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn syscall() {
        assert_eq!(
            decode_instruction([0x00, 0x00]).unwrap(),
            Chip8InstructionSet::Chip8(InstructionSetChip8::Sys { syscall: 0 })
        )
    }
}
