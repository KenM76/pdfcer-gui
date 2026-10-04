//! # `markuppen` — the colour and width the next markup is authored with
//!
//! ## What this closes
//!
//! `RIBBON_IA.md` §5.5 specifies a **Style** group on the Markup tab —
//! *"Colour · Line width · Fill · Opacity"* — and marks it `partial G`,
//! *"colour only"*, describing the **old** shell. This shell had none of it:
//! `MarkupKind::rgb()` returned a hard-coded red, `PEN_WIDTH_PTS` was a
//! hard-coded `2.0`, and the manifest's `colour_swatch` item was declared and
//! never built, so the Style group rendered an empty caption.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/markuppen.md`.

use egui::Color32;

use crate::markupkind::MarkupKind;
use crate::markuppalette as palette;

/// **Which pen** — one variant per default Acrobat keeps a separate key for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PenSlot {
    /// Every kind drawn as **linework by a pointer gesture**: rectangle,
    /// ellipse, arrow, polyline, polygon, revision cloud, freehand.
    ///
    /// One slot for seven kinds because Acrobat holds one value across their
    /// seven keys. Called `Shape` rather than `Ink` because [`Pen::ink`] is the
    /// field and `MarkupKind::Ink` is the *freehand* kind — three meanings of
    /// one word, and the enum is where they are told apart.
    Shape,
    /// The highlight band, whether drawn as an area or over found text.
    Highlighter,
    /// `/Underline`.
    Underline,
    /// `/StrikeOut`.
    StrikeOut,
    /// `/Squiggly`.
    ///
    /// Ships at the same red as [`Self::Shape`] and is a separate slot anyway —
    /// see the module header on why two slots that agree today are not one slot.
    Squiggly,
    /// `/Text` — the sticky note.
    Note,
    /// `/FreeText` — the text box.
    ///
    /// Acrobat splits this in two and this shell cannot. `cFreeText` holds a
    /// *border* colour (`#F86464`) and a *text* colour (`#DB3425`), and
    /// `canvas::textannot` authors one `ink` used for both. This slot ships at
    /// Acrobat's **text** colour, because the words are what the operator reads
    /// and a frame is a frame. The divergence is real and is written down here
    /// rather than smoothed over: a pdfcer text box has an Acrobat-red frame
    /// where Acrobat's would be pink.
    TextBox,
    /// `/Stamp` — a framed label.
    Stamp,
    /// `/Caret` — insert and replace text.
    Caret,
}

impl PenSlot {
    /// Every slot, in the order the module header's table lists them.
    pub const ALL: &'static [PenSlot] = &[
        PenSlot::Shape,
        PenSlot::Highlighter,
        PenSlot::Underline,
        PenSlot::StrikeOut,
        PenSlot::Squiggly,
        PenSlot::Note,
        PenSlot::TextBox,
        PenSlot::Stamp,
        PenSlot::Caret,
    ];

    /// **Which pen draws this kind.**
    ///
    #[must_use]
    pub const fn of(kind: MarkupKind) -> Self {
        match kind {
            MarkupKind::Highlight => Self::Highlighter,
            MarkupKind::Rectangle
            | MarkupKind::Ellipse
            | MarkupKind::Arrow
            | MarkupKind::PolyLine
            | MarkupKind::Polygon
            | MarkupKind::Cloud
            | MarkupKind::Ink => Self::Shape,
        }
    }

    /// **Which pen writes this text annotation.**
    ///
    #[must_use]
    pub const fn of_text_annot(kind: crate::wordmarkup::TextAnnotKind) -> Self {
        match kind {
            crate::wordmarkup::TextAnnotKind::TextBox => Self::TextBox,
            crate::wordmarkup::TextAnnotKind::Sticky
            | crate::wordmarkup::TextAnnotKind::Attachment => Self::Note,
            crate::wordmarkup::TextAnnotKind::Stamp => Self::Stamp,
            crate::wordmarkup::TextAnnotKind::Caret => Self::Caret,
        }
    }
}

