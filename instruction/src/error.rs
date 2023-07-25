/// Error Type
pub enum Error {
    EnvironmentCall,
    Breakpoint,
    FailedDeocdeInstructon,
}

/// Error type
pub type Result<T> = core::result::Result<T, Error>;
