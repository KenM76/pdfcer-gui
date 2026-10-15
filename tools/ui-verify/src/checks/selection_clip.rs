//! Copy, Cut and Duplicate over the whole selection: three annotations copy,
//! duplicate and cut together, the cut taken back by one Undo; a page object
//! and a part of a placed drawing each duplicate by Ctrl+D.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/selection_clip.md`.

use super::annot_group_drag::select_members;
use super::annot_multiselect::{CIRCLE, FIXTURE as ANNOTS, LINK, SQUARE};
use super::dimdrive::{click_on, run_on_with};
use super::form_node_move::{enter_leaf_at, run_body};
use super::multiselect::{after, save, step};
use super::nudgecontent::{BOXES, RED_ONLY};
use super::savedannots::{SavedAnnot, listed_on_page, read};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
const CLIPBOARD_ENV: &str = "PDFCER_DIAG_CLIPBOARD_DIR"; // ui-text-exempt: an environment variable name
const COPIED: &str = "clipboard-copy"; // ui-text-exempt: a trace event name
const DUPLICATED: &str = "selection-duplicate"; // ui-text-exempt: a trace event name
const PASTED: &str = "paste-objects-applied"; // ui-text-exempt: a trace event name
const DELETED: &str = "delete-annotations-applied"; // ui-text-exempt: a trace event name
const FORM_FIXTURE: &str = "../../fixtures/half-scale-form.pdf";
/// On the line inside the half-scale drawing, page space: `form_style`'s aim.
const ON_THE_LINE: (f64, f64) = (120.0, 95.0);
/// `canvas::clipboard::PASTE_OFFSET_PT`: a duplicate lands this far down.
const OFFSET: f64 = 10.0;

/// See the module documentation.
pub struct SeveralAnnotationsCopyCutDuplicateAsOneStep;

impl Check for SeveralAnnotationsCopyCutDuplicateAsOneStep {
    fn name(&self) -> &'static str {
        "several_annotations_copy_cut_duplicate_as_one_step"
    }

    fn defect(&self) -> &'static str {
        "Copy, Cut or Duplicate with several annotations selected acts on the primary one only, \
         or the cut takes one Undo per annotation to put back"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let [cut, undone] = saves(ctx);
        for p in [&cut, &undone] {
            let _ = std::fs::remove_file(p);
        }
        let env = [
            (
                SAVE_PATH_ENV,
                format!("{};{}", cut.display(), undone.display()),
            ),
            (
                CLIPBOARD_ENV,
                ctx.out("selection-clip-clipboard").display().to_string(),
            ),
        ];
        run_on_with(
            self,
            ctx,
            &ANNOTS,
            "mode.edit",
            "selection-clip",
            &env,
            annots,
        )
    }
}

/// A trace event, a field on it, and the value that field must carry.
type Want<'a> = (&'a str, &'a str, &'a str);

/// The two saved copies: after the cut, and after one Undo.
fn saves(ctx: &CheckContext) -> [std::path::PathBuf; 2] {
    ["cut", "undone"].map(|s| ctx.out(&format!("selection-clip.{s}.pdf")))
}

/// Press Ctrl+`key`; the failure sentence unless every `(event, field,
/// want)` in `wants` was traced after it with `field` reading `want`.
fn chord(
    session: &Session,
    pointer: &ScriptedPointer,
    report: &mut CheckReport,
    key: &str,
    wants: &[(&str, &str, &str)],
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    pointer.key(session, None, key, Some("ctrl"))?;
    session.settle(40);
    for &(event, field, want) in wants {
        let line = step!(after(
            session,
            event,
            mark,
            &format!("Ctrl+{key} was pressed")
        ));
        report.note(format!("Ctrl+{key}: `{}`", line.raw));
        if line.get(field) != Some(want) {
            return Ok(Some(format!(
                "Ctrl+{key} over three selected annotations traced `{}`: {field}={want} was wanted.",
                line.raw
            )));
        }
    }
    Ok(None)
}

fn annots(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if let Some(why) = select_members(ctx, session, pointer, &[CIRCLE, LINK])? {
        return Ok(Some(why));
    }
    let chords: [(&str, &[Want]); 3] = [
        ("C", &[(COPIED, "annots", "3")]),
        ("D", &[(DUPLICATED, "annots", "3"), (PASTED, "annots", "3")]),
        (
            "X",
            &[(DELETED, "deleted", "3/3"), (DELETED, "one_step", "true")],
        ),
    ];
    for (key, wants) in chords {
        if let Some(why) = chord(session, pointer, report, key, wants)? {
            return Ok(Some(why));
        }
    }
    judge(ctx, session, pointer)
}

