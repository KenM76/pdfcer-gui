//! Three Layers checks on `fixtures/layer-hiding.pdf`:
//! `a_parent_switch_carries_its_sublayers` (O316),
//! `a_band_takes_nothing_on_a_hidden_layer` (O317) and
//! `objects_list_under_their_layers` (O318).
//!
//! Design and rationale: `docs/modules/ui-verify/checks/layer_hiding.md`.

use super::dimdrive::{Fixture, launch_with, press, reveal, ui_rect_event};
use super::multiselect::band;
use crate::checks::forms_spotlight::open_from_tab;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "layer-hiding.pdf",
    method: "Rebuild it with `python fixtures/layer-hiding.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 800.0,
        height_pt: 800.0,
    },
};
const PANEL_ITEM: &str = "ribbon.item.view.panel_layers"; // ui-text-exempt: a trace region
const ROW: &str = "layer-row"; // ui-text-exempt: a trace event name
const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name
const OBJECTS_TAB: &str = "dock.tab.view.panel_objects"; // ui-text-exempt: a trace region
const OBJECTS_BODY: &str = "dock.body.view.panel_objects"; // ui-text-exempt: a trace region
const HEADINGS: &str = "objects-layers"; // ui-text-exempt: a trace event name

/// A window band (left to right) around the Steel square and the Bolts
/// square nested in its span, and nothing else.
const STEEL_BAND: ((f64, f64), (f64, f64)) = ((270.0, 750.0), (490.0, 370.0));
/// A point on the Notes square, clicked so the band's own line differs.
const ON_NOTES: (f64, f64) = (640.0, 660.0);

fn switch(name: &str) -> String {
    format!("panel.layers.switch.{name}")
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    stem: &str,
) -> Result<(Session, ScriptedPointer)> {
    launch_with(ctx, report, &FIXTURE, "mode.edit", stem, &[])
}

fn open_layers(ctx: &CheckContext, session: &Session, pointer: &ScriptedPointer) -> Result<()> {
    let ui_rect = ui_rect_event(ctx)?;
    if crate::checks::driving::declared(&session.trace()?, ui_rect, &switch("Structure")).is_none()
    {
        open_from_tab(session, pointer, ui_rect, "view", PANEL_ITEM)?;
        session.settle(24);
    }
    Ok(())
}

/// Each named layer's `visible=` on its last `layer-row` line after `mark`.
fn visibility(session: &Session, mark: usize, names: &[&str]) -> Result<Vec<Option<String>>> {
    let trace = session.trace()?;
    Ok(names
        .iter()
        .map(|n| {
            trace
                .events(ROW)
                .filter(|l| l.lineno > mark && l.get("name") == Some(n))
                .last()
                .and_then(|l| l.get("visible").map(str::to_owned))
        })
        .collect())
}

fn finish(
    report: CheckReport,
    outcome: Result<Option<String>>,
    session: Option<Session>,
) -> CheckReport {
    drop(session);
    match outcome {
        Ok(Some(why)) => report.fail(why),
        Ok(None) => report.pass(),
        Err(why) => report.from_error(&why),
    }
}

fn run_body(
    ctx: &CheckContext,
    name: &'static str,
    defect: &'static str,
    stem: &str,
    body: fn(&CheckContext, &mut CheckReport, &Session, &ScriptedPointer) -> Result<Option<String>>,
) -> CheckReport {
    let mut report = CheckReport::new(name, defect);
    match launch(ctx, &mut report, stem) {
        Ok((session, pointer)) => {
            let outcome = body(ctx, &mut report, &session, &pointer);
            finish(report, outcome, Some(session))
        }
        Err(why) => finish(report, Err(why), None),
    }
}

/// See the module documentation.
pub struct AParentSwitchCarriesItsSublayers;

impl Check for AParentSwitchCarriesItsSublayers {
    fn name(&self) -> &'static str {
        "a_parent_switch_carries_its_sublayers"
    }

    fn defect(&self) -> &'static str {
        "switching a parent layer off in the Layers panel leaves its unlocked sublayers on, \
         switches a locked sublayer, or switches a layer that is not under it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_body(
            ctx,
            self.name(),
            self.defect(),
            "layer-hiding-switch",
            parent_switch,
        )
    }
}

