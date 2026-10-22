//! # `colourpick` — a document colour in Grey, RGB or CMYK, with its opacity
//!
//! Contract: [`Pick`] is a colour in the device space the document stores it
//! in, plus an opacity where the target can carry one. [`body`] draws the
//! inside of a colour popup: one tab per space the target accepts
//! ([`Spaces`]), a number for every component, and an opacity slider when
//! [`Pick::opacity`] is `Some`. A value is shown in its own space and is never
//! converted unless the operator switches tab. A switch seeds the new space
//! with the colour pdfcer draws the old one as: into CMYK by
//! `pdfcer_core::color::srgb_to_cmyk`, out of it by `cmyk_to_srgb`. A colour
//! CMYK cannot draw is seeded with the nearest it can, and the picker says so
//! beneath the CMYK numbers (`{region}.outside-cmyk`).
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/colourpick.md`.

use egui::{Color32, Ui};

use crate::text::colourpick as t;

/// A device colour space a target can store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Space {
    /// `DeviceGray`.
    Gray,
    /// `DeviceRGB`.
    Rgb,
    /// `DeviceCMYK`.
    Cmyk,
}

/// A device colour, each component in `0.0..=1.0` as the file stores it
/// (grey 0 is black; CMYK 1 is full ink).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ink {
    /// One component.
    Gray(f32),
    /// Red, green, blue.
    Rgb([f32; 3]),
    /// Cyan, magenta, yellow, black.
    Cmyk([f32; 4]),
}

impl Ink {
    /// Black in `space`.
    #[must_use]
    pub const fn black(space: Space) -> Self {
        match space {
            Space::Gray => Self::Gray(0.0),
            Space::Rgb => Self::Rgb([0.0; 3]),
            Space::Cmyk => Self::Cmyk([0.0, 0.0, 0.0, 1.0]),
        }
    }

    /// From 8-bit sRGB.
    #[must_use]
    pub fn from_srgb(rgb: [u8; 3]) -> Self {
        Self::Rgb(rgb.map(|c| f32::from(c) / 255.0))
    }

    /// The space this colour is in.
    #[must_use]
    pub const fn space(self) -> Space {
        match self {
            Self::Gray(_) => Space::Gray,
            Self::Rgb(_) => Space::Rgb,
            Self::Cmyk(_) => Space::Cmyk,
        }
    }

    /// 8-bit RGB, for an RGB colour only.
    #[must_use]
    pub fn srgb(self) -> Option<[u8; 3]> {
        match self {
            Self::Rgb(c) => Some(c.map(byte)),
            _ => None,
        }
    }

    /// This colour in `space`; itself when already there.
    #[must_use]
    pub fn to_space(self, space: Space) -> Self {
        self.convert(space).ink
    }

    /// This colour in `space` as pdfcer draws it, and whether `space` can
    /// draw it. Grey takes the §10.4.2 luminance of the drawn RGB.
    #[must_use]
    pub fn convert(self, space: Space) -> Converted {
        let exact = |ink| Converted {
            ink,
            in_gamut: true,
        };
        if self.space() == space {
            return exact(self);
        }
        let [r, g, b] = self.rgb();
        match space {
            Space::Gray => exact(Self::Gray(0.11f32.mul_add(b, 0.3f32.mul_add(r, 0.59 * g)))),
            Space::Rgb => exact(Self::Rgb([r, g, b])),
            Space::Cmyk => {
                let fit = pdfcer_core::color::srgb_to_cmyk(r, g, b);
                Converted {
                    ink: Self::Cmyk(fit.cmyk),
                    in_gamut: fit.in_gamut,
                }
            }
        }
    }

    /// Red, green and blue as pdfcer draws this colour.
    fn rgb(self) -> [f32; 3] {
        match self {
            Self::Gray(v) => [v; 3],
            Self::Rgb(c) => c,
            Self::Cmyk([c, m, y, k]) => pdfcer_core::color::cmyk_to_srgb(c, m, y, k),
        }
    }

    /// A screen colour for the swatch, as pdfcer renders the stored colour
    /// (`annot_author::Color::to_device_rgb`), so the swatch matches the page.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)] // colour channels are 0..1
    pub fn screen(self) -> Color32 {
        let [r, g, b] = pdfcer_core::annot_author::Color::from(self)
            .to_device_rgb()
            .map(|v| byte(v as f32));
        // DOCUMENT COLOUR: the ink being edited, not chrome.
        Color32::from_rgb(r, g, b)
    }
}

#[allow(clippy::cast_possible_truncation)] // colour channels are 0..1
impl From<pdfcer_core::annot_author::Color> for Ink {
    fn from(colour: pdfcer_core::annot_author::Color) -> Self {
        use pdfcer_core::annot_author::Color;
        match colour {
            Color::Gray(v) => Self::Gray(v as f32),
            Color::Rgb(r, g, b) => Self::Rgb([r, g, b].map(|v| v as f32)),
            Color::Cmyk(c, m, y, k) => Self::Cmyk([c, m, y, k].map(|v| v as f32)),
        }
    }
}

impl From<Ink> for pdfcer_core::annot_author::Color {
    fn from(ink: Ink) -> Self {
        match ink {
            Ink::Gray(v) => Self::Gray(f64::from(v)),
            Ink::Rgb([r, g, b]) => Self::Rgb(f64::from(r), f64::from(g), f64::from(b)),
            Ink::Cmyk([c, m, y, k]) => {
                Self::Cmyk(f64::from(c), f64::from(m), f64::from(y), f64::from(k))
            }
        }
    }
}

impl Ink {
    /// A text run's fill in its own space; `None` for a space this picker
    /// cannot edit (`/Separation`, `/DeviceN`, ICC-based, or one the engine
    /// adds later), which must not be rewritten as process colour.
    #[must_use]
    pub fn from_text(colour: pdfcer_core::text_extract::TextColor) -> Option<Self> {
        use pdfcer_core::text_extract::TextColor;
        match colour {
            TextColor::Gray(v) => Some(Self::Gray(v)),
            TextColor::Rgb(r, g, b) => Some(Self::Rgb([r, g, b])),
            TextColor::Cmyk(c, m, y, k) => Some(Self::Cmyk([c, m, y, k])),
            _ => None,
        }
    }

    /// The text fill that writes this ink in its own space.
    #[must_use]
    pub fn fill(self) -> Option<pdfcer_core::text_edit::NewFill> {
        use pdfcer_core::text_edit::{FillModel, NewFill};
        let (model, parts): (FillModel, &[f32]) = match &self {
            Self::Gray(v) => (FillModel::Gray, std::slice::from_ref(v)),
            Self::Rgb(c) => (FillModel::Rgb, c),
            Self::Cmyk(c) => (FillModel::Cmyk, c),
        };
        NewFill::new(model, parts.iter().map(|&v| f64::from(v)).collect()).ok()
    }

    /// One whitespace-free token for a trace line: `cmyk:0.000,…`, `rgb:…`
    /// or `gray:…`.
    #[must_use]
    pub fn token(self) -> String {
        let parts: &[f32] = match &self {
            Self::Gray(v) => std::slice::from_ref(v),
            Self::Rgb(c) => c,
            Self::Cmyk(c) => c,
        };
        let joined: Vec<String> = parts.iter().map(|v| format!("{v:.3}")).collect();
        format!("{}:{}", key(self.space()), joined.join(","))
    }
}

fn byte(v: f32) -> u8 {
    // In range by the clamp, so the cast cannot truncate.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let b = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    b
}

/// The spaces a target accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spaces {
    /// `DeviceGray`.
    pub gray: bool,
    /// `DeviceRGB`.
    pub rgb: bool,
    /// `DeviceCMYK`.
    pub cmyk: bool,
}

impl Spaces {
    /// RGB only, for a verb that takes nothing else.
    pub const RGB: Self = Self {
        gray: false,
        rgb: true,
        cmyk: false,
    };
    /// All three device spaces.
    pub const ALL: Self = Self {
        gray: true,
        rgb: true,
        cmyk: true,
    };

    /// Whether `space` is accepted.
    #[must_use]
    pub const fn accepts(self, space: Space) -> bool {
        match space {
            Space::Gray => self.gray,
            Space::Rgb => self.rgb,
            Space::Cmyk => self.cmyk,
        }
    }
}

/// A colour and, where the target can carry one, its opacity in `0.0..=1.0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pick {
    /// The colour.
    pub ink: Ink,
    /// The opacity; `None` where the target has none, and no slider is drawn.
    pub opacity: Option<f32>,
}

