//! `the_inspector_is_one_tab_bar` — Edit's right dock opens as ONE tab bar,
//! Properties showing and Objects a tab beside it, with the document's own
//! metadata kept out of the selection inspector and object rows that fit.
//! See `docs/modules/ui-verify/checks/inspector.md`.

use super::dimdrive::{Fixture, press, run_on, ui_rect_event};
use super::driving::{declared, declared_since, list, live_names};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::image::Image;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "a1-titleblock.pdf",
    method: "It is checked in under fixtures/; its object rows are long and carry \
             subset-tagged font names, so a too-narrow pane shortens them.",
    page: PageGeometry {
        width_pt: 2383.94,
        height_pt: 1683.78,
    },
};

const OBJECTS_TAB: &str = "dock.tab.view.panel_objects"; // ui-text-exempt: a trace region, never displayed
const OBJECTS_BODY: &str = "dock.body.view.panel_objects"; // ui-text-exempt: a trace region, never displayed
const PROPERTIES_BODY: &str = "dock.body.file.properties"; // ui-text-exempt: a trace region, never displayed
const DOC_METADATA: &str = "properties.info"; // ui-text-exempt: a trace region, never displayed
const ROWS_EVENT: &str = "objects-rows"; // ui-text-exempt: a trace event, never displayed
/// The width of the Objects pane's right edge sampled for ink, in points.
const EDGE_STRIP_PTS: f32 = 8.0;

/// See the module documentation.
pub struct TheInspectorIsOneTabBar;

impl Check for TheInspectorIsOneTabBar {
    fn name(&self) -> &'static str {
        "the_inspector_is_one_tab_bar"
    }

    fn defect(&self) -> &'static str {
        "Edit's right dock opens as more than one tab bar, or not on the selection's \
         properties, or without the object list one tab away, or with the document's metadata \
         inside the selection inspector, or with object rows cut short"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &FIXTURE,
            "mode.edit,view.reset_layout",
            "inspector",
            drive,
        )
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    session.settle(30);
    let trace = session.trace()?;
    if !trace
        .events("layout-reset")
        .last()
        .is_some_and(|l| l.raw.contains("changed="))
    {
        return Ok(Some(
            "the layout reset did not land: every measurement would be of a layout an \
             earlier run saved."
                .to_owned(),
        ));
    }
    let ui_rect = ui_rect_event(ctx)?;
    let bars: Vec<String> = live_names(&trace, ui_rect, "dock.right.")
        .into_iter()
        .filter(|n| n.ends_with(".tabbar"))
        .collect();
    report.note(format!("right-side tab bars: {}", list(&bars)));
    if bars.len() != 1 {
        return Ok(Some(format!(
            "★★★ Edit's right dock drew {} tab bars ({}), not one.",
            bars.len(),
            list(&bars)
        )));
    }
    if let Some(failure) = layout_failure(&trace, ui_rect, &bars[0]) {
        return Ok(Some(failure));
    }
    objects_fit(ctx, report, session, pointer)
}

/// Properties showing, Objects a tab on `bar`, and the metadata form outside
/// the selection inspector. Later tabs may sit behind the bar's "more" button
/// at the default width, so only Objects is required on the bar itself.
fn layout_failure(trace: &crate::trace::Trace, ui_rect: &str, bar: &str) -> Option<String> {
    let tabs = live_names(trace, ui_rect, "dock.tab.");
    let Some(properties) = declared(trace, ui_rect, PROPERTIES_BODY) else {
        return Some(format!(
            "★★★ Edit opened without the selection's properties on screen; tabs: {}.",
            list(&tabs)
        ));
    };
    let bar_rect = declared(trace, ui_rect, bar)?;
    let on_bar = |tab: &str| {
        declared(trace, ui_rect, tab)
            .map(|r| (r.min.y + r.max.y) / 2.0)
            .is_some_and(|y| y >= bar_rect.min.y && y <= bar_rect.max.y)
    };
    if !on_bar(OBJECTS_TAB) {
        return Some(format!(
            "★★ `{OBJECTS_TAB}` is not a tab on `{bar}`; tabs drawn: {}.",
            list(&tabs)
        ));
    }
    declared(trace, ui_rect, DOC_METADATA)
        .filter(|m| properties.contains_rect(*m))
        .map(|m| {
            format!(
                "★★★ the document's own metadata `{DOC_METADATA}` drew at {m:?}, inside the \
                 selection inspector at {properties:?}."
            )
        })
}

/// Open the Objects tab and require every drawn row to fit, by the panel's
/// own arithmetic and by the pixels at the pane's right edge.
fn objects_fit(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, OBJECTS_TAB)?;
    session.settle(30);
    let trace = session.trace()?;
    let ui_rect = ui_rect_event(ctx)?;
    let Some(objects) = declared_since(&trace, ui_rect, OBJECTS_BODY, mark) else {
        return Ok(Some(format!(
            "★★ pressing `{OBJECTS_TAB}` did not bring the object list on screen."
        )));
    };
    let Some(rows) = trace.last_after(ROWS_EVENT, mark) else {
        return Ok(Some(format!(
            "★★ the object list drew no `{ROWS_EVENT}` line on a fixture with many objects."
        )));
    };
    let count = |key: &str| -> Result<usize> {
        rows.get(key)
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| Error::new(format!("`{ROWS_EVENT}` carries no `{key}=`")))
    };
    let (visible, elided) = (count("visible")?, count("elided")?);
    report.note(format!(
        "{visible} rows drawn, {elided} shortened; pane={} pt",
        rows.get("pane").unwrap_or("?")
    ));
    if visible == 0 {
        return Err(Error::new(
            "the object list drew zero rows, so there is no width to judge.",
        ));
    }
    if elided > 0 {
        return Ok(Some(format!(
            "★★★ {elided} of {visible} object rows do not fit: pane={} pt, widest shortened \
             row overflow={} pt.",
            rows.get("pane").unwrap_or("?"),
            rows.get("overflow").unwrap_or("?")
        )));
    }
    let path = ctx.out("inspector.objects.png");
    pointer.screenshot(session, &path)?;
    report.artifact(path.clone());
    let image = Image::load_png(&path)?;
    let strip = LRect::new(
        Pt::new(objects.max.x - EDGE_STRIP_PTS, objects.min.y),
        Pt::new(objects.max.x, objects.max.y),
    );
    let ink =
        crate::pixels::ink_run_into(&image, session.frame()?.logical_to_capture_pixels(strip));
    if ink.is_text() {
        return Ok(Some(format!(
            "★★★ ink runs into the object pane's right edge ({}) though no row reported \
             shortening. Picture: {}.",
            ink.summary(),
            path.display()
        )));
    }
    Ok(None)
}
