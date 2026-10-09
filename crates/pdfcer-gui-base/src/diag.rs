//! # diag — an opt-in trace of what the shell actually received
//!
//! This file is the channel itself — [`enabled`], [`trace`], and the
//! change-gated writers built on them. Nothing in it is a feature; it is the
//! instrument every other module of the application is measured with, and the
//! contract below is what keeps it safe to leave switched on in the source.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/diag.md`.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard, OnceLock};

/// Whether tracing was requested for this process.
pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| {
        // ui-text-exempt: environment variable name, never displayed
        std::env::var_os("PDFCER_DIAG").is_some_and(|v| !v.is_empty())
    })
}

thread_local! {
    /// Set while [`muted`] runs its closure.
    static MUTED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether a writer should emit now: tracing is on and not [`muted`].
fn on() -> bool {
    enabled() && !MUTED.get()
}

/// Run `f` with every writer in this module silenced on this thread.
///
/// For a caller that reuses a planning path which traces what it decides,
/// such as a preview built from the commit's own plan. Without it the trace
/// would record a decision that was never acted on.
///
/// Also for a pass drawn only to be read, such as the settings window's
/// search index: its regions and lines describe nothing on screen.
pub fn muted<R>(f: impl FnOnce() -> R) -> R {
    let was = MUTED.replace(true);
    let out = f();
    MUTED.set(was);
    out
}

/// Emit one trace line, building the message only if tracing is on.
pub fn trace(f: impl FnOnce() -> String) {
    if on() {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        eprintln!("pdfcer-diag {}", f());
    }
}

// ---------------------------------------------------------------------------
// The de-duplicating gate
// ---------------------------------------------------------------------------

/// The last line emitted under each [`trace_changed`] slot.
static LAST_LINE: LazyLock<Mutex<HashMap<&'static str, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// The last rect emitted for each named region by [`ui_rect`].
static LAST_UI_RECT: LazyLock<Mutex<HashMap<String, egui::Rect>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// The region names [`ui_rect`] has been called with **so far this frame**.
static UI_RECTS_THIS_FRAME: LazyLock<Mutex<std::collections::HashSet<String>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));

/// The region names that were drawn during the **previous** frame.
static UI_RECTS_LAST_FRAME: LazyLock<Mutex<std::collections::HashSet<String>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));

