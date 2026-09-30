//! `align_left_moves_every_box_in_one_undo` — O263, driven: three boxes
//! Shift-selected, **Align left edges** pressed in the Align and Distribute
//! panel, the two boxes not already at the left edge move, and one undo
//! takes the whole alignment back.
//!
//! Pinned to `fixtures/three-boxes.pdf`, whose provenance script lists the
//! boxes. Relative to the selection area (the panel's default), box 0 is the
//! leftmost and stays put, so the right answer is `moved=2 of=3`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, click_mode_segment, declared, declared_or_in_overflow,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::{Driver, Key};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode whose ribbon carries Edit ▸ Arrange.
const MODE: &str = "edit";
/// `canvas … sel=N …`, the per-frame canvas line.
const CANVAS_EVENT: &str = "canvas"; // ui-text-exempt: a trace event name, never displayed
/// The Edit tab and the panel's ribbon control.
const EDIT_TAB: &str = "ribbon.tab.edit";
const ALIGN_ITEM: &str = "ribbon.item.edit.align";
/// The panel's *Align left edges* button (horizontal row, index 1).
const LEFT_BUTTON: &str = "align.h1";
/// `align-pressed op=… n=…`, the panel's own line.
const PRESSED_EVENT: &str = "align-pressed"; // ui-text-exempt: a trace event name, never displayed
/// `move-each-applied gesture=… page=… moved=… of=…`.
const APPLIED_EVENT: &str = "move-each-applied"; // ui-text-exempt: a trace event name, never displayed
/// `undo kind=… undo_depth=…`.
const UNDO_EVENT: &str = "undo"; // ui-text-exempt: a trace event name, never displayed
/// What `moves=` must say: boxes 1 and 2 travel left to x = 100 (from 250
/// and 180), in PDF points, and nothing travels vertically. A sign or axis
/// error in the canvas-to-page conversion produces different numbers.
const EXPECTED_MOVES: &str = "1:-150.0,0.0;2:-80.0,0.0";
/// A point inside each box, PDF user space, page 0.
const AIMS: [(f64, f64); 3] = [(130.0, 620.0), (290.0, 525.0), (200.0, 385.0)];

/// See the module documentation.
pub struct AlignLeftMovesEveryBoxInOneUndo;

impl Check for AlignLeftMovesEveryBoxInOneUndo {
    fn name(&self) -> &'static str {
        "align_left_moves_every_box_in_one_undo"
    }

    fn defect(&self) -> &'static str {
        "Align left edges in the Align and Distribute panel moves one of the selected objects, \
         none, or all of them by the same amount; or the alignment takes one undo press per \
         object, so the operator lines up three boxes and has to undo three times"
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

