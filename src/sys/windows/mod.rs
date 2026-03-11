use crate::{error::HadesResult, term_signals::TermSignal};

/// Stub for Windows signal registration.
pub fn register_signal(_sig: TermSignal) -> HadesResult<()> {
    // Windows logic (SetConsoleCtrlHandler) will go here soon!
    Ok(())
}
