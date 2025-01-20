use crate::{
    processor::{InstructionSet, InstructionTextRepresentation},
    scheduler::query::memory::MemoryMut,
};
use std::borrow::Cow;

// https://www.pagetable.com/c64ref/6502/?tab=2

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AddressingMode {
    Accumulator,
    Immediate(u8),
    Absolute(u16),
    XIndexedAbsolute(u16),
    YIndexedAbsolute(u16),
    AbsoluteIndirect(u16),
    ZeroPage(u8),
    XIndexedZeroPage(u8),
    YIndexedZeroPage(u8),
    ZeroPageYIndexed(u8),
    XIndexedZeroPageIndirect(u8),
    ZeroPageIndirectYIndexed(u8),
    Relative(i8),
}

impl AddressingMode {
    pub fn from_group1_addressing(
        addressing_mode_id: u8,
        memory: &MemoryMut,
        cursor: usize,
    ) -> (Self, u64, u8) {
        todo!()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum M6502InstructionSetSpecifier {
    Adc,
    Anc,
    And,
    Arr,
    Asl,
    Asr,
    Bcc,
    Bcs,
    Beq,
    Bit,
    Bmi,
    Bne,
    Bpl,
    Brk,
    Bvc,
    Bvs,
    Clc,
    Cld,
    Cli,
    Clv,
    Cmp,
    Cpx,
    Cpy,
    Dcp,
    Dec,
    Dex,
    Dey,
    Eor,
    Inc,
    Inx,
    Iny,
    Isc,
    Jam,
    Jmp,
    Jsr,
    Las,
    Lax,
    Lda,
    Ldx,
    Ldy,
    Lsr,
    Nop,
    Ora,
    Pha,
    Php,
    Pla,
    Plp,
    Rla,
    Rol,
    Ror,
    Rra,
    Rti,
    Rts,
    Sax,
    Sbc,
    Sbx,
    Sec,
    Sed,
    Sei,
    Sha,
    Shs,
    Shx,
    Shy,
    Slo,
    Sre,
    Sta,
    Stx,
    Sty,
    Tax,
    Tay,
    Tsx,
    Txa,
    Txs,
    Tya,
    Xaa,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct M6502InstructionSet {
    pub specifier: M6502InstructionSetSpecifier,
    pub addressing_mode: Option<AddressingMode>,
}

impl InstructionSet for M6502InstructionSet {
    fn to_text_representation(&self) -> InstructionTextRepresentation {
        InstructionTextRepresentation {
            instruction_mnemonic: Cow::Borrowed("TODO"),
        }
    }
}