/// The tabs in the order drawn.
const TABS: [Space; 3] = [Space::Rgb, Space::Cmyk, Space::Gray];

/// Draw the picker's inside. `true` when the operator changed `pick`.
/// Publishes `{region}.space.<rgb|cmyk|gray>` per tab and
/// `{region}.opacity` for a driven check.
pub fn body(ui: &mut Ui, pick: &mut Pick, spaces: Spaces, region: &str) -> bool {
    let mut changed = false;
    if !spaces.accepts(pick.ink.space()) {
        // A value in a space the target cannot take is shown in one it can.
        let first = TABS.into_iter().find(|s| spaces.accepts(*s));
        switch(ui, pick, first.unwrap_or(Space::Rgb), region);
    }
    if TABS.iter().filter(|s| spaces.accepts(**s)).count() > 1 {
        ui.horizontal(|ui| {
            for space in TABS.into_iter().filter(|s| spaces.accepts(*s)) {
                let tab = ui
                    .selectable_label(pick.ink.space() == space, t::space(space))
                    .on_hover_text(t::space_tooltip(space));
                crate::diag::ui_rect(&format!("{region}.space.{}", key(space)), tab.rect);
                if tab.clicked() && pick.ink.space() != space {
                    switch(ui, pick, space, region);
                    changed = true;
                }
            }
        });
    }
    let edited = match &mut pick.ink {
        Ink::Rgb(c) => rgb_rows(ui, c),
        Ink::Cmyk(c) => cmyk_rows(ui, c),
        Ink::Gray(v) => gray_rows(ui, v),
    };
    if edited || pick.ink.space() != Space::Cmyk {
        // Numbers typed are the operator's own; nothing was approximated.
        ui.data_mut(|d| d.insert_temp(outside_id(region), false));
    }
    changed |= edited;
    if ui.data(|d| d.get_temp(outside_id(region))).unwrap_or(false) {
        let line = ui.label(egui::RichText::new(t::outside_cmyk()).small());
        crate::diag::ui_rect(&format!("{region}.outside-cmyk"), line.rect);
    }
    if let Some(alpha) = &mut pick.opacity {
        let row = ui.horizontal(|ui| {
            ui.label(t::opacity());
            percent_slider(ui, alpha)
        });
        crate::diag::ui_rect(&format!("{region}.opacity"), row.response.rect);
        changed |= row.inner;
    }
    changed
}

