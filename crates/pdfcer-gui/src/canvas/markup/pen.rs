//! # `canvas::markup::pen` — the colour and width the next markup is authored with
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
//! §5.5's own note on it is the operator's complaint restated in advance:
//!
//! > The `Style` group sets defaults for the next markup. … Both must exist;
//! > today only the first does, **which is why a placed markup feels final**.
//!
//! ## ★ The seam was already named, and this took it
//!
//! `MarkupKind::rgb`'s doc comment predicted this module almost exactly:
//!
//! > The old shell carried `markup_color`/`markup_width` on the application
//! > and a swatch in its ribbon. This shell has neither … So the pen is a
//! > **default**, stated once, in the one place that builds a spec, and the
//! > seam for a real pen control is exactly this function: **give it a colour
//! > and a width from the document's markup state and nothing else in the
//! > module changes.**
//!
//! That is what happened: `spec` and `action` gained a `Pen` parameter and
//! nothing else in `canvas::markup` moved. The prediction was right down to
//! the shape of the change, which is worth recording — a doc comment that
//! names its own seam is the cheapest refactoring aid this project has.
//!
//! ## Why the pen lives on the application and not on the document
//!
//! A pen is a **tool setting**, not document content. An operator who picks
//! green expects the next rectangle to be green in *whatever* file they draw
//! it in, exactly as a pencil does not change colour when you turn the page.
//! Putting it on `OpenDoc` would reset it on every open, which is the
//! behaviour of a program that has forgotten what you told it.
//!
//! It is deliberately **not persisted** to the settings file, and that is a
//! narrower statement than it sounds. `pdfcer_core::settings` is for choices the
//! *standard* leaves open — its window says so in its own first paragraph —
//! and a pen colour is not an ambiguity, it is a preference. Persisting it
//! belongs with the ribbon layout and the keymap, under the same `userdata/`
//! roof and in their own file, which is `SHELL_FRAMEWORK.md`'s subject rather
//! than this one's.
//!
//! ## ★ Two pens, not one, and it is not a per-kind palette
//!
//! [`Pen::ink`] is the comment-linework colour; [`Pen::highlighter`] is the
//! highlight band's. They are separate because they answer different
//! questions — *what colour is my pen?* and *what colour is my highlighter?* —
//! and an operator who sets the pen to green does not thereby want a green
//! highlight, any more than picking a green biro changes the marker in their
//! other hand.
//!
//! What this is **not** is a colour per markup kind. `MarkupKind::rgb`'s own
//! note argued that down when three kinds were added: they are comment
//! linework, they have to be seen against a drawing that is already black on
//! white, and *"a per-kind palette would be a style decision made in code where
//! the Style group is the surface that owns it."* One pen for every geometric
//! kind is the same answer, now made choosable.

use egui::Color32;

use super::MarkupKind;

/// The colour and width the next markup gesture will be authored with.
///
/// `Copy`, and small enough that it is passed by value everywhere. That is
/// deliberate: a pen borrowed from the application while a gesture is being
/// committed would conflict with the `&mut self` the commit needs, and the
/// alternative — cloning at each call — would be the same bytes with a
/// question about whether the copy is stale.
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
    /// Border and stroke width, in PDF points.
    ///
    /// Clamped to [`MIN_WIDTH_PTS`]`..=`[`MAX_WIDTH_PTS`] by the control that
    /// sets it — see those constants for why the range is what it is.
    pub width_pts: f64,
    /// ★★★ **The annotation's constant opacity, `/CA`** — `0.0`–`1.0`, where
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
    /// # ★★ `1.0` writes no key at all, and that is deliberate
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
    /// # ★ Clamped by the control, refused by the engine
    ///
    /// [`MIN_OPACITY`]`..=1.0` at the widget. The engine **refuses** an
    /// out-of-range author-time alpha by name rather than clamping it, because
    /// *"quietly authoring 1.0 would put an opaque annotation on the page while
    /// reporting success"* — so a value that escaped this range would produce a
    /// refusal, not a silent surprise.
    pub opacity: f64,
}

