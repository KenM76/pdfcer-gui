//! `a_view_carried_off_the_sheet_comes_back` — O186 stage one, made
//! falsifiable.
//!
//! # The report
//!
//!
//! > *"the canvas will just stop zooming in"*
//!
//! and, separately, that the page can disappear entirely with no way back.
//!
//! # What is wrong — the theory this check was written against, and what it measured
//!
//!
//! ## The theory: an unclamped deep anchor
//!
//! Above the deep threshold — `longest_page_pt × zoom` over
//! `viewer::ceiling::SUB_PIXEL_CONTENT_EXTENT` — the view's position stops
//! being an `f32` `egui` scroll offset and becomes a
//! `viewer::deep::DeepAnchor`: an `f64` page point plus the screen pixel it
//! sits under. The `egui` scroll area clamps its offset to the content it
//! scrolls; **nothing clamped the anchor.** So an anchor whose page point is
//! *off the sheet* can be magnified until the sheet itself is thousands of
//! viewports away, and the canvas draws nothing. That was real; it is fixed by
//! `canvas::deep::confine`; and this check's passing run traced the clamp
//! firing on `y` nine times on the way up to 107,105,325 %.
//!
//! ## ★★★ The measurement: his blank frame is two orders of magnitude below that
//!
//! Driven against the pre-fix binary. The canvas went blank after **thirty**
//! notches at a peak zoom of **3,981 %**, with `tier=scroll` on every frame,
//! the deep tier never reached, and `confine` never called. This A1 sheet's
//! `f64` hand-over is at a zoom of about **440** — that is **44,000 %**.
//!
//! So the clamp on the anchor cannot be what the operator hit, and **a fix
//! confined to it would have left his own reproduction untouched while every
//! instrument in this crate went green.**
//!
//! The cause is one term in `canvas::geometry::pasteboard`, documented in full
//! at `MIN_SHEET_ON_SCREEN` rather than restated here. In short: the pasteboard
//! was **exactly one viewport**, which makes the two extremes of
//! `geometry::visible_origin_range` the placements at which viewport and strip
//! touch with *zero* overlap — at both ends, on both axes, **at every zoom**.
//! Narrowing the range by a sliver fixes it, and because every other bound in
//! that module is derived from `pasteboard`, one subtraction covers the shallow
//! tier the operator hit, the deep tier's new clamp, **and** a scroll bar
//! dragged to its stop.
//!
//! ★ The scroll-bar route deserves a sentence of its own: it was blank too,
//! reachable at any zoom with no deep tier involved and no pointer off the
//! sheet, and nobody had ever reported it — because a scroll bar sitting at its
//! own stop does not feel like a defect.
//!
//! That state was **terminal**, and the reason it was terminal is worth
//! stating because it is not obvious from the symptom:
//!
//! * `canvas::present` returns early when no page was drawn, *above* every
//!   input handler in the function. Only the movers inside
//!   `canvas::deep::track` run, because `viewpos::position` is called higher
//!   up.
//! * Those movers are a pan and a plain wheel. `smooth_scroll_delta` is zero
//!   while Ctrl is held, so the deep pan route cannot zoom out at all.
//! * `DeepAnchor::panned(delta, zoom)` moves the page point by `delta / zoom`.
//!   At zoom 540 one plain wheel notch is about 0.09 pt. Escaping by panning
//!   would take roughly seven and a half thousand rolls.
//!
//! ★ **That terminality belongs to the deep tier, and the operator's blank
//! frame was not in the deep tier** — so on his own reproduction the escape
//! hatch was never the thing standing between him and a page. Measured: from
//! the shallow-tier blank at 3,981 %, eighty Ctrl+wheel notches out drew a page
//! again at 10 %. The hatch works. It simply was not the broken half, and the
//! argument above describes a state he had to climb two orders of magnitude
//! further to reach.
//!
//! # What this check drives, and why it is shaped like this
//!
//! Aiming Ctrl+wheel at a point **above the sheet's top edge** is the whole
//! reproduction. `DeepAnchor::zoomed_about(at, from_zoom)` re-states the anchor
//! as *"the page point that was under `at` at the old zoom, pinned to `at`"* —
//! so every notch re-seeds the anchor from whatever is under the pointer, and
//! if that is blank pasteboard above the sheet the anchor's page y is
//! **negative** and grows in screen magnitude with the zoom.
//!
//! ★ `DocPoint` y is PDF y-up; `DeepAnchor.page` y is y-down from the page's
//! top-left. A point *above* the sheet is therefore a **high** `DocPoint` y and
//! a **negative** anchor page y. That sign flip is what made the measured
//! anchor `(1199.50, −0.54)` read as "almost on the page" when it was in fact
//! half a point off the top of it, with a zoom of several hundred multiplying
//! that half point into a view that had left the sheet.
//!
//! ⚠ The magnitude of the initial aim is deliberately **not** load-bearing —
//! and ★ the reason this paragraph originally gave for that is the sentence
//! that had the defect written into it. Below the deep threshold the `egui`
//! scroll area clamps the offset to its own content range, so however far above
//! the sheet the first notch aims, by the time the threshold is crossed the
//! page's top edge is **at most one pasteboard below the viewport** — which is
//! why the measured anchor was a fraction of a point off the page rather than
//! the eighty-odd points this check starts by aiming at.
//!
//! ★★★ That clause was written as a reassurance. It **is** the defect. One
//! pasteboard below the viewport was one whole *viewport* below it, which is
//! precisely the placement at which the sheet occupies zero of the canvas — so
//! the harmless-sounding bound the aim converges to was itself the blank frame,
//! and it is reached long before there is an anchor to clamp. Every ingredient
//! of the diagnosis was in that sentence and it was read as an all-clear.
//!
//! The conclusion survives: the aim only has to be *above the sheet at all*,
//! and to be reachable on screen; see [`OFF_PAGE_FRACTION`] and
//! [`ZOOM_OUT_NOTCHES`]. But it now holds because `MIN_SHEET_ON_SCREEN` keeps
//! that convergence point **visible**, not because the convergence point was
//! ever safe.
//!
//! # The three verdicts, and the order they are asked in
//!
//! The order is load-bearing and is the opposite of the order the sentences
//! suggest:
//!
//! 1. **Did the canvas go blank?** `canvas-unavailable reason=nothing-visible`
//!    after the climb began is the defect, and it FAILS. This is asked first so
//!    that a build with the clamp removed goes **red**, not SKIPPED — a
//!    falsification run must be able to fail.
//! 2. **Was a raster ceiling learned while it was blank?** The non-obvious
//!    half. The zoom ceiling is learned from what the renderer refused, and a
//!    blank canvas refuses everything, so a ceiling learned in this state is
//!    learned from the defect and then **outlives the fix**.
//! 3. **Did the zoom keep climbing?** The operator's own sentence. A run whose
//!    peak zoom never passed [`ZOOM_FLOOR`] reproduces *"the canvas will just
//!    stop zooming in"* whether or not anything went blank.
//!
//! and only then the two vacuity guards — *did the run reach the deep tier*,
//! and *did the clamp ever actually fire* — which SKIP rather than pass,
//! because a run that never drove the anchor out of range has measured nothing
//! about a mechanism whose only job is to catch an anchor out of range.
//!
//! ★★★ **The first run of this check answered the open design question it was
//! written to settle, and the answer was worse than the question allowed for.**
//!
//! The question: clamping the anchor to the ends of
//! `geometry::visible_origin_range` parks the page at a placement where
//! viewport and sheet touch with **zero overlap** — a near-blank view. The
//! draft's defence was that the shallow tier permits exactly the same placement
//! (`egui` clamps scroll to `content − viewport`, and with a full-viewport
//! pasteboard the maximum offset shows a viewport of grey beyond the strip), so
//! it is pre-existing behaviour rather than an O186 regression, and tier
//! agreement is the design's whole correctness claim.
//!
//! **Every clause of that defence is true and the conclusion is wrong.** The
//! placement is a near-blank view; the shallow tier does permit it; the tiers
//! do agree — and what they agree on is a blank frame. Verdict 1 fired against
//! the build of the day at the **shallow** tier, thirty notches in, before the
//! clamp existed to be exercised. That is what sent the fix to
//! `geometry::pasteboard` instead of to either tier.
//!
//! ★ The lesson to carry, because it generalises past this defect: *"the other
//! tier does the same thing"* is an argument about **blame**, not about
//! correctness, and a clamp whose range endpoints are themselves the failure
//! state will park the view on that failure and report itself as having
//! confined it.
//!
//! # What this check does NOT cover, stated rather than implied
//!
//! `canvas::escape::offer` restores two gestures above the blank-frame early
//! return: `paging::flip` and `zoom::wheel_step`. **Only the second of those
//! has any coverage here, and the first has none anywhere.**
//!
//! `DESIGNS.md`'s own obligation for this stage asked for a check that
//! *"confirms a page-flip gesture does nothing"* on the old behaviour. That
//! obligation is **wrong**, and it is worth recording why rather than quietly
//! satisfying it: `paging::flips_pages` requires
//! `prefs.wheel_paging.flips()`, and `WheelPaging::Scroll` carries
//! `#[default]`. The flip route is off on any build nobody has configured, so
//! "confirm the page-flip gesture does nothing" is satisfied by a correctly
//! implemented preference and would have been recorded as evidence about the
//! clamp.
//!
//! The route that distinguishes the builds is **Ctrl+wheel out**, through
//! `escape::offer` → `zoom::wheel_step`, which is what [`RECOVER_NOTCHES`]
//! drives. The paging route needs a non-default preference *and* a blank frame,
//! and on a fixed build the clamp prevents the blank frame — so it is only
//! reachable on a deliberately broken binary, and it is left uncovered and said
//! so here instead of being asserted vacuously.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// `VK_CONTROL`, held while the wheel rolls to make it a zoom.
///
/// ★ Plain wheel would be the wrong gesture twice over: it scrolls rather than
/// zooms below the threshold, and above it `smooth_scroll_delta` is what the
/// deep pan route reads — so a plain-wheel climb would exercise the pan this
/// check is not about and never reach the depth it is about.
const VK_CONTROL: u16 = 0x11;

