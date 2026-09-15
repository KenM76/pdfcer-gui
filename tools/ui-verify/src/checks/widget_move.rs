//! `dragging_a_form_field_moves_it` — **a form field's box follows the
//! pointer.**
//!
//! # What this is for
//!
//! The operator's standing instruction that week was *"work on form field
//! editing next and the rest of the features required for editing."* A field
//! could be placed, selected, renamed, deleted, and its position and size typed
//! into four boxes with an Apply button — and **dragging it did nothing.**
//!
//! ★★ Four numbers and an Apply button are a form for editing a rectangle.
//! **Dragging is how a person moves a box**, and every program in this class
//! does it; the typed fields are the precise route, not the primary one.
//!
//! ## ★★★ Why the defect was found rather than reported
//!
//! Ten days earlier the same shape was found on the annotation surface: the
//! canvas forked on *"is an annotation selected?"*, the only module on that
//! branch answered for ce dimensions, and a drag on a stamp was **consumed and
//! discarded** — no move, no decline, nothing.
//!
//! This was the identical state one surface along, and it was found by asking
//! *"where else does this shape exist?"* ⇒ **A class of defect that has been
//! named once is cheap to look for; the same class waiting for an operator to
//! trip over it is not.**
//!
//! ★ The routing here was worse in one way: a widget is deliberately **not** an
//! annotation selection — `canvas::selection::annot` excludes `/Widget` so the
//! form surface owns the press — so a selected field did not even reach the
//! annotation branch. It fell into the CONTENT branch, where the mover found no
//! content, and was dropped there.
//!
//! ## ★★ The oracle is the engine's own `move-widget-applied`, with BOTH deltas
//!
//! `dy` is the one with a sign convention to lose: PDF user space increases
//! **upward** (§8.3.2.3) and every screen coordinate in this harness increases
//! downward, so a term dropped in the conversion shows up as a move that
//! travels in x only. The drag is diagonal by construction and both axes are
//! asserted.
//!
//! ## ★ Why it authors its own field rather than needing a form fixture
//!
//! `edit.form_text_field` places one, and `PDFCER_DIAG_FORM_ACCEPT=1` makes the
//! placement dialog press its own Add — the same seam `form_field` uses. So this
//! runs on any `--pdf` with a page, which is what keeps it in the ordinary
//! sweep rather than behind a fixture nobody remembers to pass.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Edit mode, then arm the text-field tool.
const INVOKE: &str = "mode.edit,edit.form_text_field";
/// Makes the placement dialog accept itself, so no dialog driving is needed.
const ACCEPT_ENV: (&str, &str) = ("PDFCER_DIAG_FORM_ACCEPT", "1");
/// The per-widget census line the canvas publishes, carrying each box's rect.
///
/// ★ Parsed by `checks::formaim::targets` rather than by a copy in this file.
/// Three checks read this census, and the third copy is where copies start to
/// disagree; the shared module's header carries the finding that made one worth
/// having.
const BOX_LINE: &str = crate::checks::formaim::TARGET_LINE;
/// The line the canvas writes when a click selects a widget.
const SELECTED: &str = "form-field-selected";
/// The line the drag writes when it decides to commit.
const DRAG_EVENT: &str = "widget-drag";
/// The line a RESIZE writes when it commits, so a press that landed on a grip
/// can be reported as what it was rather than as a dropped gesture. See phase C.
const RESIZE_EVENT: &str = "resize-widget-commit";
/// The line the apply arm writes when the engine has moved it.
///
/// ★ `-applied`, per the convention this project adopted after making the
/// same-name mistake twice: `vector_edit` writes its own `move-widget …` line
/// for the identical edit and `.last()` on the bare name reads that one.
const MOVED: &str = "move-widget-applied";
/// The page region, so a failure can say whether a sheet was drawn at all.
const PAGE_REGION: &str = "page";

/// Where the field is placed and where it is dragged to, as page fractions.
///
/// ★ Both well inside the sheet: the first so the placement lands on paper, the
/// second so the move has somewhere to go. Diagonal, for the `dy` reason in the
/// module header.
const PLACE_AT: (f64, f64) = (0.30, 0.55);
const MOVE_TO: (f64, f64) = (0.50, 0.35);

/// See the module documentation.
pub struct DraggingAFormFieldMovesIt;