/// The result of [`Ink::convert`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Converted {
    /// The colour in the asked-for space.
    pub ink: Ink,
    /// `false` when the space cannot draw the colour and `ink` is the nearest
    /// it can.
    pub in_gamut: bool,
}

/// The memory slot holding whether the CMYK shown in `region`'s picker was
/// seeded from a colour CMYK cannot draw.
fn outside_id(region: &str) -> egui::Id {
    egui::Id::new(("colourpick-outside-cmyk", region)) // ui-text-exempt: an egui id salt, never displayed
}

/// Convert `pick` to `space`, remembering whether the result is the nearest
/// colour rather than the same one.
fn switch(ui: &Ui, pick: &mut Pick, space: Space, region: &str) {
    let converted = pick.ink.convert(space);
    pick.ink = converted.ink;
    let outside = !converted.in_gamut;
    ui.data_mut(|d| d.insert_temp(outside_id(region), outside));
    crate::diag::trace(|| {
        format!(
            "colour-switched region={region} to={} in_gamut={}",
            key(space),
            converted.in_gamut
        )
    });
}

/// What a selection says its colour is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shown {
    /// Every member is this colour.
    Agreed(Pick),
    /// The members disagree; the control still applies to all of them.
    Mixed {
        /// Whether the target carries an opacity, so the picker offers one.
        with_opacity: bool,
    },
}

/// The picker's in-progress value across the frames it is open for.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Editing {
    working: Pick,
    /// Moved since this open; a look-and-close authors nothing.
    dirty: bool,
    /// Open on the previous frame; the close edge is the commit trigger.
    was_open: bool,
}