/// The fixture, relative to the workspace root.
///
/// ★★ **A1 landscape, and the size is the reason.** Page 0 of this document is
/// `/MediaBox [0 0 2383.937 1683.78]`, so the deep threshold —
/// `longest_page_pt × zoom` over `SUB_PIXEL_CONTENT_EXTENT`, which is 2²⁰ since
/// O49 — is crossed at a zoom of about **440**. On a US Letter sheet the same
/// constant puts it at about **1,324**, which is another twenty-odd Ctrl+wheel
/// notches and several seconds of rasterizing for no additional evidence.
///
/// ★ Four pages rather than one, so the uncovered page-flip escape route named
/// in the module header at least *exists* on the document being driven. A
/// single-page fixture would make that route unreachable by construction and
/// the omission harder to see.
const FIXTURE: &str = "fixtures/four-pages.pdf";

/// The commands to ring once at startup, one per frame, via `PDFCER_DIAG_INVOKE`.
///
/// ★★ Ribbon clicks were the first design and this is better for a specific
/// reason: it removes three dependencies this check does not want. A ribbon
/// click needs the group to be uncollapsed, needs `ribbon.item.*` rects to be
/// published, and needs the View tab to be raised first — three ways for a run
/// to fail on something that is somebody else's check's subject.
/// `view.page_single` and `view.zoom_fit_page` are both asserted from the
/// canvas's own trace afterwards, so the env var is a *request* that is then
/// *verified*, not a request that is assumed.
const INVOKE: &str = "mode.review,view.page_single,view.zoom_fit_page";

