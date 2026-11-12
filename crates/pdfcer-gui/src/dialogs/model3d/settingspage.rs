//! The Settings window's 3D viewer page: how a viewer starts, the eye
//! height, and every setting *Save as default* writes, edited as the
//! preferences' `model3d_` keys (see [`super::memory`]). A key the file
//! lacks shows [`memory::factory`]'s value and is written only once changed;
//! Save writes them with the window's other settings and Restore defaults
//! empties them, as it does every other page.
//!
//! The page is a table of [`Row`]s, one per key, so a key the viewer keeps
//! and the page cannot reach fails a test rather than going unnoticed.

use std::ops::RangeInclusive;

use egui::Ui;
use pdfcer_3d_nav::AUTO_RANGE;
use pdfcer_gui_base::settingspages::widgets;

use super::looks::{LAMP_NEAREST, lamp_cut_text, nearest_text};
use super::memory::{self, HIGHLIGHTS, SEE_THROUGH, SENSES, Settings};
use super::sunlight::{GAIN_RANGE, KELVIN_RANGE, aim_row};
use crate::app::prefs::Prefs;
use crate::text::model3d as m;
use crate::text::model3dlens as l;
use crate::text::model3dlight as g;
use crate::text::model3dlook::{self as k, CardPart};
use crate::text::model3dshot as s;
use crate::text::panels::modelnav as t;

/// What one key holds.
enum Kind {
    Flag,
    /// An index into these choices.
    Pick(Vec<String>),
    /// A number in `range`, written with `decimals` places; `percent` shows
    /// it times 100.
    Number {
        range: RangeInclusive<f64>,
        decimals: usize,
        percent: bool,
    },
    /// The sun disc, holding this key's bearing and `height`'s height.
    SunDisc {
        height: &'static str,
    },
    /// Drawn by the row before it.
    Within,
}

/// One key, without the `model3d_` prefix, and its name on the page.
struct Row {
    key: String,
    label: String,
    kind: Kind,
}

fn flag(key: &str, label: &str) -> Row {
    Row {
        key: key.to_owned(),
        label: label.to_owned(),
        kind: Kind::Flag,
    }
}

fn pick(key: &str, label: &str, choices: impl IntoIterator<Item = impl Into<String>>) -> Row {
    Row {
        key: key.to_owned(),
        label: label.to_owned(),
        kind: Kind::Pick(choices.into_iter().map(Into::into).collect()),
    }
}

fn number(key: &str, label: &str, range: RangeInclusive<f64>, decimals: usize) -> Row {
    Row {
        key: key.to_owned(),
        label: label.to_owned(),
        kind: Kind::Number {
            range,
            decimals,
            percent: false,
        },
    }
}

fn percent(key: &str, label: &str) -> Row {
    Row {
        kind: Kind::Number {
            range: GAIN_RANGE,
            decimals: 3,
            percent: true,
        },
        ..number(key, label, GAIN_RANGE, 3)
    }
}

/// The View tab's lens and the Moving tab's switches.
fn view_rows() -> Vec<Row> {
    vec![
        pick("lens", l::lens_label(), l::lens_names()),
        pick("lens_curve", l::curve_label(), l::curve_names()),
        number("lens_auto_at", l::auto_at_label(), AUTO_RANGE, 0),
        flag("ghost", t::ghost()),
        flag("level", t::level()),
        flag("gimbal", t::gimbal()),
        flag("stairs", t::stairs()),
        flag("behind", t::behind()),
        flag("controls", t::controls()),
        number("speed", t::speed(), 0.25..=4.0, 3),
    ]
}

/// The graphics card's switches.
fn card_rows() -> Vec<Row> {
    let mut rows = vec![flag("card_picture", k::CARD_PICTURE)];
    for (i, part) in CardPart::ALL.into_iter().enumerate() {
        rows.push(flag(&format!("card_part_{i}"), k::card_part(part)));
    }
    rows
}

