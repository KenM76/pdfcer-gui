//! # `ribbon::overflow` — the band scrolls; it does not hide behind a menu
//!
//! **`RIBBON_SCALING.md` §3.3.** The last rung of the width ladder in that
//! document's §1, reached only after every group has re-wrapped and every group
//! that may collapse has collapsed (§3.2).
//!
//! ## An arrow, not a menu
//!
//! The band scrolls, on the operator's instruction — *"do the scroll like
//! Word"*, asked twice.
//!
//! Word's answer, photographed at 460 pt by `tools/word-ribbon-study.ps1`: a
//! `›` at the band's right edge that **shifts the band sideways**. Every group
//! stays a group, in its place, in manifest order; the window is a viewport
//! onto a row that is wider than it.
//!
//! The alternative is a `⏷ N more` dropdown, and the trade is worth stating
//! because the dropdown is better at one thing. A menu **names what is
//! hidden** — it says *"3 more"* and lists them. An arrow does not; it makes
//! the operator push and look. What the arrow gives back is that a group never
//! changes its nature: it is never simultaneously "on the ribbon" and "in a
//! menu", which is two mental models for one surface. The convention across
//! the product class is the arrow, and *"use the conventional interaction,
//! never invent one"* settles ties like this one.
//!
//! ## Why scrolling is LAST, and why that ordering is not taste
//!
//! Because scrolling is the only rung that puts a command **off screen**.
//! Re-wrapping keeps every control visible and labelled. Collapsing keeps every
//! control one click away with its group's caption still readable on the band.
//! Scrolling removes the group from view entirely until the operator acts.
//!
//! Word agrees, and the agreement is measurable rather than assumed: its arrow
//! appears at 460 pt and **not** at 800, by which width four groups have
//! already collapsed. A surface that scrolled first would be pushing commands
//! off screen while the space to show them, compacted, was still there.
//!
//! ## The position is remembered per tab, and clamped every frame
//!
//! `first` — the index of the leftmost group drawn — lives in `egui::Memory`,
//! keyed on the tab id. Two consequences, both wanted:
//!
//! * switching tabs and coming back returns to where the operator left the
//!   band, which is what every scroll position in every application does;
//! * it is session-scoped and never written to disk, so a window resized on
//!   another monitor cannot strand a tab scrolled past its own contents.
//!
//! **It is clamped against a freshly computed layout on every frame, not
//! trusted.** This is the one place S4 could reintroduce the feedback loop
//! R128 forbids. The stored index is an *input* to
//! layout, so a stale one — left behind by a widened window — would show blank
//! space at the right of the band, and a naive fix that recomputed `first` from
//! what was drawn would be a measurement feeding the size that produced it.
//! [`clamp`] is a pure function of the width and the group widths; it never
//! reads what was drawn.

use egui::{Align, Layout, Rect, RichText, UiBuilder, Vec2};

use super::ctx::Ctx;

/// The glyphs. Left and right, and nothing else — this affordance has exactly
/// two states and a disabled one is not drawn (R9).
const LEFT: &str = "‹";
const RIGHT: &str = "›";

/// Which way an arrow points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Direction {
    /// Back towards the first group.
    Left,
    /// On towards the last.
    Right,
}

impl Direction {
    const fn glyph(self) -> &'static str {
        match self {
            Self::Left => LEFT,
            Self::Right => RIGHT,
        }
    }

    /// The `ui_rect` name this arrow publishes.
    ///
    /// The RIGHT arrow's name is `ribbon.overflow`, which names the question
    /// rather than the glyph. Region names are a **cross-repo stability
    /// contract** with `tools/ui-verify`, and what those checks ask of this
    /// control is: is the affordance on screen at every width, is it
    /// hit-testable under real metrics, does any visible group overlap it.
    /// None of that is a claim about the mechanism, so naming the region after
    /// the mechanism would put an implementation detail into a cross-repo
    /// contract.
    ///
    /// The left arrow answers a question nothing asked before, so it carries a
    /// name of its own.
    const fn region(self) -> &'static str {
        match self {
            Self::Left => "ribbon.scroll.left",
            Self::Right => super::report::OVERFLOW,
        }
    }
}