/// The window size, so the arithmetic below is quoted against a known canvas.
const VIEWPORT: &str = "0,0,1600,1380";

/// The canvas's own state line, read for `zoom=`, `display=` and `drawn=`.
const CANVAS_EVENT: &str = "canvas"; // ui-text-exempt: a trace event name, never displayed

/// The canvas's `f64` position line, read for `tier=`.
const POS_EVENT: &str = "canvas-pos"; // ui-text-exempt: a trace event name, never displayed

/// ★★★ **The clamp's own trace** — `canvas-confined axes=none|x|y|xy`.
///
/// This is what lets the check say *the mechanism ran* separately from *the
/// symptom is gone*. Without it, a PASS is satisfied equally by the clamp
/// working and by the run never having driven the anchor out of range, and an
/// assertion both outcomes satisfy measures neither.
const CONFINED_EVENT: &str = "canvas-confined"; // ui-text-exempt: a trace event name

/// The blank-frame line. Its `reason=` field has several values; only one of
/// them is this defect.
const BLANK_EVENT: &str = "canvas-unavailable"; // ui-text-exempt: a trace event name

/// The one `reason=` that means *the page left the screen*.
const BLANK_REASON: &str = "nothing-visible"; // ui-text-exempt: a trace field value

/// The rasterizer's learned-ceiling line, for verdict 2.
const LEARNED_EVENT: &str = "raster-ceiling-learned"; // ui-text-exempt: a trace event name

/// The canvas scroll area, whose grey margin is the only bound an off-sheet
/// point may be converted against.
///
/// ★ Not the page's own rect. Every point this check aims at is outside that by
/// construction; bounding against it rejects the whole check with a message
/// about margin that is plausible and wrong. See
/// `CanvasMapping::doc_to_window_off_page`.
const VIEWPORT_REGION: &str = "canvas-viewport"; // ui-text-exempt: a trace region name

/// The page's own rect, asserted present so "no sheet on screen" is a clear
/// precondition failure rather than a mysterious mapping error.
const PAGE_REGION: &str = "page"; // ui-text-exempt: a trace region name

/// ★★ **Ctrl+wheel OUT this many notches before aiming**, and this is not
/// cosmetic — without it the check cannot aim at all.
///
/// Fit on an A1 landscape sheet in a 1600 × 1380 window is width-limited: the
/// canvas is about 1600 × 1150 logical points (aspect 1.39) and the page's
/// aspect is 1.416, so the sheet is drawn the full width and leaves roughly
/// **ten points** of vertical margin. A point eighty-odd page points above the
/// top edge maps above the *viewport*, and
/// `CanvasMapping::doc_to_window_off_page` correctly refuses it — refuses to
/// clamp, too, so there is no quiet wrong answer to misread.
///
/// Six notches out multiplies the zoom by about 1.2048⁻⁶ ≈ 0.29, which turns
/// ten points of margin into roughly four hundred. The aim point then sits
/// about sixteen points above the sheet with four hundred to spare, and the
/// conversion stops depending on the window size on the day.
const ZOOM_OUT_NOTCHES: usize = 6;

