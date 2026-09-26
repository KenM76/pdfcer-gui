//! # `editmodel::pen` — the face, size and colour **new** page text is
//! written in
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/editmodel/pen.md`.

use pdfcer_core::fontdata::Std14;
use pdfcer_core::text_edit::NewTextColor;
use pdfcer_core::text_edit::addtext::NewTextFace;

/// The `egui::Memory` key the text pen is parked under.
const KEY: &str = "pdfcer.textedit.pen"; // ui-text-exempt: memory key, never displayed

/// What new page text is written in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextPen {
    /// Which of the fourteen bundled faces.
    pub face: Std14,
    /// The type size in points.
    pub size_pt: f64,
    /// The ink, as sRGB bytes.
    pub colour: [u8; 3],
}

impl Default for TextPen {
    /// **The engine's own documented default**, restated here rather than
    /// invented: `AddTextRequest::new` is *"Helvetica, bundled, 12 pt,
    /// black"*.
    fn default() -> Self {
        Self {
            face: Std14::Helvetica,
            size_pt: 12.0,
            colour: [0, 0, 0],
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

    /// The engine colour, preferring `Black` over an equivalent `Rgb`.
    ///
    /// See the struct's own note: `0 g` is one operator where `0 0 0 rg` is
    /// four, and they draw the same ink.
    #[must_use]
    pub fn engine_colour(self) -> NewTextColor {
        if self.colour == [0, 0, 0] {
            NewTextColor::Black
        } else {
            NewTextColor::Rgb(
                f64::from(self.colour[0]) / 255.0,
                f64::from(self.colour[1]) / 255.0,
                f64::from(self.colour[2]) / 255.0,
            )
        }
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
        let red = TextPen {
            colour: [255, 0, 0],
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