/// The colour and width the next markup gesture will be authored with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pen {
    /// The comment-linework colour, as PDF `/DeviceRGB` components in
    /// `0.0..=1.0`.
    ///
    /// **DOCUMENT COLOUR.** This is written into the annotation's `/C` and
    /// therefore into the saved file — restyling the application must never
    /// move it, which is the case the theme gate's escape hatch exists for.
    pub ink: (f64, f64, f64),
    /// The highlight band's colour, same units and the same warning.
    pub highlighter: (f64, f64, f64),
    /// `/Underline`'s colour — [`PenSlot::Underline`]. Same units, same warning.
    pub underline: (f64, f64, f64),
    /// `/StrikeOut`'s colour — [`PenSlot::StrikeOut`].
    pub strike_out: (f64, f64, f64),
    /// `/Squiggly`'s colour — [`PenSlot::Squiggly`].
    pub squiggly: (f64, f64, f64),
    /// The sticky note's colour — [`PenSlot::Note`].
    pub note: (f64, f64, f64),
    /// The text box's border **and** painted text — [`PenSlot::TextBox`]. See
    /// that variant on why one value serves two of Acrobat's.
    pub text_box: (f64, f64, f64),
    /// The stamp's colour — [`PenSlot::Stamp`].
    pub stamp: (f64, f64, f64),
    /// The caret's colour — [`PenSlot::Caret`].
    pub caret: (f64, f64, f64),
    /// Border and stroke width, in PDF points.
    ///
    /// Clamped to [`MIN_WIDTH_PTS`]`..=`[`MAX_WIDTH_PTS`] by the control that
    /// sets it — see those constants for why the range is what it is.
    pub width_pts: f64,
    /// **The annotation's constant opacity, `/CA`** — `0.0`–`1.0`, where
    /// `1.0` is fully opaque and is the default.
    ///
    /// # This field is why the mark can be seen THROUGH
    ///
    /// A comment on an engineering drawing sits on top of the thing it is about.
    /// An opaque cloud round a dimension hides the dimension; a 40% one does
    /// not. That is the whole use, and it is the reason the operator's own
    /// argument against a **fill** does not apply here — a translucent outline
    /// obscures nothing.
    ///
    /// # `1.0` writes no key at all, and that is deliberate
    ///
    /// [`Self::opacity_option`] answers `None` at `1.0`. §12.5.2 Table 164 makes
    /// 1.0 the default, so writing it explicitly would add a key that changes
    /// nothing and make a pdfcer-authored opaque annotation textually different
    /// from every other producer's — which is the engine's own reasoning on
    /// `MarkupOptions::opacity`, adopted rather than re-derived.
    ///
    /// ⇒ It also keeps the standing rule for a capability becoming choosable:
    /// **a build which omits nothing must behave as it did before the choice
    /// existed**, byte for byte.
    ///
    /// # Clamped by the control, refused by the engine
    ///
    /// [`MIN_OPACITY`]`..=1.0` at the widget. The engine **refuses** an
    /// out-of-range author-time alpha by name rather than clamping it, because
    /// *"quietly authoring 1.0 would put an opaque annotation on the page while
    /// reporting success"* — so a value that escaped this range would produce a
    /// refusal, not a silent surprise.
    pub opacity: f64,
    /// **The border line style the next mark is drawn in** — `/BS` `/S` and
    /// `/D` (§12.5.4, Table 166).
    ///
    /// # It is a [`crate::linestyle::LineStyle`] and not a `BorderDash`
    ///
    /// Because that type is `Copy` and the engine's is not — see its header,
    /// which carries the whole argument and the reason this struct's `Copy` is
    /// load-bearing. [`Self::dash_option`] is the boundary that builds the
    /// engine's value.
    ///
    /// # Solid ships, and that keeps the standing rule
    ///
    /// [`Self::opacity`]'s doc states the rule this project applies when a
    /// capability becomes choosable: *"a build which omits nothing must behave
    /// as it did before the choice existed, byte for byte."* Unlike the colour
    /// change of 2026-09-06, this one **does** keep it: the default is
    /// [`crate::linestyle::LineStyle::Solid`], `dash_option` answers `None`,
    /// and `MarkupOptions::dash: None` authors *"the solid border pdfcer
    /// authored exclusively before `Pass 258.0`"*. An operator who never opens
    /// the chooser gets the file they got yesterday.
    ///
    /// # ⚠ Ignored by the text-markup family, and that is the format
    ///
    /// A highlight is a colour wash and an underline is its own line; neither
    /// draws a `/BS` border, so `MarkupOptions::dash` is ignored for all four.
    /// The pen carries one value and the highlighter shares it, so the
    /// chooser's tooltip says so — [`crate::text::markup::pen_dash_tooltip`] —
    /// rather than leaving an operator to conclude the setting did not take.
    pub dash: crate::linestyle::LineStyle,
    /// **Draw into the page's own content instead of as a comment.**
    ///
    /// Off by default, so an operator who never touches the switch authors
    /// annotations exactly as before. On, a shape or text markup is committed
    /// through `EditSession::add_markup_as_content`: the same appearance, as
    /// ordinary page objects the object tool moves and deletes. Text boxes,
    /// sticky notes and stamps have no such form and stay comments.
    pub on_page: bool,
}

