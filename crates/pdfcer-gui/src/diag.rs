//! # diag — an opt-in trace of what the shell actually received
//!
//! Salvaged from `D:\Dev\pdfce\crates\pdfce-gui\src\diag.rs` (Class A,
//! `SALVAGE.md`). **The header below is carried across verbatim**, because
//! it records *why* the channel exists — an argument that took a real
//! investigation to earn and that a paraphrase would lose.
//!
//! What is salvaged at S0 is the trace channel itself ([`enabled`],
//! [`trace`]). The other 800 lines of the original — the `PDFCER_DIAG_SCRIPT`
//! scripted-input harness, its `Step` grammar, `ScriptTool`, the font-folder
//! preload — land with `tools/ui-verify` at stage S1, which is the thing
//! that consumes them. Salvaging a script grammar before there is a harness
//! to run it would be shipping a language with no speakers.
//!
//! ---
//!
//! ## Why this exists
//!
//! A GUI defect in this project has exactly one honest oracle: the running
//! application (standing rule R86). Everything else — reading the dispatch
//! chain, unit-testing the pure decision functions, checking the CLI's answer
//! to the same query — can be entirely green while the operator still cannot
//! select an object, because the thing that failed sits between the window
//! manager and our first line of code.
//!
//! That happened. On 2026-08-04 the operator reported that clicking a drawing
//! object selected nothing. The hit-test was verified correct through
//! `pdfcer` (the same `pdfcer-core` query, same fixture, right answer), every
//! selection decision function passed headless, and the dispatch from toolbar
//! toggle to `run_vector_edit_tool` read correctly line by line. Reading harder
//! was not going to close the gap: the remaining candidates were all of the
//! form "does `Response::clicked()` fire at all", which is unobservable from
//! the source.
//!
//! ## Why it does not just take a screenshot
//!
//! The operator was using the machine for real work and explicitly asked that
//! the screen not be commandeered. So the diagnostic has to come out of the
//! process as *text*, from a window that need never be looked at — which also
//! makes it usable from a script, a CI run, or a machine with no display at
//! all.
//!
//! ## Contract
//!
//! - **Off unless asked.** Enabled only when the `PDFCER_DIAG` environment
//!   variable is set to a non-empty value, read once per process. With it
//!   unset, [`enabled`] is a relaxed atomic load and [`trace`]'s argument
//!   closure is never called — so a call site costs nothing and may be left in
//!   place permanently rather than added and deleted around each investigation
//!   (which is how the *next* defect ends up needing this file written again).
//! - **Writes to stderr, one line per event, `key=value` fields.** stderr
//!   because it needs no path, no handle to keep open, no failure mode of its
//!   own, and redirects with `2>`. `key=value` because the consumer is a grep
//!   or an LLM, not a person reading a log.
//! - **Never a user-facing string.** Nothing here is shown in the interface, so
//!   none of it belongs in [`crate::text`] (the ui-string catalog governs
//!   operator-visible copy).
//! - **Never load-bearing.** No behaviour may depend on the trace. If deleting
//!   this module changed what the application does, the trace would have become
//!   a feature with no tests.
//!
//! ## Usage
//!
//! ```text
//! PDFCER_DIAG=1 pdfcer-gui file.pdf 2> trace.txt
//! ```
//!
//! ---
//!
//! ## What stage S2 added, and why: three things the harness needs
//!
//! `PROJECT_PLAN.md` §4.3 tabulates *"what the application owes the
//! harness"* — three requirements discovered by **building** `tools/ui-verify`
//! at S1 rather than by reading code. Each removes a harness workaround.
//! Two of the three are implemented in terms of machinery added here.
//!
//! ### The de-duplicating gate ([`trace_changed`])
//!
//! The trace is written for a *machine* consumer, and the machine's question
//! is almost always *"what is the current value of X?"* — answered by the
//! **last** line carrying X. A call site in the frame loop that re-emits an
//! unchanged value 60 times a second answers that question no better and
//! buries every other event while doing it. Measured on the S1 binary: the
//! `canvas-pointer` line produced **50 identical lines in 9 seconds** with
//! the pointer stationary, because it fired once per frame rather than once
//! per movement.
//!
//! That is not merely untidy. `ui-verify` reads the trace file repeatedly
//! while it drives (`Session::trace` re-parses the whole capture after every
//! settle), so per-frame noise is re-parsed on every read and grows the
//! capture without adding information. Worse, it makes a human reading the
//! trace scroll past thousands of lines to find the one event that mattered
//! — which is exactly how pdfcer's own investigation missed a `UNPARSEABLE`
//! rejection that was traced on every single run.
//!
//! So: [`trace_changed`] remembers the last line emitted under a **slot**
//! and emits only when the newly built line differs. "Changed" is defined as
//! *the formatted line differs*, which is deliberately the same definition
//! the consumer uses — a difference too small to change the printed text is,
//! by construction, a difference the consumer could not have read anyway.
//!
//! ### The named-region sink ([`ui_rect`])
//!
//! §4.3 requirement 2. A pixel check needs to know **where** to look, and
//! there are only two honest sources: the application measures the rect on
//! the frame it reports (correct under every layout change), or the harness
//! hard-codes a fraction of the window (stale the first time a panel is
//! resized — the hazard §4.2 prerequisite 1 names). [`ui_rect`] is the first
//! source.
//!
//! It is a **process-global sink on purpose**, and that is the seam: the
//! ribbon is being built in `egui-shell`, which cannot depend on this crate
//! (`tools/gates/check-shell-purity.sh` enforces the one-directional
//! dependency), so it will expose a *callback* that the application supplies.
//! [`ui_rect`] already has the exact `fn(&str, egui::Rect)` shape such a
//! callback takes, captures nothing, and needs no `&mut` threaded through
//! every widget signature. Wiring the ribbon to it is therefore a single
//! registration line at start-up and **no change to this file** — which is
//! the property that lets the two agents' work land independently.
//!
//! ### Zero-cost when off, in both
//!
//! Both check [`enabled`] before touching their registries, so with
//! `PDFCER_DIAG` unset a call site costs one relaxed atomic load and no lock,
//! no hash, no allocation and no formatting. That is what makes it correct
//! to leave these calls in permanently — see the contract above.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard, OnceLock};

