/// Error Type
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    EnvironmentCall,
    Breakpoint,
    ErrFailedDeocdeInstructon,
    ErrBytecodeLengthNotEnough,
}

/// Error type
pub type Result<T> = core::result::Result<T, Error>;
