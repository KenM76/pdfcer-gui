//! `checks::model_occlusion` — **Insert 3D model's occlusion switch reaches
//! the glTF conversion**
//!
//! Places a one-quad glTF whose material carries an occlusion map over a base
//! colour factor, twice: with the window's defaults, and with *Darken creases
//! from the occlusion map* cleared. The engine's converted scene is the oracle,
//! not the switch's echo. Baked, `model-insert-gltf` must read
//! `occlusion=Bake` with one textured surface (the map becomes a texture of
//! the base colour) and the status row must say `occlusion texture baked into`.
//! Cleared, it must read `occlusion=Drop textured=0` and say
//! `occlusion texture not drawn, as asked`.

use crate::checks::model_view_window::{launch_with_model_env, press_insert_with};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;

/// The occlusion switch (`dialogs::insert_model::REGION_OCCLUSION`).
const OCCLUSION: &str = "insert-model.gltf.occlusion";
/// One unit quad, orange by factor, with a 2x2 checker as its occlusion map.
const AGED_QUAD: &str = include_str!("model_occlusion.gltf");

/// See the module documentation.
pub struct TheOcclusionSwitchReachesTheConversion;

impl Check for TheOcclusionSwitchReachesTheConversion {
    fn name(&self) -> &'static str {
        "insert_3d_model_occlusion_reaches_the_conversion"
    }

    fn defect(&self) -> &'static str {
        "Insert 3D model's occlusion switch does not decide whether a glTF occlusion map is \
         baked into the colour or left out, or the status row does not say which"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = match occlusion(ctx, &mut report, true) {
            Ok(None) => occlusion(ctx, &mut report, false),
            other => other,
        };
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Place [`AGED_QUAD`] with the map baked (the default) or dropped, and
/// judge the conversion and its disclosure.
fn occlusion(ctx: &CheckContext, report: &mut CheckReport, bake: bool) -> Result<Option<String>> {
    let stem = if bake {
        "occlusion-bake"
    } else {
        "occlusion-drop"
    }; // ui-text-exempt: file stems
    let model = ctx.out(&format!("{stem}.gltf"));
    std::fs::write(&model, AGED_QUAD)
        .map_err(|e| crate::error::Error::new(format!("writing the model: {e}")))?;
    let (session, pointer) =
        launch_with_model_env(ctx, report, stem, &model.to_string_lossy(), &[])?;
    pointer.gone(&session)?;
    session.settle(10);
    let choices: &[&str] = if bake { &[] } else { &[OCCLUSION] };
    if !press_insert_with(ctx, &session, &pointer, choices)? {
        return Ok(Some("no 3D model button on the Edit tab.".to_owned()));
    }
    session.settle(30);
    let trace = session.trace()?;
    let last = |event: &str| trace.events(event).last().map(|l| l.raw.clone());
    let (placed, added) = (last("insert-model-placed"), last("add-3d"));
    report.note(format!("{stem} placed: {placed:?}"));
    report.note(format!("{stem} converted: {:?}", last("model-insert-gltf")));
    report.note(format!("{stem} add-3d: {added:?}"));
    let Some(placed) = placed else {
        return Ok(Some(format!(
            "{stem}: no `insert-model-placed`: Place was not reached."
        )));
    };
    if !placed.contains(&format!(" occlusion={bake}")) {
        return Ok(Some(format!(
            "★★★ {stem}: the window placed `{placed}`; occlusion={bake} was asked for."
        )));
    }
    let (mode, textured, said) = if bake {
        ("Bake", 1, "occlusion texture baked into")
    } else {
        ("Drop", 0, "occlusion texture not drawn, as asked")
    };
    let Some(gltf) = trace.events("model-insert-gltf").last() else {
        return Ok(Some(format!("{stem}: the glTF was not converted.")));
    };
    if gltf.get("occlusion") != Some(mode) || gltf.get("textured").and_then(|n| n.parse::<u32>().ok()) != Some(textured) {
        return Ok(Some(format!(
            "★★★ {stem}: the conversion read `{}`; occlusion={mode} textured={textured} is \
             the scene the switch asks for.",
            gltf.raw
        )));
    }
    match added {
        Some(line) if line.contains(said) => Ok(None),
        other => Ok(Some(format!(
            "★★★ {stem}: the status row's sentences `{other:?}` do not say `{said}`."
        ))),
    }
}
