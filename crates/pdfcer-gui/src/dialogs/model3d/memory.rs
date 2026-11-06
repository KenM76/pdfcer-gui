//! The viewer's settings kept between sessions: the navigation switches and
//! speed, the options menu's tab, the lens, the screen's look, and the saved picture's
//! size, resolution, backdrop, detail and own look, and the OS window's
//! size. Each is one `model3d_`
//! preferences key, written as a number or a 0/1 flag; a choice from a list
//! is its index in the list here, so the lists' order is part of the file's
//! format and new choices go on their ends.
//!
//! The settings are the operator's defaults: written only by *Save as
//! default* (or the Settings window), and read as each viewer opens. *Reset
//! to defaults* takes up [`factory`]'s, the built-in ones, without writing.
//! The OS window's size and the menu's tab alone are written as they change
//! and no pointer is held, merged into the file's keys so a viewer never
//! overwrites another's saved defaults. A key missing or unreadable leaves
//! that setting at its default. The projection and the camera are the
//! view's and are not kept; the eye height is kept by its own key.

use std::collections::BTreeMap;
use std::str::FromStr;

use pdfcer_3d::{LampCut, LightSense, RenderModeKind};
use pdfcer_3d_nav::{AUTO_RANGE, CURVES, LensKind};
use pdfcer_core::threed::ThreeDLighting;

use super::ModelView;
use super::card::CardParts;
use super::looks::LAMP_NEAREST;
use super::nav;
use super::shot::Shot;
use super::shotlook::LookSet;
use super::sunlight::{BEARING_RANGE, GAIN_RANGE, HEIGHT_RANGE, KELVIN_RANGE, Light};
use crate::app::actions::Action;
use crate::app::actions::prefs::PrefAction;
use crate::text::model3dlook::{Highlights, SeeThrough};

/// The settings, keyed as in the preferences file.
pub(super) type Settings = BTreeMap<String, String>;

/// The keys written as they change rather than by *Save as default*.
pub(super) const KEPT_AS_THEY_CHANGE: [&str; 3] = ["tab", "window_w", "window_h"];

pub(super) const SENSES: [LightSense; 2] = [LightSense::TowardsLight, LightSense::Emitted];
pub(super) const SEE_THROUGH: [SeeThrough; 3] =
    [SeeThrough::Tinted, SeeThrough::Cutout, SeeThrough::None];
pub(super) const HIGHLIGHTS: [Highlights; 3] =
    [Highlights::Auto, Highlights::Exponent, Highlights::Fraction];

/// The lamp reach choices, in the drop-down's order.
pub(super) fn lamp_cuts() -> Vec<LampCut> {
    std::iter::once(LampCut::Auto)
        .chain(super::looks::LAMP_CUT_STEPS.map(LampCut::Threshold))
        .chain(std::iter::once(LampCut::Off))
        .collect()
}

/// The lighting choices: the view's own, then each scheme.
pub(super) fn lightings() -> Vec<Option<ThreeDLighting>> {
    std::iter::once(None)
        .chain(crate::text::model3d::lighting_schemes().map(Some))
        .collect()
}

/// The render mode choices: the view's own, then each mode.
pub(super) fn modes() -> Vec<Option<RenderModeKind>> {
    std::iter::once(None)
        .chain(RenderModeKind::ALL.map(Some))
        .collect()
}

/// Writes keys under one prefix.
struct Put<'a>(&'a mut Settings, &'a str);

impl Put<'_> {
    fn value(&mut self, key: &str, value: impl ToString) {
        self.0.insert(
            format!(
                "{}{}{key}",
                pdfcer_gui_base::prefs::modelview::PREFIX,
                self.1
            ),
            value.to_string(),
        );
    }
    fn flag(&mut self, key: &str, on: bool) {
        self.value(key, u8::from(on));
    }
    fn pick<T: PartialEq>(&mut self, key: &str, list: &[T], chosen: &T) {
        if let Some(i) = list.iter().position(|item| item == chosen) {
            self.value(key, i);
        }
    }
}

/// Reads keys under one prefix.
struct Get<'a>(&'a Settings, &'a str);

impl Get<'_> {
    fn value<T: FromStr>(&self, key: &str) -> Option<T> {
        let key = format!(
            "{}{}{key}",
            pdfcer_gui_base::prefs::modelview::PREFIX,
            self.1
        );
        self.0.get(&key)?.parse().ok()
    }
    fn flag(&self, key: &str, into: &mut bool) {
        match self.value::<u8>(key) {
            Some(0) => *into = false,
            Some(1) => *into = true,
            _ => {}
        }
    }
    fn pick<T: Clone>(&self, key: &str, list: &[T], into: &mut T) {
        if let Some(item) = self.value::<usize>(key).and_then(|i| list.get(i)) {
            *into = item.clone();
        }
    }
    fn number<T: FromStr + PartialOrd>(
        &self,
        key: &str,
        range: std::ops::RangeInclusive<T>,
        into: &mut T,
    ) {
        if let Some(v) = self.value::<T>(key).filter(|v| range.contains(v)) {
            *into = v;
        }
    }
}