fn parent_switch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    open_layers(ctx, session, pointer)?;
    let names = ["Structure", "Steel", "Survey", "Bolts", "Notes"];
    let before = visibility(session, 0, &names)?;
    if before.iter().any(|v| v.as_deref() != Some("true")) {
        return Ok(Some(format!(
            "before any click the rows read {before:?} for {names:?}; every layer starts on."
        )));
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, &switch("Structure"))?;
    session.settle(20);
    let after = visibility(session, mark, &names)?;
    report.note(format!("after Structure's switch: {names:?} = {after:?}"));
    let want = ["false", "false", "true", "true", "true"];
    let wrong: Vec<String> = names
        .iter()
        .zip(&want)
        .zip(&after)
        .filter(|((_, w), got)| got.as_deref() != Some(**w))
        .map(|((n, w), got)| format!("{n} is {got:?}, wants {w}"))
        .collect();
    Ok((!wrong.is_empty()).then(|| {
        format!(
            "Structure's switch was clicked off. Structure and its unlocked sublayer Steel must \
             go off; locked Survey and the top-level Bolts and Notes must stay on. Wrong: {}.",
            wrong.join("; ")
        )
    }))
}

/// See the module documentation.
pub struct ABandTakesNothingOnAHiddenLayer;

impl Check for ABandTakesNothingOnAHiddenLayer {
    fn name(&self) -> &'static str {
        "a_band_takes_nothing_on_a_hidden_layer"
    }

    fn defect(&self) -> &'static str {
        "a band dragged over content on a hidden layer, or nested in a hidden layer's span, \
         selects it; or, the layer shown again, the same band does not"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_body(
            ctx,
            self.name(),
            self.defect(),
            "layer-hiding-band",
            hidden_band,
        )
    }
}

/// Drag [`STEEL_BAND`] and return the `sel=` of the band's own line.
fn band_count(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<std::result::Result<usize, String>> {
    let mark = session.trace()?.mark();
    band(ctx, session, pointer, &FIXTURE, STEEL_BAND)?;
    let trace = session.trace()?;
    Ok(trace
        .events(SELECTION)
        .filter(|l| l.lineno > mark && l.get("via") == Some("pv.marquee"))
        .last()
        .and_then(|l| l.get_usize("sel"))
        .ok_or_else(|| {
            format!(
                "a band was dragged and no `{SELECTION} via=pv.marquee` line followed, so no band \
                 began. Trace: {}.",
                session.trace_path().display()
            )
        }))
}

fn toggle_steel(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    want: &str,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, &switch("Steel"))?;
    session.settle(20);
    let got = visibility(session, mark, &["Steel"])?.remove(0);
    Ok((got.as_deref() != Some(want))
        .then(|| format!("Steel's switch was clicked and its row reads {got:?}, not {want}.")))
}

fn hidden_band(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    open_layers(ctx, session, pointer)?;
    super::dimdrive::click_on(ctx, session, pointer, &FIXTURE, 0, ON_NOTES)?;
    session.settle(20);
    if let Some(why) = toggle_steel(ctx, session, pointer, "false")? {
        return Ok(Some(why));
    }
    let hidden = match band_count(ctx, session, pointer)? {
        Ok(n) => n,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!("Steel hidden: the band selected {hidden}"));
    if let Some(why) = toggle_steel(ctx, session, pointer, "true")? {
        return Ok(Some(why));
    }
    let shown = match band_count(ctx, session, pointer)? {
        Ok(n) => n,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!("Steel shown: the same band selected {shown}"));
    Ok((hidden != 0 || shown != 2).then(|| {
        format!(
            "the band around the Steel square and the Bolts square nested in Steel's span took \
             {hidden} with Steel hidden (wants 0) and {shown} with Steel shown (wants 2)."
        )
    }))
}

/// See the module documentation.
pub struct ObjectsListUnderTheirLayers;

impl Check for ObjectsListUnderTheirLayers {
    fn name(&self) -> &'static str {
        "objects_list_under_their_layers"
    }

    fn defect(&self) -> &'static str {
        "the Objects panel lists a layered page's objects flat, or under headings out of \
         front-to-back order, or puts the content on no layer anywhere but last"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_body(
            ctx,
            self.name(),
            self.defect(),
            "layer-hiding-objects",
            headings,
        )
    }
}

fn headings(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    reveal(ctx, session, pointer, OBJECTS_BODY, OBJECTS_TAB)?;
    session.settle(20);
    let trace = session.trace()?;
    let Some(line) = trace.last(HEADINGS) else {
        return Ok(Some(format!(
            "the Objects panel drew no `{HEADINGS}` line on a page whose objects are on layers: \
             it lists them with no headings. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("`{}`", line.raw));
    // Front-most first: the unlayered square is painted last, so it is
    // front-most but its heading still goes last.
    let want = "Survey|Notes|Bolts|Steel|Structure|-";
    Ok((line.get("headings") != Some(want)).then(|| {
        format!(
            "the headings read {:?}; want {want:?} (layers by front-most object, Bolts under \
             its own innermost layer, the content on no layer last).",
            line.get("headings")
        )
    }))
}
