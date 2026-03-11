use super::{SIGINT_PTR, SIGQUIT_PTR, SIGTERM_PTR, SIGUSR1_PTR, SIGUSR2_PTR};
use crate::{
    error::{HadesError, HadesResult},
    term_signals::TermSignal,
};
use std::sync::atomic::{AtomicI32, Ordering};

static GLOBAL_PIPE_WRITER: AtomicI32 = AtomicI32::new(-1);

pub fn register_signal(sig: TermSignal) -> HadesResult<()> {
    let libc_sig = match sig {
        TermSignal::SIGINT => libc::SIGINT,
        TermSignal::SIGTERM => libc::SIGTERM,
        TermSignal::SIGQUIT => libc::SIGQUIT,
        TermSignal::SIGUSR1 => libc::SIGUSR1,
        TermSignal::SIGUSR2 => libc::SIGUSR2,
    };

    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = hades_handler as *const () as usize;
        libc::sigemptyset(&mut action.sa_mask);
        action.sa_flags = libc::SA_RESTART;

        if libc::sigaction(libc_sig, &action, std::ptr::null_mut()) != 0 {
            let err = *libc::__errno_location();
            return Err(HadesError::RegistrationFailed(libc_sig.to_string(), err));
        }
    }

    Ok(())
}

pub fn setup_pipe() -> HadesResult<isize> {
    let mut fds = [0i32; 2];
    unsafe {
        if libc::pipe(fds.as_mut_ptr()) != 0 {
            let err = *libc::__errno_location();
            return Err(HadesError::BackendCreationFailed("pipe", err));
        }

        for fd in fds {
            let flags = libc::fcntl(fd, libc::F_GETFL);
            if libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) != 0 {
                let err = *libc::__errno_location();
                return Err(HadesError::BackendCreationFailed("fcntl(O_NONBLOCK)", err));
            }
        }

        GLOBAL_PIPE_WRITER.store(fds[1], Ordering::Release);
    }

    Ok(fds[0] as isize)
}

extern "C" fn hades_handler(sig: libc::c_int) {
    let original_errno = unsafe { *libc::__errno_location() };

    let flag_ptr = match sig {
        libc::SIGINT => SIGINT_PTR.load(Ordering::Relaxed),
        libc::SIGTERM => SIGTERM_PTR.load(Ordering::Relaxed),
        libc::SIGQUIT => SIGQUIT_PTR.load(Ordering::Relaxed),
        libc::SIGUSR1 => SIGUSR1_PTR.load(Ordering::Relaxed),
        libc::SIGUSR2 => SIGUSR2_PTR.load(Ordering::Relaxed),
        _ => std::ptr::null_mut(),
    };

    if !flag_ptr.is_null() {
        unsafe {
            (*flag_ptr).store(true, Ordering::Release);
        }
    }

    let pipe_writer = GLOBAL_PIPE_WRITER.load(Ordering::Relaxed);
    if pipe_writer != -1 {
        let sig_byte = sig as u8;
        unsafe {
            libc::write(pipe_writer, &sig_byte as *const _ as *const libc::c_void, 1);
        }
    }

    unsafe { *libc::__errno_location() = original_errno };
}

pub fn read_signal(fd_isize: isize) -> HadesResult<u8> {
    let fd = fd_isize as i32;
    let mut byte = 0u8;
    loop {
        let n = unsafe {
            libc::read(fd, &mut byte as *mut _ as *mut libc::c_void, 1)
        };

        if n == 1 {
            return Ok(byte);
        }

        if n == -1 {
            let err = unsafe { *libc::__errno_location() };
            if err == libc::EAGAIN || err == libc::EWOULDBLOCK {
                let mut fds = libc::pollfd {
                    fd,
                    events: libc::POLLIN,
                    revents: 0,
                };
                unsafe {
                    libc::poll(&mut fds, 1, -1);
                }
                continue;
            }
            return Err(HadesError::ReadFailed("read(pipe)", err));
        }
        
        return Err(HadesError::PipeClosedUnexpectedly);
    }
}
