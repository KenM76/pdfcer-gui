//! `move_line_of_text` — **a drag on one line inside a block of text is
//! refused, and the operator is TOLD.**
//!
//! The driven half of `OPERATOR_REQUESTS.md` **O188** change (B). It is the
//! only thing that can say the feature works, because the defect O188 names is
//! a *silence*, and a silence is exactly what a green unit-test suite looks
//! like.
//!
//! # The report, and why it was not a feature request
//!
//! **Ken, 2026-09-15:** he selected one line of a title block on a SolidWorks
//! export, dragged it, and nothing happened. Not an error, not a refusal, not
//! a greyed control — the outline did not move and the status bar said
//! nothing. Every part of that was *correct* except the last one:
//!
//! * `pdfcer-core` has no verb that moves one show operator out of a text
//!   object. It has `move_subpath`, `move_node`, `move_objects`; there is no
//!   `move_text_run`. The engine request for one is O188 change (C).
//! * `canvas::moving::eligible` therefore refuses at the Part rung with
//!   `Refusal::NoVerbForPart(PartKind::Run)`, which is right — declining
//!   before the ghost slides is what keeps the preview truthful.
//! * …and `canvas::moving::decline` wrote a trace line and **nothing else**.
//!   One refusal out of eleven raised an operator-facing sentence
//!   (`InsideForm`), and the comment above the mechanism said *"one refusal out
//!   of the eight"* — a count that had been wrong long enough that nobody
//!   re-read the sentence it introduced, which is a large part of why this one
//!   was never weighed.
//!
//! ⇒ **The defect is the third bullet only.** A build that moves the line is
//! not what this check wants; a build that refuses *audibly* is.
//!
//! # ★★★ WHY THIS CHECK EXISTS WHEN FOUR UNIT TESTS ALREADY COVER IT
//!
//! `canvas::moving::tests` asserts, at the seam, that `decline` pushes
//! `Action::DeclineOnCanvas(CanvasDecline::TextRunCannotMoveAlone)`. Those
//! tests are right and they are not evidence, for this project's founding
//! reason (**R1**): they call the function. They cannot see the chain in front
//! of it — whether a real drag at a real rung reaches `decline` at all,
//! whether the apply phase has an arm for the action, whether the status bar
//! draws what the store holds, or whether a mode gate swallows the gesture two
//! layers earlier. Eight green tests once sat in front of a feature that did
//! one step of fourteen.
//!
//! So this drives the OS: a real click into Edit mode, a real chord to arm the
//! Points tool, a real click to select one line, a real press-move-release,
//! and then it reads what three independent subsystems wrote down.
//!
//! # The oracle — three lines, three subsystems, in order
//!
//! ```text
//! canvas       canvas-move-declined level=Part sel=1 reason=no-verb-for-text-run …
//! apply phase  canvas-decline-recorded what=text-run-cannot-move-alone
//! status bar   ui-rect name=status-group:decline rect=…
//! ```
//!
//! | line | question it answers | who writes it |
//! |---|---|---|
//! | `canvas-move-declined` | did the gesture reach the move rules, and refuse for the reason this check is about? | `crate::canvas::moving`, holding `&OpenDoc` |
//! | `canvas-decline-recorded` | did the refusal's **sentence** cross the `Action` boundary and reach the store? | `crate::app::status::decline::canvas`, holding `&mut` |
//! | `status-group:decline` | was it **drawn**, on a frame, where he could read it? | `crate::app::status::disclosure` |
//!
//! ★★ **That is a chain measurement, not the application agreeing with
//! itself.** The three writers are three subsystems separated by the exact
//! boundary O188's design is about — a canvas gesture holds no `&mut` and can
//! only *ask*. A shell that raised the action and had no apply arm for it
//! writes the first line and not the second. A shell that recorded the
//! sentence into a store the bar never reads writes the first two and not the
//! third.
//!
//! ## ★★★ Why the region alone would have been worthless
//!
//! `status-group:decline` is **one region shared by every decline in the
//! application** — a save that failed, a bookmark that would not move, a zoom
//! with nothing to frame. A check asserting only *"that region is on screen
//! after the drag"* is satisfied just as well by a build that raised the wrong
//! sentence, and by a stale sentence left on the bar by an earlier gesture.
//!
//! > An assertion both outcomes satisfy is not a measurement of which one
//! > shipped. Name what the WRONG mechanism cannot produce.
//!
//! `canvas-decline-recorded what=…` is that name. It carries a **stable
//! token**, not a `Debug` rendering, for the reason this project has recorded
//! twice: a `{:?}` field is a property of how a variant is *spelled*, so a
//! rename turns a driven check into a confident false negative that quotes the
//! truth in its own failure message.
//!
//! # ⚠ HOW TO FALSIFY THIS CHECK — do this before believing a PASS
//!
//! A check that has never been seen to fail is not evidence.
//!
//! 1. **Copy the file aside first.**
//!    `cp crates/pdfcer-gui/src/canvas/moving/mod.rs $SCRATCH/moving.rs.bak`.
//!    **Never `git checkout` to undo it** — this project runs parallel tracks
//!    and that discards another track's uncommitted work.
//! 2. **Plant the pre-O188 behaviour**: in `Refusal::worded`, change
//!    `Self::NoVerbForPart(PartKind::Run) => Some(CanvasDecline::TextRunCannotMoveAlone)`
//!    to `=> None`. That is exactly the build Ken reported: the refusal still
//!    happens, still traces, and says nothing.
//! 3. **Prove the plant is in the artifact.** `cargo build --release -p
//!    pdfcer-gui`, then
//!    `grep -c text-run-cannot-move-alone target/release/pdfcer-gui.exe` — the
//!    planted build still contains the string (the token function is
//!    unchanged), so this step instead needs the check's own output: a stale
//!    binary is the commonest cause of a falsification that "did not
//!    reproduce", so `touch` the source if the build is skipped.
//! 4. **Require the `[FAIL]` line**, not the exit code — a SKIP exits the way
//!    a PASS does. The message must name *the refusal happened and raised no
//!    sentence*, and must quote the `canvas-move-declined` line it saw.
//! 5. **Restore from the byte copy**, rebuild, confirm the PASS returns.
//!
//! ★ A second, cheaper plant exercises the third link on its own: comment out
//! the `Declined::TextRunCannotMoveAlone` arm's sentence lookup in
//! `app::status::decline::line` and the check should fail on the *region*
//! rather than on the record.
//!
//! # Fixture — pinned here, not passed on the command line
//!
//! `fixtures/paragraph.pdf` at page 0, `(120, 704)` in PDF user space.
//!
//! The same pin `deeper_rung_delete`'s label rung uses, and for the same
//! measured reasons: one `BT`…`ET` block holding **six** `Tj` operators at
//! 12 pt on a 612 × 792 page, so a Part-rung pick lands on a show operator
//! that is one of several, and the text is legible at fit zoom. The aim is
//! inside the first line, whose baseline is 700 and whose cap height at that
//! size reaches about 708.
//!
//! ★★★ **Pinned in code and not read from `--pdf`**, because the 2026-09-12
//! sweep hands every chunked check one shared A1 sheet and one shared aim.
//! `deeper_rung_delete` carried a correct fixture table in *prose* for a week
//! and all three of its rungs still ran against a document their own header
//! said they could not use. Knowledge a check cannot run without belongs in
//! the check.
//!
//! ⚠ A missing fixture is a **FAIL**, not a SKIP: it is committed to this
//! repository, so its absence is a broken checkout rather than an unavailable
//! precondition, and a SKIP would say the opposite.
//!
//! # ★★ The Points tool, and why there is no double-click here
//!
//! The Part rung on a **text** object is not reachable by double-click, by
//! design: `canvas::clicking`'s O70 arm opens the caret instead, which is the
//! operator's own ruling (*"double-clicking inside the bounding box should
//! edit the text"*). The route that exists is the **Points** tool — chord
//! `A`, labelled *Points* because a draughtsman says point — whose branch
//! takes the click before every other claimant and calls
//! `SelectionState::click_direct`, landing on the Part rung whenever the probe
//! found a part. On a text object a "part" **is** a show operator.
//!
//! `deeper_rung_delete::Rung::arms_the_points_tool` carries the measurement
//! that established this, including the trace lines it was read out of.
//!
//! ★ The chord is pressed **before** the click, not after: with the arrow
//! armed, the first click would select the whole text object and the Points
//! tool's own branch would then be entering an object it did not pick.
//!
//! ★★ It also does the check a second favour, and the check depends on it.
//! Arming the tool is a **command**, and `app::status::decline::retire` runs
//! at the top of `dispatch_command` — so any decline left on the bar by an
//! earlier gesture is cleared before this one starts. That is what makes the
//! "nothing on the bar before the drag" control below assertable rather than
//! hopeful.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode whose canvas may select and edit page content.
///
/// The shell's default is Read, where a canvas click on content is refused BY
/// DESIGN. A check that skipped this step would report the mode gate as a
/// selection defect — which `delete_key`'s own header records as having
/// happened.
const MODE: &str = "edit";

