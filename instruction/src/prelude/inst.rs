use crate::{Error, Memory, MemoryMut, Result};

/// Instruction
pub trait Instruction: Sized {
    /// Register type, usually u32 or u64
    type Register;

    fn new(bytes: &[u8]) -> Result<Self>;

    /// Execute an anstruction.
    fn execute<M>(
        &mut self,
        pc: &mut Self::Register,
        regs: &mut [Self::Register],
        memory: &mut M,
    ) -> Result<()>
    where
        M: Memory<Register = Self::Register> + MemoryMut;
}

impl Instruction for () {
    type Register = u32;

    fn new(_bytes: &[u8]) -> Result<Self> {
        Ok(())
    }

    fn execute<M>(
        &mut self,
        _pc: &mut Self::Register,
        _regs: &mut [Self::Register],
        _memory: &mut M,
    ) -> Result<()>
    where
        M: Memory<Register = Self::Register> + MemoryMut,
    {
        Err(Error::ErrFailedDeocdeInstructon)
    }
}
