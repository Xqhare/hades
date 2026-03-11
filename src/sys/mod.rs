use crate::error::HadesResult;
use crate::term_signals::TermSignal;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
use unix as os;
#[cfg(windows)]
use windows as os;

// Global pointers to the AtomicBools provided by the user via Arc.
// These are private to the `sys` module so only the handlers can see them.
pub(crate) static SIGINT_PTR: AtomicPtr<AtomicBool> = AtomicPtr::new(std::ptr::null_mut());
pub(crate) static SIGTERM_PTR: AtomicPtr<AtomicBool> = AtomicPtr::new(std::ptr::null_mut());
pub(crate) static SIGQUIT_PTR: AtomicPtr<AtomicBool> = AtomicPtr::new(std::ptr::null_mut());

/// The unified entry point for registering a flag.
pub fn register_flag(sig: TermSignal, flag: *mut AtomicBool) -> HadesResult<()> {
    match sig {
        TermSignal::SIGINT => SIGINT_PTR.store(flag, Ordering::Release),
        TermSignal::SIGTERM => SIGTERM_PTR.store(flag, Ordering::Release),
        TermSignal::SIGQUIT => SIGQUIT_PTR.store(flag, Ordering::Release),
    }

    os::register_signal(sig)
}