/// Whether tracing was requested for this process.
///
/// Resolved once and cached: the check sits in a per-frame path, and re-reading
/// the environment there would put a lock and an allocation in the frame loop
/// to answer a question that cannot change after start-up.
pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| {
        // ui-text-exempt: environment variable name, never displayed
        std::env::var_os("PDFCER_DIAG").is_some_and(|v| !v.is_empty())
    })
}

/// Emit one trace line, building the message only if tracing is on.
///
/// Takes a closure rather than a `String` so a disabled build path performs no
/// formatting — the call sites interpolate rects, pointer positions and hit
/// counts, and doing that work every frame to throw it away would be a real
/// cost in the one loop that must not get slower.
pub fn trace(f: impl FnOnce() -> String) {
    if enabled() {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        eprintln!("pdfcer-diag {}", f());
    }
}

// ---------------------------------------------------------------------------
// The de-duplicating gate
// ---------------------------------------------------------------------------

/// The last line emitted under each [`trace_changed`] slot.
///
/// A `Mutex` rather than a `thread_local!` or a `RefCell` because
/// [`ui_rect`] is designed to be handed to `egui-shell` as a plain
/// `fn(&str, Rect)` callback (see the module docs), and a callback whose
/// correctness depends on which thread invokes it is a trap for whoever wires
/// it up. The lock is uncontended in practice — everything that traces layout
/// runs on the UI thread — and it is only ever taken when tracing is on.
///
/// Keys are `&'static str`, which is not an accident: a slot names a *call
/// site*, and call sites are known at compile time. It also means the
/// steady-state (nothing changed) path performs **no allocation at all** —
/// only a hash of a string that already exists.
static LAST_LINE: LazyLock<Mutex<HashMap<&'static str, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// The last rect emitted for each named region by [`ui_rect`].
///
/// Separate from [`LAST_LINE`] and typed as a [`egui::Rect`] rather than as a
/// rendered string for two reasons: region names are runtime values (a ribbon
/// group's caption id is data, not a literal), so they cannot key
/// [`LAST_LINE`]; and comparing the rect itself rather than its rendering
/// keeps the comparison independent of the format the line happens to be
/// printed in.
static LAST_UI_RECT: LazyLock<Mutex<HashMap<String, egui::Rect>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// The region names [`ui_rect`] has been called with **so far this frame**.
///
/// ## ★ Why this exists: the trace is a CHANGE LOG, and a change log cannot
/// say that something stopped
///
/// [`ui_rect`] emits only when a region's rect *differs* from the last one
/// emitted for that name, which is what keeps the channel usable — a per-frame
/// dump of ~60 regions at 60 fps is a torrent nobody can read. The cost is
/// that a region which stops being drawn **emits nothing**, so its last known
/// rect stands in the trace forever and a reader has no way to tell "still
/// there, unmoved" from "gone forty frames ago".
///
/// That is not academic. It made `ui-verify`'s UI-scale check report **18
/// ribbon controls as lying outside the window** at a large scale. They did
/// not: the ribbon's overflow had correctly swallowed them, and every one of
/// those rects was its position from an earlier frame at a smaller scale. The
/// screenshot showed a perfectly laid-out ribbon with a *5 more* button. The
/// harness was reading a fossil and reporting it as a live layout defect —
/// the exact false-defect outcome `crate::diag`'s own contract is written to
/// avoid.
///
/// So [`end_ui_frame`] diffs this set against the previous frame's and emits
/// `ui-rect-gone name=…` for anything that disappeared. The log stays a change
/// log and becomes an *honest* one, reporting both directions of change.
static UI_RECTS_THIS_FRAME: LazyLock<Mutex<std::collections::HashSet<String>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));