/// `canvas-selection via=… sel=… level=… first=…`.
const SELECTION_EVENT: &str = "canvas-selection";

/// The rung this check must be standing on before it drags.
const PART_LEVEL: &str = "Part";

/// `canvas-move-declined level=… sel=… reason=… detail=…` — written by
/// `canvas::moving::decline`, **on release only**.
///
/// ★ Not per frame. An in-flight drag is re-evaluated 60 times a second and a
/// refusal traced per frame would bury every other event on the channel — the
/// lesson `canvas-pointer` taught when a stationary pointer emitted fifty
/// identical lines in nine seconds.
const MOVE_DECLINED_EVENT: &str = "canvas-move-declined";

/// The stable token `canvas::moving::Refusal::token` writes for
/// `NoVerbForPart(PartKind::Run)`.
///
/// ★★ **Twelve tokens for eleven variants**, because `NoVerbForPart` splits by
/// part kind: its `Run` instance speaks to the operator and its `Subpath`
/// instance is unreachable, so the two have opposite operator-facing outcomes
/// and cannot share a name. The unit of a trace token is *a distinguishable
/// cause*, not an enum variant.
const REASON_RUN: &str = "no-verb-for-text-run";

/// `canvas-decline-recorded what=…` — the apply phase's line, written once per
/// decline by `app::status::decline::canvas::record_canvas`.
const RECORDED_EVENT: &str = "canvas-decline-recorded";

