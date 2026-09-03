//! # verify — an opt-in trace of what the shell actually received
//!
//! *Salvaged from `D:\Dev\pdfce\crates\pdfce-gui\src\diag.rs` (963 lines,
//! 2026-08-12). What came across is the **channel**; what did not is
//! recorded under "What did not come across" below.*
//!
//! ## Why this exists
//!
//! A GUI defect has exactly one honest oracle: the running application.
//! Everything else — reading the dispatch chain, unit-testing the pure
//! decision functions, checking a headless engine's answer to the same
//! query — can be entirely green while the operator still cannot select an
//! object, because the thing that failed sits between the window manager
//! and the first line of application code.
//!
//! That happened. On 2026-08-04 the operator reported that clicking a
//! drawing object selected nothing. The hit-test was verified correct
//! through the CLI (same engine query, same fixture, right answer), every
//! selection decision function passed headless, and the dispatch from
//! toolbar toggle to the tool's entry point read correctly line by line.
//! Reading harder was not going to close the gap: the remaining
//! candidates were all of the form "does `Response::clicked()` fire at
//! all", which is unobservable from the source.
//!
//! ## Why it does not just take a screenshot
//!
//! The operator was using the machine for real work and explicitly asked
//! that the screen not be commandeered. So the diagnostic has to come out
//! of the process as *text*, from a window that need never be looked at —
//! which also makes it usable from a script, a CI run, or a machine with
//! no display at all.
//!
//! A screenshot is still the only oracle for a *layout or clipping*
//! question, and this module does not pretend otherwise. The two are
//! complementary: this answers "did the event reach my code and what did
//! my code decide", a capture answers "and was the result visible".
//!
//! ## Contract
//!
//! - **Off unless asked.** Enabled only when the [`ENV_VAR`] environment
//!   variable is set to a non-empty value, read **once per process**.
//!   With it unset, [`enabled`] is a single relaxed atomic load and
//!   [`trace`]'s argument closure is never called — so a call site costs
//!   nothing and may be left in place permanently rather than added and
//!   deleted around each investigation (which is how the *next* defect
//!   ends up needing this file written again).
//! - **Writes to stderr, one line per event, `key=value` fields.** stderr
//!   because it needs no path, no handle to keep open, no failure mode of
//!   its own, and redirects with `2>`. `key=value` because the consumer
//!   is a grep or an LLM, not a person reading a log.
//! - **Never a user-facing string.** Nothing here is shown in the
//!   interface, so none of it belongs in an application's string
//!   catalogue.
//! - **Never load-bearing.** No behaviour may depend on the trace. If
//!   deleting this module changed what the application does, the trace
//!   would have become a feature with no tests.
//!
//! ## Usage
//!
//! ```text
//! EGUI_SHELL_DIAG=1 my-app file.ext 2> trace.txt
//! ```
//!
//! ```
//! use egui_shell::verify;
//!
//! // Once, at start-up, before anything traces.
//! verify::set_prefix("myapp");
//!
//! // Anywhere. Costs one atomic load when tracing is off.
//! verify::trace(|| format!("ribbon-tab-activated tab={} groups={}", "view", 4));
//! // → `myapp-diag ribbon-tab-activated tab=view groups=4`
//!
//! // Or, for the common shape, without hand-formatting:
//! verify::event("ribbon-tab-activated")
//!     .kv("tab", "view")
//!     .kv("groups", 4)
//!     .emit();
//! ```
//!
//! ## What changed in the salvage
//!
//! **The environment variable is shell-level, and the line prefix is the
//! application's.** The source hard-coded `PDFCER_DIAG` and the literal
//! prefix `pdfcer-diag`, which is correct for one application and useless
//! for a shell. Splitting them keeps both halves right:
//!
//! - **One variable name** ([`ENV_VAR`]) means a harness that drives *any*
//!   `egui-shell` application arms tracing the same way, and does not
//!   have to be told the application's name before it can ask a question.
//!   `tools/ui-verify` drives whatever binary it is pointed at; a
//!   per-application variable would make the harness's first job
//!   discovering what to set.
//! - **A per-application prefix** ([`set_prefix`]) keeps the *output*
//!   attributable, which is what the prefix was ever for: a line that
//!   begins `pdfcer-diag` is greppable out of a stream that also contains
//!   the window manager's chatter and the graphics driver's warnings.
//!
//! The two must not be conflated in the other direction either. Deriving
//! the variable name from the prefix would make [`enabled`] depend on
//! whether [`set_prefix`] had been called yet — an ordering hazard in a
//! function whose entire contract is that it resolves once and never
//! changes.
//!
//! ## What did not come across, and why
//!
//! The source's larger half was a **scripted-input harness**: a `Step`
//! enum of about forty variants (`move:800,550`, `tool:measure`,
//! `print-tab:copies`, `export:dxf-go`) parsed from a second environment
//! variable and injected into `egui`'s `RawInput` one step per frame.
//!
//! None of that is here, and the reason is the same reason the theme's
//! overlay roles moved to [`crate::theme::Overlays`]: **a script
//! vocabulary is an application's vocabulary.** `tool:placefield` and
//! `print-orientation:landscape` are not shell concepts and never will
//! be. A shell that shipped them would be asserting that every future
//! application has a print dialog with three tabs.
//!
//! What *is* generic is the injection seam — "hand `egui` a synthetic
//! event before the frame is built" — and the discipline the source
//! learned around it, which is worth stating here so it is not
//! re-derived:
//!
//! - **Injecting at `egui`'s seam beats posting OS messages.** The
//!   obvious harness posts `WM_MOUSEMOVE`/`WM_LBUTTONDOWN` to the window.
//!   That was tried first, on 2026-08-04, and does not work for an
//!   off-screen window: `winit` calls `TrackMouseEvent` on the move,
//!   Windows answers `WM_MOUSELEAVE` because the physical cursor is
//!   elsewhere, and the button message is dropped before it becomes an
//!   `egui` event. The observed event list was `[PointerMoved,
//!   PointerGone]` — forever, no matter how the messages were ordered.
//! - **A dropped step must announce itself.** The source's parser skipped
//!   unparseable steps silently, and on 2026-08-07 a misspelled step
//!   (`placefield` for `tool:placefield`) was dropped — the resulting
//!   silence was read as a defect in the feature under test, and was
//!   caught only by running a known-good sibling step and noticing the
//!   difference. That is luck, not method. *An absent trace line is
//!   indistinguishable from a step that ran and produced no output.*
//! - **This is a diagnostic, not a substitute for a unit test.** It
//!   proves what the *live application* does with an input; a passing
//!   script is evidence, not a regression guard. Anything it discovers
//!   should end up pinned by a headless test as well.
//!
//! The seam and those rules become the shell's `verify` hooks at stage
//! S1, driven by `tools/ui-verify` — which is the consumer that will say
//! what shape they need. Building them now, with no consumer, is the
//! mistake `SHELL_FRAMEWORK.md` §7 warns about.

