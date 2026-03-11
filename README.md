# Hades

Hades is the signal-handling library for my ecosystem. Named after the Greek god of the underworld, it serves as the silent observer that manages the "end" of a process's execution thread when a termination signal is received.

- No dependencies beside `libc` and rusts standard library.
- Educational in nature.
- Simple to use.
- Replacement for `signal_hook` for Rust.


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

