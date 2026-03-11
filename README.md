# Hades

Hades is the signal-handling library for my ecosystem. Named after the Greek god of the underworld, it serves as the silent observer that manages the "end" of a process's execution thread when a termination signal is received.

As always, no dependencies are allowed, educational in nature, only rusts standard library and `libc`.


## Supported platforms

- Linux
- macOS
- Windows


## API

Dead simple to use, not dissimilar to `signal_hook`.

Example 1:
```rust
let term_now = Arc::new(AtomicBool::new(false));
for signal in TERM_SIGNALS {
    if let Err(e) = flag::register(*signal, Arc::clone(&term_now)) {
        panic!("Unable to register signal handler - Error: {e}");
    }
}

while !term_now.load(Ordering::Relaxed) {}
```

Example 2:
```rust
let mut signals = Signals::new(&[SIGTERM])?;
// This blocks the main thread completely. 
// No CPU usage, no threads, just waiting for the signal.
if let Some(_signal) = signals.forever().next() {}
```

## Prior research

When a signal is generated—whether by a keystroke (Ctrl+C), a kernel exception (segmentation fault), or another process (the kill command)—the operating system interrupts the target process.
The kernel suspends the process's execution context, often between two individual machine instructions, and forces the instruction pointer to jump to a registered signal handler function. This handler runs on the stack of the interrupted thread (or a dedicated signal stack if configured). This creates a unique form of concurrency: the handler and the main program share the same memory space and thread identity, yet they execute asynchronously relative to each other.
Rust's standard library (std) is designed for general-purpose programming and relies heavily on synchronization primitives (mutexes, thread-local storage) and memory allocation. These are generally unsafe to use inside a signal handler. For instance, println! locks standard output. If the main program is interrupted while holding the stdout lock, and the signal handler attempts to call println!, the handler will wait forever for the lock held by the suspended main program—a classic deadlock.
Therefore, we cannot use standard Rust features within the handler itself. It must rely on libc—the raw bindings to the system's C library—to perform minimal, safe operations. The libc crate provides the necessary type definitions (c_int, sigaction) and function prototypes (write, pipe, sigaction) required to interact with the kernel at this low level.
I must use the handler solely as a bridge to notify the main application thread, converting an asynchronous interrupt into a synchronous event.

### The Self-Pipe Trick

The strategy involves creating a Unix pipe—a unidirectional data channel—owned by the process itself. The signal handler, upon triggering, performs a single, safe action: it writes a byte to the write-end of this pipe. The main application loop, which is monitoring the read-end of the pipe, wakes up, reads the byte, and executes the actual handling logic in a safe, standard context.

I will require global static storage to maintain the state of the pipe, as signal handlers cannot capture environments (they must be extern "C" functions).

### Init

- Pipe Creation: The library calls libc::pipe() (or libc::pipe2() on Linux to set flags atomically). This returns two file descriptors: fds (read) and fds[1] (write). **Recommendation:** Always use `O_CLOEXEC` (via `pipe2` or `fcntl`) to ensure the signal pipe doesn't leak into child processes.
- Non-Blocking Mode: Crucially, both ends of the pipe must be set to non-blocking mode (O_NONBLOCK) using libc::fcntl. This ensures that if the pipe fills up (unlikely, but possible under signal flooding), the handler will not block and freeze the system.
- Global Storage: The fds[1] value is stored in GLOBAL_PIPE_WRITER. The fds value is returned to the user (or wrapped in a Receiver struct). **Recommendation:** Use `std::sync::atomic::AtomicI32` for `GLOBAL_PIPE_WRITER` to ensure thread-safe access during initialization and signal handling.

### Signal Handler

The handler function must be minimal. It acts as a trampoline.

```rust
extern "C" fn hades_handler(sig: libc::c_int) {
    // 1. Save errno.
    // Signal handlers can interrupt syscalls. If we modify errno,
    // the interrupted syscall might see the wrong error code.
    let original_errno = unsafe { *libc::__errno_location() };

    // 2. Load the pipe writer FD.
    // GLOBAL_PIPE_WRITER should be an AtomicI32.
    let fd = GLOBAL_PIPE_WRITER.load(Ordering::Relaxed);

    // 3. Write the signal number to the pipe.
    // 'write' is one of the few async-signal-safe syscalls.
    // We ignore errors; if the pipe is full (flooding), we drop the signal 
    // to maintain system stability.
    let sig_byte = sig as u8;
    unsafe {
        libc::write(fd, &sig_byte as *const _ as *const libc::c_void, 1);
    }

    // 4. Restore errno.
    unsafe { *libc::__errno_location() = original_errno };
}
```

