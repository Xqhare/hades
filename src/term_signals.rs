#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TermSignal {
    SIGINT,
    SIGTERM,
    SIGQUIT,
}

/// A collection of signals that are commonly used to terminate a process.
pub const TERM_SIGNALS: &[TermSignal] = &[
    TermSignal::SIGINT,
    TermSignal::SIGTERM,
    TermSignal::SIGQUIT,
];