/// **How wide a scroll arrow is**, and therefore what the band must reserve.
///
/// Deliberately a function of the theme rather than a constant: the arrow sits
/// in a row of controls and an arrow that did not scale with them would be a
/// misalignment at every scale but one.
pub(crate) fn arrow_width(ctx: &Ctx<'_>) -> f32 {
    ctx.theme.metrics.control_height
}

/// The memory key holding a tab's scroll position.
fn key(ctx: &Ctx<'_>, tab_id: &str) -> egui::Id {
    ctx.id("band-scroll", tab_id)
}

/// Where this tab's band is scrolled to, as a leading group index.
pub(crate) fn first(ui: &egui::Ui, ctx: &Ctx<'_>, tab_id: &str) -> usize {
    ui.ctx()
        .data(|d| d.get_temp::<usize>(key(ctx, tab_id)))
        .unwrap_or(0)
}

/// Move this tab's band.
pub(crate) fn set_first(ui: &egui::Ui, ctx: &Ctx<'_>, tab_id: &str, at: usize) {
    ui.ctx().data_mut(|d| d.insert_temp(key(ctx, tab_id), at));
}

/// **The furthest left index that still fills the band**, given the widths.
///
/// # ★★★ Why this is a pure function and why it runs every frame
///
/// A remembered scroll position is an input to layout. Widen the window and a
/// position that was correct becomes one that leaves blank space at the right
/// of the band — the group list ends before the viewport does. The operator did
/// nothing wrong and there is nothing for them to press.
///
/// The tempting fix is to notice the blank space after drawing and pull the
/// band back. That is a measurement feeding the size that produced it — the
/// feedback loop R128 forbids. So instead: compute, from
/// the offered width and the group widths alone, the largest `first` at which
/// the remaining groups still reach the right edge — and clamp to it before
/// anything is drawn.
///
/// Walks from the end backwards, accumulating until the budget is exceeded.
/// The last index that fitted is the answer.
pub(crate) fn clamp(widths: &[f32], available: f32, separator: f32) -> usize {
    let n = widths.len();
    if n == 0 {
        return 0;
    }
    // A degenerate width is NOT special-cased to 0. Returning 0 at width 0 and
    // `n-1` at width 1 would mean growing the band by one point scrolls it to
    // the far end, which is the monotonicity violation
    // `widening_never_pushes_the_band_further_right` exists to forbid. Letting
    // the loop below answer keeps the function monotonic across its whole
    // domain, including the part of it nobody can see.
    let available = if available.is_finite() {
        available.max(0.0)
    } else {
        0.0
    };
    let separator = if separator.is_finite() {
        separator.max(0.0)
    } else {
        0.0
    };

    let mut used = 0.0_f32;
    let mut first = n;
    for (i, w) in widths.iter().enumerate().rev() {
        let step = if used == 0.0 { *w } else { separator + *w };
        if used + step > available {
            break;
        }
        used += step;
        first = i;
    }
    // `n` would mean "scrolled past everything", which is never a legal
    // position: a band showing nothing is worse than a band overflowing.
    first.min(n.saturating_sub(1))
}

/// Draw one arrow, and report whether it was pressed.
///
/// `rect` is computed by the caller from the band's own edge **before any group
/// is laid out**, for the reason `plan`'s header gives: the affordance must
/// not be the thing that gets squeezed out when the band is short of room,
/// because it is the only way back.
///
/// Returns the `Response`, not a bare `clicked()`. The band publishes its `Id`
/// as `BandOutcome::overflow_id`, which is how a driven check finds the control
/// to click without knowing where it is; two tests fail loudly if it stops
/// being honoured.
pub(crate) fn arrow(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    dir: Direction,
    rect: Rect,
    announce: String,
) -> egui::Response {
    let response = ui
        .scope_builder(
            UiBuilder::new()
                .id_salt(ctx.id(dir.region(), ""))
                .max_rect(rect)
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                ui.set_max_width(rect.width());
                ui.add(
                    egui::Button::new(RichText::new(dir.glyph()))
                        .min_size(Vec2::new(rect.width(), rect.height()))
                        .truncate(),
                )
            },
        )
        .inner;

    // A glyph is not an accessible name. The count is the information.
    let name = announce.clone();
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, name.clone()));
    ctx.reporter.report_static(response.rect, dir.region());
    response.on_hover_text(announce)
}

