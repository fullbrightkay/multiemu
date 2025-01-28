use super::{
    instruction::{M6502InstructionSet, M6502InstructionSetSpecifier},
    FlagRegister, ProcessorState, M6502,
};
use crate::definitions::misc::processor::m6502::instruction::AddressingMode;
use bitvec::{order::Lsb0, view::BitView};
use enumflags2::BitFlag;

// NOTE: The M6502 should ignore all memory errors

macro_rules! load_m6502_addressing_modes {
    ($instruction:expr, $register_store:expr, $memory_translation_table:expr, $assigned_address_space:expr, [$($modes:ident),*]) => {{
        match $instruction.addressing_mode {
            $(
                Some(AddressingMode::$modes(argument)) => {
                    load_m6502_addressing_modes!(@handler $modes, argument, $register_store, $memory_translation_table, $assigned_address_space)
                },
            )*
            _ => unreachable!(),
        }
    }};

    (@handler Immediate, $argument:expr, $register_store:expr, $memory_translation_table:expr, $assigned_address_space:expr) => {{
        $argument
    }};

    (@handler Absolute, $argument:expr, $register_store:expr, $memory_translation_table:expr, $assigned_address_space:expr) => {{
        let mut value = 0;

        let _ = $memory_translation_table
            .read($argument as usize, bytemuck::bytes_of_mut(&mut value), $assigned_address_space);

        value
    }};

    (@handler XIndexedAbsolute, $argument:expr, $register_store:expr, $memory_translation_table:expr, $assigned_address_space:expr) => {{
        let mut value = 0;

        let _ = $memory_translation_table
            .read($argument as usize, &mut [0], $assigned_address_space);


        let actual_address = $argument.wrapping_add($register_store.index_registers[0] as u16);
        let _ = $memory_translation_table
            .read(actual_address as usize, bytemuck::bytes_of_mut(&mut value), $assigned_address_space);

        value
    }};

    (@handler YIndexedAbsolute, $argument:expr, $register_store:expr, $memory_translation_table:expr, $assigned_address_space:expr) => {{
        let mut value: u8 = 0;

        let _ = $memory_translation_table
            .read($argument as usize, &mut [0], $assigned_address_space);


        let actual_address = $argument.wrapping_add($register_store.index_registers[1] as u16);
        let _ = $memory_translation_table
            .read(actual_address as usize, bytemuck::bytes_of_mut(&mut value), $assigned_address_space);

        value
    }};

    (@handler ZeroPage, $argument:expr, $register_store:expr, $memory_translation_table:expr, $assigned_address_space:expr) => {{
        let mut value: u8 = 0;

        let _ = $memory_translation_table
            .read($argument as usize, bytemuck::bytes_of_mut(&mut value), $assigned_address_space);

        value
    }};

    (@handler XIndexedZeroPage, $argument:expr, $register_store:expr, $memory_translation_table:expr, $assigned_address_space:expr) => {{
        let mut value: u8 = 0;

        let actual_address = $argument.wrapping_add($register_store.index_registers[0]);

        let _ = $memory_translation_table
            .read(actual_address as usize, bytemuck::bytes_of_mut(&mut value), $assigned_address_space);

        value
    }};

    (@handler YIndexedZeroPage, $argument:expr, $register_store:expr, $memory_translation_table:expr, $assigned_address_space:expr) => {{
        let mut value: u8 = 0;

        let actual_address = $argument.wrapping_add($register_store.index_registers[1]);

        let _ = $memory_translation_table
            .read(actual_address as usize, bytemuck::bytes_of_mut(&mut value), $assigned_address_space);

        value
    }};

    (@handler XIndexedZeroPageIndirect, $argument:expr, $register_store:expr, $memory_translation_table:expr, $assigned_address_space:expr) => {{
        let mut value: u8 = 0;

        let indirection_address = $argument.wrapping_add($register_store.index_registers[0]);
        let mut actual_address = [0; 2];

        let _ = $memory_translation_table
            .read(indirection_address as usize, &mut actual_address, $assigned_address_space);

        let actual_address = u16::from_le_bytes(actual_address);

        let _ = $memory_translation_table
            .read(actual_address as usize, bytemuck::bytes_of_mut(&mut value), $assigned_address_space);

        value
    }};

    (@handler ZeroPageIndirectYIndexed, $argument:expr, $register_store:expr, $memory_translation_table:expr, $assigned_address_space:expr) => {{
        let mut value: u8 = 0;
        let mut indirection_address: u8= 0;

        let _ = $memory_translation_table
            .read($argument as usize, bytemuck::bytes_of_mut(&mut indirection_address), $assigned_address_space);

        let indirection_address = (indirection_address as u16)
            .wrapping_add($register_store.index_registers[1] as u16);

        let _ = $memory_translation_table
            .read(indirection_address as usize, bytemuck::bytes_of_mut(&mut value), $assigned_address_space);

        value
    }};
}

