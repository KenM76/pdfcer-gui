//! `dock::tear` — a drag carried out of the dock, and the window it would make.
//!
//! The dock answers a tab drag in three places, and this is the third.
//! [`super::drag`] owns a drag along its own tab strip, where the affordance is
//! an insertion caret. [`super::overlay`] owns a drag over a *different*
//! compartment, where it is the drop compass. This module owns the answer when
//! neither of those does: the operator has pulled the panel **out of the dock
//! altogether**, and a release there gives it a window of its own.
//!
//! ## What is offered, and where
//!
//! The pointer must be outside every side the dock drew, by the width of the
//! side's own splitter — see [`outside_the_dock`]. Inside that boundary the
//! drag belongs to one of the other two modules, and a frame on which either of
//! them published is one this module stands down on rather than one it
//! re-decides.
//!
//! The affordance is an outline of the window the release would open, drawn at
//! the pointer. It is a pre-commit affordance — the cursor, not content — so it
//! is welcome on the canvas it is drawn over; nothing about the document
//! underneath is marked, and the outline is gone the instant the button is.
//!
//! ## ★ The window opens where the operator let go
//!
//! Not at [`super::float::opening_position`]'s cascade, which is the answer for
//! the *command* route, where there is no pointer and no place the operator has
//! indicated. A drag ends somewhere on purpose, and a window that ignored that
//! and opened at the application's corner would have thrown away the one piece
//! of information the gesture carries over the command.
//!
//! The position is carried on the intent as **desktop** points, because that is
//! what [`FloatingPanel::pos_pts`] stores and what
//! `egui::ViewportBuilder::with_position` speaks. The conversion from this
//! frame's screen points is the application window's inner origin, which the
//! platform reports; a headless frame reports nothing and the two coordinate
//! spaces coincide at the origin.
//!
//! ## ★★ Two places inside the dock's footprint where a tear is still offered
//!
//! A **collapsed side's rail** and nothing else: it draws, but it publishes no
//! side rectangle, because there are no columns under it to resolve a drop
//! against. So a release on a collapsed rail tears the panel out rather than
//! doing nothing.
//!
//! That is deliberate and it is the better of the two available answers. There
//! is no grammar for dropping *into* a collapsed side — the side would have to
//! be re-opened to have a compartment to land in — so the alternative is a
//! release that does nothing at all, at a place the operator aimed at. The tear
//! is disclosed before it happens, by the outline, and it is reversible from
//! the window's own header. When a collapsed side learns to accept a drop, this
//! predicate tightens and the case disappears.

use egui::{Rect, Stroke, Vec2};

use super::ctx::Ctx;
use super::model::{DockSide, PanelId};
use super::{drag, float, overlay, plan, report};

/// **A drag held outside the dock, and the window a release would open.**
///
/// Published on [`super::DockFrameReport`] for [`super::overlay::DropPreview`]'s
/// reason: the visible form of the affordance is an outline, which is precise
/// to look at and nothing a harness can assert on.
#[derive(Clone, Debug, PartialEq)]
pub struct TearPreview {
    /// The panel being dragged.
    pub panel: PanelId,
    /// **The window's outline**, in this frame's screen points — the rectangle
    /// that was drawn.
    pub rect: Rect,
    /// **Where the window would open**, in desktop points.
    ///
    /// The same quantity as [`Self::rect`]'s corner, in the space
    /// [`super::float::FloatingPanel::pos_pts`] is stored in. Equal to it when
    /// the platform has not reported the application window's geometry — a
    /// first frame, or a headless harness.
    pub at_pts: [f32; 2],
}

