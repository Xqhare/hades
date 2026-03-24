mod error;
mod sys;
pub mod term_signals;

use crate::error::HadesResult;
use crate::term_signals::TermSignal;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub mod flag {
    use super::{TermSignal, Arc, AtomicBool, HadesResult, sys, Ordering};

    /// Registers a flag to be set to `true` when the given signal is received.
    pub fn register(sig: TermSignal, flag: Arc<AtomicBool>) -> HadesResult<()> {
        let new_ptr = Arc::into_raw(flag).cast_mut();

        let old_ptr = match sig {
            TermSignal::SIGINT => sys::SIGINT_PTR.swap(new_ptr, Ordering::Release),
            TermSignal::SIGTERM => sys::SIGTERM_PTR.swap(new_ptr, Ordering::Release),
            TermSignal::SIGQUIT => sys::SIGQUIT_PTR.swap(new_ptr, Ordering::Release),
            TermSignal::SIGUSR1 => sys::SIGUSR1_PTR.swap(new_ptr, Ordering::Release),
            TermSignal::SIGUSR2 => sys::SIGUSR2_PTR.swap(new_ptr, Ordering::Release),
        };

        if !old_ptr.is_null() {
            unsafe {
                let _ = Arc::from_raw(old_ptr);
            }
        }

        sys::register_flag(sig, new_ptr)
    }
}

pub struct Signals {
    handle: isize,
}

impl Signals {
    /// Creates a new `Signals` instance and registers the given signals.
    pub fn new(signals: &[TermSignal]) -> HadesResult<Self> {
        let handle = sys::setup_signals_backend()?;
        sys::register_signals(signals)?;
        Ok(Self { handle })
    }

    /// Returns an iterator that blocks indefinitely, yielding signals as they occur.
    pub fn forever(&mut self) -> SignalsForever<'_> {
        SignalsForever { signals: self }
    }
}

pub struct SignalsForever<'a> {
    signals: &'a mut Signals,
}

impl Iterator for SignalsForever<'_> {
    type Item = TermSignal;

    fn next(&mut self) -> Option<Self::Item> {
        match sys::wait_for_signal(self.signals.handle) {
            Ok(sig_byte) => {
                // Map the raw byte back to our TermSignal enum.
                #[cfg(unix)]
                {
                    match i32::from(sig_byte) {
                        libc::SIGINT => Some(TermSignal::SIGINT),
                        libc::SIGTERM => Some(TermSignal::SIGTERM),
                        libc::SIGQUIT => Some(TermSignal::SIGQUIT),
                        libc::SIGUSR1 => Some(TermSignal::SIGUSR1),
                        libc::SIGUSR2 => Some(TermSignal::SIGUSR2),
                        _ => None,
                    }
                }
                #[cfg(windows)]
                {
                    // On Windows, we'll map our internal signal codes (0, 1, 2).
                    match sig_byte {
                        0 => Some(TermSignal::SIGINT),
                        1 => Some(TermSignal::SIGQUIT),
                        2 => Some(TermSignal::SIGTERM),
                        _ => None,
                    }
                }
            }
            Err(_) => None,
        }
    }
}

#[cfg(unix)]
impl Drop for Signals {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.handle as i32);
        }
    }
}

#[cfg(windows)]
impl Drop for Signals {
    fn drop(&mut self) {
        sys::os::SignalsHandle::close(self.handle);
    }
}