fn put_look(put: &mut Put<'_>, look: &LookSet) {
    put.pick("lighting", &lightings(), &look.view_lighting);
    put.flag("lights_turn", look.lights_turn);
    put.flag("shadows", look.shadows);
    put.flag("occlusion", look.occlusion);
    put.pick("light_sense", &SENSES, &look.light_sense);
    put.flag("sun", look.sun);
    put_light(put, &look.light);
    let x = &look.extras;
    put.pick("mode", &modes(), &x.mode);
    put.flag("every_light", x.every_light);
    put.pick("see_through", &SEE_THROUGH, &x.see_through);
    put.flag("lamp_shadows", x.lamp_shadows);
    put.pick("lamp_cut", &lamp_cuts(), &x.lamp_cut);
    put.pick("lamp_nearest", &LAMP_NEAREST, &x.nearest);
    put.flag("skip_hidden", x.skip_hidden);
    put.flag("headlight", x.headlight);
    put.pick("highlights", &HIGHLIGHTS, &x.highlights);
}

fn put_light(put: &mut Put<'_>, light: &Light) {
    for (i, gain) in light.gains.iter().enumerate() {
        put.value(&format!("gain_{i}"), format!("{gain:.3}"));
    }
    let a = &light.sun;
    put.flag("sun_aimed", a.on);
    put.value("sun_bearing", format!("{:.0}", a.bearing));
    put.value("sun_height", format!("{:.0}", a.height));
    put.flag("sun_by_height", a.by_height);
    put.value("sun_kelvin", format!("{:.0}", a.kelvin));
    put.value("sun_strength", format!("{:.3}", a.strength));
}

fn get_light(get: &Get<'_>, light: &mut Light) {
    for (i, gain) in light.gains.iter_mut().enumerate() {
        get.number(&format!("gain_{i}"), GAIN_RANGE, gain);
    }
    let a = &mut light.sun;
    get.flag("sun_aimed", &mut a.on);
    get.number("sun_bearing", BEARING_RANGE, &mut a.bearing);
    get.number("sun_height", HEIGHT_RANGE, &mut a.height);
    get.flag("sun_by_height", &mut a.by_height);
    get.number("sun_kelvin", KELVIN_RANGE, &mut a.kelvin);
    get.number("sun_strength", GAIN_RANGE, &mut a.strength);
}

fn get_look(get: &Get<'_>, look: &mut LookSet) {
    get.pick("lighting", &lightings(), &mut look.view_lighting);
    get.flag("lights_turn", &mut look.lights_turn);
    get.flag("shadows", &mut look.shadows);
    get.flag("occlusion", &mut look.occlusion);
    get.pick("light_sense", &SENSES, &mut look.light_sense);
    get.flag("sun", &mut look.sun);
    get_light(get, &mut look.light);
    let x = &mut look.extras;
    get.pick("mode", &modes(), &mut x.mode);
    get.flag("every_light", &mut x.every_light);
    get.pick("see_through", &SEE_THROUGH, &mut x.see_through);
    get.flag("lamp_shadows", &mut x.lamp_shadows);
    get.pick("lamp_cut", &lamp_cuts(), &mut x.lamp_cut);
    get.pick("lamp_nearest", &LAMP_NEAREST, &mut x.nearest);
    get.flag("skip_hidden", &mut x.skip_hidden);
    get.flag("headlight", &mut x.headlight);
    get.pick("highlights", &HIGHLIGHTS, &mut x.highlights);
}

/// What one viewer keeps, apart from its tab and OS window.
struct Kept<'a> {
    prefs: &'a nav::Prefs,
    lens: &'a pdfcer_3d_nav::Lens,
    look: &'a LookSet,
    shot: &'a Shot,
    card: &'a CardParts,
}

