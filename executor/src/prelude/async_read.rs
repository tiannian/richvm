pub trait AsyncBytecodeReader {
    type Error;

    type Register;

    async fn read(&mut self, offset: &Self::Register, length: u8) -> Result<&[u8], Self::Error>;
}
