use crate::{Error, Result};

/// Instruction
pub trait Instruction {
    /// Number of Register
    const REGISTER_NUMBER: usize;

    /// Register type, usually u32 or u64
    type Register;

    /// Execute an anstruction.
    fn execute(self, pc: &mut Self::Register, regs: &mut [Self::Register]) -> Result<()>;
}

impl Instruction for () {
    const REGISTER_NUMBER: usize = 0;

    type Register = u32;

    fn execute(self, _pc: &mut Self::Register, _regs: &mut [Self::Register]) -> Result<()> {
        Err(Error::FailedDeocdeInstructon)
    }
}
