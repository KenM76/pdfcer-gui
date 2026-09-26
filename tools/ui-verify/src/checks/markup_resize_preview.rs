//! # `markup_resize_preview` — dragging a comment's corner shows where it is going
//!
//!
//! > *"the Markup Items don't have a live preview — the bounding box stays the
//! > same size when I drag the handles — the items that I can resize do
//! > resize."*
//!
//! Design and rationale: `docs/modules/ui-verify/checks/markup_resize_preview.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The commands the run needs armed at launch.
const INVOKE: &str = "mode.review,markup.rectangle";

/// The region that means *a document is on screen*.
const PAGE_REGION: &str = "page";

/// The region the ghost publishes, one line per outline per frame.
const GHOST_REGION: &str = "canvas-resize-ghost";

const COMMIT_EVENT: &str = "markup-commit";
const SELECT_EVENT: &str = "annot-select";
const RESIZE_EVENT: &str = "resize-annotation";

/// Where the shape is drawn, as fractions of the page.
const SHAPE: ((f64, f64), (f64, f64)) = ((0.35, 0.35), (0.55, 0.50));

/// Where the corner is dragged to — **outward in both axes**, so the ghost has
/// to grow rather than merely move.
const DRAG_TO: (f64, f64) = (0.72, 0.66);

/// See the module documentation.
pub struct DraggingACommentsCornerShowsWhereItIsGoing;

impl Check for DraggingACommentsCornerShowsWhereItIsGoing {
    fn name(&self) -> &'static str {
        "dragging_a_comments_corner_shows_where_it_is_going"
    }

    fn defect(&self) -> &'static str {
        "a markup annotation's resize draws NO live preview — `draw_resize_ghost` reads the \
         page-content outline list, which is empty for an annotation selection, so the operator \
         drags a corner and the box on screen does not move. The resize commits; only the \
         picture of it is missing, which from their chair is indistinguishable from a resize \
         that does not work"
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

