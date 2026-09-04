//! `shift_arrows_select_text` — **there is a selection inside a text draft**,
//! driven on a real drawing.
//!
//! # What this is for
//!
//! `OPERATOR_REQUESTS.md` O14 item 11, from the conventions sweep of
//! 2026-08-20:
//!
//! > **No selection inside a draft** — no Shift+arrow, no Ctrl+A, no
//! > drag-select.
//!
//! Every text field the operator has ever used has all three. Without them,
//! replacing a word means pressing Backspace once per character, and replacing
//! a whole title-block cell means pressing it a dozen times.
//!
//! # ★★★ Why this check reads a TRACE and changes nothing
//!
//! The honest way to prove that Shift+Right selected three characters is to
//! type over them and watch the text shrink. This check refuses to.
//!
//! It drives **the operator's own drawing** — that is deliberate and it is what
//! `FEATURES.md` records as the lesson that cost three weeks: *"a check that
//! drives a document this project authored tests the shape this project
//! imagined, and the operator's documents are the only ones with the shape that
//! broke."* But proving a **selection** by making an **edit** is a bad trade:
//! it puts a real change on his document to observe something that was already
//! observable, and a check that has to mutate to measure will eventually mutate
//! and fail to clean up.
//!
//! So the shell publishes `text-select`, carrying the two indices. That is
//! `DEFECTS.md` D14's rule — *a trace line must carry the number a wrong build
//! would get wrong* — used to avoid a side effect rather than to catch a bug.
//!
//! # ★★ The second half is the one nobody writes: the selection must GO AWAY
//!
//! Rule 4 of the four in `canvas::textedit::caret`'s selection section: any
//! movement without Shift drops the selection. It is as important as the
//! selecting, and its absence is invisible until it bites — a highlight left on
//! screen after the caret has walked out of it, and the next keystroke deleting
//! text the operator is no longer looking at.
//!
//! A build with no rule 4 passes every "does Shift+Right select" assertion.
//! This check presses Right afterwards and requires the shell to say `none`.
//!
//! # ★★ The pointer half, added 2026-08-21
//!
//! Step 6 sweeps the pointer across the editor box and requires a selection to
//! come out of it. It is here rather than in a check of its own because it
//! needs everything steps 1-3 establish — a mode, a tool, and a caret in a real
//! run — and a second check would be a second copy of all of it.
//!
//! ★ **Double-click-to-select-a-word is built and is NOT driven here.** Its
//! logic is unit-tested against a real galley, and a driven double click is a
//! gesture this harness has had trouble synthesising before (see
//! `a_synthetic_double_click_must_not_be_two_calls_to_a_settling_click_helper`
//! in the egui RAG). Named rather than quietly implied by a green run.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::save_copy::{click_command, click_tab};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode whose canvas may edit content.
const MODE: &str = "edit";
/// The Edit tab, where the text control lives.
const EDIT_TAB: (&str, &str) = ("ribbon.tab.edit", "edit");
/// The control that arms the caret on existing text.
const TOOL: (&str, &str) = ("ribbon.item.edit.text", "edit.text");
/// `text-edit-caret kind=… page=… run=… len=…`.
const CARET_EVENT: &str = "text-edit-caret";
/// `text-select from=… to=… n=…`, or `text-select none caret=…`.
const SELECT_EVENT: &str = "text-select";
/// How many characters to select, when the run is long enough for that many.
const PRESSES: usize = 3;
/// `text-edit-typing … len=…` — the length of the run the caret is in NOW,
/// which is not always the run that was clicked. See step 3.
const TYPING_EVENT: &str = "text-edit-typing";
/// The editor box's own declared region — the only way to aim at it. See step 6.
const BOX_REGION: &str = "text-edit.box";

/// See the module documentation.
pub struct ShiftArrowsSelectText;

