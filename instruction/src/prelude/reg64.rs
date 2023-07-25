use crate::Reg32;

/// 64-bit register
pub trait Reg64: Reg32 {
    fn reg64(&self) -> u64;

    fn set_reg64(&mut self, v: u64);

    fn symbol64(&self) -> i64 {
        self.reg64() as i64
    }
    fn set_symbol64(&mut self, v: i64) {
        self.set_reg64(v as u64)
    }
}

impl Reg64 for u64 {
    fn reg64(&self) -> u64 {
        *self
    }

    fn set_reg64(&mut self, v: u64) {
        *self = v
    }
}
