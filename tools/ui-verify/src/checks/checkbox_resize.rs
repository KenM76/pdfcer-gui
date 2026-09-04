//! `a_resized_check_box_is_redrawn_not_stretched` — **drag a check box bigger
//! and its border stays the weight it was.**
//!
//! # The report
//!
//! `OPERATOR_REQUESTS.md` **O76**, 2026-08-31:
//!
//! > *"Form shape outlines of checkboxes and such scale when I drag them
//! > larger."*
//!
//! ## ★★★ The cause was neither of the two the row first guessed
//!
//! The border did not thicken because pdfcer wrote a bigger `/BS /W`. It
//! thickened because **nothing was rewritten at all**: the engine rebuilt a
//! field's appearance for Text and Choice fields only, and a check box is a
//! `/Btn`. So the artwork pdfcer itself had drawn — at the original size, with a
//! hard-coded 1 pt stroke — was kept, and §12.5.5's placement matrix stretched
//! it into the new box. Drag a 12 pt check box to 40 pt and its 1 pt border
//! draws at about 3.3 pt.
//!
//! That was an engine gap, filed rather than worked around, and answered by
//! `pdfcer-core` **Pass 187.0**: a `/Btn` appearance **pdfcer authored** is now
//! redrawn at the new size, and a foreign one refuses by name rather than
//! stretching. The shell's half is passing the operator's three scale answers
//! through `WidgetEdit::with_resize`, which the same Pass made possible.
//!
//! ## ★★ Why the oracle is `regenerated=`, not a pixel
//!
//! **A screenshot cannot tell the two apart.** A border that thickened because
//! `/BS /W` changed and one that thickened because the placement matrix scaled
//! pdfcer's own artwork are *the same pixels*, and so are a redrawn 1 pt border
//! at the new size and a lucky crop. The distinguishing fact is which of three
//! things the engine did, and it says so:
//!
//! ```text
//! edit-widget-applied field=… widget=0 resized=true regenerated=true stale=false
//! ```
//!
//! | field | meaning | the defect's value |
//! |---|---|---|
//! | `resized` | the extent changed | `true` — it always was |
//! | `regenerated` | the appearance was **rebuilt** at the new size | ★ `false` |
//! | `stale` | the engine says the artwork no longer fits | `false` either way |
//!
//! ⇒ `regenerated=false` on a resize IS the operator's complaint, stated
//! exactly. That is the same argument `markup_move` makes for reading `keys=`
//! and `scale_switch` for reading `stroke=`: where the picture is identical,
//! the trace is the only oracle that exists.
//!
//! ★ And note what a weaker check would have passed. `edit-widget-applied`
//! being present at all, or `resized=true`, is true on the broken build —
//! this check must read the third field or it is measuring nothing.
//!
//! ## The sequence
//!
//! | # | step | oracle |
//! |---|---|---|
//! | A | drag out a check box, big enough to have grips | the widget census names it |
//! | B | clear, then click it | `form-field-selected field=…` |
//! | C | drag a corner grip outward | `resize-widget-commit … grip=…` |
//! | D | the engine redrew it | `edit-widget-applied … regenerated=true` |
//!
//! ★★ Step A **drags** rather than clicks, and that is not a stylistic choice.
//! A clicked check box is authored at its default 14 pt, which on this sweep's
//! 1584 pt sheet at fit zoom is four pixels — smaller than one grip's hit
//! square, so there is no corner to aim at and the gesture under test cannot be
//! started. The drag route is the operator's own (*"click to place the position
//! or drag a box for size"*, O53) and it makes the box big enough to have
//! corners.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Edit mode, then arm the check-box tool.
const INVOKE: &str = "mode.edit,edit.form_check_box";
/// Makes the placement dialog accept itself, so no dialog driving is needed.
const ACCEPT_ENV: (&str, &str) = ("PDFCER_DIAG_FORM_ACCEPT", "1");
/// The per-widget census line the canvas publishes.
const BOX_LINE: &str = crate::checks::formaim::TARGET_LINE;
/// The line the canvas writes when a click selects a widget.
const SELECTED: &str = "form-field-selected"; // ui-text-exempt: a trace event name, never displayed
/// The line the resize gesture writes when it commits.
const RESIZE_EVENT: &str = "resize-widget-commit"; // ui-text-exempt: a trace event name, never displayed
/// ★ The line this check exists to read.
const APPLIED: &str = "edit-widget-applied"; // ui-text-exempt: a trace event name, never displayed
/// The page region, so a failure can say whether a sheet was drawn at all.
const PAGE_REGION: &str = "page"; // ui-text-exempt: a trace region name, never displayed

