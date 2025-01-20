use std::fmt::Debug;
use std::{borrow::Cow, fmt::Display};
use thiserror::Error;

/// The result of compiling an instruction was not ok
#[derive(Error, Debug)]
pub enum InstructionDecompilingError {
    #[error("The instruction could not be decompiled: {0:x?}")]
    InstructionDecompilingFailed(Vec<u8>),
}

#[derive(Debug)]
pub struct InstructionTextRepresentation {
    pub instruction_mnemonic: Cow<'static, str>,
}

impl Display for InstructionTextRepresentation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.instruction_mnemonic)
    }
}

pub trait InstructionSet: Debug + Sized {
    fn to_text_representation(&self) -> InstructionTextRepresentation;
}
