//! `checks::model_render_mode` — **the 3D viewer draws the render mode
//! picked in its options menu, and returns to the view's own**
//!
//! Places `fixtures/floor-box.prc`, opens it with *View…*, then in the
//! options menu sets *Draw as* to *Solid with edges*, then back to *As the
//! view says*.
//!
//! Oracles, from the `model-view-rendered` line (the viewer is an immediate
//! viewport no screenshot reaches):
//!
//! - Opened: `render-mode=Solid gpu=1`, no `gpu-declined`.
//! - Solid with edges: `render-mode=SolidWireframe gpu=1`, no
//!   `gpu-declined` (the card draws the edges), and a hash differing from
//!   the opened one: a mode named in the trace but whose edges are not
//!   drawn keeps the solid picture.
//! - Back to the view's: `render-mode=Solid gpu=1`, no `gpu-declined`.

use crate::checks::driving::{declared_in, repo_fixture};
use crate::checks::model_view_poster::click;
use crate::checks::model_view_window::{
    VIEW_REGION, launch_with_model_file, press_insert, ui_rect,
};
use crate::checks::viewer_options;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const MODEL: &str = "floor-box.prc";
const METHOD: &str = "Rebuild it with `python fixtures/floor-box.PROVENANCE.py <pdfcer.exe>`.";
/// `dialogs::model3d::looks::REGION_MODE`.
const MODE: &str = "model3d.render_mode";
/// Its *Solid with edges* item.
const EDGES: &str = "model3d.render_mode.SolidWireframe";
/// Its *As the view says* item.
const VIEWS: &str = "model3d.render_mode.view";

/// See the module documentation.
pub struct TheModelViewerDrawsAPickedMode;

impl Check for TheModelViewerDrawsAPickedMode {
    fn name(&self) -> &'static str {
        "the_3d_viewer_draws_a_picked_render_mode"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer's Draw as choice changes nothing in the picture, or leaves the graphics \
         card drawing a mode it cannot draw"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = repo_fixture(MODEL, METHOD)
            .and_then(|model| {
                launch_with_model_file(ctx, &mut report, "model-mode", &model.to_string_lossy())
            })
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer));
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The last `model-view-rendered` line's `render-mode`, `gpu`,
/// `gpu-declined` and `hash`; an absent field reads as `-`.
fn drawn(session: &Session) -> Result<Option<[String; 4]>> {
    let trace = session.trace()?;
    Ok(trace.events("model-view-rendered").last().map(|l| {
        ["render-mode", "gpu", "gpu-declined", "hash"].map(|k| l.get(k).unwrap_or("-").to_owned())
    }))
}

/// Choose `item` from the *Draw as* drop-down; `false` when the drop-down
/// or the item is drawn nowhere.
fn choose(session: &Session, pointer: &ScriptedPointer, ui_rect: &str, item: &str) -> Result<bool> {
    let Some((combo, vp)) = viewer_options::reach(session, pointer, ui_rect, MODE)? else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(combo))?;
    session.settle(20);
    let Some((rect, vp)) = declared_in(&session.trace()?, ui_rect, item) else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
    viewer_options::close(session, pointer, ui_rect)?;
    Ok(true)
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    pointer.gone(session)?;
    session.settle(10);
    if !press_insert(ctx, session, pointer)? {
        return Ok(Some("no 3D model button on the Edit tab.".to_owned()));
    }
    pointer.gone(session)?;
    session.settle(30);
    let ui_rect = ui_rect(ctx)?;
    if !click(session, pointer, ui_rect, VIEW_REGION)? {
        return Ok(Some(format!(
            "no `{VIEW_REGION}` region beside the placed model (look for `add-3d`)."
        )));
    }
    pointer.gone(session)?;
    let Some(first) = drawn(session)? else {
        return Ok(Some("the viewer drew no picture line.".to_owned()));
    };
    report.note(format!("opened [mode, gpu, declined, hash]: {first:?}"));
    if first[..3] != ["Solid", "1", "-"] {
        return Ok(Some(format!(
            "the viewer opened with render-mode={} gpu={} gpu-declined={}; expected Solid on \
             the graphics card.",
            first[0], first[1], first[2]
        )));
    }
    if !choose(session, pointer, ui_rect, EDGES)? {
        return Ok(Some(format!("no `{MODE}` drop-down or `{EDGES}` item.")));
    }
    let Some(edges) = drawn(session)? else {
        return Ok(Some(
            "no picture line after picking Solid with edges.".to_owned(),
        ));
    };
    report.note(format!("solid with edges: {edges:?}"));
    if edges[..3] != ["SolidWireframe", "1", "-"] || edges[3] == first[3] {
        return Ok(Some(format!(
            "Solid with edges drew render-mode={} gpu={} gpu-declined={} with hash {} against \
             {}; expected the graphics card drawing the edges into a new picture.",
            edges[0], edges[1], edges[2], edges[3], first[3]
        )));
    }
    if !choose(session, pointer, ui_rect, VIEWS)? {
        return Ok(Some(format!("no `{VIEWS}` item.")));
    }
    let Some(back) = drawn(session)? else {
        return Ok(Some(
            "no picture line after picking As the view says.".to_owned(),
        ));
    };
    report.note(format!("back to the view's: {back:?}"));
    if back[..3] != ["Solid", "1", "-"] {
        return Ok(Some(format!(
            "As the view says drew render-mode={} gpu={} gpu-declined={}; expected Solid back \
             on the graphics card.",
            back[0], back[1], back[2]
        )));
    }
    Ok(None)
}
