//! `freehand_detail_sets_how_many_segments_a_stroke_keeps` — Markup ▸ Style's
//! Detail control changes how many points a freehand stroke keeps. Scripted
//! pointer, window off the desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/freehand_detail.md`.

use super::dimdrive::{Fixture, press_on_tab, reveal, run_on, ui_rect_event};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::Result;
use crate::input::scripted::{ScriptedPointer, xy};
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.review";
const FIXTURE: Fixture = Fixture {
    file: "square-no-appearance.pdf",
    method: "Committed with the repository; restore it from git.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const STEM: &str = "freehand-detail";
const MARKUP_TAB: &str = "ribbon.tab.markup";
/// Mirrors `markupswatch::REGION_DETAIL`.
const DETAIL: &str = "markup.style.detail";
const INK: &str = "ribbon.item.markup.ink";
const PEN: &str = "markup-pen"; // ui-text-exempt: a trace event name, never displayed
const COMMIT: &str = "markup-commit"; // ui-text-exempt: a trace event name, never displayed
const TOOL: &str = "markup-tool"; // ui-text-exempt: a trace event name, never displayed
/// The wave's samples, its left end and span (page points), and amplitude.
const SAMPLES: usize = 60;
const LEFT: f64 = 120.0;
const SPAN: f64 = 360.0;
const AMPLITUDE: f64 = 12.0;

/// See the module documentation.
pub struct FreehandDetailSetsHowManySegmentsAStrokeKeeps;

impl Check for FreehandDetailSetsHowManySegmentsAStrokeKeeps {
    fn name(&self) -> &'static str {
        "freehand_detail_sets_how_many_segments_a_stroke_keeps"
    }

    fn defect(&self) -> &'static str {
        "Markup ▸ Style has no Detail control, or moving it does not change how many points a \
         freehand stroke keeps"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, INVOKE, STEM, steps)
    }
}

/// Draw one wave at page height `y` and answer the commit's `(raw, kept)`.
fn wave(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    y: f64,
) -> Result<Option<(usize, usize)>> {
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, FIXTURE.page, 0)?;
    let mut points = Vec::with_capacity(SAMPLES + 1);
    for i in 0..=SAMPLES {
        #[allow(clippy::cast_precision_loss)]
        let t = i as f64 / SAMPLES as f64;
        let x = LEFT + SPAN * t;
        let dy = AMPLITUDE * (t * std::f64::consts::TAU * 5.0).sin();
        points.push(mapping.doc_to_window(DocPoint::new(0, x, y + dy))?);
    }
    let mark = session.trace()?.mark();
    pointer.send(session, &format!("down {}", xy(points[0])))?;
    for p in &points[1..] {
        pointer.send(session, &format!("move {}", xy(*p)))?;
    }
    pointer.send(session, &format!("up {}", xy(points[SAMPLES])))?;
    session.settle(30);
    let trace = session.trace()?;
    Ok(trace
        .last_after(COMMIT, mark)
        .filter(|l| l.get("kind") == Some("Ink"))
        .and_then(|l| Some((l.get_usize("raw")?, l.get_usize("kept")?))))
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    reveal(ctx, session, pointer, DETAIL, MARKUP_TAB)?;
    let Some((rect, vp)) = declared_in(&session.trace()?, ui_rect_event(ctx)?, DETAIL) else {
        return Ok(Some(format!(
            "★★★ Markup ▸ Style drew no Detail control (`{DETAIL}`). Trace: {path}."
        )));
    };

    press_on_tab(ctx, session, pointer, INK, MARKUP_TAB)?;
    let Some((raw_default, kept_default)) = wave(ctx, session, pointer, 620.0)? else {
        return Ok(Some(format!(
            "a freehand wave at the default detail traced no `{COMMIT} kind=Ink`. Trace: {path}."
        )));
    };
    report.note(format!(
        "default detail: raw={raw_default} kept={kept_default}"
    ));

    // Drag the control well past its right end: one point per unit, so 150
    // points from the centre clamps at 100 whatever the default is.
    let centre = WindowPoint::centre_of(rect);
    let vp = vp.map_or(String::new(), |v| format!(" vp={v}"));
    pointer.send(
        session,
        &format!(
            "drag {} {:.1} {:.1} steps=20{vp}",
            xy(centre),
            centre.x() + 150.0,
            centre.y()
        ),
    )?;
    session.settle(20);
    let detail = session
        .trace()?
        .last(PEN)
        .and_then(|l| l.get("detail").map(str::to_owned))
        .unwrap_or_default();
    report.note(format!("pen after the drag: detail={detail}"));
    if detail != "100" {
        return Ok(Some(format!(
            "★★★ dragging the Detail control right left the pen at `detail={detail}`, not 100. \
             Trace: {path}."
        )));
    }

    let armed = session
        .trace()?
        .last(TOOL)
        .and_then(|l| l.get("tool").map(str::to_owned));
    if armed.as_deref() != Some("Markup(Ink)") {
        press_on_tab(ctx, session, pointer, INK, MARKUP_TAB)?;
    }
    let Some((raw_full, kept_full)) = wave(ctx, session, pointer, 700.0)? else {
        return Ok(Some(format!(
            "a freehand wave at full detail traced no `{COMMIT} kind=Ink`. Trace: {path}."
        )));
    };
    report.note(format!("full detail: raw={raw_full} kept={kept_full}"));
    Ok((kept_full <= kept_default).then(|| {
        format!(
            "★★★ full detail kept {kept_full} of {raw_full} points and the default kept \
             {kept_default} of {raw_default}: the setting did not reach the stroke. Trace: {path}."
        )
    }))
}
