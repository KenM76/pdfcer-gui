//! `scale_ratio_reads_as_the_drawing_states_it` — O242: the Set-scale ratio
//! row carries a unit on each side, and a unit typed into a box converts.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/scale_ratio_units.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, TAB_EVENT, declared, declared_in, declared_names, list, shell_trace,
};
use crate::checks::scripteddrive::offscreen;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The ratio row's regions, left to right: paper number, paper unit, world
/// number, world unit.
const ROW: [&str; 4] = [
    "scale.ratio_paper",
    "scale.basis",
    "scale.ratio_real",
    "scale.real_unit",
];

/// Twenty feet, in inches. The world length every typing below spells.
const TWENTY_FT_IN: f64 = 240.0;

/// See the module documentation.
pub struct ScaleRatioReadsAsTheDrawingStatesIt;

impl Check for ScaleRatioReadsAsTheDrawingStatesIt {
    fn name(&self) -> &'static str {
        "scale_ratio_reads_as_the_drawing_states_it"
    }

    fn defect(&self) -> &'static str {
        "the Set-scale ratio is two bare numbers with the units in separate rows below, so \
         it is not clear which unit each side is in, and a unit typed into a ratio box is \
         not read"
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

/// Inches in one of the unit named by its `Debug` spelling. An independent
/// table, not the engine's: 25.4 mm to the inch, 12 in to the foot, 36 to the
/// yard, 63,360 to the mile.
fn inches_per(unit: &str) -> Option<f64> {
    Some(match unit {
        "Millimeter" => 1.0 / 25.4,
        "Centimeter" => 1.0 / 2.54,
        "Meter" => 1.0 / 0.0254,
        "Kilometer" => 1.0 / 0.000_025_4,
        "Inch" => 1.0,
        "DecimalFeet" | "FeetInches" => 12.0,
        "Yard" => 36.0,
        "Mile" => 63_360.0,
        _ => return None,
    })
}

/// What to type for twenty feet into a box held in `unit`: always a unit the
/// box is NOT in, so a build that ignores the typed unit reads a different
/// number.
fn typed_for(unit: &str) -> &'static str {
    match unit {
        "DecimalFeet" | "FeetInches" => "240 in",
        _ => "20 ft",
    }
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
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("scale_ratio_units.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = offscreen(ctx, &mut spec, "scale_ratio_units")?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(40);

    // --- 1: Review, Measure, Set scale ---------------------------------------
    crate::checks::driving::click_mode_segment(&session, &pointer, ui_rect, "review")?;
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.measure").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.measure` region in Review. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    pointer.click(&session, WindowPoint::centre_of(tab))?;
    session.settle(14);
    if !shell_trace(&session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some("measure"))
    {
        return Err(Error::new(
            "the click on the Measure tab produced no tab-selected line.",
        ));
    }
    open_set_scale(&session, &pointer, ui_rect)?;

    // --- 2: the row reads left to right on one line --------------------------
    let trace = session.trace()?;
    if declared(&trace, ui_rect, "dialog:set-scale").is_none() {
        return Ok(Some(
            "`measure.set_scale` was clicked and no `dialog:set-scale` region appeared.".to_owned(),
        ));
    }
    let mut rects = Vec::new();
    for name in ROW {
        let Some(r) = declared(&trace, ui_rect, name) else {
            return Ok(Some(format!(
                "the Set-scale window declared no `{name}` region, so the ratio row does not \
                 put a unit beside each of its numbers. Declared: {}.",
                list(&declared_names(&trace, ui_rect, "scale."))
            )));
        };
        rects.push(r);
    }
    let first = rects[0];
    for (name, r) in ROW.iter().zip(&rects).skip(1) {
        let overlaps = r.min.y < first.max.y && r.max.y > first.min.y;
        if !overlaps {
            return Ok(Some(format!(
                "`{name}` is not on the same line as `{}` ({r:?} vs {first:?}): a unit on a \
                 different row is the layout O242 reported.",
                ROW[0]
            )));
        }
    }
    if !rects.windows(2).all(|w| w[0].max.x <= w[1].min.x + 1.0) {
        return Ok(Some(format!(
            "the ratio row is not in reading order {ROW:?}: {rects:?}."
        )));
    }
    report.note("the ratio row reads number, unit, number, unit on one line");

    // --- 3: a unit typed into the world box converts -------------------------
    let Some(real_unit) = trace
        .events("scale-seeded")
        .filter_map(|l| l.get("real_unit").map(str::to_owned))
        .last()
    else {
        return Ok(Some(
            "the window traced no `scale-seeded` line carrying `real_unit=`, so the world \
             side has no unit of its own."
                .to_owned(),
        ));
    };
    let Some(ipu) = inches_per(&real_unit) else {
        return Err(Error::new(format!(
            "the world side is in `{real_unit}`, a unit this check has no inch factor for."
        )));
    };
    let typed = typed_for(&real_unit);
    let vp = declared_in(&trace, ui_rect, "scale.ratio_real").and_then(|(_, v)| v);
    pointer.click_in(&session, vp.as_deref(), WindowPoint::centre_of(rects[2]))?;
    session.settle(10);
    pointer.type_text(&session, vp.as_deref(), typed)?;
    pointer.key(&session, vp.as_deref(), "Enter", None)?; // ui-text-exempt: a key name, never displayed
    session.settle(16);
    report.note(format!(
        "typed `{typed}` into a world box held in {real_unit}"
    ));

    let trace = session.trace()?;
    let accept = declared(&trace, ui_rect, "dialog:set-scale.accept")
        .ok_or_else(|| Error::new("the window declared no `dialog:set-scale.accept` region."))?;
    let vp = declared_in(&trace, ui_rect, "dialog:set-scale.accept").and_then(|(_, v)| v);
    pointer.click_in(&session, vp.as_deref(), WindowPoint::centre_of(accept))?;
    session.settle(20);
    pointer.gone(&session)?;
    let trace = session.trace()?;
    let Some(line) = trace.events("scale-commit").last() else {
        return Ok(Some(format!(
            "Accept was pressed after typing `{typed}` and no `scale-commit` line was traced."
        )));
    };
    let committed_unit = line.get("real_unit").unwrap_or_default().to_owned();
    let Some(real) = line
        .get("ratio")
        .and_then(|r| r.split_once(':'))
        .and_then(|(_, real)| real.parse::<f64>().ok())
    else {
        return Ok(Some(format!(
            "the `scale-commit` line carries no readable `ratio=`: `{}`",
            line.raw
        )));
    };
    if committed_unit != real_unit {
        return Ok(Some(format!(
            "the world unit changed from `{real_unit}` to `{committed_unit}` between opening \
             and Accept, and nothing here chose a unit."
        )));
    }
    let expected = TWENTY_FT_IN / ipu;
    if (real - expected).abs() > expected * 1e-6 {
        return Ok(Some(format!(
            "`{typed}` typed into a world box held in {real_unit} committed {real}, not \
             {expected}: the typed unit was ignored or mis-converted."
        )));
    }
    report.note(format!(
        "`{typed}` committed as {real} {real_unit}, twenty feet exactly"
    ));
    Ok(None)
}

/// Click Measure ▸ Scale.
fn open_set_scale(session: &Session, pointer: &ScriptedPointer, ui_rect: &str) -> Result<()> {
    let trace = session.trace()?;
    let item = declared(&trace, ui_rect, "ribbon.item.measure.set_scale").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.item.measure.set_scale` region on the Measure tab. Items declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.item.measure."))
        ))
    })?;
    pointer.click(session, WindowPoint::centre_of(item))?;
    session.settle(16);
    Ok(())
}
