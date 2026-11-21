//! `field_carets` — **the in-place field editor sets the typed text where the
//! field's own appearance will draw it — comb cells, `/Q` right, multiline
//! wrap at the `/DA` size — and a click places the caret where it lands**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/field_carets.md`.

use super::dimdrive::launch_path;
use super::print_shop::await_line;
use crate::checks::driving;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::{LRect, PixRect, Pt};
use crate::image::Image;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.review"; // ui-text-exempt: a command id
const FIXTURE: &str = "field-carets.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/field-carets.PROVENANCE.py`.";
const FOCUS: &str = "form-focus"; // ui-text-exempt: a trace event name
const FILL: &str = "form-fill-text"; // ui-text-exempt: a trace event name

/// One field of the fixture: name, `/Rect` (PDF user space) and what is typed.
struct Field {
    name: &'static str,
    rect: (f64, f64, f64, f64),
    typed: &'static str,
}

const FIELDS: [Field; 3] = [
    Field {
        name: "Comb",
        rect: (72.0, 600.0, 372.0, 640.0),
        typed: "ABCD",
    },
    Field {
        name: "Right",
        rect: (72.0, 520.0, 372.0, 560.0),
        typed: "Hello",
    },
    Field {
        name: "Para",
        rect: (72.0, 360.0, 232.0, 480.0),
        typed: "The quick brown fox jumps over the lazy dog",
    },
];

/// The bounding box of the dark pixels in a region, in capture pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Ink {
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
}

/// See the module documentation.
pub struct AFieldEditorSetsTextWhereTheFieldDrawsIt;

