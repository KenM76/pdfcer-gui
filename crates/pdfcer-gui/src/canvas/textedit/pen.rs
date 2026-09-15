//! # `canvas::textedit::pen` — the face, size and colour **new** page text is
//! written in
//!
//! ## What this closes
//!
//! `FEATURES.md`'s Phase 5 row, verbatim:
//!
//! > **`edit.add_text` has no font, size or colour surface** — it arms, sets an
//! > origin, takes keystrokes and commits through `EditSession::add_text` with
//! > the engine's documented default face. Unit-tested, **not driven**.
//! > *Choosing what those three controls are is a decision, not an omission.*
//!
//! The decision is made here, and the operator's instruction of 2026-08-19 is
//! why it is made now rather than deferred again: *"finish off phase 1 and
//! phase 5. Get everything unblocked on phase 5 — no excuses."*
//!
//! Nothing in the engine was blocking it. `AddTextRequest` has carried `face`,
//! `size` and `color` since it shipped, and `apply.rs`'s arm passed
//! `AddTextRequest::new(…)` — *a bundled 12-pt black Helvetica run* — and
//! overrode none of them.
//!
//! ## ★ Why this is a TOOL option and not a Format-tab property
//!
//! `RIBBON_IA.md` §5.8 sends `Text run → Font · Size · Colour · Spacing ·
//! Alignment` to the Format tab, and that is right **for a run already on the
//! page**. This is the opposite question: *what will the next thing I type look
//! like*, which is the definition `crate::panels::tool` gives of its own
//! subject — *this panel is about the NEXT gesture; Properties and Format are
//! about the placed thing.*
//!
//! It is the same split `canvas::markup::pen` already makes, in the same words:
//! the Markup ▸ Style group sets the pen for the next markup, and changing a
//! placed one is Format's job.
//!
//! ## ★★ Why it lives in `egui::Memory` and the markup pen does not
//!
//! `canvas::markup::pen::Pen` is a field on `PdfcerApp`, and `panels::tool`'s
//! header records the consequence: a panel body is handed `&OpenDoc` and
//! `&mut PanelsState` and **nothing else**, so it cannot reach that pen — which
//! is why the markup swatch is still absent from the Tool panel and why the
//! honest interim there is to show nothing rather than a control that accepts a
//! click and discards it.
//!
//! This one is in `egui::Memory`, beside the armed tool and the measure tool's
//! authoring group, so **a panel can read and write it through `ui.ctx()` with
//! no plumbing at all**. That is the precedent `canvas::measure::set_active_group`
//! set and `panels::dimension_groups` already uses: transient UI state that
//! decides what the *next* gesture produces is not document state, contributes
//! nothing to the undo log, and has nothing to order against.
//!
//! It is **application-scoped**, like the armed tool itself: an operator who
//! picks 8 pt Courier for a note expects it still to be 8 pt Courier in the next
//! document, and a per-document reset would be a preference silently thrown
//! away.
//!
//! ## What is deliberately NOT here
//!
//! - **A donor font.** `NewTextFace::Embedded` takes a whole `FontEmbedPlan`,
//!   which is a font *file* the operator has to choose and pdfcer has to subset.
//!   That is a real feature with a file picker and a licensing disclosure, and
//!   it is what the `gui` column means by *"add non-Latin text via donor
//!   font"* — offering a fourteenth entry in this combo that opened a file
//!   dialog would be two features wearing one control.
//! - **Alignment and a wrap box.** `AddTextRequest` carries both, and both
//!   describe a *paragraph*. This gesture places a caret and takes keystrokes;
//!   there is no box to align inside, and inventing one would be inventing the
//!   text-box tool, which is `markup.text_box` and is a different command.

use pdfcer_core::fontdata::Std14;
use pdfcer_core::text_edit::NewTextColor;
use pdfcer_core::text_edit::addtext::NewTextFace;

/// The `egui::Memory` key the text pen is parked under.
const KEY: &str = "pdfcer.textedit.pen"; // ui-text-exempt: memory key, never displayed

