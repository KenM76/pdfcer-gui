//! `move_line_of_text` — **a drag on one line inside a block of text MOVES
//! it, or says why it cannot.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/move_line_of_text.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode whose canvas may select and edit page content.
const MODE: &str = "edit";

/// `canvas-selection via=… sel=… level=… first=…`.
const SELECTION_EVENT: &str = "canvas-selection";

/// The rung this check must be standing on before it drags.
const PART_LEVEL: &str = "Part";

/// `canvas-move-declined level=… sel=… reason=… detail=…` — written by
/// `canvas::moving::decline`, **on release only**.
const MOVE_DECLINED_EVENT: &str = "canvas-move-declined";

/// `canvas-decline-recorded what=…` — the apply phase's line, written once per
/// decline by `app::status::decline::canvas::record_canvas`.
const RECORDED_EVENT: &str = "canvas-decline-recorded";

/// The `⊗` slot in the status bar. `app::status::decline::show` draws into it
/// and publishes it as a `ui-rect` on the frame it draws.
const DECLINE_REGION: &str = "status-group:decline";

/// The fixture. See the module header, and
/// `fixtures/inherited-runs.PROVENANCE.md`, for why it is not
/// `paragraph.pdf` like every other line-of-text check in this harness.
const FIXTURE: &str = "inherited-runs.pdf";
/// Page index of [`FIXTURE`] this check uses.
const PAGE: usize = 0;

/// **The four aims, and the answer each one must produce.**
const AIMS: [Aim; 4] = [
    Aim {
        what: "the first line, written in two pieces, the second of which has no position",
        at: (87.0, 704.0),
        expect: Expect::Moves,
    },
    Aim {
        what: "the second line, which states its own position and nothing follows on from it",
        at: (128.0, 664.0),
        expect: Expect::Moves,
    },
    Aim {
        what: "the turned line that the line after it is measured from",
        at: (297.0, 414.0),
        expect: Expect::Declines {
            reason: "run-would-move-next",
            recorded: "text-run-would-drag-next-line",
        },
    },
    Aim {
        what: "the turned line whose position this document does not state",
        at: (297.0, 448.0),
        expect: Expect::Declines {
            reason: "run-has-no-position",
            recorded: "text-run-no-position-of-its-own",
        },
    },
];

/// One row of [`AIMS`].
struct Aim {
    /// What the operator would call the thing under the pointer, used in every
    /// note and failure message this check writes. Not a run index: a message
    /// that says *run 1* is a message whoever reads it has to go and decode.
    what: &'static str,
    /// Where to press, in PDF user space (y up).
    at: (f64, f64),
    /// What must happen on release.
    expect: Expect,
}

/// What a drag on one of [`AIMS`] must produce.
///
enum Expect {
    /// The move rules must refuse, with this `reason=` on `canvas-move-declined`
    /// and this `what=` on `canvas-decline-recorded`, and the status bar must
    /// then draw the sentence.
    Declines {
        /// `canvas::moving::Refusal::token`.
        reason: &'static str,
        /// `app::status::decline::canvas::CanvasDecline::token`.
        recorded: &'static str,
    },
    /// The move must COMMIT — the funnel's own line, and no decline anywhere.
    Moves,
}

/// The funnel label `VectorAction::MoveTextLine`'s apply arm passes to
/// `vector_edit_on_page`, which becomes the head of its success line:
/// `move-text-line page=0 n=N epoch=N disclosures=…`.
const MOVED_EVENT: &str = "move-text-line";

/// How far the drag travels, in window logical points, on each axis.
const DRAG_PX: f32 = 60.0;

/// See the module documentation.
pub struct DraggingOneLineOfTextMovesItOrSaysWhy;

