//! `checks::model_light` — the 3D viewer's brightness, aimed sun and lamps
//! drawn reach the picture.
//!
//! Each places a floor-box fixture, opens it with *View…*, and reads the
//! `model-view-rendered` line (the viewer is an immediate viewport no
//! screenshot reaches): `light-gain`, `caster`, the `sun-*` fields,
//! `lamp-cost`, `lamps=<in model>/<in view>/<lit>`, `gpu`, `gpu-declined`
//! and the picture `hash`. A changed `hash` is what shows a choice reached
//! the shader rather than only the trace.
//!
//! - [`BrightnessIsKeptPerLighting`]: the brightness slider pressed near its
//!   top raises `light-gain` past 2 and changes the picture; choosing *Day*
//!   reads 1 again, and choosing *Artwork* brings back the raised value. A
//!   single brightness for every scheme fails the middle step; one never
//!   kept fails the last.
//! - [`AnAimedSunCastsWhereItIsAimed`]: *A sun you aim* draws `caster=aimed`
//!   at bearing 45 and height 45, coloured by its height (4943 K), on the
//!   card; the sun disc pressed at its left rim lowers the sun under 10°,
//!   warms it under 3500 K and changes the picture. With the model turned,
//!   the disc pressed at its right rim aims the sun 90° round from the
//!   side the camera looks from (`sun-disc facing=`), that facing has
//!   turned at least 20° with the model, and the picture draws that sun.
//! - [`FewerLampsAreDrawnWhenAsked`]: `floor-lamps.prc`'s six lamps all
//!   reach the opening view and are lit (`lamps=6/6/6 lamp-cost=all`);
//!   *Brightest 4 at the view* draws `lamps=6/6/4 lamp-cost=4` on the card,
//!   and a different picture.

use crate::checks::driving::{declared_in, repo_fixture};
use crate::checks::model_shadow_sun::{open, rendered, turn};
use crate::checks::model_view_window::{launch_with_model_file, ui_rect};
use crate::checks::viewer_options;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::geom::{LRect, Pt};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const METHOD: &str =
    "Rebuild it with `python fixtures/floor-box.PROVENANCE.py <pdfcer.exe> [lamps]`.";
/// `dialogs::model3d::sunlight::REGION_BRIGHTNESS`.
const BRIGHTNESS: &str = "model3d.brightness";
/// `dialogs::model3d::lights::REGION_LIGHTING` and two of its items.
const LIGHTING: &str = "model3d.lighting";
const DAY: &str = "model3d.lighting.day";
const ARTWORK: &str = "model3d.lighting.artwork";
/// `dialogs::model3d::effects::REGION_SHADOW_LIGHT` and its aimed item.
const CASTER: &str = "model3d.shadow_light";
const AIMED: &str = "model3d.shadow_light.aimed";
/// `dialogs::model3d::sunlight::REGION_SUN_DISC`.
const SUN_DISC: &str = "model3d.sun.disc";
/// `dialogs::model3d::looks::REGION_LAMPS_DRAWN` and its four item.
const LAMPS_DRAWN: &str = "model3d.lamps_drawn";
const FOUR: &str = "model3d.lamps_drawn.4";
/// egui's slider rail, points from the slider's left edge: the value box
/// follows it inside the same rect.
const RAIL: f32 = 100.0;

/// Pick `item` from the drop-down `combo` in the options menu; `false` when
/// either is drawn nowhere.
fn choose(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    combo: &str,
    item: &str,
) -> Result<bool> {
    let Some((rect, vp)) = viewer_options::reach(session, pointer, ui_rect, combo)? else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(20);
    let Some((rect, vp)) = declared_in(&session.trace()?, ui_rect, item) else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
    viewer_options::close(session, pointer, ui_rect)?;
    Ok(true)
}

/// Press the sun disc at `(fx, fy)`, shares of its rect from the top left.
/// Press at mid-height: the menu scrolls only until most of the disc shows,
/// so its foot can sit under the scroll edge where a press reaches nothing;
/// `false` when it is drawn nowhere.
fn press_disc(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    (fx, fy): (f32, f32),
) -> Result<bool> {
    let Some((rect, vp)) = viewer_options::reach(session, pointer, ui_rect, SUN_DISC)? else {
        return Ok(false);
    };
    let at = Pt {
        x: rect.min.x + (rect.max.x - rect.min.x) * fx,
        y: rect.min.y + (rect.max.y - rect.min.y) * fy,
    };
    let point = WindowPoint::centre_of(LRect { min: at, max: at });
    pointer.click_in(session, vp.as_deref(), point)?;
    session.settle(30);
    viewer_options::close(session, pointer, ui_rect)?;
    Ok(true)
}

