mod error;
mod sys;
pub mod term_signals;

use crate::term_signals::TermSignal;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub mod flag {
    use super::*;

    /// Registers a flag to be set to `true` when the given signal is received.
    ///
    /// This function takes an `Arc<AtomicBool>`. It "leaks" one reference of this Arc
    /// to a global static so the signal handler can safely access it.
    pub fn register(sig: TermSignal, flag: Arc<AtomicBool>) -> crate::error::HadesResult<()> {
        // 1. Convert Arc to a raw pointer. This increments the reference count 

        // internally (via into_raw) so the memory stays alive even if the user drops their Arcs.
        let new_ptr = Arc::into_raw(flag) as *mut AtomicBool;

        // 2. We swap the pointer in our global state. If there was a previous flag
        // registered for this signal, we take it back and let it drop naturally.
        let old_ptr = match sig {
            TermSignal::SIGINT => sys::SIGINT_PTR.swap(new_ptr, Ordering::Release),
            TermSignal::SIGTERM => sys::SIGTERM_PTR.swap(new_ptr, Ordering::Release),
            TermSignal::SIGQUIT => sys::SIGQUIT_PTR.swap(new_ptr, Ordering::Release),
        };

        if !old_ptr.is_null() {
            unsafe {
                // This recreates the Arc from the old pointer and immediately drops it,
                // correctly decrementing the reference count.
                let _ = Arc::from_raw(old_ptr);
            }
        }

        // 3. Inform the OS to use our handler.
        sys::register_flag(sig, new_ptr)
    }
}
