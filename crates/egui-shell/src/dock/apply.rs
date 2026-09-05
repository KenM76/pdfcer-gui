//! `dock::apply` — the one place the layout is mutable.
//!
//! # Why this is a file of its own
//!
//! [`super::Dock::show`]'s three phases are **snapshot, draw, apply**, and
//! its own documentation calls the third *"the one place the layout is
//! mutable"*. A phase with that property is a subject, and a subject gets a
//! file — the same reasoning [`super::plan`] is split on.
//!
//! Split out of [`super`] under **R2** on 2026-09-04, when the three float
//! intents took `dock/mod.rs` past 1,500 lines. Nothing moved but the
//! module boundary: [`apply`] is the same function it was, and it is
//! `pub(super)` rather than public because it is `Dock::show`'s internals,
//! not an API.
//!
//! # ★★ The property every arm here depends on
//!
//! **"Did anything change" is decided ONCE, by comparing the whole layout
//! against a clone taken before the loop.** No arm sets a flag. That is
//! what stops a new intent — and three arrived the day this file was
//! created — from forgetting to say it changed something, which would leave
//! the application not persisting an edit the operator can see.
//!
//! [`super::floatwin::apply_float_intents`] is the float windows' half of
//! the same phase and holds the same property by the same mechanism. The two
//! are separate functions because they run at two different points in the
//! frame and act on disjoint intent sets, not because the discipline
//! differs.

use super::DockFrameReport;
use super::ctx::Intent;
use super::model::{DockLayout, DockSide};
use super::plan;

/// Apply one frame's intents to the layout.
///
/// The **only** function in this module that takes `&mut DockLayout`.
/// Returns whether anything changed, which the application uses to decide
/// whether the layout is worth persisting.
///
/// Splitter drags are applied by resolving the *current* spans, moving
/// one boundary with [`plan::drag_boundary`], and converting back — which
/// is the one place [`plan::spans_to_shares`] may be called, and its
/// documentation says why.
pub(super) fn apply(
    layout: &mut DockLayout,
    intents: &[Intent],
    report: &mut DockFrameReport,
) -> bool {
    if intents.is_empty() {
        return false;
    }
    let before = layout.clone();

    for intent in intents {
        match intent {
            // ★★ Collapse a side, or bring it back. One toggle, two controls:
            // the chevron on an open side and the rail on a shut one, neither
            // of which can be pressed in the state the other lives in.
            Intent::ToggleSide(side) => {
                let s = match side {
                    DockSide::Left => &mut layout.left,
                    DockSide::Right => &mut layout.right,
                };
                s.visible = !s.visible;
                // The arrangement is UNTOUCHED. Collapsing is a view state, not
                // a structural edit — which is the whole difference between
                // "collapse the dock" and "reset the dock", and the reason an
                // operator can minimise a side and get their columns back
                // exactly as they left them.
                //
                // No `changed` flag: this function compares the whole layout
                // against a clone taken before the loop, so a mutation IS the
                // signal. One place decides "did anything move", which is what
                // stops a new intent forgetting to say so.
            }
            Intent::Activate(panel) => {
                if layout.activate(panel) {
                    report.activated = Some(panel.clone());
                }
            }
            Intent::Close(panel) => {
                if layout.close(panel) {
                    report.closed = Some(panel.clone());
                }
            }
            // The three float arms are one line each on purpose: the rules
            // live in `float.rs`, where they can be tested with no window
            // open, and this function stays a router rather than growing a
            // second place that knows what floating means.
            Intent::Float(panel) => {
                if layout.float(panel) {
                    report.floated = Some(panel.clone());
                }
            }
            Intent::Dock(panel) => {
                if layout.dock_back(panel) {
                    report.docked = Some(panel.clone());
                }
            }
            Intent::FloatGeometry { panel, pos, size } => {
                layout.set_float_geometry(panel, *pos, *size);
            }
            Intent::DragSide { side, delta } => {
                let s = layout.side_mut(*side);
                s.width_pts = (s.width_pts + delta).max(plan::MIN_SIDE_WIDTH);
            }
            Intent::DragColumns {
                side,
                boundary,
                delta,
            } => {
                let s = layout.side_mut(*side);
                let shares: Vec<f32> = s.columns.iter().map(|c| c.share).collect();
                // Resolved against a nominal total rather than the real
                // one. The real width is not available here — this runs
                // after the frame — and it does not need to be: a drag
                // of `delta` points is a fraction of the side's width,
                // and the side's width is `width_pts`. Using it keeps the
                // delta in the same units the operator moved the pointer
                // in.
                let total = s.width_pts.max(plan::MIN_SIDE_WIDTH);
                let mut spans = plan::resolve_spans(
                    &shares,
                    total,
                    plan::MIN_COLUMN_WIDTH,
                    plan::SPLITTER_THICKNESS,
                );
                plan::drag_boundary(&mut spans, *boundary, *delta, plan::MIN_COLUMN_WIDTH);
                for (c, share) in s.columns.iter_mut().zip(plan::spans_to_shares(&spans)) {
                    c.share = share;
                }
            }
            Intent::EqualizeColumns { side, boundary } => {
                let s = layout.side_mut(*side);
                equalize(
                    &mut s
                        .columns
                        .iter_mut()
                        .map(|c| &mut c.share)
                        .collect::<Vec<_>>(),
                    *boundary,
                );
            }
            Intent::DragStacks {
                side,
                column,
                boundary,
                delta,
            } => {
                let s = layout.side_mut(*side);
                let Some(col) = s.columns.get_mut(*column) else {
                    continue;
                };
                let shares: Vec<f32> = col.stacks.iter().map(|s| s.share).collect();
                // The column's height in points is not known here either.
                // A nominal total works because `drag_boundary`'s delta
                // and the minimums are both in points and the conversion
                // back to shares is scale-free; the only visible effect
                // of a nominal total is that a drag near the minimum
                // resists slightly sooner or later than the pixel it was
                // drawn at. A real height would have to be smuggled out
                // of the draw phase, which is exactly the `&mut`-during-
                // draw this design refuses.
                let total = NOMINAL_COLUMN_HEIGHT;
                let mut spans = plan::resolve_spans(
                    &shares,
                    total,
                    plan::MIN_STACK_HEIGHT,
                    plan::SPLITTER_THICKNESS,
                );
                plan::drag_boundary(&mut spans, *boundary, *delta, plan::MIN_STACK_HEIGHT);
                for (st, share) in col.stacks.iter_mut().zip(plan::spans_to_shares(&spans)) {
                    st.share = share;
                }
            }
            Intent::EqualizeStacks {
                side,
                column,
                boundary,
            } => {
                let s = layout.side_mut(*side);
                let Some(col) = s.columns.get_mut(*column) else {
                    continue;
                };
                equalize(
                    &mut col
                        .stacks
                        .iter_mut()
                        .map(|s| &mut s.share)
                        .collect::<Vec<_>>(),
                    *boundary,
                );
            }
        }
    }

    layout.normalize();
    *layout != before
}

/// The height a stack drag is resolved against when the real one is not
/// available. See the call site.
const NOMINAL_COLUMN_HEIGHT: f32 = 800.0;

/// Give the two children either side of `boundary` equal share, leaving
/// every other child alone — failure mode #7 applied to the double-click
/// gesture as well as to the drag.
pub(super) fn equalize(shares: &mut [&mut f32], boundary: usize) {
    if boundary + 1 >= shares.len() {
        return;
    }
    let mean = (*shares[boundary] + *shares[boundary + 1]) / 2.0;
    *shares[boundary] = mean;
    *shares[boundary + 1] = mean;
}
