use crate::{
    riscv::{InstB, InstI, InstJ, InstU},
    Memory, MemoryMut, Reg32,
};

fn next_inst<R: Reg32>(pc: &mut R) {
    pc.add_symbol32(4)
}

pub fn lui<R: Reg32>(inst: &InstU, pc: &mut R, regs: &mut [R]) {
    regs[inst.rd()].set_reg32(inst.imm());
    next_inst(pc)
}

pub fn auipc<R: Reg32>(inst: &InstU, pc: &mut R, regs: &mut [R]) {
    regs[inst.rd()].set_reg32(pc.reg32() + inst.imm());
    next_inst(pc)
}

pub fn jal<R: Reg32>(inst: &InstJ, pc: &mut R, regs: &mut [R]) {
    regs[inst.rd()].set_reg32(pc.reg32() + 4);
    pc.add_symbol32(inst.imm_symbol())
}

pub fn jalr<R: Reg32>(inst: &InstI, pc: &mut R, regs: &mut [R]) {
    regs[inst.rd()].set_reg32(pc.reg32() + 4);
    let r = regs[inst.rs1()].symbol32() + inst.imm_symbol();
    pc.set_symbol32(r & (!1));
}

fn branch<R: Reg32>(b: bool, inst: &InstB, pc: &mut R) {
    if b {
        pc.add_symbol32(inst.imm_symbol());
    } else {
        next_inst(pc)
    }
}

pub fn beq<R: Reg32>(inst: &InstB, pc: &mut R, regs: &[R]) {
    let b = regs[inst.rs1()].reg32() == regs[inst.rs2()].reg32();
    branch(b, inst, pc)
}

pub fn bne<R: Reg32>(inst: &InstB, pc: &mut R, regs: &[R]) {
    let b = regs[inst.rs1()].reg32() != regs[inst.rs2()].reg32();
    branch(b, inst, pc)
}

pub fn blt<R: Reg32>(inst: &InstB, pc: &mut R, regs: &[R]) {
    let b = regs[inst.rs1()].symbol32() < regs[inst.rs2()].symbol32();
    branch(b, inst, pc)
}

pub fn bge<R: Reg32>(inst: &InstB, pc: &mut R, regs: &[R]) {
    let b = regs[inst.rs1()].symbol32() >= regs[inst.rs2()].symbol32();
    branch(b, inst, pc)
}

pub fn bltu<R: Reg32>(inst: &InstB, pc: &mut R, regs: &[R]) {
    let b = regs[inst.rs1()].reg32() < regs[inst.rs2()].reg32();
    branch(b, inst, pc)
}

pub fn bgeu<R: Reg32>(inst: &InstB, pc: &mut R, regs: &[R]) {
    let b = regs[inst.rs1()].reg32() >= regs[inst.rs2()].reg32();
    branch(b, inst, pc)
}

pub fn lb<R: Reg32, M>(inst: &InstI, pc: &mut R, regs: &mut [R], memory: &M)
where
    R: Reg32 + Clone,
    M: Memory<Register = R>,
{
    let mut offset = regs[inst.rs1()].clone();
    offset.add_symbol32(inst.imm_symbol());
    let m = memory.load(offset, 1);

    regs[inst.rd()].set_symbol32(i32::from_le_bytes([0, 0, 0, m[0]]) >> 24);

    next_inst(pc)
}

pub fn lh<R: Reg32, M>(inst: &InstI, pc: &mut R, regs: &mut [R], memory: &M)
where
    R: Reg32 + Clone,
    M: Memory<Register = R>,
{
    let mut offset = regs[inst.rs1()].clone();
    offset.add_symbol32(inst.imm_symbol());
    let m = memory.load(offset, 2);

    regs[inst.rd()].set_symbol32(i32::from_le_bytes([0, 0, m[0], m[1]]) >> 24);

    next_inst(pc)
}

pub fn lw<R: Reg32, M>(inst: &InstI, pc: &mut R, regs: &mut [R], memory: &M)
where
    R: Reg32 + Clone,
    M: Memory<Register = R>,
{
    let mut offset = regs[inst.rs1()].clone();
    offset.add_symbol32(inst.imm_symbol());
    let m = memory.load(offset, 4);

    regs[inst.rd()].set_symbol32(i32::from_le_bytes([m[0], m[1], m[2], m[3]]) >> 24);

    next_inst(pc)
}

pub fn lbu<R: Reg32, M>(inst: &InstI, pc: &mut R, regs: &mut [R], memory: &M)
where
    R: Reg32 + Clone,
    M: Memory<Register = R>,
{
    let mut offset = regs[inst.rs1()].clone();
    offset.add_symbol32(inst.imm_symbol());
    let m = memory.load(offset, 1);

    regs[inst.rd()].set_reg32(u32::from_le_bytes([0, 0, 0, m[0]]) >> 24);

    next_inst(pc)
}

pub fn lhu<R: Reg32, M>(inst: &InstI, pc: &mut R, regs: &mut [R], memory: &M)
where
    R: Reg32 + Clone,
    M: Memory<Register = R>,
{
    let mut offset = regs[inst.rs1()].clone();
    offset.add_symbol32(inst.imm_symbol());
    let m = memory.load(offset, 2);

    regs[inst.rd()].set_reg32(u32::from_le_bytes([0, 0, m[0], m[1]]) >> 24);

    next_inst(pc)
}

pub fn lwu<R: Reg32, M>(inst: &InstI, pc: &mut R, regs: &mut [R], memory: &M)
where
    R: Reg32 + Clone,
    M: Memory<Register = R>,
{
    let mut offset = regs[inst.rs1()].clone();
    offset.add_symbol32(inst.imm_symbol());
    let m = memory.load(offset, 4);

    regs[inst.rd()].set_reg32(u32::from_le_bytes([m[0], m[1], m[2], m[3]]) >> 24);

    next_inst(pc)
}
