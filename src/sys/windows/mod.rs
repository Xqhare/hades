use crate::{
    error::{HadesError, HadesResult},
    term_signals::TermSignal,
};

/// Stub for Windows signal registration.
pub fn register_signal(_sig: TermSignal) -> HadesResult<()> {
    // Windows logic (SetConsoleCtrlHandler) will go here soon!
    Err(HadesError::Generic(
        "Register signal API not yet implemented for Windows".to_string(),
    ))
}

pub fn setup_pipe() -> HadesResult<i32> {
    // Windows Event Object logic will go here.
    Err(HadesError::Generic(
        "Signals API not yet implemented for Windows".to_string(),
    ))
}

pub fn read_signal(_handle: i32) -> HadesResult<u8> {
    // Windows WaitForSingleObject logic will go here.
    Err(HadesError::Generic(
        "Signals API not yet implemented for Windows".to_string(),
    ))
}