/// How far above the sheet's top edge to aim, as a fraction of the page height.
///
/// ⚠ Read the module header before tuning this. It is **not** the distance the
/// defect needs; the shallow tier's own scroll clamp normalises it long before
/// the deep threshold is crossed. It only has to be unambiguously above the
/// sheet — far enough that a point of `f32` rounding cannot put it back on the
/// page — and near enough that it is still inside the viewport after
/// [`ZOOM_OUT_NOTCHES`].
const OFF_PAGE_FRACTION: f64 = 0.05;

/// Total Ctrl+wheel notches to climb.
///
/// ★ Sized from the arithmetic rather than guessed. A notch multiplies the zoom
/// by about 1.2048 (derived in `deep_pan`'s `PRESSES`: twenty notches from
/// `1.0` reach about 4,155 %). Starting from 0.29 after
/// [`ZOOM_OUT_NOTCHES`], reaching the A1 sheet's deep threshold of about 440
/// takes roughly **forty** notches, and driving the anchor far enough out of
/// range to leave the viewport takes roughly **nine** more: the anchor's screen
/// offset grows by 20 % a notch and has to exceed a viewport, which from the
/// measured starting offset of a couple of hundred pixels is `ln(4.8) / ln(1.2)`
/// notches.
///
/// Eighty is that fifty with a wide margin, because every term in it is a
/// derivation and the one thing this check must not do is stop short of the
/// tier it is named after and report PASS. It costs a few seconds.
const CLIMB_NOTCHES: usize = 80;

/// How many notches per batch, between readings.
///
/// ★ Batched rather than rolled all at once because the evidence is a
/// *trajectory*: the peak zoom, the tier, the first blank frame and the first
/// clamp are all "when did this happen" questions, and a single eighty-notch
/// roll answers none of them. Ten is small enough to locate the transition to
/// within about a factor of six in zoom and large enough that the settle cost
/// is paid eight times rather than eighty.
const CLIMB_BATCH: usize = 10;

/// Frames to settle after each batch.
///
/// ★ Generous: at this depth the region rasterizer is doing real work and a
/// reading taken before the frame settles reports the *previous* batch's zoom,
/// which would make the trajectory lag the gesture by one batch and the located
/// transition wrong by a factor of six.
const SETTLE_PER_BATCH: u32 = 24;

/// ★★ **The zoom the climb must pass**, as a multiplier (1.0 = 100 %).
///
/// This is the operator's *"the canvas will just stop zooming in"*, made into a
/// number. It is deliberately set between the two outcomes rather than near
/// either: the A1 sheet's deep threshold is about **440**, the defect's own
/// measured trajectory stalled around **329** (the ceiling
/// `render::settle` learned from a refused whole-page raster at scale 438), and
/// a healthy eighty-notch climb from 0.29 ends in the hundreds of thousands.
/// A thousand is comfortably above every stall that has been measured and four
/// orders of magnitude below where a working build finishes.
///
/// ★ Both ends have since been measured on this very check, and the gap is
/// wider than the derivation assumed. The fixed build's eighty notches reached
/// **1,071,053** (107,105,325 %); the falsification build went blank at
/// **39.8** (3,981 %). This floor sits between them with a factor of 25 of
/// margin below and a factor of 1,071 above.
///
/// ⚠ Which also means this verdict would **not** have caught O186 on its own —
/// the blank arrived at 39.8, far under the floor. Verdict 1 is what fires, and
/// that is why verdict 1 is asked first. A floor on the peak zoom detects *"it
/// stopped climbing"*; it does not detect *"it climbed into a blank frame"*.
///
/// ⚠ It is a floor on the **peak** zoom the run reached, not on the final one.
/// The recovery probe deliberately rolls the zoom back down, and reading the
/// final value would assert that the recovery failed.
const ZOOM_FLOOR: f64 = 1000.0;

/// Ctrl+wheel notches to roll back OUT, as the recovery probe.
///
/// ★★ This is the only driven coverage of `canvas::escape::offer` anywhere, and
/// on a *fixed* build it is a sanity check rather than a test of the hatch —
/// the hatch exists for a blank frame, and a fixed build has none. Its value is
/// in the falsification run: against a binary with the clamp removed this check
/// reaches the blank state, and these notches are what prove a blank canvas is
/// no longer terminal. See the module header for the paging route, which has no
/// coverage and cannot have any on a fixed build.
///
/// Eighty out undoes eighty in, with the same margin and for the same reason.
const RECOVER_NOTCHES: usize = 80;

/// See the module documentation.
pub struct AViewCarriedOffTheSheetComesBack;