/// The thinnest pen offered.
pub const MIN_WIDTH_PTS: f64 = 0.25;

/// **The most transparent mark offered**, as a fraction.
pub const MIN_OPACITY: f64 = 0.1;

/// The thickest pen offered.
pub const MAX_WIDTH_PTS: f64 = 12.0;

impl Default for Pen {
    /// **The shipped pen: Acrobat's own eight defaults, at 2 pt, opaque.**
    fn default() -> Self {
        Self {
            // DOCUMENT COLOUR: Acrobat's shape-tool red, written into `/C`.
            ink: palette::components(palette::MARKUP_RED),
            // DOCUMENT COLOUR: Acrobat's highlighter — ORANGE, measured, see
            // `palette`'s header for why that is not the mistake it looks like.
            highlighter: palette::components(palette::HIGHLIGHTER_ORANGE),
            // DOCUMENT COLOUR: Acrobat's `cUnderline`.
            underline: palette::components(palette::UNDERLINE_BLUE),
            // DOCUMENT COLOUR: Acrobat's `cStrikeOut`.
            strike_out: palette::components(palette::STRIKEOUT_PINK),
            // DOCUMENT COLOUR: Acrobat's `cSquiggly` — the same red as the shape
            // pen, under its own key, so it is its own slot.
            squiggly: palette::components(palette::MARKUP_RED),
            // DOCUMENT COLOUR: Acrobat's `cText` — the sticky note's violet.
            note: palette::components(palette::NOTE_PURPLE),
            // DOCUMENT COLOUR: Acrobat's `cFreeText` TEXT colour. See
            // `PenSlot::TextBox` on why the text colour and not the border's.
            text_box: palette::components(palette::MARKUP_RED),
            // DOCUMENT COLOUR: Acrobat's `cStamp`.
            stamp: palette::components(palette::MARKUP_RED),
            // DOCUMENT COLOUR: Acrobat's `cCaret`.
            caret: palette::components(palette::CARET_MAGENTA),
            width_pts: 2.0,
            opacity: 1.0,
            // Solid, which writes no dash at all — see the field's own doc
            // comment. Unlike the eight colours above, this default DOES keep
            // the "omits nothing" rule: a build whose operator never opens the
            // chooser authors the same bytes it authored before the chooser
            // existed.
            dash: crate::linestyle::LineStyle::Solid,
            on_page: false,
        }
    }
}

impl Pen {
    /// **The freehand simplification tolerance this pen implies**, in PDF
    /// points — a quarter of the stroke width.
    #[must_use]
    pub fn simplify_tolerance_pts(self) -> f32 {
        (self.width_pts as f32) / 4.0
    }

    /// The colour this kind is authored in.
    #[must_use]
    pub fn colour_for(self, kind: MarkupKind) -> (f64, f64, f64) {
        self.colour_of(PenSlot::of(kind))
    }

