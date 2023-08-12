use crate::define_from_inner;

use super::Inst;

pub struct InstI(Inst);

define_from_inner!(Inst, InstI);

impl InstI {
    pub fn new(inst: [u8; 4]) -> Self {
        Self(Inst::new(inst))
    }

    pub fn inst(&self) -> &Inst {
        &self.0
    }

    pub fn opcode(&self) -> u8 {
        self.0.opcode()
    }

    pub fn rd(&self) -> usize {
        self.0.rd()
    }

    pub fn funct3(&self) -> u8 {
        self.0.funct3()
    }

    pub fn rs1(&self) -> usize {
        self.0.rs1()
    }

    pub fn imm(&self) -> u32 {
        self.0.imm_i()
    }

    pub fn imm_symbol(&self) -> i32 {
        self.0.imm_i_symbol()
    }
}
