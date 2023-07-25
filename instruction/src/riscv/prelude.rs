pub trait RV32Register {
    fn reg32(&self) -> u32;

    fn set_reg32(&mut self, v: u32);

    fn symbol32(&self) -> i32 {
        self.reg32() as i32
    }

    fn set_symbol32(&mut self, v: i32) {
        self.set_reg32(v as u32)
    }

    fn add_symbol32(&mut self, v: i32) {
        let (r, o) = self.reg32().overflowing_add_signed(v);
        if o {
            log::debug!("Overflow");
        }
        self.set_reg32(r)
    }

    fn add_reg32(&mut self, v: u32) {
        let (r, o) = self.reg32().overflowing_add(v);
        if o {
            log::debug!("Overflow");
        }
        self.set_reg32(r)
    }
}

impl RV32Register for u32 {
    fn reg32(&self) -> u32 {
        *self
    }

    fn set_reg32(&mut self, v: u32) {
        *self = v;
    }
}

impl RV32Register for u64 {
    fn reg32(&self) -> u32 {
        *self as u32
    }

    fn set_reg32(&mut self, v: u32) {
        *self = v as u64;
    }

    fn set_symbol32(&mut self, v: i32) {
        *self = (v as i64) as u64;
    }
}

pub trait RV64Register: RV32Register {
    fn reg64(&self) -> u64;

    fn set_reg64(&mut self, v: u64);

    fn symbol64(&self) -> i64 {
        self.reg64() as i64
    }
    fn set_symbol64(&mut self, v: i64) {
        self.set_reg64(v as u64)
    }
}

impl RV64Register for u64 {
    fn reg64(&self) -> u64 {
        *self
    }

    fn set_reg64(&mut self, v: u64) {
        *self = v
    }
}