    /// **The colour in one slot**, as PDF `/DeviceRGB` components.
    #[must_use]
    pub fn colour_of(self, slot: PenSlot) -> (f64, f64, f64) {
        match slot {
            PenSlot::Shape => self.ink,
            PenSlot::Highlighter => self.highlighter,
            PenSlot::Underline => self.underline,
            PenSlot::StrikeOut => self.strike_out,
            PenSlot::Squiggly => self.squiggly,
            PenSlot::Note => self.note,
            PenSlot::TextBox => self.text_box,
            PenSlot::Stamp => self.stamp,
            PenSlot::Caret => self.caret,
        }
    }

    /// **Set one slot's colour** from a screen colour, discarding alpha.
    pub fn set_colour(&mut self, slot: PenSlot, colour: Color32) {
        let rgb = rgb_of(colour);
        match slot {
            PenSlot::Shape => self.ink = rgb,
            PenSlot::Highlighter => self.highlighter = rgb,
            PenSlot::Underline => self.underline = rgb,
            PenSlot::StrikeOut => self.strike_out = rgb,
            PenSlot::Squiggly => self.squiggly = rgb,
            PenSlot::Note => self.note = rgb,
            PenSlot::TextBox => self.text_box = rgb,
            PenSlot::Stamp => self.stamp = rgb,
            PenSlot::Caret => self.caret = rgb,
        }
    }

    /// One slot's colour as a screen colour, for the swatch that shows it.
    #[must_use]
    pub fn color32_of(self, slot: PenSlot) -> Color32 {
        color32_of(self.colour_of(slot))
    }

    /// **The colour a sticky note, text box or stamp is authored in.**
    #[must_use]
    pub fn text_annot_colour(self, kind: crate::wordmarkup::TextAnnotKind) -> (f64, f64, f64) {
        self.colour_of(PenSlot::of_text_annot(kind))
    }

    /// Set the ink colour from a screen colour, discarding alpha.
    pub fn set_ink(&mut self, colour: Color32) {
        self.set_colour(PenSlot::Shape, colour);
    }

    /// **The `/CA` value to author with, or `None` for "write no key".**
    #[must_use]
    pub fn opacity_option(&self) -> Option<f64> {
        (self.opacity < 1.0).then_some(self.opacity)
    }

    /// **The `/BS` dash to author with, or `None` for "write a solid border".**
    #[must_use]
    pub fn dash_option(&self) -> Option<pdfcer_core::annot_author::BorderDash> {
        self.dash.dash()
    }

    /// Set the highlighter colour from a screen colour. As [`Self::set_ink`].
    pub fn set_highlighter(&mut self, colour: Color32) {
        self.set_colour(PenSlot::Highlighter, colour);
    }

    /// The ink colour as a screen colour, for the swatch that sets it.
    #[must_use]
    pub fn ink_color32(self) -> Color32 {
        self.color32_of(PenSlot::Shape)
    }

    /// The highlighter colour as a screen colour.
    #[must_use]
    pub fn highlighter_color32(self) -> Color32 {
        self.color32_of(PenSlot::Highlighter)
    }
}

/// PDF components from a screen colour.
fn rgb_of(c: Color32) -> (f64, f64, f64) {
    (
        f64::from(c.r()) / 255.0,
        f64::from(c.g()) / 255.0,
        f64::from(c.b()) / 255.0,
    )
}

