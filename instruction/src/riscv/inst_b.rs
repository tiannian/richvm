use crate::define_from_inner;

use super::Inst;

pub struct InstB(Inst);

define_from_inner!(Inst, InstB);

impl InstB {
    pub fn new(inst: [u8; 4]) -> Self {
        Self(Inst::new(inst))
    }

    pub fn inst(&self) -> &Inst {
        &self.0
    }

    pub fn opcode(&self) -> u8 {
        self.0.opcode()
    }

    pub fn funct3(&self) -> u8 {
        self.0.funct3()
    }

    pub fn rs1(&self) -> usize {
        self.0.rs1()
    }

    pub fn rs2(&self) -> usize {
        self.0.rs2()
    }

    pub fn imm(&self) -> u32 {
        self.0.imm_sb()
    }

    pub fn imm_symbol(&self) -> i32 {
        self.0.imm_sb_symbol()
    }
}
