pub enum Error {
    UnsupportFunct3,
    UnsupportOpcode,
    ErrorImpl,
}

pub type Result<T> = core::result::Result<T, Error>;