impl M6502 {
    pub(super) fn interpret_instruction(
        &self,
        state: &mut ProcessorState,
        instruction: M6502InstructionSet,
    ) {
        let memory_translation_table = self.memory_translation_table.get().unwrap();

        match instruction.specifier {
            M6502InstructionSetSpecifier::Adc => {
                let value = load_m6502_addressing_modes!(
                    instruction,
                    state.registers,
                    memory_translation_table,
                    self.config.assigned_address_space,
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

                let carry_value = state.registers.flags.contains(FlagRegister::Carry) as u8;

                let (first_operation_result, first_operation_overflow) =
                    state.registers.accumulator.overflowing_add(value);

                let (second_operation_result, second_operation_overflow) =
                    first_operation_result.overflowing_add(carry_value);

                state.registers.flags.set(
                    FlagRegister::Overflow,
                    // If it overflowed at any point this is set
                    first_operation_overflow || second_operation_overflow,
                );

                state.registers.flags.set(
                    FlagRegister::Carry,
                    first_operation_overflow || second_operation_overflow,
                );

                state.registers.flags.set(
                    FlagRegister::Negative,
                    // Check would be sign value
                    second_operation_result.view_bits::<Lsb0>()[7],
                );

                state.registers.flags.set(
                    FlagRegister::Zero,
                    // Check would be carry value
                    second_operation_result == 0,
                );

                state.registers.accumulator = second_operation_result;
            }
            M6502InstructionSetSpecifier::Anc => {
                let value = load_m6502_addressing_modes!(
                    instruction,
                    state.registers,
                    memory_translation_table,
                    self.config.assigned_address_space,
                    [Immediate]
                );

                let new_value = state.registers.accumulator & value;

                state
                    .registers
                    .flags
                    .set(FlagRegister::Negative, new_value.view_bits::<Lsb0>()[7]);

                state
                    .registers
                    .flags
                    .set(FlagRegister::Carry, new_value.view_bits::<Lsb0>()[7]);

                state
                    .registers
                    .flags
                    .set(FlagRegister::Zero, new_value == 0);

                state.registers.accumulator = new_value;
            }
            M6502InstructionSetSpecifier::And => {
                let value = load_m6502_addressing_modes!(
                    instruction,
                    state.registers,
                    memory_translation_table,
                    self.config.assigned_address_space,
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

                let new_value = state.registers.accumulator & value;

                state
                    .registers
                    .flags
                    .set(FlagRegister::Negative, new_value.view_bits::<Lsb0>()[7]);

                state
                    .registers
                    .flags
                    .set(FlagRegister::Zero, new_value == 0);

                state.registers.accumulator = new_value;
            }
            M6502InstructionSetSpecifier::Arr => todo!(),
            M6502InstructionSetSpecifier::Asl => todo!(),
            M6502InstructionSetSpecifier::Asr => todo!(),
            M6502InstructionSetSpecifier::Bcc => {
                let value = match instruction.addressing_mode {
                    Some(AddressingMode::Relative(value)) => value,
                    _ => unreachable!(),
                };

                if !state.registers.flags.contains(FlagRegister::Carry) {
                    state.registers.program =
                        state.registers.program.wrapping_add_signed(value as i16);
                }
            }
            M6502InstructionSetSpecifier::Bcs => {
                let value = match instruction.addressing_mode {
                    Some(AddressingMode::Relative(value)) => value,
                    _ => unreachable!(),
                };

                if state.registers.flags.contains(FlagRegister::Carry) {
                    state.registers.program =
                        state.registers.program.wrapping_add_signed(value as i16);
                }
            }
            M6502InstructionSetSpecifier::Beq => {
                let value = match instruction.addressing_mode {
                    Some(AddressingMode::Relative(value)) => value,
                    _ => unreachable!(),
                };

                if state.registers.flags.contains(FlagRegister::Zero) {
                    state.registers.program =
                        state.registers.program.wrapping_add_signed(value as i16);
                }
            }
            M6502InstructionSetSpecifier::Bit => todo!(),
            M6502InstructionSetSpecifier::Bmi => {
                let value = match instruction.addressing_mode {
                    Some(AddressingMode::Relative(value)) => value,
                    _ => unreachable!(),
                };

                if state.registers.flags.contains(FlagRegister::Negative) {
                    state.registers.program =
                        state.registers.program.wrapping_add_signed(value as i16);
                }
            }
            M6502InstructionSetSpecifier::Bne => {
                let value = match instruction.addressing_mode {
                    Some(AddressingMode::Relative(value)) => value,
                    _ => unreachable!(),
                };

                if !state.registers.flags.contains(FlagRegister::Zero) {
                    state.registers.program =
                        state.registers.program.wrapping_add_signed(value as i16);
                }
            }
            M6502InstructionSetSpecifier::Bpl => {
                let value = match instruction.addressing_mode {
                    Some(AddressingMode::Relative(value)) => value,
                    _ => unreachable!(),
                };

                if !state.registers.flags.contains(FlagRegister::Negative) {
                    state.registers.program =
                        state.registers.program.wrapping_add_signed(value as i16);
                }
            }
            M6502InstructionSetSpecifier::Brk => todo!(),
            M6502InstructionSetSpecifier::Bvc => {
                let value = match instruction.addressing_mode {
                    Some(AddressingMode::Relative(value)) => value,
                    _ => unreachable!(),
                };

                if !state.registers.flags.contains(FlagRegister::Overflow) {
                    state.registers.program =
                        state.registers.program.wrapping_add_signed(value as i16);
                }
            }
            M6502InstructionSetSpecifier::Bvs => {
                let value = match instruction.addressing_mode {
                    Some(AddressingMode::Relative(value)) => value,
                    _ => unreachable!(),
                };

                if state.registers.flags.contains(FlagRegister::Overflow) {
                    state.registers.program =
                        state.registers.program.wrapping_add_signed(value as i16);
                }
            }
            M6502InstructionSetSpecifier::Clc => {
                state.registers.flags.remove(FlagRegister::Carry);
            }
            M6502InstructionSetSpecifier::Cld => {
                state.registers.flags.remove(FlagRegister::Decimal);
            }
            M6502InstructionSetSpecifier::Cli => {
                state.registers.flags.remove(FlagRegister::InterruptDisable);
            }
            M6502InstructionSetSpecifier::Clv => {
                state.registers.flags.remove(FlagRegister::Overflow);
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
                    state.registers,
                    memory_translation_table,
                    self.config.assigned_address_space,
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

                let new_value = state.registers.accumulator | value;

                state
                    .registers
                    .flags
                    .set(FlagRegister::Negative, new_value.view_bits::<Lsb0>()[7]);

                state
                    .registers
                    .flags
                    .set(FlagRegister::Zero, new_value == 0);

                state.registers.accumulator = new_value;
            }
            M6502InstructionSetSpecifier::Pha => {
                let _ = memory_translation_table.write(
                    state.registers.stack_pointer as usize,
                    &state.registers.accumulator.to_le_bytes(),
                    self.config.assigned_address_space,
                );

                state.registers.stack_pointer = state.registers.stack_pointer.wrapping_sub(1);
            }
            M6502InstructionSetSpecifier::Php => {
                // https://www.nesdev.org/wiki/Status_flags

                let mut flags = state.registers.flags;
                flags.insert(FlagRegister::__Unused);

                let _ = memory_translation_table.write(
                    state.registers.stack_pointer as usize,
                    &flags.bits().to_be_bytes(),
                    self.config.assigned_address_space,
                );

                state.registers.stack_pointer = state.registers.stack_pointer.wrapping_sub(1);
            }
            M6502InstructionSetSpecifier::Pla => {
                state.registers.stack_pointer = state.registers.stack_pointer.wrapping_add(1);

                let mut value = 0;

                let _ = memory_translation_table.read(
                    state.registers.stack_pointer as usize,
                    std::array::from_mut(&mut value),
                    self.config.assigned_address_space,
                );

                state.registers.accumulator = value;
            }
            M6502InstructionSetSpecifier::Plp => {
                state.registers.stack_pointer = state.registers.stack_pointer.wrapping_add(1);

                let mut value = 0;

                let _ = memory_translation_table.read(
                    state.registers.stack_pointer as usize,
                    std::array::from_mut(&mut value),
                    self.config.assigned_address_space,
                );

                state.registers.flags = FlagRegister::from_bits(value).unwrap();
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
                state.registers.flags.insert(FlagRegister::Carry);
            }
            M6502InstructionSetSpecifier::Sed => {
                state.registers.flags.insert(FlagRegister::Decimal);
            }
            M6502InstructionSetSpecifier::Sei => {
                state.registers.flags.insert(FlagRegister::InterruptDisable);
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
                let value = load_m6502_addressing_modes!(
                    instruction,
                    state.registers,
                    memory_translation_table,
                    self.config.assigned_address_space,
                    [Immediate]
                );
            }
        }
    }
}
