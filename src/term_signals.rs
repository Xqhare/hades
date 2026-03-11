#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TermSignal {
    SIGINT,
    SIGTERM,
    SIGQUIT,
    /// User-defined signal 1 (Unix only)
    SIGUSR1,
    /// User-defined signal 2 (Unix only)
    SIGUSR2,
}

/// A collection of signals that are commonly used to terminate a process.
pub const TERM_SIGNALS: &[TermSignal] = &[
    TermSignal::SIGINT,
    TermSignal::SIGTERM,
    TermSignal::SIGQUIT,
];
