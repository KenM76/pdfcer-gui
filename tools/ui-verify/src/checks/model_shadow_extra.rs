//! `checks::model_shadow_extra` — **every light of a scheme casts its own
//! shadow on the graphics card**
//!
//! Places `fixtures/floor-box.prc`, opens it with *View…*, sets *Shadows cast
//! by* to *The lighting* and the lighting to *Night* (three directional
//! lights), with *Every light* casting, the default.
//!
//! Oracle, from the `model-view-rendered` line (the viewer is an immediate
//! viewport no screenshot reaches): `lighting=Night caster=scheme
//! casters=every`, `gpu=1` with no `gpu-declined`, and `gpu-extra` at least
//! one — the casters past the key light drawn by the card's extra pass. The
//! sun caster before the switch is the control: it has no extras, so a
//! `gpu-extra` that never leaves 0 cannot pass.

use crate::checks::driving::{declared_in, repo_fixture};
use crate::checks::model_shadow_sun::{choose_scheme, open, rendered};
use crate::checks::model_view_window::{launch_with_model_file, ui_rect};
use crate::checks::viewer_options;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const MODEL: &str = "floor-box.prc";
const METHOD: &str = "Rebuild it with `python fixtures/floor-box.PROVENANCE.py <pdfcer.exe>`.";
/// `dialogs::model3d::lights::REGION_LIGHTING`.
const LIGHTING: &str = "model3d.lighting";
/// Its Night item: the region plus the scheme's PDF name in lower case.
const NIGHT: &str = "model3d.lighting.night";

/// See the module documentation.
pub struct EveryLightCastsOnTheCard;

impl Check for EveryLightCastsOnTheCard {
    fn name(&self) -> &'static str {
        "every_light_of_a_scheme_casts_on_the_card"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer drops the shadows of every light past the first, or draws them off the \
         graphics card"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = repo_fixture(MODEL, METHOD)
            .and_then(|model| {
                launch_with_model_file(ctx, &mut report, "shadow-extra", &model.to_string_lossy())
            })
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer));
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

const FIELDS: [&str; 6] = [
    "lighting",
    "caster",
    "casters",
    "gpu",
    "gpu-extra",
    "gpu-declined",
];

/// Pick Night in the Lighting list; `false` when it is drawn nowhere.
fn choose_night(session: &Session, pointer: &ScriptedPointer, ui_rect: &str) -> Result<bool> {
    let Some((combo, vp)) = viewer_options::reach(session, pointer, ui_rect, LIGHTING)? else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(combo))?;
    session.settle(20);
    let Some((item, vp)) = declared_in(&session.trace()?, ui_rect, NIGHT) else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(item))?;
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
    if let Some(failure) = open(ctx, session, pointer)? {
        return Ok(Some(failure));
    }
    let ui_rect = ui_rect(ctx)?;
    let Some(first) = rendered(session, FIELDS)? else {
        return Ok(Some("the viewer drew no picture line.".to_owned()));
    };
    report.note(format!("opened {FIELDS:?}: {first:?}"));
    if first[1] != "sun" || first[4] != "0" {
        return Ok(Some(format!(
            "the viewer opened with caster={} gpu-extra={}; expected the sun casting alone.",
            first[1], first[4]
        )));
    }
    if !choose_scheme(session, pointer, ui_rect)? {
        return Ok(Some(
            "no `model3d.shadow_light` drop-down with a The lighting item.".to_owned(),
        ));
    }
    if !choose_night(session, pointer, ui_rect)? {
        return Ok(Some(format!(
            "no `{LIGHTING}` drop-down with a `{NIGHT}` item."
        )));
    }
    let Some(night) = rendered(session, FIELDS)? else {
        return Ok(Some("no picture line after picking Night.".to_owned()));
    };
    report.note(format!("night: {night:?}"));
    let extras: usize = night[4].parse().unwrap_or(0);
    if night[..3] != ["Night", "scheme", "every"] || night[3] != "1" || night[5] != "-" {
        return Ok(Some(format!(
            "after picking Night cast by the lighting the picture drew lighting={} caster={} \
             casters={} gpu={} declined={}; expected Night, scheme, every, on the card.",
            night[0], night[1], night[2], night[3], night[5]
        )));
    }
    if extras == 0 {
        return Ok(Some(
            "Night's three lights cast, but the card drew no extra shadow (gpu-extra=0)."
                .to_owned(),
        ));
    }
    Ok(None)
}