/// What new page text is written in.
///
/// # Why the colour is `[u8; 3]` and not the engine's `NewTextColor`
///
/// Because the engine's type has a `Black` variant *and* an `Rgb` variant, and
/// `Rgb(0, 0, 0)` is a third spelling of the same ink. A control bound to it
/// would let an operator reach black two ways and would then have to decide
/// which one a colour picker produces — a distinction with no meaning on the
/// page.
///
/// So this holds the operator's answer in the one form a colour picker speaks,
/// and [`Self::colour`] resolves it, **preferring `Black`** where it can:
/// `0 g` is one operator and one byte where `0 0 0 rg` is four, and pdfcer's
/// standing preference is the smaller edit where the two are equivalent.
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
    ///
    /// ★ Matching it exactly is what makes this addition invisible to anybody
    /// who does not touch the controls. A shell whose default differed from the
    /// engine's would change what every existing operator's next Add-text
    /// produced, silently, on the release that added a control they had not
    /// asked for.
    fn default() -> Self {
        Self {
            face: Std14::Helvetica,
            size_pt: 12.0,
            colour: [0, 0, 0],
        }
    }
}

/// The smallest size offered.
///
/// Below about four points a Standard-14 face is unreadable at any zoom this
/// shell allows, so a smaller value would be a number an operator could set and
/// could not then find on the page.
pub const MIN_SIZE_PT: f64 = 4.0;

/// The largest.
///
/// A title block's largest text is rarely past 24 pt and a drawing title rarely
/// past 48; 144 is two inches, which is beyond any use this operator's
/// documents have and is a round number to stop at rather than a measured one.
pub const MAX_SIZE_PT: f64 = 144.0;

impl TextPen {
    /// The engine face this pen writes in.
    ///
    /// ★ Kept even though `apply` reaches for `AddTextRequest::with_font`,
    /// which takes a bare `Std14`. The two builders exist because
    /// `NewTextFace` has a second variant — `Embedded(Box<FontEmbedPlan>)` —
    /// and this is the accessor a donor-font surface will use when it lands.
    /// It is a **statement about the boundary**, and its test is what would
    /// notice if `Std14` stopped being expressible as a `NewTextFace`.
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
    ///
    /// Applied at **read** time rather than at write time, so a
    /// hand-edited or future value that falls outside the range is corrected
    /// where it is used rather than silently rewritten where it is stored —
    /// the same rule `app::prefs`' loader follows for a clamped setting.
    #[must_use]
    pub fn size(self) -> f64 {
        self.size_pt.clamp(MIN_SIZE_PT, MAX_SIZE_PT)
    }
}

/// Every bundled face, in the order a control lists them.
///
/// Grouped by family and then by weight — Helvetica, Times, Courier, Symbol,
/// ZapfDingbats — because that is how an operator looks for a face, and it is
/// the order Acrobat's own font list uses. **Not** alphabetical, which would
/// interleave the four Helveticas with the four Times.
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

    /// ★★ **The default is the engine's default**, so adding these controls
    /// changed nothing for anybody who does not touch them.
    ///
    /// A shell whose default differed would silently change what every
    /// operator's next Add-text produced, on a release that added a control
    /// they had not asked for. Asserted against the values `AddTextRequest::new`
    /// documents — Helvetica, 12 pt, black — because that is the claim.
    #[test]
    fn the_default_is_the_engines_default() {
        let p = TextPen::default();
        assert_eq!(p.face, Std14::Helvetica);
        assert!((p.size_pt - 12.0).abs() < f64::EPSILON);
        assert_eq!(p.engine_colour(), NewTextColor::Black);
    }

    /// ★ **Black resolves to `Black`, not to `Rgb(0, 0, 0)`.**
    ///
    /// One operator and one byte instead of four, for the same ink. The
    /// property is worth a test rather than a comment because a colour picker
    /// hands back `[0, 0, 0]` and the obvious implementation forwards it.
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

    /// ★ **All fourteen bundled faces are offered, each exactly once.**
    ///
    /// The count is the claim: `Std14` has fourteen members, and a list that
    /// quietly held thirteen would be a face an operator could never reach
    /// with no error anywhere. Asserted as a set as well as a length, so a
    /// duplicate cannot make up the number.
    #[test]
    fn every_bundled_face_is_offered_once() {
        let mut seen = std::collections::BTreeSet::new();
        for f in FACES {
            assert!(seen.insert(format!("{f:?}")), "{f:?} is listed twice");
        }
        assert_eq!(FACES.len(), 14, "Std14 has fourteen members");
    }
}