/// Press the slider `region` at `fraction` along its rail; `false` when it
/// is drawn nowhere.
fn press_slider(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    region: &str,
    fraction: f32,
) -> Result<bool> {
    let Some((rect, vp)) = viewer_options::reach(session, pointer, ui_rect, region)? else {
        return Ok(false);
    };
    let at = Pt {
        x: rect.min.x + RAIL.min(rect.max.x - rect.min.x) * fraction,
        y: (rect.min.y + rect.max.y) / 2.0,
    };
    let point = WindowPoint::centre_of(LRect { min: at, max: at });
    pointer.click_in(session, vp.as_deref(), point)?;
    session.settle(30);
    viewer_options::close(session, pointer, ui_rect)?;
    Ok(true)
}

/// Whether a number field reads above `limit`; an absent or unreadable
/// field is neither above nor below.
fn above(field: &str, limit: f64) -> bool {
    field.parse::<f64>().is_ok_and(|v| v > limit)
}

/// Whether a number field reads below `limit`.
fn below(field: &str, limit: f64) -> bool {
    field.parse::<f64>().is_ok_and(|v| v < limit)
}

/// Run `drive` on `model` and report it.
fn run_on(
    ctx: &CheckContext,
    mut report: CheckReport,
    model: &str,
    profile: &str,
    drive: fn(
        &CheckContext,
        &mut CheckReport,
        &Session,
        &ScriptedPointer,
    ) -> Result<Option<String>>,
) -> CheckReport {
    let driven = repo_fixture(model, METHOD)
        .and_then(|m| launch_with_model_file(ctx, &mut report, profile, &m.to_string_lossy()))
        .and_then(|(session, pointer)| {
            if let Some(failure) = open(ctx, &session, &pointer)? {
                return Ok(Some(failure));
            }
            drive(ctx, &mut report, &session, &pointer)
        });
    match driven {
        Ok(Some(failure)) => report.fail(failure),
        Ok(None) => report.pass(),
        Err(why) => report.from_error(&why),
    }
}

/// Whether a picture line is drawn on the card, undeclined.
fn on_card(gpu: &str, declined: &str) -> bool {
    gpu == "1" && declined == "-"
}

/// See the module documentation.
pub struct BrightnessIsKeptPerLighting;

impl Check for BrightnessIsKeptPerLighting {
    fn name(&self) -> &'static str {
        "the_3d_brightness_is_kept_for_each_lighting"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer's brightness slider leaves the picture as it was, or one brightness \
         serves every lighting choice instead of each keeping its own"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let report = CheckReport::new(self.name(), self.defect());
        run_on(ctx, report, "floor-box.prc", "brightness", brightness)
    }
}

const GAIN: [&str; 4] = ["light-gain", "gpu", "gpu-declined", "hash"];

fn brightness(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let ui_rect = ui_rect(ctx)?;
    let Some(first) = rendered(session, GAIN)? else {
        return Ok(Some("the viewer drew no picture line.".to_owned()));
    };
    report.note(format!("opened {GAIN:?}: {first:?}"));
    if first[0] != "1.000" {
        return Ok(Some(format!(
            "the viewer opened at light-gain={}; expected 1.000.",
            first[0]
        )));
    }
    if !press_slider(session, pointer, ui_rect, BRIGHTNESS, 0.95)? {
        return Ok(Some(format!(
            "no `{BRIGHTNESS}` slider in the options menu."
        )));
    }
    let Some(bright) = rendered(session, GAIN)? else {
        return Ok(Some(
            "no picture line after the brightness press.".to_owned(),
        ));
    };
    report.note(format!("brightness near its top: {bright:?}"));
    if !above(&bright[0], 2.0) || !on_card(&bright[1], &bright[2]) || bright[3] == first[3] {
        return Ok(Some(format!(
            "pressing the brightness slider near its top drew light-gain={} gpu={} declined={} \
             with hash {} against {}: the brightness did not reach the card's picture.",
            bright[0], bright[1], bright[2], bright[3], first[3]
        )));
    }
    if !choose(session, pointer, ui_rect, LIGHTING, DAY)? {
        return Ok(Some(format!("no `{DAY}` item under `{LIGHTING}`.")));
    }
    let Some(day) = rendered(session, GAIN)? else {
        return Ok(Some("no picture line after choosing Day.".to_owned()));
    };
    report.note(format!("Day: {day:?}"));
    if day[0] != "1.000" {
        return Ok(Some(format!(
            "Day lighting drew light-gain={} after only Artwork's brightness was raised: one \
             brightness serves every lighting.",
            day[0]
        )));
    }
    if !choose(session, pointer, ui_rect, LIGHTING, ARTWORK)? {
        return Ok(Some(format!("no `{ARTWORK}` item under `{LIGHTING}`.")));
    }
    let Some(back) = rendered(session, GAIN)? else {
        return Ok(Some("no picture line after choosing Artwork.".to_owned()));
    };
    report.note(format!("Artwork again: {back:?}"));
    if back[0] != bright[0] {
        return Ok(Some(format!(
            "Artwork came back at light-gain={} after it was raised to {}: its brightness was \
             not kept.",
            back[0], bright[0]
        )));
    }
    Ok(None)
}

