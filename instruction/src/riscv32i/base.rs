use core::marker::PhantomData;

use crate::{
    prelude::{Instruction, Memory},
    riscv::{Inst, RV32Register},
    Error, MemoryMut, Result,
};

/// Instruction for base of RiscVi32
///
/// These instruction have no CSR and FENCE include
pub struct RiscV32iBaseInstruction<'a, M, I> {
    inst: Inst,
    memory: &'a mut M,
    marker: PhantomData<I>,
}

fn clear_x0<R: RV32Register>(regs: &mut [R]) {
    regs[0].set_reg32(0);
}

fn next_inst<R: RV32Register>(pc: &mut R) {
    pc.add_symbol32(4)
}

impl<'a, M, I, R> RiscV32iBaseInstruction<'a, M, I>
where
    I: Instruction<Register = R>,
    R: RV32Register + Clone,
{
    /// Create instruction
    pub fn new(inst: [u8; 4], memory: &'a mut M) -> Self {
        Self {
            inst: Inst::new(inst),
            memory,
            marker: PhantomData,
        }
    }

    /// LUI instruction
    pub fn lui(self, pc: &mut R, regs: &mut [R]) {
        let inst = self.inst;

        regs[inst.rd()].set_reg32(inst.imm_u());

        clear_x0(regs);
        next_inst(pc);
    }

    /// AUIPC instruction
    pub fn auipc(self, pc: &mut R, regs: &mut [R]) {
        let inst = self.inst;

        regs[inst.rd()].set_reg32(pc.reg32() + inst.imm_u());

        clear_x0(regs);
        next_inst(pc);
    }

    /// JAL instruction
    pub fn jal(self, pc: &mut R, regs: &mut [R]) {
        let inst = self.inst;

        regs[inst.rd()].set_reg32(pc.reg32() + 4);
        pc.add_symbol32(inst.imm_uj_symbol());
        // TODO: Jump check

        clear_x0(regs);
    }

    /// JALR instruction
    pub fn jalr(self, pc: &mut R, regs: &mut [R]) {
        let inst = self.inst;

        regs[inst.rd()].set_reg32(pc.reg32() + 4);

        let r = regs[inst.rs1()].symbol32() + inst.imm_i_symbol();

        pc.set_symbol32(r & (!1));
    }

    /// BEQ, BNE, BLT, BGE, BLTU, BLGE instruction
    pub fn bset(self, pc: &mut R, regs: &[R]) -> Result<()> {
        let inst = self.inst;

        let res = match inst.funct3() {
            0b000 => regs[inst.rs1()].reg32() == regs[inst.rs2()].reg32(),
            0b001 => regs[inst.rs1()].reg32() != regs[inst.rs2()].reg32(),
            0b100 => regs[inst.rs1()].symbol32() < regs[inst.rs2()].symbol32(),
            0b101 => regs[inst.rs1()].symbol32() >= regs[inst.rs2()].symbol32(),
            0b110 => regs[inst.rs1()].reg32() < regs[inst.rs2()].reg32(),
            0b111 => regs[inst.rs1()].reg32() >= regs[inst.rs2()].reg32(),
            _ => return Err(Error::UnsupportFunct3),
        };

        if res {
            pc.add_symbol32(inst.imm_sb_symbol());
        } else {
            next_inst(pc)
        }

        Ok(())
    }

    /// LB, LH, LW, LBU, LHU, LWU instruction
    pub fn lset(self, pc: &mut R, regs: &mut [R]) -> Result<()>
    where
        M: Memory<Register = R>,
    {
        let inst = self.inst;
        let funct3 = inst.funct3();

        let mut offset = regs[inst.rs1()].clone();
        offset.add_symbol32(inst.imm_i_symbol());

        let m = self.memory.load(offset, 4);

        match funct3 {
            0b000 => regs[inst.rd()].set_symbol32(i32::from_le_bytes([0, 0, 0, m[0]]) >> 24),
            0b001 => regs[inst.rd()].set_symbol32(i32::from_le_bytes([0, 0, m[0], m[1]]) >> 24),
            0b010 => {
                regs[inst.rd()].set_symbol32(i32::from_le_bytes([m[0], m[1], m[2], m[3]]) >> 24)
            }
            0b100 => regs[inst.rd()].set_reg32(u32::from_le_bytes([0, 0, 0, m[0]])),
            0b101 => regs[inst.rd()].set_reg32(u32::from_le_bytes([0, 0, m[0], m[1]])),
            0b110 => regs[inst.rd()].set_reg32(u32::from_le_bytes([m[0], m[1], m[2], m[3]])),
            _ => return Err(Error::UnsupportFunct3),
        };

        clear_x0(regs);
        next_inst(pc);

        Ok(())
    }

    /// SB, SH, SW instruction
    pub fn sset(self, pc: &mut R, regs: &mut [R]) -> Result<()>
    where
        M: Memory<Register = R> + MemoryMut,
    {
        let inst = self.inst;

        let mut offset = regs[inst.rs1()].clone();
        offset.add_symbol32(inst.imm_i_symbol());

        let data = regs[inst.rs2()].reg32().to_le_bytes();

        match inst.funct3() {
            0b000 => self.memory.store(offset, &data[0..1]),
            0b001 => self.memory.store(offset, &data[0..2]),
            0b010 => self.memory.store(offset, &data),
            _ => return Err(Error::UnsupportFunct3),
        }

        clear_x0(regs);
        next_inst(pc);

        Ok(())
    }

    pub fn iset(self, pc: &mut R, regs: &mut [R]) -> Result<()> {
        let inst = self.inst;
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
            _ => return Err(Error::UnsupportFunct3),
        };
        regs[rd].set_symbol32(res);

        clear_x0(regs);
        next_inst(pc);
        Ok(())
    }

    pub fn opset(self, pc: &mut R, regs: &mut [R]) -> Result<()> {
        let inst = self.inst;
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
            _ => return Err(Error::UnsupportFunct3),
        };

        clear_x0(regs);
        next_inst(pc);

        Ok(())
    }
}

impl<M, I, R> Instruction for RiscV32iBaseInstruction<'_, M, I>
where
    I: Instruction<Register = R>,
    R: RV32Register + Clone,
    M: Memory<Register = R> + MemoryMut,
{
    const REGISTER_NUMBER: usize = 32;

    type Register = R;

    fn execute(self, pc: &mut R, regs: &mut [R]) -> Result<()> {
        let opcode = self.inst.opcode();

        match opcode {
            0b0110111 => self.lui(pc, regs),
            0b0010111 => self.auipc(pc, regs),
            0b1101111 => self.jal(pc, regs),
            0b1100111 => self.jalr(pc, regs),
            0b1100011 => self.bset(pc, regs)?,
            0b0000011 => self.lset(pc, regs)?,
            0b0100011 => self.sset(pc, regs)?,
            0b0010011 => self.iset(pc, regs)?,
            0b0110011 => self.opset(pc, regs)?,
            _ => return Err(Error::UnsupportOpcode),
        }

        Ok(())
    }
}