This architecture ensures that the complex logic of parsing the signal, logging it, or shutting down subsystems happens outside the handler, avoiding all concurrency hazards.

I will use libc::sigaction rather than libc::signal. The signal function is deprecated in modern POSIX contexts because its behavior across different Unix versions varies (System V vs. BSD semantics) regarding whether the handler remains installed after triggering.
The sigaction struct allows precise control:
- sa_handler: Pointer to hades_handler.
- sa_mask: A set of signals to block while the handler is running. We typically block all signals to prevent recursive interruptions.
- sa_flags:
    - SA_RESTART: This flag tells the kernel to automatically restart interrupted system calls (like read or open) after the handler finishes. For us, this presents a choice. If we want read operations in the main loop to fail with EINTR (waking them up), we disable this. However, since we use the self-pipe, the main loop's select/poll will wake up due to data on the pipe, so SA_RESTART is generally safer to leave enabled to prevent random syscall failures elsewhere in the app.

#### Windows specifics

The equivalent of sigaction on Windows is SetConsoleCtrlHandler. This API allows an application to register a callback function that the system invokes when specific console events occur (like CTRL_C_EVENT).
Key Differences from Unix:

1. Thread Injection: When a generic signal like Ctrl+C occurs, the Windows kernel creates a new thread in the process to run the handler routine. This is a massive architectural divergence. On Unix, the handler interrupts an existing thread; on Windows, it runs concurrently.
2. Manual FFI Necessity: The libc crate for Rust generally exposes POSIX definitions. It does not expose the Windows kernel32 APIs required for advanced event handling. To stick to the "libc only" (no extra crates) philosophy, I will include a sys module that manually defines the necessary extern "system" functions.

Instead of relying on a polling loop with an AtomicBool (which wastes CPU cycles or reacts slowly), I will implement a blocking mechanism analogous to the Unix Self-Pipe by manually binding to Windows Synchronization Objects.

We will use a Windows Event Object (CreateEventW). This is a kernel object that can be "signaled" or "unsignaled". **Recommendation:** Store the event `HANDLE` in a `std::sync::atomic::AtomicPtr` for safe cross-thread access during initialization.

- Initialization: Create a manual-reset event object using CreateEventW.
- The Handler: When SetConsoleCtrlHandler triggers, it calls SetEvent on the global event handle.
- The Waiter: The main application thread blocks on WaitForSingleObject. When the event is signaled, the wait completes immediately.

Since we are avoiding winapi or windows-sys, we must define these signatures ourselves. This is standard practice for zero-dependency libraries.

```rust
// Architecture: x86_64-pc-windows-msvc
// Calling convention: "system" (stdcall on 32-bit, C on 64-bit)

type BOOL = i32;
type DWORD = u32;
type HANDLE = *mut libc::c_void;

extern "system" {
    fn CreateEventW(
        lpEventAttributes: *mut libc::c_void,
        bManualReset: BOOL,
        bInitialState: BOOL,
        lpName: *const u16,
    ) -> HANDLE;

    fn SetEvent(hEvent: HANDLE) -> BOOL;
    
    fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;

    fn SetConsoleCtrlHandler(
        HandlerRoutine: Option<unsafe extern "system" fn(DWORD) -> BOOL>,
        Add: BOOL,
    ) -> BOOL;
}
```

### Signals

| Signal | Default action | Catchable | Description |
| --------------- | --------------- | --------------- | --------------- |
| SIGINT | Terminate | Yes | Interrupted by Ctrl+C |
| SIGTERM | Terminate | Yes | Graceful shutdown |
| SIGKILL | Terminate | No | Ends process immediately |
| SIGQUIT | Core Dump | Yes | Similar to SIGINT but dumps core |

All other signals are not required.
