//! `a_dimension_drag_previews_what_it_places` — dragging a ce dimension's
//! label shows, with the button still down, the dimension the release will
//! write, and the release writes it where the preview showed it.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/dimension_label_drag.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_or_in_overflow, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, ScreenPoint, WindowFrame};
use crate::error::{Error, Result};
use crate::geom::{LRect, PixRect, Pt};
use crate::image::Image;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::pixels;
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode the shape is drawn in.
const MODE: &str = "review";
/// The Measure tab.
const TAB: &str = "ribbon.tab.measure";
/// The ribbon item that arms the Perimeter tool.
const ITEM: &str = "ribbon.item.measure.perimeter";
/// `measure-tool tool=…` — the canvas reporting what armed.
const ARM_EVENT: &str = "measure-tool";
/// The `Debug` spelling of `CanvasTool::Measure(MeasureKind::Perimeter)`.
const ARM_VALUE: &str = "Measure(Perimeter)";
/// `add-dimension …` — the engine accepted the traced shape.
const COMMIT_EVENT: &str = "add-dimension";
/// `dim-preview baked=1 px=… faults=… label=x0,y0,…,x3,y3` — the painter drew
/// the engine's bake; `baked=0 refused=…` — the engine refused it.
const PREVIEW_EVENT: &str = "dim-preview";
/// `dimension-place id=… offset=… text_along=…` — the release.
const PLACE_EVENT: &str = "dimension-place";
/// The canvas viewport region, the clip for every pixel rectangle.
const VIEWPORT_REGION: &str = "canvas-viewport"; // ui-text-exempt: a trace region name
/// The four corners of the traced square, as fractions of the page box.
const CORNERS: [(f64, f64); 4] = [(0.30, 0.30), (0.60, 0.30), (0.60, 0.60), (0.30, 0.60)];
/// How far right the label is dragged, as a fraction of the page width. Far
/// enough that the moved label clears the square entirely.
const DRAG_DX: f64 = 0.25;
/// Pixels added round the label box before it is read, so antialiasing at
/// its edge counts.
const PAD_PX: u32 = 2;
/// The fraction of the label box that must be ink for the label to count as
/// drawn there. A text run's box is mostly paper between glyphs; one percent
/// separates a drawn label from blank paper with a wide margin.
const INKED: f64 = 0.01;
/// The least the previewed label's ink may be as a fraction of the original
/// label's, read from the same mid-drag frame. Both are the same text at the
/// same size, so a lighter preview is a resampled one.
const AS_DARK: f64 = 0.85;

/// See the module documentation.
pub struct ADimensionDragPreviewsWhatItPlaces;

impl Check for ADimensionDragPreviewsWhatItPlaces {
    fn name(&self) -> &'static str {
        "a_dimension_drag_previews_what_it_places"
    }

    fn defect(&self) -> &'static str {
        "dragging a ce dimension's label showed a thin outline of segments rather than the \
         dimension itself, so the operator could not see that only the text was moving, nor \
         where it would land. His ask, verbatim: \"when I click on the dimension text and drag \
         it should live preview so that it is apparent I am just moving the dimension text\""
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

/// The eight numbers of a `label=` field, as four page-space points.
pub(crate) fn label_quad(field: &str) -> Option<[(f64, f64); 4]> {
    let n: Vec<f64> = field
        .split(',')
        .map(str::parse)
        .collect::<std::result::Result<_, _>>()
        .ok()?;
    let [x0, y0, x1, y1, x2, y2, x3, y3] = n.as_slice() else {
        return None;
    };
    Some([(*x0, *y0), (*x1, *y1), (*x2, *y2), (*x3, *y3)])
}

/// The capture-pixel box round a page-space quad, padded and clipped to `to`.
pub(crate) fn quad_pixels(
    mapping: &CanvasMapping,
    frame: &WindowFrame,
    quad: [(f64, f64); 4],
    to: PixRect,
) -> Result<Option<PixRect>> {
    let mut min = Pt::new(f32::MAX, f32::MAX);
    let mut max = Pt::new(f32::MIN, f32::MIN);
    for (x, y) in quad {
        let w = mapping.doc_to_window(DocPoint::new(0, x, y))?;
        min = Pt::new(min.x.min(w.x()), min.y.min(w.y()));
        max = Pt::new(max.x.max(w.x()), max.y.max(w.y()));
    }
    let r = frame.logical_to_capture_pixels(LRect::new(min, max));
    let x0 = r.x.saturating_sub(PAD_PX).max(to.x);
    let y0 = r.y.saturating_sub(PAD_PX).max(to.y);
    let x1 = (r.x + r.w + PAD_PX).min(to.x + to.w);
    let y1 = (r.y + r.h + PAD_PX).min(to.y + to.h);
    Ok((x1 > x0 && y1 > y0).then(|| PixRect::new(x0, y0, x1 - x0, y1 - y0)))
}