/// The region names that were drawn during the **previous** frame.
static UI_RECTS_LAST_FRAME: LazyLock<Mutex<std::collections::HashSet<String>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));

/// Lock a registry, ignoring poisoning.
///
/// A panic while one of these locks was held would otherwise disable the
/// trace for the rest of the process — and the trace is the thing you reach
/// for *because* something went wrong. `into_inner` keeps the channel alive
/// on a possibly-stale map, which can at worst cost one duplicate or one
/// suppressed line. The contract says the trace is never load-bearing; this
/// is that contract applied to its own failure mode.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Emit one trace line **only when it differs from the last line emitted
/// under the same slot**.
///
/// # What this is for
///
/// Frame-loop call sites. A value that is re-reported unchanged 60 times a
/// second tells a consumer nothing it did not already know from the previous
/// line, and buries the events that *are* news. See the module docs for the
/// measured case (50 identical `canvas-pointer` lines in 9 seconds) and for
/// why noise costs the harness real work rather than merely looking untidy.
///
/// # The definition of "changed", and why it is the formatted line
///
/// Not the underlying value: the **rendered text**. Two consequences, both
/// wanted:
///
/// * A difference too small to change the printed text is a difference the
///   consumer could not have read anyway, so suppressing it loses nothing.
///   The pointer trace prints `{:.2}`; sub-hundredth jitter is invisible to
///   the parser by construction.
/// * A call site does not have to invent an epsilon, or keep a parallel copy
///   of its own state to compare against. There is one rule, in one place.
///
/// # Slots
///
/// A slot is the event name, plus a discriminator when one event has several
/// independent subjects. Two call sites sharing a slot will each suppress the
/// other's lines, which is a real bug and the reason the parameter is
/// `&'static str` — it is meant to be a literal you can grep for.
///
/// Costs nothing when tracing is off: the closure is not called and neither
/// registry is touched.
pub fn trace_changed(slot: &'static str, f: impl FnOnce() -> String) {
    if !enabled() {
        return;
    }
    let line = f();
    // The lock is released before the write: `eprintln!` takes stderr's own
    // lock, and holding two locks in a fixed order across a call that can
    // block is a deadlock waiting for a second tracer to be added.
    let changed = record_if_changed(&mut lock(&LAST_LINE), slot, &line);
    if changed {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        eprintln!("pdfcer-diag {line}");
    }
}

/// The whole decision [`trace_changed`] makes, over an explicit map.
///
/// Split out from [`trace_changed`] — and taking the map as an argument
/// rather than reaching for the global — so the de-duplication rule is
/// testable without an environment variable, without stderr capture, and
/// without two parallel tests fighting over one process-global registry. The
/// rule is the interesting part; `eprintln!` is not.
///
/// Returns whether the caller should print, and records the line if so.
fn record_if_changed(
    map: &mut HashMap<&'static str, String>,
    slot: &'static str,
    line: &str,
) -> bool {
    if map.get(slot).is_some_and(|prev| prev == line) {
        return false;
    }
    map.insert(slot, line.to_owned());
    true
}

