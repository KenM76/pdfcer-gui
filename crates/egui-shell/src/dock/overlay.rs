//! `dock::overlay` — the five releases a drag is offered over the compartment
//! it is being held over, and the compartment a release would produce.
//!
//! [`super::drag`] owns a drag along its own tab strip, where the affordance is
//! an insertion caret. This module owns everything else: a drag carried over a
//! *different* compartment, which is the whole drop grammar
//! ([`super::drop::DropTarget`]) and the gesture an operator of any docking
//! application expects to reach it by.
//!
//! ## The three things it draws, and the one it does not
//!
//! 1. **The five zones**, filled over the hovered compartment's body — the
//!    quadrilaterals [`Compass::outline`] returns, which are the shapes
//!    [`Compass::zone_at`] hit-tests. Drawing the hit test means the operator
//!    cannot aim at a corner and be resolved somewhere else.
//! 2. **The armed zone**, at a stronger wash.
//! 3. **The outcome** — an outline around the rectangle the panel would occupy,
//!    obtained by replaying the candidate through the drop verb. See
//!    [`super::preview`] for why that is a replay and not a lookup.
//!
//! What it does not draw is anything over a compartment the drag is not over.
//! A compass painted on every stack at once is the affordance most of this
//! product class abandoned: it turns a hover into a search.
//!
//! ## ★ Why this runs after both sides have drawn
//!
//! The resolution needs the *whole* geometry — a drop against the left edge of
//! a right-hand stack is a column boundary on that side, and the replay walks
//! that side's shares. Nothing inside a compartment's own draw can see it, so
//! this runs once, from [`super::Dock::show`], immediately before
//! [`super::drag::settle`] reads what it published.
//!
//! The paint therefore goes to a foreground layer of its own rather than to a
//! `Ui`: by the time it runs, every panel body has already been painted into
//! its own layer, and an overlay drawn into the `Ui` that hosts the dock would
//! be *under* the panels it is describing.

use egui::{Color32, Rect, Shape, Stroke};

use super::compass::{Compass, DropLanding, DropZone};
use super::ctx::Ctx;
use super::drop::DropTarget;
use super::model::{DockLayout, PanelId};
use super::{drag, report};

/// **A drag held over a compartment, and what releasing it would do.**
///
/// Published on [`super::DockFrameReport`] because the visible form of this
/// affordance is a wash of colour over a rectangle: precise to look at, and
/// nothing a harness can assert on. Same reason as
/// [`super::drag::TabDragPreview`], which is its sibling for the other half of
/// the gesture.
#[derive(Clone, Debug, PartialEq)]
pub struct DropPreview {
    /// The panel being dragged.
    pub panel: PanelId,
    /// Which compartment the pointer is over, which zone of it, and the target
    /// the drop grammar would be asked for.
    pub landing: DropLanding,
    /// **The compartment the panel would occupy**, in the coordinates this
    /// frame's geometry was recorded in — the outcome, not the target.
    pub rect: Rect,
    /// Whether the release would change the layout at all.
    ///
    /// False for the legal releases that permute nothing: back into the middle
    /// of the group the panel already leads, or against an edge of a column it
    /// is already alone in. The highlight is knocked back rather than withheld,
    /// for the reason the caret is — refusing a legal drop would be a lie, and
    /// promising a move that will not happen is the other half of the same lie.
    pub lands: bool,
}

