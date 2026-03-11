pub type HadesResult<T> = Result<T, HadesError>;

#[derive(Debug)]
pub enum HadesError {
    /// Failed to register a signal handler with the OS (e.g., sigaction failed).
    RegistrationFailed(String, i32),
    /// Failed to initialize the internal signaling backend (e.g., pipe creation failed).
    BackendCreationFailed(&'static str, i32),
    /// An error occurred while waiting for or reading a signal.
    ReadFailed(&'static str, i32),
    /// The signal pipe was closed unexpectedly.
    PipeClosedUnexpectedly,
    /// The requested feature is not yet implemented on this platform.
    NotImplemented(&'static str),
    /// A generic error message - used for development and platform-specific prototyping.
    Generic(String),
}

impl std::fmt::Display for HadesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HadesError::RegistrationFailed(sig, err) => {
                write!(f, "Failed to register handler for signal {}: errno {}", sig, err)
            }
            HadesError::BackendCreationFailed(op, err) => {
                write!(f, "Failed to initialize backend ({}): errno {}", op, err)
            }
            HadesError::ReadFailed(op, err) => {
                write!(f, "Failed to read signal ({}): errno {}", op, err)
            }
            HadesError::PipeClosedUnexpectedly => {
                write!(f, "Internal signal pipe was closed unexpectedly")
            }
            HadesError::NotImplemented(feature) => {
                write!(f, "Feature '{}' is not implemented on this platform", feature)
            }
            HadesError::Generic(msg) => write!(f, "{}", msg),
        }
    }
}