/// The `ui_rect` names this module publishes, for `ui-verify`'s stability
/// contract.
#[cfg(test)]
pub(crate) const REGIONS: [&str; 2] = ["ribbon.scroll.left", super::report::OVERFLOW];

#[cfg(test)]
mod tests {
    use super::*;

    /// **A band with room is never scrolled.**
    #[test]
    fn everything_fitting_means_the_band_starts_at_the_first_group() {
        assert_eq!(clamp(&[100.0, 100.0, 100.0], 400.0, 8.0), 0);
    }

    /// **Widening pulls the band back**, which is the whole reason this is
    /// clamped rather than remembered.
    #[test]
    fn a_wider_band_admits_an_earlier_first_group() {
        let w = [100.0, 100.0, 100.0, 100.0];
        // Room for two: 100 + 8 + 100 = 208.
        assert_eq!(clamp(&w, 210.0, 8.0), 2);
        // Room for three: 100 + 8 + 100 + 8 + 100 = 316.
        assert_eq!(clamp(&w, 320.0, 8.0), 1);
        // Room for all four.
        assert_eq!(clamp(&w, 500.0, 8.0), 0);
    }

    /// **The clamp is monotonic in the width**, swept rather than
    /// spot-checked — the same discipline the compaction ladder's own test
    /// applies, and for the same reason: two widths compared and no others is
    /// how a monotonicity claim passes while being false in between.
    #[test]
    fn widening_never_pushes_the_band_further_right() {
        let w = [90.0, 140.0, 60.0, 200.0, 75.0];
        let mut prev = usize::MAX;
        for width in 0..800 {
            let first = clamp(&w, width as f32, 8.0);
            assert!(
                first <= prev || prev == usize::MAX,
                "at {width} pt the band starts at group {first}, further right \
                 than the {prev} it started at when the band was NARROWER — a \
                 wider window must never scroll the ribbon on"
            );
            prev = first;
        }
    }

    /// **Never scrolled past the end.** A band showing nothing at all is worse
    /// than a band that overflows, and it is unrecoverable without the arrow
    /// that is no longer on screen.
    #[test]
    fn the_band_never_scrolls_past_its_last_group() {
        // One group far wider than the band: it still gets drawn, clipped.
        assert_eq!(clamp(&[500.0], 10.0, 8.0), 0);
        assert_eq!(clamp(&[100.0, 100.0, 500.0], 10.0, 8.0), 2);
    }

    /// Degenerate inputs answer the same as "no room at all" rather than
    /// panicking in a paint loop — and *not* zero, which would break
    /// monotonicity at the bottom of the range. See [`clamp`].
    #[test]
    fn nonsense_inputs_behave_like_no_room() {
        assert_eq!(clamp(&[], 100.0, 8.0), 0);
        assert_eq!(clamp(&[100.0], f32::NAN, 8.0), 0);
        assert_eq!(clamp(&[100.0], -5.0, 8.0), 0);
        // Three groups and no room: the LAST one, not the first.
        assert_eq!(clamp(&[100.0, 100.0, 100.0], f32::NAN, 8.0), 2);
    }

    /// The published region names are a cross-repo stability contract with
    /// `ui-verify`; changing one is changing an API.
    #[test]
    fn the_region_names_are_stable() {
        assert_eq!(Direction::Left.region(), REGIONS[0]);
        assert_eq!(Direction::Right.region(), REGIONS[1]);
        assert_eq!(
            Direction::Right.region(),
            "ribbon.overflow",
            "the right arrow inherits the dropdown's published name on purpose — four ui-verify checks name it, and what they assert is still true of the arrow"
        );
    }
}
