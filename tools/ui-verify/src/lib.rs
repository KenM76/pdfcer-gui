//! # ui-verify — verification that drives the running application
//!
//! ## The standing rule this crate exists to serve
//!
//! `GUI_ROADMAP.md` § "A standing rule this investigation earned":
//!
//! > **Every GUI change needs a verification that drives the running binary,
//! > not only a test that passes.**
//!
//! That rule was earned, not adopted. Two defects in `DEFECTS.md` were
//! invisible to a fully green test suite and obvious within thirty seconds of
//! using the application:
//!
//! * **D1 — the Delete key.** The only test of `collect_keyboard_actions`
//!   builds a bare `egui::Context` with **no widgets**, so the focused-widget
//!   condition that breaks the real application cannot occur in the harness.
//!   The regression commit says so in its own message: *"analysis-confirmed,
//!   NOT empirically verified."* The property that fails in the product is
//!   structurally absent from the only test of it.
//! * **D2 — invisible headings.** Two theme tests sit adjacent to the bug and
//!   neither measures a rendered foreground/background pair. One of them
//!   asserts the very colour that is wrong, correctly, for a different purpose.
//!
//! Neither defect is a logic error that a better unit test would have caught.
//! Both are *integration between our code and a framework*, and both have
//! exactly one honest oracle: the built binary, running, with a real window.
//!
//! ## What this harness is, in one paragraph
//!
//! It launches a **built binary** with its diagnostic trace enabled, opens a
//! fixture document, drives a scripted input sequence **through the operating
//! system** (real cursor, real keystrokes, real window manager), captures the
//! window to a PNG, and asserts against two oracles: the `key=value` stderr
//! trace, and the pixels.
//!
//! ## The two oracles, and why both are needed
//!
//! | Oracle | Answers | Blind to |
//! |---|---|---|
//! | [`trace`] — the `key=value` stderr diagnostic | "did the click land", "what did the hit test return", "how many objects were deleted" | anything about what was *drawn* |
//! | [`pixels`] — the rendered screenshot | "is that caption legible", "did the control get clipped out of its pane" | anything about internal state |
//!
//! D1 is a trace defect: the state never changed. D2 is a pixel defect: the
//! state is right and the drawing is wrong. A harness with one oracle catches
//! one of them. `PROJECT_PLAN.md` §4.2 prerequisite 2 makes the screenshot
//! oracle a named S1 deliverable for exactly this reason, citing two recorded
//! cases where a traced rect was correct and the control was still clipped out
//! of its pane.
//!
//! ## The coordinate contract — the one rule a check author must not break
//!
//! **Scripts are written in document coordinates. Never in absolute screen
//! coordinates.** See [`coords`] for the enforcement and the full rationale.
//! In summary: panel widths become user-variable in this project
//! (`MODES_AND_PANELS.md`), so a hard-coded screen point silently stops hitting
//! anything — and a stale coordinate is *symptom-identical* to a broken
//! coordinate conversion. That confusion has already produced one
//! filed-then-retracted coordinate-space defect in this codebase, which is why
//! `PROJECT_PLAN.md` §4.2 lists the document-space seam as an S1 prerequisite
//! rather than a nicety.
//!
//! ## No false passes — the SKIPPED state
//!
//! The application these checks target is **under construction**: as of S2 it
//! runs, opens a document, and traces its canvas layout, its named UI regions
//! and its page object count — but it has no ribbon, no selection subsystem
//! and no Settings dialog, and two of the three checks are about surfaces that
//! do not exist yet. The checks must therefore be runnable today, and they
//! must never report success for a run that did not happen. Every check
//! resolves to exactly one of:
//!
//! * **PASS** — it ran, drove the binary, and the assertion held.
//! * **FAIL** — it ran, drove the binary, and the assertion did not hold.
//! * **SKIPPED** — a *precondition* was absent (no binary, no fixture, no
//!   coordinate mapping, no diag vocabulary), with the specific missing thing
//!   named in the reason.
//!
//! The line between SKIP and FAIL is drawn deliberately and it is the most
//! important design decision in this crate:
//!
//! > **A missing *precondition* skips. A missing *postcondition* fails.**
//!
//! If the harness cannot get as far as pressing Delete, that is a SKIP — it
//! learned nothing. If it pressed Delete and the object count did not drop,
//! that is a FAIL — including when the "object count dropped" evidence is a
//! trace line the binary is perfectly capable of emitting and did not. See
//! [`checks::delete_key`] for how that distinction is drawn concretely, and
//! why it is what makes the old binary FAIL rather than SKIP.
//!
//! ## Module map
//!
//! | Module | Job |
//! |---|---|
//! | [`error`] | one error type, no dependencies |
//! | [`geom`] | points and rectangles, including the fraction-of-window rect |
//! | [`coords`] | **the coordinate seam** — document space to screen, and the type-level rule that checks may not skip it |
//! | [`trace`] | parse the `key=value` diagnostic trace |
//! | [`profile`] | what a target binary's trace vocabulary and regions are |
//! | [`launch`] | start the binary, capture stderr, find its window, never leak it |
//! | [`input`] | drive the pointer and keyboard through the OS |
//! | [`image`] | the pixel buffer, PNG out, PNG in |
//! | [`png`] | a minimal PNG encoder (no dependencies) |
//! | [`capture`] | grab a screen region into an [`image::Image`] |
//! | [`pixels`] | **the pixel oracle** — `contrast_at`, `region_not_uniform` |
//! | [`fixture`] | what the harness knows about the document it opened |
//! | [`report`] | PASS / FAIL / SKIPPED and how a run is summarised |
//! | [`sandbox`] | **a profile directory per check** — the portable binary keeps its settings beside itself, so a shared `--exe` is a shared profile and a suite built on one measures the order it ran in |
//! | [`checks`] | the three named checks |
//! | [`sys`] | every `unsafe` line in the crate, isolated |
//!
//! ## Running it
//!
//! ```text
//! cargo run -p ui-verify -- --list
//! cargo run -p ui-verify -- --exe target/release/pdfcer-gui.exe --pdf fixture.pdf
//! cargo run -p ui-verify -- --profile pdfcer-legacy --image evidence/crop_settings.png \
//!                           --check settings_headings_legible
//! ```

pub mod capture;
pub mod checks;
pub mod coords;
pub mod error;
pub mod fixture;
pub mod geom;
pub mod image;
pub mod input;
pub mod launch;
pub mod pixels;
pub mod png;
pub mod profile;
pub mod report;
pub mod sandbox;
pub mod sys;
pub mod trace;
