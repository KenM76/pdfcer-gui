//! # `dock::banner` — a permanent one-line strip above a side's columns
//!
//! ## What this is, in one sentence
//!
//! A caller-drawn horizontal strip that a dock side reserves off the top of
//! its own rectangle, **before** the columns are laid out, so whatever the
//! application draws there is on screen for as long as the side is — no tab
//! to click, no stack to keep open, no panel that can be closed and lost.
//!
//! ## ★★★ Why it is dock CHROME and cannot be a panel
//!
//! `SHELL_LAYOUT_PROPOSAL.md` §3.2 worked this out and the arithmetic is the
//! whole argument. The strip an application wants here is one row —
//! twenty-something points. A dock stack cannot be that:
//!
//! * [`super::plan::MIN_STACK_HEIGHT`] is **80 pt**, and it is a *layout*
//!   floor rather than only a drag floor: it is handed to
//!   [`super::plan::resolve_spans`] when a column's stacks are measured, so a
//!   stack asking for less is silently given more.
//! * [`super::plan::TAB_BAR_HEIGHT`] is **24 pt** on its own, so a 28 pt
//!   stack would be a tab bar with four points under it — which reads as a
//!   rendering fault, which is exactly the reasoning `MIN_STACK_HEIGHT`'s own
//!   doc comment gives for existing.
//!
//! So the strip is drawn **over** the side's rectangle rather than inside its
//! column layout, in the same place and for the same reason as
//! [`super::collapse`]'s chevron: *"Over rather than inside, because inserting
//! it into the column layout would take height from a panel body on every
//! frame — and it is dock chrome, not a panel's content."*
//!
//! ⚠ The one difference from the chevron, and it is deliberate: the chevron is
//! painted over the columns and takes no space, while a banner **does** take
//! its height off the top before the columns are resolved. A strip that
//! overlapped the first stack's tab bar would be unclickable chrome sitting on
//! top of clickable chrome, which is failure mode #1 (*never put a control
//! under something that looks the same*) in miniature.
//!
//! ## ★★ R7 — this module must not learn what the application puts in it
//!
//! `tools/gates/check-shell-purity.sh` forbids `egui-shell` naming anything
//! from `pdfcer-*`. Everything here is a rectangle, a height and a closure.
//! The banner does not know it is showing a tool status any more than
//! [`super::tabs`] knows a panel shows a PDF: the caller draws, this module
//! reserves.
//!
//! ## The height is negotiated, not obeyed
//!
//! [`resolve_height`] clamps a request into a band. A banner is permanent
//! chrome and the panels underneath it are the point of the dock, so:
//!
//! | rule | why |
//! |---|---|
//! | at least [`MIN_HEIGHT`] | below one text row the strip is a coloured line an operator cannot read, which is a placeholder with a rectangle (R9) |
//! | at most [`MAX_FRACTION`] of the side | a banner that could take half the dock would be a panel that cannot be closed, which is the one thing chrome must never become |
//! | zero when the side is too short to give it | ★ **absent rather than squeezed**: a 4 pt strip publishes a rectangle and shows nothing, and a rectangle with nothing in it is precisely the shape that let three panels ship unreachable with every gate green |
//!
//! The clamp is a pure function so the third row above is testable without a
//! frame, which is the only way anybody would ever notice it.

use egui::{Align, Layout, Rect, UiBuilder};

use super::model::DockSide;
use super::{Ctx, report};

/// What an application draws into a side's banner.
///
/// Called at most once per side per frame, with a [`egui::Ui`] whose
/// `max_rect` **and clip rectangle** are the strip. The clip is the load
/// bearing half: a caller that draws two rows into a one-row strip gets the
/// second row clipped away rather than pushing the columns down, which is the
/// content-driven-height feedback loop (pdfcer's R128) this crate is arranged
/// to make unwritable.
pub type BannerHandler<'a> = dyn FnMut(&mut egui::Ui) + 'a;

/// The least height a banner may be drawn at, in points.
///
/// One row of ordinary text plus its padding. Below this the strip cannot
/// carry a sentence, and a strip that cannot carry a sentence is a coloured
/// band — see the module header on why that is worse than nothing.
pub const MIN_HEIGHT: f32 = 18.0;

/// The most of a side's height a banner may take, as a fraction.
///
/// Chrome that could take a quarter of the dock is already too much; this is
/// a backstop against a caller passing a nonsense height, not a design
/// target. A realistic banner is one row.
pub const MAX_FRACTION: f32 = 0.25;

/// How tall the banner actually gets, given what the caller asked for and how
/// much side there is.
///
/// Returns `0.0` when the request cannot be honoured at all, and a caller
/// seeing zero draws **nothing** — no rectangle is published and the columns
/// get the whole side back.
///
/// # ★ Why zero rather than [`MIN_HEIGHT`] when the side is short
///
/// Because the alternative is a strip that exists in the trace and not on the
/// screen. `report::RectSink`'s header records what that costs on this
/// project: Bookmarks, Layers and Signatures shipped **unreachable** with a
/// rail entry and a perfectly healthy rectangle each. A banner squeezed into a
/// side too short for both it and a panel would reproduce that exactly — the
/// region publishes, the check goes green, and the operator sees a sliver.
#[must_use]
pub fn resolve_height(requested: f32, side_height: f32) -> f32 {
    if !requested.is_finite() || !side_height.is_finite() {
        return 0.0;
    }
    if requested < MIN_HEIGHT {
        return 0.0;
    }
    let ceiling = side_height * MAX_FRACTION;
    if ceiling < MIN_HEIGHT {
        // The side is so short that even the smallest legible banner would be
        // more than a quarter of it. Chrome loses; the panels keep the room.
        return 0.0;
    }
    requested.min(ceiling)
}

