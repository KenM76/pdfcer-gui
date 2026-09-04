//! `read_mode_refuses_canvas_edits` — the regression test for **a mode gate
//! that is a pure function, tested as a pure function, and whose entire value
//! lies in something no pure function can observe: what a real click does to a
//! real canvas.**
//!
//! # The defect class this exists for
//!
//! The operator asked for it in one sentence, recorded in
//! `app::modes::capability`'s header:
//!
//! > *"in read mode the document shouldn't allow editing and should allow only
//! > selecting of objects that acrobat reader would allow."*
//!
//! `HANDOFF.md` §9 records what the code did before the gate: *"clicking a
//! line in Read selected it, dragging it moved it, and Delete deleted it:
//! three edits in a mode whose entire purpose is that it does not author
//! anything."*
//!
//! The gate that closed it is four links, and every one has a passing unit
//! test:
//!
//! | # | Link | Where | Its own test |
//! |---|---|---|---|
//! | 1 | a mode's capabilities are derived from its **tab list** | `app/modes/capability.rs` | yes |
//! | 2 | the running application asks for them, per frame, from the **ribbon's** mode | `app/gating.rs::capabilities` | yes |
//! | 3 | they reach the canvas and are handed to the gesture machine | `canvas::show` → `canvas::interact`'s `Frame::caps` | — |
//! | 4 | `press_kind` returns `PressMeaning { click: caps.edit_content, … }` and the machine swallows the click | `canvas::gesture::meaning` | yes |
//!
//! Link 3 is the one with no test, and it is the one a refactor breaks
//! silently. `Frame::caps` is a field on a struct built once per frame; a build
//! that sampled it from `self.modes` instead of from the ribbon would be one
//! frame stale on exactly the frame a stray click is most likely (the pointer
//! is already down over the chrome), and a build that defaulted it would get
//! [`Capabilities::FULL`] — because `Default for Capabilities` is `FULL`, on
//! purpose, so that a test which does not mention modes is not silently
//! asserting one. **A `..Default::default()` added to `Frame` would reopen this
//! defect completely and break no test in the workspace.**
//!
//! # ★ Why the Edit half is load-bearing, and not a courtesy
//!
//! The whole check is an assertion about an **absence**: no
//! `canvas-selection via=click` line after a click in Read. `crate::report`'s
//! rule for that is blunt — *never treat an absence as evidence unless you
//! have shown the thing that would have produced it was working* — and the
//! reason is that the most likely way to write this check wrong is to write one
//! that passes against a build where the click simply missed the page. Empty
//! paper produces the same silence as a working gate, and so does a click that
//! landed on the grey surround, on a panel, or on a page that had not finished
//! rastering.
//!
//! So the same document point is clicked twice, in two modes, and the check
//! only reaches a verdict when the second click **does** select something:
//!
//! | Phase | Mode | Click | Expected | If it does not hold |
//! |---|---|---|---|---|
//! | A | Read | at `P` | **no** `canvas-selection via=click` | FAIL — Read edited the document |
//! | B | Edit | at `P` | `canvas-selection via=click` with `sel=` > 0 | this point had no content; try the next `P` |
//! | C | Read | — | `mode-capabilities … cleared_selection=true` | FAIL — the selection survived into Read |
//!
//! Phase B failing is **not** a failure of the application. It means the
//! harness aimed at empty paper, which `crate::coords` documents as
//! symptom-identical to a broken hit test and which has already produced one
//! filed-then-retracted defect in this codebase. So the pair is retried at the
//! next candidate point, and if no candidate has content under it the check
//! reports SKIP naming exactly that — never PASS.
//!
//! # Phase C, and why it is a separate fact
//!
//! Refusing a press and retiring what is already there are two mechanisms, and
//! `app/gating.rs`'s header says why neither is sufficient alone:
//!
//! > Refusal alone leaves an armed pen drawing a crosshair over a page it
//! > cannot draw on, and eight resize handles on a selection nothing will move.
//! > Retirement alone leaves every gesture available for as long as the
//! > operator stays put.
//!
//! Phases A and B test the refusal. Phase C tests the retirement, and the
//! defect it closes is not "Delete works in Read" — it is the *outline and
//! eight resize handles left on the page*, which are visible controls the
//! operator can aim at and which would do nothing. That is precisely the
//! *"visible control, silently inert"* failure `MODES_AND_PANELS.md` Part 1
//! forbids, and it is why `on_mode_capabilities_changed` clears the selection
//! on the way in: so that `press_kind` never has to refuse a grip the operator
//! is looking at.
//!
//! # ★ The one thing about the trace that shapes this whole file
//!
//! `canvas-selection` is emitted through `crate::diag::trace_changed`, which
//! **suppresses a line identical to the last one written to the same slot**.
//! That is right for the application — a marquee dragged across a sheet would
//! otherwise bury the events around it — and it is a trap for a check that
//! clicks the same object twice and expects two lines.
//!
//! It is why the phases are interleaved per candidate rather than run as
//! "calibrate first, then test". In this order the *first* successful
//! selection of the run is the only `sel=` > 0 line that has to appear, so
//! there is never a previous identical line to suppress it. Misses are
//! immune by construction: a miss clears the selection and prints `sel=0`, and
//! a second miss printing nothing at all leads to the same conclusion the line
//! would have.
//!
//! Written down because a future reader restructuring these phases into the
//! obvious "find a content point, then run the three phases" shape would get a
//! check that FAILs at phase B against a working build, and the cause would be
//! four modules away.
//!
//! # Mouse only
//!
//! Nothing here needs a key.
//!
//! ★ **CORRECTED 2026-08-18.** These headers used to say synthetic keyboard
//! input does not reach the target window on this machine. It DOES — see
//! [`crate::checks::add_text`], which types real characters into a caret
//! draft and asserts they landed. The belief came from `Ctrl+E` producing no
//! trace, which was the dead-keymap defect (fourteen of twenty-one declared
//! chords were dispatched by nothing) misread as a property of the machine —
//! and while it stood nobody drove a chord, so nothing could contradict it.
//! The Delete
//! key is *also* gated by `Capabilities::edit_content`
//! (`canvas::keys::canvas_keys` takes `caps`), and asserting that from outside
//! the process is blocked on the same environment limit; it is covered by unit
//! test alone and named here so the gap is on the record rather than implied.
//!
//! # Every way this reports SKIP, and why none of them is a pass
//!
//! * no binary, no `--pdf`, `--no-input` — the harness never began;
//! * the diagnostic switches did not reach the process;
//! * the page size could not be read from the fixture and no `--page-size` was
//!   given — without the page height there is no y-flip;
//! * a mode segment was never declared, or was declared at no usable size, or
//!   took no click — [`crate::checks::driving::click_mode_segment`] carries
//!   the reasons and the argument for each;
//! * the canvas is not showing page 1, so the harness's one known page size
//!   does not describe the page it would be clicking on;
//! * **no candidate point had content under it** — phase B never succeeded, so
//!   phase A's silence proves nothing and is not reported as though it did.

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, ScreenPoint};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The mode under test: the one whose tab list is `["file", "view"]`, so
/// `Capabilities::for_mode` grants it neither `edit`, nor `markup`, nor
/// `measure`.
const READ: &str = "read";