/// **Offer the tear, and publish what a release would do.**
///
/// Runs from [`super::Dock::show`] after [`super::overlay::draw`] and before
/// [`super::drag::settle`], so that the two affordances it defers to have
/// already had their say and the settlement reads one decision.
///
/// ## ★ The stand-down is redundant today, and is kept anyway
///
/// Both of the other two affordances require the pointer to be *inside* a
/// compartment — a caret needs the strip it is inserting into, a compass needs
/// the body it divides — so [`outside_the_dock`] already excludes every frame
/// on which either of them published. The guard below therefore cannot be
/// reached by any input, which means no test can falsify it: planting a defect
/// in it leaves the suite green.
///
/// It stays because the implication is a property of a predicate that is
/// expected to move — the module header names the case that will tighten it —
/// and a release build where it no longer holds should draw one affordance
/// rather than two. What measures the implication is the assertion in
/// [`super::drag::settle`], which runs on every debug frame and fails by name
/// the moment two of the three answer one drag.
pub(super) fn draw(ui: &egui::Ui, ctx: &mut Ctx<'_>) {
    if ctx.tab_drag.is_some() || ctx.drop_preview.is_some() {
        return;
    }
    let Some(panel) = drag::in_flight(ui, ctx) else {
        return;
    };
    let Some(pointer) = ui.ctx().pointer_latest_pos() else {
        return;
    };
    if !outside_the_dock(ctx, pointer) {
        return;
    }

    let size = float::clamp_size(float::DEFAULT_SIZE_PTS);
    let rect = Rect::from_min_size(pointer - GRAB_PTS, Vec2::new(size[0], size[1]));

    // A layer of its own, above the panel bodies, for the reason
    // [`super::overlay`] gives: by the time this runs every body has been
    // painted into its own layer, and an outline drawn into the `Ui` that hosts
    // the dock would be underneath the thing it is describing.
    let painter = ui.ctx().layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        ctx.id_salt.with("dock-tear-outline"), // ui-text-exempt: an id, never displayed
    ));
    let accent = ctx.theme.palette.accent;
    painter.rect_filled(rect, 0.0, overlay::wash(accent, overlay::RESTING_A));
    // The header band is what makes the outline read as a *window* rather than
    // as one more drop region. The operator is being offered a different kind
    // of thing from the five the compass offers, and the two affordances are on
    // screen within a few points of each other as the pointer crosses the dock's
    // edge.
    let header = Rect::from_min_size(rect.min, Vec2::new(rect.width(), plan::TAB_BAR_HEIGHT));
    painter.rect_filled(header, 0.0, overlay::wash(accent, overlay::ARMED_A));
    painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(overlay::OUTCOME_PTS, accent),
        egui::StrokeKind::Inside,
    );
    ctx.reporter.report(ui, rect, report::tear_outline);
    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);

    let origin = ui
        .ctx()
        .input(|i| i.viewport().inner_rect)
        .map_or(egui::Vec2::ZERO, |r| r.min.to_vec2());
    let at = rect.min + origin;

    ctx.tear = Some(TearPreview {
        panel,
        rect,
        at_pts: [at.x, at.y],
    });
}

/// **Whether `pointer` is clear of every side the dock drew.**
///
/// Each side is grown by [`super::plan::SPLITTER_THICKNESS`] before the test,
/// because [`super::geometry::DockGeometry::side_rect`] holds the side's
/// *columns*, and the side's width handle sits outside them on the edge facing
/// the document. Without the growth the handle would be a few points of tear
/// zone lying along the whole height of the dock's inner edge — the one place a
/// drag crossing between two compartments passes through.
fn outside_the_dock(ctx: &Ctx<'_>, pointer: egui::Pos2) -> bool {
    !DockSide::ALL.iter().any(|&side| {
        ctx.geometry
            .side_rect(side)
            .is_some_and(|r| r.expand(plan::SPLITTER_THICKNESS).contains(pointer))
    })
}

/// Where the pointer sits inside the window it is about to make, on both axes.
///
/// The window's top-left goes this far up and to the left of the release, which
/// leaves the cursor on the platform's own title bar — the window's outer frame
/// begins above its inner rectangle by about this much. So the gesture ends with
/// the pointer holding the thing it just made, and an operator who keeps the
/// button down can carry on dragging the window itself.
const GRAB_PTS: Vec2 = Vec2::splat(24.0);