/// The thinnest pen offered.
///
/// **A quarter point, not zero.** Zero is a legal PDF border width and means
/// *"the thinnest line the device can draw"*, which on a 2400 dpi plotter is
/// invisible and on screen is one pixel — so it is a width whose appearance
/// depends on the output device, which is precisely the property a comment
/// annotation must not have. A quarter point is the thinnest value that means
/// the same thing everywhere.
pub const MIN_WIDTH_PTS: f64 = 0.25;

/// **The most transparent mark offered**, as a fraction.
///
/// A tenth, not zero. An annotation at `/CA 0` is **invisible** — it is in the
/// file, it is selectable, it prints as nothing, and nothing on screen says it
/// is there. A control whose bottom end authors an invisible mark is a control
/// whose bottom end is a defect report waiting to be filed, and the operator
/// would have no way to tell it from a markup that failed to author at all.
///
/// A tenth is faint enough to be a wash over dense linework and still visible
/// as a mark.
pub const MIN_OPACITY: f64 = 0.1;

/// The thickest pen offered.
///
/// Twelve points is about a sixth of an inch — a marker rather than a pen, and
/// already heavy enough to obscure the drawing underneath, which is the failure
/// a comment on an engineering sheet has to avoid. Beyond it the annotation
/// stops being linework and starts being a fill, and `MarkupSpec` has a
/// separate concept for that which this shell deliberately does not offer.
pub const MAX_WIDTH_PTS: f64 = 12.0;

impl Default for Pen {
    /// The shipped pen: red linework at 2 pt, yellow highlighter.
    ///
    /// Every value here is the one the hard-coded constants held before this
    /// module existed, so a build that never touches the Style group authors
    /// byte-identical annotations to the one before it. That is the standing
    /// rule for a capability becoming choosable: *a build which omits nothing
    /// must behave as it did before the choice existed.*
    ///
    /// **Red** because that is what every PDF reader draws a comment shape in
    /// by default, and *"make it work the way other programs do"* is the
    /// operator's stated tie-breaker. **Yellow** for the highlighter for the
    /// same reason.
    ///
    /// **2 pt** because it is the width a comment shape reads at on a dense CAD
    /// export without dominating it — a hairline vanishes among the drawing's
    /// own 0.25 pt linework, which is the specific failure a markup on an
    /// engineering drawing has to avoid.
    fn default() -> Self {
        Self {
            // DOCUMENT COLOUR: the default markup pen, written into `/C`.
            ink: (0.85, 0.16, 0.16),
            // DOCUMENT COLOUR: highlighter yellow, likewise `/C` in the file.
            highlighter: (1.0, 1.0, 0.0),
            width_pts: 2.0,
            // Fully opaque, which writes no `/CA` at all — see the field's own
            // doc comment. This is what every markup this shell authored before
            // 2026-08-28 did, so the default is the old behaviour exactly.
            opacity: 1.0,
        }
    }
}