/// Declare where a **named UI region** is, in window logical points.
///
/// `PROJECT_PLAN.md` §4.3 requirement 2. Emits
///
/// ```text
/// pdfcer-diag ui-rect name=<name> rect=[[x0 y0] - [x1 y1]]
/// ```
///
/// the first time a region is seen and again whenever it moves or resizes,
/// and nothing at all on the frames in between.
///
/// # Why the application declares this rather than the harness computing it
///
/// A pixel check — "is this caption legible?" — has to know which pixels to
/// measure. The alternative source is a fraction of the window written into
/// the harness, and such a fraction is correct exactly until the first panel
/// is resized, the ribbon collapses to an icon rail, or a workspace is
/// switched. `MODES_AND_PANELS.md` puts all three on the roadmap. A rect
/// measured on the frame it is reported for cannot go stale, because there is
/// no interval between the measurement and the claim.
///
/// # The `rect=` format is not a choice
///
/// It is `egui::Rect`'s own `Debug`, `[[x0 y0] - [x1 y1]]`, because
/// `tools/ui-verify`'s parser already reads that shape (`trace.rs`'s
/// `parse_egui_rect`) for the canvas rect. Emitting a second, tidier spelling
/// would mean two parsers for one concept, which is how the two ends of a
/// bridge drift apart. Pass the `Rect` and let `Debug` write it.
///
/// # The seam for the ribbon
///
/// This function takes `&str` and `egui::Rect`, captures nothing, and returns
/// nothing — it *is* an `fn(&str, egui::Rect)`. `egui-shell` cannot call into
/// this crate (the dependency is one-directional and gated), so it exposes a
/// callback of that shape and the application registers this function.
/// Nothing here needs to change when it does.
///
/// # Naming regions
///
/// Names are matched literally by checks, so they are part of the contract:
/// pick a stable, lowercase, hyphenated noun for the thing an operator would
/// point at (`page`, `canvas-viewport`, `ribbon-group-caption:view/zoom`).
/// Renaming one silently un-aims whatever check was measuring it.
/// [`ui_rect`], but **only if the region is actually visible** inside `clip`.
///
/// # ★ Why a scroll area needs this, and why the plain call is a trap there
///
/// `egui` lays out every child of a `ScrollArea` and then *clips* the ones
/// outside the viewport. So a collapsible header scrolled below the fold still
/// runs its layout, still has a perfectly good `Rect`, and calling [`ui_rect`]
/// with it publishes coordinates for something **nobody can see**.
///
/// A harness reading that declaration measures the pixels at those coordinates
/// — which belong to whatever is genuinely on screen there: another panel, the
/// document, the desktop. It then reports a contrast figure that is a fact
/// about the wrong widget.
///
/// That is not hypothetical. The first live run of `settings_headings_legible`
/// — the regression check for `DEFECTS.md` **D2**, which had SKIPPED for the
/// whole life of the project — reported three of eight headings as illegible
/// or blank. The dialog was fine: the two headings actually on screen measured
/// **13.91:1** against a 3:1 floor. All three "failures" were headings
/// scrolled out of view, and the check was reading the Pages panel and the
/// drawing behind the dialog.
///
/// A check that fires when nothing is wrong is one that gets switched off, and
/// this one guards the defect that justified building the harness.
///
/// # Why the fix is here rather than in the harness
///
/// The harness *could* intersect every rect with the dialog's body. Doing it
/// here is better for a reason that outlives this dialog: it makes the
/// declaration **mean** something — *this region is on screen at this rect* —
/// so every consumer gets the guarantee rather than each one re-deriving it.
/// It is the same repair as `ui-rect-gone`: the channel should describe what
/// is visible, not what was laid out.
///
/// # ★★★ The test is MOSTLY VISIBLE, and it used to be bare intersection
///
/// The rule here read: *"a heading half-scrolled off the bottom is still partly
/// on screen and still worth measuring — a contrast check samples what it can
/// reach. Requiring full containment would silently drop the boundary case."*
///
/// **Measured false on 2026-08-21.** A settings heading sitting two points
/// inside the scroll area's bottom edge published a rect, and the contrast
/// sampler measured **1.53:1** on it — reading the anti-aliased top rows of
/// glyphs whose bodies had been clipped away, at 5.3 % coverage, and reporting
/// an illegible heading in a dialog whose other headings measured 15.07:1.
///
/// So the test is a *proportion*: a region must be at least
/// [`VISIBLE_FRACTION`] inside the clip before it is worth naming. Both ends of
/// the old argument survive — a heading three-quarters visible is still
/// measured, and full containment is still not required — but a sliver is no
/// longer offered to a sampler as though it were a surface.
///
/// ★ The general form, and it is the second instance of it in one afternoon:
/// **a measurement of the wrong surface is indistinguishable from a measurement
/// of a broken one.** The first was a capture of the wrong window; this is the
/// wrong part of the right one. A diagnostic channel that publishes a region
/// nobody can read is not being generous, it is manufacturing false failures.
/// How much of a region must be inside the clip before it is published.
///
/// Three fifths, and the number is a judgement rather than a measurement: it is
/// low enough to keep a heading that is mostly there and high enough to drop
/// the sliver that produced a false 1.53:1. A heading is a row of glyphs about
/// two-thirds the height of its rect, so at 0.6 the glyph bodies are inside the
/// clip whichever end is cut.
const VISIBLE_FRACTION: f32 = 0.6;