/// The control mode: the one whose tab list contains `edit`, so the identical
/// click at the identical document point is expected to select.
///
/// Edit rather than Review, deliberately. Review would *also* refuse the click
/// — `edit_content` is false there too — so a check that compared Read against
/// Review would be comparing two refusals and would pass against a build where
/// the click never reached the canvas at all. Edit is the only shipped mode
/// that answers "would this click have selected something?".
const EDIT: &str = "edit";

/// `canvas-selection via=… mod=… sel=… level=…` — `canvas::trace`'s report of
/// what the selection layer just did.
///
/// Emitted from exactly two places in `canvas::interact`: the `Click` arm's
/// non-measure branch, and the completed select-marquee arm. Both are inside
/// gesture outcomes that `press_kind` refuses in a mode without
/// `edit_content`, so in Read there is nothing that can produce this line.
const SELECTION_EVENT: &str = "canvas-selection";

/// The `via=` value that means a completed click with no drag.
///
/// Matched rather than taking any `canvas-selection`: a marquee is a different
/// gesture with a different gate (`MarqueeIntent::Select` needs
/// `edit_content`, `MarqueeIntent::Zoom` needs nothing at all and is offered in
/// every mode including Read), and a check that conflated them could be
/// satisfied by a navigation gesture.
const VIA_CLICK: &str = "click";

