//! # `editmodel::pen` — the face, size, colour and opacity **new** page text
//! is written in
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/editmodel/pen.md`.

use pdfcer_core::fontdata::Std14;
use pdfcer_core::text_edit::NewTextColor;
use pdfcer_core::text_edit::addtext::NewTextFace;

use crate::colourpick::Ink;

/// The `egui::Memory` key the text pen is parked under.
const KEY: &str = "pdfcer.textedit.pen"; // ui-text-exempt: memory key, never displayed

/// What new page text is written in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextPen {
    /// Which of the fourteen bundled faces.
    pub face: Std14,
    /// The type size in points.
    pub size_pt: f64,
    /// The ink, in the device space the operator picked it in.
    pub colour: Ink,
    /// The fill opacity, `0.0..=1.0`; `1.0` is opaque.
    pub opacity: f32,
    /// Written in rendering mode 3: present, searchable and copyable, and
    /// never drawn, which is how a scan's recognised text is stored.
    pub invisible: bool,
}

impl Default for TextPen {
    /// **The engine's own documented default**, restated here rather than
    /// invented: `AddTextRequest::new` is *"Helvetica, bundled, 12 pt,
    /// black"*, at full opacity.
    fn default() -> Self {
        Self {
            face: Std14::Helvetica,
            size_pt: 12.0,
            colour: Ink::Gray(0.0),
            opacity: 1.0,
            invisible: false,
        }
    }
}

/// The smallest size offered.
pub const MIN_SIZE_PT: f64 = 4.0;

/// The largest.
pub const MAX_SIZE_PT: f64 = 144.0;

impl TextPen {
    /// The engine face this pen writes in.
    #[must_use]
    pub fn engine_face(self) -> NewTextFace {
        NewTextFace::Std14(self.face)
    }

    /// The engine colour in the pen's own space, preferring `Black` for grey
    /// or RGB black (`0 g` is one operator where `0 0 0 rg` is four).
    #[must_use]
    pub fn engine_colour(self) -> NewTextColor {
        let f = f64::from;
        match self.colour {
            Ink::Gray(v) if v <= 0.0 => NewTextColor::Black,
            Ink::Rgb([r, g, b]) if r <= 0.0 && g <= 0.0 && b <= 0.0 => NewTextColor::Black,
            Ink::Gray(v) => NewTextColor::Gray(f(v)),
            Ink::Rgb([r, g, b]) => NewTextColor::Rgb(f(r), f(g), f(b)),
            Ink::Cmyk([c, m, y, k]) => NewTextColor::Cmyk(f(c), f(m), f(y), f(k)),
        }
    }

    /// The fill opacity to write, or `None` when opaque so the run carries
    /// no graphics state.
    #[must_use]
    pub fn fill_alpha(self) -> Option<f64> {
        (self.opacity < 1.0).then(|| f64::from(self.opacity.clamp(0.0, 1.0)))
    }

    /// The rendering mode the engine writes: 3 when invisible, else 0.
    #[must_use]
    pub const fn render_mode(self) -> u8 {
        if self.invisible { 3 } else { 0 }
    }

    /// The size, clamped to what the controls offer.
    #[must_use]
    pub fn size(self) -> f64 {
        self.size_pt.clamp(MIN_SIZE_PT, MAX_SIZE_PT)
    }
}

/// Every bundled face, in the order a control lists them.
pub const FACES: &[Std14] = &[
    Std14::Helvetica,
    Std14::HelveticaBold,
    Std14::HelveticaOblique,
    Std14::HelveticaBoldOblique,
    Std14::TimesRoman,
    Std14::TimesBold,
    Std14::TimesItalic,
    Std14::TimesBoldItalic,
    Std14::Courier,
    Std14::CourierBold,
    Std14::CourierOblique,
    Std14::CourierBoldOblique,
    Std14::Symbol,
    Std14::ZapfDingbats,
];

/// Read the pen, or the default if nothing has been chosen.
#[must_use]
pub fn read(ctx: &egui::Context) -> TextPen {
    ctx.data(|d| d.get_temp::<TextPen>(egui::Id::new(KEY)))
        .unwrap_or_default()
}

/// Write it back.
pub fn store(ctx: &egui::Context, pen: TextPen) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(KEY), pen));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The default is the engine's default**, so adding these controls
    /// changed nothing for anybody who does not touch them.
    #[test]
    fn the_default_is_the_engines_default() {
        let p = TextPen::default();
        assert_eq!(p.face, Std14::Helvetica);
        assert!((p.size_pt - 12.0).abs() < f64::EPSILON);
        assert_eq!(p.engine_colour(), NewTextColor::Black);
    }

    /// **Black resolves to `Black`, not to `Rgb(0, 0, 0)`.**
    #[test]
    fn black_ink_is_written_as_black() {
        assert_eq!(TextPen::default().engine_colour(), NewTextColor::Black);
        let rgb_black = TextPen {
            colour: Ink::Rgb([0.0; 3]),
            ..TextPen::default()
        };
        assert_eq!(rgb_black.engine_colour(), NewTextColor::Black);
        let red = TextPen {
            colour: Ink::Rgb([1.0, 0.0, 0.0]),
            ..TextPen::default()
        };
        match red.engine_colour() {
            NewTextColor::Rgb(r, g, b) => {
                assert!((r - 1.0).abs() < 1e-9);
                assert!(g.abs() < 1e-9);
                assert!(b.abs() < 1e-9);
            }
            other => panic!("red must be Rgb, got {other:?}"),
        }
    }

    /// Grey and CMYK reach the engine in their own space.
    #[test]
    fn cmyk_and_grey_are_written_in_their_own_space() {
        let grey = TextPen {
            colour: Ink::Gray(0.5),
            ..TextPen::default()
        };
        assert_eq!(grey.engine_colour(), NewTextColor::Gray(0.5));
        let cmyk_black = TextPen {
            colour: Ink::Cmyk([0.0, 0.0, 0.0, 1.0]),
            ..TextPen::default()
        };
        assert_eq!(
            cmyk_black.engine_colour(),
            NewTextColor::Cmyk(0.0, 0.0, 0.0, 1.0)
        );
    }

    /// An opaque pen writes no opacity; a translucent one writes its own.
    #[test]
    fn an_opaque_pen_writes_no_opacity() {
        assert_eq!(TextPen::default().fill_alpha(), None);
        let half = TextPen {
            opacity: 0.5,
            ..TextPen::default()
        };
        assert_eq!(half.fill_alpha(), Some(0.5));
    }

    /// The size is clamped where it is READ, so an out-of-range stored value
    /// cannot reach the engine.
    #[test]
    fn the_size_is_clamped_on_the_way_out() {
        let tiny = TextPen {
            size_pt: 0.01,
            ..TextPen::default()
        };
        assert!((tiny.size() - MIN_SIZE_PT).abs() < f64::EPSILON);
        let huge = TextPen {
            size_pt: 10_000.0,
            ..TextPen::default()
        };
        assert!((huge.size() - MAX_SIZE_PT).abs() < f64::EPSILON);
    }

    /// **All fourteen bundled faces are offered, each exactly once.**
    #[test]
    fn every_bundled_face_is_offered_once() {
        let mut seen = std::collections::BTreeSet::new();
        for f in FACES {
            assert!(seen.insert(format!("{f:?}")), "{f:?} is listed twice");
        }
        assert_eq!(FACES.len(), 14, "Std14 has fourteen members");
    }
}
