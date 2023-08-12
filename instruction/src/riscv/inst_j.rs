use crate::define_from_inner;

use super::Inst;

pub struct InstJ(Inst);

define_from_inner!(Inst, InstJ);

impl InstJ {
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

    pub fn imm(&self) -> u32 {
        self.0.imm_uj()
    }

    pub fn imm_symbol(&self) -> i32 {
        self.0.imm_uj_symbol()
    }
}