/// Lock a registry, ignoring poisoning.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Emit one trace line **only when it differs from the last line emitted
/// under the same slot**.
pub fn trace_changed(slot: &'static str, f: impl FnOnce() -> String) {
    if !on() {
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
/// # Why a scroll area needs this, and why the plain call is a trap there
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
/// That is not hypothetical: `settings_headings_legible` — the regression
/// check for `DEFECTS.md` **D2** — measures headings in a scrolled dialog, and
/// an ungated declaration hands it the Pages panel and the drawing behind the
/// window instead. Those read as illegible against the 3:1 floor while the
/// headings genuinely on screen measure **13.91:1**.
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
/// # The test is MOSTLY VISIBLE, not bare intersection
///
/// Bare intersection is the tempting rule — a heading half-scrolled off the
/// bottom is still partly on screen, and a contrast check samples what it can
/// reach — but it is measurably wrong at the boundary. A settings heading
/// sitting two points inside the scroll area's bottom edge is 5.3 % visible,
/// and a contrast sampler reads **1.53:1** off the anti-aliased top rows of
/// glyphs whose bodies are clipped away, in a dialog whose other headings
/// measure 15.07:1.
///
/// So the test is a *proportion*: a region must be at least
/// [`VISIBLE_FRACTION`] inside the clip before it is worth naming. Both ends of
/// the argument survive — a heading three-quarters visible is still measured,
/// and full containment is still not required — but a sliver is not offered to
/// a sampler as though it were a surface.
///
/// ⇒ The general form: **a measurement of the wrong surface is
/// indistinguishable from a measurement of a broken one.** A capture of the
/// wrong window and a capture of the wrong part of the right one fail the same
/// way. A diagnostic channel that publishes a region nobody can read is not
/// being generous, it is manufacturing false failures.
/// How much of a region must be inside the clip before it is published.
///
/// Three fifths, and the number is a judgement rather than a measurement: it is
/// low enough to keep a heading that is mostly there and high enough to drop
/// the sliver that produced a false 1.53:1. A heading is a row of glyphs about
/// two-thirds the height of its rect, so at 0.6 the glyph bodies are inside the
/// clip whichever end is cut.
const VISIBLE_FRACTION: f32 = 0.6;

/// **The verdict, on its own, with no side effect.**
#[must_use]
pub fn visible_enough(rect: egui::Rect, clip: egui::Rect) -> bool {
    let shown = clip.intersect(rect);
    let area = rect.width() * rect.height();
    let visible = shown.width().max(0.0) * shown.height().max(0.0);
    area > 0.0 && visible / area >= VISIBLE_FRACTION
}

/// Returns whether the region was published — i.e. whether it is visible
/// enough to be worth naming. **Not** whether the channel is on: a caller
/// asking "can the operator see this?" gets the same answer with
/// `PDFCER_DIAG` unset, which is what makes the answer testable.
pub fn ui_rect_visible(name: &str, rect: egui::Rect, clip: egui::Rect) -> bool {
    // The verdict is computed BEFORE the `enabled()` short-circuit, which
    // costs six floating-point operations per region on a channel-off build.
    // That is deliberate and it is cheap: the alternative is a function whose
    // return value means "visible" when the channel is on and "no" when it is
    // off, i.e. a two-valued answer to a three-valued question, and every test
    // written against it would be asserting the environment rather than the
    // layout. `ui_rect` below still returns immediately when the channel is
    // off, so the map lock and the `format!` — the parts that actually cost
    // anything — are unchanged.
    if !visible_enough(rect, clip) {
        report_clipped(name, rect, clip);
        return false;
    }
    ui_rect(name, rect);
    true
    // NOT silent when the region fails the test - see [`report_clipped`] for
    // why an absence is not an answer. It is still not a retirement:
    // `end_ui_frame` owns that, and a region that scrolls out of view and back
    // is exactly the case it was built for - it emits `ui-rect-gone` on the
    // frame the region stops being declared, and the rect is re-emitted when
    // it returns.
}

/// **Why a region was not published, on the frame the answer changes.**
///
/// ```text
/// pdfcer-diag ui-rect-clipped name=<region> rect=[[..]] clip=[[..]] shown=0.41 floor=0.60
/// ```
///
/// # Why silence is not an answer
///
/// [`ui_rect_visible`] answers a real question, *can the operator see this?*,
/// and answering `no` by saying nothing at all makes the two causes
/// indistinguishable: a driven check reading the trace cannot tell a region
/// that was never drawn from one that drew and was clipped, and they have
/// completely different fixes. A check reporting the first sends a reader into
/// the drawing code — which is correct — while the section drew perfectly and
/// only its own bounding box failed [`visible_enough`].
///
/// ⇒ **A diagnostic channel that declines to publish owes the reason.** The
/// rule generalises past this function: an unevidenced absence reads as an
/// answered question, and a reader who believes it goes looking somewhere
/// else.
///
/// # Why `trace_on_change` and not `eprintln!`
///
/// Because a region clipped out of view is clipped out of view on every frame
/// until something moves, and a per-frame line at sixty hertz is not a
/// diagnostic. The key carries the region name so two regions cannot suppress
/// each other, and the value carries the three numbers that decide the
/// verdict, so a changed layout re-reports rather than staying quiet on a
/// stale line.
///
/// ⚠ It inherits a change log's known weakness, stated rather than
/// discovered: this line does NOT retract when the region becomes visible
/// again. The retraction is the `ui-rect` line that then appears for the same
/// name, and a reader comparing the two must compare their ORDER.
fn report_clipped(name: &str, rect: egui::Rect, clip: egui::Rect) {
    if !on() {
        return;
    }
    let shown = clip.intersect(rect);
    let area = rect.width() * rect.height();
    let visible = shown.width().max(0.0) * shown.height().max(0.0);
    // A zero-area region is reported as `shown=0.00` rather than as a division
    // by zero. `visible_enough` already calls it invisible; this only has to
    // name it in a way a reader and a check can both parse.
    let fraction = if area > 0.0 { visible / area } else { 0.0 };
    // ui-text-exempt: diagnostic trace, never displayed in the UI
    trace_on_change(&format!("ui-rect-clipped name={name}"), || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("rect={rect:?} clip={clip:?} shown={fraction:.2} floor={VISIBLE_FRACTION:.2}")
    });
}

/// **A named control's rectangle, and whether it would respond to a press.**
///
/// ```text
/// pdfcer-diag ui-rect    name=properties.choice_opts.row0.up rect=[[..] - [..]]
/// pdfcer-diag ui-control name=properties.choice_opts.row0.up enabled=false
/// ```
///
/// Returns what [`ui_rect_visible`] returns: whether the region was published.
///
/// # What the second line is for, and why an assertion needs it
///
/// **A correctly greyed control and a broken one are identical to a driven
/// check.** A harness can press the control and observe that nothing changed,
/// and that outcome is produced equally by a control that was disabled and by
/// one that was live, reported `clicked()`, and had its result thrown away by
/// a condition written behind the call. The second is a defect this shell has
/// shipped; `D:/dev/rag/egui/` records it, and the instrument it asks for is
/// this one.
///
/// So the line carries the one fact the wrong mechanism cannot produce.
/// [`egui::Response::enabled`] is false **only** for a widget allocated inside
/// a disabled `Ui` — never for one merely painted grey, and never for one given
/// a weaker `Sense`, both of which leave the response reporting itself enabled.
/// `enabled=false` is therefore evidence about the mechanism and not only about
/// the appearance.
///
/// # Why it takes the response rather than a rect and a flag
///
/// So the pair cannot be half-published. A call site that emits the rectangle
/// and forgets the state leaves a check reading a region it has no way to
/// judge, and that omission is indistinguishable from a control which only
/// ever has one state.
pub fn ui_control(name: &str, response: &egui::Response, clip: egui::Rect) -> bool {
    let published = ui_rect_visible(name, response.rect, clip);
    // Emitted whether or not the rectangle was published, because being
    // clipped and being disabled are independent facts and a reader asking
    // the second one deserves an answer either way.
    //
    // ui-text-exempt: diagnostic trace, never displayed in the UI
    trace_on_change(&format!("ui-control name={name}"), || {
        // `Display`, not `Debug`: a machine reads this field, and a `{:?}`
        // wrapper is something a parser has to strip and sometimes does not.
        //
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("enabled={}", response.enabled())
    });
    published
}

/// **Where a child viewport's client area sits on the DESKTOP.**
///
/// ```text
/// pdfcer-diag viewport-inner id=<hash> rect=[[x0 y0] - [x1 y1]]
/// ```
///
/// # Why this line has to exist, and what breaks silently without it
///
/// [`ui_rect`] publishes a named region's rectangle **relative to the viewport
/// that drew it**, and a harness converting to desktop coordinates has to add
/// that viewport's client origin. With one window there is nothing to get
/// wrong; `pdfcer_gui::dialogs::host` makes a dialog a real OS window, and its
/// regions publish rectangles that look exactly like the application window's
/// while naming a completely different place on the desktop — typically by the
/// few hundred points between the two windows' corners.
///
/// **That is a coordinate-space defect with plausible numbers**, a class
/// `D:/dev/rag/egui/` records twice on this project: invisible to every unit
/// test, and presenting only as *"the click lands somewhere else"*. This line
/// removes the assumption instead of asking for care — the harness is handed
/// the child's origin.
///
/// It is also the only way a check can **assert that a dialog opened in its own
/// window at all**, which is what makes `ui-conventions/dialogs.md` G1 testable
/// rather than a matter of looking. A build that reverted to an in-viewport
/// panel emits no `viewport-inner` line, and its absence is the failure.
///
/// Emitted on change only, like every other line in this file, so a dialog
/// sitting still costs nothing per frame and a dragged one reports its travel.
pub fn viewport_inner(id: egui::ViewportId, rect: egui::Rect) {
    if !on() {
        return;
    }
    // Keyed by id, so two dialogs open at once are two independent change
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

/// **Where a child viewport's whole WINDOW sits on the desktop, decoration
/// included.**
///
/// ```text
/// pdfcer-diag viewport-outer id=<hash> rect=[[x0 y0] - [x1 y1]]
/// ```
///
/// # Why the inner rectangle cannot answer the question this one answers
///
/// [`viewport_inner`] publishes the **client** area, and the client area is not
/// the quantity any window-placement API speaks.
/// `egui::ViewportBuilder::with_position` takes the **outer** corner while
/// `with_inner_size` takes the client extent, so a check asking *"did the window
/// open where the affordance promised"* is comparing a promise expressed in
/// outer points against a measurement expressed in client points. On Windows 11
/// those differ by the border and the title bar — measured here as 8 pt across
/// and 31 pt down — and a harness reconciling them has to encode the host's
/// chrome in the instrument, where it silently becomes wrong on the next
/// platform, the next theme, or the next undecorated window.
///
/// The failure this closes is the one `viewport_inner`'s own doc comment names
/// as a class: **a coordinate-space defect with plausible numbers.** A tear-out
/// outline is drawn in *application-window* points and converted to desktop
/// points by adding the application's own origin. Drop that addition and the
/// window opens hundreds of points from the outline the operator was shown —
/// while the client rectangle is still exactly the promised size, still on the
/// desktop, still a real window. Every assertion available without this line
/// goes on passing.
///
/// Emitted on change only and keyed by id, on the same terms as
/// [`viewport_inner`], so the two lines for one window move together and a
/// window sitting still costs nothing per frame.
pub fn viewport_outer(id: egui::ViewportId, rect: egui::Rect) {
    if !on() {
        return;
    }
    let key = format!("viewport-outer:{:?}", id);
    lock(&UI_RECTS_THIS_FRAME).insert(key.clone());
    if record_rect_if_changed(&mut lock(&LAST_UI_RECT), &key, rect) {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        eprintln!("pdfcer-diag viewport-outer id={:?} rect={rect:?}", id);
    }
}

thread_local! {
    /// Which viewport's coordinate space [`ui_rect`] is currently publishing
    /// in, or `None` for the application's own window.
    static VIEWPORT: std::cell::RefCell<Option<String>> = const {
        std::cell::RefCell::new(None)
    };
}

/// **Publish every [`ui_rect`] inside this scope as belonging to `id`.**
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
    if !on() {
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
const FRAME_TICK_EVERY: u64 = 10;

/// **A monotonic frame counter on the diagnostic channel.**
///
/// ```text
/// pdfcer-diag frame n=1230
/// ```
///
/// # Why this exists, and it is a fix for a whole class of false failure
///
/// A `settle(frames)` implemented as `sleep(frames * 25ms)` is a **wall clock
/// wearing the word "frames"**. On an idle machine 25 ms is about a frame and
/// the name is nearly true; under load it is not — the application renders
/// fewer frames in the same wall time, so a check that "settles" and then
/// clicks acts before the interface has caught up.
///
/// The failures that produces are the expensive kind: substantive, believable
/// messages — a bookmark that went to the page and did not zoom, a canvas that
/// stopped seeing the pointer, a list of rows that never drew — that pass when
/// re-run alone against the same binary. "Contention" explains nothing and
/// excuses everything; the mechanism is that the harness measured a UI that had
/// not finished responding.
///
/// ⇒ With a counter on the channel, `Session::settle` waits for the
/// application to actually **produce** frames: fast when idle and patient when
/// loaded, which is what the name always claimed.
///
/// Only under `PDFCER_DIAG`, like everything here, and only every tenth frame.
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

/// A frame that held the UI thread at least this long is traced as `frame-long`.
const LONG_FRAME: std::time::Duration = std::time::Duration::from_millis(50);

thread_local! {
    /// When the frame now being built started, set by [`begin_ui_frame`].
    static FRAME_STARTED: std::cell::Cell<Option<std::time::Instant>> =
        const { std::cell::Cell::new(None) };
    /// The last [`frame_phase`] mark, or the frame's start.
    static PHASE_MARK: std::cell::Cell<Option<std::time::Instant>> =
        const { std::cell::Cell::new(None) };
    /// This frame's phases so far: name and milliseconds.
    static PHASES: std::cell::RefCell<Vec<(&'static str, u128)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// A phase shorter than this is left out of a `frame-long` line.
const PHASE_SHOWN_MS: u128 = 5;

/// Mark the top of a frame, so [`end_ui_frame`] can time it.
///
/// ```text
/// pdfcer-diag frame-long ms=182 phases=docks:131,settle:40
/// ```
///
/// `phases=` names each [`frame_phase`] section of 5 ms or more, `-` for none.
///
/// The time is the application's own work between the top of `ui` and the end
/// of the frame — what holds the next input back — not paint or upload time.
pub fn begin_ui_frame() {
    if on() {
        let now = std::time::Instant::now();
        FRAME_STARTED.set(Some(now));
        PHASE_MARK.set(Some(now));
        PHASES.with_borrow_mut(Vec::clear);
    }
}

/// Close the frame section that ends here, under `name`, for the
/// `frame-long` line's `phases=`. A no-op unless tracing.
pub fn frame_phase(name: &'static str) {
    let Some(mark) = PHASE_MARK.get() else {
        return;
    };
    let now = std::time::Instant::now();
    PHASE_MARK.set(Some(now));
    PHASES.with_borrow_mut(|p| p.push((name, (now - mark).as_millis())));
}

/// Trace the frame [`begin_ui_frame`] opened if it took [`LONG_FRAME`] or more.
fn trace_long_frame() {
    let Some(started) = FRAME_STARTED.take() else {
        return;
    };
    PHASE_MARK.set(None);
    let phases = PHASES.take();
    let took = started.elapsed();
    if took >= LONG_FRAME {
        let shown: Vec<String> = phases
            .iter()
            .filter(|(_, ms)| *ms >= PHASE_SHOWN_MS)
            .map(|(name, ms)| format!("{name}:{ms}"))
            .collect();
        let shown = if shown.is_empty() {
            "-".to_owned()
        } else {
            shown.join(",")
        };
        let ms = took.as_millis();
        eprintln!("pdfcer-diag frame-long ms={ms} phases={shown}"); // ui-text-exempt: diagnostic trace
    }
}

/// **Close a frame's region census and report anything that stopped being
/// drawn.**
pub fn end_ui_frame() {
    if !on() {
        return;
    }
    frame_tick();
    trace_long_frame();
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
pub fn reset_change_gates() {
    if !on() {
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
pub fn trace_on_change(key: &str, value: impl FnOnce() -> String) {
    if !on() {
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

/// Indices comma-joined for a trace field; `-` for none, so the field is
/// never empty and still parses as one token.
pub fn index_list(indices: &[usize]) -> String {
    if indices.is_empty() {
        return "-".to_owned();
    }
    indices
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`trace`] must not evaluate its closure when tracing is off.
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
    /// The measurement behind it: an ungated `canvas-pointer` emits 50
    /// identical lines in 9 seconds with the pointer stationary.
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