impl Check for AFieldEditorSetsTextWhereTheFieldDrawsIt {
    fn name(&self) -> &'static str {
        "a_field_editor_sets_text_where_the_field_draws_it"
    }

    fn defect(&self) -> &'static str {
        "while typing into a form field the text, caret and selection sit where the editor's own \
         font puts them rather than where the field's font, size, comb cells, alignment and wrap \
         draw it once committed, so the text jumps on commit and a click places the caret on the \
         wrong character"
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
    let doc = ctx.out("field_carets.pdf");
    let source = driving::repo_fixture(FIXTURE, METHOD)?;
    std::fs::copy(&source, &doc)
        .map_err(|e| Error::new(format!("copying {}: {e}", source.display())))?;
    let page = crate::fixture::page_geometry(&doc)
        .ok_or_else(|| Error::new(format!("no page size in {}.", source.display())))?;
    let (session, pointer) = launch_path(ctx, report, doc, INVOKE, "field_carets", &[])?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(45);
    let drives = (&session, &pointer);
    let mut outcome = Ok(None);
    for field in &FIELDS {
        outcome = type_and_commit(ctx, report, drives, page, field);
        if !matches!(outcome, Ok(None)) {
            break;
        }
    }
    if matches!(outcome, Ok(None)) {
        outcome = click_places(ctx, report, drives, page);
    }
    let parked = pointer.gone(&session);
    match outcome? {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// Click a field, type into it, picture it, commit, picture it again: the
/// ink must sit in the same place in both pictures.
fn type_and_commit(
    ctx: &CheckContext,
    report: &mut CheckReport,
    (session, pointer): (&Session, &ScriptedPointer),
    page: PageGeometry,
    field: &Field,
) -> Result<Option<String>> {
    let (x1, y0) = (field.rect.2, field.rect.1);
    if let Some(failure) = open(ctx, (session, pointer), page, field, (x1 - 4.0, y0 + 4.0))? {
        return Ok(Some(failure));
    }
    pointer.type_text(session, None, field.typed)?;
    session.settle(25);
    let region = pixels(ctx, session, page, field.rect)?;
    let editing = picture(ctx, session, pointer, &format!("{}-editing", field.name))?;
    let mark = session.trace()?.mark();
    leave(ctx, (session, pointer), page)?;
    if await_line(session, FILL, mark, |_| true)?.is_none() {
        return Ok(Some(format!(
            "★★★ {}: leaving the editor committed nothing (no `{FILL}` line).",
            field.name
        )));
    }
    session.settle(45);
    let saved = picture(ctx, session, pointer, &format!("{}-saved", field.name))?;
    let (Some(a), Some(b)) = (ink(&editing, region), ink(&saved, region)) else {
        return Ok(Some(format!(
            "★★★ {}: no ink in the box while editing or after the commit.",
            field.name
        )));
    };
    report.note(format!("{} editing {a:?} saved {b:?}", field.name));
    let slack = (f64::from(region.h).min(f64::from(region.w)) * 0.06).max(4.0) as u32;
    let off = |p: u32, q: u32| p.abs_diff(q) > slack;
    if off(a.x0, b.x0) || off(a.x1, b.x1) || off(a.y0, b.y0) || off(a.y1, b.y1) {
        return Ok(Some(format!(
            "★★★ {}: the typed text sat at {a:?} while editing and is drawn at {b:?} once \
             committed (capture pixels, slack {slack}). The editor is not setting it in the \
             field's own layout.",
            field.name
        )));
    }
    Ok(None)
}

/// Click into the left half of the comb's first cell and type a hyphen: the
/// caret must have gone before `A`, so cell 1 holds the short hyphen and
/// cell 2 the tall `A` — not `ABCD-`, which is the caret left at the end.
fn click_places(
    ctx: &CheckContext,
    report: &mut CheckReport,
    (session, pointer): (&Session, &ScriptedPointer),
    page: PageGeometry,
) -> Result<Option<String>> {
    let comb = &FIELDS[0];
    let (x0, y0, _, y1) = comb.rect;
    let at = (x0 + 6.0, f64::midpoint(y0, y1));
    if let Some(failure) = open(ctx, (session, pointer), page, comb, at)? {
        return Ok(Some(failure));
    }
    pointer.type_text(session, None, "-")?;
    session.settle(10);
    let mark = session.trace()?.mark();
    leave(ctx, (session, pointer), page)?;
    if await_line(session, FILL, mark, |_| true)?.is_none() {
        return Ok(Some(
            "★★★ Comb: the second edit committed nothing.".to_owned(),
        ));
    }
    session.settle(45);
    let saved = picture(ctx, session, pointer, "Comb-clicked")?;
    let cell = |i: f64| (x0 + 50.0 * i + 2.0, y0, x0 + 50.0 * (i + 1.0) - 2.0, y1);
    let one = ink(&saved, pixels(ctx, session, page, cell(0.0))?);
    let two = ink(&saved, pixels(ctx, session, page, cell(1.0))?);
    report.note(format!(
        "Comb after the click: cell 1 {one:?}, cell 2 {two:?}"
    ));
    let (Some(one), Some(two)) = (one, two) else {
        return Ok(Some(format!(
            "★★★ Comb: after typing `-` at the first cell, cell 1 or 2 is empty ({one:?}, \
             {two:?})."
        )));
    };
    if (one.y1 - one.y0) * 2 > two.y1 - two.y0 {
        return Ok(Some(format!(
            "★★★ Comb: a click in the first cell and a `-` left a tall glyph there ({one:?} \
             against cell 2's {two:?}): the caret was not placed where the click landed, so \
             the hyphen went to the end."
        )));
    }
    Ok(None)
}

/// Click at `at` (PDF space) in `field` and wait for its editor.
fn open(
    ctx: &CheckContext,
    (session, pointer): (&Session, &ScriptedPointer),
    page: PageGeometry,
    field: &Field,
    at: (f64, f64),
) -> Result<Option<String>> {
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let mark = session.trace()?.mark();
    pointer.click(
        session,
        mapping.doc_to_window(DocPoint::new(0, at.0, at.1))?,
    )?;
    let seen = await_line(session, FOCUS, mark, |l| l.get("field") == Some(field.name))?;
    if seen.is_none() {
        return Ok(Some(format!(
            "★★★ {}: the click opened no editor (no `{FOCUS}` line).",
            field.name
        )));
    }
    session.settle(15);
    Ok(None)
}

/// Click empty page, which commits the open editor.
fn leave(
    ctx: &CheckContext,
    (session, pointer): (&Session, &ScriptedPointer),
    page: PageGeometry,
) -> Result<()> {
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    pointer.click(
        session,
        mapping.doc_to_window(DocPoint::new(0, 480.0, 250.0))?,
    )?;
    Ok(())
}

fn picture(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    stem: &str,
) -> Result<Image> {
    let png = ctx.out(&format!("field-carets-{stem}.png"));
    pointer.screenshot(session, &png)?;
    Image::load_png(&png)
}

/// A PDF-space rectangle, less 1.5 pt, in capture pixels.
fn pixels(
    ctx: &CheckContext,
    session: &Session,
    page: PageGeometry,
    (x0, y0, x1, y1): (f64, f64, f64, f64),
) -> Result<PixRect> {
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let a = mapping.doc_to_window(DocPoint::new(0, x0 + 1.5, y0 + 1.5))?;
    let z = mapping.doc_to_window(DocPoint::new(0, x1 - 1.5, y1 - 1.5))?;
    let rect = LRect::new(
        Pt::new(a.x().min(z.x()), a.y().min(z.y())),
        Pt::new(a.x().max(z.x()), a.y().max(z.y())),
    );
    Ok(session.frame()?.logical_to_capture_pixels(rect))
}

/// The bounding box of the pixels darker than mid-grey in `px`.
fn ink(image: &Image, px: PixRect) -> Option<Ink> {
    let mut found: Option<Ink> = None;
    for y in px.y..px.y + px.h {
        for x in px.x..px.x + px.w {
            let Some(p) = image.pixel(x, y) else { continue };
            let luma = (u32::from(p.r) * 3 + u32::from(p.g) * 6 + u32::from(p.b)) / 10;
            if luma >= 110 {
                continue;
            }
            found = Some(match found {
                None => Ink {
                    x0: x,
                    y0: y,
                    x1: x,
                    y1: y,
                },
                Some(i) => Ink {
                    x0: i.x0.min(x),
                    y0: i.y0.min(y),
                    x1: i.x1.max(x),
                    y1: i.y1.max(y),
                },
            });
        }
    }
    found
}
