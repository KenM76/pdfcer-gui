//! `a_measurement_on_a_user_unit_page_reads_physically` — a Linear ce dimension
//! drawn across a known span of a `/UserUnit 2` page is shown at the span's
//! physical length, twice its length in the page's box units. Driven through
//! the scripted pointer on a window placed off the desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/measure_user_unit.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const MODE: &str = "ribbon.mode.review"; // ui-text-exempt: a trace region name, never displayed
const TAB: &str = "ribbon.tab.measure"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.measure.linear"; // ui-text-exempt: a trace region name, never displayed
/// Prefix of the Measure tab's groups, for finding a collapsed one.
const GROUPS: &str = "ribbon.group.measure."; // ui-text-exempt: a trace region name, never displayed
const SHOWN_EVENT: &str = "dimension-member-shown"; // ui-text-exempt: a trace event name, never displayed
const ADDED_EVENT: &str = "dimension-added"; // ui-text-exempt: a trace event name, never displayed
/// Off the desktop, so no OS input can reach it and none of his is taken.
const OFFSCREEN: &str = "-4200,-4200,1400,1000";
const FIXTURE: &str = "user-unit.pdf";
const METHOD: &str = "A page whose /UserUnit makes its box half its physical size; any other file \
                      cannot tell the two measures apart. Rebuild it from its PROVENANCE file.";
/// The fixture's frame, `18 18 270 360 re S`: its bottom edge's two corners,
/// in the page's user-space units. Corners, so the snap lands on the drawing.
const FROM: (f64, f64) = (18.0, 18.0);
const TO: (f64, f64) = (288.0, 18.0);
/// Where the dimension line is put: the third pick, inside the frame.
const LINE_AT: (f64, f64) = (153.0, 90.0);
/// Points per inch, ISO 32000-1 §8.3.2.3.
const PT_PER_IN: f64 = 72.0;
/// The label is printed to hundredths; this absorbs a pick a pixel off the
/// corner when the snap does not take it.
const TOLERANCE_PT: f64 = 2.0;

/// See the module documentation.
pub struct AMeasurementOnAUserUnitPageReadsPhysically;

impl Check for AMeasurementOnAUserUnitPageReadsPhysically {
    fn name(&self) -> &'static str {
        "a_measurement_on_a_user_unit_page_reads_physically"
    }

    fn defect(&self) -> &'static str {
        "a ce dimension measured on a page with /UserUnit reads the length in box units, so on a \
         /UserUnit 2 page every measurement shows half its printed length"
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

/// The launched window, the pointer that drives it and the region event.
struct Rig {
    session: Session,
    pointer: ScriptedPointer,
    ui_rect: &'static str,
}

impl Rig {
    fn click(&self, region: &str) -> Result<()> {
        let trace = self.session.trace()?;
        let rect = declared(&trace, self.ui_rect, region).ok_or_else(|| {
            let prefix = region.rsplit_once('.').map_or(region, |(head, _)| head);
            Error::new(format!(
                "no `{region}` region. Declared under `{prefix}`: {}.",
                list(&declared_names(&trace, self.ui_rect, prefix))
            ))
        })?;
        self.pointer
            .click(&self.session, WindowPoint::centre_of(rect))?;
        self.session.settle(15);
        Ok(())
    }

    /// Click `item`, opening each collapsed group under `groups` until it shows.
    fn click_item(&self, item: &str, groups: &str) -> Result<()> {
        if declared(&self.session.trace()?, self.ui_rect, item).is_none() {
            let collapsed: Vec<String> =
                declared_names(&self.session.trace()?, self.ui_rect, groups)
                    .into_iter()
                    .filter(|n| n.ends_with(".collapsed"))
                    .collect();
            for group in collapsed {
                self.click(&group)?;
                if declared(&self.session.trace()?, self.ui_rect, item).is_some() {
                    break;
                }
            }
        }
        self.click(item)
    }
}

fn launch(ctx: &CheckContext, report: &mut CheckReport, doc: std::path::PathBuf) -> Result<Rig> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("measure-user-unit.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("measure-user-unit.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    Ok(Rig {
        session,
        pointer,
        ui_rect,
    })
}