/// The Look tab's rows, under `prefix` (`look_`, or `shot_look_` for the
/// picture's own look).
fn look_rows(prefix: &str) -> Vec<Row> {
    let lightings = memory::lightings()
        .into_iter()
        .map(|l| l.map_or(m::lighting_unset(), m::lighting));
    let modes = memory::modes()
        .into_iter()
        .map(|mode| mode.map_or(s::MODE_FROM_VIEW, |m| k::mode(m.as_name())));
    let mut rows = vec![
        pick("lighting", m::lighting_label(), lightings),
        flag("lights_turn", m::lights_turn()),
        flag("shadows", m::shadows()),
        flag("sun", m::shadow_light(true)),
        flag("sun_aimed", g::aimed_sun()),
        Row {
            kind: Kind::SunDisc {
                height: "sun_height", // ui-text-exempt: a preferences key, never displayed
            },
            ..flag("sun_bearing", g::sun_bearing_label())
        },
        Row {
            kind: Kind::Within,
            ..flag("sun_height", g::sun_height_label())
        },
        flag("sun_by_height", g::sun_colour_by_height()),
        number("sun_kelvin", g::sun_kelvin_label(), KELVIN_RANGE, 0),
        percent("sun_strength", g::sun_strength_label()),
        pick(
            "light_sense",
            m::light_sense_label(),
            SENSES.map(|x| m::light_sense(x == pdfcer_3d::LightSense::Emitted)),
        ),
        flag("occlusion", m::occlusion()),
        pick("mode", k::mode_label(), modes),
        // A flag, so its two choices in the order 0, 1.
        pick(
            "every_light",
            k::casters_label(),
            [k::casters(false), k::casters(true)],
        ),
        pick("see_through", k::alpha_label(), SEE_THROUGH.map(k::alpha)),
        flag("lamp_shadows", k::lamp_shadows()),
        pick(
            "lamp_cut",
            k::lamp_cut_label(),
            memory::lamp_cuts().into_iter().map(lamp_cut_text),
        ),
        pick(
            "lamp_nearest",
            g::lamps_drawn_label(),
            LAMP_NEAREST.map(nearest_text),
        ),
        flag("skip_hidden", k::skip_hidden()),
        flag("headlight", k::headlight()),
        pick(
            "highlights",
            k::shininess_label(),
            HIGHLIGHTS.map(k::shininess),
        ),
    ];
    for (i, scheme) in m::lighting_schemes().into_iter().enumerate() {
        rows.push(percent(
            &format!("gain_{i}"),
            &s::brightness_of(m::lighting(scheme)),
        ));
    }
    for row in &mut rows {
        row.key.insert_str(0, prefix);
    }
    rows
}

/// The Picture tab's rows, the own look's switch last.
fn shot_rows() -> Vec<Row> {
    let mut rows = vec![
        number("scale", s::PICTURE_SCALE, 0.1..=16.0, 3),
        number("dpi", s::dpi_label(), 24.0..=2400.0, 0),
        pick("backdrop", s::background_label(), (0..3).map(s::background)),
        pick("detail", s::detail_label(), (0..3).map(s::detail)),
        flag("wide_lines", s::wide_lines()),
        flag("sphere", s::SPHERE),
        pick("sphere_size", s::SPHERE_SIZE, (0..3).map(s::sphere_size)),
        flag("own_look", s::own_look()),
    ];
    for row in &mut rows {
        row.key.insert_str(0, "shot_");
    }
    rows
}

/// The value `key` holds, or the built-in one.
fn stored(map: &Settings, factory: &Settings, key: &str) -> Option<String> {
    map.get(key).or_else(|| factory.get(key)).cloned()
}

/// One row's widget; the new value when it was changed.
fn row_ui(ui: &mut Ui, row: &Row, value: Option<&str>) -> Option<String> {
    match &row.kind {
        Kind::Flag => {
            let mut on = value == Some("1");
            let before = on;
            widgets::toggle(ui, &mut on, &row.label, None);
            (on != before).then(|| u8::from(on).to_string())
        }
        Kind::Pick(choices) => {
            let mut at = value.and_then(|v| v.parse::<usize>().ok()).unwrap_or(0);
            let before = at;
            ui.horizontal_wrapped(|ui| {
                widgets::label(ui, &row.label);
                let shown = choices.get(at).cloned().unwrap_or_default();
                egui::ComboBox::from_id_salt(("model3d-setting", &row.key))
                    .selected_text(shown)
                    .show_ui(ui, |ui| {
                        for (i, choice) in choices.iter().enumerate() {
                            ui.selectable_value(&mut at, i, choice);
                        }
                    });
            });
            (at != before).then(|| at.to_string())
        }
        Kind::Number {
            range,
            decimals,
            percent,
        } => {
            let scale = if *percent { 100.0 } else { 1.0 };
            let stored = value
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(*range.start());
            let mut shown = stored * scale;
            let changed = ui
                .horizontal_wrapped(|ui| {
                    widgets::label(ui, &row.label);
                    let mut field = egui::DragValue::new(&mut shown)
                        .range(range.start() * scale..=range.end() * scale)
                        .max_decimals(if *percent { 0 } else { *decimals });
                    if *percent {
                        field = field.suffix(s::PERCENT);
                    }
                    ui.add(field).changed()
                })
                .inner;
            changed.then(|| format!("{:.*}", *decimals, shown / scale))
        }
        Kind::SunDisc { .. } | Kind::Within => None,
    }
}

