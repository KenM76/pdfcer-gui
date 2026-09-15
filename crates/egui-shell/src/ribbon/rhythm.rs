//! `ribbon::rhythm` — **the band's vertical rhythm**: the height of a control
//! row, the area the rows are laid into, the caption that hangs off the bottom
//! of it, and the band height that is the sum of all three.
//!
//! Every function here answers *"how tall is this?"* from the theme and two
//! constants, **and from nothing the manifest can vary**. That independence is
//! the R128 property the whole band is arranged around, and the module boundary
//! enforces it cheaply: a file that contains no `Group` cannot read one.
//!
//! ## The sum, in one place, because it is the thing that must add up
//!
//! ```text
//! band_height = ribbon_pad_top            6      .ribbon { padding: 6px 8px 0 }
//!             + ribbon_rows              68      3 × 22 + 2 × 1
//!             + CAPTION_GAP               3
//!             + one line at 11 pt      ≈ 12.7    .grp .cap { font-size: 11px }
//!             + BAND_PADDING_BOTTOM       5
//!             ────────────────────────────────
//!                                      ≈ 94.7    against the mockup's 96 px row
//! ```
//!
//! Every term is spent by a **named line** in [`super::band::group_body`], and
//! the group's own column has its `item_spacing.y` zeroed so that stays true.
//! `egui` inserts that spacing after the *last* row as well as between rows, so
//! leaving it in place adds a gap to the sum above that no line names, the rows
//! overshoot their budget by exactly one gap, and the caption is drawn into the
//! clearance the band reserves. **A gap the framework inserts on your behalf is
//! a term nothing names**, and at three rows there is no slack left for it to
//! disappear into.

use super::band::{BAND_PADDING_BOTTOM, CAPTION_GAP};
use super::ctx::Ctx;
use super::plan;

/// Vertical spacing between the band's control rows — `.grp .col { gap: 1px }`
/// in `mockups/pdfcer-shell.html`.
///
/// **The band's only row pitch** — the ordinary case and the re-wrapped case
/// share it, because [`band_row_height`] is unconditional. One, because the
/// mockup's arithmetic needs it: `3 × 22 + 2 × 1 = 68`, which is
/// [`crate::theme::Metrics::ribbon_rows`] exactly.
///
/// Tighter than the theme's `item_spacing.y`, and it has to be: the band's
/// height is fixed (R128) and three rows must fit an area sized for them.
pub(crate) const BAND_ROW_SPACING: f32 = 1.0;

/// **How tall a small control is drawn on the band** — `.rb { height: 22px }`.
///
/// # Why this is unconditional
///
/// It would be natural to draw an ordinary group's controls at the theme's
/// `control_height` (24 pt at `Quiet`) and reserve a shorter row only for a
/// group the collapse ladder has re-wrapped. That does not survive
/// [`plan::GROUP_ROWS`] being **three**: a three-row group is then the ordinary
/// case, and `3 × (24 + 4) = 84` does not fit the 68 pt area the theme
/// reserves. The band would grow by sixteen points on every tab, which is R128
/// arriving through the one door this module is arranged against.
///
/// So the row height is **one number for every group**, derived from the fixed
/// area and the fixed row count:
///
/// ```text
/// ribbon_rows / GROUP_ROWS − BAND_ROW_SPACING   =   68 / 3 − 1   =   21.67 pt
/// ```
///
/// against the mockup's `.rb { height: 22px }`. Within a third of a point, and
/// the third of a point is `egui` advancing the cursor past the last row by
/// `item_spacing` as it does past every other — so `3 × (21.67 + 1)` is 68.0
/// on the nose, which is the number that has to be exact.
///
/// # It applies to a one-row group too, and that is the mockup's rule
///
/// `.rb { height: 22px }` has no qualifier: a control on the band is 22 px
/// whether its group used one row or three. What varies is the **slack**
/// underneath — `.grp .items { align-items: flex-start }` lays the rows into
/// the top of the area and `.grp .cap { margin-top: auto }` hangs the caption
/// off the bottom of it. A one-row group whose control filled the area would
/// put its caption a row and a half below its neighbours', which is the
/// baseline invariant [`crate::ribbon::height_tests`] asserts.
///
/// So this deliberately does **not** read `rows.counts.len()`. A height that
/// varied with the row count would make a group's controls a different size
/// from the group beside it, which no ribbon in the product class does.
///
/// # The floor, and the self-disabling behaviour it preserves
///
/// Floored at [`crate::theme::Metrics::icon_pts`], because a control still has
/// to show its icon. [`rewrap_is_legible`] asks the same question of the same
/// numbers so the plan and the renderer cannot disagree: a theme whose
/// arithmetic did not clear the icon reports no gain from re-wrapping and the
/// ladder declines to spend a rung on it. **The feature turns itself off
/// rather than clipping.**
pub(crate) fn band_row_height(_ui: &egui::Ui, ctx: &Ctx<'_>) -> f32 {
    #[allow(clippy::cast_precision_loss)] // single digits
    let n = plan::GROUP_ROWS.max(1) as f32;
    (ctx.theme.metrics.ribbon_rows / n - BAND_ROW_SPACING).max(ctx.theme.metrics.icon_pts)
}