impl Check for DraggingAFormFieldMovesIt {
    fn name(&self) -> &'static str {
        "dragging_a_form_field_moves_it"
    }

    fn defect(&self) -> &'static str {
        "a form field's box can be placed, selected, renamed and deleted and cannot be DRAGGED — \
         the press is consumed by the canvas's content branch, which finds no content under it \
         and drops the gesture, so the operator drags a field across the sheet and it stays where \
         it was with no message anywhere"
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
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check places a field with a click, selects it \
             with another, and then drags it — the drag being the gesture under test.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. This check needs a page to place a field on."))?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. This check places its field in page fractions \
                 and needs the box to turn them into points. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("widget-move.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env
        .push((ACCEPT_ENV.0.to_owned(), ACCEPT_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with PDFCER_DIAG_INVOKE={INVOKE}",
        exe.display(),
        session.pid()
    ));
    session.settle(45);
    let driver = Driver::new(session.window());

    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "the application declared no `{PAGE_REGION}` region, so no sheet is on screen. \
             Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // --- A: place a field ---------------------------------------------------
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let frame = session.frame()?;
    let at = DocPoint::new(0, PLACE_AT.0 * page.width_pt, PLACE_AT.1 * page.height_pt);
    driver.click_at(frame.to_screen(mapping.doc_to_window(at)?))?;
    session.settle(35);

    let trace = session.trace()?;
    let placed = crate::checks::formaim::targets(&trace);
    let Some(widget) = placed.iter().find(|b| b.page == 0).cloned() else {
        return Ok(Some(format!(
            "THE TEXT-FIELD TOOL PLACED NOTHING: a click on the page produced no `{BOX_LINE}` \
             line for page 1, so `edit.form_text_field` did not arm or the placement dialog did \
             not accept. This is the step BEFORE the one under test — there is no field to drag. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let field = widget.field.clone();
    let centre = widget.centre();
    report.note(format!(
        "★ placed the field {field:?} at canvas ({:.1}, {:.1})",
        centre.0, centre.1
    ));

    // --- B: disarm, CLEAR, then select it ----------------------------------
    //
    // ★ Escape first. The tool stays armed after a placement, exactly as a
    // markup pen does, so a second click without this would place a SECOND
    // field rather than select one — and the check would fail with a message
    // about selection when the cause was arming. `form_field` records the same
    // step for the same reason.
    driver.press(crate::sys::vk::ESCAPE)?;
    session.settle(12);

    // ★★★ AND THEN A CLICK ON BLANK PAPER, WHICH IS NOT SCENERY — see
    // `checks::formaim`'s header for the trace lines that put it here.
    //
    // The field this check just authored is ALREADY SELECTED: the authoring arm
    // selects what it places (`OPERATOR_REQUESTS.md` O53), and
    // `canvas::forms::select_click` raises its action — and writes its trace
    // line — only when the selection CHANGES. So the old shape of this phase
    // clicked a field that was already selected and then required the program
    // to announce a selection that had not moved. It failed for a whole sweep
    // with a sentence about a click that had in fact landed dead centre.
    //
    // ⇒ Clearing first is what makes the assertion below answerable, and it
    // adds a second observation rather than working around the first: a primary
    // click on blank paper CLEARS (that is `select_click`'s own table), so the
    // clearing line proves the selection channel is live before its silence is
    // read as a failure. `crate::checks` rule 4.
    let boxes = crate::checks::formaim::targets(&trace);
    let blank =
        crate::checks::formaim::blank_canvas_point(&boxes, page, 0, centre).ok_or_else(|| {
            Error::new(format!(
                "no blank paper could be found on page 1 near the field this check placed: every \
                 candidate around ({:.1}, {:.1}) is inside one of the {} widget(s) the canvas \
                 named, or off the sheet. Without a clearing click the field stays selected from \
                 authoring and the selecting click below changes nothing to observe. Reported as \
                 a SKIP: this is a property of the document, not of the drag under test.",
                centre.0,
                centre.1,
                boxes.len()
            ))
        })?;
    let blank_point = mapping.doc_to_window(DocPoint::new(0, blank.0, page.height_pt - blank.1))?;
    driver.click_at(session.frame()?.to_screen(blank_point))?;
    session.settle(20);

    let trace = session.trace()?;
    // ★ `field=` absent IS the cleared line: the application writes
    // `form-field-selected none` with no key/value pairs at all for a cleared
    // selection, and `field=…` for every other one.
    if !trace.events(SELECTED).any(|l| l.get("field").is_none()) {
        return Ok(Some(format!(
            "THE SELECTION COULD NOT BE CLEARED: a click on blank paper at canvas ({:.1}, {:.1}) \
             produced no `{SELECTED} none` line. A primary click on paper is an unambiguous \
             deselect and `canvas::forms::select_click` traces it, so this says the click never \
             reached the form surface at all — which would make the assertion below unreadable \
             either way. Trace: {}.",
            blank.0,
            blank.1,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ the selection was cleared on blank paper at canvas ({:.1}, {:.1})",
        blank.0, blank.1
    ));

    // The census is canvas space and `doc_to_window` takes PDF space, so the
    // flip is the one arithmetic this check does: `canvas_y = height - doc_y`.
    let doc_y = page.height_pt - centre.1;
    let field_point = mapping.doc_to_window(DocPoint::new(0, centre.0, doc_y))?;
    let field_screen = session.frame()?.to_screen(field_point);
    driver.click_at(field_screen)?;
    session.settle(25);

    let trace = session.trace()?;
    if !trace
        .events(SELECTED)
        .any(|l| l.get("field").is_some_and(|f| f == field))
    {
        return Ok(Some(format!(
            "THE FIELD COULD NOT BE SELECTED: a click at its centre produced no `{SELECTED}` \
             line naming {field:?}, on a frame where the clearing click above proved the \
             selection channel is live. So this says the click missed the widget's rect or the \
             tool was still armed. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!("★ selected {field:?}"));

    // --- C: drag it ---------------------------------------------------------
    //
    // ★★★ THE PRESS IS AT A QUARTER OF THE BOX'S WIDTH, NOT AT ITS CENTRE, AND
    // THAT IS THE ONE PLACE ON A SHORT BOX WHERE THE CENTRE IS THE WORST
    // CHOICE.
    //
    // Eight resize grips are hit-tested on any selected widget, anchored at the
    // four corners and the four edge MIDPOINTS, each an 8 pt square with 2 pt of
    // slack (`canvas::handles`). A text field is authored 160 × 20 pt, and this
    // sweep's document is a 1584 × 1224 pt CAD sheet that opens at fit — zoom
    // 0.2955 — so the box on screen is **47.3 × 5.9 px**. Every grip's 12 px hit
    // square therefore spans the box's whole height, and the North grip sits at
    // the centre of the top edge: **dead centre of the box is inside the North
    // grip**, on every zoom below about 60 %.
    //
    // That is what the 2026-08-29 re-run found once the selection was fixed —
    // the drag committed `resize-widget-commit … grip=North sy=-42.5314` and the
    // engine correctly refused the result, *"the new field's rectangle has no
    // area"*. A drag that becomes a resize is not the defect this check names,
    // and reporting it as one would send a reader to `dragroute`.
    //
    // ⇒ The grip anchors on the x axis are at 0, w/2 and w, so the points
    // furthest from all of them are w/4 and 3w/4 — that is a property of a
    // handle layout every direct-manipulation surface has, not this build's
    // pixel arithmetic, which is why it is safe for a harness to rely on. The
    // vertical position stays at the centre; on a short box every y is equally
    // near an edge and on a tall one the centre is the safest.
    //
    // ★ `markup_move` presses ITS shape's centre and is right to: it draws a
    // 0.20 × 0.15 page-fraction rectangle, 154 × 89 px here, whose centre is
    // dozens of pixels from every grip. The difference is the size of the
    // object, not the gesture.
    let body = (widget.min.0 + widget.size.0 / 4.0, centre.1);
    let body_px = widget.size.0 * f64::from(mapping.zoom);
    if body_px < 32.0 {
        return Err(Error::new(format!(
            "the field is {body_px:.1} px wide on screen at zoom {:.4}, and a quarter of that is \
             inside the corner grips. There is no point on this box that presses its BODY rather \
             than a resize grip, so the drag under test cannot be started. Reported as a SKIP: \
             open the document at a zoom where a 160 pt box is at least 32 px wide, or pass a \
             smaller page. It is also a finding about the product — a field this small on screen \
             is one an operator cannot drag either.",
            mapping.zoom
        )));
    }
    let body_screen = session
        .frame()?
        .to_screen(mapping.doc_to_window(DocPoint::new(0, body.0, page.height_pt - body.1))?);
    report.note(format!(
        "★ pressing the body at canvas ({:.1}, {:.1}) — a quarter of the box's width in, which \
         is the furthest any point of a {:.1} px-wide box gets from a grip anchor",
        body.0, body.1, body_px
    ));

    let landing = mapping.doc_to_window(DocPoint::new(
        0,
        MOVE_TO.0 * page.width_pt,
        MOVE_TO.1 * page.height_pt,
    ))?;
    driver.drag(body_screen, session.frame()?.to_screen(landing))?;
    session.settle(40);

    let trace = session.trace()?;
    let Some(dragged) = trace.events(DRAG_EVENT).last() else {
        // ★ If the press became a RESIZE, say so instead of telling the reader
        // to go and look at the router. That is what happened on the run that
        // moved the press point off the centre, and a check that cannot tell
        // "the gesture was dropped" from "the gesture went to another verb"
        // sends every future reader to the wrong file.
        if let Some(resized) = trace.events(RESIZE_EVENT).last() {
            return Ok(Some(format!(
                "★★ THE PRESS BECAME A RESIZE, NOT A MOVE: `{}` and no `{DRAG_EVENT}` line.\n\
                 The pointer went down inside one of the eight resize grips rather than on the \
                 box's body, so `meaning::press_kind` returned a `Resize` before it ever reached \
                 its `widget_body` arm. That is an AIM problem in this check or a grip layout \
                 that leaves a selected field with no body to press — `canvas::handles` drops a \
                 mid-edge grip below `MIN_MID_GRIP_EXTENT_PX` of extent for exactly that reason, \
                 and a box shorter than a grip's hit square has the same problem on its other \
                 axis. Trace: {}.",
                resized.raw,
                session.trace_path().display()
            )));
        }
        return Ok(Some(format!(
            "★★★ THE DRAG ON A SELECTED FORM FIELD RAISED NOTHING: no `{DRAG_EVENT}` line, and \
             no `{RESIZE_EVENT}` line either, so the press reached no verb at all.\n\
             **This is the exact state the feature shipped in.** A widget is deliberately not an \
             annotation selection — `canvas::selection::annot` excludes `/Widget` so the form \
             surface owns the press — so a selected field does not reach the annotation branch \
             of `canvas::dragroute` at all. Without its own arm there it falls into the CONTENT \
             branch, where the mover finds nothing under the pointer and the gesture is dropped. \
             Check `dragroute`'s `doc.selected_field` arm and `meaning::press_kind`'s \
             `widget_body` branch, which is what turns the press into a `DragKind::Move` to \
             route. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★★ the drag decided to move it: `{}`", dragged.raw));

    let Some(moved) = trace.events(MOVED).last() else {
        return Ok(Some(format!(
            "★★ THE MOVE WAS RAISED AND NOTHING REACHED THE DOCUMENT: `{}` and no `{MOVED}` \
             line.\n\
             The action was raised and its apply arm never ran, or `move_widget` refused — it \
             refuses a widget with no `/Rect` and an index out of range, both by name, and a \
             refused `vector_edit` traces `move-widget-refused`. Trace: {}.",
            dragged.raw,
            session.trace_path().display()
        )));
    };
    report.note(format!("★★★ the engine moved it: `{}`", moved.raw));

    // --- the oracle: it travelled, in both axes -----------------------------
    let dx: f64 = moved.get("dx").and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let dy: f64 = moved.get("dy").and_then(|v| v.parse().ok()).unwrap_or(0.0);
    if dx.abs() < 1.0 || dy.abs() < 1.0 {
        return Ok(Some(format!(
            "★ THE MOVE TRAVELLED IN ONLY ONE AXIS: `{}` reports dx={dx:.3} dy={dy:.3}, and the \
             drag was diagonal by construction.\n\
             A zero on one axis means the canvas→page conversion dropped it, and `dy` is the one \
             to suspect: PDF user space increases **upward** (§8.3.2.3) while every screen \
             coordinate here increases downward, so a sign or a term lost in \
             `moving::page_delta` shows up here and nowhere else. Trace: {}.",
            moved.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ moved dx={dx:.2} pt, dy={dy:.2} pt — both axes travelled; other boxes of this field \
         left standing: {}",
        moved.get("siblings").unwrap_or("?")
    ));

    // ★ `siblings` is REPORTED and never asserted. It is zero for the
    // single-widget field this check authors, and a non-zero value is a
    // disclosure about a field drawn on several pages — a fact about the
    // document, not about the move.
    Ok(None)
}
