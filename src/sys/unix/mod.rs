use super::{SIGINT_PTR, SIGQUIT_PTR, SIGTERM_PTR};
use crate::{
    error::{HadesError, HadesResult},
    term_signals::TermSignal,
};
use std::sync::atomic::Ordering;

/// Registers the global signal handler for the given signal via `sigaction`.
pub fn register_signal(sig: TermSignal) -> HadesResult<()> {
    let libc_sig = match sig {
        TermSignal::SIGINT => libc::SIGINT,
        TermSignal::SIGTERM => libc::SIGTERM,
        TermSignal::SIGQUIT => libc::SIGQUIT,
    };

    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = hades_handler as *const () as usize;
        // We want to block all signals while the handler is running to avoid recursion.
        libc::sigemptyset(&mut action.sa_mask);
        // SA_RESTART ensures that syscalls interrupted by signals are automatically resumed.
        action.sa_flags = libc::SA_RESTART;

        if libc::sigaction(libc_sig, &action, std::ptr::null_mut()) != 0 {
            return Err(HadesError::FlagRegisterSigactionFailed(
                libc_sig.to_string(),
            ));
        }
    }

    Ok(())
}

/// The global "trampoline" handler.
extern "C" fn hades_handler(sig: libc::c_int) {
    // 1. Save errno to prevent side effects on interrupted syscalls.
    let original_errno = unsafe { *libc::__errno_location() };

    // We check which signal was received and if a corresponding AtomicPtr is registered.
    let flag_ptr = match sig {
        libc::SIGINT => SIGINT_PTR.load(Ordering::Relaxed),
        libc::SIGTERM => SIGTERM_PTR.load(Ordering::Relaxed),
        libc::SIGQUIT => SIGQUIT_PTR.load(Ordering::Relaxed),
        _ => std::ptr::null_mut(),
    };

    if !flag_ptr.is_null() {
        unsafe {
            // We use Ordering::Release here so that if the application uses Ordering::Acquire,
            // it can synchronize safely with any work done before the signal.
            (*flag_ptr).store(true, Ordering::Release);
        }
    }

    // 3. Handle Pipe
    // (This part will be expanded once we implement the `Signals` iterator)

    // 4. Restore errno.
    unsafe { *libc::__errno_location() = original_errno };
}