pub fn ui_rect_visible(name: &str, rect: egui::Rect, clip: egui::Rect) {
    if !enabled() {
        return;
    }
    let shown = clip.intersect(rect);
    let area = rect.width() * rect.height();
    let visible = shown.width().max(0.0) * shown.height().max(0.0);
    if area > 0.0 && visible / area >= VISIBLE_FRACTION {
        ui_rect(name, rect);
    }
    // Deliberately silent when it does not intersect. This is not a retirement
    // — `end_ui_frame` handles that, and a region that scrolls out of view and
    // back is exactly the case it was built for: it emits `ui-rect-gone` on the
    // frame the region stops being declared, and the rect is re-emitted when it
    // returns.
}

/// **Where a child viewport's client area sits on the DESKTOP.**
///
/// ```text
/// pdfcer-diag viewport-inner id=<hash> rect=[[x0 y0] - [x1 y1]]
/// ```
///
/// # ★★ Why this line has to exist, and what breaks silently without it
///
/// [`ui_rect`] publishes a named region's rectangle **relative to the viewport
/// that drew it**. Until 2026-08-20 there was exactly one viewport, so a
/// harness could add the application window's client origin and be right — and
/// that assumption is baked into every driven check in `tools/ui-verify/`.
///
/// `crate::dialogs::host` makes a dialog a real OS window, which is a second
/// viewport with its own origin. Its regions keep publishing rectangles that
/// look exactly like the ones the harness has always converted, and they now
/// name a completely different place on the desktop — typically by the few
/// hundred points between the two windows' corners.
///
/// **That is a coordinate-space defect with plausible numbers**, which
/// `D:/dev/rag/egui/` already records twice on this project. Both cost days,
/// both were invisible to every unit test, and both presented as *"the click
/// lands somewhere else"*. This line is the fix rather than care: the harness
/// is handed the child's origin instead of assuming one.
///
/// It is also the only way a check can **assert that a dialog opened in its own
/// window at all**, which is what makes `ui-conventions/dialogs.md` G1 testable
/// rather than a matter of looking. A build that reverted to an in-viewport
/// panel emits no `viewport-inner` line, and its absence is the failure.
///
/// Emitted on change only, like every other line in this file, so a dialog
/// sitting still costs nothing per frame and a dragged one reports its travel.
pub fn viewport_inner(id: egui::ViewportId, rect: egui::Rect) {
    if !enabled() {
        return;
    }
    // ★ Keyed by id, so two dialogs open at once are two independent change
    // logs. Keying by "the last viewport" would make each one's move retire the
    // other's rect and republish it, which is a change log that reports motion
    // nothing moved.
    let key = format!("viewport-inner:{:?}", id);
    lock(&UI_RECTS_THIS_FRAME).insert(key.clone());
    if record_rect_if_changed(&mut lock(&LAST_UI_RECT), &key, rect) {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        eprintln!("pdfcer-diag viewport-inner id={:?} rect={rect:?}", id);
    }
}

thread_local! {
    /// Which viewport's coordinate space [`ui_rect`] is currently publishing
    /// in, or `None` for the application's own window.
    ///
    /// A thread-local rather than a parameter because `ui_rect` is called from
    /// ~40 sites, none of which knows or should know that dialogs exist. It is
    /// safe as a thread-local for a reason specific to *immediate* viewports:
    /// egui runs a child's callback **synchronously, inside the parent's
    /// frame, on the parent's thread**, so the scope is a straight-line region
    /// of one call stack rather than a global mode.
    static VIEWPORT: std::cell::RefCell<Option<String>> = const {
        std::cell::RefCell::new(None)
    };
}

/// **Publish every [`ui_rect`] inside this scope as belonging to `id`.**
///
/// Entered by [`crate::dialogs::host::Host::show`] around a dialog's body. A
/// guard rather than a closure because the body needs `&mut` on the dialog it
/// belongs to, and threading that through a closure parameter would push the
/// borrow problem into every caller.
///
/// # ★ What this is for, and the defect it is a fix for rather than a nicety
///
/// A region's rectangle is **relative to the viewport that drew it**. There was
/// one viewport until 2026-08-20, so `tools/ui-verify` adds the application
/// window's client origin and is right. A dialog in its own OS window keeps
/// publishing rectangles that look exactly the same and name a different place
/// on the desktop.
///
/// That is a coordinate-space defect with plausible numbers, which this project
/// has met three times — the snap marker off by the scroll origin, the vertex
/// drag tracking at `1/zoom`, and the caret measured against the wrong font.
/// Every one presented as *"it lands somewhere else"* and every one cost a day.
/// The tag plus [`viewport_inner`] is the fix in the instrument, not in the
/// care.
pub struct ViewportScope;

