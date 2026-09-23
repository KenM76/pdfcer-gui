//! # pdfcer-gui-base — the floor of the pdfcer-gui crate stack
//!
//! Modules that reference no other module of `pdfcer-gui`, lifted out of
//! it so that the compiler, rather than a review, is what keeps them beneath
//! the application.
//!
//! ## What may live here
//!
//! A module whose dependency count on the rest of `pdfcer-gui` is **zero**, as
//! measured by `python tools/module-graph.py`. That is the whole admission
//! test, and it is mechanical on purpose — "is this low-level enough?" is a
//! question two people answer differently, and answering it differently twice
//! is how the crate above ended up with 25 mutually recursive module pairs.
//!
//! ## What this crate is NOT
//!
//! It is **not** `egui-shell`, and the two must not be confused when deciding
//! where something goes:
//!
//! | | `egui-shell` | `pdfcer-gui-base` |
//! |---|---|---|
//! | may name a PDF concept | **never** — `check-shell-purity.sh` fails the build | yes; [`units`] converts against `pdfcer_core` |
//! | intended reuse | another application entirely | this application only |
//!
//! A reusable, domain-free thing belongs in `egui-shell`. A pdfcer thing that
//! simply sits low belongs here.
//!
//! ## Why the callers do not mention this crate
//!
//! `pdfcer-gui`'s crate root re-exports these names, so every one of the
//! roughly 1,200 existing call sites still spells them `crate::diag::…`,
//! `crate::units::…` and so on, unchanged.
//!
//! That is deliberate, and it is not laziness. Rewriting an import line in
//! several hundred files is a wide, shallow, mechanical diff — the worst
//! possible neighbour for anyone editing the same files that day — and it buys
//! nothing, because **the boundary is enforced by the crate graph, not by how
//! a caller spells the path**. A module in here cannot reach up into the
//! application whatever the call sites look like: the code does not compile.
//!
//! The one thing the re-export costs is that a reader of `crate::diag` cannot
//! see from the call site that it crosses a crate. `pdfcer-gui`'s crate root
//! says so at the re-export.

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
///
/// The instrument every other module is measured with, and the reason R1 can
/// be satisfied at all: a GUI defect's only honest oracle is the running
/// application, and what happens between the window manager and the first line
/// of our code is unobservable from the source.
///
/// Lowest thing in the stack by fan-in — sixteen modules report through it —
/// which is exactly why it is the anchor of this crate.
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

/// Where signature trust ANCHORS come from, and the three facts they let
/// this shell state.
///
/// The shell half of `pdfcer-core`'s `Pass 10.2`–`10.5`: locating the trust
/// list an installed Acrobat/Reader has downloaded, reading it (read-only, no
/// network, opt-in and off by default), and threading it into
/// `signature::verify_all_with_trust`.
///
/// Its header carries the rule that governs the whole subject — **this is the
/// one place in the product where a wrong answer is worse than no answer** —
/// and the consequence: integrity, coverage and trust are reported separately,
/// never folded into one badge, and `NotChecked` renders as itself.
pub mod trust;

/// **The one length-conversion table for this program.**
///
/// Points to millimetres, inches, metres or any other engine [`Unit`], and
/// back. Every operator-facing length goes through it, and the gate
/// `tools/gates/check-unit-conversion.sh` fails the build when a second copy
/// of the constant appears anywhere under either crate's `src/`.
///
/// ★ It exists because of a measured defect, not for tidiness: a sheet of
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
