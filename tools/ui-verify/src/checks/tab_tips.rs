//! `a_glyph_only_tab_names_itself_on_hover` (O300): with both tab rows set
//! to show glyphs alone, hovering a ribbon tab and a dock tab shows a tip
//! that leads with the tab's name.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/tab_tips.md`.

use super::dimdrive::{Fixture, launch_with, ui_rect_event};
use crate::checks::driving::{declared_in, shell_trace};
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, WindowPoint};
use crate::error::{Error, Result};
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
const TIP: &str = "tab-tip"; // ui-text-exempt: a trace event name
const PREFS: &str = "ribbon_tabs_show = glyph\ndock_tabs_show = glyph\n"; // ui-text-exempt: a preferences file body

/// The tabs hovered: surface, tab id, its region, and the name it must lead with.
const TABS: [(&str, &str, &str, &str); 2] = [
    ("ribbon", "view", "ribbon.tab.view", "View"), // ui-text-exempt: trace ids and a tab name
    (
        "dock",
        "view.panel_objects",          // ui-text-exempt: a panel id
        "dock.tab.view.panel_objects", // ui-text-exempt: a trace region
        "Objects",                     // ui-text-exempt: a tab name
    ),
];

/// See the module documentation.
pub struct AGlyphOnlyTabNamesItselfOnHover;

impl Check for AGlyphOnlyTabNamesItselfOnHover {
    fn name(&self) -> &'static str {
        "a_glyph_only_tab_names_itself_on_hover"
    }

    fn defect(&self) -> &'static str {
        "a tab drawn as its glyph alone shows a hover tip that does not lead with its name, \
         so the operator cannot tell what the picture means (O300)"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = seed(ctx).and_then(|userdata| {
            let driven = launch_with(ctx, &mut report, &FIXTURE, "mode.edit", "tab-tips", &[])
                .and_then(|(session, pointer)| {
                    let outcome = drive(ctx, &mut report, &session, &pointer);
                    let parked = pointer.gone(&session);
                    drop(session);
                    match outcome? {
                        Some(why) => Ok(Some(why)),
                        None => parked.map(|_| None),
                    }
                });
            let _ = crate::sandbox::reset_prefs(&userdata);
            driven
        });
        match outcome {
            Ok(Some(why)) => report.fail(why),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Write the glyph-only preferences beside the binary; returns the folder.
fn seed(ctx: &CheckContext) -> Result<std::path::PathBuf> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let userdata = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata");
    crate::sandbox::write_prefs(&userdata, PREFS)
        .map_err(|e| Error::new(format!("could not write preferences: {e}")))?;
    Ok(userdata)
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let ui_rect = ui_rect_event(ctx)?;
    for (surface, id, region, name) in TABS {
        let (rect, _) = declared_in(&session.trace()?, ui_rect, region).ok_or_else(|| {
            Error::new(format!(
                "`{region}` was never declared. Trace: {}.",
                session.trace_path().display()
            ))
        })?;
        let mark = session.trace()?.mark();
        pointer.hover(session, WindowPoint::centre_of(rect))?;
        session.settle(60);
        // The shell writes `tab-tip` under its own prefix.
        let trace = shell_trace(session)?;
        let tip = trace
            .events(TIP)
            .filter(|l| l.lineno > mark)
            .filter(|l| l.get("surface") == Some(surface) && l.get("tab") == Some(id))
            .last();
        let Some(tip) = tip else {
            return Ok(Some(format!(
                "hovering the {surface} tab `{id}` drew no tip (no `{TIP}` line after the hover)"
            )));
        };
        let text = tip.get("text").unwrap_or_default();
        report.note(format!("{surface} tab `{id}` tip: {text:?}"));
        if !text.starts_with(name) {
            return Ok(Some(format!(
                "the {surface} tab `{id}`, drawn as a glyph alone, shows the tip {text:?}, which \
                 does not lead with its name `{name}`"
            )));
        }
    }
    Ok(None)
}