/// Save after the cut and after one Undo, judging page 1 only: the fixture's
/// page 2 carries the same three annotations. The cut removed the three
/// originals and left their duplicates; one Undo put all three back.
fn judge(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let [cut, undone] = saves(ctx);
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let after_cut = listed_on_page(&read(&cut)?, 0);
    for (key, _, top) in [SQUARE, CIRCLE, LINK] {
        if after_cut.iter().any(|a| a.body.contains(key)) {
            return Ok(Some(format!(
                "after Ctrl+X, page 1 of {} still lists `{key}`.",
                cut.display()
            )));
        }
        if !at_top(&after_cut, top - OFFSET) {
            return Ok(Some(format!(
                "after Ctrl+D then Ctrl+X, page 1 of {} lists no duplicate of `{key}` with its top at {}.",
                cut.display(),
                top - OFFSET
            )));
        }
    }
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(40);
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let restored = listed_on_page(&read(&undone)?, 0);
    for (key, _, top) in [SQUARE, CIRCLE, LINK] {
        if !restored
            .iter()
            .any(|a| a.body.contains(key) && (a.rect[3] - top).abs() < 0.5)
        {
            return Ok(Some(format!(
                "after ONE Undo of the cut, page 1 of {} lists no `{key}` with its top at {top}. The cut was not one undo step.",
                undone.display()
            )));
        }
    }
    Ok(None)
}

/// Whether any listed annotation has its top edge at `top`, page space.
fn at_top(listed: &[SavedAnnot], top: f64) -> bool {
    listed.iter().any(|a| (a.rect[3] - top).abs() < 0.5)
}

/// The failure sentence unless Ctrl+D traced a duplicate of `kind` carrying
/// one item and pasted one object on page 0.
fn duplicated_one(
    session: &Session,
    pointer: &ScriptedPointer,
    report: &mut CheckReport,
    kind: &str,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    pointer.key(session, None, "D", Some("ctrl"))?;
    session.settle(40);
    let duplicated = step!(after(session, DUPLICATED, mark, "Ctrl+D was pressed"));
    let pasted = step!(after(session, PASTED, mark, "Ctrl+D planned a duplicate"));
    report.note(format!("`{}` then `{}`", duplicated.raw, pasted.raw));
    if duplicated.get("kind") != Some(kind) || duplicated.get("items") != Some("1") {
        return Ok(Some(format!(
            "Ctrl+D traced `{}`: kind={kind} items=1 was wanted.",
            duplicated.raw
        )));
    }
    if pasted.get("pasted") != Some("1") || pasted.get("page") != Some("0") {
        return Ok(Some(format!(
            "Ctrl+D must paste one object on page 0; it traced `{}`.",
            pasted.raw
        )));
    }
    if let Some(copied) = session.trace()?.last_after(COPIED, mark) {
        return Ok(Some(format!(
            "Ctrl+D wrote the clipboard: `{}`. A duplicate must leave it alone.",
            copied.raw
        )));
    }
    Ok(None)
}

/// See the module documentation.
pub struct CtrlDDuplicatesAPageObject;

impl Check for CtrlDDuplicatesAPageObject {
    fn name(&self) -> &'static str {
        "ctrl_d_duplicates_a_page_object"
    }

    fn defect(&self) -> &'static str {
        "Ctrl+D with a shape on the page selected says nothing is selected and adds nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on_with(
            self,
            ctx,
            &BOXES,
            "mode.edit",
            "duplicate-object",
            &[],
            object,
        )
    }
}

fn object(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    click_on(ctx, session, pointer, &BOXES, 0, RED_ONLY)?;
    session.settle(20);
    duplicated_one(session, pointer, report, "selection")
}

/// See the module documentation.
pub struct CtrlDDuplicatesAPartOfAPlacedDrawing;

impl Check for CtrlDDuplicatesAPartOfAPlacedDrawing {
    fn name(&self) -> &'static str {
        "ctrl_d_duplicates_a_part_of_a_placed_drawing"
    }

    fn defect(&self) -> &'static str {
        "Ctrl+D with a part of a placed drawing selected says nothing is selected and adds \
         nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match run_body(ctx, &mut report, FORM_FIXTURE, "duplicate-part", part) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn part(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<Option<String>> {
    enter_leaf_at(ctx, session, pointer, page, ON_THE_LINE)?;
    session.settle(20);
    duplicated_one(session, pointer, report, "form-leaves")
}