/// One frame of the swatch and its popup. `Some(pick)` only on the frame the
/// popup closed after the operator changed it, so a whole gesture is one edit.
/// Publishes `{region}` (the swatch), `{region}.picker` (the popup) and the
/// regions [`body`] names.
pub fn show(
    ui: &mut Ui,
    id_salt: &str,
    shown: Shown,
    spaces: Spaces,
    region: &str,
    mixed_hint: &str,
) -> Option<Pick> {
    let id = ui.make_persistent_id(id_salt);
    let popup_id = id.with("popup"); // ui-text-exempt: an egui id salt, never displayed
    let seed = match shown {
        Shown::Agreed(pick) => pick,
        Shown::Mixed { with_opacity } => Pick {
            ink: Ink::black(
                TABS.into_iter()
                    .find(|s| spaces.accepts(*s))
                    .unwrap_or(Space::Rgb),
            ),
            opacity: with_opacity.then_some(1.0),
        },
    };
    let mut state: Editing = ui.data(|d| d.get_temp(id)).unwrap_or(Editing {
        working: seed,
        dirty: false,
        was_open: false,
    });
    let response = swatch_button(ui, shown);
    crate::diag::ui_rect_visible(region, response.rect, ui.clip_rect());
    // A fresh open re-seeds, so a picker opened over B never shows A's colour.
    if egui::Popup::is_id_open(ui.ctx(), popup_id) && !state.was_open {
        state.working = seed;
        state.dirty = false;
        ui.data_mut(|d| d.insert_temp(outside_id(region), false));
    }
    egui::Popup::menu(&response)
        .id(popup_id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            if let Shown::Mixed { .. } = shown {
                ui.label(egui::RichText::new(mixed_hint).small().weak());
            }
            state.dirty |= body(ui, &mut state.working, spaces, region);
            // Its own name: the swatch and the popup are two rectangles in one
            // frame. `ui_rect`, because a popup's clip rect is the whole screen.
            crate::diag::ui_rect(&format!("{region}.picker"), ui.min_rect());
        });
    let open_after = egui::Popup::is_id_open(ui.ctx(), popup_id);
    let committed = (state.was_open && !open_after && state.dirty).then_some(state.working);
    if committed.is_some() {
        state.dirty = false;
    }
    state.was_open = open_after;
    ui.data_mut(|d| d.insert_temp(id, state));
    committed
}

/// A swatch editing one stored colour with no opacity, in place, in any
/// device space. `true` on the frame the edit commits.
pub fn edit_color(
    ui: &mut Ui,
    id_salt: &str,
    value: &mut pdfcer_core::annot_author::Color,
    region: &str,
) -> bool {
    let shown = Shown::Agreed(Pick {
        ink: Ink::from(*value),
        opacity: None,
    });
    let Some(pick) = show(ui, id_salt, shown, Spaces::ALL, region, "") else {
        return false;
    };
    *value = pick.ink.into();
    true
}

/// A swatch over one text fill with no opacity, in any device space: the
/// fill to write, in the space picked, on the frame the edit commits.
pub fn edit_text_fill(
    ui: &mut Ui,
    id_salt: &str,
    current: Ink,
    region: &str,
) -> Option<pdfcer_core::text_edit::NewFill> {
    let shown = Shown::Agreed(Pick {
        ink: current,
        opacity: None,
    });
    show(ui, id_salt, shown, Spaces::ALL, region, "").and_then(|pick| pick.ink.fill())
}

/// The swatch: the agreed colour, or the indeterminate marker over a mix.
fn swatch_button(ui: &mut Ui, shown: Shown) -> egui::Response {
    let height = ui.spacing().interact_size.y;
    let size = egui::vec2(height * 1.7, height);
    match shown {
        Shown::Agreed(pick) => ui.add(egui::Button::new("").fill(pick.ink.screen()).min_size(size)),
        Shown::Mixed { .. } => {
            let (plate, ink) = egui_shell::Theme::indeterminate_pair(ui.ctx());
            let marker = crate::text::panels::properties::text_value_absent();
            ui.add(
                egui::Button::new(egui::RichText::new(marker).color(ink))
                    .fill(plate)
                    .min_size(size),
            )
        }
    }
}

/// The machine name of a space, for regions; never displayed.
#[must_use]
pub const fn key(space: Space) -> &'static str {
    match space {
        Space::Gray => "gray",
        Space::Rgb => "rgb",
        Space::Cmyk => "cmyk",
    }
}

/// The colour square, then red, green and blue as 0–255.
fn rgb_rows(ui: &mut Ui, c: &mut [f32; 3]) -> bool {
    let mut colour = Ink::Rgb(*c).screen();
    let mut changed = false;
    if egui::color_picker::color_picker_color32(ui, &mut colour, egui::color_picker::Alpha::Opaque)
    {
        *c = Ink::from_srgb([colour.r(), colour.g(), colour.b()]).rgb();
        changed = true;
    }
    ui.horizontal(|ui| {
        for (value, label) in c.iter_mut().zip(t::rgb_labels()) {
            let mut b = byte(*value);
            ui.label(label);
            if ui
                .add(egui::DragValue::new(&mut b).range(0..=255))
                .changed()
            {
                *value = f32::from(b) / 255.0;
                changed = true;
            }
        }
    });
    changed
}

