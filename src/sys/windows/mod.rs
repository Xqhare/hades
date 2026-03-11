use super::{SIGINT_PTR, SIGQUIT_PTR, SIGTERM_PTR};
use crate::{
    error::{HadesError, HadesResult},
    term_signals::TermSignal,
};
use std::sync::atomic::{AtomicI32, AtomicPtr, Ordering};

// --- Windows FFI Types and Constants ---
type BOOL = i32;
type DWORD = u32;
type HANDLE = *mut libc::c_void;

const TRUE: BOOL = 1;
const FALSE: BOOL = 0;

const CTRL_C_EVENT: DWORD = 0;
const CTRL_BREAK_EVENT: DWORD = 1;
const CTRL_CLOSE_EVENT: DWORD = 2;

const INFINITE: DWORD = 0xFFFFFFFF;
const WAIT_OBJECT_0: DWORD = 0;

// --- Atomic Ring Buffer for Signal Queuing ---
// This ensures we don't lose signals if multiple arrive before the main thread can read them.
const RING_BUFFER_SIZE: usize = 16;
static RING_BUFFER: [AtomicI32; RING_BUFFER_SIZE] = [
    const { AtomicI32::new(-1) }, const { AtomicI32::new(-1) }, const { AtomicI32::new(-1) }, const { AtomicI32::new(-1) },
    const { AtomicI32::new(-1) }, const { AtomicI32::new(-1) }, const { AtomicI32::new(-1) }, const { AtomicI32::new(-1) },
    const { AtomicI32::new(-1) }, const { AtomicI32::new(-1) }, const { AtomicI32::new(-1) }, const { AtomicI32::new(-1) },
    const { AtomicI32::new(-1) }, const { AtomicI32::new(-1) }, const { AtomicI32::new(-1) }, const { AtomicI32::new(-1) },
];
static WRITE_CURSOR: AtomicI32 = AtomicI32::new(0);
static READ_CURSOR: AtomicI32 = AtomicI32::new(0);

// --- Global Event Handle ---
static GLOBAL_EVENT: AtomicPtr<libc::c_void> = AtomicPtr::new(std::ptr::null_mut());

extern "system" {
    fn SetConsoleCtrlHandler(
        HandlerRoutine: Option<unsafe extern "system" fn(DWORD) -> BOOL>,
        Add: BOOL,
    ) -> BOOL;

    fn CreateEventW(
        lpEventAttributes: *mut libc::c_void,
        bManualReset: BOOL,
        bInitialState: BOOL,
        lpName: *const u16,
    ) -> HANDLE;

    fn SetEvent(hEvent: HANDLE) -> BOOL;
    fn ResetEvent(hEvent: HANDLE) -> BOOL;
    fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
    fn CloseHandle(hObject: HANDLE) -> BOOL;
    fn GetLastError() -> DWORD;
}

/// Registers the Windows console control handler.
pub fn register_signal(_sig: TermSignal) -> HadesResult<()> {
    unsafe {
        if SetConsoleCtrlHandler(Some(hades_console_handler), TRUE) == FALSE {
            return Err(HadesError::RegistrationFailed(
                "SetConsoleCtrlHandler".to_string(),
                GetLastError() as i32,
            ));
        }
    }
    Ok(())
}

/// Initializes the Windows Event Object for the backend.
pub fn setup_pipe() -> HadesResult<isize> {
    unsafe {
        // Create a manual-reset event, initially nonsignaled.
        let handle = CreateEventW(std::ptr::null_mut(), TRUE, FALSE, std::ptr::null());
        if handle.is_null() {
            return Err(HadesError::BackendCreationFailed(
                "CreateEventW",
                GetLastError() as i32,
            ));
        }
        GLOBAL_EVENT.store(handle, Ordering::Release);
        Ok(handle as isize)
    }
}

/// Blocks until the global event is signaled, then drains the ring buffer.
pub fn read_signal(handle_isize: isize) -> HadesResult<u8> {
    let handle = handle_isize as HANDLE;

    loop {
        // 1. Try to read from the ring buffer first.
        let read_idx = READ_CURSOR.load(Ordering::Relaxed) as usize % RING_BUFFER_SIZE;
        let sig = RING_BUFFER[read_idx].swap(-1, Ordering::Acquire);

        if sig != -1 {
            READ_CURSOR.fetch_add(1, Ordering::Relaxed);
            return Ok(sig as u8);
        }

        // 2. Buffer empty? Wait for the event.
        unsafe {
            let res = WaitForSingleObject(handle, INFINITE);
            if res != WAIT_OBJECT_0 {
                return Err(HadesError::ReadFailed("WaitForSingleObject", GetLastError() as i32));
            }
            // Once woken, reset the event so we can wait again next time.
            ResetEvent(handle);
        }
        // Loop back to try reading the buffer again.
    }
}

/// The Windows Console Control Handler (runs in a new thread).
unsafe extern "system" fn hades_console_handler(ctrl_type: DWORD) -> BOOL {
    // 1. Handle Flags (API Example 1)
    let flag_ptr = match ctrl_type {
        CTRL_C_EVENT => SIGINT_PTR.load(Ordering::Relaxed),
        CTRL_BREAK_EVENT => SIGQUIT_PTR.load(Ordering::Relaxed),
        CTRL_CLOSE_EVENT => SIGTERM_PTR.load(Ordering::Relaxed),
        _ => std::ptr::null_mut(),
    };

    if !flag_ptr.is_null() {
        (*flag_ptr).store(true, Ordering::Release);
    }

    // 2. Handle Backend (API Example 2)
    // Map Windows events to our internal 0, 1, 2 mapping used in lib.rs
    let sig_byte = match ctrl_type {
        CTRL_C_EVENT => 0,
        CTRL_BREAK_EVENT => 1,
        CTRL_CLOSE_EVENT => 2,
        _ => return FALSE, // We don't handle logoff/shutdown yet.
    };

    // Store in ring buffer.
    let write_idx = WRITE_CURSOR.fetch_add(1, Ordering::Relaxed) as usize % RING_BUFFER_SIZE;
    RING_BUFFER[write_idx].store(sig_byte as i32, Ordering::Release);

    // Signal the event to wake the main thread.
    let event = GLOBAL_EVENT.load(Ordering::Acquire);
    if !event.is_null() {
        SetEvent(event);
    }

    TRUE // We handled the signal.
}

/// Ensure we clean up the Windows handle.
impl Drop for SignalsHandle {
    pub fn close(handle: isize) {
        unsafe {
            CloseHandle(handle as HANDLE);
        }
    }
}

// We'll need to call this from lib.rs Drop.
pub struct SignalsHandle;
