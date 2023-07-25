use core::fmt::Debug;

#[derive(Debug)]
pub enum Error<E: Debug> {
    AppError(E),
    InstructionError(tangram_instruction::Error),
}

impl<E: Debug> From<tangram_instruction::Error> for Error<E> {
    fn from(value: tangram_instruction::Error) -> Self {
        Self::InstructionError(value)
    }
}
