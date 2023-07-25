/// Readable Linear Memory
pub trait Memory {
    type Register;

    /// Memory length
    fn length(&self) -> Self::Register;

    /// Load data from memory
    // TODO: Add return result
    fn load(&self, pos: Self::Register, length: u8) -> &[u8];
}

/// Writable Linear memory
pub trait MemoryMut: Memory {
    fn store(&mut self, pos: Self::Register, data: &[u8]);
}

impl<const N: usize> Memory for [u8; N] {
    type Register = u32;

    fn length(&self) -> Self::Register {
        N as u32
    }

    fn load(&self, pos: Self::Register, length: u8) -> &[u8] {
        let pos = pos as usize;
        let end = pos + length as usize;

        &self[pos..end]
    }
}