/// **How tall a control is drawn inside a re-wrapped group**, given the band's
/// fixed row area.
///
/// # Why this exists at all — Word's third row is shorter
///
/// The obvious implementation of S5 is "allow three rows", and it fails
/// immediately: three rows of `control_height` are half again as tall as two,
/// so the band grows, so the canvas beneath it moves on every tab click, which
/// is R128 and is the defect this whole module is arranged around.
///
/// Word does not grow its band. Measured in `evidence/word-ribbon/`, its band
/// is the same height at 1900 pt and at 1000 pt while the Font group goes from
/// two rows to three — because **its rows are not uniform**. The tall row is
/// the one with combo boxes in it; the icon rows are shorter. The band is a
/// fixed budget and rows are packed into it, rather than the band being two
/// rows tall by definition.
///
/// So a re-wrapped group here divides the **same** row area into
/// [`plan::MAX_GROUP_ROWS`], and its controls are drawn shorter to suit.
/// `Theme::apply` pins `spacing.interact_size.y` to `control_height`, which is
/// what makes every control exactly one row tall; overriding it on the group's
/// own `Ui` is what makes an extra row possible without touching the band.
///
/// While [`plan::MAX_GROUP_ROWS`] equals [`plan::GROUP_ROWS`] this returns the
/// same number as [`band_row_height`], so a re-wrapped group draws at the
/// ordinary row height. The two are kept separate because they answer different
/// questions — the ladder's ceiling and the band's natural depth — and raising
/// the ceiling must not change the ordinary case.
///
/// # The guard, and why the eligibility test is a measurement
///
/// A control still has to show its icon, so the result is compared against
/// [`crate::theme::Metrics::icon_pts`] by [`rewrap_is_legible`] rather than
/// assumed to clear it. A theme whose numbers did not clear would make this
/// return less than `icon_pts`, `measure_group_rows` would then report the
/// re-wrapped width as no better than the natural one, the ladder would decline
/// to spend a rung on it, and the group would simply never re-wrap. **The
/// feature turns itself off rather than clipping.**
pub(crate) fn compressed_control_height(ui: &egui::Ui, ctx: &Ctx<'_>) -> f32 {
    #[allow(clippy::cast_precision_loss)] // single digits
    let n = plan::MAX_GROUP_ROWS.max(1) as f32;
    rows_height(ui, ctx) / n - BAND_ROW_SPACING
}

/// The point size the band draws its **secondary** text at: group captions,
/// and the label under a `Large` control.
///
/// One accessor rather than two reads of the metric, because the number is
/// used in three places that must not drift — the caption's own
/// `RichText::size`, [`band_height`]'s prediction of how tall that caption
/// will be, and [`super::sizing`]'s Large label. A band whose height
/// prediction and whose caption disagree is R128 by a fraction of a line,
/// which is precisely the class of drift `group_body`'s closing
/// `allocate_space` exists to absorb and would rather not have to.
pub(crate) fn caption_font(ctx: &Ctx<'_>) -> egui::FontId {
    egui::FontId::proportional(ctx.theme.metrics.ribbon_caption_pts)
}

/// How tall one line of [`caption_font`] is, in the fonts this context has.
pub(crate) fn caption_height(ui: &egui::Ui, ctx: &Ctx<'_>) -> f32 {
    let font = caption_font(ctx);
    ui.ctx().fonts_mut(|fonts| fonts.row_height(&font))
}

/// Whether a group may be drawn re-wrapped at all, under this theme.
///
/// See [`compressed_control_height`]. Separated so the plan and the renderer
/// ask the same question of the same numbers.
pub(crate) fn rewrap_is_legible(ui: &egui::Ui, ctx: &Ctx<'_>) -> bool {
    compressed_control_height(ui, ctx) >= ctx.theme.metrics.icon_pts
}