impl ViewportScope {
    /// Enter the scope. Restores the previous value on drop, so nesting is
    /// correct even though nothing nests today.
    #[must_use]
    pub fn enter(id: egui::ViewportId) -> Self {
        if enabled() {
            VIEWPORT.with(|v| *v.borrow_mut() = Some(format!("{id:?}")));
        }
        Self
    }
}

impl Drop for ViewportScope {
    fn drop(&mut self) {
        VIEWPORT.with(|v| *v.borrow_mut() = None);
    }
}

/// The current viewport's suffix for a `ui-rect` line, or an empty string.
///
/// Empty for the application's own window, so **every existing trace line is
/// byte-identical to what it was** and no consumer has to learn anything to go
/// on working. A harness that never opens a dialog sees no change at all; one
/// that does gets a field it can ask for. That is the cheaper half of the
/// change and it was a deliberate choice over tagging every line with `root`.
fn viewport_suffix() -> String {
    VIEWPORT.with(|v| {
        v.borrow()
            .as_ref()
            // ui-text-exempt: a diagnostic field name, never displayed.
            .map(|id| format!(" viewport={id}"))
            .unwrap_or_default()
    })
}

pub fn ui_rect(name: &str, rect: egui::Rect) {
    if !enabled() {
        return;
    }
    // Recorded before the change test, so a region that is drawn at an
    // unchanged rect still counts as PRESENT this frame. Getting this the
    // other way round would make every unmoved region look retired, which is
    // the failure this set exists to prevent, inverted.
    lock(&UI_RECTS_THIS_FRAME).insert(name.to_owned());
    let changed = record_rect_if_changed(&mut lock(&LAST_UI_RECT), name, rect);
    if changed {
        let where_ = viewport_suffix();
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        eprintln!("pdfcer-diag ui-rect name={name} rect={rect:?}{where_}");
    }
}

/// How many frames between `frame` lines.
///
/// Ten, which is one line per ~250 ms of animation and per ~0.2 s of a busy
/// redraw — negligible beside the hundreds of `ui-rect` lines a frame already
/// emits, and fine enough that `Session::settle` can wait on a count rather than
/// on a clock.
const FRAME_TICK_EVERY: u64 = 10;

/// **A monotonic frame counter on the diagnostic channel.**
///
/// ```text
/// pdfcer-diag frame n=1230
/// ```
///
/// # ★★★ Why this exists, and it is a fix for a whole class of false failure
///
/// `ui-verify`'s `Session::settle(frames)` was
/// `sleep(frames * 25ms)` — a **wall clock wearing the word "frames"**. On an
/// idle machine 25 ms is about a frame and the name is nearly true. Under load
/// it is not: the application renders fewer frames in the same wall time, so
/// every check that "settled" then clicked was acting before the interface had
/// caught up.
///
/// Measured 2026-09-02, running the suite in batches: three checks failed with
/// substantive, believable messages — a bookmark that went to the page and did
/// not zoom, a canvas that stopped seeing the pointer, a list of rows that never
/// drew — and **all three passed when re-run alone against the same binary**.
/// The convenient reading was "contention", which explains nothing and excuses
/// everything. The real mechanism is that the harness was measuring a UI that
/// had not finished responding.
///
/// ⇒ With a counter on the channel, `settle` can wait for the application to
/// actually **produce** frames, and becomes fast when idle and patient when
/// loaded — which is what it always claimed to be.
///
/// ★ Only under `PDFCER_DIAG`, like everything here, and only every tenth frame.
/// A per-frame line would be the one diagnostic that measurably changed the
/// thing it measures.
fn frame_tick() {
    use std::sync::atomic::{AtomicU64, Ordering};
    static FRAMES: AtomicU64 = AtomicU64::new(0);
    let n = FRAMES.fetch_add(1, Ordering::Relaxed) + 1;
    if n.is_multiple_of(FRAME_TICK_EVERY) {
        trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("frame n={n}")
        });
    }
}