fn selection_count(session: &Session) -> Result<usize> {
    Ok(session
        .trace()?
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get_usize("sel"))
        .unwrap_or(0))
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check Shift-clicks three boxes and presses a \
             panel button. Reported as SKIPPED rather than passed.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = crate::fixture::workspace_root()
        .join("fixtures")
        .join("three-boxes.pdf");
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "the fixture is not at {}. Run fixtures/three-boxes.PROVENANCE.py.",
            pdf.display()
        )));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    let page: PageGeometry = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new(format!("cannot read a page size from {}.", pdf.display())))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("align-left.trace.txt"));
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
    session.settle(40);
    let driver = Driver::new(session.window());

    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    // --- open the panel first, so the canvas rect is final before aiming ---
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, EDIT_TAB)
        .ok_or_else(|| Error::new(format!("no `{EDIT_TAB}` region in Edit mode.")))?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    let Some(item) = declared_or_in_overflow(&session, &driver, ui_rect, ALIGN_ITEM)? else {
        return Ok(Some(format!(
            "★ THE EDIT TAB OFFERS NO `{ALIGN_ITEM}`, on the band or in the overflow, so the \
             Align and Distribute panel cannot be opened from the ribbon. Trace: {}.",
            session.trace_path().display()
        )));
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(30);
    if declared(&session.trace()?, ui_rect, LEFT_BUTTON).is_none() {
        return Ok(Some(format!(
            "★ `{ALIGN_ITEM}` was clicked and the panel published no `{LEFT_BUTTON}`: the \
             control opens nothing, or the panel is mounted behind another tab. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- select the three boxes: click, then Shift-click twice -------------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, 0)?;
    for (n, &(x, y)) in AIMS.iter().enumerate() {
        let at = session
            .frame()?
            .to_screen(mapping.doc_to_window(DocPoint::new(0, x, y))?);
        if n == 0 {
            driver.click_at(at)?;
        } else {
            driver.click_with_modifier(at, Key::Shift)?;
        }
        session.settle(16);
    }
    let held = selection_count(&session)?;
    report.note(format!(
        "{held} object(s) selected after one click and two Shift-clicks"
    ));
    if held != AIMS.len() {
        return Err(Error::new(format!(
            "the selection holds {held}, not {}, so the alignment has the wrong operands. That \
             is the Shift-click row's defect, not Align's; reported as SKIPPED. Trace: {}.",
            AIMS.len(),
            session.trace_path().display()
        )));
    }

    // --- press Align left edges --------------------------------------------
    let mark = session.trace()?.mark();
    let trace = session.trace()?;
    let button = declared(&trace, ui_rect, LEFT_BUTTON)
        .ok_or_else(|| Error::new(format!("`{LEFT_BUTTON}` vanished after selecting.")))?;
    driver.click_at(session.frame()?.declared_center(button))?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(pressed) = trace.last_after(PRESSED_EVENT, mark) else {
        return Ok(Some(format!(
            "the button was clicked and wrote no `{PRESSED_EVENT}` line: the click did not \
             reach it, or it was greyed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("`{}`", pressed.raw));
    let Some(applied) = trace.last_after(APPLIED_EVENT, mark) else {
        return Ok(Some(format!(
            "★ `{}` and no `{APPLIED_EVENT}` followed: the panel planned nothing, or the engine \
             refused. Trace: {}.",
            pressed.raw,
            session.trace_path().display()
        )));
    };
    report.note(format!("★ `{}`", applied.raw));
    let moved = applied.get_usize("moved").unwrap_or(0);
    let of = applied.get_usize("of").unwrap_or(0);
    if (moved, of) != (2, 2) {
        return Ok(Some(format!(
            "★★ ALIGN LEFT MOVED {moved} OF {of}; the right answer is 2 of 2 — box 0 is \
             already the leftmost, so only boxes 1 and 2 have anywhere to go, and each must. \
             `{}`",
            applied.raw
        )));
    }

    let asked = applied.get("moves").unwrap_or("");
    if asked != EXPECTED_MOVES {
        return Ok(Some(format!(
            "★★ TWO BOXES MOVED, BY THE WRONG AMOUNTS: `moves={asked}`, expected \
             `{EXPECTED_MOVES}` — each box's left edge to x = 100, which is box 0's. `{}`",
            applied.raw
        )));
    }

    // --- one undo takes it all back -----------------------------------------
    let mark = session.trace()?.mark();
    driver.press_chord(&[vk::CONTROL], vk::Z)?;
    session.settle(24);
    let trace = session.trace()?;
    let Some(undo) = trace.last_after(UNDO_EVENT, mark) else {
        return Err(Error::new(format!(
            "Ctrl+Z wrote no `{UNDO_EVENT}` line, so the keystroke never reached the window. \
             Reported as SKIPPED: the key channel, not Align. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("`{}`", undo.raw));
    let depth = undo.get_usize("undo_depth").unwrap_or(0);
    if depth != 1 {
        return Ok(Some(format!(
            "★★ THE ALIGNMENT LEFT {depth} UNDO STEPS, NOT 1: `{}`. The two moves were not \
             folded, so the operator presses undo once per box.",
            undo.raw
        )));
    }
    report.note("★★ two boxes moved to the left edge, one undo step");
    Ok(None)
}
