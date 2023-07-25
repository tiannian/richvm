use crate::{riscv::Inst, Error, Instruction, Memory, MemoryMut, Reg32};

/// Instruction for Env call for RISCV32i
pub struct RV32iEnvInst<I> {
    inst: Inst,
    sub: I,
}

impl<I, R> Instruction for RV32iEnvInst<I>
where
    I: Instruction<Register = R>,
    R: Reg32,
{
    type Register = R;

    fn new(bytes: &[u8]) -> crate::Result<Self> {
        if bytes.len() < 4 {
            return Err(Error::ErrBytecodeLengthNotEnough);
        }

        let inst = Inst::new([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let sub = I::new(bytes)?;

        Ok(Self { inst, sub })
    }

    fn execute<M>(
        &mut self,
        pc: &mut Self::Register,
        regs: &mut [Self::Register],
        memory: &mut M,
    ) -> crate::Result<()>
    where
        M: Memory<Register = R> + MemoryMut,
    {
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
                self.sub.execute(pc, regs, memory)
            }
        } else {
            self.sub.execute(pc, regs, memory)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Error, Instruction};

    use super::RV32iEnvInst;

    #[test]
    fn test_a() {
        let mut memory = [0u8; 32];
        let mut pc = 0u32;
        let mut regs = [0u32; 32];

        let code = [0u8; 4];
        let mut base = RV32iEnvInst::<()>::new(&code).unwrap();
        let r = base.execute(&mut pc, &mut regs, &mut memory);

        assert_eq!(r, Err(Error::ErrFailedDeocdeInstructon))
    }
}