/// **Close a frame's region census and report anything that stopped being
/// drawn.**
///
/// Called once at the end of every frame, from `crate::app::frame`. See
/// [`UI_RECTS_THIS_FRAME`] for the defect this exists to remove — in one
/// sentence: a change log that only reports appearances lets a consumer read a
/// stale rect as a live one, and that produced a confident, wrong,
/// eighteen-item layout-defect report.
///
/// # What it emits
///
/// One `ui-rect-gone name=…` line per region that was drawn last frame and was
/// not drawn this frame. Nothing at all on a steady frame, which is the common
/// case and keeps the channel as quiet as it was before.
///
/// # It also forgets the region's last rect
///
/// Deliberately, and it is the half that is easy to omit. Without it, a region
/// that disappears and later comes back **at the same rect** would emit
/// nothing on its return — `record_rect_if_changed` would compare against the
/// remembered value and suppress it — leaving the trace saying the region went
/// away and never saying it returned. Forgetting on retirement makes a
/// reappearance always visible.
pub fn end_ui_frame() {
    if !enabled() {
        return;
    }
    frame_tick();
    let mut this = lock(&UI_RECTS_THIS_FRAME);
    let mut last = lock(&UI_RECTS_LAST_FRAME);
    let mut retired: Vec<String> = last.difference(&this).cloned().collect();
    if !retired.is_empty() {
        // Sorted so a diff between two runs of the same scenario is stable.
        // `HashSet` iteration order is not, and an unstable trace is one that
        // cannot be compared against a previous capture.
        retired.sort();
        let mut rects = lock(&LAST_UI_RECT);
        for name in &retired {
            rects.remove(name);
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            eprintln!("pdfcer-diag ui-rect-gone name={name}");
        }
    }
    std::mem::swap(&mut *last, &mut this);
    this.clear();
}

/// The decision [`ui_rect`] makes, over an explicit map — see
/// [`record_if_changed`] for why it is split out this way.
///
/// The comparison is exact rather than epsilon-based, deliberately. An
/// unmoved region is laid out from the same inputs every frame and produces a
/// bit-identical `Rect`; a region that moved by a quarter of a point moved,
/// and a check measuring it wants to know. There is no third case in which an
/// epsilon would help.
fn record_rect_if_changed(
    map: &mut HashMap<String, egui::Rect>,
    name: &str,
    rect: egui::Rect,
) -> bool {
    if map.get(name).is_some_and(|prev| *prev == rect) {
        return false;
    }
    // Only allocates when the region is new or has actually moved.
    map.insert(name.to_owned(), rect);
    true
}

/// Forget every de-duplication slot, so the next frame re-declares
/// everything.
///
/// Called when a document is opened. Without it, opening a second document
/// whose layout happens to be identical to the first would emit **no** canvas
/// line for the new document, and §4.3 requirement 1 is specifically *"at
/// least once per document open"* — a guarantee the consumer is entitled to
/// read as "there is a line for this document", not "there is a line for some
/// document whose numbers still happen to apply".
///
/// It is cheap and it is not per-frame, so it clears both registries rather
/// than trying to decide which slots a document open could have invalidated.
pub fn reset_change_gates() {
    if !enabled() {
        return;
    }
    lock(&LAST_LINE).clear();
    lock(&LAST_UI_RECT).clear();
    // The frame census goes with them. Not clearing it would make the first
    // frame after a document open emit `ui-rect-gone` for every region of the
    // PREVIOUS document that the new one happens not to draw yet — a burst of
    // retirements that describe a document nobody has open, at the one moment
    // a reader is most likely to be looking.
    lock(&UI_RECTS_THIS_FRAME).clear();
    lock(&UI_RECTS_LAST_FRAME).clear();
}

/// The values last emitted under each `trace_on_change` key.
static LAST_BY_KEY: LazyLock<Mutex<std::collections::HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));