impl Check for AViewCarriedOffTheSheetComesBack {
    fn name(&self) -> &'static str {
        "a_view_carried_off_the_sheet_comes_back"
    }

    fn defect(&self) -> &'static str {
        "Ctrl+wheeling in at a deep zoom with the pointer just off the sheet carries the page off \
         the screen entirely, and the canvas then accepts no gesture that brings it back — the \
         f64 deep anchor is never confined to a placement from which the page is visible"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Everything one reading of the trace can say about the climb so far.
///
/// ★ A single struct built by a single function, on `deep_pan::position`'s
/// lesson: reading two of these quantities at two different moments is how this
/// harness has produced confident wrong verdicts before. Every field here comes
/// from the same parse of the same capture.
#[derive(Debug, Default, Clone)]
struct Climb {
    /// The highest `zoom=` any `canvas` line after the mark reported.
    peak_zoom: f64,
    /// The newest `zoom=` after the mark, or `None` if the canvas has said
    /// nothing since — which is a different verdict from "the same thing".
    last_zoom: Option<f64>,
    /// Did any `canvas-pos` line after the mark say `tier=deep`?
    saw_deep: bool,
    /// The distinct non-`none` `axes=` values the clamp reported, in order.
    confined: Vec<String>,
    /// The line number of the first blank frame after the mark.
    blank_at: Option<usize>,
    /// A ceiling learned *while* the canvas was blank, raw, if there was one.
    learned_while_blank: Option<String>,
}

