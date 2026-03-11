use crate::{error::{HadesResult, HadesError}, term_signals::TermSignal};

/// Stub for Windows signal registration.
pub fn register_signal(_sig: TermSignal) -> HadesResult<()> {
    // Windows logic (SetConsoleCtrlHandler) will go here soon!
    Ok(())
}

pub fn setup_pipe() -> HadesResult<i32> {
    Err(HadesError::NotImplemented("Signals API (Self-Pipe/Event Objects)"))
}

pub fn read_signal(_handle: i32) -> HadesResult<u8> {
    Err(HadesError::NotImplemented("Signals API (Read/Wait)"))
}
