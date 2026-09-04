//! One error type for the whole harness.
//!
//! ## Why it is hand-rolled
//!
//! Forty lines, versus a `thiserror` dependency. The trade is deliberate and
//! it is about failure modes, not about purity: this crate is the thing you
//! reach for when the application is behaving strangely, and every dependency
//! it carries is a way for it to fail to build on the day it is most needed.
//!
//! ## Why there is only one variant
//!
//! Nothing in this crate *recovers* from an error. A harness either performed
//! its observation or it did not, and when it did not, the only useful output
//! is a sentence a human can read. Structured variants would exist to be
//! matched on, and there is nothing here that would match on them.
//!
//! What matters instead is that the sentence is *specific*. `Error::new`
//! messages in this crate are written to name the thing that was missing and,
//! where it is not obvious, what to do about it — because a harness error is
//! read by someone who is already confused about something else.

use std::fmt;

/// The harness's only error type. See the module documentation for why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    message: String,
    /// Whether this is a **failure of the thing under test** rather than a
    /// missing precondition.
    ///
    /// # ★★★ Why an `Error` needs to say which it is
    ///
    /// Every check in this harness returns `Result` and, by long convention,
    /// turns `Err` into **SKIPPED** — because almost every error here really is
    /// a precondition: no binary, no fixture, no window, input disabled. That
    /// convention is right for those, and it was silently wrong for one case.
    ///
    /// On 2026-09-03 an outside reviewer found that `pdfcer ▸ Keyboard
    /// shortcuts` **aborted the process**, and
    /// `dialogs_open_in_their_own_window` — which drives that exact dialog —
    /// had been reporting PASS. The line it greps for is written before the
    /// panic, so a crash after the evidence is invisible to it.
    ///
    /// The guard for that lives in `Session::trace`, and if it returned an
    /// ordinary `Err` the crash would have become a **SKIP**. This project's
    /// own record on that: *"a SKIP is not red, so a check can stop running
    /// unnoticed."* A crashed program must be RED.
    ///
    /// So: `fatal` is `false` for everything that was already here, and `true`
    /// only where the harness has positively established that the subject
    /// misbehaved. `Check::run` reads it and calls `fail` rather than `skip`.
    fatal: bool,
}

impl Error {
    /// Build an error from anything printable.
    ///
    /// Write the message as a complete statement naming the missing or failing
    /// thing: `"no window appeared for pid 1234 within 20s"`, not `"timeout"`.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            fatal: false,
        }
    }

    /// Mark this as a failure of the subject, not a missing precondition.
    ///
    /// See the [`Error::fatal`] field. Use it only where the harness has
    /// **observed** the program misbehave — a crash, an abort, a hang past a
    /// stated bound — never for something the harness could not set up.
    #[must_use]
    pub fn fatal(mut self) -> Self {
        self.fatal = true;
        self
    }

    /// Is this a failure of the subject rather than of the setup?
    #[must_use]
    pub fn is_fatal(&self) -> bool {
        self.fatal
    }

    /// The message, for callers that want to fold it into a longer report.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::new(e.to_string())
    }
}

/// The harness's result alias.
pub type Result<T> = std::result::Result<T, Error>;

/// Shorthand for `Err(Error::new(format!(...)))`.
///
/// Exists so the call sites read as prose rather than as three layers of
/// wrapping, which is what encourages the specific messages this module asks
/// for.
#[macro_export]
macro_rules! bail {
    ($($arg:tt)*) => {
        return Err($crate::error::Error::new(format!($($arg)*)))
    };
}
