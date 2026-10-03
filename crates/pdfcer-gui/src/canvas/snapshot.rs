//! # `canvas::snapshot` — the View > Snapshot tool on the canvas
//!
//! A drag with the tool armed lays a [`SnapshotBox`] on the document, in PDF
//! user space; a drag that starts on the box moves it, and one that starts on
//! a grip resizes it. The box lives while the tool is armed and is cleared
//! when the tool is put down. It is a pre-commit affordance: nothing is
//! authored, and the saved document never contains it.
//!
//! Trace: `snapshot-tool armed=` on arming, `snapshot-box page= llx= lly= urx=
//! ury= part=` when a drag is released, `snapshot-cleared reason=` when the box
//! goes, and the screen rect as region `canvas.snapshot` on every painted
//! frame.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/snapshot.md`.

use egui::{CornerRadius, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};
use pdfcer_gui_base::handles::{Grip, GripSet, grip_at};
use pdfcer_gui_base::snapshotbox::SnapshotBox;

use crate::app::state::OpenDoc;
use crate::canvas::gesture::Phase;
use crate::canvas::mapping::PageMapping;
use crate::canvas::markup::band::Preview;
use crate::canvas::tool::CanvasTool;
use crate::viewer;

/// The box's screen rect, declared to the trace.
pub const SNAPSHOT_REGION: &str = "canvas.snapshot"; // ui-text-exempt: trace region name, never displayed

const GRAB_MEMORY_KEY: &str = "pdfcer-canvas-snapshot-grab"; // ui-text-exempt: internal memory id, never displayed

/// What a drag on the tool is doing, decided once at its first frame.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Grab {
    /// The press point in canvas space; a different one is a new drag.
    from: Pos2,
    /// `None` draws a new box; `Some` moves or resizes `original`.
    grip: Option<Grip>,
    /// The box's canvas rect when the drag began.
    original: Rect,
}

/// Arm the snapshot tool, or put it down when it is already armed.
pub fn toggle(ctx: &egui::Context) -> CanvasTool {
    let next = if crate::canvas::tool::selected(ctx) == CanvasTool::Snapshot {
        CanvasTool::Select
    } else {
        CanvasTool::Snapshot
    };
    crate::canvas::tool::select(ctx, next);
    let armed = next == CanvasTool::Snapshot;
    // ui-text-exempt: diagnostic trace, never displayed.
    crate::diag::trace(|| format!("snapshot-tool armed={armed}"));
    next
}

/// Clear the box once the tool is no longer armed, whatever put it down.
pub fn settle(ctx: &egui::Context, doc: &mut OpenDoc) {
    if doc.snapshot.is_none() || crate::canvas::tool::selected(ctx) == CanvasTool::Snapshot {
        return;
    }
    doc.snapshot = None;
    // ui-text-exempt: diagnostic trace, never displayed.
    crate::diag::trace(|| "snapshot-cleared reason=tool".to_owned());
}

/// The rubber band to paint for one frame of a drag on the tool: `Some`
/// while it draws a new box, `None` while it moves or resizes one.
#[must_use]
pub fn band(
    ctx: &egui::Context,
    doc: &OpenDoc,
    map: &PageMapping,
    from: Pos2,
    to: Pos2,
) -> Option<Preview> {
    grab_for(ctx, doc, map, from)
        .grip
        .is_none()
        .then(|| crate::canvas::placing::band(from, to))
}

/// Apply one frame of a drag on the tool to the box; called once the frame's
/// object borrow is dropped, after [`band`] has decided the drag.
pub fn dragged(
    ctx: &egui::Context,
    doc: &mut OpenDoc,
    map: &PageMapping,
    from: Pos2,
    to: Pos2,
    phase: Phase,
) {
    let grab = grab_for(ctx, doc, map, from);
    if phase == Phase::Complete {
        ctx.data_mut(|d| d.remove::<Grab>(egui::Id::new(GRAB_MEMORY_KEY)));
    }
    let Some(grip) = grab.grip else {
        if phase == Phase::Complete {
            lay(doc, from, to, "new");
        }
        return;
    };
    let r = reshaped(grip, grab.original, to - from);
    let part = if grip == Grip::Move { "move" } else { "resize" };
    if phase == Phase::Complete {
        lay(doc, r.min, r.max, part);
    } else {
        set(doc, r.min, r.max);
    }
}

/// The drag's [`Grab`], decided on its first frame and remembered after.
fn grab_for(ctx: &egui::Context, doc: &OpenDoc, map: &PageMapping, from: Pos2) -> Grab {
    let id = egui::Id::new(GRAB_MEMORY_KEY);
    if let Some(held) = ctx
        .data(|d| d.get_temp::<Grab>(id))
        .filter(|g| g.from == from)
    {
        return held;
    }
    let fresh = match canvas_rect(doc, doc.view.page_index) {
        Some(original) => Grab {
            from,
            grip: part_at(map.rect_to_screen(original), map.to_screen(from)),
            original,
        },
        None => Grab {
            from,
            grip: None,
            original: Rect::NOTHING,
        },
    };
    ctx.data_mut(|d| d.insert_temp(id, fresh));
    fresh
}

/// The pointer over the box: a grip's resize cursor, a move cursor over the
/// body, `Grabbing` while a move is held; `None` leaves the tool's crosshair.
#[must_use]
pub fn cursor(
    ctx: &egui::Context,
    doc: &OpenDoc,
    map: &PageMapping,
    tool: CanvasTool,
) -> Option<egui::CursorIcon> {
    if tool != CanvasTool::Snapshot {
        return None;
    }
    if let Some(held) = ctx.data(|d| d.get_temp::<Grab>(egui::Id::new(GRAB_MEMORY_KEY))) {
        return held.grip.map(|g| match g {
            Grip::Move => egui::CursorIcon::Grabbing,
            other => other.cursor(),
        });
    }
    let pointer = ctx.pointer_latest_pos()?;
    let screen = screen_rect(doc, doc.view.page_index, map)?;
    part_at(screen, pointer).map(Grip::cursor)
}

