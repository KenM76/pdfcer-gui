//! # `canvas::overlay::anchors` — the marks that say **where the points are**
//!
//! ## The seam against the parent
//!
//! Everything in `overlay` answers *"where is the thing you have selected?"* —
//! an outline, eight grips, a rotate handle, a ghost, a marquee, a wash.
//! Everything here answers *"where are its POINTS?"* — the anchor marks, the
//! Bézier handles, and the two published-region namings a driven check aims at.
//!
//! They change for different reasons: the first when the selection model gains
//! a kind, the second when node editing does.
//!
//! ## What did NOT move, and why
//!
//! `draw_grips` stays in the parent. A grip is not a point on the drawing — it
//! is a handle on the selection's box — and the parent's own header argues
//! that the box and its handles are one decision drawn in one place, which is
//! what stops a handle being painted and not hit-tested.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/overlay/anchors.md`.

use egui::{CornerRadius, Painter, Rect, StrokeKind, Visuals, epaint::Stroke};

use crate::canvas::mapping::PageMapping;

/// The size of an anchor mark, in screen pixels, edge to edge.
pub const ANCHOR_PX: f32 = 7.0;

/// The most anchors that will be drawn as *unselected* marks.
pub const MAX_UNSELECTED_ANCHORS: usize = 400;

