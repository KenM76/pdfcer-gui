//! `ribbon_and_dock_tabs_follow_their_own_face_setting` — the ribbon's and
//! the dock's tabs each show picture, name or both as their own preference
//! says, independently (O299). See `docs/modules/ui-verify/checks/tab_faces.md`.

use super::dimdrive::{Fixture, launch_with, ui_rect_event};
use super::driving::{declared, live_names};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::{Error, Result};
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "a1-titleblock.pdf",
    method: "It is checked in under fixtures/.",
    page: PageGeometry {
        width_pt: 2383.94,
        height_pt: 1683.78,
    },
};

/// A picture-only ribbon tab is at most this fraction of a named one's width.
const NARROWER: f32 = 0.8;

/// See the module documentation.
pub struct RibbonAndDockTabsFollowTheirOwnFaceSetting;

impl Check for RibbonAndDockTabsFollowTheirOwnFaceSetting {
    fn name(&self) -> &'static str {
        "ribbon_and_dock_tabs_follow_their_own_face_setting"
    }

    fn defect(&self) -> &'static str {
        "The tab-face settings do not reach the tabs: picture-only ribbon tabs are as wide as \
         named ones or lose their picture, or name-only panel tabs still draw a picture"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = drive(ctx, &mut report);
        if let Some(dir) = ctx.resolve_exe().as_deref().and_then(|e| e.parent()) {
            let _ = crate::sandbox::reset_prefs(&dir.join("userdata"));
        }
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// What one launch drew.
#[derive(Debug)]
struct Bars {
    ribbon_tabs: usize,
    ribbon_mean_width: f32,
    ribbon_icons: usize,
    dock_tabs: usize,
    dock_icons: usize,
}

fn measure(ctx: &CheckContext, report: &mut CheckReport, prefs: &str, stem: &str) -> Result<Bars> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let dir = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata");
    crate::sandbox::write_prefs(&dir, prefs)
        .map_err(|e| Error::new(format!("could not write preferences: {e}")))?;
    let (session, pointer) = launch_with(ctx, report, &FIXTURE, "view.reset_layout", stem, &[])?;
    session.settle(30);
    let trace = session.trace()?;
    let ui_rect = ui_rect_event(ctx)?;
    let widths: Vec<f32> = live_names(&trace, ui_rect, "ribbon.tab.")
        .iter()
        .filter_map(|n| declared(&trace, ui_rect, n))
        .map(|r| r.width())
        .collect();
    let bars = Bars {
        ribbon_tabs: widths.len(),
        ribbon_mean_width: widths.iter().sum::<f32>() / widths.len().max(1) as f32,
        ribbon_icons: live_names(&trace, ui_rect, "tabicon.ribbon.").len(),
        dock_tabs: live_names(&trace, ui_rect, "dock.tab.").len(),
        dock_icons: live_names(&trace, ui_rect, "tabicon.dock.").len(),
    };
    pointer.gone(&session)?;
    report.note(format!("{stem}: {bars:?}"));
    Ok(bars)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let both = measure(ctx, report, "", "tab-faces-both")?;
    let mixed = measure(
        ctx,
        report,
        "ribbon_tabs_show = glyph\ndock_tabs_show = text\n",
        "tab-faces-mixed",
    )?;
    if both.ribbon_tabs == 0 || both.dock_tabs == 0 || both.ribbon_icons == 0 {
        return Ok(Some(format!(
            "the default launch drew {both:?}: no ribbon tabs, dock tabs or ribbon glyphs to \
             compare against."
        )));
    }
    let mut failures = Vec::new();
    if mixed.ribbon_icons == 0 {
        failures.push("★ picture-only ribbon tabs drew no picture".to_owned());
    }
    if mixed.ribbon_mean_width > both.ribbon_mean_width * NARROWER {
        failures.push(format!(
            "★ picture-only ribbon tabs average {:.1} pt against {:.1} pt named: the name is \
             still being laid out",
            mixed.ribbon_mean_width, both.ribbon_mean_width
        ));
    }
    if mixed.dock_tabs == 0 {
        failures.push("★ name-only panel tabs were not drawn at all".to_owned());
    }
    if mixed.dock_icons > 0 {
        failures.push(format!(
            "★★ name-only panel tabs still drew {} pictures",
            mixed.dock_icons
        ));
    }
    Ok((!failures.is_empty()).then(|| failures.join("; ")))
}
