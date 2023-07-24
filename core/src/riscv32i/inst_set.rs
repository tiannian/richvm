use core::ops::{Deref, DerefMut};

use crate::{
    prelude::{Instruction, Memory},
    Error, Result,
};

use super::inst::Inst;

pub enum RiscVTrap {}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RiscVRegister(pub u32);

impl Deref for RiscVRegister {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RiscVRegister {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RiscVRegister {
    pub fn symbol(&self) -> i32 {
        self.0 as i32
    }
}

pub struct RiscV32iInstruction<'a, M> {
    inst: [u8; 4],
    memory: &'a mut M,
}

impl<'a, M> RiscV32iInstruction<'a, M> {
    pub fn new(inst: [u8; 4], memory: &'a mut M) -> Self {
        Self { inst, memory }
    }

    fn clear_x0(&self, regs: &mut [RiscVRegister]) {
        *regs[0] = 0;
    }

    fn next_inst(&self, pc: &mut RiscVRegister) {
        **pc += 4;
    }

    pub fn lui(self, pc: &mut RiscVRegister, regs: &mut [RiscVRegister]) {
        let inst = Inst::new(self.inst);

        let pos = inst.rd();
        *regs[pos] = inst.imm_u();

        self.clear_x0(regs);
        self.next_inst(pc);
    }

    pub fn auipc(self, pc: &mut RiscVRegister, regs: &mut [RiscVRegister]) {
        let inst = Inst::new(self.inst);

        **pc += inst.imm_u();
        *regs[inst.rd()] = **pc;

        self.clear_x0(regs);
        self.next_inst(pc);
    }

    pub fn jal(self, pc: &mut RiscVRegister, regs: &mut [RiscVRegister]) {
        let inst = Inst::new(self.inst);

        **pc = ((**pc as i32) + inst.imm_uj_symbol()) as u32;

        // TODO: Jump check

        *regs[inst.rd()] = **pc + 4;

        *regs[0] = 0;
    }

    pub fn jalr(self, pc: &mut RiscVRegister, regs: &mut [RiscVRegister]) {
        let inst = Inst::new(self.inst);

        **pc = (inst.imm_i() + *regs[inst.rs1()]) & (!1);
        *regs[inst.rd()] = **pc;
    }

    pub fn bset(self, pc: &mut RiscVRegister, regs: &[RiscVRegister]) -> Result<()> {
        let inst = Inst::new(self.inst);

        let res = match inst.funct3() {
            0b000 => regs[inst.rs1()] == regs[inst.rs2()],
            0b001 => regs[inst.rs1()] != regs[inst.rs2()],
            0b100 => (*regs[inst.rs1()] as i32) < (*regs[inst.rs2()] as i32),
            0b101 => (*regs[inst.rs1()] as i32) >= (*regs[inst.rs2()] as i32),
            0b110 => regs[inst.rs1()] < regs[inst.rs2()],
            0b111 => regs[inst.rs1()] >= regs[inst.rs2()],
            _ => return Err(Error::UnsupportFunct3),
        };

        if res {
            **pc += inst.imm_sb();
        } else {
            **pc += 4;
        }

        Ok(())
    }

    pub fn lset(self, pc: &mut RiscVRegister, regs: &mut [RiscVRegister])
    where
        M: Memory<Register = RiscVRegister>,
    {
        let inst = Inst::new(self.inst);
        let funct3 = inst.funct3();
        let length = funct3 & 0x3;
        let m = self.memory.load(regs[inst.rd()], length);

        let mut res = [0u8; 4];

        for i in 0..(length + 1) {
            let i = i as usize;
            res[i] = m[i];
        }
        if 0x4 & funct3 != 0 && m[length as usize] & 0x80 != 0 {
            for i in (3 - length)..4 {
                res[i as usize] = 0xFF;
            }
        }

        *regs[inst.rd()] = u32::from_le_bytes(res);

        self.clear_x0(regs);
        self.next_inst(pc);
    }

    pub fn iset(self, pc: &mut RiscVRegister, regs: &mut [RiscVRegister]) -> Result<()> {
        let inst = Inst::new(self.inst);
        let imm = inst.imm_i_symbol();
        let rd = inst.rd();
        let rd_data = (*regs[rd]) as i32;

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
                if *regs[rd] < inst.imm_i() {
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
        *regs[rd] = res as u32;

        self.clear_x0(regs);
        self.next_inst(pc);
        Ok(())
    }

    pub fn opset(self, pc: &mut RiscVRegister, regs: &mut [RiscVRegister]) -> Result<()> {
        let inst = Inst::new(self.inst);
        let funct3 = inst.funct3();

        let rs1 = inst.rs1();
        let rs2 = inst.rs2();

        *regs[inst.rd()] = match funct3 {
            0b000 => {
                if inst.funct7() == 0 {
                    *regs[rs1] + *regs[rs2]
                } else {
                    *regs[rs1] - *regs[rs2]
                }
            }
            0b001 => *regs[rs1] << *regs[rs2],
            0b010 => {
                if (*regs[rs1] as i32) < (*regs[rs2] as i32) {
                    1
                } else {
                    0
                }
            }
            0b011 => {
                if *regs[rs1] < *regs[rs2] {
                    1
                } else {
                    0
                }
            }
            0b100 => *regs[rs1] ^ *regs[rs2],
            0b101 => {
                if inst.funct7() == 0 {
                    *regs[rs1] >> *regs[rs2]
                } else {
                    ((*regs[rs1] as i32) >> *regs[rs2]) as u32
                }
            }
            0b110 => *regs[rs1] & *regs[rs2],
            0b111 => *regs[rs1] | *regs[rs2],
            _ => return Err(Error::UnsupportFunct3),
        };

        self.clear_x0(regs);
        self.next_inst(pc);

        Ok(())
    }
}

impl<M: Memory<Register = RiscVRegister>> Instruction<M> for RiscV32iInstruction<'_, M> {
    const REGISTER_NUMBER: usize = 32;

    type Register = RiscVRegister;

    fn execute(self, pc: &mut RiscVRegister, regs: &mut [RiscVRegister]) -> Result<()> {
        let opcode = self.inst[0] & 0x7f;

        match opcode {
            0b0110111 => self.lui(pc, regs),
            0b0010111 => self.auipc(pc, regs),
            0b1101111 => self.jal(pc, regs),
            0b1100111 => self.jalr(pc, regs),
            0b1100011 => self.bset(pc, regs)?,
            0b0000011 => self.lset(pc, regs),
            0b0100011 => self.iset(pc, regs)?,
            0b0110011 => self.opset(pc, regs)?,
            _ => return Err(Error::UnsupportOpcode),
        }

        Ok(())
    }
}
