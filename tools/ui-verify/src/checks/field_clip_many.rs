//! `several_form_fields_copy_cut_duplicate_as_one_undo_step` — a text field
//! and a check box selected together are duplicated by `Ctrl+D` as one undo
//! step, then cut by `Ctrl+X` and pasted back by `Ctrl+V` with their layout
//! kept, judged by the boxes saved copies list on the page; then one field
//! still copies, pastes and duplicates by the one-field route.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/field_clip_many.md`.

use super::dimdrive::{Fixture, click_on, launch_with};
use super::multiselect::{after, save, shift_click, step, window_at};
use super::savedannots::listed_annots;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "all-field-kinds.pdf",
    method: "Rebuild it with `python fixtures/all-field-kinds.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 420.0,
        height_pt: 620.0,
    },
};
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
const CLIPBOARD_ENV: &str = "PDFCER_DIAG_CLIPBOARD_DIR"; // ui-text-exempt: an environment variable name
const TOGGLED: &str = "form-field-toggled"; // ui-text-exempt: a trace event name
const COPIED: &str = "fieldclip-copy"; // ui-text-exempt: a trace event name
const DUPLICATED: &str = "fieldclip-duplicate-many"; // ui-text-exempt: a trace event name
const PASTED: &str = "paste-fields-applied"; // ui-text-exempt: a trace event name
const PASTED_ONE: &str = "paste-field-applied"; // ui-text-exempt: a trace event name
const DELETED: &str = "delete-widgets-applied"; // ui-text-exempt: a trace event name