/// Read the whole climb out of one parse of the capture.
///
/// `mark` is a [`Trace::mark`] taken before the first notch, so nothing the
/// setup did can satisfy an assertion about the climb. That anchoring is not
/// optional: the setup fits the page and zooms out six notches, each of which
/// emits `canvas` lines, and an unanchored "the canvas reported a zoom" would
/// be satisfied by the fit.
fn survey(trace: &Trace, mark: usize) -> Climb {
    let mut c = Climb::default();

    for line in trace.events(CANVAS_EVENT).filter(|l| l.lineno > mark) {
        if let Some(z) = line.get_f32("zoom") {
            let z = f64::from(z);
            c.peak_zoom = c.peak_zoom.max(z);
            c.last_zoom = Some(z);
        }
    }

    c.saw_deep = trace
        .events(POS_EVENT)
        .any(|l| l.lineno > mark && l.get("tier") == Some("deep"));

    for line in trace.events(CONFINED_EVENT).filter(|l| l.lineno > mark) {
        let axes = line.get("axes").unwrap_or("?");
        if axes != "none" && !c.confined.iter().any(|a| a == axes) {
            c.confined.push(axes.to_owned());
        }
    }

    c.blank_at = trace
        .events(BLANK_EVENT)
        .find(|l| l.lineno > mark && l.get("reason") == Some(BLANK_REASON))
        .map(|l| l.lineno);

    // ★★ Verdict 2, and the window matters. A ceiling learned *before* the
    // canvas went blank was learned from a real refusal and is legitimate; one
    // learned *after* the next drawn frame likewise. Only a line between the
    // blank frame and the next `canvas` line was learned from a canvas that
    // refused everything because there was nothing on it.
    if let Some(blank) = c.blank_at {
        let next_drawn = trace
            .events(CANVAS_EVENT)
            .find(|l| l.lineno > blank)
            .map_or(usize::MAX, |l| l.lineno);
        c.learned_while_blank = trace
            .events(LEARNED_EVENT)
            .find(|l| l.lineno > blank && l.lineno < next_drawn)
            .map(|l| l.raw.clone());
    }

    c
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check rolls the wheel with Ctrl held for a \
             hundred and sixty notches and has no static form. Reported as SKIPPED rather than \
             passed: a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its canvas is and there is no bound to convert an off-sheet point against.",
            ctx.profile.name
        ))
    })?;

    let pdf = crate::fixture::workspace_root().join(FIXTURE);
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "{FIXTURE} is missing from the repository, so this check has no A1 sheet whose deep \
             threshold is reachable in forty notches. SKIPPED."
        )));
    }
    if ctx.pdf.is_some() {
        report.note(format!(
            "--pdf was IGNORED; this check pins {FIXTURE} because the deep threshold it must \
             cross is a bound on `longest_page_pt * zoom`, so the page SIZE decides how many \
             notches the climb takes and every number in this file is quoted against A1"
        ));
    }

    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}, so the point above its top edge cannot be \
                 located. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("off-sheet.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    if let Some(name) = ctx.profile.viewport_env {
        spec.env.push((name.to_owned(), VIEWPORT.to_owned()));
    } else {
        return Err(Error::new(format!(
            "the `{}` profile declares no viewport override, so this check cannot fix the window \
             size — and every margin figure in it is quoted against a canvas of known height. \
             SKIPPED rather than measured at whatever size the window happened to open at.",
            ctx.profile.name
        )));
    }

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {} on {} at viewport {VIEWPORT}, invoking `{INVOKE}`",
        exe.display(),
        session.pid(),
        pdf.display()
    ));
    report.artifact(session.trace_path().to_path_buf());
    // Generous: `INVOKE` rings one command per frame and the last of the three
    // is a Fit, which is applied by draining the action queue at the end of the
    // frame that raised it.
    session.settle(48);

    let trace = session.trace()?;
    if !trace.started(vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process and \
             `{INVOKE}` was never rung. Captured stderr is at {}.",
            vocab.start_event,
            session.trace_path().display()
        )));
    }
    if declared(&trace, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "no `{PAGE_REGION}` region, so no sheet is on screen before the climb even starts \
             and there is nothing to carry off it. Regions beginning `page`: {}.",
            list(&declared_names(&trace, ui_rect, "page"))
        )));
    }

    // ★ VERIFY what `INVOKE` was asked for rather than assuming it. The env var
    // is a request; the canvas's own `display=` field is the answer. A run in
    // continuous display would scroll between pages during the climb and the
    // anchor would belong to a page this check is not aiming at.
    let display = trace
        .last(CANVAS_EVENT)
        .and_then(|l| l.get("display"))
        .unwrap_or("?")
        .to_owned();
    if display != "single" {
        return Err(Error::new(format!(
            "the canvas reports `display={display}` after `{INVOKE}` was rung, not `single`. \
             This check needs single-page display: in continuous display the climb would cross \
             between pages of different sizes and the deep threshold — a bound on \
             `longest_page_pt * zoom` — would be crossed for a page the aim point is not on. \
             SKIPPED rather than measured against the wrong sheet."
        )));
    }

    let driver = Driver::new(session.window());
    let frame = session.frame()?;
    let canvas = declared(&trace, ui_rect, VIEWPORT_REGION)
        .ok_or_else(|| Error::new(format!("no `{VIEWPORT_REGION}`; is the document open?")))?;
    if !canvas.is_substantial() {
        return Err(Error::new(format!(
            "the `{VIEWPORT_REGION}` region is {:.1} x {:.1} pt, which is not a laid-out canvas. \
             SKIPPED.",
            canvas.width(),
            canvas.height()
        )));
    }

    // --- win some margin -----------------------------------------------------
    //
    // See `ZOOM_OUT_NOTCHES`: at Fit this sheet leaves about ten points of
    // vertical margin and the aim point would be off the viewport, which the
    // off-page conversion correctly refuses.
    report.note(format!(
        "rolling Ctrl+wheel OUT {ZOOM_OUT_NOTCHES} notches first, to widen the grey margin above \
         the sheet from about ten points to about four hundred — at Fit there is nowhere above \
         this A1 sheet to put the pointer"
    ));
    driver.scroll_at_held(
        frame.declared_at(canvas, 0.5, 0.5),
        &[VK_CONTROL],
        -1,
        ZOOM_OUT_NOTCHES,
    )?;
    session.settle(32);

    // --- aim -----------------------------------------------------------------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, 0)?;
    let viewport = declared(&trace, ui_rect, VIEWPORT_REGION).ok_or_else(|| {
        Error::new(format!(
            "the application stopped declaring `{VIEWPORT_REGION}` after the zoom-out, so there \
             is no bound to convert an off-sheet point against. It cannot fall back to the page's \
             own rect: the aim point is outside that by construction."
        ))
    })?;
    // ★ PDF y-up. `height_pt * (1 + fraction)` is ABOVE the sheet's top edge,
    // and becomes a NEGATIVE `DeepAnchor` page y — see the module header's note
    // on the sign flip, which is why the measured anchor read `-0.54`.
    let above = page.height_pt * (1.0 + OFF_PAGE_FRACTION);
    let at_off_sheet = frame.to_screen(
        mapping.doc_to_window_off_page(DocPoint::new(0, page.width_pt / 2.0, above), viewport)?,
    );
    report.note(format!(
        "aiming the climb at document point (x = half the page width, y = {above:.1}), which is \
         {:.0} points ABOVE the sheet's {:.1}-point top edge — the pointer is over grey \
         pasteboard, which is the whole reproduction",
        page.height_pt * OFF_PAGE_FRACTION,
        page.height_pt
    ));

    // --- climb ---------------------------------------------------------------
    let mark = trace.mark();
    report.note(format!(
        "climbing {CLIMB_NOTCHES} Ctrl+wheel notches in batches of {CLIMB_BATCH}; the A1 sheet's \
         deep threshold (SUB_PIXEL_CONTENT_EXTENT / 2383.937 pt) is a zoom of about 440, reached \
         around notch forty"
    ));
    let mut climb = Climb::default();
    let mut notches = 0;
    while notches < CLIMB_NOTCHES {
        let batch = CLIMB_BATCH.min(CLIMB_NOTCHES - notches);
        driver.scroll_at_held(at_off_sheet, &[VK_CONTROL], 1, batch)?;
        notches += batch;
        session.settle(SETTLE_PER_BATCH);
        climb = survey(&session.trace()?, mark);
        report.note(format!(
            "after {notches} notches: zoom {} (peak {:.0}%), deep tier {}, confined {}{}",
            climb.last_zoom.map_or_else(
                || "— (the canvas said nothing)".to_owned(),
                |z| format!("{:.0}%", z * 100.0)
            ),
            climb.peak_zoom * 100.0,
            if climb.saw_deep { "reached" } else { "not yet" },
            if climb.confined.is_empty() {
                "never".to_owned()
            } else {
                climb.confined.join("+")
            },
            if climb.blank_at.is_some() {
                " — ★ THE CANVAS WENT BLANK"
            } else {
                ""
            }
        ));
        // ★★ Stop at the first blank frame rather than climbing on. The
        // recovery probe below has to start from the state the operator is
        // stuck in, and every further notch multiplies the zoom the hatch must
        // undo by another 20 %.
        if climb.blank_at.is_some() {
            break;
        }
    }

    // --- recover -------------------------------------------------------------
    //
    // Run BEFORE any verdict is returned, deliberately. On a build with the
    // clamp removed the blank frame above is a failure, and the failure is far
    // more useful if it can also say whether the escape hatch worked — which
    // cannot be learned after the verdict has returned and the session has been
    // dropped.
    let recover_mark = session.trace()?.mark();
    report.note(format!(
        "rolling Ctrl+wheel OUT {RECOVER_NOTCHES} notches as the recovery probe — on a blank \
         canvas this is `canvas::escape::offer` -> `zoom::wheel_step`, the only gesture above \
         the blank-frame early return that can reduce the zoom"
    ));
    driver.scroll_at_held(at_off_sheet, &[VK_CONTROL], -1, RECOVER_NOTCHES)?;
    session.settle(64);
    let after = session.trace()?;
    let recovered = after
        .last_after(CANVAS_EVENT, recover_mark)
        .and_then(|l| l.get_f32("zoom"))
        .map(f64::from);
    report.note(match recovered {
        Some(z) => format!(
            "after the recovery probe the canvas drew a page at {:.0}%",
            z * 100.0
        ),
        None => format!(
            "after the recovery probe the canvas had traced NO `{CANVAS_EVENT}` line since the \
             probe began — no page was drawn"
        ),
    });

    // --- verdicts, in the order the module header sets out --------------------

    // 1. The defect itself.
    if let Some(blank) = climb.blank_at {
        let hatch = match recovered {
            Some(z) if z < climb.peak_zoom / 10.0 => format!(
                "The escape hatch DID work: {RECOVER_NOTCHES} Ctrl+wheel notches out brought the \
                 zoom back to {:.0}% and a page was drawn again.",
                z * 100.0
            ),
            Some(z) => format!(
                "★ And the escape hatch barely moved: after {RECOVER_NOTCHES} notches out the \
                 zoom is {:.0}%, against a peak of {:.0}%. `canvas::escape::offer` is reached but \
                 is not reducing the zoom.",
                z * 100.0,
                climb.peak_zoom * 100.0
            ),
            None => format!(
                "★★ And THE STATE IS TERMINAL: {RECOVER_NOTCHES} Ctrl+wheel notches out drew no \
                 page at all. `canvas::present` returns above every input handler when nothing \
                 was drawn, and `canvas::escape::offer` is what is supposed to run the two \
                 gestures that need no drawn page."
            ),
        };
        let cause = if climb.saw_deep {
            "The f64 `DeepAnchor`'s page point is off the sheet and nothing confined it to a \
             placement from which the page is visible."
        } else {
            "★ The deep tier was never reached, so this is NOT the anchor: read \
             `canvas::geometry::pasteboard` and `MIN_SHEET_ON_SCREEN`. A pasteboard of exactly \
             one viewport makes the ends of `visible_origin_range` the zero-overlap placement, so \
             the shallow tier parks the sheet off screen with no anchor involved at all."
        };
        return Ok(Some(format!(
            "★★★ THE PAGE LEFT THE SCREEN. After {notches} Ctrl+wheel notches aimed {:.0} points \
             above the sheet's top edge, the canvas traced `{BLANK_EVENT} reason={BLANK_REASON}` \
             at trace line {blank} — peak zoom {:.0}%, deep tier {}, clamp {}. {cause} {hatch}",
            page.height_pt * OFF_PAGE_FRACTION,
            climb.peak_zoom * 100.0,
            if climb.saw_deep {
                "reached"
            } else {
                "NOT reached"
            },
            if climb.confined.is_empty() {
                "never fired".to_owned()
            } else {
                format!("fired on axes {}", climb.confined.join("+"))
            }
        )));
    }

    // 2. The half that outlives the fix.
    if let Some(line) = &climb.learned_while_blank {
        return Ok(Some(format!(
            "★★★ A RASTER CEILING WAS LEARNED WHILE THE CANVAS WAS BLANK. `{line}`. The zoom \
             ceiling is learned from what the rasterizer refused, and a blank canvas refuses \
             everything — so this ceiling is learned from the defect and then caps the zoom for \
             the rest of the session, outliving any fix to the placement. \
             `render::settle::raster_order_fillable` is the guard that is supposed to make this \
             unreachable."
        )));
    }

    // 3. The operator's own sentence.
    if climb.peak_zoom < ZOOM_FLOOR {
        return Ok(Some(format!(
            "★★ THE CANVAS STOPPED ZOOMING IN. {notches} Ctrl+wheel notches from about 29 % took \
             the zoom no higher than {:.0}% — the floor this check holds is {:.0}%, which is \
             itself well above the A1 sheet's deep threshold of about 44,000 %. This is the \
             operator's \"the canvas will just stop zooming in\". Read the learned raster ceiling \
             first: `{LEARNED_EVENT}` lines say what scale the rasterizer refused and what zoom \
             it pulled the view back to.",
            climb.peak_zoom * 100.0,
            ZOOM_FLOOR * 100.0
        )));
    }

    // --- the two vacuity guards ---------------------------------------------
    //
    // Asked LAST, and SKIP rather than fail, because they are statements about
    // this run rather than about the application. They come after the verdicts
    // so that a binary with the clamp removed reports RED above rather than
    // SKIPPED here — a check that cannot fail is not evidence.
    if !climb.saw_deep {
        return Err(Error::new(format!(
            "no `{POS_EVENT}` line after the climb began said `tier=deep`, so the run never \
             reached the mechanism this check is about: below the threshold the position is an \
             f32 `egui` scroll offset, which the scroll area clamps for itself, and there is no \
             `DeepAnchor` to confine. The threshold is `longest_page_pt * zoom > 1048576`, which \
             on this fixture's 2383.937-point A1 page is a zoom of about 440 — the climb peaked \
             at {:.2}. Raise CLIMB_NOTCHES, or read the `{LEARNED_EVENT}` lines for a ceiling \
             that ratcheted the zoom back below the threshold. SKIPPED rather than passed.",
            climb.peak_zoom
        )));
    }
    if climb.confined.is_empty() {
        return Err(Error::new(format!(
            "the clamp never fired: every `{CONFINED_EVENT}` line after the climb began said \
             `axes=none`, so the anchor stayed inside `geometry::visible_origin_range` throughout \
             and this run proves nothing about what happens when it does not. The page stayed on \
             screen, which is the right outcome — but it is the right outcome of a gesture that \
             never posed the question, and an assertion both outcomes satisfy measures neither. \
             The climb reached a peak zoom of {:.0}% on the deep tier aimed {:.0} points above \
             the sheet; if that no longer drives the anchor out of range, the aim or \
             OFF_PAGE_FRACTION needs re-deriving against `DeepAnchor::zoomed_about`. SKIPPED \
             rather than passed.",
            climb.peak_zoom * 100.0,
            page.height_pt * OFF_PAGE_FRACTION
        )));
    }

    // 4. The recovery probe, which on a fixed build is a sanity check: the
    //    zoom must come back down and a page must still be drawn.
    match recovered {
        None => {
            return Ok(Some(format!(
                "★★ THE VIEW DID NOT COME BACK. The clamp held the page on screen throughout the \
                 climb (axes {}), but {RECOVER_NOTCHES} Ctrl+wheel notches out then drew no page \
                 at all — no `{CANVAS_EVENT}` line since the probe began. Something on the way \
                 DOWN loses the view: read `canvas::deep`'s hand-over back to the f32 offset, \
                 which is `zoom_out_keeps_place`'s subject.",
                climb.confined.join("+")
            )));
        }
        Some(z) if z >= climb.peak_zoom / 10.0 => {
            return Ok(Some(format!(
                "★ THE ZOOM WOULD NOT COME BACK DOWN. {RECOVER_NOTCHES} Ctrl+wheel notches out \
                 moved the zoom from a peak of {:.0}% only as far as {:.0}%, where a notch is a \
                 factor of about 1.2 and eighty of them should be a factor of three million. \
                 Ctrl+wheel out is not reaching `zoom::wheel_step`.",
                climb.peak_zoom * 100.0,
                z * 100.0
            )));
        }
        Some(_) => {}
    }

    report.note(format!(
        "the clamp fired on axes {} and the page was drawn on every frame of an {notches}-notch \
         climb to {:.0}% aimed off the sheet, then came back to {:.0}% on the way out",
        climb.confined.join("+"),
        climb.peak_zoom * 100.0,
        recovered.unwrap_or_default() * 100.0
    ));
    Ok(None)
}
