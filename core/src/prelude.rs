use crate::Result;

pub trait Instruction<M> {
    const REGISTER_NUMBER: usize;

    type Register: Default + Copy;

    fn execute(self, pc: &mut Self::Register, regs: &mut [Self::Register]) -> Result<()>;
}

pub trait Memory {
    type Register;

    fn length(&self) -> Self::Register;

    fn load(&self, pos: Self::Register, length: u8) -> &[u8];
}
