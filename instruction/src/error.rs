/// Error Type
pub enum Error {
    UnsupportFunct3,
    UnsupportOpcode,
    ErrorImpl,
}

/// Error type
pub type Result<T> = core::result::Result<T, Error>;