/// See the module documentation.
pub struct AnAimedSunCastsWhereItIsAimed;

impl Check for AnAimedSunCastsWhereItIsAimed {
    fn name(&self) -> &'static str {
        "an_aimed_sun_lights_the_3d_model_where_it_is_aimed"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer's aimed sun is not drawn, or its height and colour do not change the \
         picture"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let report = CheckReport::new(self.name(), self.defect());
        run_on(ctx, report, "floor-box.prc", "aimed-sun", aimed_sun)
    }
}

const SUN: [&str; 7] = [
    "caster",
    "sun-bearing",
    "sun-height",
    "sun-kelvin",
    "gpu",
    "gpu-declined",
    "hash",
];

fn aimed_sun(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let ui_rect = ui_rect(ctx)?;
    let Some(first) = rendered(session, SUN)? else {
        return Ok(Some("the viewer drew no picture line.".to_owned()));
    };
    report.note(format!("opened {SUN:?}: {first:?}"));
    if first[0] != "sun" || first[1] != "-" {
        return Ok(Some(format!(
            "the viewer opened with caster={} sun-bearing={}; expected the fixed sun, unaimed.",
            first[0], first[1]
        )));
    }
    if !choose(session, pointer, ui_rect, CASTER, AIMED)? {
        return Ok(Some(format!("no `{AIMED}` item under `{CASTER}`.")));
    }
    let Some(aimed) = rendered(session, SUN)? else {
        return Ok(Some(
            "no picture line after choosing the aimed sun.".to_owned(),
        ));
    };
    report.note(format!("aimed: {aimed:?}"));
    if aimed[..4] != ["aimed", "45", "45", "4943"]
        || !on_card(&aimed[4], &aimed[5])
        || aimed[6] == first[6]
    {
        return Ok(Some(format!(
            "choosing the aimed sun drew caster={} bearing={} height={} kelvin={} gpu={} \
             declined={} with hash {} against {}; expected an aimed sun at 45°, 45° and \
             4943 K on the card, and a different picture.",
            aimed[0], aimed[1], aimed[2], aimed[3], aimed[4], aimed[5], aimed[6], first[6]
        )));
    }
    let mark = session.trace()?.mark();
    if !press_disc(session, pointer, ui_rect, (0.04, 0.5))? {
        return Ok(Some(format!("no `{SUN_DISC}` under the aimed sun.")));
    }
    let Some(before) = disc_line(session, mark)? else {
        return Ok(Some(
            "pressing the disc's left rim moved no sun.".to_owned(),
        ));
    };
    let Some(low) = rendered(session, SUN)? else {
        return Ok(Some("no picture line after lowering the sun.".to_owned()));
    };
    report.note(format!("lowered: {low:?}"));
    if !below(&low[2], 10.0) || !below(&low[3], 3500.0) || low[6] == aimed[6] {
        return Ok(Some(format!(
            "pressing the sun disc at its left rim drew height={} kelvin={} with hash \
             {} against {}; expected a sun under 10°, warmer than 3500 K, and a different \
             picture.",
            low[2], low[3], low[6], aimed[6]
        )));
    }
    turned_disc(report, session, (pointer, ui_rect), before[0])
}