impl Kept<'_> {
    fn put(&self, map: &mut Settings) {
        let mut put = Put(map, "");
        let p = self.prefs;
        put.flag("ghost", p.ghost);
        put.flag("level", p.level);
        put.flag("gimbal", p.gimbal);
        put.flag("controls", p.controls);
        put.value("speed", format!("{:.3}", p.speed));
        put.flag("stairs", p.stairs);
        let lens = self.lens;
        put.pick("lens", &LensKind::ALL, &lens.kind);
        put.pick("lens_curve", &CURVES, &lens.curve);
        put.value("lens_auto_at", format!("{:.0}", lens.auto_at));
        put.flag("card_picture", self.card.picture);
        for (i, on) in self.card.parts.iter().enumerate() {
            put.flag(&format!("card_part_{i}"), *on);
        }
        put_look(&mut Put(map, "look_"), self.look);
        let s = self.shot;
        let mut put = Put(map, "shot_");
        put.value("scale", format!("{:.3}", s.scale));
        put.value("dpi", s.dpi);
        put.value("backdrop", s.backdrop);
        put.value("detail", s.detail);
        put.flag("wide_lines", s.wide_lines);
        put.flag("sphere", s.sphere);
        put.value("sphere_size", s.sphere_size);
        put.value("base_w", s.base[0]);
        put.value("base_h", s.base[1]);
        put.flag("own_look", s.own.is_some());
        if let Some(own) = &s.own {
            put_look(&mut Put(map, "shot_look_"), own);
        }
    }
}

/// The built-in defaults: every key a viewer keeps but its tab and window.
pub(crate) fn factory() -> Settings {
    let mut map = Settings::new();
    Kept {
        prefs: &nav::Prefs::default(),
        lens: &pdfcer_3d_nav::Lens::default(),
        look: &LookSet::factory(),
        shot: &Shot::default(),
        card: &CardParts::default(),
    }
    .put(&mut map);
    map
}

impl ModelView {
    /// Every kept setting, as it now stands.
    pub(super) fn remember(&self) -> Settings {
        let mut map = Settings::new();
        let look = self.look_set();
        Kept {
            prefs: &self.nav.prefs,
            lens: &self.lens,
            look: &look,
            shot: &self.shot,
            card: &self.gpu.card,
        }
        .put(&mut map);
        let mut put = Put(&mut map, "");
        put.value("tab", self.nav.options_tab);
        if let Some([w, h]) = self.nav.window.or(self.nav.window_open) {
            put.value("window_w", w);
            put.value("window_h", h);
        }
        map
    }

