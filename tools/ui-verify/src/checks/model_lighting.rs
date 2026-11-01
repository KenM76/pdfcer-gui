//! `checks::model_lighting` — **the 3D viewer's Lighting list lights the
//! live picture, and its switch turns the lights with the view or not**
//!
//! Places the engine corpus's `coloured.prc` from the ribbon, as
//! `checks::model_colours` does, opens it with *View…*, then in the options
//! menu picks *Night lights* and clears *Lights turn with the view*.
//!
//! Oracles, from the `model-view-rendered` line (the viewer is an immediate
//! viewport no screenshot reaches): before, `lighting=Artwork lit-by=default`;
//! after the pick, `lighting=Night lit-by=chosen` with a different picture
//! hash; after the switch, `lights-turn=0` with a hash different again. A
//! viewer that lists schemes but draws with one light keeps the first hash,
//! which neither later assertion accepts.

use crate::checks::driving::declared_in;
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

/// A part-coloured PRC model from the engine corpus.
const MODEL: &str = "D:/Dev/pdfcer/fixtures/synthetic/prc/coloured.prc";
/// `dialogs::model3d::lights::REGION_LIGHTING`.
const LIGHTING: &str = "model3d.lighting";
/// Its Night item: the region plus the scheme's PDF name in lower case.
const NIGHT: &str = "model3d.lighting.night";
/// `dialogs::model3d::lights::REGION_LIGHT_FRAME`.
const LIGHT_FRAME: &str = "model3d.light_frame";

/// See the module documentation.
pub struct TheModelViewerIsLitAsChosen;

impl Check for TheModelViewerIsLitAsChosen {
    fn name(&self) -> &'static str {
        "the_3d_viewer_is_lit_as_chosen"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer's Lighting list changes nothing in the picture, or its lights cannot be \
         fixed to the model"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch_with_model_file(ctx, &mut report, "model-lighting", MODEL)
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer));
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The last `model-view-rendered` line's lighting, source, switch and hash.
fn lit(session: &Session) -> Result<Option<[String; 4]>> {
    let trace = session.trace()?;
    Ok(trace.events("model-view-rendered").last().and_then(|l| {
        Some([
            l.get("lighting")?.to_owned(),
            l.get("lit-by")?.to_owned(),
            l.get("lights-turn")?.to_owned(),
            l.get("hash")?.to_owned(),
        ])
    }))
}

/// Click `name` where it is declared now; `false` when it is not.
fn click(session: &Session, pointer: &ScriptedPointer, ui_rect: &str, name: &str) -> Result<bool> {
    let Some((rect, vp)) = declared_in(&session.trace()?, ui_rect, name) else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
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
    let Some(first) = lit(session)? else {
        return Ok(Some(
            "the viewer's picture line names no lighting (`lighting=`, `lit-by=`).".to_owned(),
        ));
    };
    report.note(format!("opened: {first:?}"));
    if first[0] != "Artwork" || first[1] != "default" {
        return Ok(Some(format!(
            "a model whose file asks for no lighting opened lit by {} ({}); expected the \
             default.",
            first[0], first[1]
        )));
    }
    let Some((combo, vp)) = viewer_options::reach(session, pointer, ui_rect, LIGHTING)? else {
        return Ok(Some(format!(
            "no `{LIGHTING}` drop-down in the options menu."
        )));
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(combo))?;
    session.settle(20);
    if !click(session, pointer, ui_rect, NIGHT)? {
        return Ok(Some(format!("the Lighting list has no `{NIGHT}` item.")));
    }
    let Some(night) = lit(session)? else {
        return Ok(Some(
            "no picture line after picking Night lights.".to_owned(),
        ));
    };
    report.note(format!("night: {night:?}"));
    if night[0] != "Night" || night[1] != "chosen" || night[3] == first[3] {
        return Ok(Some(format!(
            "after picking Night lights the picture is lit by {} ({}), hash {} against {} \
             before: the choice did not light it.",
            night[0], night[1], night[3], first[3]
        )));
    }
    if !click(session, pointer, ui_rect, LIGHT_FRAME)? {
        return Ok(Some(format!(
            "no `{LIGHT_FRAME}` switch in the options menu."
        )));
    }
    viewer_options::close(session, pointer, ui_rect)?;
    let Some(fixed) = lit(session)? else {
        return Ok(Some("no picture line after the lights switch.".to_owned()));
    };
    report.note(format!("fixed to the model: {fixed:?}"));
    if fixed[2] != "0" || fixed[3] == night[3] {
        return Ok(Some(format!(
            "clearing *Lights turn with the view* drew lights-turn={} with hash {} against {}: \
             the lights did not move to the model's frame.",
            fixed[2], fixed[3], night[3]
        )));
    }
    Ok(None)
}