/// The page's `/UserUnit`, read from the file's bytes rather than from the
/// program, so the oracle does not share the program's reading of it.
fn user_unit_of(pdf: &std::path::Path) -> Option<f64> {
    let bytes = std::fs::read(pdf).ok()?;
    let text: String = bytes.iter().map(|&b| b as char).collect();
    let rest = &text[text.find("/UserUnit")? + "/UserUnit".len()..];
    rest.split(|c: char| c.is_whitespace() || c == '/' || c == '>')
        .find(|t| !t.is_empty())?
        .parse()
        .ok()
}

/// The number in a never-set group's raw readout, `"540.00 pt"`, with either
/// decimal marker.
fn points_in(text: &str) -> Option<f64> {
    text.trim()
        .strip_suffix("pt")?
        .trim()
        .replace(',', ".")
        .parse()
        .ok()
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let doc = ctx.out(FIXTURE);
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let page = crate::fixture::page_geometry(&doc)
        .ok_or_else(|| Error::new(format!("cannot read a page box from {}.", doc.display())))?;
    let unit = user_unit_of(&doc)
        .filter(|u| *u > 1.0)
        .ok_or_else(|| Error::new(format!("{FIXTURE} carries no /UserUnit above 1. {METHOD}")))?;
    let span = TO.0 - FROM.0;
    let physical = span * unit;
    report.note(format!(
        "oracle: {span} box units x /UserUnit {unit} = {physical} pt = {} in",
        physical / PT_PER_IN
    ));

    let rig = launch(ctx, report, doc)?;
    rig.click(MODE)?;
    rig.click(TAB)?;
    rig.click_item(ITEM, GROUPS)?;
    let mapping = CanvasMapping::from_trace(&rig.session.trace()?, &ctx.profile.vocab, page, 0)?;
    let at = |(x, y): (f64, f64)| mapping.doc_to_window(DocPoint::new(0, x, y));
    let added = rig.session.trace()?.events(ADDED_EVENT).count();
    for end in [FROM, TO, LINE_AT] {
        rig.pointer.hover(&rig.session, at(end)?)?;
        rig.session.settle(8);
        rig.pointer.click(&rig.session, at(end)?)?;
        rig.session.settle(15);
    }
    rig.session.settle(20);
    let trace = rig.session.trace()?;
    if trace.events(ADDED_EVENT).count() <= added {
        return Ok(Some(format!(
            "the picks along the frame's bottom edge and the placing pick committed no ce dimension: no new \
             `{ADDED_EVENT}` line. Trace: {}.",
            rig.session.trace_path().display()
        )));
    }
    let shot = ctx.out("measure-user-unit.png");
    rig.pointer.screenshot(&rig.session, &shot)?;
    report.artifact(shot);
    let Some(text) = trace
        .last(SHOWN_EVENT)
        .and_then(|l| l.get("text").map(str::to_owned))
    else {
        return Ok(Some(format!(
            "a ce dimension was committed and no `{SHOWN_EVENT}` line reports what it shows."
        )));
    };
    report.note(format!("the committed ce dimension shows `{text}`"));
    let Some(read) = points_in(&text) else {
        return Ok(Some(format!(
            "the ce dimension shows `{text}`, not a never-set group's raw readout in points."
        )));
    };
    if (read - span).abs() <= TOLERANCE_PT {
        return Ok(Some(format!(
            "the {} in span reads `{text}`: its {span} box units shown as points, ignoring \
             `/UserUnit {unit}`, so it reads 1/{unit} of its printed length.",
            physical / PT_PER_IN
        )));
    }
    if (read - physical).abs() > TOLERANCE_PT {
        return Ok(Some(format!(
            "the span reads `{text}`, neither its physical {physical} pt nor its box {span}."
        )));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_oracle_reads_the_fixture_and_the_label() {
        let pdf = repo_fixture(FIXTURE, METHOD).expect("fixture");
        assert_eq!(user_unit_of(&pdf), Some(2.0));
        assert_eq!(points_in("540.00 pt"), Some(540.0));
        assert_eq!(points_in("540,00 pt"), Some(540.0));
        assert_eq!(points_in("7.50 in"), None);
    }
}