/// **The band's control-row area** — how far a group's cursor is padded out to
/// before its caption is drawn, and therefore the one baseline every caption in
/// the band shares.
///
/// # It is a budget stated by the theme, not a multiple of a row
///
/// The alternative spelling is `GROUP_ROWS × (control_height + item_spacing)`,
/// meaning *"exactly as tall as the rows"*, and it has a property that reads as
/// a defect once named: a group that uses every row fills the area edge to
/// edge, so its caption is drawn immediately beneath its last control, while a
/// one-row group's caption sits a whole row lower. The captions still share a
/// baseline, but the band has no headroom anywhere and reads as cramped.
///
/// The mockup's band is a **budget** instead: `.grp .items { align-items:
/// flex-start }` lays the rows into the top of a 68 px area and `.grp .cap
/// { margin-top: auto }` hangs the caption off the bottom of it, whatever the
/// rows did. So the area is stated by the theme
/// ([`crate::theme::Metrics::ribbon_rows`]) and the rows are laid into it.
///
/// # The trailing `item_spacing`, which is the term that bites
///
/// `rows × height + (rows − 1) × spacing` is the right answer for the *ink* and
/// the wrong one for the **cursor**: `egui` advances the cursor past every
/// laid-out rect by `item_spacing`, after the last row as much as after the
/// first. A group that used every row therefore leaves its cursor one gap
/// beyond that figure, its padding computes as zero, and it ends up exactly
/// `item_spacing` taller than a one-row neighbour whose padding *was* applied.
///
/// **A fixture can hide this entirely, so the fixture has to be checked too.**
/// `super::width_tests`' context installs a font but applies no
/// [`crate::theme::Theme`], so `egui`'s default `interact_size.y` (18 pt) sits
/// well under the theme's `control_height` (24 pt) and every row carries 6 pt
/// of slack for a stray gap to hide in. In the running application the two are
/// equal by construction — `Theme::apply` sets `spacing.interact_size.y =
/// control_height` — so there is no slack and the discrepancy is visible in the
/// band's own trace. `super::height_tests::context` applies the theme for
/// exactly this reason: a fixture that is more forgiving than the program
/// flatters the thing it measures.
///
/// # What the collapse ladder does with this area
///
/// `RIBBON_SCALING.md`'s three rungs are re-wrap → collapse → scroll, and rung
/// one divides *this* area into [`plan::MAX_GROUP_ROWS`] rows rather than
/// [`plan::GROUP_ROWS`]. Both row counts are constants, so the ladder changes
/// the divisor and never the area — the band cannot grow a rung. See
/// [`rewrap_is_legible`] for the self-disabling behaviour that protects.
pub(crate) fn rows_height(_ui: &egui::Ui, ctx: &Ctx<'_>) -> f32 {
    ctx.theme.metrics.ribbon_rows
}

/// **The band's height, on every tab, whatever it contains.**
///
/// Five terms, in the order they are drawn:
/// [`crate::theme::Metrics::ribbon_pad_top`], the control rows,
/// [`CAPTION_GAP`], one line of [`caption_font`], and
/// [`BAND_PADDING_BOTTOM`]. Derived from the theme, the font and two
/// constants, and from nothing the manifest can vary — see the module header
/// on R128 for why that independence is the whole point.
///
/// The top padding (`.ribbon { padding: 6px 8px 0 }`) is a term *here* rather
/// than an `add_space` before the first group, for the reason
/// [`BAND_PADDING_BOTTOM`] gives at length: space emitted only when there is a
/// group to emit it before would be absent on a tab whose groups all went into
/// the overflow menu, and a band six points shorter on one tab than on its
/// neighbour moves the canvas on a tab click. `group_body` spends it, out of
/// the group box's top padding, on every group and on none.
///
/// The bottom padding belongs in this derivation for the same reason: space
/// emitted after the last group would be absent on a tab that drew no group,
/// which is a reachable state, and would make the height content-derived
/// through the back door.
///
/// With the shipped `Quiet` theme the sum is `6 + 68 + 3 + 12.7 + 5 ≈ 94.7` pt,
/// against the mockup's own `grid-template-rows` figure of 96 px for the ribbon
/// row (which includes its 1 px bottom border).
///
/// `pub(crate)` so a test can state the claim in the same terms the
/// renderer does rather than by re-deriving it.
pub(crate) fn band_height(ui: &egui::Ui, ctx: &Ctx<'_>) -> f32 {
    ctx.theme.metrics.ribbon_pad_top
        + rows_height(ui, ctx)
        + CAPTION_GAP
        + caption_height(ui, ctx)
        + BAND_PADDING_BOTTOM
}
