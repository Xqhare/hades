pub type HadesResult<T> = Result<T, HadesError>;

#[derive(Debug)]
pub enum HadesError {
    FlagRegisterSigactionFailed(String),
    Generic(String), // Generic error message - use until you can group errors that use it
}

impl std::fmt::Display for HadesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HadesError::FlagRegisterSigactionFailed(msg) => {
                write!(f, "Failed to register sigaction for signal: '{}'", msg)
            }
            HadesError::Generic(msg) => write!(f, "{}", msg),
        }
    }
}
