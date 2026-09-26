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
//!
//! Design and rationale: `docs/modules/egui-shell/ribbon/rhythm.md`.

use super::band::{BAND_PADDING_BOTTOM, CAPTION_GAP};
use super::ctx::Ctx;
use super::plan;

/// Vertical spacing between the band's control rows — `.grp .col { gap: 1px }`
/// in `mockups/pdfcer-shell.html`.
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
pub(crate) fn compressed_control_height(ui: &egui::Ui, ctx: &Ctx<'_>) -> f32 {
    #[allow(clippy::cast_precision_loss)] // single digits
    let n = plan::MAX_GROUP_ROWS.max(1) as f32;
    rows_height(ui, ctx) / n - BAND_ROW_SPACING
}

/// The point size the band draws its **secondary** text at: group captions,
/// and the label under a `Large` control.
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
pub(crate) fn rows_height(_ui: &egui::Ui, ctx: &Ctx<'_>) -> f32 {
    ctx.theme.metrics.ribbon_rows
}

/// **The band's height, on every tab, whatever it contains.**
pub(crate) fn band_height(ui: &egui::Ui, ctx: &Ctx<'_>) -> f32 {
    ctx.theme.metrics.ribbon_pad_top
        + rows_height(ui, ctx)
        + CAPTION_GAP
        + caption_height(ui, ctx)
        + BAND_PADDING_BOTTOM
}