/// The `facing`, `bearing` and `height` of the last `sun-disc` line after
/// `mark`; `None` when there is none or it does not read as numbers.
fn disc_line(session: &Session, mark: usize) -> Result<Option<[f64; 3]>> {
    let trace = session.trace()?;
    let Some(disc) = trace.last_after("sun-disc", mark) else {
        return Ok(None);
    };
    let field = |k: &str| disc.get(k).and_then(|v| v.parse::<f64>().ok());
    Ok(match (field("facing"), field("bearing"), field("height")) {
        (Some(f), Some(b), Some(h)) => Some([f, b, h]),
        _ => None,
    })
}

/// The signed difference `a - b` in degrees, in -180..180.
fn off(a: f64, b: f64) -> f64 {
    ((a - b).rem_euclid(360.0) + 180.0).rem_euclid(360.0) - 180.0
}

/// Turn the model, press the disc's right rim, and require the sun aimed
/// 90° round from a facing that turned with the model away from `before`.
fn turned_disc(
    report: &mut CheckReport,
    session: &Session,
    (pointer, ui_rect): (&ScriptedPointer, &str),
    before: f64,
) -> Result<Option<String>> {
    if turn(session, pointer, ui_rect)?.is_none() {
        return Ok(Some(
            "the picture could not be dragged to turn the model.".to_owned(),
        ));
    }
    let mark = session.trace()?.mark();
    if !press_disc(session, pointer, ui_rect, (0.98, 0.5))? {
        return Ok(Some(format!("no `{SUN_DISC}` after turning the model.")));
    }
    let Some([facing, bearing, height]) = disc_line(session, mark)? else {
        return Ok(Some(
            "pressing the disc's right rim moved no sun.".to_owned(),
        ));
    };
    let drawn = rendered(session, SUN)?.unwrap_or_else(|| SUN.map(|_| "-".to_owned()));
    report.note(format!(
        "turned: facing={facing} (was {before}) bearing={bearing} height={height}; \
         drawn {drawn:?}"
    ));
    let drawn_at = drawn[1].parse::<f64>().ok();
    if off(bearing, facing + 90.0).abs() > 1.5
        || off(facing, before).abs() < 20.0
        || height > 1.0
        || drawn_at.is_none_or(|b| off(b, bearing).abs() > 0.5)
    {
        return Ok(Some(format!(
            "after turning the model, the disc's right rim aimed bearing={bearing} \
             height={height} at facing={facing} (was {before}), drawn at bearing {}; \
             expected the facing turned at least 20° with the model, the sun 90° round \
             from it on the horizon, and that sun drawn.",
            drawn[1]
        )));
    }
    Ok(None)
}

/// See the module documentation.
pub struct FewerLampsAreDrawnWhenAsked;

impl Check for FewerLampsAreDrawnWhenAsked {
    fn name(&self) -> &'static str {
        "the_3d_viewer_draws_fewer_lamps_when_asked"
    }

    fn defect(&self) -> &'static str {
        "choosing to draw only a model's brightest few lamps still draws every lamp, or sends \
         the 3D picture off the graphics card"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let report = CheckReport::new(self.name(), self.defect());
        run_on(ctx, report, "floor-lamps.prc", "lamps-drawn", lamps_drawn)
    }
}

const LAMPS: [&str; 5] = ["lamps", "lamp-cost", "gpu", "gpu-declined", "hash"];

fn lamps_drawn(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let ui_rect = ui_rect(ctx)?;
    let Some(first) = rendered(session, LAMPS)? else {
        return Ok(Some("the viewer drew no picture line.".to_owned()));
    };
    report.note(format!("opened {LAMPS:?}: {first:?}"));
    if first[..2] != ["6/6/6", "all"] {
        return Ok(Some(format!(
            "a model with six lamps in view opened with lamps={} lamp-cost={}; expected all \
             six lit.",
            first[0], first[1]
        )));
    }
    if !choose(session, pointer, ui_rect, LAMPS_DRAWN, FOUR)? {
        return Ok(Some(format!("no `{FOUR}` item under `{LAMPS_DRAWN}`.")));
    }
    let Some(four) = rendered(session, LAMPS)? else {
        return Ok(Some("no picture line after the pick.".to_owned()));
    };
    report.note(format!("brightest four: {four:?}"));
    if four[..2] != ["6/6/4", "4"] || !on_card(&four[2], &four[3]) || four[4] == first[4] {
        return Ok(Some(format!(
            "drawing the brightest four drew lamps={} lamp-cost={} gpu={} declined={} with \
             hash {} against {}: the choice did not reach the card's picture.",
            four[0], four[1], four[2], four[3], four[4], first[4]
        )));
    }
    Ok(None)
}
