//! `ribbon::rhythm` — **the band's vertical rhythm**: the height of a control
//! row, the area the rows are laid into, the caption that hangs off the bottom
//! of it, and the band height that is the sum of all three.
//!
//! Split out of [`super::band`] on 2026-09-05, when that file crossed R2's
//! 1,500-line limit. The seam is a real one rather than a convenient cut: every
//! function here answers *"how tall is this?"* from the theme and two
//! constants, **and from nothing the manifest can vary** — which is the R128
//! property the whole band is arranged around, and it is easier to check that
//! nothing in a file reads a `Group` when the file contains no `Group`.
//!
//! ## ★★★ The sum, in one place, because it is the thing that must add up
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
//! that is deliberate: on 2026-09-05 the group's own column still carried the
//! theme's `item_spacing.y`, which `egui` inserts after the *last* row as well
//! as between them, and the rows overshot their budget by exactly one gap. With
//! two 24 pt rows there had been twelve points of slack for it to disappear
//! into; with three there is none, and the caption was drawn 3 pt into the
//! clearance the band reserves. **A gap the framework inserts on your behalf is
//! a term in that sum that nothing names**, so the column's spacing is zeroed
//! and every gap that remains is a line somebody can point at.

use super::band::{BAND_PADDING_BOTTOM, CAPTION_GAP};
use super::ctx::Ctx;
use super::plan;

/// Vertical spacing between the band's control rows — `.grp .col { gap: 1px }`
/// in `mockups/pdfcer-shell.html`.
///
/// ★★ **It was 2.0 and named `COMPRESSED_ROW_SPACING` until 2026-09-05**, when
/// it stopped being the re-wrap case's private number and became the band's
/// only row pitch. See [`band_row_height`] on why the two cases merged; the
/// value came down to 1 because the mockup's own arithmetic needs it:
/// `3 × 22 + 2 × 1 = 68`, which is [`crate::theme::Metrics::ribbon_rows`]
/// exactly.
///
/// Tighter than the theme's `item_spacing.y`, and it has to be: the band's
/// height is fixed (R128) and three rows must fit an area sized for them.
pub(crate) const BAND_ROW_SPACING: f32 = 1.0;

/// **How tall a small control is drawn on the band** — `.rb { height: 22px }`.
///
/// # ★★★ Why this is unconditional, and what it replaced — 2026-09-05
///
/// Until this date the band had **two** row heights. An ordinary group drew
/// its controls at the theme's `control_height` (24 pt at `Quiet`); a group
/// that the collapse ladder had **re-wrapped** onto a third row drew them at
/// `ribbon_rows / MAX_GROUP_ROWS − 2`, under a branch reading
/// `if rows.counts.len() > plan::GROUP_ROWS`.
///
/// That was correct while the natural row count was two. It stopped being
/// correct the moment [`plan::GROUP_ROWS`] became **three** — see that
/// constant's own note — because a three-row group is then the ordinary case
/// and `3 × (24 + 4) = 84` does not fit the 68 pt area the theme reserves.
/// The band would have grown by sixteen points on every tab, which is R128
/// arriving through the one door the whole module is arranged against.
///
/// ⇒ So the row height is now **one number for every group**, derived from
/// the fixed area and the fixed row count:
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
/// # ★★ It applies to a ONE-row group too, and that is the mockup's rule
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
/// # ★★★ Why this exists at all — Word's third row is SHORTER
///
/// The obvious implementation of S5 is "allow three rows", and it fails
/// immediately: three rows of `control_height` are half again as tall as two,
/// so the band grows, so the canvas beneath it moves on every tab click, which
/// is R128 and is the defect this whole module is arranged around. The height
/// test caught it on the first build.
///
/// Word does not grow its band. Measured in `evidence/word-ribbon/`, its band
/// is the same height at 1900 pt and at 1000 pt while the Font group goes from
/// two rows to three — because **its rows are not uniform**. The tall row is
/// the one with combo boxes in it; the icon rows are shorter. The band is a
/// fixed budget and rows are packed into it, rather than the band being two
/// rows tall by definition.
///
/// So a re-wrapped group here divides the SAME row area into three, and its
/// controls are drawn shorter to suit. `Theme::apply` pins
/// `spacing.interact_size.y` to `control_height`, which is what makes every
/// control exactly one row tall; overriding it on the group's own `Ui` is what
/// makes three rows possible without touching the band.
///
/// # The guard, and why the eligibility test is a measurement
///
/// A control still has to show its icon. With the shipped theme the arithmetic
/// is `(24 + 4) × 2 / 3 − 2 = 16.7 pt` against a 16 pt icon — it clears, and
/// only just. Under the Compact preset it is `(28 + 4) × 2 / 3 − 2 = 19.3` pt
/// against 17 pt. Both work, and neither was assumed: a theme whose numbers did
/// not clear would make this return less than `icon_pts`, and
/// `measure_group_rows` then reports the re-wrapped width as no better than the
/// natural one, so the ladder declines to spend a rung on it and the group
/// simply never re-wraps. **The feature turns itself off rather than clipping.**
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

