#[cfg(unix)]
#[test]
fn test_unix_signals_iterator() {
    use hades::{Signals, term_signals::TermSignal};
    use std::thread;
    use std::time::Duration;

    // We use SIGUSR1 because it won't kill the test runner by default.
    let mut signals = Signals::new(&[TermSignal::SIGUSR1]).unwrap();

    // Spawn a thread to send us a signal in a moment.
    thread::spawn(|| {
        thread::sleep(Duration::from_millis(100));
        unsafe {
            libc::raise(libc::SIGUSR1);
        }
    });

    // This should block and then return our signal.
    let mut iter = signals.forever();
    let received = iter.next();

    assert_eq!(received, Some(TermSignal::SIGUSR1));
}

#[cfg(unix)]
#[test]
fn test_unix_flag_registration() {
    use hades::{flag, term_signals::TermSignal};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;
    use std::time::Duration;

    let flag = Arc::new(AtomicBool::new(false));
    flag::register(TermSignal::SIGUSR2, Arc::clone(&flag)).unwrap();

    // Send the signal.
    unsafe {
        libc::raise(libc::SIGUSR2);
    }

    // Give the handler a tiny bit of time to run (though it's usually instant).
    thread::sleep(Duration::from_millis(10));

    assert!(flag.load(Ordering::Relaxed));
}