/// The box dragged out for the check box, as page fractions.
///
/// ★ Deliberately far larger than a check box ever is on paper. The subject is
/// the appearance rebuild, not the size, and a box that is generous on screen
/// is one whose eight grips do not overlap each other — see the header.
const DRAG_FROM: (f64, f64) = (0.28, 0.58);
const DRAG_TO: (f64, f64) = (0.40, 0.46);

/// How far the corner grip is pulled, as a fraction of the page.
///
/// Outward on both axes, so every candidate grip in the corner — the corner
/// itself or either neighbouring edge grip — enlarges rather than collapses the
/// box. A drag that produced a rectangle with no area would be refused by the
/// engine, correctly, and this check would report a redraw failure that was
/// really an aim failure.
const PULL: (f64, f64) = (0.10, -0.08);

pub struct AResizedCheckBoxIsRedrawn;

impl Check for AResizedCheckBoxIsRedrawn {
    fn name(&self) -> &'static str {
        "a_resized_check_box_is_redrawn_not_stretched"
    }

    fn defect(&self) -> &'static str {
        "dragging a check box larger stretches the artwork pdfcer itself drew instead of redrawing \
         it, so a 1 pt border draws at 3 pt and the operator sees the outline thicken with the box"
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

#[allow(
    clippy::too_many_lines,
    reason = "one gesture with four oracles, each reading a rectangle the step before it resolved" // ui-text-exempt: a lint justification, never displayed
)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check drags out a field and then drags one of \
             its grips; both are real pointer gestures.",
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
                "cannot read a page size from {}, and this check works in page fractions. Pass \
                 --page-size WxH.",
                pdf.display()
            ))
        })?,
    };
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("checkbox-resize.trace.txt"));
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
        "launched {} as pid {} with the check-box tool armed",
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

    // --- A: drag out a check box big enough to have grips ------------------
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let frame = session.frame()?;
    let from = frame.to_screen(mapping.doc_to_window(DocPoint::new(
        0,
        DRAG_FROM.0 * page.width_pt,
        DRAG_FROM.1 * page.height_pt,
    ))?);
    let to = frame.to_screen(mapping.doc_to_window(DocPoint::new(
        0,
        DRAG_TO.0 * page.width_pt,
        DRAG_TO.1 * page.height_pt,
    ))?);
    driver.drag(from, to)?;
    session.settle(40);

    let trace = session.trace()?;
    let Some(widget) = crate::checks::formaim::targets(&trace)
        .into_iter()
        .find(|b| b.page == 0)
    else {
        return Ok(Some(format!(
            "THE CHECK-BOX TOOL PLACED NOTHING: a drag across the page produced no `{BOX_LINE}` \
             line for page 1, so `edit.form_check_box` did not arm or the drag-to-size route is \
             gone. That is the step BEFORE the one under test. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let field = widget.field.clone();
    let centre = widget.centre();
    let width_px = widget.size.0 * f64::from(mapping.zoom);
    let height_px = widget.size.1 * f64::from(mapping.zoom);
    report.note(format!(
        "★ dragged out {field:?}, {:.0}×{:.0} pt — {width_px:.0}×{height_px:.0} px on screen",
        widget.size.0, widget.size.1
    ));
    if width_px < 40.0 || height_px < 40.0 {
        return Err(Error::new(format!(
            "the field is {width_px:.0}×{height_px:.0} px on screen and its eight grips are 8 px \
             squares with 2 px of slack, so a corner grip cannot be aimed at without touching its \
             neighbours. SKIPPED rather than failed: this is the harness unable to start the \
             gesture, not the application refusing it."
        )));
    }

    // --- B: disarm, clear, then select it ----------------------------------
    //
    // ★ Escape first: the tool stays armed after a placement, so the next press
    // would author a SECOND field. Then a click on blank paper, because
    // authoring already selected this one and `select_click` traces only a
    // CHANGE — the finding `checks::formaim`'s header carries.
    driver.press(crate::sys::vk::ESCAPE)?;
    session.settle(12);

    let boxes = crate::checks::formaim::targets(&session.trace()?);
    let blank =
        crate::checks::formaim::blank_canvas_point(&boxes, page, 0, centre).ok_or_else(|| {
            Error::new(
                "no blank paper could be found near the field this check placed, so the \
                 selection cannot be cleared and the selecting click below would change nothing \
                 to observe."
                    .to_owned(),
            )
        })?;
    driver.click_at(
        session
            .frame()?
            .to_screen(mapping.doc_to_window(DocPoint::new(
                0,
                blank.0,
                page.height_pt - blank.1,
            ))?),
    )?;
    session.settle(20);

    let centre_screen = session
        .frame()?
        .to_screen(mapping.doc_to_window(DocPoint::new(0, centre.0, page.height_pt - centre.1))?);
    driver.click_at(centre_screen)?;
    session.settle(25);

    let trace = session.trace()?;
    if !trace
        .events(SELECTED)
        .any(|l| l.get("field").is_some_and(|f| f == field))
    {
        return Ok(Some(format!(
            "THE FIELD COULD NOT BE SELECTED: a click at its centre produced no `{SELECTED}` \
             line naming {field:?}, so there are no grips on screen and the resize under test \
             cannot be started. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!("★★ selected {field:?} — its grips are on screen"));

    // --- C: drag a corner grip outward -------------------------------------
    //
    // The bottom-right corner of the widget in canvas space, pulled away from
    // the box on both axes.
    let corner = (widget.min.0 + widget.size.0, widget.min.1 + widget.size.1);
    let grip_screen = session
        .frame()?
        .to_screen(mapping.doc_to_window(DocPoint::new(0, corner.0, page.height_pt - corner.1))?);
    let pulled = session
        .frame()?
        .to_screen(mapping.doc_to_window(DocPoint::new(
            0,
            corner.0 + PULL.0 * page.width_pt,
            page.height_pt - (corner.1 - PULL.1 * page.height_pt),
        ))?);
    driver.drag(grip_screen, pulled)?;
    session.settle(45);

    let trace = session.trace()?;
    if trace.events(RESIZE_EVENT).count() == 0 {
        return Ok(Some(format!(
            "THE GRIP DRAG WAS NOT A RESIZE: no `{RESIZE_EVENT}` line after pressing the box's \
             bottom-right corner and dragging outward. The press landed on the body, or on \
             nothing, so `meaning::press_kind` never returned a `Resize`. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- D: ★★★ and the engine REDREW it ------------------------------------
    let Some(applied) = trace.last(APPLIED) else {
        return Ok(Some(format!(
            "THE RESIZE NEVER REACHED THE ENGINE: `{RESIZE_EVENT}` was traced and no `{APPLIED}` \
             line followed, so `FieldAction::EditWidget` was raised and the apply arm refused it \
             or dropped it. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let resized = applied.get("resized") == Some("true");
    let regenerated = applied.get("regenerated") == Some("true");
    if !resized {
        return Ok(Some(format!(
            "THE ENGINE DID NOT SEE A RESIZE: `{}`. The rectangle sent had the same extent as the \
             one it replaced, so the grip drag committed a move. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    if !regenerated {
        return Ok(Some(format!(
            "★★★ THE CHECK BOX WAS STRETCHED, NOT REDRAWN: `{}`.\n\
             `regenerated=false` on a resize IS the operator's complaint, stated exactly — the \
             appearance stream pdfcer drew at the OLD size is kept and §12.5.5's placement matrix \
             scales it into the new box, so a 1 pt border draws thicker in proportion to how far \
             the box was dragged. Note that a screenshot cannot see this: a scaled 1 pt border \
             and a redrawn 3 pt one are the same pixels, which is why this check reads the trace.\n\
             Two halves have to hold. `pdfcer-core` Pass 187.0 redraws a `/Btn` appearance pdfcer \
             authored (a FOREIGN one refuses by name instead, which would show as \
             `stale=true`), and this shell must pass the operator's scale answers through \
             `WidgetEdit::with_resize` — from `canvas::resizing` for the drag and from \
             `panels::properties::widgetedit` for the typed route. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ the engine redrew the appearance at the new size: `{}`",
        applied.raw
    ));
    Ok(None)
}