    /// Take up `settings`, the last session's, over the defaults the viewer
    /// opened with. Opening walking keeps ghost on.
    pub(crate) fn recall(&mut self, settings: &Settings) {
        let get = Get(settings, "");
        let mut p = self.nav.prefs;
        get.flag("ghost", &mut p.ghost);
        get.flag("level", &mut p.level);
        get.flag("gimbal", &mut p.gimbal);
        get.flag("controls", &mut p.controls);
        get.number("speed", 0.25..=4.0, &mut p.speed);
        get.flag("stairs", &mut p.stairs);
        self.nav.recalled(p);
        get.number("tab", 0..=4, &mut self.nav.options_tab);
        get.pick("lens", &LensKind::ALL, &mut self.lens.kind);
        get.pick("lens_curve", &CURVES, &mut self.lens.curve);
        get.number("lens_auto_at", AUTO_RANGE, &mut self.lens.auto_at);
        get.flag("card_picture", &mut self.gpu.card.picture);
        for (i, on) in self.gpu.card.parts.iter_mut().enumerate() {
            get.flag(&format!("card_part_{i}"), on);
        }
        let mut size = [0.0_f32; 2];
        get.number("window_w", 200.0..=20_000.0, &mut size[0]);
        get.number("window_h", 200.0..=20_000.0, &mut size[1]);
        if size.iter().all(|v| *v > 0.0) {
            self.nav.window_open = Some(size);
        }
        let mut look = self.look_set();
        get_look(&Get(settings, "look_"), &mut look);
        self.swap_look(&mut look);
        let get = Get(settings, "shot_");
        let s = &mut self.shot;
        get.number("scale", 0.1..=16.0, &mut s.scale);
        get.number("dpi", 24..=2400, &mut s.dpi);
        get.number("backdrop", 0..=2, &mut s.backdrop);
        get.number("detail", 0..=2, &mut s.detail);
        get.flag("wide_lines", &mut s.wide_lines);
        get.flag("sphere", &mut s.sphere);
        get.number("sphere_size", 0..=2, &mut s.sphere_size);
        get.number("base_w", 1..=10_000, &mut s.base[0]);
        get.number("base_h", 1..=10_000, &mut s.base[1]);
        let mut own = self.shot.own.is_some();
        get.flag("own_look", &mut own);
        self.shot.own = own.then(|| {
            let mut set = self.shot.own.unwrap_or_else(|| self.look_set());
            get_look(&Get(settings, "shot_look_"), &mut set);
            set
        });
        self.remembered = self.kept_as_they_change();
        let (keys, p, tab) = (settings.len(), self.nav.prefs, self.nav.options_tab);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-view-settings-recalled keys={keys} stairs={} speed={:.2} tab={tab}",
                p.stairs, p.speed
            )
        });
    }

    /// The settings written as they change: [`KEPT_AS_THEY_CHANGE`]'s.
    fn kept_as_they_change(&self) -> Settings {
        let prefix = pdfcer_gui_base::prefs::modelview::PREFIX;
        let mut map = self.remember();
        map.retain(|key, _| {
            key.strip_prefix(prefix)
                .is_some_and(|k| KEPT_AS_THEY_CHANGE.contains(&k))
        });
        map
    }

    /// Ask for the tab and the OS window's size to be written when they
    /// differ from those last written, no pointer is held and a second has
    /// passed since the last write (an OS window resized by its edge changes
    /// size every frame with no pointer down here), or `now`, as the viewer
    /// closes.
    pub(super) fn keep_settings(
        &mut self,
        ctx: &egui::Context,
        now: bool,
        actions: &mut Vec<Action>,
    ) {
        let settings = self.kept_as_they_change();
        if settings == self.remembered {
            return;
        }
        let (held, time) = ctx.input(|i| (i.pointer.any_down(), i.time));
        let wait = 1.0 - (time - self.nav.kept_at);
        if !now && (held || wait > 0.0) {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(wait.clamp(0.05, 1.0)));
            return;
        }
        self.remembered = settings.clone();
        self.nav.kept_at = time;
        actions.push(Action::Pref(PrefAction::ModelViewMerge(settings)));
    }

    /// Write every setting and the eye height as the operator's defaults.
    pub(super) fn save_defaults(&mut self, actions: &mut Vec<Action>) {
        let settings = self.remember();
        let keys = settings.len();
        self.remembered = self.kept_as_they_change();
        let mm = self.nav.prefs.eye_mm.map(|mm| mm.round().max(1.0) as u32);
        actions.push(Action::Pref(PrefAction::ModelView(settings)));
        actions.push(Action::Pref(PrefAction::WalkEyeHeight(mm)));
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| format!("model-view-defaults-saved keys={keys}"));
    }

    /// Take up the built-in defaults, the eye height's share of the model
    /// included, without writing them.
    pub(super) fn reset_defaults(&mut self) {
        self.recall(&factory());
        self.nav.prefs.eye_mm = None;
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| "model-view-defaults-reset".to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_look_round_trips_through_its_keys() {
        let mut look = LookSet::default_for_tests();
        look.view_lighting = Some(ThreeDLighting::Day);
        look.shadows = !look.shadows;
        look.light_sense = LightSense::Emitted;
        look.extras.mode = RenderModeKind::ALL.last().copied();
        look.extras.see_through = SeeThrough::Cutout;
        look.extras.lamp_cut = LampCut::Off;
        look.extras.nearest = Some(8);
        look.extras.highlights = Highlights::Fraction;
        look.light.gains[3] = 2.5;
        look.light.sun.on = true;
        look.light.sun.bearing = 200.0;
        look.light.sun.by_height = false;
        look.light.sun.kelvin = 3200.0;
        look.light.sun.strength = 0.5;
        let mut map = Settings::new();
        put_look(&mut Put(&mut map, "look_"), &look);
        assert!(map.keys().all(|k| k.starts_with("model3d_look_")));
        let mut back = LookSet::default_for_tests();
        get_look(&Get(&map, "look_"), &mut back);
        assert_eq!(back, look);
    }

    #[test]
    fn the_built_in_defaults_hold_every_saved_key_but_tab_and_window() {
        let keys = factory();
        assert!(keys.contains_key("model3d_card_part_6"));
        assert!(keys.contains_key("model3d_look_shadows"));
        assert!(!keys.contains_key("model3d_shot_look_shadows"));
        for kept in KEPT_AS_THEY_CHANGE {
            assert!(!keys.contains_key(&format!("model3d_{kept}")));
        }
    }

    #[test]
    fn a_value_out_of_range_or_unreadable_keeps_the_default() {
        let mut map = Settings::new();
        map.insert("model3d_speed".to_owned(), "99".to_owned());
        map.insert("model3d_stairs".to_owned(), "yes".to_owned());
        map.insert("model3d_tab".to_owned(), "7".to_owned());
        let get = Get(&map, "");
        let (mut speed, mut stairs, mut tab) = (1.0, false, 2usize);
        get.number("speed", 0.25..=4.0, &mut speed);
        get.flag("stairs", &mut stairs);
        get.number("tab", 0..=3, &mut tab);
        assert_eq!((speed, stairs, tab), (1.0, false, 2));
    }
}