/// The stable token `CanvasDecline::token` writes for the sentence O188 added.
const RECORDED_TEXT_RUN: &str = "text-run-cannot-move-alone";

/// The `⊗` slot in the status bar. `app::status::decline::show` draws into it
/// and publishes it as a `ui-rect` on the frame it draws.
///
/// ⚠ Read through [`driving::declared`], never with `Trace::last`. The
/// application's `ui-rect` channel is a **change log** — it emits when a rect
/// moves and emits a `ui-rect-gone` when a region stops being drawn — so
/// `.last()` returns a fossil for a region that has been retired, and a caller
/// cannot tell it from a live one. That misreading once produced a confident,
/// detailed, entirely wrong layout-defect report about eighteen ribbon
/// controls.
const DECLINE_REGION: &str = "status-group:decline";

/// The fixture, and where on it to aim. See the module header for the
/// measurements behind both.
const FIXTURE: &str = "paragraph.pdf";
/// Page index of [`FIXTURE`] this check uses.
const PAGE: usize = 0;
/// Aim, in PDF user space (y up), inside the first line of the paragraph.
const AIM: (f64, f64) = (120.0, 704.0);

/// How far the drag travels, in window logical points, on each axis.
///
/// ★ Comfortably past any drag threshold and past
/// `canvas::moving::Refusal::NoTravel`'s floor, and small enough that the
/// pointer stays well inside the canvas on a 612 × 792 page at fit zoom. The
/// destination does not matter: the release is refused before any geometry is
/// computed, so this only has to be *a drag* rather than *a click*.
const DRAG_PX: f32 = 60.0;

/// See the module documentation.
pub struct ARefusedDragOnOneLineOfTextSaysSo;

impl Check for ARefusedDragOnOneLineOfTextSaysSo {
    fn name(&self) -> &'static str {
        "a_refused_drag_on_one_line_of_text_says_so"
    }

    fn defect(&self) -> &'static str {
        "Dragging one line inside a block of text does nothing and says nothing — the move \
         rules correctly refuse (`pdfcer-core` has no verb that moves one show operator) and \
         the refusal goes to the trace alone, so the operator is left with a gesture that \
         silently did not happen"
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

