use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmulatorError {
    MemoryOutOfBounds { address: u32, width: usize },
    Unaligned { address: u32, width: usize },
    IllegalInstruction(u32),
    StepLimitExceeded(usize),
}

impl fmt::Display for EmulatorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for EmulatorError {}
