use core::fmt::Debug;

use tangram_instruction::{Instruction, MemoryMut};

use crate::{BytecodeReader, Error};

pub struct Executor<const RS: usize, I, R, M>
where
    I: Instruction,
{
    pc: I::Register,
    regs: [I::Register; RS],
    reader: R,
    memory: M,
}

impl<const RS: usize, I, R, M, E> Executor<RS, I, R, M>
where
    I: Instruction,
    I::Register: Default + Clone + Copy,
    R: BytecodeReader<Register = I::Register, Error = E>,
    E: Debug,
    M: MemoryMut<Register = I::Register>,
{
    pub fn new(reader: R, memory: M) -> Self {
        let pc = I::Register::default();
        let regs = [I::Register::default(); RS];

        Self {
            pc,
            regs,
            memory,
            reader,
        }
    }

    pub fn run(&mut self, bytes_len: u8) -> Result<(), Error<E>> {
        loop {
            let bytes = self
                .reader
                .read(&self.pc, bytes_len)
                .map_err(Error::AppError)?;

            let mut inst = I::new(bytes)?;

            inst.execute(&mut self.pc, &mut self.regs, &mut self.memory)?;
        }
    }
}