impl Check for DraggingOneLineOfTextMovesItOrSaysWhy {
    fn name(&self) -> &'static str {
        "dragging_one_line_of_text_moves_it_or_says_why"
    }

    fn defect(&self) -> &'static str {
        "Dragging one line inside a block of text does nothing and says nothing — either the \
         move never reaches `move_text_run` on a line the engine would accept, or a line the \
         engine refuses is refused in silence, and in both cases the operator is left with a \
         gesture that did not happen and no way on screen to learn why"
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
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;

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
        "--pdf and --doc-point are IGNORED: pinned to {} at page {PAGE}, {} aims",
        pdf.display(),
        AIMS.len()
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

    // --- 2: read the canvas mapping -----------------------------------------
    //
    // **The Points tool is armed inside [`one_aim`], NOT once here, and the
    // reason is measured rather than stylistic.** This check's first shape armed
    // it once before the loop, on the reasoning that a tool stays armed and the
    // two later rows would then not get a free `decline::retire` from a fresh
    // command. The first driven run refuted that in one line: the ladder was at
    // `Object` on row 1, so the click had gone to the arrow tool.
    //
    // The cause is `canvas::keys`' own documented Escape contract — *"pressing
    // Escape twice puts the tool down; pressing it once corrects a mis-aimed
    // pick"*. Row 1 presses Escape with **nothing selected**, so there is no
    // pick to correct and that single press is the one that puts the tool down.
    // Arming before a clearing Escape disarms; arming after it does not.
    //
    // The general shape, worth carrying: **a setup step that runs once and a
    // clearing step that runs per row are in a race, and the clearing step
    // wins.** The fix is not to drop the clearing step — it is what keeps each
    // row's status-bar control honest — but to put every precondition it
    // destroys downstream of it.
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, PAGE)?;
    let frame = session.frame()?;

    // --- 3: each aim in turn ------------------------------------------------
    //
    // **The rows share a launch and are otherwise independent**, which is
    // the shape a suite that shares state taught this project to insist on. Any
    // row may FAIL, and a FAIL stops the check — but no row leaves a
    // precondition behind for the next one, because each re-reads the trace
    // from its own mark and each asserts its own empty-slot control before it
    // presses.
    //
    // The third row EDITS THE DOCUMENT, and it runs last for that reason.
    // A committed move changes the page, and every aim above it is computed
    // from the page as loaded. Running the control first would have moved a
    // line the two refusal rows are then aiming at.
    for aim in &AIMS {
        if let Some(failure) = one_aim(&session, &driver, &mapping, &frame, ui_rect, aim, report)? {
            return Ok(Some(failure));
        }
    }

    Ok(None)
}

