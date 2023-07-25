use tangram_instruction::{Instruction, MemoryMut};

pub trait Monitor<I: Instruction> {
    fn monitor<M>(&mut self, inst: &I, pc: &I::Register, regs: &[I::Register], memory: &M)
    where
        M: MemoryMut<Register = I::Register>;
}