/// `mode-capabilities content=… markup=… measure=… retired_tool=… \
/// cleared_selection=… abandoned_drag=…` — `app/gating.rs`'s report of what a
/// mode change **put down on the way in**.
///
/// Emitted only when something was actually retired, cleared or abandoned, so
/// its presence is itself news: entering Read with nothing selected and no tool
/// armed writes no line at all. That is what makes counting them a usable
/// oracle for phase C.
const CAPABILITIES_EVENT: &str = "mode-capabilities";

/// The field on it that says a selection was dropped on the way into the mode.
const CLEARED_FIELD: &str = "cleared_selection";

/// **Where to look for content, as fractions of the page box.**
///
/// # Why a ladder rather than a `--doc-point`
///
/// [`crate::checks::delete_key`] takes its target from `--doc-point` and SKIPs
/// without one, and that is right for a check whose *subject* is the point: it
/// needs an object, and a wrong point is indistinguishable from a broken hit
/// test. This check is different in one respect that changes the answer: it
/// does not merely hope the point has content, it **proves** it, in phase B,
/// with the application's own `sel=`. A candidate that proves nothing is
/// discarded and the next is tried; a ladder that proves nothing at all is a
/// SKIP.
///
/// So the ladder is a search whose every step is confirmed by the program under
/// test, which is the opposite of the guess `crate::coords` warns about. It
/// also keeps the check runnable with no arguments beyond `--pdf`, which
/// matters because a check nobody can run without knowing a magic coordinate is
/// a check that stops being run.
///
/// `--doc-point`, when given, is tried **first**: an operator who knows where
/// their fixture keeps an object should not have to wait for the search.
///
/// # Why these fractions, in this order
///
/// Ordered cheapest-first for the drawing fixtures this project actually uses
/// (`HANDOFF.md` §2's table). A SolidWorks sheet is a border frame, a title
/// block in the bottom-right, and drawing views across the middle, so the
/// ladder walks the middle band first and then the title block, rather than
/// starting at a page centre that on a two-view drawing is often paper.
///
/// Every entry is well inside the page box, so all of them land on paper rather
/// than on the grey surround whatever size the fixture is.
const CANDIDATES: [(f64, f64); 12] = [
    (0.50, 0.50),
    (0.30, 0.60),
    (0.70, 0.60),
    (0.30, 0.35),
    (0.70, 0.35),
    (0.50, 0.75),
    (0.85, 0.12),
    (0.70, 0.12),
    (0.15, 0.50),
    (0.88, 0.50),
    (0.50, 0.90),
    (0.50, 0.10),
];

/// See the module documentation.
pub struct ReadModeRefusesCanvasEdits;

impl Check for ReadModeRefusesCanvasEdits {
    fn name(&self) -> &'static str {
        "read_mode_refuses_canvas_edits"
    }

    fn defect(&self) -> &'static str {
        "a click on page content in Read mode still selects it, or a selection made in Edit \
         survives into Read — the mode gate's capabilities not reaching the canvas, which is \
         the one link in that chain no unit test observes"
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