/// Emit `key value` **only when `value` differs from the last one under `key`**.
///
/// # Why this exists beside [`trace`]
///
/// Some facts are worth reporting and are only true per frame — whether a text
/// draft exists, whether the keyboard is owned, how long the draft is. Tracing
/// those with [`trace`] produces a line every frame at sixty hertz, which is
/// not a diagnostic; it is a denial of service on the reader, and the reader is
/// somebody already having a bad day.
///
/// A change log is the honest shape for a *state* rather than an *event*, and
/// this crate already has one: [`ui_rect`] emits only when a rect moves. This
/// is the same idea for a string, keyed so several callers can use it without
/// interfering.
///
/// ★ **It has [`ui_rect`]'s known weakness and it is stated rather than
/// discovered.** A change log cannot report that something *stopped* — see
/// [`end_ui_frame`], added after that exact gap made `ui-verify` report
/// eighteen controls as mislaid. Here the equivalent is a state that ceases:
/// the last line stands, and a reader must not take it for "still true". Where
/// that matters, include the *ceasing* in the value — `draft=false` is a value,
/// not an absence, which is why the text-edit line reports it that way.
///
/// The closure is not called at all when tracing is off, exactly as [`trace`]'s
/// is: the whole cost of a disabled diagnostic is one atomic read.
pub fn trace_on_change(key: &str, value: impl FnOnce() -> String) {
    if !enabled() {
        return;
    }
    let value = value();
    let mut last = lock(&LAST_BY_KEY);
    if last.get(key).is_some_and(|previous| *previous == value) {
        return;
    }
    last.insert(key.to_owned(), value.clone());
    drop(last);
    // ui-text-exempt: diagnostic trace, never displayed in the UI
    eprintln!("pdfcer-diag {key} {value}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`trace`] must not evaluate its closure when tracing is off.
    ///
    /// This is the property that lets call sites be left in permanently:
    /// the moment a disabled trace still formats its message, every one of
    /// them becomes a per-frame allocation and the next engineer starts
    /// deleting them again.
    ///
    /// The test is written so it is meaningful in BOTH environments: if the
    /// harness itself runs under `PDFCER_DIAG`, the closure is expected to
    /// run, so the assertion follows `enabled()` rather than assuming it.
    #[test]
    fn a_disabled_trace_never_builds_its_message() {
        let mut built = false;
        trace(|| {
            built = true;
            String::new()
        });
        assert_eq!(built, enabled());
    }

    /// [`trace_changed`] must not evaluate its closure when tracing is off,
    /// for exactly the same reason as [`trace`].
    #[test]
    fn a_disabled_change_gated_trace_never_builds_its_message() {
        let mut built = false;
        trace_changed("test-disabled", || {
            built = true;
            String::new()
        });
        assert_eq!(built, enabled());
    }

    /// The property the gate exists for: a repeated identical line is
    /// emitted once.
    ///
    /// This is the fix for the measured defect — 50 identical
    /// `canvas-pointer` lines in 9 seconds with the pointer stationary.
    #[test]
    fn an_unchanged_line_is_emitted_once_and_then_suppressed() {
        let mut map = HashMap::new();
        assert!(
            record_if_changed(
                &mut map,
                "canvas-pointer",
                "canvas-pointer screen=(1.0,2.0)"
            ),
            "the first sighting of a value is news and must be emitted"
        );
        for _ in 0..50 {
            assert!(
                !record_if_changed(
                    &mut map,
                    "canvas-pointer",
                    "canvas-pointer screen=(1.0,2.0)"
                ),
                "an unchanged value tells the consumer nothing the previous line did not"
            );
        }
    }

    #[test]
    fn a_changed_line_is_emitted_again() {
        let mut map = HashMap::new();
        assert!(record_if_changed(&mut map, "canvas", "canvas zoom=1.0"));
        assert!(record_if_changed(&mut map, "canvas", "canvas zoom=1.5"));
        // …and the new value is now the one that suppresses.
        assert!(!record_if_changed(&mut map, "canvas", "canvas zoom=1.5"));
        assert!(
            record_if_changed(&mut map, "canvas", "canvas zoom=1.0"),
            "returning to a previously seen value is a change, not a repeat: the \
             consumer's last line says 1.5"
        );
    }

    /// Two slots must not suppress each other. The whole point of a slot is
    /// that one noisy call site cannot silence another.
    #[test]
    fn slots_are_independent() {
        let mut map = HashMap::new();
        assert!(record_if_changed(&mut map, "a", "same text"));
        assert!(
            record_if_changed(&mut map, "b", "same text"),
            "an identical line under a different slot is a different fact"
        );
    }

    /// A region is declared once, then again only when it actually moves.
    #[test]
    fn a_ui_rect_is_declared_once_per_layout_change() {
        use egui::{Pos2, Rect};
        let mut map = HashMap::new();
        let a = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 20.0));
        let moved = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 21.0));

        assert!(record_rect_if_changed(&mut map, "page", a));
        assert!(!record_rect_if_changed(&mut map, "page", a));
        assert!(
            record_rect_if_changed(&mut map, "page", moved),
            "a one-point resize moved the pixels a legibility check measures"
        );
        assert!(
            record_rect_if_changed(&mut map, "canvas-viewport", a),
            "regions are keyed by name; one must not suppress another"
        );
    }
}
