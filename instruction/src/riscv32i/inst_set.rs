use crate::{
    prelude::{Instruction, Memory},
    riscv::Inst,
    Error, MemoryMut, Result,
};

/// Instruction for lite of RiscVi32
///
/// These instruction no CSR and FENCE include
pub struct RiscV32iLiteInstruction<'a, M> {
    inst: [u8; 4],
    memory: &'a mut M,
}

impl<'a, M> RiscV32iLiteInstruction<'a, M> {
    pub fn new(inst: [u8; 4], memory: &'a mut M) -> Self {
        Self { inst, memory }
    }

    fn clear_x0(&self, regs: &mut [u32]) {
        regs[0] = 0;
    }

    fn next_inst(&self, pc: &mut u32) {
        *pc += 4;
    }

    pub fn lui(self, pc: &mut u32, regs: &mut [u32]) {
        let inst = Inst::new(self.inst);

        regs[inst.rd()] = inst.imm_u();

        self.clear_x0(regs);
        self.next_inst(pc);
    }

    pub fn auipc(self, pc: &mut u32, regs: &mut [u32]) {
        let inst = Inst::new(self.inst);

        regs[inst.rd()] = *pc + inst.imm_u();

        self.clear_x0(regs);
        self.next_inst(pc);
    }

    pub fn jal(self, pc: &mut u32, regs: &mut [u32]) {
        let inst = Inst::new(self.inst);

        regs[inst.rd()] = *pc + 4;
        let (r, of) = pc.overflowing_add_signed(inst.imm_uj_symbol());

        if of {
            log::debug!("JAL overflow");
        }

        *pc = r;

        // TODO: Jump check

        self.clear_x0(regs);
    }

    pub fn jalr(self, pc: &mut u32, regs: &mut [u32]) {
        let inst = Inst::new(self.inst);

        regs[inst.rd()] = *pc + 4;
        let (r, of) = pc.overflowing_add_signed(inst.imm_i_symbol());

        if of {
            log::debug!("JALR overflow");
        }

        *pc = r & (!1);
    }

    pub fn bset(self, pc: &mut u32, regs: &[u32]) -> Result<()> {
        let inst = Inst::new(self.inst);

        let res = match inst.funct3() {
            0b000 => regs[inst.rs1()] == regs[inst.rs2()],
            0b001 => regs[inst.rs1()] != regs[inst.rs2()],
            0b100 => (regs[inst.rs1()] as i32) < (regs[inst.rs2()] as i32),
            0b101 => (regs[inst.rs1()] as i32) >= (regs[inst.rs2()] as i32),
            0b110 => regs[inst.rs1()] < regs[inst.rs2()],
            0b111 => regs[inst.rs1()] >= regs[inst.rs2()],
            _ => return Err(Error::UnsupportFunct3),
        };

        if res {
            let (r, of) = pc.overflowing_add_signed(inst.imm_sb_symbol());
            if of {
                log::debug!("BRANCH overflow");
            }
            *pc = r;
        } else {
            self.next_inst(pc)
        }

        Ok(())
    }

    pub fn lset(self, pc: &mut u32, regs: &mut [u32]) -> Result<()>
    where
        M: Memory<Register = u32>,
    {
        let inst = Inst::new(self.inst);
        let funct3 = inst.funct3();

        let (offset, of) = regs[inst.rs1()].overflowing_add_signed(inst.imm_i_symbol());
        if of {
            log::debug!("LOAD overflow");
        }

        let m = self.memory.load(offset, 4);

        regs[inst.rd()] = match funct3 {
            0b000 => (i32::from_le_bytes([0, 0, 0, m[0]]) >> 24) as u32,
            0b001 => (i32::from_le_bytes([0, 0, m[0], m[1]]) >> 18) as u32,
            0b010 => (i32::from_le_bytes([m[0], m[1], m[2], m[3]])) as u32,
            0b100 => u32::from_le_bytes([0, 0, 0, m[0]]),
            0b101 => u32::from_le_bytes([0, 0, m[0], m[1]]),
            0b110 => u32::from_le_bytes([m[0], m[1], m[2], m[3]]),
            _ => return Err(Error::UnsupportFunct3),
        };

        self.clear_x0(regs);
        self.next_inst(pc);

        Ok(())
    }

    pub fn sset(self, pc: &mut u32, regs: &mut [u32]) -> Result<()>
    where
        M: Memory<Register = u32> + MemoryMut,
    {
        let inst = Inst::new(self.inst);

        let (offset, of) = regs[inst.rs1()].overflowing_add_signed(inst.imm_i_symbol());
        if of {
            log::debug!("LOAD overflow");
        }

        let data = regs[inst.rs2()].to_le_bytes();

        match inst.funct3() {
            0b000 => self.memory.store(offset, &data[0..1]),
            0b001 => self.memory.store(offset, &data[0..2]),
            0b010 => self.memory.store(offset, &data),
            _ => return Err(Error::UnsupportFunct3),
        }

        self.clear_x0(regs);
        self.next_inst(pc);

        Ok(())
    }

    pub fn iset(self, pc: &mut u32, regs: &mut [u32]) -> Result<()> {
        let inst = Inst::new(self.inst);
        let imm = inst.imm_i_symbol();
        let rd = inst.rd();
        let rd_data = (regs[rd]) as i32;

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
                if regs[rd] < inst.imm_i() {
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
        regs[rd] = res as u32;

        self.clear_x0(regs);
        self.next_inst(pc);
        Ok(())
    }

    pub fn opset(self, pc: &mut u32, regs: &mut [u32]) -> Result<()> {
        let inst = Inst::new(self.inst);
        let funct3 = inst.funct3();

        let rs1 = inst.rs1();
        let rs2 = inst.rs2();

        regs[inst.rd()] = match funct3 {
            0b000 => {
                if inst.funct7() == 0 {
                    regs[rs1] + regs[rs2]
                } else {
                    regs[rs1] - regs[rs2]
                }
            }
            0b001 => regs[rs1] << regs[rs2],
            0b010 => {
                if (regs[rs1] as i32) < (regs[rs2] as i32) {
                    1
                } else {
                    0
                }
            }
            0b011 => {
                if regs[rs1] < regs[rs2] {
                    1
                } else {
                    0
                }
            }
            0b100 => regs[rs1] ^ regs[rs2],
            0b101 => {
                if inst.funct7() == 0 {
                    regs[rs1] >> regs[rs2]
                } else {
                    ((regs[rs1] as i32) >> regs[rs2]) as u32
                }
            }
            0b110 => regs[rs1] & regs[rs2],
            0b111 => regs[rs1] | regs[rs2],
            _ => return Err(Error::UnsupportFunct3),
        };

        self.clear_x0(regs);
        self.next_inst(pc);

        Ok(())
    }
}

impl<M: Memory<Register = u32> + MemoryMut> Instruction<M> for RiscV32iLiteInstruction<'_, M> {
    const REGISTER_NUMBER: usize = 32;

    type Register = u32;

    fn execute(self, pc: &mut u32, regs: &mut [u32]) -> Result<()> {
        let opcode = self.inst[0] & 0x7f;

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