/// The ink fraction of `region`, and a one-line account of it.
pub(crate) fn ink(img: &Image, region: PixRect) -> (f64, String) {
    let report = pixels::ink_run_into(img, region);
    #[allow(clippy::cast_precision_loss)]
    let fraction = report.ink as f64 / report.sampled.max(1) as f64;
    (fraction, report.summary())
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no fixture document. Pass --pdf."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check draws a shape, selects it and drags its \
             label with a capture taken while the button is held. Reported as SKIPPED rather \
             than passed: a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("dimension_label_drag.trace.txt"));
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
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- 1: Review, Measure, Perimeter ------------------------------------
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    let trace = session.trace()?;
    let Some(tab) = declared(&trace, ui_rect, TAB) else {
        return Ok(Some(format!(
            "the `{MODE}` mode declares no `{TAB}` region."
        )));
    };
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    let Some(item) = declared_or_in_overflow(&session, &driver, ui_rect, ITEM)? else {
        return Ok(Some(format!(
            "the Measure tab declares no `{ITEM}`. Items declared: {}.",
            list(&crate::checks::driving::declared_names(
                &session.trace()?,
                ui_rect,
                "ribbon.item.measure."
            ))
        )));
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(16);
    let trace = session.trace()?;
    if !trace
        .events(ARM_EVENT)
        .any(|l| l.get("tool") == Some(ARM_VALUE))
    {
        return Ok(Some(format!(
            "the Perimeter item was clicked and no `{ARM_EVENT} tool={ARM_VALUE}` followed."
        )));
    }

    // --- 2: trace a closed square — the ce dimension to drag ---------------
    if trace
        .last("canvas")
        .and_then(|l| l.get("page"))
        .and_then(|v| v.parse::<usize>().ok())
        != Some(0)
    {
        return Err(Error::new(
            "the canvas is not showing page 1, so the page geometry does not describe what is \
             on screen.",
        ));
    }
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;
    let frame = session.frame()?;
    let at = |fx: f64, fy: f64| -> Result<ScreenPoint> {
        Ok(frame.to_screen(mapping.doc_to_window(DocPoint::new(
            0,
            fx * page.width_pt,
            fy * page.height_pt,
        ))?))
    };
    let aimed: Vec<ScreenPoint> = CORNERS
        .iter()
        .map(|&(fx, fy)| at(fx, fy))
        .collect::<Result<_>>()?;
    for screen in &aimed {
        driver.click_at(*screen)?;
        session.settle(12);
    }
    driver.click_at(aimed[0])?;
    session.settle(18);
    // The placing click, at the vertex centroid: the text where it always sat.
    driver.click_at(at(
        f64::midpoint(CORNERS[0].0, CORNERS[2].0),
        f64::midpoint(CORNERS[0].1, CORNERS[2].1),
    )?)?;
    session.settle(30);
    if session.trace()?.events(COMMIT_EVENT).count() == 0 {
        return Ok(Some(format!(
            "the square did not commit, so there is no ce dimension to drag. That is \
             `measure_perimeter`'s subject — run it first. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- 3: select it — a click on an EDGE, since the fill is a hole ------
    if !crate::checks::driving::arm_select_from_ribbon(&session, &driver, ui_rect, report)? {
        driver.press(vk::V)?;
        session.settle(12);
    }
    let edge = (
        f64::midpoint(CORNERS[0].0, CORNERS[1].0),
        f64::midpoint(CORNERS[0].1, CORNERS[1].1),
    );
    let grab = at(edge.0, edge.1)?;
    driver.click_at(grab)?;
    session.settle(18);

    let viewport = session.trace()?;
    let canvas_px = frame.logical_to_capture_pixels(
        declared(&viewport, ui_rect, VIEWPORT_REGION).ok_or_else(|| {
            Error::new(format!(
                "the application declares no `{VIEWPORT_REGION}` region, so there is no clip for \
                 the label box. SKIPPED."
            ))
        })?,
    );
    let before_shot = ctx.out("dimension_label_drag_before.png");
    let before = crate::capture::window_to_png(&session, &before_shot)?;
    report.artifact(before_shot);

    // --- 4: drag, and photograph it with the button down ------------------
    let drop_at = at(edge.0 + DRAG_DX, edge.1)?;
    let during_shot = ctx.out("dimension_label_drag_during.png");
    let during = driver.drag_observed(grab, drop_at, || {
        crate::capture::window_to_png(&session, &during_shot)
    })?;
    report.artifact(during_shot);
    let trace = session.trace()?;
    if let Some(refused) = trace
        .events(PREVIEW_EVENT)
        .find(|l| l.get("baked") == Some("0"))
    {
        return Ok(Some(format!(
            "★ THE ENGINE REFUSED TO BAKE THE MOVED DIMENSION: `{}`. The drag fell back to the \
             segment outline. `EditSession::dimension_preview` is the verb and its refusal text \
             is in the line.",
            refused.raw
        )));
    }
    let Some(baked) = trace
        .events(PREVIEW_EVENT)
        .filter(|l| l.get("baked") == Some("1"))
        .last()
    else {
        return Ok(Some(format!(
            "★ NO BAKED PREVIEW WAS DRAWN WHILE THE LABEL WAS DRAGGED: no `{PREVIEW_EVENT} \
             baked=1` line. Either the press did not reach `canvas::dimdrag::drag` (a \
             `{PLACE_EVENT}` line after the release would say it did), or `dimpreview::paint` \
             declined — box off screen or over its size cap. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let Some(quad) = baked.get("label").and_then(label_quad) else {
        return Err(Error::new(format!(
            "the `{PREVIEW_EVENT}` line carries no readable `label=` field: `{}`",
            baked.raw
        )));
    };
    let Some(label_px) = quad_pixels(&mapping, &frame, quad, canvas_px)? else {
        return Err(Error::new(format!(
            "the previewed label box {quad:?} is off the canvas. SKIPPED — the window is too \
             small for this check's geometry."
        )));
    };
    let (was, was_account) = ink(&before, label_px);
    let (now, now_account) = ink(&during, label_px);
    if was >= INKED {
        return Err(Error::new(format!(
            "the label's destination {label_px:?} already carried ink before the drag ({was_account}), \
             so ink there afterwards would prove nothing. SKIPPED — use a fixture with blank paper \
             right of the middle of the page."
        )));
    }
    if now < INKED {
        return Ok(Some(format!(
            "★ THE PREVIEW IS NOT ON THE GLASS. The trace says `{}`, and the label box it names, \
             {label_px:?}, carries {now_account} with the button down — blank paper. Between the \
             painter's account and the frame lie the texture upload, the clip and the page-to-\
             screen transform `dimpreview::page_to_screen` reads off three mapped points.",
            baked.raw
        )));
    }
    report.note(format!(
        "★ the dimension is drawn at the drop with the button down: `{}`; label box {label_px:?} \
         went from {was_account} to {now_account}",
        baked.raw
    ));

    // The original label is still on the page mid-drag, a drag's width to the
    // left: the same text, drawn by the page raster. The preview must be as
    // dark as it.
    let dx = DRAG_DX * page.width_pt;
    let original = quad.map(|(x, y)| (x - dx, y));
    if let Some(original_px) = quad_pixels(&mapping, &frame, original, canvas_px)? {
        let (base, base_account) = ink(&during, original_px);
        if base >= INKED && now < base * AS_DARK {
            return Ok(Some(format!(
                "★ THE PREVIEW IS LIGHTER THAN THE DIMENSION IT PREVIEWS. Mid-drag, the original \
                 label at {original_px:?} carries {base_account}; the preview at {label_px:?} \
                 carries {now_account}. Same text, same size: a lighter copy is a texture drawn \
                 off the pixel grid and resampled. `dimpreview::paint` snaps its box to whole \
                 pixels for this."
            )));
        }
        report.note(format!(
            "the preview is as dark as the original label beside it ({base_account})"
        ));
    }

    // --- 5: release — the commit lands where the preview was --------------
    session.settle(30);
    let trace = session.trace()?;
    let Some(place) = trace.last(PLACE_EVENT) else {
        return Ok(Some(format!(
            "the preview was drawn and the release placed nothing: no `{PLACE_EVENT}` line. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let after_shot = ctx.out("dimension_label_drag_after.png");
    let after = crate::capture::window_to_png(&session, &after_shot)?;
    report.artifact(after_shot);
    let (then, then_account) = ink(&after, label_px);
    if then < INKED {
        return Ok(Some(format!(
            "★ THE RELEASE PUT THE LABEL SOMEWHERE ELSE. The preview drew it in {label_px:?}; after \
             `{}` that box carries {then_account}. The preview and the commit disagree, which is \
             the one thing a baked preview exists to rule out.",
            place.raw
        )));
    }
    report.note(format!(
        "the release placed it where the preview showed: `{}`; the label box carries \
         {then_account}",
        place.raw
    ));
    Ok(None)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::float_cmp)]
mod tests {
    use super::label_quad;

    #[test]
    fn a_label_field_reads_as_four_points() {
        let q = label_quad("1.0,2.0,3.0,4.0,5.5,6.0,7.0,8.0").unwrap();
        assert_eq!(q[2], (5.5, 6.0));
        assert!(label_quad("1,2,3").is_none());
        assert!(label_quad("1,2,3,4,5,6,7,x").is_none());
    }
}