impl Pen {
    /// **The freehand simplification tolerance this pen implies**, in PDF
    /// points — a quarter of the stroke width.
    ///
    /// # ★ This function exists because the constant it replaced went stale
    /// the day the pen became a control
    ///
    /// `ink::SIMPLIFY_TOLERANCE_PTS` was `PEN_WIDTH_PTS / 4.0`, a `const`
    /// derived from the pen's **shipped** width of 2 pt. That was exactly
    /// right while the width was a constant, and `canvas::markup::ink`'s §3.2
    /// wrote down what to do when it stopped being one:
    ///
    /// > it is a *rule* — **if the pen ever becomes an operator control, the
    /// > tolerance follows it** rather than being re-tuned by eye.
    ///
    /// The pen became an operator control on 2026-08-17 and the tolerance did
    /// not follow. This is that rule, honoured — and the module wrote the rule
    /// down, was read by the session that broke it, and was broken **the same
    /// day**. Recording the rule in prose was not enough; only a test that
    /// varies the input could have caught it, and that test now exists.
    ///
    /// # Why the stale value was wrong in a way an operator would see
    ///
    /// The derivation is not arbitrary. Ramer–Douglas–Peucker guarantees that
    /// no removed point lay further than ε from the line replacing it, so ε
    /// bounds how far the **drawn centreline** can move. Setting ε to half of
    /// the stroke's *half*-width means the simplified centreline stays strictly
    /// inside the body of the stroke the raw trail would have drawn: **no pixel
    /// of the mark can move outside the mark.** That is the strongest statement
    /// available about a lossy simplification, and it is the whole reason the
    /// number is defensible rather than tuned.
    ///
    /// A fixed 0.5 pt breaks it in one direction and merely wastes work in the
    /// other:
    ///
    /// | pen width | half-width | fixed ε = 0.5 | verdict |
    /// |---:|---:|---:|---|
    /// | 0.25 pt | 0.125 pt | **4× the half-width** | ⛔ the centreline can move well outside the stroke. An operator drawing a fine detail line gets a visibly different curve from the one they drew |
    /// | 2 pt | 1 pt | 0.5 = half the half-width | ✅ the shipped case, and the only one that was ever right |
    /// | 12 pt | 6 pt | 0.17× the half-width | ⚠ correct but pointlessly tight — keeps far more points than the guarantee needs |
    ///
    /// The thin-pen row is the one that matters: it is the direction that
    /// **changes what is authored**, it is silent, and it is worst on exactly
    /// the drawings this shell is for, where a 0.25 pt pen exists to match a
    /// CAD sheet's own linework.
    ///
    /// # It lives here rather than in `ink`
    ///
    /// Because the rule is *"the tolerance follows the pen"*, and a derivation
    /// kept beside the value it derives from cannot be forgotten when that
    /// value changes — which is precisely what happened when it lived
    /// elsewhere as a `const`.
    #[must_use]
    pub fn simplify_tolerance_pts(self) -> f32 {
        (self.width_pts as f32) / 4.0
    }

    /// The colour this kind is authored in.
    ///
    /// The one place the two-pens rule is applied. `MarkupKind::rgb`'s
    /// hard-coded `match` moved here whole: every geometric kind takes
    /// [`Self::ink`] and Highlight takes [`Self::highlighter`], which is the
    /// same split the constant version made and is now the operator's to set.
    #[must_use]
    pub fn colour_for(self, kind: MarkupKind) -> (f64, f64, f64) {
        match kind {
            MarkupKind::Highlight => self.highlighter,
            _ => self.ink,
        }
    }

    /// Set the ink colour from a screen colour, discarding alpha.
    ///
    /// # ★ Alpha is dropped, deliberately, and this is not a limitation
    ///
    /// `egui`'s colour picker offers an alpha channel; a PDF annotation's `/C`
    /// entry is **three components and no more** (§12.5.2 Table 164: an array
    /// of 0, 1, 3 or 4 numbers in the annotation's colour space, with
    /// transparency carried by `/CA` instead). Feeding the picker's alpha into
    /// `/C` would be a value with nowhere to go.
    ///
    /// Opacity is therefore a **separate control**, and it is not built yet:
    /// `/CA` support was filed against `pdfcer-core` and is accepted-and-
    /// scheduled rather than shipped. Until it lands, offering an alpha slider
    /// here would be an affordance for something that cannot happen — the
    /// no-placeholders rule — so the picker is asked for an opaque colour and
    /// the operator is never shown a channel pdfcer would silently ignore.
    pub fn set_ink(&mut self, colour: Color32) {
        self.ink = rgb_of(colour);
    }

    /// **The `/CA` value to author with, or `None` for "write no key".**
    ///
    /// `None` at fully opaque. See [`Self::opacity`] for why that is not the
    /// same as `Some(1.0)` even though it is the same on screen, and why the
    /// difference is worth a method rather than a comparison at each call site
    /// — there are three, and the one that forgot would be the one that made a
    /// pdfcer annotation textually unlike everybody else's.
    #[must_use]
    pub fn opacity_option(&self) -> Option<f64> {
        (self.opacity < 1.0).then_some(self.opacity)
    }

    /// Set the highlighter colour from a screen colour. As [`Self::set_ink`].
    pub fn set_highlighter(&mut self, colour: Color32) {
        self.highlighter = rgb_of(colour);
    }

    /// The ink colour as a screen colour, for the swatch that sets it.
    #[must_use]
    pub fn ink_color32(self) -> Color32 {
        color32_of(self.ink)
    }

    /// The highlighter colour as a screen colour.
    #[must_use]
    pub fn highlighter_color32(self) -> Color32 {
        color32_of(self.highlighter)
    }
}

