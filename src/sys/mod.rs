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

pub(crate) static SIGINT_PTR: AtomicPtr<AtomicBool> = AtomicPtr::new(std::ptr::null_mut());
pub(crate) static SIGTERM_PTR: AtomicPtr<AtomicBool> = AtomicPtr::new(std::ptr::null_mut());
pub(crate) static SIGQUIT_PTR: AtomicPtr<AtomicBool> = AtomicPtr::new(std::ptr::null_mut());
pub(crate) static SIGUSR1_PTR: AtomicPtr<AtomicBool> = AtomicPtr::new(std::ptr::null_mut());
pub(crate) static SIGUSR2_PTR: AtomicPtr<AtomicBool> = AtomicPtr::new(std::ptr::null_mut());

pub fn register_flag(sig: TermSignal, flag: *mut AtomicBool) -> HadesResult<()> {
    match sig {
        TermSignal::SIGINT => SIGINT_PTR.store(flag, Ordering::Release),
        TermSignal::SIGTERM => SIGTERM_PTR.store(flag, Ordering::Release),
        TermSignal::SIGQUIT => SIGQUIT_PTR.store(flag, Ordering::Release),
        TermSignal::SIGUSR1 => SIGUSR1_PTR.store(flag, Ordering::Release),
        TermSignal::SIGUSR2 => SIGUSR2_PTR.store(flag, Ordering::Release),
    }

    os::register_signal(sig)
}

pub fn setup_signals_backend() -> HadesResult<isize> {
    os::setup_pipe()
}

pub fn wait_for_signal(backend_handle: isize) -> HadesResult<u8> {
    os::read_signal(backend_handle)
}

pub fn register_signals(signals: &[TermSignal]) -> HadesResult<()> {
    for &sig in signals {
        os::register_signal(sig)?;
    }
    Ok(())
}
