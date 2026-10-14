//! `several_form_fields_align_as_one_undo_step` — form fields selected by
//! Shift-click and by marquee, aligned together, the alignment taken back by
//! one Undo, judged in saved copies.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/field_multiselect.md`.

use super::dimdrive::{Fixture, click_on, launch_with};
use super::multiselect::{after, band, save, shift_click, step, tops};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
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

/// Each field's name, the key that finds it in a saved copy, and a point on
/// it: `[170 520 370 542]`, `[170 486 186 502]`, `[170 412 370 434]`.
const TEXT: (&str, &str, (f64, f64)) = ("TextOne", "/T (TextOne)", (270.0, 531.0)); // ui-text-exempt: a fixture field name
const CHECK: (&str, &str, (f64, f64)) = ("CheckOne", "/T (CheckOne)", (178.0, 494.0)); // ui-text-exempt: a fixture field name
const COMBO: (&str, &str, (f64, f64)) = ("ComboOne", "/T (ComboOne)", (270.0, 423.0)); // ui-text-exempt: a fixture field name
/// TextOne's top edge, where Align top puts the other two.
const TOP: f64 = 542.0;
/// A window band from empty page that holds CheckOne and both radio members
/// (`[170 452 186 468]`, `[260 452 276 468]`) and nothing else.
const BAND: ((f64, f64), (f64, f64)) = ((160.0, 510.0), (290.0, 444.0));

const SELECTED: &str = "form-field-selected"; // ui-text-exempt: a trace event name
const TOGGLED: &str = "form-field-toggled"; // ui-text-exempt: a trace event name
const BANDED: &str = "form-field-band"; // ui-text-exempt: a trace event name
const APPLIED: &str = "move-mixed-applied"; // ui-text-exempt: a trace event name

/// See the module documentation.
pub struct SeveralFormFieldsAlignAsOneUndoStep;

impl Check for SeveralFormFieldsAlignAsOneUndoStep {
    fn name(&self) -> &'static str {
        "several_form_fields_align_as_one_undo_step"
    }

    fn defect(&self) -> &'static str {
        "Shift-click or a marquee selects only one form field, so Align and Distribute cannot \
         act on fields; or the fields move as one undo step per field, so one Undo leaves the \
         alignment half applied"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let aligned = ctx.out("field-multiselect.aligned.pdf");
        let undone = ctx.out("field-multiselect.undone.pdf");
        for p in [&aligned, &undone] {
            let _ = std::fs::remove_file(p);
        }
        let env = [(
            SAVE_PATH_ENV,
            format!("{};{}", aligned.display(), undone.display()),
        )];
        let outcome = launch_with(
            ctx,
            &mut report,
            &FIXTURE,
            "mode.edit",
            "field-multiselect",
            &env,
        )
        .and_then(|(session, pointer)| {
            let outcome = drive(ctx, &mut report, &session, &pointer, &aligned, &undone);
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
    aligned: &std::path::Path,
    undone: &std::path::Path,
) -> Result<Option<String>> {
    // --- one field by a plain click, two more by Shift-click ---------------
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, TEXT.2)?;
    let selected = step!(after(session, SELECTED, mark, "TextOne was clicked"));
    if selected.get("field") != Some(TEXT.0) {
        return Ok(Some(format!(
            "a click on TextOne selected `{}`.",
            selected.raw
        )));
    }
    for (n, (name, _, at)) in [CHECK, COMBO].into_iter().enumerate() {
        let mark = session.trace()?.mark();
        shift_click(ctx, session, pointer, &FIXTURE, at)?;
        let toggled = step!(after(
            session,
            TOGGLED,
            mark,
            &format!("{name} was Shift-clicked")
        ));
        report.note(format!("`{}`", toggled.raw));
        let before = (n + 1).to_string();
        if toggled.get("field") != Some(name) || toggled.get("before") != Some(before.as_str()) {
            return Ok(Some(format!(
                "Shift-clicking {name} with {before} field(s) selected traced `{}`.",
                toggled.raw
            )));
        }
    }

    // --- Align top: two fields move, as one undo step ----------------------
    let mark = session.trace()?.mark();
    pointer.key(session, None, "8", Some("ctrl+alt"))?;
    session.settle(40);
    let applied = step!(after(
        session,
        APPLIED,
        mark,
        "Ctrl+Alt+8 (Align top) was pressed"
    ));
    report.note(format!("`{}`", applied.raw));
    if applied.get("widgets") != Some("2/2") || applied.get("one_step") != Some("true") {
        return Ok(Some(format!(
            "Align top over three fields, two below the third's top, traced `{}`: two \
             widgets moved and one undo step was wanted.",
            applied.raw
        )));
    }
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    if let Some(why) = tops(aligned, &[(TEXT.1, TOP), (CHECK.1, TOP), (COMBO.1, TOP)])? {
        return Ok(Some(format!("after Align top, {why}")));
    }

    // --- one Undo takes both moves back ------------------------------------
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(40);
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    if let Some(why) = tops(undone, &[(TEXT.1, TOP), (CHECK.1, 502.0), (COMBO.1, 434.0)])? {
        return Ok(Some(format!(
            "after ONE Undo of the alignment, {why} The moves were not one undo step."
        )));
    }

    // --- a marquee from empty page catches three widgets -------------------
    let mark = session.trace()?.mark();
    band(ctx, session, pointer, &FIXTURE, BAND)?;
    let banded = step!(after(
        session,
        BANDED,
        mark,
        "a marquee was dragged over three widgets"
    ));
    report.note(format!("`{}`", banded.raw));
    if banded.get("caught") != Some("3") || banded.get("merge") != Some("replace") {
        return Ok(Some(format!(
            "a window marquee around CheckOne and both radio members traced `{}`: three \
             caught, replacing the selection, was wanted.",
            banded.raw
        )));
    }
    // The marquee's selection is what Align reads: only radio member B is off
    // the left edge, so Align left moves exactly one widget.
    let mark = session.trace()?.mark();
    pointer.key(session, None, "4", Some("ctrl+alt"))?;
    session.settle(40);
    let applied = step!(after(
        session,
        APPLIED,
        mark,
        "Ctrl+Alt+4 (Align left) was pressed"
    ));
    report.note(format!("`{}`", applied.raw));
    if applied.get("widgets") != Some("1/1") {
        return Ok(Some(format!(
            "Align left over the marquee's three widgets, one of them right of the others, \
             traced `{}`.",
            applied.raw
        )));
    }
    Ok(None)
}