/// The sun disc over the bearing at `key` and the height at `height_key`,
/// the model's front at its bottom: Settings has no camera to turn it by.
fn sun_disc_row(
    ui: &mut Ui,
    (key, height_key): (&str, &str),
    map: &mut Settings,
    factory: &Settings,
) {
    let read = |k: &str, map: &Settings| {
        stored(map, factory, k)
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0)
    };
    let (mut bearing, mut height) = (read(key, map), read(height_key, map));
    let before = (bearing, height);
    aim_row(ui, (&mut bearing, &mut height), 0.0);
    if bearing != before.0 {
        map.insert(key.to_owned(), format!("{bearing:.0}"));
    }
    if height != before.1 {
        map.insert(height_key.to_owned(), format!("{height:.0}"));
    }
}

/// Draw `rows` over `map`, writing what changes.
fn rows_ui(ui: &mut Ui, rows: &[Row], map: &mut Settings, factory: &Settings) {
    let prefix = pdfcer_gui_base::prefs::modelview::PREFIX;
    for row in rows {
        let key = format!("{prefix}{}", row.key);
        if let Kind::SunDisc { height } = row.kind {
            let height = format!("{prefix}{}", row.key.replace("sun_bearing", height));
            sun_disc_row(ui, (&key, &height), map, factory);
            continue;
        }
        if let Some(new) = row_ui(ui, row, stored(map, factory, &key).as_deref()) {
            map.insert(key, new);
        }
    }
}

fn section(ui: &mut Ui, heading: &str) {
    ui.add_space(10.0);
    widgets::label(ui, heading);
    ui.separator();
}

/// The page.
pub(crate) fn page(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(ui, s::PAGE_TITLE, s::PAGE_SILENCE, s::PAGE_RADIUS);
    widgets::option(
        ui,
        &mut prefs.model_start_walk,
        true,
        s::START_WALKING,
        None,
    );
    widgets::option(
        ui,
        &mut prefs.model_start_walk,
        false,
        s::START_ORBITING,
        None,
    );
    eye_row(ui, &mut prefs.walk_eye_mm);
    let factory = memory::factory();
    let map = &mut prefs.model_view;
    let tabs = s::tabs();
    section(ui, tabs[0]);
    rows_ui(ui, &view_rows(), map, &factory);
    section(ui, tabs[1]);
    rows_ui(ui, &look_rows("look_"), map, &factory);
    section(ui, k::CARD_HEADING);
    rows_ui(ui, &card_rows(), map, &factory);
    section(ui, tabs[3]);
    rows_ui(ui, &shot_rows(), map, &factory);
    let own = format!("{}shot_own_look", pdfcer_gui_base::prefs::modelview::PREFIX);
    if stored(map, &factory, &own).as_deref() == Some("1") {
        ui.indent("model3d-shot-look", |ui| {
            rows_ui(ui, &look_rows("shot_look_"), map, &factory);
        });
    }
}

/// The eye height, whole millimetres, or automatic.
fn eye_row(ui: &mut Ui, eye_mm: &mut Option<u32>) {
    let mut auto = eye_mm.is_none();
    widgets::toggle(ui, &mut auto, s::EYE_AUTO, None);
    if auto {
        *eye_mm = None;
        return;
    }
    let mut mm = eye_mm.unwrap_or(1600);
    ui.horizontal_wrapped(|ui| {
        widgets::label(ui, t::eye_height());
        ui.add(
            egui::DragValue::new(&mut mm)
                .range(25..=100_000)
                .suffix(s::MM),
        );
    });
    *eye_mm = Some(mm);
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_3d_nav::LensKind;

    #[test]
    fn every_saved_key_has_a_row_but_the_pictures_screen_size() {
        let prefix = pdfcer_gui_base::prefs::modelview::PREFIX;
        let rows: Vec<String> = [view_rows(), look_rows("look_"), card_rows(), shot_rows()]
            .into_iter()
            .flatten()
            .map(|r| format!("{prefix}{}", r.key))
            .collect();
        let missing: Vec<String> = memory::factory()
            .into_keys()
            .filter(|k| !k.ends_with("shot_base_w") && !k.ends_with("shot_base_h"))
            .filter(|k| !rows.contains(k))
            .collect();
        assert!(
            missing.is_empty(),
            "saved keys the page cannot reach: {missing:?}"
        );
        assert!(
            rows.iter().all(|r| memory::factory().contains_key(r)),
            "rows naming no saved key"
        );
    }

    #[test]
    fn a_choice_list_is_as_long_as_the_viewers() {
        for row in look_rows("look_")
            .iter()
            .chain(&view_rows())
            .chain(&shot_rows())
        {
            if let Kind::Pick(choices) = &row.kind {
                assert!(choices.len() >= 2, "{}", row.key);
            }
        }
        assert_eq!(LensKind::ALL.len(), l::lens_names().len());
    }
}