/// A screen colour from PDF components.
fn color32_of((r, g, b): (f64, f64, f64)) -> Color32 {
    let byte = |v: f64| {
        // `clamp` first: a settings file or a future loader could hand this a
        // component outside the range, and `as u8` on an out-of-range float is
        // a saturating cast in Rust but on a NaN produces 0 — a silent black.
        // Clamping states the intent instead of relying on that.
        let scaled = (v.clamp(0.0, 1.0) * 255.0).round();
        // The value is provably in `0..=255` and finite, so this cast is exact.
        scaled as u8
    };
    // DOCUMENT COLOUR: the operator's own pen, arriving from the file's
    // `/DeviceRGB` components rather than from the palette. No theme may move
    // it — a restyle that changed the swatch would be claiming the annotation
    // had changed colour, and the annotation is in the document.
    Color32::from_rgb(byte(r), byte(g), byte(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The shipped pen is Acrobat's, slot for slot.**
    #[test]
    fn every_slot_ships_at_the_acrobat_value_it_was_measured_from() {
        let pen = Pen::default();
        let expected = [
            (PenSlot::Shape, palette::MARKUP_RED),
            (PenSlot::Highlighter, palette::HIGHLIGHTER_ORANGE),
            (PenSlot::Underline, palette::UNDERLINE_BLUE),
            (PenSlot::StrikeOut, palette::STRIKEOUT_PINK),
            (PenSlot::Squiggly, palette::MARKUP_RED),
            (PenSlot::Note, palette::NOTE_PURPLE),
            (PenSlot::TextBox, palette::MARKUP_RED),
            (PenSlot::Stamp, palette::MARKUP_RED),
            (PenSlot::Caret, palette::CARET_MAGENTA),
        ];
        assert_eq!(
            expected.len(),
            PenSlot::ALL.len(),
            "a slot was added and this table was not told about it"
        );
        for (slot, bytes) in expected {
            assert_eq!(
                pen.colour_of(slot),
                palette::components(bytes),
                "{slot:?} does not ship at the Acrobat value it is documented as"
            );
        }
        assert!((pen.width_pts - 2.0).abs() < f64::EPSILON);
    }

    /// **The highlighter is ORANGE, and that is not a typo.**
    #[test]
    fn the_highlighter_is_acrobats_orange_and_not_the_old_yellow() {
        let pen = Pen::default();
        assert_eq!(pen.highlighter, palette::components([255, 98, 0]));
        assert_ne!(
            pen.highlighter,
            (1.0, 1.0, 0.0),
            "the highlighter has been put back to this shell's old invented \
             yellow — Acrobat's is #FF6200, measured from cHighlight, and the \
             operator asked for Adobe's value. Yellow is still one click away \
             in the palette as `CLASSIC_YELLOW`."
        );
    }

    /// **Every markup kind reaches a slot, and the geometric family shares
    /// one.**
    #[test]
    fn every_kind_takes_the_slot_it_is_documented_to_take() {
        let pen = planted();
        for kind in MarkupKind::ALL {
            let slot = PenSlot::of(*kind);
            let expected = if matches!(kind, MarkupKind::Highlight) {
                PenSlot::Highlighter
            } else {
                PenSlot::Shape
            };
            assert_eq!(slot, expected, "{kind:?} took the wrong pen");
            assert_eq!(
                pen.colour_for(*kind),
                pen.colour_of(slot),
                "{kind:?}'s colour did not come from its own slot"
            );
        }
    }

    /// **Every slot is separately settable, and setting one moves nothing
    /// else.**
    #[test]
    fn setting_one_slot_leaves_the_other_seven_alone() {
        for target in PenSlot::ALL {
            let mut pen = planted();
            let before = pen_slots(&pen);
            // NOT A THEME COLOUR: an arbitrary value distinct from every
            // planted one, so "did this slot move" is unambiguous.
            pen.set_colour(*target, Color32::from_rgb(1, 2, 3));
            assert_eq!(
                pen.colour_of(*target),
                (1.0 / 255.0, 2.0 / 255.0, 3.0 / 255.0),
                "{target:?} did not take the colour it was given"
            );
            for (i, other) in PenSlot::ALL.iter().enumerate() {
                if other == target {
                    continue;
                }
                assert_eq!(
                    pen.colour_of(*other),
                    before[i],
                    "setting {target:?} also moved {other:?}"
                );
            }
        }
    }

    /// The three text-annotation kinds land on three different slots.
    #[test]
    fn the_three_text_annotation_kinds_do_not_share_a_pen() {
        use crate::wordmarkup::TextAnnotKind;
        // The attachment marker is an icon on the page like a note's, and takes
        // the note's colour by design; it is pinned to that slot below.
        let kinds: Vec<TextAnnotKind> = TextAnnotKind::ALL
            .iter()
            .copied()
            .filter(|k| *k != TextAnnotKind::Attachment)
            .collect();
        let slots: Vec<PenSlot> = kinds.iter().map(|k| PenSlot::of_text_annot(*k)).collect();
        for i in 0..slots.len() {
            for j in (i + 1)..slots.len() {
                assert_ne!(
                    slots[i], slots[j],
                    "{:?} and {:?} share a pen slot",
                    kinds[i], kinds[j]
                );
            }
        }
        // …and the sticky note is on the note slot specifically, which is the
        // one whose Acrobat value differs from the shape pen's.
        assert_eq!(PenSlot::of_text_annot(TextAnnotKind::Sticky), PenSlot::Note);
        assert_eq!(
            PenSlot::of_text_annot(TextAnnotKind::Attachment),
            PenSlot::Note
        );
        assert_eq!(
            Pen::default().text_annot_colour(TextAnnotKind::Sticky),
            palette::components(palette::NOTE_PURPLE)
        );
    }

    /// **THE TOLERANCE FOLLOWS THE WIDTH.**
    #[test]
    fn the_tolerance_follows_the_width() {
        for width in [MIN_WIDTH_PTS, 0.5, 2.0, 7.5, MAX_WIDTH_PTS] {
            let pen = Pen {
                width_pts: width,
                ..Pen::default()
            };
            let expected = (width as f32) / 4.0;
            assert!(
                (pen.simplify_tolerance_pts() - expected).abs() < 1e-6,
                "at {width} pt the tolerance is {} and must be {expected} — a \
                 quarter of the width, so the simplified centreline stays \
                 inside the stroke",
                pen.simplify_tolerance_pts()
            );
        }
    }

    /// A pen with eight distinguishable colours, one per slot.
    fn planted() -> Pen {
        let mut pen = Pen::default();
        for (i, slot) in PenSlot::ALL.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let step = (i as u8) * 16 + 8;
            // NOT A THEME COLOUR: eight distinguishable test values, so an
            // assertion says which slot was taken rather than which default
            // happened to match.
            pen.set_colour(*slot, Color32::from_rgb(step, step, step));
        }
        pen
    }

    /// Every slot's colour, in [`PenSlot::ALL`] order.
    fn pen_slots(pen: &Pen) -> Vec<(f64, f64, f64)> {
        PenSlot::ALL.iter().map(|s| pen.colour_of(*s)).collect()
    }

    /// A colour survives the round trip through the picker unchanged.
    #[test]
    fn a_colour_round_trips_through_the_swatch() {
        // DOCUMENT COLOUR: four `/DeviceRGB` values round-tripping through the
        // picker. Nothing here is chrome and no theme is involved; the endpoints
        // are chosen to catch an off-by-one scale factor.
        for original in [
            Color32::from_rgb(0, 0, 0),
            Color32::from_rgb(255, 255, 255),
            Color32::from_rgb(217, 41, 41),
            Color32::from_rgb(1, 128, 254),
        ] {
            let mut pen = Pen::default();
            pen.set_ink(original);
            assert_eq!(
                pen.ink_color32(),
                original,
                "a colour changed on its way to the document and back"
            );
        }
    }

    /// An out-of-range component clamps rather than wrapping to a nonsense
    /// colour.
    #[test]
    fn an_impossible_component_clamps_instead_of_wrapping() {
        // DOCUMENT COLOUR: expected `/DeviceRGB` results, not palette entries.
        assert_eq!(color32_of((2.0, -1.0, 0.5)), Color32::from_rgb(255, 0, 128));
        assert_eq!(
            color32_of((f64::NAN, 1.0, 0.0)),
            Color32::from_rgb(0, 255, 0)
        );
    }

    /// The shipped width is reachable on the control that sets it.
    #[test]
    fn the_shipped_width_is_reachable_on_its_own_control() {
        const {
            assert!(MIN_WIDTH_PTS > 0.0, "a zero width is device-dependent");
            assert!(MAX_WIDTH_PTS > MIN_WIDTH_PTS);
        }
        assert!(
            (MIN_WIDTH_PTS..=MAX_WIDTH_PTS).contains(&Pen::default().width_pts),
            "the shipped width is not reachable on its own control"
        );
    }
}
