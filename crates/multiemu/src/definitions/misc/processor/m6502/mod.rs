use crate::{
    component::{
        processor::ProcessorComponent, schedulable::SchedulableComponent, Component, FromConfig,
    },
    machine::ComponentBuilder,
    processor::InstructionDecompilingError,
    scheduler::query::memory::MemoryMut,
};
use bitvec::{prelude::Lsb0, view::BitView};
use decode::decode_instruction;
use enumflags2::{bitflags, BitFlag, BitFlags};
use instruction::{AddressingMode, M6502InstructionSet, M6502InstructionSetSpecifier};
use num::rational::Ratio;

pub mod decode;
pub mod instruction;
/*
#[cfg(test)]
pub mod test;
*/

pub enum M6502Kind {
    /// Standard
    M6502 {
        /// Whether to emulated the broken ROR instruction
        quirk_broken_ror: bool,
    },
    /// Slimmed down atari 2600 version
    M6507,
    /// NES version
    R2A03,
    /// NES version
    R2A07,
}

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
enum FlagRegister {
    /// Set when bit 7 is set on various math operations
    Negative = 0b1000_0000,
    /// Set when a math operation involves an overflow
    Overflow = 0b0100_0000,
    /// This flag is usually 1, it doesn't mean anything
    __Unused = 0b0010_0000,
    /// Flag to inform software the reason behind some behaviors
    Break = 0b0001_0000,
    /// Decimal math mode, it enables bcd operations on a lot of math instructions and introduces some bugs
    Decimal = 0b0000_1000,
    /// Interrupt disable
    InterruptDisable = 0b0000_0100,
    /// Set when the result of a math operation is 0
    Zero = 0b0000_0010,
    Carry = 0b0000_0001,
}

pub struct M6502Registers {
    stack_pointer: u8,
    accumulator: u8,
    index_registers: [u8; 2],
    flags: BitFlags<FlagRegister>,
}

#[derive(Debug)]
pub struct M6502Config {
    pub frequency: Ratio<u32>,
}

pub struct M6502 {
    config: M6502Config,
    registers: M6502Registers,
}

impl Component for M6502 {}

impl FromConfig for M6502 {
    type Config = M6502Config;

    fn from_config(component_builder: &mut ComponentBuilder<Self>, config: Self::Config) {
        component_builder
            .set_component(Self {
                config,
                registers: M6502Registers {
                    stack_pointer: 0xff,
                    accumulator: 0,
                    index_registers: [0, 0],
                    flags: BitFlags::empty(),
                },
            })
            .set_scheduable();
    }
}

macro_rules! load_m6502_addressing_modes {
    ($instruction:expr, $register_store:expr, $memory:expr, [$($modes:ident),*]) => {{
        match $instruction.addressing_mode {
            $(
                Some(AddressingMode::$modes(argument)) => {
                    load_m6502_addressing_modes!(@handler $modes, argument, $register_store, $memory)
                },
            )*
            _ => unreachable!(),
        }
    }};

    (@handler Immediate, $argument:expr, $register_store:expr, $memory:expr) => {{
        $argument
    }};

    (@handler Absolute, $argument:expr, $register_store:expr, $memory:expr) => {{
        let mut value = 0;

        $memory
            .read($argument as usize, bytemuck::bytes_of_mut(&mut value));

        value
    }};

    (@handler XIndexedAbsolute, $argument:expr, $register_store:expr, $memory:expr) => {{
        let mut value = 0;

        $memory
            .read($argument as usize, &mut [0]);


        let actual_address = $argument.wrapping_add($register_store.index_registers[0] as u16);
        $memory
            .read(actual_address as usize, bytemuck::bytes_of_mut(&mut value));

        value
    }};

    (@handler YIndexedAbsolute, $argument:expr, $register_store:expr, $memory:expr) => {{
        let mut value = 0;

        $memory
            .read($argument as usize, &mut [0]);


        let actual_address = $argument.wrapping_add($register_store.index_registers[1] as u16);
        $memory
            .read(actual_address as usize, bytemuck::bytes_of_mut(&mut value));

        value
    }};

    (@handler ZeroPage, $argument:expr, $register_store:expr, $memory:expr) => {{
        let mut value = 0;

        $memory
            .read($argument as usize, bytemuck::bytes_of_mut(&mut value));

        value
    }};

    (@handler XIndexedZeroPage, $argument:expr, $register_store:expr, $memory:expr) => {{
        let mut value = 0;

        let actual_address = $argument.wrapping_add($register_store.index_registers[0]);

        $memory
            .read(actual_address as usize, bytemuck::bytes_of_mut(&mut value));

        value
    }};

    (@handler YIndexedZeroPage, $argument:expr, $register_store:expr, $memory:expr) => {{
        let mut value = 0;

        let actual_address = $argument.wrapping_add($register_store.index_registers[1]);

        $memory
            .read(actual_address as usize, bytemuck::bytes_of_mut(&mut value));

        value
    }};

    (@handler XIndexedZeroPageIndirect, $argument:expr, $register_store:expr, $memory:expr) => {{
        let mut value = 0;

        let indirection_address = $argument.wrapping_add($register_store.index_registers[0]);
        let mut actual_address = [0; 2];

        $memory
            .read(indirection_address as usize, &mut actual_address);

        let actual_address = u16::from_le_bytes(actual_address);

        $memory
            .read(actual_address as usize, bytemuck::bytes_of_mut(&mut value));

        value
    }};

    (@handler ZeroPageIndirectYIndexed, $argument:expr, $register_store:expr, $memory:expr) => {{
         let mut value = 0;

        let mut indirection_address = 0;

        $memory
            .read($argument as usize, bytemuck::bytes_of_mut(&mut indirection_address));

        let indirection_address = (indirection_address as u16)
            .wrapping_add($register_store.index_registers[1] as u16);

        $memory
            .read(indirection_address as usize, bytemuck::bytes_of_mut(&mut value));

        value
    }};
}