/// Parse `rect=[[x0 y0] - [x1 y1]]` into its width and height.
fn extent(raw: &str) -> Option<(f64, f64)> {
    let body = raw.split("rect=").nth(1)?;
    let nums: Vec<f64> = body
        .replace(['[', ']', '-', ','], " ")
        .split_whitespace()
        .filter_map(|t| t.parse::<f64>().ok())
        .collect();
    let [x0, y0, x1, y1] = nums.get(..4)? else {
        return None;
    };
    Some(((x1 - x0).abs(), (y1 - y0).abs()))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check draws a shape with a drag, clicks it, \
             and drags a corner grip. The subject is what is painted WHILE the button is held, \
             so there is no version of it that does not need a real pointer.",
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
        .ok_or_else(|| Error::new("no --pdf. This check needs a page to draw a shape on."))?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "could not read a page size from {}, and this check places its shape in page \
                 fractions. Pass --page-size.",
                pdf.display()
            ))
        })?,
    };
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("markup-resize-preview.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with PDFCER_DIAG_INVOKE={INVOKE}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "the application declared no `{PAGE_REGION}` region, so no sheet is on screen and \
             there is nothing to draw on. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // --- 1: draw a rectangle ------------------------------------------------
    let corner = |f: (f64, f64)| DocPoint::new(0, f.0 * page.width_pt, f.1 * page.height_pt);
    let from = aim(ctx, &session, page, corner(SHAPE.0))?;
    let to = aim(ctx, &session, page, corner(SHAPE.1))?;
    driver.drag(from, to)?;
    session.settle(30);

    let trace = session.trace()?;
    if trace.events(COMMIT_EVENT).last().is_none() {
        return Ok(Some(format!(
            "THE RECTANGLE TOOL AUTHORED NOTHING: no `{COMMIT_EVENT}` line. This is two steps \
             before the subject — there is no annotation to select, let alone resize. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- 2: put the tool down and select it ---------------------------------
    //
    // `markup_move`'s hard-won step: with a markup tool armed a click on the
    // page is a PICK, so a check that skips this draws a SECOND rectangle and
    // then reports that selection is broken.
    let centre = corner((
        f64::midpoint(SHAPE.0.0, SHAPE.1.0),
        f64::midpoint(SHAPE.0.1, SHAPE.1.1),
    ));
    let centre_screen = aim(ctx, &session, page, centre)?;
    if !crate::checks::driving::arm_select_from_ribbon(&session, &driver, ui_rect, report)? {
        driver.press(crate::sys::vk::V)?;
        session.settle(12);
    }
    driver.click_at(centre_screen)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(selected) = trace.events(SELECT_EVENT).last() else {
        return Ok(Some(format!(
            "THE SHAPE COULD NOT BE SELECTED: no `{SELECT_EVENT}` line after a click at its \
             centre. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the shape was selected: `{}`", selected.raw));
    if selected.get("locked") == Some("true") {
        return Err(Error::new(
            "the annotation reports itself LOCKED (§12.5.3 bit 8), so it is offered no grips at \
             all and there is no resize to preview. SKIPPED rather than failed — that is \
             correct behaviour and it means this fixture cannot exercise the subject.",
        ));
    }
    let before = extent(&selected.raw);

    // --- 3: drag the CORNER, not the body -----------------------------------
    //
    let grip = aim(ctx, &session, page, corner(SHAPE.1))?;
    let landing = aim(ctx, &session, page, corner(DRAG_TO))?;
    driver.drag(grip, landing)?;
    session.settle(40);

    let trace = session.trace()?;
    if trace.events(RESIZE_EVENT).count() == 0 {
        return Ok(Some(format!(
            "★★ THE CORNER DRAG RAISED NO RESIZE: no `{RESIZE_EVENT}` line, so the press landed \
             on the body (a MOVE) or on nothing rather than on a grip. The grips are laid out on \
             the selection outline and this aimed at its corner; if the outline is not where the \
             shape is, `SelectionState::resolve_annot` is the suspect. **This is not the defect \
             under test** — the subject is the PICTURE during that drag, and there was no drag \
             to picture. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- 4: the ghost, and whether it MOVED ---------------------------------
    let ghosts: Vec<(f64, f64)> = trace
        .events(ui_rect)
        .filter(|e| e.get("name") == Some(GHOST_REGION))
        .filter_map(|e| extent(&e.raw))
        .collect();

    if ghosts.is_empty() {
        return Ok(Some(format!(
            "★★★ NO RESIZE PREVIEW WAS DRAWN: the resize reached the engine and not one \
             `{GHOST_REGION}` rect was published, so the operator held a corner and watched \
             nothing happen.\n\
             **This is the exact state the feature shipped in.** `overlay::draw_resize_ghost` \
             iterates `SelectionState::outlines()`, which holds page-CONTENT entries and is \
             empty for an annotation selection; and `canvas::painting`'s guard beside it calls \
             `overlay::grip_box`, which answers `None` for the same reason, so the ghost is not \
             even reached. Its sibling `draw_move_ghost` has had the annotation arm all along — \
             which is why dragging the middle previews and dragging the corner does not.\n\
             Fix: `overlay::ghost_box`, and the annotation arm in `draw_resize_ghost`. Trace: {}.",
            session.trace_path().display()
        )));
    }

    let widest = ghosts.iter().fold(0.0_f64, |a, g| a.max(g.0));
    let narrowest = ghosts.iter().fold(f64::MAX, |a, g| a.min(g.0));
    report.note(format!(
        "★★ {} ghost rect(s) published, widths {narrowest:.1}–{widest:.1}",
        ghosts.len()
    ));

    if (widest - narrowest) < 1.0 {
        return Ok(Some(format!(
            "★★★ THE PREVIEW WAS DRAWN AND NEVER CHANGED SIZE: {} `{GHOST_REGION}` rect(s), all \
             within a pixel of {widest:.1} wide. That is the operator's report in his own \
             words — *\"the bounding box stays the same size when I drag the handles\"* — and a \
             check asserting only that a ghost EXISTS would have passed on it.\n\
             The ghost is drawn from the scale factors `canvas::resizing::drag` returns, so \
             either those are constant (the grip is being read as the wrong one, or `factors` \
             is not seeing the delta) or the ghost is being painted from the selection's own \
             box instead of the scaled one. Trace: {}.",
            ghosts.len(),
            session.trace_path().display()
        )));
    }

    if let Some((w, h)) = before {
        report.note(format!(
            "★ the selection was {w:.1} x {h:.1} before the drag, and the preview grew to \
             {widest:.1} wide"
        ));
    }
    report.note(
        "★★★ the preview tracked the drag — the picture and the commit agree, which is what \
         O154 was missing",
    );
    Ok(None)
}