/// A box's name, the lower-left corner of its `/Rect`, and a point on it.
type Spot = (&'static str, (f64, f64), (f64, f64));
/// `[170 520 370 542]`.
const TEXT: Spot = ("TextOne", (170.0, 520.0), (270.0, 531.0)); // ui-text-exempt: a fixture field name
/// `[170 486 186 502]`.
const CHECK: Spot = ("CheckOne", (170.0, 486.0), (178.0, 494.0)); // ui-text-exempt: a fixture field name
/// A point on `ComboOne`, `[170 412 370 434]`, for the one-field steps.
const COMBO: (f64, f64) = (270.0, 423.0);
/// The duplicates: each box moved by the paste offset, `(+10, -10)`.
const TEXT_DUP: (f64, f64) = (180.0, 510.0);
const CHECK_DUP: (f64, f64) = (180.0, 476.0);
/// Where the paste is aimed. The two boxes' union, `[170 486 370 542]`, is
/// centred on it: a displacement of `(-60, +66)` for both.
const PASTE_AT: (f64, f64) = (210.0, 580.0);
const TEXT_PASTED: (f64, f64) = (110.0, 586.0);
const CHECK_PASTED: (f64, f64) = (110.0, 552.0);
/// The fixture's page lists ten widgets.
const LISTED: usize = 10;

/// See the module documentation.
pub struct SeveralFormFieldsCopyCutDuplicateAsOneUndoStep;

impl Check for SeveralFormFieldsCopyCutDuplicateAsOneUndoStep {
    fn name(&self) -> &'static str {
        "several_form_fields_copy_cut_duplicate_as_one_undo_step"
    }

    fn defect(&self) -> &'static str {
        "Copy, Cut or Duplicate with several form fields selected acts on the primary one only, \
         or the pasted fields lose their layout, or take one Undo per field to remove"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let saves = ["dup", "undone", "cut", "pasted"]
            .map(|s| ctx.out(&format!("field-clip-many.{s}.pdf")));
        for p in &saves {
            let _ = std::fs::remove_file(p);
        }
        let queue = saves
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(";");
        let env = [
            (SAVE_PATH_ENV, queue),
            (
                CLIPBOARD_ENV,
                ctx.out("field-clip-many-clipboard").display().to_string(),
            ),
        ];
        let outcome = launch_with(
            ctx,
            &mut report,
            &FIXTURE,
            "mode.edit",
            "field-clip-many",
            &env,
        )
        .and_then(|(session, pointer)| {
            let outcome = drive(ctx, &mut report, &session, &pointer, &saves);
            let parked = pointer.gone(&session);
            match outcome? {
                Some(failure) => Ok(Some(failure)),
                None => parked.map(|_| None),
            }
        });
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    saves: &[std::path::PathBuf; 4],
) -> Result<Option<String>> {
    // --- Ctrl+D over two fields: two new boxes, one step ------------------
    if let Some(why) = select_both(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let mark = session.trace()?.mark();
    pointer.key(session, None, "D", Some("ctrl"))?;
    session.settle(40);
    let routed = step!(after(session, DUPLICATED, mark, "Ctrl+D was pressed"));
    if routed.get("fields") != Some("2") {
        return Ok(Some(format!(
            "Ctrl+D over two selected fields traced `{}`.",
            routed.raw
        )));
    }
    if let Some(why) = one_step(session, report, mark, "Ctrl+D")? {
        return Ok(Some(why));
    }
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let dup = [
        (TEXT.1, true),
        (CHECK.1, true),
        (TEXT_DUP, true),
        (CHECK_DUP, true),
    ];
    if let Some(why) = listed(&saves[0], &dup, LISTED + 2)? {
        return Ok(Some(format!("after Ctrl+D, {why}")));
    }

    // --- one Undo removes both duplicates -----------------------------------
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(40);
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let undone = [(TEXT_DUP, false), (CHECK_DUP, false)];
    if let Some(why) = listed(&saves[1], &undone, LISTED)? {
        return Ok(Some(format!(
            "after ONE Undo of the duplicate, {why} The duplicates were not one undo step."
        )));
    }

    cut_and_paste(ctx, report, session, pointer, saves)
}

/// Copy and cut both fields, then paste them back at a point.
fn cut_and_paste(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    saves: &[std::path::PathBuf; 4],
) -> Result<Option<String>> {
    // --- Ctrl+C, Ctrl+X: both copied, both deleted in one step ----------------
    if let Some(why) = select_both(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let mark = session.trace()?.mark();
    pointer.key(session, None, "C", Some("ctrl"))?;
    session.settle(20);
    let copied = step!(after(session, COPIED, mark, "Ctrl+C was pressed"));
    if copied.get("fields") != Some("2") {
        return Ok(Some(format!(
            "Ctrl+C over two selected fields traced `{}`.",
            copied.raw
        )));
    }
    let mark = session.trace()?.mark();
    pointer.key(session, None, "X", Some("ctrl"))?;
    session.settle(40);
    let cut = step!(after(session, DELETED, mark, "Ctrl+X was pressed"));
    report.note(format!("`{}`", cut.raw));
    if cut.get("deleted") != Some("2/2") || cut.get("one_step") != Some("true") {
        return Ok(Some(format!(
            "Ctrl+X over two selected fields traced `{}`: both deleted as one step was wanted.",
            cut.raw
        )));
    }
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let gone = [(TEXT.1, false), (CHECK.1, false)];
    if let Some(why) = listed(&saves[2], &gone, LISTED - 2)? {
        return Ok(Some(format!("after Ctrl+X, {why}")));
    }

    // --- Ctrl+V at a point: both back, layout kept ----------------------------
    pointer.hover(session, window_at(ctx, session, &FIXTURE, PASTE_AT)?)?;
    session.settle(10);
    let mark = session.trace()?.mark();
    pointer.key(session, None, "V", Some("ctrl"))?;
    session.settle(40);
    if let Some(why) = one_step(session, report, mark, "Ctrl+V")? {
        return Ok(Some(why));
    }
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let pasted = [(TEXT_PASTED, true), (CHECK_PASTED, true)];
    if let Some(why) = listed(&saves[3], &pasted, LISTED)? {
        return Ok(Some(format!(
            "after Ctrl+V, {why} The pasted fields did not keep their layout around the pointer."
        )));
    }
    one_field(ctx, session, pointer)
}

/// One field still copies, pastes and duplicates by the one-field route.
fn one_field(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    click_on(ctx, session, pointer, &FIXTURE, 0, COMBO)?;
    session.settle(20);
    let mark = session.trace()?.mark();
    pointer.key(session, None, "C", Some("ctrl"))?;
    session.settle(20);
    let copied = step!(after(
        session,
        COPIED,
        mark,
        "Ctrl+C was pressed on ComboOne"
    ));
    if copied.get("fields") != Some("1") {
        return Ok(Some(format!(
            "Ctrl+C on one field traced `{}`.",
            copied.raw
        )));
    }
    for (key, what) in [("V", "Ctrl+V"), ("D", "Ctrl+D")] {
        let mark = session.trace()?.mark();
        pointer.key(session, None, key, Some("ctrl"))?;
        session.settle(30);
        let placed = step!(after(
            session,
            PASTED_ONE,
            mark,
            &format!("{what} was pressed with one field selected")
        ));
        if placed.get("widgets") != Some("1") {
            return Ok(Some(format!(
                "{what} on one field traced `{}`.",
                placed.raw
            )));
        }
    }
    Ok(None)
}

/// Click TextOne, then Shift-click CheckOne.
fn select_both(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    click_on(ctx, session, pointer, &FIXTURE, 0, TEXT.2)?;
    session.settle(20);
    let mark = session.trace()?.mark();
    shift_click(ctx, session, pointer, &FIXTURE, CHECK.2)?;
    let toggled = step!(after(session, TOGGLED, mark, "CheckOne was Shift-clicked"));
    if toggled.get("before") != Some("1") {
        return Ok(Some(format!(
            "Shift-clicking {} with TextOne selected traced `{}`.",
            CHECK.0, toggled.raw
        )));
    }
    Ok(None)
}

/// The failure sentence unless `gesture` placed both fields as one step.
fn one_step(
    session: &Session,
    report: &mut CheckReport,
    mark: usize,
    gesture: &str,
) -> Result<Option<String>> {
    let pasted = step!(after(
        session,
        PASTED,
        mark,
        &format!("{gesture} was pressed")
    ));
    report.note(format!("`{}`", pasted.raw));
    if pasted.get("pasted") != Some("2/2") || pasted.get("one_step") != Some("true") {
        return Ok(Some(format!(
            "{gesture} over two fields traced `{}`: both placed as one undo step was wanted.",
            pasted.raw
        )));
    }
    Ok(None)
}

/// The failure sentence when the saved page lists a box where it should not,
/// misses one, or lists the wrong number. Boxes are found by their lower-left
/// corner.
fn listed(
    path: &std::path::Path,
    want: &[((f64, f64), bool)],
    count: usize,
) -> Result<Option<String>> {
    let bytes = std::fs::read(path)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", path.display())))?;
    let annots = listed_annots(&bytes);
    for (corner, present) in want {
        let found = annots
            .iter()
            .any(|a| (a.rect[0] - corner.0).abs() < 0.5 && (a.rect[1] - corner.1).abs() < 0.5);
        if found != *present {
            let state = if found { "lists a" } else { "lists no" };
            return Ok(Some(format!(
                "{} {state} box at ({}, {}).",
                path.display(),
                corner.0,
                corner.1
            )));
        }
    }
    if annots.len() != count {
        return Ok(Some(format!(
            "{} lists {} boxes on the page; {count} were wanted.",
            path.display(),
            annots.len()
        )));
    }
    Ok(None)
}