/// Which part of a box at `screen` a press at `pointer` takes hold of.
fn part_at(screen: Rect, pointer: Pos2) -> Option<Grip> {
    grip_at(screen, pointer, GripSet::scale_only())
}

/// `original` with the edges `grip` holds moved by `delta`, all in canvas
/// space; a resize past the opposite edge flips rather than inverts.
fn reshaped(grip: Grip, original: Rect, delta: Vec2) -> Rect {
    if grip == Grip::Move {
        return original.translate(delta);
    }
    let (mut min, mut max) = (original.min, original.max);
    if matches!(grip, Grip::NorthWest | Grip::North | Grip::NorthEast) {
        min.y += delta.y;
    }
    if matches!(grip, Grip::SouthWest | Grip::South | Grip::SouthEast) {
        max.y += delta.y;
    }
    if matches!(grip, Grip::NorthWest | Grip::West | Grip::SouthWest) {
        min.x += delta.x;
    }
    if matches!(grip, Grip::NorthEast | Grip::East | Grip::SouthEast) {
        max.x += delta.x;
    }
    Rect::from_two_pos(min, max)
}

/// The box with canvas corners `a`, `b` on the current page, if big enough.
fn boxed(doc: &OpenDoc, a: Pos2, b: Pos2) -> Option<SnapshotBox> {
    let page = doc.current_page()?;
    let (a, b) = crate::canvas::markup::band::endpoints(a, b, page)?;
    SnapshotBox::from_corners(doc.view.page_index, a, b)
}

/// Follow the pointer mid-drag; a box too small to keep leaves the last one.
fn set(doc: &mut OpenDoc, a: Pos2, b: Pos2) {
    if let Some(next) = boxed(doc, a, b) {
        doc.snapshot = Some(next);
    }
}

/// Settle the box at the drag's release, and trace where it landed.
fn lay(doc: &mut OpenDoc, a: Pos2, b: Pos2, part: &'static str) {
    set(doc, a, b);
    if let Some(laid) = doc.snapshot {
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| format!("snapshot-box {} part={part}", laid.trace_fields()));
    }
}

/// The box's canvas rect on `page_index`; `None` when it is on another page.
fn canvas_rect(doc: &OpenDoc, page_index: usize) -> Option<Rect> {
    let laid = doc.snapshot.filter(|s| s.page == page_index)?;
    let page = doc.pages.get(page_index)?;
    let r = laid.rect;
    #[allow(clippy::cast_possible_truncation)]
    // ui-text-exempt: a clippy lint name, never displayed
    let corner = |x: f64, y: f64| viewer::pdf_space_to_canvas(Pos2::new(x as f32, y as f32), page);
    Some(Rect::from_two_pos(
        corner(r.llx, r.lly)?,
        corner(r.urx, r.ury)?,
    ))
}

/// The box's screen rect on `page_index` through `map`; `None` when the box is
/// on another page or the page cannot be mapped.
#[must_use]
pub fn screen_rect(doc: &OpenDoc, page_index: usize, map: &PageMapping) -> Option<Rect> {
    canvas_rect(doc, page_index).map(|r| map.rect_to_screen(r))
}

/// Paint the box on the page being drawn, and declare where it landed.
pub fn paint(painter: &Painter, doc: &OpenDoc, page_index: usize, map: &PageMapping) {
    let Some(screen) = screen_rect(doc, page_index, map) else {
        return;
    };
    let ink = egui_shell::theme::Theme::canvas_selection_ink(painter.ctx());
    painter.rect_stroke(
        screen,
        CornerRadius::ZERO,
        Stroke::new(1.0, ink),
        StrokeKind::Middle,
    );
    crate::diag::ui_rect(SNAPSHOT_REGION, screen);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r() -> Rect {
        Rect::from_min_max(Pos2::new(100.0, 100.0), Pos2::new(300.0, 200.0))
    }

    #[test]
    fn a_press_inside_moves_and_one_outside_draws_a_new_box() {
        assert_eq!(part_at(r(), Pos2::new(200.0, 150.0)), Some(Grip::Move));
        assert_eq!(part_at(r(), Pos2::new(500.0, 500.0)), None);
        assert_eq!(part_at(r(), Pos2::new(300.0, 200.0)), Some(Grip::SouthEast));
    }

    #[test]
    fn a_move_keeps_the_size_and_a_side_grip_moves_one_edge() {
        let d = Vec2::new(10.0, 20.0);
        assert_eq!(reshaped(Grip::Move, r(), d), r().translate(d));
        let east = reshaped(Grip::East, r(), d);
        assert_eq!((east.min, east.max), (r().min, Pos2::new(310.0, 200.0)));
        let north = reshaped(Grip::North, r(), d);
        assert_eq!((north.min, north.max), (Pos2::new(100.0, 120.0), r().max));
    }

    #[test]
    fn a_corner_dragged_past_its_opposite_flips_the_box() {
        let flipped = reshaped(Grip::SouthEast, r(), Vec2::new(-300.0, 0.0));
        assert_eq!(flipped.min, Pos2::new(0.0, 100.0));
        assert_eq!(flipped.max, Pos2::new(100.0, 200.0));
    }
}