impl<'a> super::Dock<'a> {
    /// Draw a permanent one-line strip above `side`'s columns.
    ///
    /// `height_pts` is a request, clamped by [`resolve_height`]; ask for one
    /// text row's worth and expect to get it at any sane window size.
    ///
    /// The handler is called only when the side is **drawn** — not when it is
    /// collapsed to a rail, and not when it holds no panels, because in both
    /// of those cases there is no side for a banner to sit above.
    ///
    /// ```no_run
    /// # use egui_shell::dock::{Dock, DockSide, DockState};
    /// # fn frame(ui: &mut egui::Ui, state: &mut DockState) {
    /// let mut banner = |ui: &mut egui::Ui| {
    ///     ui.label("Select — click to pick, drag to marquee");
    /// };
    /// Dock::new()
    ///     .with_side_banner(DockSide::Right, 26.0, &mut banner)
    ///     .show(ui, state, |_panel, _ui| {});
    /// # }
    /// ```
    ///
    /// # Borrowing
    ///
    /// The same rule [`Dock::with_tab_menu`](super::Dock::with_tab_menu)
    /// documents: this handler and the `body` closure both live across
    /// [`super::Dock::show`], so they cannot both capture `&mut` to the same
    /// thing. Record into a local and act after `show` returns.
    #[must_use]
    pub fn with_side_banner(
        mut self,
        side: DockSide,
        height_pts: f32,
        handler: &'a mut (impl FnMut(&mut egui::Ui) + 'a),
    ) -> Self {
        self.banner = Some((side, height_pts, handler));
        self
    }
}

/// Reserve and draw the banner for `side`, returning the rectangle the
/// columns get.
///
/// Returns `area` unchanged when there is no banner for this side or the
/// height resolved to zero — so the no-banner path costs one comparison and
/// changes no geometry, which is what keeps every existing layout test valid.
///
/// # ★★ The region is published against the SIDE's `Ui`, not the child's
///
/// [`report::Reporter::report`]'s own doc states the rule and the reason:
/// reporting a region against a clip derived from itself is *"the tautology
/// `visible == 1.0` dressed up as a measurement"*. The question asked of
/// `dock.<side>.banner` is *can the operator see this strip*, and only the
/// side's clip can answer it — in a window narrower than
/// [`super::plan::MIN_SIDE_WIDTH`] the side is drawn at the floor and clipped,
/// and the banner goes off screen with an entirely ordinary rectangle.
pub(super) fn draw(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    side: DockSide,
    area: Rect,
    banner: Option<&mut (DockSide, f32, &mut BannerHandler<'_>)>,
) -> Rect {
    let Some((wanted_side, requested, handler)) = banner else {
        return area;
    };
    if *wanted_side != side {
        return area;
    }
    let height = resolve_height(*requested, area.height());
    if height <= 0.0 {
        return area;
    }
    let strip = Rect::from_min_max(area.min, egui::pos2(area.right(), area.top() + height));
    ctx.reporter.report(ui, strip, || report::banner(side));

    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(strip)
            .layout(Layout::left_to_right(Align::Center)),
    );
    // ★ The clip is set explicitly rather than inherited. A child `Ui` built
    // from `max_rect` alone keeps its parent's clip, so a caller drawing a
    // second row would paint it straight over the first stack's tab bar —
    // visible, unclickable, and indistinguishable in a screenshot from a
    // layout fault.
    child.set_clip_rect(strip.intersect(ui.clip_rect()));
    handler(&mut child);

    Rect::from_min_max(egui::pos2(area.left(), strip.bottom()), area.max)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A request under the legible floor is refused outright rather than
    /// rounded up — the caller asked for chrome that cannot be read.
    #[test]
    fn a_request_below_the_floor_gets_no_banner() {
        assert_eq!(resolve_height(4.0, 800.0), 0.0);
        assert_eq!(resolve_height(MIN_HEIGHT - 0.01, 800.0), 0.0);
    }

    /// An ordinary request at an ordinary side height is honoured exactly.
    #[test]
    fn an_ordinary_request_is_honoured() {
        assert!((resolve_height(26.0, 800.0) - 26.0).abs() < f32::EPSILON);
    }

    /// ★ A greedy request is capped at a quarter of the side, so chrome can
    /// never become the majority of the dock.
    #[test]
    fn a_greedy_request_is_capped_at_a_quarter_of_the_side() {
        assert!((resolve_height(10_000.0, 800.0) - 200.0).abs() < f32::EPSILON);
    }

    /// ★★★ **A side too short for both a legible banner and a panel gets no
    /// banner at all** — the case the module header argues is the whole point
    /// of the clamp being a function.
    #[test]
    fn a_short_side_keeps_its_room_and_publishes_nothing() {
        // A quarter of 60 pt is 15 pt, under the 18 pt floor.
        assert_eq!(resolve_height(26.0, 60.0), 0.0);
        // And the boundary: a quarter of 72 pt is exactly the floor.
        assert!((resolve_height(26.0, 72.0) - MIN_HEIGHT).abs() < f32::EPSILON);
    }

    /// Nonsense in, nothing out. A `NaN` height from a caller's arithmetic
    /// must not become a `NaN` rectangle that silently swallows the side.
    #[test]
    fn a_non_finite_request_gets_no_banner() {
        assert_eq!(resolve_height(f32::NAN, 800.0), 0.0);
        assert_eq!(resolve_height(26.0, f32::NAN), 0.0);
        assert_eq!(resolve_height(f32::INFINITY, 800.0), 0.0);
    }
}
