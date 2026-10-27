//! `a_freehand_line_can_end_in_an_arrowhead` — choosing an arrowhead on
//! Markup ▸ Style makes the next freehand stroke a `/PolyLine` whose saved
//! `/LE` carries it. Scripted pointer, window off the desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/freehand_arrowheads.md`.

use super::dimdrive::{Fixture, press, press_on_tab, reveal, ui_rect_event};
use super::multiselect::{Saves, run_saving, save};
use super::savedannots::{listed_annots, read};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
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
const STEM: &str = "freehand-arrowheads";
const MARKUP_TAB: &str = "ribbon.tab.markup";
/// Mirrors `markupswatch::REGION_ENDS` and `REGION_ENDS_END`.
const ENDS: &str = "markup.style.ends";
const AT_END: &str = "markup.style.ends.end";
const INK: &str = "ribbon.item.markup.ink";
const PEN: &str = "markup-pen"; // ui-text-exempt: a trace event name, never displayed
const COMMIT: &str = "markup-commit"; // ui-text-exempt: a trace event name, never displayed
/// The saved arrowhead pair "none at the start, open arrow at the end",
/// whitespace removed.
const WANT_LE: &str = "/LE[/None/OpenArrow]";
const SAMPLES: usize = 40;
const SELECT: &str = "annot-select"; // ui-text-exempt: a trace event name, never displayed
const FORMAT_TAB: &str = "ribbon.tab.format";
const FORMAT_ARROWHEADS: &str = "ribbon.item.format.arrowheads";

/// See the module documentation.
pub struct AFreehandLineCanEndInAnArrowhead;

impl Check for AFreehandLineCanEndInAnArrowhead {
    fn name(&self) -> &'static str {
        "a_freehand_line_can_end_in_an_arrowhead"
    }

    fn defect(&self) -> &'static str {
        "Markup ▸ Style offers no arrowhead for a freehand line, or the drawn line is saved \
         without it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_saving(self, ctx, &FIXTURE, INVOKE, STEM, steps)
    }
}

/// Open the arrowhead list and pick "At the end"; the failure sentence when
/// either surface did not draw or the pen did not take it.
fn choose_end(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    path: &str,
) -> Result<Option<String>> {
    reveal(ctx, session, pointer, ENDS, MARKUP_TAB)?;
    if declared_in(&session.trace()?, ui_rect_event(ctx)?, ENDS).is_none() {
        return Ok(Some(format!(
            "★★★ Markup ▸ Style drew no arrowhead chooser (`{ENDS}`). Trace: {path}."
        )));
    }
    press(ctx, session, pointer, ENDS)?;
    session.settle(15);
    if declared_in(&session.trace()?, ui_rect_event(ctx)?, AT_END).is_none() {
        return Ok(Some(format!(
            "pressing the arrowhead chooser opened no list with `{AT_END}`. Trace: {path}."
        )));
    }
    press(ctx, session, pointer, AT_END)?;
    session.settle(15);
    let ends = session
        .trace()?
        .last(PEN)
        .and_then(|l| l.get("ends").map(str::to_owned))
        .unwrap_or_default();
    Ok((ends != "None,OpenArrow").then(|| {
        format!("★★★ picking \"At the end\" left the pen at `ends={ends}`. Trace: {path}.")
    }))
}

/// Draw a shallow freehand arc; the commit line's `kind`.
fn arc(ctx: &CheckContext, session: &Session, pointer: &ScriptedPointer) -> Result<Option<String>> {
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, FIXTURE.page, 0)?;
    let mut points = Vec::with_capacity(SAMPLES + 1);
    for i in 0..=SAMPLES {
        #[allow(clippy::cast_precision_loss)]
        let t = i as f64 / SAMPLES as f64;
        let dy = 40.0 * (t * std::f64::consts::PI).sin();
        points.push(mapping.doc_to_window(DocPoint::new(0, 150.0 + 300.0 * t, 640.0 + dy))?);
    }
    let mark = session.trace()?.mark();
    pointer.send(session, &format!("down {}", xy(points[0])))?;
    for p in &points[1..] {
        pointer.send(session, &format!("move {}", xy(*p)))?;
    }
    pointer.send(session, &format!("up {}", xy(points[SAMPLES])))?;
    session.settle(30);
    Ok(session
        .trace()?
        .last_after(COMMIT, mark)
        .filter(|l| l.get("freehand") == Some("1"))
        .and_then(|l| l.get("kind").map(str::to_owned)))
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    saves: &Saves,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    if let Some(why) = choose_end(ctx, session, pointer, &path)? {
        return Ok(Some(why));
    }
    press_on_tab(ctx, session, pointer, INK, MARKUP_TAB)?;
    let Some(kind) = arc(ctx, session, pointer)? else {
        return Ok(Some(format!(
            "a freehand stroke traced no `{COMMIT} … freehand=1`. Trace: {path}."
        )));
    };
    report.note(format!("freehand commit kind={kind}"));
    if kind != "PolyLine" {
        return Ok(Some(format!(
            "★★★ a freehand stroke with an arrowhead was committed as `{kind}`, not PolyLine; \
             /Ink cannot carry one. Trace: {path}."
        )));
    }
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let annots = listed_annots(&read(&saves.moved)?);
    let polylines: Vec<String> = annots
        .iter()
        .filter(|a| a.subtype == "PolyLine")
        .map(|a| a.body.split_whitespace().collect())
        .collect();
    report.note(format!("saved PolyLines: {}", polylines.len()));
    if !polylines.iter().any(|b| b.contains(WANT_LE)) {
        return Ok(Some(format!(
            "★★★ {} lists {} /PolyLine, none with `{WANT_LE}`: the arrowhead did not reach the \
             file.",
            saves.moved.display(),
            polylines.len()
        )));
    }
    format_offers_arrowheads(ctx, session, pointer, &path)
}

/// Put the tool down, click the drawn polyline, and require Format to draw
/// its arrowhead chooser; the failure sentence otherwise.
fn format_offers_arrowheads(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    path: &str,
) -> Result<Option<String>> {
    pointer.key(session, None, "Escape", None)?;
    session.settle(10);
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, FIXTURE.page, 0)?;
    let apex = mapping.doc_to_window(DocPoint::new(0, 300.0, 680.0))?;
    let mark = session.trace()?.mark();
    pointer.click(session, apex)?;
    session.settle(20);
    let picked = session
        .trace()?
        .last_after(SELECT, mark)
        .and_then(|l| l.get("subtype").map(str::to_owned))
        .unwrap_or_default();
    if picked != "PolyLine" {
        return Ok(Some(format!(
            "a click on the drawn polyline's apex selected `{picked}`, not a PolyLine. Trace: \
             {path}."
        )));
    }
    reveal(ctx, session, pointer, FORMAT_ARROWHEADS, FORMAT_TAB)?;
    // An item the band does not offer is still declared, at zero size, so the
    // oracle is the extent, not the declaration.
    let offered = declared_in(&session.trace()?, ui_rect_event(ctx)?, FORMAT_ARROWHEADS)
        .is_some_and(|(r, _)| r.width() >= 10.0 && r.height() >= 10.0);
    Ok((!offered).then(|| {
        format!(
            "★★★ Format drew no arrowhead chooser (`{FORMAT_ARROWHEADS}` absent or empty) for \
             a selected polyline. Trace: {path}."
        )
    }))
}
