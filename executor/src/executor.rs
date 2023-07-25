use core::fmt::Debug;

use tangram_instruction::{Instruction, MemoryMut};

use crate::{AsyncBytecodeReader, BytecodeReader, Error, Monitor};

/// VM Executor
pub struct Executor<const RS: usize, I, R, M, MM>
where
    I: Instruction,
{
    pc: I::Register,
    regs: [I::Register; RS],
    reader: R,
    memory: M,
    monitor: MM,
}

impl<const RS: usize, I, R, M, MM> Executor<RS, I, R, M, MM>
where
    I: Instruction,
    I::Register: Default + Clone + Copy,
{
    pub fn new(reader: R, memory: M, monitor: MM) -> Self {
        let pc = I::Register::default();
        let regs = [I::Register::default(); RS];

        Self {
            pc,
            regs,
            memory,
            reader,
            monitor,
        }
    }
}

impl<const RS: usize, I, R, M, MM, E> Executor<RS, I, R, M, MM>
where
    I: Instruction,
    I::Register: Default + Clone + Copy,
    R: BytecodeReader<Register = I::Register, Error = E>,
    E: Debug,
    M: MemoryMut<Register = I::Register>,
    MM: Monitor<I>,
{
    pub fn run(&mut self, bytes_len: u8) -> Result<(), Error<E>> {
        loop {
            let bytes = self
                .reader
                .read(&self.pc, bytes_len)
                .map_err(Error::AppError)?;

            let mut inst = I::new(bytes)?;

            inst.execute(&mut self.pc, &mut self.regs, &mut self.memory)?;
            self.monitor
                .monitor(&inst, &self.pc, &self.regs, &self.memory);
        }
    }
}

impl<const RS: usize, I, R, M, MM, E> Executor<RS, I, R, M, MM>
where
    I: Instruction,
    I::Register: Default + Clone + Copy,
    R: AsyncBytecodeReader<Register = I::Register, Error = E>,
    E: Debug,
    M: MemoryMut<Register = I::Register>,
    MM: Monitor<I>,
{
    pub async fn async_run(&mut self, bytes_len: u8) -> Result<(), Error<E>> {
        loop {
            let bytes = self
                .reader
                .read(&self.pc, bytes_len)
                .await
                .map_err(Error::AppError)?;

            let mut inst = I::new(bytes)?;

            inst.execute(&mut self.pc, &mut self.regs, &mut self.memory)?;
            self.monitor
                .monitor(&inst, &self.pc, &self.regs, &self.memory);
        }
    }
}