use std::fmt::Display;
use std::sync::OnceLock;

/// The environment variable that arms tracing.
///
/// Shell-level rather than per-application on purpose — see this module's
/// header, "What changed in the salvage".
pub const ENV_VAR: &str = "EGUI_SHELL_DIAG";

/// The line prefix used when an application has not set one.
pub const DEFAULT_PREFIX: &str = "egui-shell";

/// Resolved once, on first use, and never re-read.
static ON: OnceLock<bool> = OnceLock::new();

/// Set once by [`set_prefix`], read by every [`trace`].
static PREFIX: OnceLock<String> = OnceLock::new();

/// Whether tracing was requested for this process.
///
/// Resolved once and cached: the check sits in a per-frame path, and
/// re-reading the environment there would put a lock and an allocation in
/// the frame loop to answer a question that cannot change after start-up.
/// After the first call this is one relaxed atomic load, which is what
/// makes leaving trace call sites in place permanently free.
#[must_use]
pub fn enabled() -> bool {
    *ON.get_or_init(|| std::env::var_os(ENV_VAR).is_some_and(|v| !v.is_empty()))
}

/// Set the per-application line prefix. Call once, at start-up.
///
/// Every traced line becomes `<prefix>-diag <message>`, so a stream that
/// also carries the window manager's chatter and the graphics driver's
/// warnings can be reduced to this application's trace with one `grep`.
///
/// # Return value
///
/// `true` if this call set the prefix, `false` if one was already set (in
/// which case the existing prefix is kept). The boolean exists so an
/// application can `debug_assert!` that its start-up path ran once — a
/// second call is a symptom of two initialisation paths, which is worth
/// knowing about even though it is harmless here.
///
/// # Why the prefix is not simply an argument to [`trace`]
///
/// Because then every call site would carry it, and a call site that
/// carried the wrong one would produce lines that the harness's `grep`
/// silently drops. One process, one prefix, set where the process is
/// configured.
pub fn set_prefix(prefix: impl Into<String>) -> bool {
    PREFIX.set(prefix.into()).is_ok()
}

/// The line prefix in force, without the `-diag` suffix.
#[must_use]
pub fn prefix() -> &'static str {
    PREFIX.get().map_or(DEFAULT_PREFIX, String::as_str)
}

/// Emit one trace line, building the message only if tracing is on.
///
/// Takes a closure rather than a `String` so a disabled build path
/// performs no formatting — real call sites interpolate rects, pointer
/// positions and hit counts, and doing that work every frame to throw it
/// away would be a real cost in the one loop that must not get slower.
///
/// The message should be `key=value` fields separated by spaces, led by
/// an event name. [`event`] builds that shape without hand-formatting.
pub fn trace(f: impl FnOnce() -> String) {
    if enabled() {
        eprintln!("{}-diag {}", prefix(), f());
    }
}