/// **Offer the drop zones, and publish what a release would do.**
///
/// Draws nothing unless a drag is in flight over a compartment that is not the
/// strip the drag began in — [`drag::preview`] owns that one and has already
/// drawn its caret, so this stands down whenever it published.
pub(super) fn draw(ui: &egui::Ui, ctx: &mut Ctx<'_>, layout: &DockLayout) {
    if ctx.tab_drag.is_some() {
        return;
    }
    let Some(panel) = drag::in_flight(ui, ctx) else {
        return;
    };
    let Some(pointer) = ui.ctx().pointer_latest_pos() else {
        return;
    };
    let Some(landing) = layout.resolve_drop(&ctx.geometry, pointer) else {
        return;
    };
    let Some(rect) = layout.preview_drop(&ctx.geometry, &panel, landing.target) else {
        return;
    };
    let lands = layout.drop_lands(&panel, landing.target);

    let painter = ui.ctx().layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        ctx.id_salt.with("dock-drop-overlay"), // ui-text-exempt: an id, never displayed
    ));
    let accent = ctx.theme.palette.accent;
    let ink = if lands {
        accent
    } else {
        accent.gamma_multiply(drag::CARET_DIMMED)
    };

    match landing.zone {
        // The pointer is on some *other* stack's tab strip. The question there
        // is which boundary, the answer is the same caret the reorder draws,
        // and it is drawn by the same function — see [`drag::caret_rect`].
        None => draw_strip_caret(ui, ctx, &painter, landing, ink),
        Some(zone) => draw_zones(ui, ctx, &painter, landing, zone, accent),
    }

    painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(OUTCOME_PTS, ink),
        egui::StrokeKind::Inside,
    );
    ctx.reporter.report(ui, rect, report::drop_outcome);
    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);

    ctx.drop_preview = Some(DropPreview {
        panel,
        landing,
        rect,
        lands,
    });
}

/// The insertion caret over a strip the drag did not start on.
fn draw_strip_caret(
    ui: &egui::Ui,
    ctx: &mut Ctx<'_>,
    painter: &egui::Painter,
    landing: DropLanding,
    ink: Color32,
) {
    let DropTarget::Tab { gap, .. } = landing.target else {
        return;
    };
    let Some(strip) = ctx.geometry.strip_rect(landing.over) else {
        return;
    };
    let caret = drag::caret_rect(&ctx.geometry, landing.over, gap, strip);
    painter.line_segment(
        [caret.center_top(), caret.center_bottom()],
        Stroke::new(drag::CARET_PTS, ink),
    );
    ctx.reporter.report(ui, caret, || {
        report::tab_caret(landing.over.side, landing.over.column, landing.over.stack)
    });
}

/// The five zones over the hovered compartment's body, the armed one stronger.
fn draw_zones(
    ui: &egui::Ui,
    ctx: &mut Ctx<'_>,
    painter: &egui::Painter,
    landing: DropLanding,
    armed: DropZone,
    accent: Color32,
) {
    let Some(body) = super::compass::body_of(&ctx.geometry, landing.over) else {
        return;
    };
    let compass = Compass::new(body);
    for zone in DropZone::ALL {
        let quad = compass.outline(zone);
        let alpha = if zone == armed { ARMED_A } else { RESTING_A };
        painter.add(Shape::convex_polygon(
            quad.to_vec(),
            wash(accent, alpha),
            Stroke::new(ZONE_EDGE_PTS, wash(accent, EDGE_A)),
        ));
    }
    let quad = compass.outline(armed);
    ctx.reporter.report(ui, Rect::from_points(&quad), || {
        report::drop_zone(
            landing.over.side,
            landing.over.column,
            landing.over.stack,
            armed,
        )
    });
}

/// The accent at a given transparency.
///
/// Not `gamma_multiply`, which scales an opaque colour's channels and so
/// *darkens* it against the panel instead of letting the panel through. The
/// zones sit over a panel body the operator must still be able to read.
fn wash(c: Color32, alpha: u8) -> Color32 {
    // NOT A THEME COLOUR: arithmetic on a role the caller already read from
    // the palette. The channels are `c`'s; only the alpha is this module's.
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha)
}

/// How opaque a zone that is merely on offer is drawn.
const RESTING_A: u8 = 26;

/// How opaque the zone the pointer is in is drawn.
///
/// Far enough above [`RESTING_A`] to be unmistakable at a glance, and still
/// short of hiding the panel underneath — the operator is choosing between five
/// places in a compartment whose contents are how they recognise it.
const ARMED_A: u8 = 96;

/// How opaque the hairline between two zones is.
const EDGE_A: u8 = 140;

/// The weight of that hairline.
const ZONE_EDGE_PTS: f32 = 1.0;

/// The weight of the outline around the compartment a release would produce.
///
/// Heavier than the zone edges: the zones are the question and this is the
/// answer, and the answer is frequently somewhere else on the screen entirely —
/// a column that does not exist yet, on the far side of the dock.
const OUTCOME_PTS: f32 = 2.0;