/// How many `canvas-selection via=click` lines the trace carries so far.
///
/// Counted rather than "is the last line a click?", for the reason
/// [`crate::checks::driving::click_mode_segment`] counts its mode events: a run
/// makes several of these, and a check asking "did one ever appear?" would be
/// satisfied by one it provoked itself a phase ago.
fn selection_clicks(trace: &Trace) -> Vec<&crate::trace::TraceLine> {
    trace
        .events(SELECTION_EVENT)
        .filter(|l| l.get("via") == Some(VIA_CLICK))
        .collect()
}

/// Aim a document point at the canvas **as it is laid out right now**.
///
/// Re-derived on every use rather than cached, and that is not caution — it is
/// required. Read defaults to a continuous strip and Edit to a single page
/// (`viewer::display::default_for_mode`), so switching mode moves and rescales
/// the page: the same `DocPoint` is a different screen pixel in the two modes,
/// which is exactly why this crate writes document coordinates and never screen
/// ones.
///
/// # Errors
///
/// * the application has traced no canvas layout yet;
/// * it is showing a page other than the first, whose size this harness does
///   not know;
/// * the point is not currently on screen — refused rather than clamped, since
///   a clamped click lands on the canvas edge and hit-tests nothing, which
///   reads as a broken feature.
fn aim(
    ctx: &CheckContext,
    session: &Session,
    page: PageGeometry,
    at: DocPoint,
) -> Result<ScreenPoint> {
    let trace = session.trace()?;
    let shown = trace
        .last(ctx.profile.vocab.canvas_event)
        .and_then(|l| l.get_usize("page"));
    if shown != Some(at.page) {
        return Err(Error::new(format!(
            "the canvas is showing page {}, and this check's point is on page {}. Converting a \
             document point against another page's rect would put it somewhere plausible and \
             wrong — the confidently-wrong click `crate::coords` exists to refuse.",
            shown.map_or_else(|| "an unreported index".to_owned(), |p| (p + 1).to_string()),
            at.page + 1
        )));
    }
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, at.page)?;
    Ok(session.frame()?.to_screen(mapping.doc_to_window(at)?))
}

