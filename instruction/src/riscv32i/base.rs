use crate::{
    riscv::{Inst, InstB, InstI, InstJ, InstR, InstS, InstU},
    Error, Instruction, Memory, MemoryMut, Reg32, Result,
};

use super::execute;

/// Instruction for base of RISCV32i
///
/// These instruction have no CSR and FENCE included
pub enum RV32iBaseInst<I> {
    /// Load Upper Immediate
    Lui(InstU),
    /// Add Upper Immediate to PC
    Auipc(InstU),
    /// Jump and Link
    Jal(InstJ),
    /// Jump and Link Register
    Jalr(InstI),
    /// Branch of Equal
    Beq(InstB),
    /// Branch of Not Equal
    Bne(InstB),
    /// Branch of Less
    Blt(InstB),
    /// Branch of Greater and Equal
    Bge(InstB),
    /// Branch of Less in Unsigned Int
    Bltu(InstB),
    /// Branch of Greater and Equal in Unsigned Int
    Bgeu(InstB),
    /// Load Byte
    Lb(InstI),
    /// Load Half Word
    Lh(InstI),
    /// Load Word
    Lw(InstI),
    /// Load Unsigned Byte
    Lbu(InstI),
    /// Load Unsigned Half Word
    Lhu(InstI),
    /// Load Unsigned Word
    Lwu(InstI),
    /// Store Byte
    Sb(InstS),
    /// Store Half Word
    Sh(InstS),
    /// Store Word
    Sw(InstS),
    /// Add Immediate
    Addi(InstI),
    /// Set Less Than Immediate
    Slti(InstI),
    /// Set Less Than Immediate Unsigned
    Sltiu(InstI),
    /// Xor Immediate
    Xori(InstI),
    /// Or Immediate
    Ori(InstI),
    /// And Immediate
    Andi(InstI),
    /// Logic Left Shift Immediate
    Slli(InstI),
    /// Logic Right Shift Immediate
    Srli(InstI),
    /// Arithmetic Right Shift Immediate
    Srai(InstI),
    /// Add
    Add(InstR),
    /// Sub
    Sub(InstR),
    /// Logic Lift Shift
    Sll(InstR),
    /// Less than
    Slt(InstR),
    /// Less than in Unsigned
    Sltu(InstR),
    /// Xor
    Xor(InstR),
    /// Logic Right Shift
    Srl(InstR),
    /// Arithmetic Right Shift
    Sra(InstR),
    /// Or
    Or(InstR),
    /// And
    And(InstR),
    /// Env call
    ECall(InstI),
    /// Env break
    EBreak(InstI),
    /// Other Instruction
    Other(I),
}

