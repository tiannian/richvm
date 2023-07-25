use crate::{
    prelude::{Instruction, Memory},
    riscv::Inst,
    Error, MemoryMut, Reg32, Result,
};

/// Instruction for base of RiscVi32
///
/// These instruction have no CSR and FENCE include
pub struct RV32iBaseInst<I> {
    inst: Inst,
    sub: I,
}

fn clear_x0<R: Reg32>(regs: &mut [R]) {
    regs[0].set_reg32(0);
}

fn next_inst<R: Reg32>(pc: &mut R) {
    pc.add_symbol32(4)
}

impl<I, R> RV32iBaseInst<I>
where
    I: Instruction<Register = R>,
    R: Reg32 + Clone,
{
    /// LUI instruction
    pub fn lui(&mut self, pc: &mut R, regs: &mut [R]) {
        let inst = &self.inst;

        regs[inst.rd()].set_reg32(inst.imm_u());

        clear_x0(regs);
        next_inst(pc);
    }

    /// AUIPC instruction
    pub fn auipc(&mut self, pc: &mut R, regs: &mut [R]) {
        let inst = &self.inst;

        regs[inst.rd()].set_reg32(pc.reg32() + inst.imm_u());

        clear_x0(regs);
        next_inst(pc);
    }

    /// JAL instruction
    pub fn jal(&mut self, pc: &mut R, regs: &mut [R]) {
        let inst = &self.inst;

        regs[inst.rd()].set_reg32(pc.reg32() + 4);
        pc.add_symbol32(inst.imm_uj_symbol());
        // TODO: Jump check

        clear_x0(regs);
    }

    /// JALR instruction
    pub fn jalr(&mut self, pc: &mut R, regs: &mut [R]) {
        let inst = &self.inst;

        regs[inst.rd()].set_reg32(pc.reg32() + 4);

        let r = regs[inst.rs1()].symbol32() + inst.imm_i_symbol();

        pc.set_symbol32(r & (!1));
    }

    /// BEQ, BNE, BLT, BGE, BLTU, BLGE instruction
    pub fn bset<M>(&mut self, pc: &mut R, regs: &mut [R], memory: &mut M) -> Result<()>
    where
        M: Memory<Register = R> + MemoryMut,
    {
        let inst = &self.inst;

        let res = match inst.funct3() {
            0b000 => regs[inst.rs1()].reg32() == regs[inst.rs2()].reg32(),
            0b001 => regs[inst.rs1()].reg32() != regs[inst.rs2()].reg32(),
            0b100 => regs[inst.rs1()].symbol32() < regs[inst.rs2()].symbol32(),
            0b101 => regs[inst.rs1()].symbol32() >= regs[inst.rs2()].symbol32(),
            0b110 => regs[inst.rs1()].reg32() < regs[inst.rs2()].reg32(),
            0b111 => regs[inst.rs1()].reg32() >= regs[inst.rs2()].reg32(),
            _ => return self.sub.execute(pc, regs, memory),
        };

        if res {
            pc.add_symbol32(inst.imm_sb_symbol());
        } else {
            next_inst(pc)
        }

        Ok(())
    }

    /// LB, LH, LW, LBU, LHU, LWU instruction
    pub fn lset<M>(&mut self, pc: &mut R, regs: &mut [R], memory: &mut M) -> Result<()>
    where
        M: Memory<Register = R> + MemoryMut,
    {
        let inst = &self.inst;
        let funct3 = inst.funct3();

        let mut offset = regs[inst.rs1()].clone();
        offset.add_symbol32(inst.imm_i_symbol());

        let m = memory.load(offset, 4);

        match funct3 {
            0b000 => regs[inst.rd()].set_symbol32(i32::from_le_bytes([0, 0, 0, m[0]]) >> 24),
            0b001 => regs[inst.rd()].set_symbol32(i32::from_le_bytes([0, 0, m[0], m[1]]) >> 24),
            0b010 => {
                regs[inst.rd()].set_symbol32(i32::from_le_bytes([m[0], m[1], m[2], m[3]]) >> 24)
            }
            0b100 => regs[inst.rd()].set_reg32(u32::from_le_bytes([0, 0, 0, m[0]])),
            0b101 => regs[inst.rd()].set_reg32(u32::from_le_bytes([0, 0, m[0], m[1]])),
            0b110 => regs[inst.rd()].set_reg32(u32::from_le_bytes([m[0], m[1], m[2], m[3]])),
            _ => return self.sub.execute(pc, regs, memory),
        };

        clear_x0(regs);
        next_inst(pc);

        Ok(())
    }

    /// SB, SH, SW instruction
    pub fn sset<M>(&mut self, pc: &mut R, regs: &mut [R], memory: &mut M) -> Result<()>
    where
        M: Memory<Register = R> + MemoryMut,
    {
        let inst = &self.inst;

        let mut offset = regs[inst.rs1()].clone();
        offset.add_symbol32(inst.imm_i_symbol());

        let data = regs[inst.rs2()].reg32().to_le_bytes();

        match inst.funct3() {
            0b000 => memory.store(offset, &data[0..1]),
            0b001 => memory.store(offset, &data[0..2]),
            0b010 => memory.store(offset, &data),
            _ => return self.sub.execute(pc, regs, memory),
        }

        clear_x0(regs);
        next_inst(pc);

        Ok(())
    }

    pub fn iset<M>(&mut self, pc: &mut R, regs: &mut [R], memory: &mut M) -> Result<()>
    where
        M: Memory<Register = R> + MemoryMut,
    {
        let inst = &self.inst;
        let imm = inst.imm_i_symbol();
        let rd = inst.rd();
        let rd_data = regs[rd].symbol32();

        let res = match inst.funct3() {
            0b000 => rd_data + imm,
            0b010 => {
                if rd_data < imm {
                    1i32
                } else {
                    0i32
                }
            }
            0b011 => {
                if regs[rd].reg32() < inst.imm_i() {
                    1
                } else {
                    0
                }
            }
            0b100 => rd_data ^ imm,
            0b110 => rd_data | imm,
            0b111 => rd_data & imm,
            0b001 => {
                let size = inst.imm_i() & 0x1F;
                rd_data << size
            }
            0b101 => {
                let t = inst.imm_i() & 0x400;
                let size = inst.imm_i() & 0x1F;
                if t == 0 {
                    (rd_data as u32 >> size) as i32
                } else {
                    rd_data >> size
                }
            }
            _ => return self.sub.execute(pc, regs, memory),
        };
        regs[rd].set_symbol32(res);

        clear_x0(regs);
        next_inst(pc);
        Ok(())
    }

    pub fn opset<M>(&mut self, pc: &mut R, regs: &mut [R], memory: &mut M) -> Result<()>
    where
        M: Memory<Register = R> + MemoryMut,
    {
        let inst = &self.inst;
        let funct3 = inst.funct3();

        let rs1 = inst.rs1();
        let rs2 = inst.rs2();
        let rd = inst.rd();

        match funct3 {
            0b000 => {
                if inst.funct7() == 0 {
                    regs[rd].set_reg32(regs[rs1].reg32() + regs[rs2].reg32())
                } else {
                    regs[rd].set_reg32(regs[rs1].reg32() - regs[rs2].reg32())
                }
            }
            0b001 => regs[rd].set_reg32(regs[rs1].reg32() << regs[rs2].reg32()),
            0b010 => {
                let r = if regs[rs1].symbol32() < regs[rs2].symbol32() {
                    1
                } else {
                    0
                };

                regs[rd].set_reg32(r);
            }
            0b011 => {
                let r = if regs[rs1].reg32() < regs[rs2].reg32() {
                    1
                } else {
                    0
                };

                regs[rd].set_reg32(r);
            }
            0b100 => regs[rd].set_reg32(regs[rs1].reg32() ^ regs[rs2].reg32()),
            0b101 => {
                if inst.funct7() == 0 {
                    regs[rd].set_reg32(regs[rs1].reg32() >> regs[rs2].reg32());
                } else {
                    regs[rd].set_symbol32(regs[rs1].symbol32() >> regs[rs2].symbol32());
                }
            }
            0b110 => regs[rd].set_reg32(regs[rs1].reg32() | regs[rs2].reg32()),
            0b111 => regs[rd].set_reg32(regs[rs1].reg32() & regs[rs2].reg32()),
            _ => return self.sub.execute(pc, regs, memory),
        };

        clear_x0(regs);
        next_inst(pc);

        Ok(())
    }
}

impl<I, R> Instruction for RV32iBaseInst<I>
where
    I: Instruction<Register = R>,
    R: Reg32 + Clone,
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

    fn execute<M>(&mut self, pc: &mut R, regs: &mut [R], memory: &mut M) -> Result<()>
    where
        M: Memory<Register = R> + MemoryMut,
    {
        let opcode = self.inst.opcode();

        match opcode {
            0b0110111 => self.lui(pc, regs),
            0b0010111 => self.auipc(pc, regs),
            0b1101111 => self.jal(pc, regs),
            0b1100111 => self.jalr(pc, regs),
            0b1100011 => self.bset(pc, regs, memory)?,
            0b0000011 => self.lset(pc, regs, memory)?,
            0b0100011 => self.sset(pc, regs, memory)?,
            0b0010011 => self.iset(pc, regs, memory)?,
            0b0110011 => self.opset(pc, regs, memory)?,
            _ => self.sub.execute(pc, regs, memory)?,
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{riscv32i::RV32iEnvInst, Instruction};

    use super::RV32iBaseInst;

    #[test]
    fn test_a() {
        let code = [0u8; 4];
        let _base = RV32iBaseInst::<RV32iEnvInst<()>>::new(&code);
    }
}
