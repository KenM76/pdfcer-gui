//! `every_tab_draws_its_glyph` — each ribbon tab and each panel tab draws an
//! icon inside itself. See `docs/modules/ui-verify/checks/tab_glyphs.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::image::Image;
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::trace::Trace;

const OFFSCREEN: &str = "-4200,-4200,1400,900";

/// The trace line the icon painter writes when a key names no icon; the glyph
/// it then draws is the missing mark, which has ink and would pass the pixel test.
const UNKNOWN_KEY_EVENT: &str = "icon-unknown-key";

/// Slack on the containment test, in points, for sub-pixel rounding.
const SLACK_PTS: f32 = 0.5;

pub struct EveryTabDrawsItsGlyph;

impl Check for EveryTabDrawsItsGlyph {
    fn name(&self) -> &'static str {
        "every_tab_draws_its_glyph"
    }

    fn defect(&self) -> &'static str {
        "a ribbon tab or a panel tab is drawn as text alone, or its icon is blank or the missing \
         mark (OPERATOR_REQUESTS.md O298)"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report)?;
    let shot = ctx.out("tab_glyphs.png");
    pointer.screenshot(&session, &shot)?;
    let image = Image::load_png(&shot)?;
    report.artifact(shot);
    let frame = session.frame()?;
    let trace = session.trace()?;
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");

    let mut faults = Vec::new();
    for family in ["ribbon", "dock"] {
        let tabs = tabs_of(&trace, ui_rect, family);
        if tabs.len() < 2 {
            return Err(Error::new(format!(
                "the {family} declared {} live tab(s) ({}); nothing to judge.",
                tabs.len(),
                driving::list(&tabs.iter().map(|t| t.0.clone()).collect::<Vec<_>>())
            )));
        }
        report.note(format!("{family}: {} tabs", tabs.len()));
        for (id, tab) in &tabs {
            let icon_name = format!("tabicon.{family}.{id}");
            let verdict = judge(&trace, ui_rect, &icon_name, *tab, &image, &frame);
            report.note(format!("  {family}.tab.{id}: {verdict}"));
            if verdict != "glyph drawn" {
                faults.push(format!("{family} tab `{id}`: {verdict}"));
            }
        }
    }
    let unknown: Vec<String> = trace
        .events(UNKNOWN_KEY_EVENT)
        .filter_map(|l| l.get("key").map(str::to_owned))
        .collect();
    if !unknown.is_empty() {
        faults.push(format!(
            "the icon painter was asked for key(s) it has no icon for, and drew the missing \
             mark: {}",
            driving::list(&unknown)
        ));
    }
    if faults.is_empty() {
        return Ok(None);
    }
    Ok(Some(format!(
        "{} tab glyph fault(s): {}",
        faults.len(),
        faults.join("; ")
    )))
}

/// Launch off the desktop on a fixture, so the dock has panels to show.
fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let pdf = driving::repo_fixture("cropped-sheets.pdf", "")?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("tab_glyphs.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("tab_glyphs.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(pointer.path().to_path_buf());
    report.artifact(session.trace_path().to_path_buf());
    session.settle(30);
    Ok((session, pointer))
}

/// Every live tab of one family, as (id, rect). A ribbon tab id has no dot;
/// a dock panel id may.
fn tabs_of(trace: &Trace, ui_rect: &str, family: &str) -> Vec<(String, LRect)> {
    let prefix = format!("{family}.tab.");
    driving::declared_names(trace, ui_rect, &prefix)
        .into_iter()
        .filter_map(|name| {
            let id = name[prefix.len()..].to_owned();
            if family == "ribbon" && id.contains('.') {
                return None;
            }
            driving::declared(trace, ui_rect, &name).map(|rect| (id, rect))
        })
        .collect()
}

/// One tab's verdict: `"glyph drawn"`, or what is wrong with its glyph.
fn judge(
    trace: &Trace,
    ui_rect: &str,
    icon_name: &str,
    tab: LRect,
    image: &Image,
    frame: &crate::coords::WindowFrame,
) -> String {
    let Some(icon) = driving::declared(trace, ui_rect, icon_name) else {
        return format!("no `{icon_name}` region: the tab draws no icon");
    };
    let grown = LRect::new(
        crate::geom::Pt {
            x: tab.min.x - SLACK_PTS,
            y: tab.min.y - SLACK_PTS,
        },
        crate::geom::Pt {
            x: tab.max.x + SLACK_PTS,
            y: tab.max.y + SLACK_PTS,
        },
    );
    if !grown.contains_rect(icon) {
        return format!("the icon at {icon:?} lies outside the tab at {tab:?}");
    }
    let px = frame.logical_to_capture_pixels(icon);
    if px.area() == 0 {
        return "the icon's rect has no pixels in the capture".to_owned();
    }
    let ink = crate::pixels::region_not_uniform(image, px);
    if ink.is_uniform() {
        return format!("the icon's rect is blank ({})", ink.summary());
    }
    "glyph drawn".to_owned()
}
