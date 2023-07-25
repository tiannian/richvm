pub trait BytecodeReader {
    type Register;

    type Error;

    fn read(&mut self, offset: &Self::Register, length: u8) -> Result<&[u8], Self::Error>;
}
