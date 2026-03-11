# Hades

Hades is the signal-handling library for my ecosystem. Named after the Greek god of the underworld, it serves as the silent observer that manages the "end" of a process's execution thread when a termination signal is received.

- **Zero External Dependencies**: Uses only the Rust standard library and `libc`.
- **Educational Architecture**: Implements low-level OS primitives (Self-Pipe, Event Objects) from scratch.
- **Cross-Platform**: Full support for Linux, macOS, and Windows.
- **Deadly Simple API**: Designed as a minimal replacement for `signal_hook`.

## Usage

Hades provides two main ways to handle signals: atomic flags and a blocking iterator.

### Example 1: Atomic Flags
Best for applications with an existing loop that just need a "shutdown" trigger.

```rust
use hades::term_signals::TERM_SIGNALS;
use hades::flag;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

let term_now = Arc::new(AtomicBool::new(false));
for signal in TERM_SIGNALS {
    flag::register(*signal, Arc::clone(&term_now))?;
}

while !term_now.load(Ordering::Relaxed) {
    // Application logic here...
}
```

### Example 2: Blocking Iterator
Best for dedicated signal handling threads or CLI tools.

```rust
use hades::Signals;
use hades::term_signals::TermSignal;

let mut signals = Signals::new(&[TermSignal::SIGINT, TermSignal::SIGTERM])?;

// This blocks the thread indefinitely with zero CPU usage.
for signal in signals.forever() {
    println!("Received signal: {:?}", signal);
    break;
}
```

## How it Works (Evolutionary Documentation)

Handling signals in Rust is notoriously difficult because signal handlers are **asynchronously** executed on the stack of an interrupted thread. Most Rust primitives (like `Mutex` or `println!`) are not **async-signal-safe** and can cause deadlocks if a signal interrupts a thread while it holds a lock.

Hades solves this by converting asynchronous signals into synchronous events using platform-native communication channels.

### Unix: The Self-Pipe Trick
On Unix-like systems (Linux, macOS), Hades creates a non-blocking `libc::pipe`. 
1. The **Signal Handler** is a minimal `extern "C"` function that writes a single byte to the pipe's write-end.
2. The **Main Thread** blocks on a `libc::poll` call monitoring the pipe's read-end.
3. When a byte arrives, the thread wakes up and yields the signal to the application.

### Windows: Events & Atomic Ring Buffers
Windows does not have Unix signals; it uses `SetConsoleCtrlHandler` which spawns a **new thread** for every signal. 
1. The **Console Handler** stores the event type in a thread-safe **Atomic Ring Buffer**.
2. It then signals a Windows **Event Object** (`CreateEventW`).
3. The **Main Thread** blocks on `WaitForSingleObject`. Once signaled, it drains the ring buffer to process all queued signals.

## Supported Signals

| Signal | Windows Equivalent | Default Action | Description |
| :--- | :--- | :--- | :--- |
| `SIGINT` | `CTRL_C_EVENT` | Terminate | User pressed Ctrl+C |
| `SIGTERM` | `CTRL_CLOSE_EVENT` | Terminate | Graceful shutdown request |
| `SIGQUIT` | `CTRL_BREAK_EVENT` | Core Dump | Similar to SIGINT |
| `SIGUSR1` | N/A | Ignore | User-defined signal 1 (Unix only) |
| `SIGUSR2` | N/A | Ignore | User-defined signal 2 (Unix only) |

## Implementation Details

Refer to [startup-notes.md](./startup-notes.md) for the original research, FFI signatures, and technical constraints used during the development of this library.