/// Run the sequence.
///
/// The three-way return is [`crate::report`]'s rule made structural: `Err` is
/// a precondition that was absent (SKIP), `Ok(Some(_))` is an assertion that
/// did not hold (FAIL), `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    // --- preconditions -----------------------------------------------------
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check clicks page content and asserts on what the selection did \
             about it, so it needs a document with at least one selectable object on its first \
             page.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is a sequence of real clicks on real \
             mode segments and a real canvas, and every one of them needs the pointer and the \
             foreground. Reported as SKIPPED rather than passed: a check that did not run has \
             learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its mode segments are and this check has nothing to aim at.",
            ctx.profile.name
        ))
    })?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. The harness needs the page box to turn this \
                 check's document-space fractions into points, and the page height to flip PDF \
                 y (up) into window y (down). Pass --page-size WxH. It refuses to guess: a \
                 wrong page height mirrors every click about the page centre, which lands on \
                 the page and hit-tests something plausible.",
                pdf.display()
            ))
        })?,
    };
    report.note(format!(
        "fixture {} — page 1 is {:.0}x{:.0} pt",
        pdf.display(),
        page.width_pt,
        page.height_pt
    ));

    // --- launch, with BOTH diagnostic channels armed -----------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("read_mode.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {} with {}={} and {}={}",
        exe.display(),
        session.pid(),
        ctx.profile.diag_env.0,
        ctx.profile.diag_env.1,
        SHELL_DIAG_ENV.0,
        SHELL_DIAG_ENV.1
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the \
             process and this check has no oracle. Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }
    for reject in trace.rejected_steps() {
        report.note(format!(
            "the application REJECTED a script step: {}",
            reject.raw
        ));
    }
    let frame = session.frame()?;
    report.note(format!(
        "window client area {}x{} px at desktop ({}, {}), DPI scale {:.2}",
        frame.client_size.0,
        frame.client_size.1,
        frame.client_origin.0,
        frame.client_origin.1,
        frame.scale
    ));
    let driver = Driver::new(session.window());

    // The candidate list: the operator's point first if they gave one, then
    // the ladder. See `CANDIDATES` for why this is a search rather than a
    // guess.
    let mut candidates: Vec<DocPoint> = Vec::with_capacity(CANDIDATES.len() + 1);
    if let Some(target) = ctx.target {
        report.note(format!(
            "--doc-point (page {}, {:.0}, {:.0}) will be tried first",
            target.page + 1,
            target.x,
            target.y
        ));
        candidates.push(target);
    }
    candidates.extend(
        CANDIDATES
            .iter()
            .map(|(fx, fy)| DocPoint::new(0, fx * page.width_pt, fy * page.height_pt)),
    );

    // --- the interleaved phases A and B, per candidate ---------------------
    //
    // Interleaved rather than "calibrate then test" because of the trace's
    // de-duplication — see the module header's last section. The order also
    // pairs the evidence correctly: phase A's silence is asserted at the SAME
    // point phase B then proves has content under it.
    let mut unreachable: Vec<String> = Vec::new();
    let mut found: Option<(DocPoint, String)> = None;

    for (n, at) in candidates.iter().enumerate() {
        // ---- phase A: the click Read must refuse -------------------------
        driving::click_mode_segment(&session, &driver, ui_rect, READ)?;
        let read_point = match aim(ctx, &session, page, *at) {
            Ok(p) => p,
            Err(e) => {
                // Not fatal to the check: this candidate cannot be reached in
                // this mode's layout, which says nothing about the gate. It is
                // recorded so a run where NO candidate was reachable can say
                // so rather than reporting "no content".
                unreachable.push(format!(
                    "({:.0}, {:.0}) in {READ}: {}",
                    at.x,
                    at.y,
                    e.message()
                ));
                continue;
            }
        };
        let before = selection_clicks(&session.trace()?).len();
        driver.click_at(read_point)?;
        session.settle(12);
        let after_read = session.trace()?;
        let read_clicks = selection_clicks(&after_read);
        if read_clicks.len() > before {
            let line = read_clicks.last().map_or("", |l| l.raw.as_str());
            return Ok(Some(format!(
                "READ MODE EDITED THE DOCUMENT. A single click at document point ({:.0}, {:.0}) \
                 on page {} — with the ribbon's mode selector on `{READ}`, whose tab list is \
                 file and view alone — produced `{line}`. In a mode without `edit_content` \
                 there is nothing that can write that line: `canvas::gesture::press_kind` \
                 returns `click: caps.edit_content`, `GestureState::update` swallows the click, \
                 and `canvas::interact`'s `Click` arm is never entered. So the capabilities the \
                 canvas was handed this frame were not Read's. Look at link 3 — `canvas::show` \
                 sampling `PdfcerApp::capabilities()` into `canvas::interact::Frame::caps` — \
                 which is the one link in this chain with no unit test, and note that \
                 `Default for Capabilities` is `FULL`, so a `..Default::default()` anywhere on \
                 that path reopens this defect and breaks nothing.",
                at.x,
                at.y,
                at.page + 1
            )));
        }
        report.note(format!(
            "candidate {}: the click at ({:.0}, {:.0}) traced no `{SELECTION_EVENT} \
             via={VIA_CLICK}` line in {READ}",
            n + 1,
            at.x,
            at.y
        ));

        // ---- phase B: the same click Edit must honour --------------------
        driving::click_mode_segment(&session, &driver, ui_rect, EDIT)?;
        let edit_point = match aim(ctx, &session, page, *at) {
            Ok(p) => p,
            Err(e) => {
                unreachable.push(format!(
                    "({:.0}, {:.0}) in {EDIT}: {}",
                    at.x,
                    at.y,
                    e.message()
                ));
                continue;
            }
        };
        let before = selection_clicks(&session.trace()?).len();
        driver.click_at(edit_point)?;
        session.settle(12);
        let after_edit = session.trace()?;
        let edit_clicks = selection_clicks(&after_edit);
        let selected = edit_clicks
            .get(before)
            .and_then(|l| l.get_usize("sel"))
            .unwrap_or(0);
        if selected > 0 {
            let line = edit_clicks
                .get(before)
                .map_or_else(String::new, |l| l.raw.clone());
            report.note(format!(
                "candidate {}: the SAME click in {EDIT} traced `{line}` — so the point is on \
                 content, the pointer reached the canvas, and {READ}'s silence a moment ago was \
                 a refusal rather than a miss",
                n + 1
            ));
            found = Some((*at, line));
            break;
        }
        report.note(format!(
            "candidate {}: nothing under ({:.0}, {:.0}) — {EDIT} selected {selected}, so this \
             point cannot prove anything about {READ}; trying the next",
            n + 1,
            at.x,
            at.y
        ));
    }

    let Some((point, edit_line)) = found else {
        // ★ The honest SKIP. Phase A held at every candidate, and that is
        // deliberately NOT reported as a pass: with no phase B to establish
        // that a click at those points would have selected anything, Read's
        // silence is equally consistent with a working gate and with a harness
        // that never got a click onto the page.
        return Err(Error::new(format!(
            "no candidate point had content under it: {} point(s) were clicked in `{EDIT}` and \
             none of them selected anything. Read refused every one of them, and this check \
             declines to call that a pass — an absence is only evidence once the thing that \
             would have produced it is shown to be working, and nothing here has shown that a \
             click at these points reaches page content at all. Pass --doc-point PAGE,X,Y \
             naming a point in PDF user space (origin bottom-left) where this fixture has an \
             object, and it will be tried first. {}Trace: {}.",
            candidates.len(),
            if unreachable.is_empty() {
                String::new()
            } else {
                format!(
                    "Points that could not be aimed at all, which is a different problem and \
                     may be the whole of this one: {}. ",
                    driving::list(&unreachable)
                )
            },
            session.trace_path().display()
        )));
    };

    // --- phase C: entering Read drops the selection ------------------------
    //
    // The application is in Edit with a live selection, which is the only
    // state in which this assertion means anything: `on_mode_capabilities_
    // changed` traces nothing at all when there was nothing to clear, so a
    // check that ran this phase on an empty selection would be asserting on
    // the absence of a line that is correctly absent.
    let before = session.trace()?.events(CAPABILITIES_EVENT).count();
    driving::click_mode_segment(&session, &driver, ui_rect, READ)?;
    let trace = session.trace()?;
    let lines: Vec<&crate::trace::TraceLine> = trace.events(CAPABILITIES_EVENT).collect();
    let Some(entry) = lines.get(before) else {
        return Ok(Some(format!(
            "THE SELECTION SURVIVED INTO READ. `{edit_line}` was traced in {EDIT}, the mode \
             selector was then clicked back to `{READ}`, and `app/gating.rs` traced no new \
             `{CAPABILITIES_EVENT}` line — which it writes whenever a mode change retired a \
             tool, cleared a selection or abandoned a drag. So nothing was put down on the way \
             in, and the page in Read is still carrying an outline and eight resize grips: \
             visible controls the operator can aim at that will do nothing, which is the \
             *visible control, silently inert* failure `MODES_AND_PANELS.md` Part 1 forbids by \
             name. Look at `PdfcerApp::on_mode_capabilities_changed` and at whether the \
             mode-change arm in `dock_area` still calls it. {} lines were traced before the \
             switch and {} after.",
            before,
            lines.len()
        )));
    };
    if entry.get(CLEARED_FIELD) != Some("true") {
        return Ok(Some(format!(
            "THE SELECTION SURVIVED INTO READ. Entering `{READ}` did trace \
             `{CAPABILITIES_EVENT}`, so the mode change was seen and \
             `on_mode_capabilities_changed` ran — and it reports `{CLEARED_FIELD}={}` after \
             `{edit_line}`. The line is: `{}`. That combination means the clear was attempted \
             and returned false, so either `caps.edit_content` was still true for {READ} (look \
             at the mode's tab list in the manifest, which is what \
             `app::modes::capability` derives from — and note the fallback for an \
             *unrecognised* mode is `Capabilities::FULL`, on purpose) or the selection this \
             check made in {EDIT} was no longer on the document by the time the mode changed.",
            entry.get(CLEARED_FIELD).unwrap_or("absent"),
            entry.raw
        )));
    }
    report.note(format!(
        "entering {READ} traced `{}` — the selection made in {EDIT} was dropped on the way in",
        entry.raw
    ));
    report.note(format!(
        "verdict established at document point ({:.0}, {:.0}) on page {}: {READ} refused the \
         click that {EDIT} honoured, and {READ} dropped the selection {EDIT} had made",
        point.x,
        point.y,
        point.page + 1
    ));
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two modes are the two the argument needs, and they are not the same
    /// one.
    ///
    /// `EDIT` in particular: comparing Read against Review would compare two
    /// refusals, and would pass against a build where the click never reached
    /// the canvas at all. See [`EDIT`]'s own documentation.
    #[test]
    fn the_control_mode_is_the_one_that_actually_selects() {
        assert_eq!(READ, "read");
        assert_eq!(EDIT, "edit");
        assert_ne!(READ, EDIT);
    }

    /// Every candidate is well inside the page box, so all of them land on
    /// paper rather than on the grey surround whatever size the fixture is.
    #[test]
    fn every_candidate_is_inside_the_page_box() {
        for (fx, fy) in CANDIDATES {
            assert!((0.05..=0.95).contains(&fx), "x fraction {fx}");
            assert!((0.05..=0.95).contains(&fy), "y fraction {fy}");
        }
        assert!(
            CANDIDATES.len() >= 8,
            "a ladder short enough to miss every drawing view would turn a working gate into a \
             SKIP more often than it would find content"
        );
    }

    /// **A marquee is not a click**, and the two are gated differently.
    ///
    /// `MarqueeIntent::Zoom` needs no capability at all and is offered in Read
    /// — `press_kind` says so in as many words — so a check that matched any
    /// `canvas-selection` would be one navigation gesture away from a false
    /// FAIL against a correct build.
    #[test]
    fn only_a_click_counts_as_a_click() {
        let trace = Trace::parse(
            "pdfcer-diag canvas-selection via=marquee mod=false sel=7 level=Object\n\
             pdfcer-diag canvas-selection via=click mod=false sel=1 level=Object",
            "pdfcer-diag",
        );
        let clicks = selection_clicks(&trace);
        assert_eq!(clicks.len(), 1, "the marquee line must not be counted");
        assert_eq!(clicks[0].get_usize("sel"), Some(1));
    }

    /// A selection line reporting `sel=0` is a click that found nothing, which
    /// is a miss and not a hit — the distinction phase B turns on.
    #[test]
    fn a_click_that_selected_nothing_is_not_a_hit() {
        let trace = Trace::parse(
            "pdfcer-diag canvas-selection via=click mod=false sel=0 level=Object",
            "pdfcer-diag",
        );
        assert_eq!(selection_clicks(&trace)[0].get_usize("sel"), Some(0));
    }

    /// The capability line is read by field name, and `cleared_selection` is
    /// not the only boolean on it.
    ///
    /// `retired_tool` and `abandoned_drag` sit beside it and are `false` in the
    /// case this check drives, so a reader matching the *line* rather than the
    /// field would accept a mode change that put down a pen and kept the
    /// selection.
    #[test]
    fn the_cleared_flag_is_read_by_name_and_not_by_the_line() {
        let trace = Trace::parse(
            "pdfcer-diag mode-capabilities content=false markup=false measure=false \
             retired_tool=true cleared_selection=false abandoned_drag=false",
            "pdfcer-diag",
        );
        let line = trace.last(CAPABILITIES_EVENT).expect("the line");
        assert_eq!(line.get(CLEARED_FIELD), Some("false"));
        assert_eq!(line.get("retired_tool"), Some("true"));
    }
}