impl ProcessorComponent for M6502 {
    type InstructionSet = M6502InstructionSet;

    fn should_execution_occur(&self) -> bool {
        todo!()
    }

    fn decompile(
        &self,
        cursor: usize,
        memory: &mut MemoryMut,
    ) -> Result<(Self::InstructionSet, u8), InstructionDecompilingError> {
        Ok(decode_instruction(cursor, memory).unwrap())
    }

    fn interpret(
        &mut self,
        program_pointer: &mut usize,
        instruction: Self::InstructionSet,
        memory: &mut MemoryMut,
    ) -> Result<(), String> {
        match instruction.specifier {
            M6502InstructionSetSpecifier::Adc => {
                let value = load_m6502_addressing_modes!(
                    instruction,
                    self.registers,
                    memory,
                    [
                        Immediate,
                        Absolute,
                        XIndexedAbsolute,
                        YIndexedAbsolute,
                        ZeroPage,
                        XIndexedZeroPage,
                        XIndexedZeroPageIndirect,
                        ZeroPageIndirectYIndexed
                    ]
                );

                let carry_value = self.registers.flags.contains(FlagRegister::Carry) as u8;

                let (first_operation_result, first_operation_overflow) =
                    self.registers.accumulator.overflowing_add(value);

                let (second_operation_result, second_operation_overflow) =
                    first_operation_result.overflowing_add(carry_value);

                self.registers.flags.set(
                    FlagRegister::Overflow,
                    // If it overflowed at any point this is set
                    first_operation_overflow || second_operation_overflow,
                );

                self.registers.flags.set(
                    FlagRegister::Carry,
                    first_operation_overflow || second_operation_overflow,
                );

                self.registers.flags.set(
                    FlagRegister::Negative,
                    // Check would be sign value
                    second_operation_result.view_bits::<Lsb0>()[7],
                );

                self.registers.flags.set(
                    FlagRegister::Zero,
                    // Check would be carry value
                    second_operation_result == 0,
                );

                self.registers.accumulator = second_operation_result;
            }
            M6502InstructionSetSpecifier::Anc => {
                let value =
                    load_m6502_addressing_modes!(instruction, self.registers, memory, [Immediate]);

                let new_value = self.registers.accumulator & value;

                self.registers
                    .flags
                    .set(FlagRegister::Negative, new_value.view_bits::<Lsb0>()[7]);

                self.registers
                    .flags
                    .set(FlagRegister::Carry, new_value.view_bits::<Lsb0>()[7]);

                self.registers.flags.set(FlagRegister::Zero, new_value == 0);

                self.registers.accumulator = new_value;
            }
            M6502InstructionSetSpecifier::And => {
                let value = load_m6502_addressing_modes!(
                    instruction,
                    self.registers,
                    memory,
                    [
                        Immediate,
                        Absolute,
                        XIndexedAbsolute,
                        YIndexedAbsolute,
                        ZeroPage,
                        XIndexedZeroPage,
                        XIndexedZeroPageIndirect,
                        ZeroPageIndirectYIndexed
                    ]
                );

                let new_value = self.registers.accumulator & value;

                self.registers
                    .flags
                    .set(FlagRegister::Negative, new_value.view_bits::<Lsb0>()[7]);

                self.registers.flags.set(FlagRegister::Zero, new_value == 0);

                self.registers.accumulator = new_value;
            }
            M6502InstructionSetSpecifier::Arr => todo!(),
            M6502InstructionSetSpecifier::Asl => todo!(),
            M6502InstructionSetSpecifier::Asr => todo!(),
            M6502InstructionSetSpecifier::Bcc => {
                let value = match instruction.addressing_mode {
                    Some(AddressingMode::Relative(value)) => value,
                    _ => unreachable!(),
                };

                if !self.registers.flags.contains(FlagRegister::Carry) {
                    *program_pointer = program_pointer.wrapping_add_signed(value as isize);
                }
            }
            M6502InstructionSetSpecifier::Bcs => {
                let value = match instruction.addressing_mode {
                    Some(AddressingMode::Relative(value)) => value,
                    _ => unreachable!(),
                };

                if self.registers.flags.contains(FlagRegister::Carry) {
                    *program_pointer = program_pointer.wrapping_add_signed(value as isize);
                }
            }
            M6502InstructionSetSpecifier::Beq => {
                let value = match instruction.addressing_mode {
                    Some(AddressingMode::Relative(value)) => value,
                    _ => unreachable!(),
                };

                if self.registers.flags.contains(FlagRegister::Zero) {
                    *program_pointer = program_pointer.wrapping_add_signed(value as isize);
                }
            }
            M6502InstructionSetSpecifier::Bit => todo!(),
            M6502InstructionSetSpecifier::Bmi => {
                let value = match instruction.addressing_mode {
                    Some(AddressingMode::Relative(value)) => value,
                    _ => unreachable!(),
                };

                if self.registers.flags.contains(FlagRegister::Negative) {
                    *program_pointer = program_pointer.wrapping_add_signed(value as isize);
                }
            }
            M6502InstructionSetSpecifier::Bne => {
                let value = match instruction.addressing_mode {
                    Some(AddressingMode::Relative(value)) => value,
                    _ => unreachable!(),
                };

                if !self.registers.flags.contains(FlagRegister::Zero) {
                    *program_pointer = program_pointer.wrapping_add_signed(value as isize);
                }
            }
            M6502InstructionSetSpecifier::Bpl => {
                let value = match instruction.addressing_mode {
                    Some(AddressingMode::Relative(value)) => value,
                    _ => unreachable!(),
                };

                if !self.registers.flags.contains(FlagRegister::Negative) {
                    *program_pointer = program_pointer.wrapping_add_signed(value as isize);
                }
            }
            M6502InstructionSetSpecifier::Brk => todo!(),
            M6502InstructionSetSpecifier::Bvc => {
                let value = match instruction.addressing_mode {
                    Some(AddressingMode::Relative(value)) => value,
                    _ => unreachable!(),
                };

                if !self.registers.flags.contains(FlagRegister::Overflow) {
                    *program_pointer = program_pointer.wrapping_add_signed(value as isize);
                }
            }
            M6502InstructionSetSpecifier::Bvs => {
                let value = match instruction.addressing_mode {
                    Some(AddressingMode::Relative(value)) => value,
                    _ => unreachable!(),
                };

                if self.registers.flags.contains(FlagRegister::Overflow) {
                    *program_pointer = program_pointer.wrapping_add_signed(value as isize);
                }
            }
            M6502InstructionSetSpecifier::Clc => {
                self.registers.flags.remove(FlagRegister::Carry);
            }
            M6502InstructionSetSpecifier::Cld => {
                self.registers.flags.remove(FlagRegister::Decimal);
            }
            M6502InstructionSetSpecifier::Cli => {
                self.registers.flags.remove(FlagRegister::InterruptDisable);
            }
            M6502InstructionSetSpecifier::Clv => {
                self.registers.flags.remove(FlagRegister::Overflow);
            }
            M6502InstructionSetSpecifier::Cmp => todo!(),
            M6502InstructionSetSpecifier::Cpx => todo!(),
            M6502InstructionSetSpecifier::Cpy => todo!(),
            M6502InstructionSetSpecifier::Dcp => todo!(),
            M6502InstructionSetSpecifier::Dec => todo!(),
            M6502InstructionSetSpecifier::Dex => todo!(),
            M6502InstructionSetSpecifier::Dey => todo!(),
            M6502InstructionSetSpecifier::Eor => todo!(),
            M6502InstructionSetSpecifier::Inc => todo!(),
            M6502InstructionSetSpecifier::Inx => todo!(),
            M6502InstructionSetSpecifier::Iny => todo!(),
            M6502InstructionSetSpecifier::Isc => todo!(),
            M6502InstructionSetSpecifier::Jam => todo!(),
            M6502InstructionSetSpecifier::Jmp => todo!(),
            M6502InstructionSetSpecifier::Jsr => todo!(),
            M6502InstructionSetSpecifier::Las => todo!(),
            M6502InstructionSetSpecifier::Lax => todo!(),
            M6502InstructionSetSpecifier::Lda => todo!(),
            M6502InstructionSetSpecifier::Ldx => todo!(),
            M6502InstructionSetSpecifier::Ldy => todo!(),
            M6502InstructionSetSpecifier::Lsr => todo!(),
            M6502InstructionSetSpecifier::Nop => todo!(),
            M6502InstructionSetSpecifier::Ora => {
                let value = load_m6502_addressing_modes!(
                    instruction,
                    self.registers,
                    memory,
                    [
                        Immediate,
                        Absolute,
                        XIndexedAbsolute,
                        YIndexedAbsolute,
                        ZeroPage,
                        XIndexedZeroPage,
                        XIndexedZeroPageIndirect,
                        ZeroPageIndirectYIndexed
                    ]
                );

                let new_value = self.registers.accumulator | value;

                self.registers
                    .flags
                    .set(FlagRegister::Negative, new_value.view_bits::<Lsb0>()[7]);

                self.registers.flags.set(FlagRegister::Zero, new_value == 0);

                self.registers.accumulator = new_value;
            }
            M6502InstructionSetSpecifier::Pha => {
                memory.write(
                    self.registers.stack_pointer as usize,
                    &self.registers.accumulator.to_le_bytes(),
                );

                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
            }
            M6502InstructionSetSpecifier::Php => {
                // https://www.nesdev.org/wiki/Status_flags

                let mut flags = self.registers.flags;
                flags.insert(FlagRegister::__Unused);

                memory.write(
                    self.registers.stack_pointer as usize,
                    &flags.bits().to_be_bytes(),
                );

                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
            }
            M6502InstructionSetSpecifier::Pla => {
                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);

                let mut value = 0;

                memory.read(
                    self.registers.stack_pointer as usize,
                    std::array::from_mut(&mut value),
                );

                self.registers.accumulator = value;
            }
            M6502InstructionSetSpecifier::Plp => {
                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);

                let mut value = 0;

                memory.read(
                    self.registers.stack_pointer as usize,
                    std::array::from_mut(&mut value),
                );

                self.registers.flags = FlagRegister::from_bits(value).unwrap();
            }
            M6502InstructionSetSpecifier::Rla => todo!(),
            M6502InstructionSetSpecifier::Rol => todo!(),
            M6502InstructionSetSpecifier::Ror => todo!(),
            M6502InstructionSetSpecifier::Rra => todo!(),
            M6502InstructionSetSpecifier::Rti => todo!(),
            M6502InstructionSetSpecifier::Rts => todo!(),
            M6502InstructionSetSpecifier::Sax => todo!(),
            M6502InstructionSetSpecifier::Sbc => todo!(),
            M6502InstructionSetSpecifier::Sbx => todo!(),
            M6502InstructionSetSpecifier::Sec => {
                self.registers.flags.insert(FlagRegister::Carry);
            }
            M6502InstructionSetSpecifier::Sed => {
                self.registers.flags.insert(FlagRegister::Decimal);
            }
            M6502InstructionSetSpecifier::Sei => {
                self.registers.flags.insert(FlagRegister::InterruptDisable);
            }
            M6502InstructionSetSpecifier::Sha => todo!(),
            M6502InstructionSetSpecifier::Shs => todo!(),
            M6502InstructionSetSpecifier::Shx => todo!(),
            M6502InstructionSetSpecifier::Shy => todo!(),
            M6502InstructionSetSpecifier::Slo => todo!(),
            M6502InstructionSetSpecifier::Sre => todo!(),
            M6502InstructionSetSpecifier::Sta => todo!(),
            M6502InstructionSetSpecifier::Stx => todo!(),
            M6502InstructionSetSpecifier::Sty => todo!(),
            M6502InstructionSetSpecifier::Tax => todo!(),
            M6502InstructionSetSpecifier::Tay => todo!(),
            M6502InstructionSetSpecifier::Tsx => todo!(),
            M6502InstructionSetSpecifier::Txa => todo!(),
            M6502InstructionSetSpecifier::Txs => todo!(),
            M6502InstructionSetSpecifier::Tya => todo!(),
            M6502InstructionSetSpecifier::Xaa => {
                let value =
                    load_m6502_addressing_modes!(instruction, self.registers, memory, [Immediate]);
            }
        }

        Ok(())
    }
}

impl SchedulableComponent for M6502 {
    fn tick_rate(&self) -> Ratio<u32> {
        todo!()
    }

    fn run(&mut self, period: u32) {
        todo!()
    }
}
