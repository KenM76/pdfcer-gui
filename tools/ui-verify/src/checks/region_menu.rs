//! `the_region_menu_offers_every_region_verb` — the left rail's Select group
//! arms the Region tool, and a right-click inside the laid region offers
//! crop, both redacts and delete in Edit, and Delete in region from that menu
//! deletes what the region holds. In Read the same menu offers the copies and
//! none of the page-changing verbs: they are hidden, not greyed. Driven off
//! the desktop through the scripted pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/region_menu.md`.

use super::region_commands::{DELETE_REQUESTED, OVER_DRAWING, lay, selected};
use super::snapshot_box::{BOX_REGION, Rig, launch_with};
use crate::checks::driving::{declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::report::CheckReport;

const RAIL_REGION: &str = "rail.select.view.tool_snapshot"; // ui-text-exempt: a trace region name, never displayed
const MENU: &str = "canvas-menu"; // ui-text-exempt: a trace event name, never displayed
const CONTEXT: &str = "canvas.snapshot"; // ui-text-exempt: a menu context id, never displayed
const ITEMS: &str = "menu.item.canvas.snapshot."; // ui-text-exempt: a trace region name, never displayed
const COPY: &str = "menu.item.canvas.snapshot.edit.copy"; // ui-text-exempt: a trace region name, never displayed
const DELETE: &str = "menu.item.canvas.snapshot.edit.delete_in_region"; // ui-text-exempt: a trace region name, never displayed
/// The verbs that change the page, which Edit offers and Read withholds.
const EDIT_VERBS: [&str; 4] = [
    "menu.item.canvas.snapshot.pages.crop_to_region", // ui-text-exempt: a trace region name, never displayed
    "menu.item.canvas.snapshot.edit.redact_region", // ui-text-exempt: a trace region name, never displayed
    "menu.item.canvas.snapshot.edit.redact_outside_region", // ui-text-exempt: a trace region name, never displayed
    DELETE,
];

/// See the module documentation.
pub struct TheRegionMenuOffersEveryRegionVerb;

impl Check for TheRegionMenuOffersEveryRegionVerb {
    fn name(&self) -> &'static str {
        "the_region_menu_offers_every_region_verb"
    }

    fn defect(&self) -> &'static str {
        "the Region tool is not in the rail's Select group, or its right-click menu lacks crop, \
         redact or delete in Edit, or its Delete in region deletes a different number than the \
         region holds, or Read's menu shows the page-changing verbs"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = drive_edit(ctx, &mut report).and_then(|edit| match edit {
            Some(failure) => Ok(Some(failure)),
            None => drive_read(ctx, &mut report),
        });
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Right-click inside the laid region; the menu's drawn item names, or why
/// the right-click opened some other menu.
fn open_menu(rig: &Rig) -> Result<std::result::Result<Vec<String>, String>> {
    let inside = WindowPoint::centre_of(rig.region(BOX_REGION)?);
    rig.pointer.right_click(&rig.session, inside)?;
    rig.session.settle(15);
    let trace = rig.session.trace()?;
    let context = trace
        .last(MENU)
        .and_then(|l| l.get("context").map(str::to_owned));
    if context.as_deref() != Some(CONTEXT) {
        return Ok(Err(format!(
            "★ a right-click inside the region opened `{}`, not `{CONTEXT}`.",
            context.as_deref().unwrap_or("no menu")
        )));
    }
    Ok(Ok(declared_names(&trace, rig.ui_rect, ITEMS)))
}

/// Edit: arm from the rail, lay over the drawing, require every verb, delete.
fn drive_edit(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let invoke = [("PDFCER_DIAG_INVOKE".to_owned(), "mode.edit".to_owned())];
    let (rig, page) = launch_with(ctx, report, "region-menu-edit", &invoke)?;
    rig.session.settle(30);
    if declared_in(&rig.session.trace()?, rig.ui_rect, RAIL_REGION).is_none() {
        return Ok(Some(format!(
            "★ the left rail draws no `{RAIL_REGION}`. Select rows: {}.",
            list(&declared_names(
                &rig.session.trace()?,
                rig.ui_rect,
                "rail.select."
            ))
        )));
    }
    rig.click(RAIL_REGION)?;
    if let Err(why) = lay(ctx, &rig, page, OVER_DRAWING)? {
        return Ok(Some(format!("after arming from the rail: {why}")));
    }
    let Some(held) = selected(&rig, 1)?.filter(|&n| n > 0) else {
        return Ok(Some(
            "★★ a region over the drawing selected nothing in Edit.".to_owned(),
        ));
    };
    let items = match open_menu(&rig)? {
        Ok(items) => items,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!("Edit menu: {}", list(&items)));
    let missing: Vec<&str> = EDIT_VERBS
        .iter()
        .copied()
        .filter(|v| !items.iter().any(|i| i == v))
        .collect();
    if !missing.is_empty() {
        return Ok(Some(format!(
            "★★ in Edit the region's menu lacks {}. It drew: {}.",
            missing.join(", "),
            list(&items)
        )));
    }
    rig.click(DELETE)?;
    rig.session.settle(60);
    let Some(asked) = rig.session.trace()?.last(DELETE_REQUESTED).cloned() else {
        return Ok(Some(format!(
            "★★★ the menu's Delete in region was clicked and no `{DELETE_REQUESTED}` line followed."
        )));
    };
    report.note(asked.raw.clone());
    if asked.get("objects") != Some(held.to_string().as_str()) {
        return Ok(Some(format!(
            "★★★ the region held {held} objects and the menu's Delete in region asked for `{}`.",
            asked.raw
        )));
    }
    rig.pointer.gone(&rig.session)?;
    Ok(None)
}

/// Read: the copies are offered and none of the page-changing verbs is drawn.
fn drive_read(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let invoke = [(
        "PDFCER_DIAG_INVOKE".to_owned(),
        "mode.read,view.tool_snapshot".to_owned(),
    )];
    let (rig, page) = launch_with(ctx, report, "region-menu-read", &invoke)?;
    rig.session.settle(30);
    if let Err(why) = lay(ctx, &rig, page, OVER_DRAWING)? {
        return Ok(Some(format!("in Read: {why}")));
    }
    let items = match open_menu(&rig)? {
        Ok(items) => items,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!("Read menu: {}", list(&items)));
    if !items.iter().any(|i| i == COPY) {
        return Ok(Some(format!(
            "★ in Read the region's menu draws no `{COPY}`, so the absence of the Edit verbs \
             below proves nothing. It drew: {}.",
            list(&items)
        )));
    }
    let leaked: Vec<&String> = items
        .iter()
        .filter(|i| EDIT_VERBS.contains(&i.as_str()))
        .collect();
    if !leaked.is_empty() {
        return Ok(Some(format!(
            "★★ in Read the region's menu drew page-changing verbs: {leaked:?}."
        )));
    }
    rig.pointer.gone(&rig.session)?;
    Ok(None)
}