/// Begin a `key=value` trace line.
///
/// The builder form of [`trace`], for the common case where a line is an
/// event name and some fields. Cheap when tracing is off: no buffer is
/// allocated and every [`Line::kv`] is a no-op.
///
/// It is *not* free when off — the field values are still evaluated by
/// the caller, since they are arguments rather than a closure. For a call
/// site whose values are expensive to compute (a hit-test count, a
/// formatted rect), prefer [`trace`] with a closure, which defers
/// everything.
#[must_use]
pub fn event(name: &str) -> Line {
    if enabled() {
        Line(Some(name.to_owned()))
    } else {
        Line(None)
    }
}

/// A trace line under construction. See [`event`].
///
/// `None` means tracing is off and this line will never be emitted; every
/// method is then a no-op. Holding the `Option` here rather than checking
/// [`enabled`] in each method keeps the disabled path to one branch per
/// field instead of one atomic load per field.
#[derive(Debug)]
pub struct Line(Option<String>);

impl Line {
    /// Append a `key=value` field.
    ///
    /// The value is rendered with [`Display`], so pointer positions,
    /// counts and booleans all work without the caller formatting them.
    /// Neither key nor value is escaped: this is a diagnostic read by a
    /// `grep`, and a quoting scheme would make the output harder to read
    /// in exchange for handling a case that has not arisen in a year of
    /// use. Keep values free of spaces.
    #[must_use]
    pub fn kv(mut self, key: &str, value: impl Display) -> Self {
        if let Some(buf) = self.0.as_mut() {
            use std::fmt::Write as _;
            // Writing to a String cannot fail; the result is discarded
            // rather than unwrapped so a diagnostic can never panic the
            // application it is diagnosing.
            let _ = write!(buf, " {key}={value}");
        }
        self
    }

    /// Emit the line, if tracing is on.
    pub fn emit(self) {
        if let Some(buf) = self.0 {
            eprintln!("{}-diag {buf}", prefix());
        }
    }

    /// The line as it would be emitted, without the prefix, or `None` if
    /// tracing is off.
    ///
    /// Exists for this module's own tests, and for an application that
    /// wants to route a trace somewhere other than stderr. It does not
    /// emit; [`Self::emit`] does.
    #[must_use]
    pub fn into_message(self) -> Option<String> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The disabled path builds nothing.**
    ///
    /// The contract says a call site costs nothing when tracing is off,
    /// and that is what allows trace calls to be left in permanently
    /// rather than added and removed around each investigation. If a
    /// disabled [`Line`] allocated, that claim would be false at every
    /// call site in the frame loop.
    ///
    /// Tests run without [`ENV_VAR`] set, so [`enabled`] is `false` here.
    /// That is also why there is no test of the *enabled* path in this
    /// file: [`enabled`] resolves once per process, so a test that armed
    /// it would arm it for every other test in the binary — the very
    /// coupling that makes a process-global cache correct in production
    /// and untestable in place. The enabled path is exercised by
    /// `tools/ui-verify`, which runs a real process with the variable
    /// set, which is where a claim about a real process belongs.
    #[test]
    fn a_disabled_line_builds_nothing() {
        assert!(
            !enabled(),
            "the test process must not have {ENV_VAR} set, or this file's \
             assumptions about the disabled path are untested"
        );
        let line = event("anything").kv("k", 1).kv("k2", "v");
        assert_eq!(line.into_message(), None);
    }

    /// **The closure form is never called when tracing is off.**
    ///
    /// This is the property that makes an expensive call site free. A
    /// side effect inside the closure is the only way to observe it.
    #[test]
    fn a_disabled_trace_never_calls_its_closure() {
        let mut called = false;
        trace(|| {
            called = true;
            String::new()
        });
        assert!(
            !called,
            "the closure ran with tracing off, so every trace call site is \
             paying for a string it will not print"
        );
    }

    /// **A line built while enabled has the `event key=value` shape.**
    ///
    /// Constructed directly rather than through [`event`] so the shape can
    /// be asserted without arming the process-global switch — see the
    /// note on `a_disabled_line_builds_nothing`.
    #[test]
    fn an_enabled_line_has_the_key_equals_value_shape() {
        let line = Line(Some("ribbon-tab-activated".to_owned()))
            .kv("tab", "view")
            .kv("groups", 4);
        assert_eq!(
            line.into_message().as_deref(),
            Some("ribbon-tab-activated tab=view groups=4")
        );
    }

    /// The default prefix is used until an application sets one.
    ///
    /// Deliberately does **not** call [`set_prefix`]: it is a `OnceLock`,
    /// so a test that set it would decide the prefix for every other test
    /// in this binary and make their assertions order-dependent. What is
    /// checkable in-process is the default, and that is what is checked.
    #[test]
    fn the_default_prefix_applies_until_an_application_sets_one() {
        assert_eq!(prefix(), DEFAULT_PREFIX);
    }
}