/// Paint the entered object's anchors, and mark the selected ones.
pub fn draw_anchors(
    painter: &Painter,
    visuals: &Visuals,
    mapping: &PageMapping,
    points: &[(usize, egui::Pos2)],
    selected: &std::collections::BTreeSet<usize>,
) {
    // Read by its ROLE NAME, never through `visuals.selection` — that is
    // `egui`'s selected-widget channel, so pointing the canvas at it would
    // fuse this ink with every selected chrome control in the application.
    // See `overlay::ink`; `tools/gates/check-selection-channel.sh` keeps that
    // address out. Hoisted because the fill below is inside the per-anchor
    // loop and the lookup takes the context's data lock.
    let ink = egui_shell::theme::Theme::canvas_selection_ink(painter.ctx());
    let stroke = Stroke::new(1.0, ink);

    // **THE CAP COUNTS WHAT IS ON SCREEN, NOT WHAT EXISTS** —
    // `OPERATOR_REQUESTS.md` O69: *"the nodes are hard to see and click on."*
    //
    // [`MAX_UNSELECTED_ANCHORS`] bounds how many rectangles are painted, and
    // an anchor that is off screen costs a rectangle and shows nothing.
    // Comparing it against the whole set makes a CAD contour, a hatch boundary
    // or a flattened spline chain — all of which pass 400 routinely, and one
    // measured object rung reads `total=4972` — draw **not one dot**, at any
    // zoom, with no `canvas.anchor.N` region for a driven check to aim at
    // either. On the Points-tool route `selected` is empty, so nothing else
    // would be drawn in its place. A bigger dot cannot fix zero dots.
    //
    // ⇒ Counting only the visible ones keeps the bound exactly as tight while
    // making the cap fire on *what is in front of the operator* rather than on
    // what the path happens to contain. The consequence is the one that
    // matters: **zooming in makes the dots appear**, and zooming in is already
    // the gesture an operator performs to work on a point.
    //
    // `painter.clip_rect()` is `Frame::clip` — the scroll viewport, set by
    // `canvas::present` and bound in `canvas::painting` — so no new parameter
    // is needed and the cull cannot disagree with what was drawn. Expanded by
    // one mark, so a dot straddling the edge is still drawn rather than
    // popping in as it crosses.
    let view = painter.clip_rect().expand(ANCHOR_PX);
    let on_screen: Vec<(usize, egui::Pos2, egui::Pos2)> = points
        .iter()
        .map(|(i, p)| (*i, *p, mapping.to_screen(*p)))
        .filter(|(_, _, at)| view.contains(*at))
        .collect();
    let draw_unselected = on_screen.len() <= MAX_UNSELECTED_ANCHORS;

    // **The census is written BEFORE the empty return**, and the order is
    // the contract rather than a convenience.
    //
    // A census line states *what the draw was asked to draw*, which is a fact
    // even when the answer is nothing. Behind the empty return, an object with
    // no anchors and a draw that never happened emit the **same trace**:
    // nothing — the shape `tools/ui-verify`'s rule 4 forbids from the other
    // side, since an absence is not evidence unless the thing that would have
    // produced it is known to run. Every caller that reads this line has a
    // `total == 0` arm for exactly that case; suppressing the line makes that
    // arm unreachable, and turns "the aim was a text run, which has no
    // anchors" into an accusation against a named line of this function.
    //
    // ⇒ Writing it unconditionally costs one formatted string per frame in
    // which anchors are in scope, and buys the difference between "this object
    // has no points" and "this function did not run" — the question every
    // anchor check asks first.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            // `on_screen=` joined the census with the cull (O69). Without
            // it "the cap fired" and "the operator has scrolled away from the
            // points" are the same line, and they need different responses.
            // The first token is unchanged — four driven checks read it.
            "canvas-anchors total={} on_screen={} selected={} unselected_drawn={}",
            points.len(),
            on_screen.len(),
            selected.len(),
            if draw_unselected { on_screen.len() } else { 0 }
        )
    });
    if points.is_empty() {
        return;
    }

    // Iterates the culled set, whose screen positions were computed once
    // above — through the same mapping the outlines use, never a screen
    // position carried from the click, which would be a frame stale the
    // instant the operator scrolled.
    //
    // A SELECTED anchor that has been scrolled off screen is not drawn
    // either, and that is not a loss: it is off screen. What it does mean is
    // that the always-draw-selected escape hatch below is now scoped to the
    // visible set too, which keeps the painted count bounded by the viewport
    // in every case rather than in most of them.
    for (index, _, at) in &on_screen {
        let is_selected = selected.contains(index);
        if !is_selected && !draw_unselected {
            continue;
        }
        let rect = Rect::from_center_size(*at, egui::vec2(ANCHOR_PX, ANCHOR_PX));
        if is_selected {
            painter.rect(rect, CornerRadius::ZERO, ink, stroke, StrokeKind::Middle);
        } else {
            // **FILLED, not hollow** — `OPERATOR_REQUESTS.md` O69, and the
            // single highest-value half of *"the nodes are hard to see"*.
            //
            // The argument is not new: it is written out fifteen lines below,
            // in `draw_grips`' own header, and it was simply never applied
            // here — *"a filled square reads as a handle at any zoom and
            // against any page content, where an outline-only square
            // disappears over dense linework — which is precisely the document
            // class pdfcer is for."*
            //
            // A 1 px accent outline on a 7 px square, over black CAD linework,
            // is close to invisible. Filled with the window background it
            // reads as a mark sitting **on** the drawing rather than as four
            // thin lines competing with it.
            //
            // So the distinction between picked and unpicked becomes the
            // FILL COLOUR — accent versus window background — rather than
            // filled-versus-hollow. That is what Inkscape does, and it is the
            // stronger signal: two solid shapes differing in colour are told
            // apart at a glance, where solid-versus-outline needs a second
            // look at dot size.
            painter.rect(
                rect,
                CornerRadius::ZERO,
                visuals.window_fill,
                stroke,
                StrokeKind::Middle,
            );
        }
    }

    // The first selected anchor **and** the first few drawn ones, published
    // so a driven check can aim at them — the same argument
    // `SELECTION_OUTLINE_REGION`'s comment makes about the grips, and stronger
    // here: an anchor's screen position is a fact about the page's
    // decomposition, which no harness can compute without re-implementing the
    // content walk. If the application does not say where they are, they are
    // undrivable.
    //
    // Both are needed. A check that has descended only to the Part rung has
    // selected *no* anchor yet, so a selected-only region gives it nothing to
    // aim at and it can never reach the Node rung at all.
    //
    // Bounded at `PUBLISHED_ANCHORS`, because `ui-rect` is a change log and a
    // subpath with two hundred anchors would put two hundred lines in the trace
    // on every frame the layout moved. A handful is all a check needs: it aims
    // at one, and the sweep in `multi_node` finds a neighbour from there.
    if let Some((_, first)) = points.iter().find(|(i, _)| selected.contains(i)) {
        let at = mapping.to_screen(*first);
        crate::diag::ui_rect(
            SELECTED_ANCHOR_REGION,
            Rect::from_center_size(at, egui::vec2(ANCHOR_PX, ANCHOR_PX)),
        );
    }
    if draw_unselected {
        // The CULLED set, so the regions name dots that are on screen —
        // O69. Publishing a region for an anchor scrolled out of view was the
        // trap `D:/dev/rag/egui` records twice: the harness resolves a rect,
        // clicks its centre, and hits whatever is actually there. A rect that
        // is off screen is a click aimed at nothing, reported as a defect in
        // whatever the click did instead.
        for (n, (_, _, at)) in on_screen.iter().take(PUBLISHED_ANCHORS).enumerate() {
            crate::diag::ui_rect(
                anchor_region(n),
                Rect::from_center_size(*at, egui::vec2(ANCHOR_PX, ANCHOR_PX)),
            );
        }
    }
}

