use crate::{riscv::Inst, Error, Instruction, Reg32};

pub struct RiscV32iLiteInstruction<I> {
    inst: Inst,
    sub: I,
}

impl<I> RiscV32iLiteInstruction<I> {
    pub fn new(i: [u8; 4], sub: I) -> Self {
        Self {
            inst: Inst::new(i),
            sub,
        }
    }
}

impl<I> Instruction for RiscV32iLiteInstruction<I>
where
    I: Instruction,
    I::Register: Reg32,
{
    const REGISTER_NUMBER: usize = 32;

    type Register = I::Register;

    fn execute(self, pc: &mut Self::Register, regs: &mut [Self::Register]) -> crate::Result<()> {
        let opcode = self.inst.opcode();

        if opcode == 0b1110011 {
            let rd = self.inst.rd();
            let funct3 = self.inst.funct3();
            let rs1 = self.inst.rs1();
            let imm = self.inst.imm_i();

            if rd == 0 && funct3 == 0 && rs1 == 0 && imm == 0 {
                Err(Error::EnvironmentCall)
            } else if rd == 0 && funct3 == 0 && rs1 == 0 && imm == 1 {
                Err(Error::Breakpoint)
            } else {
                self.sub.execute(pc, regs)
            }
        } else {
            self.sub.execute(pc, regs)
        }
    }
}