/// # ★ The arithmetic this used to be, and the defect it shipped
/// have been laid out — which is the number a shorter group is padded out
/// to, and what pins every caption in the band to one baseline.
///
/// # ★ `GROUP_ROWS × (control_height + item_spacing)`, and the trailing
/// ★ term is the one that matters
///
/// The obvious spelling is `rows × height + (rows − 1) × spacing`: two rows
/// with one gap between them. That is the right answer for the *ink* and
/// the wrong one for the **cursor**, because `egui` advances the cursor past
/// every laid-out rect by `item_spacing` — after the last row as much as
/// after the first. So a two-row group's cursor sits one gap beyond that
/// figure, its padding computes as zero, and the group ends up exactly
/// `item_spacing` taller than its one-row neighbour, whose padding *was*
/// applied and did land on the figure.
///
/// **That defect shipped into a build and no test in this crate could see
/// it.** `super::width_tests`' context installs a font but does not apply a
/// [`crate::theme::Theme`], so `egui`'s default `interact_size.y` (18 pt) is
/// well under the theme's `control_height` (24 pt) and every row had 6 pt of
/// slack for the stray gap to hide in. In the running application the two
/// are equal by construction — `Theme::apply` sets
/// `spacing.interact_size.y = control_height` — there is no slack, and the
/// band's own trace showed Shapes at 68 pt beside Text markup at 64.
/// `super::height_tests::context` now applies the theme for exactly this
/// reason: `HANDOFF.md` §10's *"a fixture can flatter the thing it
/// measures"*, arriving through spacing rather than through a curve.
///
/// **The band's control-row area** — how far a group's cursor is padded out
/// to before its caption is drawn, and therefore the one baseline every
/// caption in the band shares.
///
/// ★★★ **This stopped being `GROUP_ROWS × (control_height + item_spacing)`
/// on 2026-09-04**, and the change is the whole of the operator's fourth
/// complaint — *"the mock's band is visibly taller with more generous rows
/// and the group caption sitting lower"*.
///
/// The old expression means *"exactly as tall as two rows"*. It has one
/// property that reads as a bug once it is named: a two-row group fills it
/// edge to edge, so its caption is drawn immediately beneath the last
/// control, while a one-row group's caption sits a whole row lower. The
/// captions share a baseline — the invariant was never violated — but the
/// band has no headroom anywhere, and the density that produces is exactly
/// what the mock does not look like.
///
/// The mockup's band is a **budget**: `.grp .items { align-items:
/// flex-start }` lays the rows into the top of a 68 px area and
/// `.grp .cap { margin-top: auto }` hangs the caption off the bottom of it,
/// whatever the rows did. So the area is now stated by the theme
/// ([`crate::theme::Metrics::ribbon_rows`]) and the rows are laid into it.
///
/// # What did NOT change, and why that matters more than the number
///
/// The **collapse ladder**. `RIBBON_SCALING.md`'s three rungs are re-wrap →
/// collapse → scroll, and rung one divides *this* area into
/// [`plan::MAX_GROUP_ROWS`] rows instead of [`plan::GROUP_ROWS`]. Both row
/// counts are untouched. What moved is the divisor's numerator, and it moved
/// **upward**, so [`compressed_control_height`] goes from
/// `56/3 − 2 = 16.67` pt against a 16 pt icon — a margin of two thirds of a
/// point, which is the margin that decides whether the rung is available at
/// all — to `68/3 − 2 = 20.67`. The rung that was one theme tweak away from
/// switching itself off now clears by 4.67 pt. See
/// [`rewrap_is_legible`] for the self-disabling behaviour this protects.
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
/// ★ **The top padding joined the sum on 2026-09-04** (`.ribbon
/// { padding: 6px 8px 0 }`), and it joined *here* rather than as an
/// `add_space` before the first group for exactly the reason
/// [`BAND_PADDING_BOTTOM`] gives at length: space emitted only when there is
/// a group to emit it before would be absent on a tab whose groups all went
/// into the overflow menu, and a band six points shorter on one tab than on
/// its neighbour moves the canvas on a tab click. [`group_body`] spends it,
/// out of [`GroupBox::pad_top`], on every group and on none.
///
/// With the shipped `Quiet` theme the sum is
/// `6 + 68 + 3 + 12.7 + 5 ≈ 94.7` pt, against the mockup's own
/// `grid-template-rows` figure of 96 px for the ribbon row (which includes
/// its 1 px bottom border). It was ≈ 74 pt before this pass.
///
/// The bottom padding belongs **in this derivation** rather than in the group
/// loop for the reason [`BAND_PADDING_BOTTOM`] gives at length: space emitted
/// after the last group would be absent on a tab that drew no group, which is
/// a reachable state (every group in the overflow menu) and would make the
/// height content-derived through the back door.
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