/// The diameter of a Bézier-handle mark, in screen pixels.
pub const HANDLE_PX: f32 = 7.0;

/// Paint the Bézier handles of the selected anchors, each tethered to its
/// anchor.
pub fn draw_handles(
    painter: &Painter,
    mapping: &PageMapping,
    handles: &[(usize, pdfcer_core::vector::Handle, egui::Pos2)],
    anchors: &[(usize, egui::Pos2)],
) {
    if handles.is_empty() {
        return;
    }
    // The content-area selection ink by name; see `overlay::ink`.
    let colour = egui_shell::theme::Theme::canvas_selection_ink(painter.ctx());
    let stroke = Stroke::new(1.0, colour);
    let radius = HANDLE_PX / 2.0;

    for (n, (node, _side, canvas)) in handles.iter().enumerate() {
        let at = mapping.to_screen(*canvas);
        // The tether, drawn FIRST so the marks sit on top of it rather than
        // being crossed by it.
        if let Some((_, anchor)) = anchors.iter().find(|(i, _)| i == node) {
            painter.line_segment([mapping.to_screen(*anchor), at], stroke);
        }
        // Hollow, always. A filled circle would read as "selected", and a
        // handle is never selected — it is grabbed and released. The one thing
        // a filled mark could mean here is a state this feature does not have.
        painter.circle_stroke(at, radius, stroke);

        if n < PUBLISHED_ANCHORS {
            crate::diag::ui_rect(
                handle_region(n),
                Rect::from_center_size(at, egui::vec2(HANDLE_PX, HANDLE_PX)),
            );
        }
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("canvas-handles n={}", handles.len())
    });
}

/// The region name for the `n`th drawn handle.
///
/// Same closed list and same reason as [`anchor_region`]: a region name is part
/// of the application's published vocabulary, not a runtime string.
#[must_use]
pub fn handle_region(n: usize) -> &'static str {
    // ui-text-exempt: diagnostic region names, never displayed.
    const NAMES: [&str; PUBLISHED_ANCHORS] = [
        "canvas.handle.0",
        "canvas.handle.1",
        "canvas.handle.2",
        "canvas.handle.3",
        "canvas.handle.4",
        "canvas.handle.5",
    ];
    NAMES[n.min(PUBLISHED_ANCHORS - 1)]
}

/// How many drawn anchors publish a `ui-rect` region.
pub const PUBLISHED_ANCHORS: usize = 6;

/// The region name for the `n`th drawn anchor.
#[must_use]
pub fn anchor_region(n: usize) -> &'static str {
    // ui-text-exempt: diagnostic region names, never displayed.
    const NAMES: [&str; PUBLISHED_ANCHORS] = [
        "canvas.anchor.0",
        "canvas.anchor.1",
        "canvas.anchor.2",
        "canvas.anchor.3",
        "canvas.anchor.4",
        "canvas.anchor.5",
    ];
    NAMES[n.min(PUBLISHED_ANCHORS - 1)]
}

/// The region the first selected anchor publishes.
pub const SELECTED_ANCHOR_REGION: &str = "canvas.selected-anchor"; // ui-text-exempt: trace region name, never displayed
