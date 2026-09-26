//! # pdfcer-gui-base — the floor of the pdfcer-gui crate stack
//!
//! Modules that reference no other module of `pdfcer-gui`, lifted out of
//! it so that the compiler, rather than a review, is what keeps them beneath
//! the application.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/lib.md`.

#![forbid(unsafe_code)]

// Finding the operator's installed Acrobat and handing the open document over
// to it. Everything that can only be true of a real machine — reading the
// registry, starting a process — is behind a trait in there, so the decisions
// are testable without either.
pub mod acrobat;

/// The one place the program reads a wall clock: PDF and ISO dates in UTC.
/// `pdfcer-core` refuses to supply a timestamp, for determinism and because a
/// date is a claim; its header carries why the answer is UTC and never a local
/// time labelled as one.
pub mod clock;

/// The opt-in trace of what the shell actually received.
pub mod diag;

/// OCR: what image the recogniser is shown, the thread it runs on, and the
/// named refusals it can come back with. It authors no PDF — `pdfcer-core`'s
/// `ocr::layer` writes the invisible mode-3 sandwich. See its header for why
/// recognition reads the document as it was OPENED, and for the y-flip it
/// deliberately does not perform.
pub mod ocr;
/// Does a document's page tree still agree with itself: the raw `/Count`
/// against the leaves actually reachable, audited on every save, and which
/// refusal a disagreement owes. The wording lives in `pdfcer-gui`'s `text`.
pub mod pagetree;

/// A string the operator typed that must never reach a log — one type, and its
/// whole reason for existing is its `Debug`. See its header: a `{:?}` on an
/// action carrying a password writes it into the trace file `tools/ui-verify`
/// keeps as evidence.
pub mod secret;

/// Poster printing: one page across many sheets, with a band along each
/// sheet's top and left for cut marks and the assembly label, and the drawing
/// of both. The tiling is `pdfcer_print::imposition::plan_poster`'s.
pub mod poster;

pub mod redact;

#[cfg(feature = "signing")]
pub mod sign;

/// Where signature trust ANCHORS come from, and the three facts they let
/// this shell state.
pub mod trust;

/// **The one length-conversion table for this program.**
///
/// Points to millimetres, inches, metres or any other engine [`Unit`], and
/// back. Every operator-facing length goes through it, and the gate
/// `tools/gates/check-unit-conversion.sh` fails the build when a second copy
/// of the constant appears anywhere under either crate's `src/`.
///
/// It exists because of a measured defect, not for tidiness: a sheet of
/// exactly 210.5 mm rendered `210` in the page-thumbnail tooltip and `211`
/// in the print dialogue. Both surfaces computed the same value; they disagreed
/// on the ROUNDING RULE, because half the program wrote `.round()` (half away
/// from zero, the CAD convention) and the other half wrote `{:.0}` in a format
/// string, which is Rust's default and rounds half to even. The module's header
/// argues that choice rather than leaving it to whichever spelling a caller
/// reached for.
///
/// [`Unit`]: pdfcer_core::dimension::Unit
pub mod units;