/// Run the sequence.
///
/// The three-way return is the SKIP/FAIL/PASS rule made structural: `Err` is a
/// precondition that was absent (SKIP), `Ok(Some(_))` is an assertion that did
/// not hold (FAIL), `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;

    // PINNED: `--pdf` and `--doc-point` are read and IGNORED here. See the
    // module header for what a prose fixture table cost on 2026-09-12.
    let pdf = crate::fixture::workspace_root()
        .join("fixtures")
        .join(FIXTURE);
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the fixture is not at {}. It is committed to this repository, so this is a \
             broken checkout rather than an unavailable precondition — reported as a failure \
             for that reason, because a SKIP would say the opposite.",
            pdf.display()
        )));
    }
    report.note(format!(
        "--pdf and --doc-point are IGNORED: pinned to {} at page {PAGE}, {:.0}, {:.0}",
        pdf.display(),
        AIM.0,
        AIM.1
    ));

    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check arms a tool with a real chord, \
             selects with a real click and drags with a real press-move-release. Reported as \
             SKIPPED rather than passed: a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so this check can neither \
             leave Read mode nor read the status bar's decline slot.",
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
                "cannot read a page size from {}. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    // --- launch -------------------------------------------------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("move_line_of_text.trace.txt"));
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
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- 1: Edit ------------------------------------------------------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    // --- 2: arm the Points tool, then select one line -----------------------
    //
    // ★ Before the click, not after — see the module header. And the chord is
    // a command, so it also retires any decline the mode switch left on the
    // bar, which is what the control in step 3 depends on.
    driver.press(vk::A)?;
    session.settle(12);

    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, PAGE)?;
    let window_point = mapping.doc_to_window(DocPoint::new(PAGE, AIM.0, AIM.1))?;
    let frame = session.frame()?;
    let at = frame.to_screen(window_point);
    driver.click_at(at)?;
    session.settle(14);

    let trace = session.trace()?;
    let selected = trace
        .last(SELECTION_EVENT)
        .and_then(|l| l.get_usize("sel"))
        .unwrap_or(0);
    if selected == 0 {
        return Err(Error::new(format!(
            "the click at document point ({:.1}, {:.1}) on page {PAGE} selected nothing, so \
             there is no line to drag. That is either an aim that is not on a glyph or a \
             broken hit test, and this harness cannot tell them apart — so it declines to \
             file either. Trace: {}.",
            AIM.0,
            AIM.1,
            session.trace_path().display()
        )));
    }
    let level = trace
        .last(SELECTION_EVENT)
        .and_then(|l| l.get("level").map(str::to_owned))
        .unwrap_or_else(|| "none".to_owned());
    if level != PART_LEVEL {
        return Err(Error::new(format!(
            "the selection ladder is at `{level}` and this check needs `{PART_LEVEL}`, so no \
             single LINE of text is selected and the drag below would be testing the Object \
             rung — which moves the whole text block and is not refused at all. The Points \
             tool should land directly on the Part rung; on a text object a part is a show \
             operator. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "the selection ladder is at the {PART_LEVEL} rung ({selected} selected) — one line \
         inside a block of text"
    ));

    // --- 3: the control — the decline slot must be EMPTY before the drag ----
    //
    // ★★★ Without this the whole verdict is vacuous. `status-group:decline` is
    // one region shared by every decline in the application, so a sentence
    // left on the bar by an earlier gesture would satisfy step 5 no matter
    // what the drag did — the "absence assertion whose baseline was taken too
    // late" failure, in its presence-shaped form.
    //
    // It is asserted rather than assumed because it is a real property of the
    // program and not of this sequence: arming the Points tool is a command,
    // and `decline::retire` runs at the top of `dispatch_command`. If a
    // decline IS on the bar here, the program has a retirement defect and this
    // check cannot discriminate — so it SKIPs and says which, instead of
    // reporting a pass it did not earn.
    if driving::declared(&trace, ui_rect, DECLINE_REGION).is_some() {
        return Err(Error::new(format!(
            "the status bar's `{DECLINE_REGION}` slot is ALREADY on screen before the drag, \
             so its presence afterwards would prove nothing. Arming the Points tool is a \
             command and `decline::retire` runs at the top of `dispatch_command`, so this \
             should be impossible — a decline surviving a command is its own defect and is \
             worth filing separately. SKIPPED rather than failed because this check's \
             subject is a different one and it can no longer measure it. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("control: the decline slot is empty before the drag");

    // --- 4: drag the line ---------------------------------------------------
    let mark = trace.mark();
    driver.drag(at, frame.offset_from(at, DRAG_PX, DRAG_PX))?;
    session.settle(26);
    let after = session.trace()?;

    // --- 5: the verdict -----------------------------------------------------
    //
    // Three links, asserted in order, each with its own message. A build that
    // breaks one of them fails at that one rather than at a summary.

    // 5a — did the gesture reach the move rules at all?
    let Some(declined) = after.last_after(MOVE_DECLINED_EVENT, mark) else {
        let moved = after.last_after("canvas-move", mark).map(|l| l.raw.clone());
        return Err(Error::new(format!(
            "the drag produced no `{MOVE_DECLINED_EVENT}` line, so the release never reached \
             `canvas::moving::drag`'s refusal and this check never exercised its subject. \
             {} Two ordinary causes, and this harness cannot tell them apart from the trace \
             alone: the press landed on paper rather than on the selected run, so the \
             gesture became a marquee; or the travel was below the drag threshold. SKIPPED \
             rather than failed. Trace: {}.",
            moved.map_or_else(
                || "No `canvas-move` line either, so nothing committed.".to_owned(),
                |raw| format!("A move COMMITTED instead: `{raw}`.")
            ),
            session.trace_path().display()
        )));
    };
    let reason = declined.get("reason").unwrap_or("?");
    if reason != REASON_RUN {
        return Err(Error::new(format!(
            "the drag was refused for `{reason}`, not `{REASON_RUN}`, so the aim did not land \
             on one line inside a block of text and this check never exercised its subject. \
             The line seen was `{}`. SKIPPED rather than failed: a refusal for another cause \
             says nothing about O188 in either direction. Trace: {}.",
            declined.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "the move rules refused, for the reason under test: `{}`",
        declined.raw
    ));

    // 5b — ★★★ THE O188 DEFECT ITSELF. Did the refusal raise a sentence?
    let at_decline = declined.lineno;
    let recorded = after
        .events(RECORDED_EVENT)
        .find(|l| l.lineno > at_decline && l.get("what") == Some(RECORDED_TEXT_RUN));
    if recorded.is_none() {
        let other = after
            .events(RECORDED_EVENT)
            .find(|l| l.lineno > at_decline)
            .map(|l| l.raw.clone());
        return Ok(Some(format!(
            "★★★ THE DEFECT: the drag on one line of text was refused and the operator was \
             told NOTHING. The refusal is correct and is not the bug — `pdfcer-core` has no \
             verb that moves one show operator, so `canvas::moving::eligible` declines with \
             `NoVerbForPart(Run)` before the ghost slides, which is what keeps the preview \
             truthful. **The bug is the silence after it**: no `{RECORDED_EVENT} \
             what={RECORDED_TEXT_RUN}` line follows `{}`, so `Refusal::worded` returned \
             `None` for this refusal, no `Action::DeclineOnCanvas` was raised, and the \
             status bar's decline slot was never given a sentence to draw. {} This is O188 \
             and it is the founding defect class of this project: the operator drags, \
             nothing moves, and nothing says why. Trace: {}.",
            declined.raw,
            other.map_or_else(
                || "No decline of any kind was recorded after the refusal.".to_owned(),
                |raw| format!("A DIFFERENT decline was recorded instead: `{raw}`.")
            ),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "the refusal's sentence crossed the `Action` boundary and reached the store: \
         `{RECORDED_EVENT} what={RECORDED_TEXT_RUN}`"
    ));

    // 5c — was it DRAWN? A sentence in a store nobody reads is still silence.
    if driving::declared(&after, ui_rect, DECLINE_REGION).is_none() {
        return Ok(Some(format!(
            "the refusal recorded its sentence and the status bar never drew it. \
             `{RECORDED_EVENT} what={RECORDED_TEXT_RUN}` is in the trace, so the gesture \
             refused and the apply phase wrote the store — and no `{DECLINE_REGION}` region \
             is on screen on any later frame, which means `decline::live` filtered it out \
             (a `still_true` predicate that retires the sentence on the frame it is \
             written) or `decline::line` has no catalog entry for it. The operator sees \
             exactly what O188 reported: a drag that did nothing, in silence. Regions \
             beginning `status-group:` that ARE declared: {}. Trace: {}.",
            list(&driving::declared_names(&after, ui_rect, "status-group:")),
            session.trace_path().display()
        )));
    }
    report.note(
        "★★ the decline slot is on screen after the drag — refused, worded, and drawn, all \
         three",
    );

    Ok(None)
}

/// Render a list of region names for a failure message, or say there were
/// none.
///
/// ★ A failure message that prints `[]` for an empty list makes the reader
/// wonder whether the query was wrong. Saying *none* answers that.
fn list(names: &[String]) -> String {
    if names.is_empty() {
        "none".to_owned()
    } else {
        names.join(", ")
    }
}