/// Cyan, magenta, yellow and black as percentages, above a swatch.
fn cmyk_rows(ui: &mut Ui, c: &mut [f32; 4]) -> bool {
    let mut changed = false;
    egui::Grid::new("ink-cmyk").num_columns(2).show(ui, |ui| {
        for (value, label) in c.iter_mut().zip(t::cmyk_labels()) {
            ui.label(label);
            changed |= percent_slider(ui, value);
            ui.end_row();
        }
    });
    chip(ui, Ink::Cmyk(*c));
    ui.label(egui::RichText::new(t::cmyk_on_screen()).small().weak());
    changed
}

/// Black as a percentage, so 100 % is solid black as in every editor.
fn gray_rows(ui: &mut Ui, v: &mut f32) -> bool {
    let mut black = 1.0 - *v;
    let changed = ui
        .horizontal(|ui| {
            ui.label(t::gray_label());
            percent_slider(ui, &mut black)
        })
        .inner;
    if changed {
        *v = 1.0 - black;
    }
    chip(ui, Ink::Gray(*v));
    changed
}

fn percent_slider(ui: &mut Ui, value: &mut f32) -> bool {
    let mut percent = *value * 100.0;
    let moved = ui
        .add(egui::Slider::new(&mut percent, 0.0..=100.0).suffix(t::percent()))
        .changed();
    if moved {
        *value = (percent / 100.0).clamp(0.0, 1.0);
    }
    moved
}

fn chip(ui: &mut Ui, ink: Ink) {
    let size = egui::vec2(160.0, ui.spacing().interact_size.y);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.painter().rect_filled(rect, 2.0, ink.screen());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_colour_in_its_own_space_is_never_converted() {
        let cmyk = Ink::Cmyk([0.1, 0.2, 0.3, 0.4]);
        assert_eq!(cmyk.to_space(Space::Cmyk), cmyk);
        let gray = Ink::Gray(0.25);
        assert_eq!(gray.to_space(Space::Gray), gray);
    }

    #[test]
    fn black_and_white_cross_every_space() {
        // CMYK's black is full K, which pdfcer draws as a warm near-black, so
        // only grey and RGB cross exactly.
        for (a, b) in [(Space::Gray, Space::Rgb), (Space::Rgb, Space::Gray)] {
            assert_eq!(Ink::black(a).to_space(b), Ink::black(b));
        }
        for space in [Space::Gray, Space::Rgb] {
            let Ink::Cmyk([.., k]) = Ink::black(space).to_space(Space::Cmyk) else {
                panic!("not CMYK");
            };
            assert!(k > 0.99, "{space:?}: k {k}");
        }
        let white = Ink::Rgb([1.0; 3]);
        let Ink::Cmyk(ink) = white.to_space(Space::Cmyk) else {
            panic!("not CMYK");
        };
        assert!(ink.iter().all(|v| *v < 1e-3), "{ink:?}");
        assert_eq!(white.to_space(Space::Gray), Ink::Gray(1.0));
    }

    #[test]
    fn rgb_to_cmyk_keeps_the_colour_pdfcer_draws() {
        let rgb = [0.8, 0.4, 0.4];
        let converted = Ink::Rgb(rgb).convert(Space::Cmyk);
        assert!(converted.in_gamut);
        for (got, want) in converted.ink.rgb().into_iter().zip(rgb) {
            assert!((got - want).abs() <= 0.5 / 255.0, "{got} vs {want}");
        }
    }

    #[test]
    fn a_colour_cmyk_cannot_draw_says_so() {
        let converted = Ink::from_srgb([0, 160, 255]).convert(Space::Cmyk);
        assert!(!converted.in_gamut);
        assert_eq!(converted.ink.space(), Space::Cmyk);
        assert!(Ink::Gray(0.5).convert(Space::Rgb).in_gamut);
    }

    #[test]
    fn an_srgb_byte_triple_round_trips() {
        for rgb in [[0, 0, 0], [255, 255, 255], [92, 92, 70], [1, 128, 254]] {
            assert_eq!(Ink::from_srgb(rgb).srgb(), Some(rgb));
        }
        assert_eq!(Ink::Gray(0.5).srgb(), None);
    }

    #[test]
    fn a_stored_colour_keeps_its_space_through_the_picker() {
        use pdfcer_core::annot_author::Color;
        for c in [
            Color::Gray(0.25),
            Color::Rgb(1.0, 0.5, 0.0),
            Color::Cmyk(0.0, 0.25, 0.5, 1.0),
        ] {
            assert_eq!(Color::from(Ink::from(c)), c);
        }
    }
}