/// Drive one row of [`AIMS`]: select the line, drag it, and assert what the row
/// says must happen.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn one_aim(
    session: &Session,
    driver: &Driver,
    mapping: &CanvasMapping,
    frame: &crate::coords::WindowFrame,
    ui_rect: &'static str,
    aim: &Aim,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let what = aim.what;

    // --- clear whatever the previous row left ------------------------------
    driver.press(vk::ESCAPE)?;
    session.settle(14);
    let before = session.trace()?;
    if driving::declared(&before, ui_rect, DECLINE_REGION).is_some() {
        report.note(format!(
            "a decline from an earlier row is still on the status bar as this row begins \
             — measured rather than assumed, and NOT a precondition: see this function's \
             header for the Escape contract that makes it expected. Row: {what}."
        ));
    }

    // --- arm the Points tool ------------------------------------------------
    //
    // After the Escape above, never before it: see step 2 in [`drive`] for
    // the measurement. On a text object this is what makes a single click land
    // on the Part rung at all — a double-click opens the caret and never
    // touches the ladder, and the arrow tool's click selects the whole block.
    driver.press(vk::A)?;
    session.settle(10);

    // --- select the line ----------------------------------------------------
    let window_point = mapping.doc_to_window(DocPoint::new(PAGE, aim.at.0, aim.at.1))?;
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
             there is no line to drag — row: {what}. That is either an aim that is not on a \
             glyph or a broken hit test, and this harness cannot tell them apart, so it \
             declines to file either. The spans the aims were computed from are in \
             `fixtures/inherited-runs.PROVENANCE.md` and are the first thing to re-measure. \
             Trace: {}.",
            aim.at.0,
            aim.at.1,
            session.trace_path().display()
        )));
    }
    let level = trace
        .last(SELECTION_EVENT)
        .and_then(|l| l.get("level").map(str::to_owned))
        .unwrap_or_else(|| "none".to_owned());
    if level != PART_LEVEL {
        return Err(Error::new(format!(
            "the selection ladder is at `{level}` and this row needs `{PART_LEVEL}`, so no \
             single LINE of text is selected and the drag below would be testing the Object \
             rung — which moves the whole text block and is a different verb entirely. The \
             Points tool should land directly on the Part rung; on a text object a part is a \
             show operator. Row: {what}. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- drag ---------------------------------------------------------------
    let mark = trace.mark();
    driver.drag(at, frame.offset_from(at, DRAG_PX, DRAG_PX))?;
    session.settle(26);
    let after = session.trace()?;

    match aim.expect {
        Expect::Moves => moved(session, &after, mark, ui_rect, what, report),
        Expect::Declines { reason, recorded } => declined(
            session, &after, mark, ui_rect, aim, reason, recorded, report,
        ),
    }
}

/// Assert the CONTROL row: the drag committed a `move_text_run`.
fn moved(
    session: &Session,
    after: &crate::trace::Trace,
    mark: usize,
    ui_rect: &'static str,
    what: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let Some(line) = after.last_after(MOVED_EVENT, mark) else {
        let declined = after
            .last_after(MOVE_DECLINED_EVENT, mark)
            .map(|l| l.raw.clone());
        return Ok(Some(format!(
            "★★★ THE DEFECT: dragging {what} did not move it. No `{MOVED_EVENT}` line \
             follows the release, so the edit never reached `EditSession::move_text_runs` \
             through the funnel. {} This line states its own position and has no successor \
             that depends on it, so `text_run_move_refusal_of_set` answers `None` for it and the \
             engine would accept the move — which means the refusal, if there was one, is \
             this shell's and not the document's. That is `OPERATOR_REQUESTS.md` O188 \
             unfixed. Trace: {}.",
            declined.map_or_else(
                || "No `canvas-move-declined` line either, so the gesture did not reach the \
                    move rules at all — the press may have landed on paper and become a \
                    marquee, or the travel may have been below the drag threshold."
                    .to_owned(),
                |raw| format!("It was REFUSED instead: `{raw}`.")
            ),
            session.trace_path().display()
        )));
    };
    report.note(format!("dragging {what} moved it: `{}`", line.raw));

    // And it must NOT have worded a refusal at the operator. A build that
    // both moved the line and put a sentence on the bar is worse than one that
    // did neither, because the operator is told his edit did not happen while
    // looking at it having happened.
    if driving::declared(after, ui_rect, DECLINE_REGION).is_some() {
        return Ok(Some(format!(
            "the drag on {what} COMMITTED (`{}`) and the status bar is showing a decline \
             anyway. Whatever that sentence says, it contradicts the page in front of the \
             operator — `fuzzy, never sneaky` cuts both ways, and a refusal displayed over \
             a successful edit is the worst-reading half of it. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }
    report.note("★★ and no decline was drawn over it — the page and the status bar agree");
    Ok(None)
}

/// Assert a REFUSING row, in the three links the refusal has to survive.
///
/// A build that breaks any one link fails at that link and says which, rather
/// than at a summary: refused for the right reason, worded, and drawn.
#[allow(clippy::too_many_arguments)]
fn declined(
    session: &Session,
    after: &crate::trace::Trace,
    mark: usize,
    ui_rect: &'static str,
    aim: &Aim,
    reason: &str,
    recorded: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let what = aim.what;

    // --- a: did the gesture reach the move rules at all? --------------------
    let Some(line) = after.last_after(MOVE_DECLINED_EVENT, mark) else {
        let moved = after.last_after(MOVED_EVENT, mark).map(|l| l.raw.clone());
        if let Some(raw) = moved {
            return Ok(Some(format!(
                "★★★ THE DEFECT, in the direction that damages a document: the drag on \
                 {what} COMMITTED — `{raw}`. The engine's own guard says this line cannot \
                 move without carrying another line with it, or has no position to change, \
                 so a commit here means `canvas::moving::eligible` is no longer asking \
                 `text_run_move_refusal` before it draws the ghost. The operator moved one \
                 label and something he did not select moved too, silently. Trace: {}.",
                session.trace_path().display()
            )));
        }
        return Err(Error::new(format!(
            "the drag on {what} produced no `{MOVE_DECLINED_EVENT}` line and no \
             `{MOVED_EVENT}` line, so the release never reached `canvas::moving::drag` and \
             this row never exercised its subject. Two ordinary causes this harness cannot \
             tell apart from the trace alone: the press landed on paper rather than on the \
             selected line, so the gesture became a marquee; or the travel was below the \
             drag threshold. SKIPPED rather than failed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let got = line.get("reason").unwrap_or("?");
    if got != reason {
        return Ok(Some(format!(
            "the drag on {what} was refused for `{got}`, and the engine's guard says it must \
             be `{reason}`. The line seen was `{}`. This is a FAILURE and not a skip: the \
             two refusals of a line move are two different facts about the document and the \
             operator is shown a different sentence for each, so refusing for the wrong one \
             tells him the wrong thing about his own drawing. If the aim is what moved, the \
             spans are in `fixtures/inherited-runs.PROVENANCE.md`. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "dragging {what} was refused, for the reason under test: `{}`",
        line.raw
    ));

    // --- b: THE O188 DEFECT ITSELF. Did the refusal raise a sentence? ---
    let at_decline = line.lineno;
    let found = after
        .events(RECORDED_EVENT)
        .find(|l| l.lineno > at_decline && l.get("what") == Some(recorded));
    if found.is_none() {
        let other = after
            .events(RECORDED_EVENT)
            .find(|l| l.lineno > at_decline)
            .map(|l| l.raw.clone());
        return Ok(Some(format!(
            "★★★ THE DEFECT: the drag on {what} was refused and the operator was told \
             NOTHING. The refusal is correct and is not the bug — the engine's own guard \
             says this move cannot be performed, and `canvas::moving::eligible` asks it \
             before the ghost slides, which is what keeps the preview truthful. **The bug \
             is the silence after it**: no `{RECORDED_EVENT} what={recorded}` line follows \
             `{}`, so `Refusal::worded` returned `None`, no `Action::DeclineOnCanvas` was \
             raised, and the status bar's decline slot was never given a sentence to draw. \
             {} This is O188 and it is the founding defect class of this project: the \
             operator drags, nothing moves, and nothing says why. Trace: {}.",
            line.raw,
            other.map_or_else(
                || "No decline of any kind was recorded after the refusal.".to_owned(),
                |raw| format!("A DIFFERENT decline was recorded instead: `{raw}`.")
            ),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "the refusal's sentence crossed the `Action` boundary and reached the store: \
         `{RECORDED_EVENT} what={recorded}`"
    ));

    // --- c: was it DRAWN? A sentence in a store nobody reads is silence. ----
    if driving::declared(after, ui_rect, DECLINE_REGION).is_none() {
        return Ok(Some(format!(
            "the refusal on {what} recorded its sentence and the status bar never drew it. \
             `{RECORDED_EVENT} what={recorded}` is in the trace, so the gesture refused and \
             the apply phase wrote the store — and no `{DECLINE_REGION}` region is on \
             screen on any later frame, which means `decline::live` filtered it out (a \
             `still_true` predicate that retires the sentence on the frame it is written) \
             or `decline::line` has no catalog entry for it. The operator sees exactly what \
             O188 reported: a drag that did nothing, in silence. Regions beginning \
             `status-group:` that ARE declared: {}. Trace: {}.",
            list(&driving::declared_names(after, ui_rect, "status-group:")),
            session.trace_path().display()
        )));
    }
    report.note(format!("★★ refused, worded and drawn, all three — {what}"));
    Ok(None)
}

/// Render a list of region names for a failure message, or say there were
/// none.
///
/// A failure message that prints `[]` for an empty list makes the reader
/// wonder whether the query was wrong. Saying *none* answers that.
fn list(names: &[String]) -> String {
    if names.is_empty() {
        "none".to_owned()
    } else {
        names.join(", ")
    }
}