impl Check for ShiftArrowsSelectText {
    fn name(&self) -> &'static str {
        "shift_arrows_select_text"
    }

    fn defect(&self) -> &'static str {
        "there is no selection inside a text draft — no Shift+arrow, no Ctrl+A, no drag-select. \
         Replacing a word means pressing Backspace once per character"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. This check needs a document with text in it."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space naming a point that sits ON TEXT — \
             the caret has to be in a run before there is anything to select.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, arms a tool, \
             clicks text and holds Shift. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("text-selection.trace.txt"));
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

    // --- 1: Edit mode, Edit tab, the caret tool -----------------------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);
    click_tab(&session, &driver, ui_rect, EDIT_TAB)?;
    click_command(&session, &driver, ui_rect, TOOL, 16)?;

    // --- 2: put a caret in real text ----------------------------------------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let at = session
        .frame()?
        .to_screen(mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?);
    driver.click_at(at)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(caret) = trace
        .events(CARET_EVENT)
        .filter(|l| l.get("run").is_some())
        .last()
    else {
        return Err(Error::new(format!(
            "the click at (page {}, {:.1}, {:.1}) placed no caret in a run, so there is nothing \
             to select. A fact about the fixture and the point rather than about the build — aim \
             at a piece of text. SKIPPED for that reason.",
            target.page + 1,
            target.x,
            target.y
        )));
    };
    report.note(format!("★ the caret landed in a run: `{}`", caret.raw));

    // --- 3: Home, so the caret is at a known end ----------------------------
    //
    // ★ Without this the caret is wherever the click put it, which on a run
    // whose right-hand end was clicked is the end — and Shift+Right there
    // selects nothing, correctly, and looks exactly like a build with no
    // selection at all.
    driver.press(vk::HOME)?;
    session.settle(20);

    // ★★ THE RUN IS RE-READ AFTER Home, and the first version of this check did
    // not do that and accused the build of the document's arithmetic.
    //
    // Home reaches the start of the **page's line**, which on a CAD sheet is
    // usually several show operators wide — so it can land the caret in a
    // DIFFERENT run from the one that was clicked. It did: the click landed in
    // an 18-character part number, Home stepped left into the 2-character cell
    // beside it, and three presses on a 2-character run correctly selected two.
    // The check reported *"selected 2, not 3"* about a build that was right.
    //
    // So the budget comes from where the caret actually IS.
    let trace = session.trace()?;
    let len: usize = trace
        .events(TYPING_EVENT)
        .filter_map(|l| l.get("len").and_then(|n| n.parse::<usize>().ok()))
        .last()
        .unwrap_or_default();
    if len < 2 {
        return Err(Error::new(format!(
            "after Home the caret sits in a run of {len} character(s), and a selection needs at \
             least two to be distinguishable from a caret movement. A fact about this point on \
             this document. Aim --doc-point at a longer piece of text. SKIPPED."
        )));
    }
    let presses = PRESSES.min(len);
    report.note(format!(
        "the caret's run holds {len} character(s), so this run presses Shift+Right {presses} times"
    ));

    // --- 4: ★★★ Shift+Right, three times ------------------------------------
    // ★★ HELD, not chorded per press. `press_chord` sends the modifier down
    // and up around each key, and the application traced `Modifiers::NONE` on
    // all three arrows when it was used here — every key arrived, not one
    // carried Shift. See `Driver::press_held`.
    driver.press_held(&[vk::LSHIFT], vk::ARROW_RIGHT, presses)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(sel) = trace
        .events(SELECT_EVENT)
        .filter(|l| l.get("n").is_some())
        .last()
    else {
        return Ok(Some(format!(
            "★ SHIFT+RIGHT SELECTED NOTHING: no `{SELECT_EVENT}` line carrying an `n` after \
             {presses} presses.\n\
             The shell publishes this line on every draft change, with `none` when nothing is \
             selected — so silence about a selection means `Draft::mark` never left `None`. Look \
             at the ArrowRight arm in `canvas::textedit::typing`: it must call `caret::moved` \
             with the event's own `modifiers.shift`, and reading the FRAME's modifiers instead \
             is the classic way for a Shift that was genuinely held to arrive as `false`. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let n: usize = sel
        .get("n")
        .and_then(|v| v.parse().ok())
        .unwrap_or_default();
    if n != presses {
        return Ok(Some(format!(
            "★ SHIFT+RIGHT SELECTED {n} CHARACTER(S), NOT {presses}: `{}`.\n\
             The count is the number a wrong build gets wrong. One short means the mark was \
             planted where the caret ENDED UP rather than where it was, which selects nothing on \
             the first press — see `caret::moved`, whose whole argument is that it takes the \
             caret's position BEFORE the movement. Trace: {}.",
            sel.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ {presses} presses selected {presses} characters: `{}`",
        sel.raw
    ));

    // --- 5: ★★ and an unshifted move DROPS it -------------------------------
    driver.press(vk::ARROW_RIGHT)?;
    session.settle(20);
    let trace = session.trace()?;
    let last = trace.events(SELECT_EVENT).last();
    let dropped = last.is_some_and(|l| l.raw.contains("none"));
    if !dropped {
        return Ok(Some(format!(
            "★ THE SELECTION SURVIVED AN UNSHIFTED MOVE: the last `{SELECT_EVENT}` line is `{}`.\n\
             Rule 4 in `canvas::textedit::caret`'s selection section — any movement without Shift \
             drops it. Its absence leaves a highlight on screen after the caret has walked out of \
             it, and the NEXT keystroke then deletes text the operator is no longer looking at, \
             which is the worst outcome available here. Every movement arm must pass \
             `modifiers.shift` through `caret::moved`; an arm that forgot is one that assigns \
             `draft.caret` without touching `draft.mark`. Trace: {}.",
            last.map_or("<none at all>", |l| l.raw.as_str()),
            session.trace_path().display()
        )));
    }
    report.note("★★ and a plain Right dropped it, so no highlight is left behind".to_owned());

    // --- 6: ★★★ SWEEP ACROSS THE TEXT WITH THE POINTER ----------------------
    //
    // The pointer half of item 11, added 2026-08-21. It is a separate step of
    // this check rather than a check of its own because it needs everything
    // steps 1-3 established — a mode, a tool, a caret in a real run — and a
    // second check would be a second copy of all of it.
    //
    // ★ The editor box is aimed at through its OWN declared region. It is
    // painted into the canvas rather than laid out as a widget, so it appears
    // in no layout the harness can read; and aiming at it through the run's
    // page coordinates would aim at the glyphs the box is COVERING, which is a
    // different rectangle in a different place once the box takes its floor
    // height at a low zoom.
    let trace = session.trace()?;
    let Some(area) = crate::checks::driving::declared(&trace, ui_rect, BOX_REGION) else {
        return Err(Error::new(format!(
            "the draft is live and no `{BOX_REGION}` region was declared, so there is nothing to \
             sweep across. The editor box publishes it every frame it draws; its absence means \
             the caret was committed or abandoned before this step. SKIPPED."
        )));
    };
    let frame = crate::checks::driving::frame_of(&session, &trace, ui_rect, BOX_REGION)?;
    // A tenth in from the left edge to nine tenths across: inside the box on
    // both ends, so the gesture cannot be mistaken for one that began on the
    // page. Vertically centred, which on a one-line draft is the text's row.
    let from = frame.declared_at(area, 0.1, 0.5);
    let to = frame.declared_at(area, 0.9, 0.5);
    driver.drag(from, to)?;
    session.settle(24);

    let trace = session.trace()?;
    let swept = trace
        .events(SELECT_EVENT)
        .filter_map(|l| l.get("n").and_then(|v| v.parse::<usize>().ok()))
        .filter(|n| *n > 0)
        .last();
    let Some(swept) = swept else {
        return Ok(Some(format!(
            "★ THE POINTER SWEEP SELECTED NOTHING: the drag across the editor box raised no \
             `{SELECT_EVENT}` line with a non-zero `n`.\n\
             Three places to look, in order. `canvas::textedit::paint` must publish the box and \
             the galley through `canvas::textedit::hit` — if it does not, the pointer handler \
             has nothing to hit-test against and answers `false` every frame. \
             `canvas::textedit::keys::pointer` must read `press_origin` rather than the current \
             position to decide the gesture belongs to the box. And `place::click` must stand \
             aside for a press inside the box, or the ladder commits the draft and opens a new \
             one from the PAGE at that point — which is not the text on screen, because the box \
             is covering it. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★★★ a pointer sweep across the box selected {swept} character(s) — the gesture that \
         needed the galley to be shared rather than laid out twice"
    ));
    Ok(None)
}
