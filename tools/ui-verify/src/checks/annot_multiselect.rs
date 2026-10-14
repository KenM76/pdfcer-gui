//! `several_annotations_align_as_one_undo_step` — markups, a link and a ce
//! dimension selected by Shift-click and by marquee, aligned together, the
//! alignment taken back by one Undo, judged in saved copies; and a marquee
//! that catches page content too keeps the content and says why.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/annot_multiselect.md`.

use super::dimdrive::{Fixture, click_on, launch_with};
use super::driving;
use super::multiselect::{after, band, save, shift_click, step, tops};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

pub(super) const FIXTURE: Fixture = Fixture {
    file: "annots-align.pdf",
    method: "Rebuild it with `python fixtures/annots-align.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 300.0,
    },
};
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name

/// Each annotation: the substring that finds it in a saved copy, a point on
/// it, and its top edge in the fixture.
pub(super) const SQUARE: (&str, (f64, f64), f64) = ("/NM (SQ)", (70.0, 220.0), 240.0); // ui-text-exempt: a PDF key
pub(super) const CIRCLE: (&str, (f64, f64), f64) = ("/NM (CI)", (170.0, 200.0), 220.0); // ui-text-exempt: a PDF key
pub(super) const LINK: (&str, (f64, f64), f64) = ("/NM (LK)", (270.0, 170.0), 180.0); // ui-text-exempt: a PDF key
const DIMENSION: (&str, (f64, f64), f64) = ("/LineDimension", (130.0, 100.0), 108.5); // ui-text-exempt: a PDF key
/// A crossing band, dragged right to left, over the link and the content
/// square and nothing else.
const MIXED_BAND: ((f64, f64), (f64, f64)) = ((380.0, 10.0), (230.0, 190.0));
/// A window band holding the square and the circle and nothing else.
const MARKUP_BAND: ((f64, f64), (f64, f64)) = ((20.0, 250.0), (210.0, 170.0));

const SELECTED: &str = "annot-select"; // ui-text-exempt: a trace event name
const TOGGLED: &str = "annot-toggled"; // ui-text-exempt: a trace event name
const BANDED: &str = "annot-band"; // ui-text-exempt: a trace event name
const APPLIED: &str = "move-mixed-applied"; // ui-text-exempt: a trace event name
const DISCLOSURE: &str = "status-group:edit-disclosure"; // ui-text-exempt: a trace region name

/// See the module documentation.
pub struct SeveralAnnotationsAlignAsOneUndoStep;

impl Check for SeveralAnnotationsAlignAsOneUndoStep {
    fn name(&self) -> &'static str {
        "several_annotations_align_as_one_undo_step"
    }

    fn defect(&self) -> &'static str {
        "Shift-click or a marquee selects only one markup, link or ce dimension, so Align and \
         Distribute cannot act on them; or they move as one undo step each; or a marquee over \
         markups and page content drops the markups without saying so"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let aligned = ctx.out("annot-multiselect.aligned.pdf");
        let undone = ctx.out("annot-multiselect.undone.pdf");
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
            "annot-multiselect",
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
    if let Some(why) = mixed_band(ctx, report, session, pointer)? {
        return Ok(Some(why));
    }

    // --- one markup by a plain click, three more by Shift-click -----------
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, SQUARE.1)?;
    let selected = step!(after(session, SELECTED, mark, "the square was clicked"));
    if selected.get("subtype") != Some("Square") {
        return Ok(Some(format!(
            "a click on the square selected `{}`.",
            selected.raw
        )));
    }
    for (n, (key, at, _)) in [CIRCLE, LINK, DIMENSION].into_iter().enumerate() {
        let mark = session.trace()?.mark();
        shift_click(ctx, session, pointer, &FIXTURE, at)?;
        let toggled = step!(after(
            session,
            TOGGLED,
            mark,
            &format!("`{key}` was Shift-clicked")
        ));
        report.note(format!("`{}`", toggled.raw));
        let (before, now) = ((n + 1).to_string(), (n + 2).to_string());
        if toggled.get("before") != Some(before.as_str())
            || toggled.get("after") != Some(now.as_str())
        {
            return Ok(Some(format!(
                "Shift-clicking `{key}` with {before} annotation(s) selected traced `{}`: \
                 before={before} after={now} was wanted.",
                toggled.raw
            )));
        }
    }

    // --- Align top: three move, the dimension by its own verb, one step ----
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
    if applied.get("annots") != Some("3/3") || applied.get("one_step") != Some("true") {
        return Ok(Some(format!(
            "Align top over four annotations, three below the square's top, traced `{}`: \
             three annotations moved and one undo step was wanted.",
            applied.raw
        )));
    }
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let top = SQUARE.2;
    let all_top = [
        (SQUARE.0, top),
        (CIRCLE.0, top),
        (LINK.0, top),
        (DIMENSION.0, top),
    ];
    if let Some(why) = tops(aligned, &all_top)? {
        return Ok(Some(format!("after Align top, {why}")));
    }

    // --- one Undo takes all three moves back ------------------------------
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(40);
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let restored = [SQUARE, CIRCLE, LINK, DIMENSION].map(|(key, _, top)| (key, top));
    if let Some(why) = tops(undone, &restored)? {
        return Ok(Some(format!(
            "after ONE Undo of the alignment, {why} The moves were not one undo step."
        )));
    }

    // --- a marquee over two markups selects both, and Align reads them ----
    let mark = session.trace()?.mark();
    band(ctx, session, pointer, &FIXTURE, MARKUP_BAND)?;
    let banded = step!(after(
        session,
        BANDED,
        mark,
        "a marquee was dragged around the square and the circle"
    ));
    report.note(format!("`{}`", banded.raw));
    if banded.get("caught") != Some("2")
        || banded.get("kept") != Some("annots")
        || banded.get("selected") != Some("2")
    {
        return Ok(Some(format!(
            "a window marquee around the square and the circle traced `{}`: both caught and \
             selected was wanted.",
            banded.raw
        )));
    }
    // Only the circle is right of the square's left edge.
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
    if applied.get("annots") != Some("1/1") {
        return Ok(Some(format!(
            "Align left over the marquee's square and circle traced `{}`: the circle alone \
             moving was wanted.",
            applied.raw
        )));
    }
    Ok(None)
}

/// A crossing band over the link and the content square keeps the content
/// and puts the reason on the status line.
fn mixed_band(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("no ui-rect event in this profile"))?;
    if driving::declared(&session.trace()?, ui_rect, DISCLOSURE).is_some() {
        return Ok(Some(format!(
            "`{DISCLOSURE}` was on screen before anything was done, so the band's own \
             sentence could not be told from it."
        )));
    }
    let mark = session.trace()?.mark();
    band(ctx, session, pointer, &FIXTURE, MIXED_BAND)?;
    let banded = step!(after(
        session,
        BANDED,
        mark,
        "a crossing marquee was dragged over the link and the content square"
    ));
    report.note(format!("`{}`", banded.raw));
    if banded.get("caught") != Some("1") || banded.get("kept") != Some("content") {
        return Ok(Some(format!(
            "a crossing marquee over the link and a content square traced `{}`: the link \
             caught and the content kept was wanted.",
            banded.raw
        )));
    }
    session.settle(20);
    if driving::declared(&session.trace()?, ui_rect, DISCLOSURE).is_none() {
        return Ok(Some(format!(
            "the marquee left the link out of a content selection and `{DISCLOSURE}` was not \
             drawn: the operator is not told why the link was not selected."
        )));
    }
    Ok(None)
}