/// PDF components from a screen colour.
///
/// Alpha is dropped — see [`Pen::set_ink`]. The division is by `255.0` rather
/// than by `256.0`: the component range is *inclusive* of both ends, so `255`
/// must map to exactly `1.0` or a "pure red" chosen in the picker would be
/// written as `0.996` and round-trip to a slightly different swatch.
fn rgb_of(c: Color32) -> (f64, f64, f64) {
    (
        f64::from(c.r()) / 255.0,
        f64::from(c.g()) / 255.0,
        f64::from(c.b()) / 255.0,
    )
}

/// A screen colour from PDF components.
///
/// The inverse of [`rgb_of`], and **opaque**: the swatch shows the colour the
/// annotation will be, and an annotation has no alpha in `/C`. Rounding rather
/// than truncating, so the round trip through the picker is stable — truncation
/// would walk a colour down by one unit per visit.
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

    /// ★ The shipped pen authors exactly what the constants did.
    ///
    /// The "a build that omits nothing behaves as it did before" rule, pinned.
    /// These three values were `MarkupKind::rgb`'s two arms and
    /// `PEN_WIDTH_PTS`; a drift here means every markup this shell has ever
    /// authored is a different colour from the ones it authors now, with
    /// nothing to say so.
    #[test]
    fn the_default_pen_is_the_constants_it_replaced() {
        let pen = Pen::default();
        assert_eq!(pen.ink, (0.85, 0.16, 0.16));
        assert_eq!(pen.highlighter, (1.0, 1.0, 0.0));
        assert!((pen.width_pts - 2.0).abs() < f64::EPSILON);
    }

    /// Every geometric kind takes the ink; only Highlight takes the highlighter.
    ///
    /// The two-pens rule, over the whole `MarkupKind::ALL` list rather than
    /// over a hand-written subset — so a kind added later is covered without
    /// anybody remembering to add it here, and lands on the ink by default,
    /// which is the answer `MarkupKind::rgb`'s own note argued for.
    #[test]
    fn only_the_highlight_kind_uses_the_highlighter() {
        let pen = Pen {
            // DOCUMENT COLOUR: two arbitrary distinguishable values, so the
            // assertion below says which pen was taken rather than which
            // default happened to match.
            ink: (0.1, 0.2, 0.3),
            highlighter: (0.4, 0.5, 0.6),
            ..Pen::default()
        };
        for kind in MarkupKind::ALL {
            let expected = if matches!(kind, MarkupKind::Highlight) {
                (0.4, 0.5, 0.6)
            } else {
                (0.1, 0.2, 0.3)
            };
            assert_eq!(
                pen.colour_for(*kind),
                expected,
                "{kind:?} took the wrong pen"
            );
        }
    }

    /// ★ A colour survives the round trip through the picker unchanged.
    ///
    /// The property that makes the swatch usable rather than merely present.
    /// A conversion that truncated would walk a colour down by one unit every
    /// time the operator opened the picker and closed it without choosing —
    /// a slow, silent drift with no event to attach a bug report to.
    ///
    /// The endpoints are the ones that catch an off-by-one scale factor:
    /// dividing by 256 rather than 255 makes pure white round-trip to 254.
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
    ///
    /// Not reachable from the picker, which cannot produce one — reachable from
    /// a hand-edited file the day the pen is persisted, and from any future
    /// loader. `as u8` on a NaN is 0, a silent black; the clamp states the
    /// intent instead of inheriting that.
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
    ///
    /// # Why the range's own bounds are asserted in a `const` block
    ///
    /// `MIN_WIDTH_PTS > 0.0` is a relationship between two literals, so an
    /// ordinary `assert!` is a statement the compiler folds away and clippy
    /// rightly refuses. It is still worth stating — **zero is legal PDF** and
    /// means *"the thinnest line the device can draw"*, a width whose
    /// appearance depends on the output device and therefore exactly what a
    /// comment annotation must not be — so it is stated where a compile-time
    /// claim belongs, and a future edit that lowered the floor to zero would
    /// fail to build rather than fail to run.
    ///
    /// The runtime half is the one that can actually change: the shipped
    /// default is a value, and a default outside its own control's bounds
    /// would be silently rewritten the first time anybody touched the swatch.
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