impl<I: Instruction> RV32iBaseInst<I> {
    fn _new(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 4 {
            return Err(Error::ErrBytecodeLengthNotEnough);
        }

        let inst = Inst::new([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let r = match inst.opcode() {
            0b0110111 => Self::Lui(inst.into()),
            0b0010111 => Self::Auipc(inst.into()),
            0b1101111 => Self::Jal(inst.into()),
            0b1100111 => Self::Jalr(inst.into()),
            0b1100011 => match inst.funct3() {
                0b000 => Self::Beq(inst.into()),
                0b001 => Self::Bne(inst.into()),
                0b100 => Self::Blt(inst.into()),
                0b101 => Self::Bge(inst.into()),
                0b110 => Self::Bltu(inst.into()),
                0b111 => Self::Bgeu(inst.into()),
                _ => Self::Other(I::new(bytes)?),
            },
            0b0000011 => {
                let funct3 = inst.funct3();
                let i = inst.into();

                match funct3 {
                    0b000 => Self::Lb(i),
                    0b001 => Self::Lh(i),
                    0b010 => Self::Lw(i),
                    0b100 => Self::Lbu(i),
                    0b101 => Self::Lhu(i),
                    0b110 => Self::Lwu(i),
                    _ => Self::Other(I::new(bytes)?),
                }
            }
            0b0100011 => {
                let funct3 = inst.funct3();
                let i = inst.into();

                match funct3 {
                    0b000 => Self::Sb(i),
                    0b001 => Self::Sh(i),
                    0b010 => Self::Sw(i),
                    _ => Self::Other(I::new(bytes)?),
                }
            }
            0b0010011 => {
                let funct3 = inst.funct3();
                let imm_i = inst.imm_i();
                let i = inst.into();

                match funct3 {
                    0b000 => Self::Addi(i),
                    0b010 => Self::Slti(i),
                    0b011 => Self::Sltiu(i),
                    0b100 => Self::Xori(i),
                    0b110 => Self::Ori(i),
                    0b111 => Self::Andi(i),
                    0b001 => Self::Slli(i),
                    0b101 => {
                        let t = imm_i & 0x400;
                        if t == 0 {
                            Self::Srli(i)
                        } else {
                            Self::Srai(i)
                        }
                    }
                    _ => Self::Other(I::new(bytes)?),
                }
            }
            0b0110011 => {
                let funct3 = inst.funct3();
                let funct7 = inst.funct7();
                let i = inst.into();

                match funct3 {
                    0b000 => {
                        if funct7 == 0 {
                            Self::Add(i)
                        } else {
                            Self::Sub(i)
                        }
                    }
                    0b001 => Self::Sll(i),
                    0b010 => Self::Slt(i),
                    0b011 => Self::Sltu(i),
                    0b100 => Self::Xor(i),
                    0b101 => {
                        if funct7 == 0 {
                            Self::Srl(i)
                        } else {
                            Self::Sra(i)
                        }
                    }
                    0b110 => Self::Or(i),
                    0b111 => Self::Add(i),
                    _ => Self::Other(I::new(bytes)?),
                }
            }
            0b1110011 => {
                let funct3 = inst.funct3();
                let rd = inst.rd();
                let rs1 = inst.rs1();
                let imm = inst.imm_i();
                let i = inst.into();

                if rd == 0 && funct3 == 0 && rs1 == 0 && imm == 0 {
                    Self::ECall(i)
                } else if rd == 0 && funct3 == 0 && rs1 == 0 && imm == 1 {
                    Self::EBreak(i)
                } else {
                    Self::Other(I::new(bytes)?)
                }
            }
            _ => Self::Other(I::new(bytes)?),
        };

        Ok(r)
    }
}

fn clear_x0<R: Reg32>(regs: &mut [R]) {
    regs[0].set_reg32(0);
}

impl<I, R> Instruction for RV32iBaseInst<I>
where
    I: Instruction<Register = R>,
    R: Reg32 + Clone,
{
    type Register = R;

    fn new(bytes: &[u8]) -> Result<Self> {
        Self::_new(bytes)
    }

    fn execute<M>(&mut self, pc: &mut R, regs: &mut [R], memory: &mut M) -> Result<()>
    where
        M: Memory<Register = R> + MemoryMut,
    {
        match self {
            Self::Lui(inst) => execute::lui(inst, pc, regs),
            Self::Auipc(inst) => execute::auipc(inst, pc, regs),
            Self::Jal(inst) => execute::jal(inst, pc, regs),
            Self::Jalr(inst) => execute::jalr(inst, pc, regs),
            Self::Beq(inst) => execute::beq(inst, pc, regs),
            Self::Bne(inst) => execute::bne(inst, pc, regs),
            Self::Blt(inst) => execute::blt(inst, pc, regs),
            Self::Bge(inst) => execute::bge(inst, pc, regs),
            Self::Bltu(inst) => execute::bltu(inst, pc, regs),
            Self::Bgeu(inst) => execute::bgeu(inst, pc, regs),
            Self::Lb(inst) => execute::lb(inst, pc, regs, memory),
            Self::Lh(inst) => execute::lh(inst, pc, regs, memory),
            Self::Lw(inst) => execute::lw(inst, pc, regs, memory),
            Self::Lbu(inst) => execute::lbu(inst, pc, regs, memory),
            Self::Lhu(inst) => execute::lhu(inst, pc, regs, memory),
            Self::Lwu(inst) => execute::lwu(inst, pc, regs, memory),
            _ => {}
        }

        clear_x0(regs);

        Ok(())
    }
}
