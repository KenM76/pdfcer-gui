//! `checks::model_shadows` — **the 3D viewer draws shadows and ambient
//! occlusion on the graphics card, and each has a switch that turns it off**
//!
//! Places `fixtures/floor-box.prc`, a cube standing on a floor, from the
//! ribbon as `checks::model_lighting` does, opens it with *View…*, then in the
//! options menu clears *Shadows*, then *Shade corners and creases*.
//!
//! Oracles, from the `model-view-rendered` line (the viewer is an immediate
//! viewport no screenshot reaches): opened, `gpu=1 shadows=1
//! shadow-light=-1 ao=1` (the default lighting is all at the camera, so the
//! fixed key light casts), the shaders having linked; after the first
//! switch, `shadows=0 ao=1` with a different picture hash; after the second,
//! `ao=0` with a hash different again. A GPU path that names the effects in
//! the trace but never draws them keeps the hash, which neither later
//! assertion accepts.

use crate::checks::driving::repo_fixture;
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

/// A cube on a floor: it shadows the floor and creases at its foot.
const MODEL: &str = "floor-box.prc";
const METHOD: &str = "Rebuild it with `python fixtures/floor-box.PROVENANCE.py <pdfcer.exe>`.";
/// `dialogs::model3d::effects::REGION_SHADOWS`.
const SHADOWS: &str = "model3d.shadows";
/// `dialogs::model3d::effects::REGION_OCCLUSION`.
const OCCLUSION: &str = "model3d.occlusion";

/// See the module documentation.
pub struct TheModelViewerCastsShadows;

impl Check for TheModelViewerCastsShadows {
    fn name(&self) -> &'static str {
        "the_3d_viewer_casts_shadows"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer draws no shadows or shaded corners on the graphics card, or its switches \
         for them change nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = repo_fixture(MODEL, METHOD)
            .and_then(|model| {
                launch_with_model_file(ctx, &mut report, "model-shadows", &model.to_string_lossy())
            })
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer));
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The last `model-view-rendered` line's `gpu`, `shadows`, `shadow-light`,
/// `ao` and `hash`; an absent field reads as `-`.
fn drawn(session: &Session) -> Result<Option<[String; 5]>> {
    let trace = session.trace()?;
    Ok(trace.events("model-view-rendered").last().map(|l| {
        ["gpu", "shadows", "shadow-light", "ao", "hash"].map(|k| l.get(k).unwrap_or("-").to_owned())
    }))
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
    report.note(format!("opened [gpu, shadows, light, ao, hash]: {first:?}"));
    if first[..4] != ["1", "1", "-1", "1"] {
        return Ok(Some(format!(
            "the viewer opened with gpu={} shadows={} shadow-light={} ao={}; expected the \
             graphics card drawing both, cast by the fixed key light (a refused shader drops \
             to the CPU and loses `gpu=1`; look for `model-view-gpu failed`).",
            first[0], first[1], first[2], first[3]
        )));
    }
    let Some((switch, vp)) = viewer_options::reach(session, pointer, ui_rect, SHADOWS)? else {
        return Ok(Some(format!("no `{SHADOWS}` switch in the options menu.")));
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(switch))?;
    session.settle(30);
    let Some(unshadowed) = drawn(session)? else {
        return Ok(Some("no picture line after clearing Shadows.".to_owned()));
    };
    report.note(format!("shadows off: {unshadowed:?}"));
    if unshadowed[1] != "0" || unshadowed[3] != "1" || unshadowed[4] == first[4] {
        return Ok(Some(format!(
            "clearing Shadows drew shadows={} ao={} with hash {} against {}: the switch did \
             not take the shadows out of the picture.",
            unshadowed[1], unshadowed[3], unshadowed[4], first[4]
        )));
    }
    if !click(session, pointer, ui_rect, OCCLUSION)? {
        return Ok(Some(format!(
            "no `{OCCLUSION}` switch in the options menu."
        )));
    }
    viewer_options::close(session, pointer, ui_rect)?;
    let Some(flat) = drawn(session)? else {
        return Ok(Some(
            "no picture line after clearing the corner shading.".to_owned(),
        ));
    };
    report.note(format!("both off: {flat:?}"));
    if flat[3] != "0" || flat[4] == unshadowed[4] {
        return Ok(Some(format!(
            "clearing Shade corners and creases drew ao={} with hash {} against {}: the \
             switch did not take the occlusion out of the picture.",
            flat[3], flat[4], unshadowed[4]
        )));
    }
    Ok(None)
}
