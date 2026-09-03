//! # `canvas::overlay::anchors` — the marks that say **where the points are**
//!
//! Split out of `canvas/overlay.rs` on 2026-08-31 under **R2**, when
//! `OPERATOR_REQUESTS.md` O69 — *"the nodes are hard to see and click on"* —
//! took the parent to 1,479 of the 1,500-line ceiling with the viewport cull
//! still to write.
//!
//! ## ★ The seam is real, and the parent's own table draws it
//!
//! Everything left in `overlay` answers *"where is the thing you have
//! selected?"* — an outline, eight grips, a rotate handle, a ghost, a marquee,
//! a wash. Everything here answers *"where are its POINTS?"* — the anchor
//! marks, the Bézier handles, and the two published-region namings a driven
//! check aims at.
//!
//! They change for different reasons. The first changes when the selection
//! model gains a kind; the second changes when node editing does. O69 moved
//! only the second, and moved four things in it at once, which is the evidence
//! that it is one subject.
//!
//! ## What did NOT move, and why
//!
//! `draw_grips` stays in the parent. A grip is not a point on the drawing — it
//! is a handle on the selection's box — and the parent's own header argues
//! that the box and its handles are one decision drawn in one place, which is
//! what stops a handle being painted and not hit-tested.

use egui::{CornerRadius, Painter, Rect, StrokeKind, Visuals, epaint::Stroke};

use crate::canvas::mapping::PageMapping;

/// The size of an anchor mark, in screen pixels, edge to edge.
///
/// ★★★ **Six until 2026-08-31, and the argument for six had gone stale** —
/// `OPERATOR_REQUESTS.md` O69: *"the nodes are hard to see and click on."*
///
/// The reasoning it replaces read: *"A grip is a target — it must be grabbable
/// — and an anchor mark is primarily a statement: these are the points, and
/// these four are the ones you picked."* That was true when it was written and
/// stopped being true on 2026-08-19, when the **Node tool** made a single
/// click on an anchor the way an operator picks one. An anchor is now a target
/// in exactly the sense a grip is, and it was drawn smaller than one and
/// caught with less forgiveness than its own Bézier control point.
///
/// Seven, and both halves are borrowed rather than invented: it is
/// [`HANDLE_PX`], so an anchor is never smaller than the handle hanging off
/// it, and it is Inkscape's node size, which is the operator's stated
/// tie-breaker for this whole family of decisions. It stays visibly under the
/// 8 px resize grip, so the three marks are still three sizes.
///
/// The rest of the old argument survives: a run of anchors along a polyline
/// must still read as a row of dots rather than as a second outline. Seven
/// does; twelve would not.
///
/// ★ **It has a second consumer** — `canvas::pressing::grabbable` inflates the
/// inner-rung move box by this amount, so that an anchor sitting on the
/// object's bounding edge (half outside it) is still draggable. Widening the
/// mark widens that box by the same pixel, which is the right direction and is
/// named here so the coupling is not rediscovered.
pub const ANCHOR_PX: f32 = 7.0;

/// The most anchors that will be drawn as *unselected* marks.
///
/// ★ A real number from a real document rather than a round one: `canvas::moving`
/// records **6,681 anchors on one measured CAD export**, and a single object on
/// this operator's drawings routinely carries thousands. Painting all of them
/// would put several thousand filled rects in the frame for a rung the operator
/// entered in order to move *one* point, and the canvas would visibly stutter
/// at the exact moment they are doing precision work.
///
/// Above the cap the **selected** anchors still draw — they always draw, at any
/// count, because they are the answer to "what did I pick?" and that question
/// has no other surface — and the fact that the rest were suppressed is
/// disclosed off-canvas. That is rule 4's half that survives: an operator who
/// cannot see the unselected anchors would otherwise conclude the object has
/// none.
pub const MAX_UNSELECTED_ANCHORS: usize = 400;

/// Paint the entered object's anchors, and mark the selected ones.
///
/// # ★★ Why this exists at all, and why it is late
///
/// `FEATURES.md` recorded, against `view.show_points`, that **this build draws
/// no anchor mark at any rung** — and the Node rung has been enterable, and
/// multi-node selection representable, since S4. So an operator could descend
/// two rungs, Shift-click four anchors, and have **no way whatever** of seeing
/// which four. The feature was not merely undiscoverable; it was invisible.
///
/// It landed with the multi-node *move*, on 2026-08-19, because the two are one
/// feature: a set the operator cannot see is a set they cannot choose
/// deliberately, and a move of an invisible set is indistinguishable from a bug.
///
/// # Why selected and unselected are drawn differently, and how
///
/// Filled for selected, hollow for not. The same language as the resize grips
/// one rung up — filled square, selection-coloured stroke — so the visual
/// vocabulary of "a thing you can grab" is one vocabulary across the ladder,
/// and a reader who has learned the grips has learned these.
///
/// # ★ This is the CURSOR, not content
///
/// Rule 4 forbids styling *applied content* to express pdfcer's own uncertainty
/// and explicitly welcomes *pre-commit affordances*: "snap indicators, hover
/// highlights, rubber-bands and selection handles are the cursor". An anchor
/// mark is a selection handle. It describes where the operator may act, changes
/// nothing about how the page renders, and disappears the moment the rung is
/// left — so the one-line test holds: a screenshot of this canvas and a
/// screenshot of the same document saved and reopened differ only in the
/// cursor.
/// `points` are in **canvas space**, already converted by the caller.
///
/// ★ The conversion is the caller's because `PageMapping` speaks canvas ⟷
/// screen and knows nothing about PDF user space — turning a `vector::Point`
/// into a canvas position needs the `Page`'s own box and rotation, which is
/// `viewer::pdf_space_to_canvas`' job. Passing the `Page` in here so that this
/// function could do it would give the overlay a second coordinate authority,
/// and `coords`' standing rule is that a coordinate is produced by exactly one
/// conversion in exactly one place.
pub fn draw_anchors(
    painter: &Painter,
    visuals: &Visuals,
    mapping: &PageMapping,
    points: &[(usize, egui::Pos2)],
    selected: &std::collections::BTreeSet<usize>,
) {
    let stroke = Stroke::new(1.0, visuals.selection.stroke.color);

    // ★★★ **THE CAP COUNTS WHAT IS ON SCREEN, NOT WHAT EXISTS** —
    // `OPERATOR_REQUESTS.md` O69: *"the nodes are hard to see and click on."*
    //
    // ## What the operator was actually seeing, which is worse than "small"
    //
    // [`MAX_UNSELECTED_ANCHORS`] was compared against the whole set. On the
    // Points-tool route the selection sits at the **Part** rung with no node
    // picked, so `selected` is empty — and a subpath with more than 400
    // anchors therefore drew **not one dot**. A CAD contour, a hatch boundary
    // or a flattened spline chain passes 400 routinely; on his own SW41177.pdf
    // an object-rung census reads `total=4972`.
    //
    // So he armed Points, clicked a shape, watched the selection box change,
    // and nothing appeared. No dot, and no `canvas.anchor.N` region either, so
    // the harness could not aim at what was not there. **A bigger dot cannot
    // fix zero dots.**
    //
    // ## The fix is a cull, and it makes zooming in the remedy
    //
    // The cap exists to bound how many rectangles are painted, and an anchor
    // that is off screen costs a rectangle and shows nothing. Counting only
    // the visible ones keeps the bound exactly as tight while making the cap
    // fire on *what is in front of the operator* rather than on what the path
    // happens to contain.
    //
    // ⇒ The consequence is the one that matters: **zooming in now makes the
    // dots appear.** That is already the gesture an operator performs to work
    // on a point, and before this it made no difference at all.
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

    // ★★★ **The census is written BEFORE the empty return, since 2026-08-29.**
    //
    // It used to sit at the bottom, after `if points.is_empty() { return; }` —
    // so an object with no anchors and a draw that never happened produced the
    // **same trace**: nothing. That is the shape `tools/ui-verify`'s own rule 4
    // forbids from the other side (an absence is not evidence unless the thing
    // that would have produced it is known to run), and it cost a whole sweep's
    // worth of misclassification:
    //
    // * `the_points_tool_shows_points_on_one_click` and
    //   `show_points_draws_an_objects_points_without_descending` **FAILED**,
    //   each accusing a named line of `painting::draw_anchors`;
    // * `multi_node_move_moves_every_picked_anchor` and
    //   `bezier_handle_drag_changes_a_curve` **SKIPPED**, on the identical
    //   absence, saying *"the point named a text run or an image"*;
    //
    // and all four had aimed at the same `--doc-point 0,1140,62` on
    // `SW41177.pdf`, where `the_text_tool_types_on_one_click` passed on
    // `text-edit-caret run=426` — i.e. the aim is a **text run**, which has no
    // anchors, and the two failures were reports about the aim wearing the
    // clothes of reports about the code. All three of the callers that read
    // this line already have a `total == 0` arm written for exactly that case;
    // none of them could reach it, because the line they read it from was the
    // one being suppressed.
    //
    // ⇒ A census line states *what the draw was asked to draw*, which is a fact
    // even when the answer is nothing. Writing it unconditionally costs one
    // formatted string per frame in which anchors are in scope at all, and buys
    // the difference between "this object has no points" and "this function did
    // not run" — which is the whole question every anchor check asks first.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            // ★ `on_screen=` joined the census with the cull (O69). Without
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

    // ★ Iterates the culled set, whose screen positions were computed once
    // above — through the same mapping the outlines use, never a screen
    // position carried from the click, which would be a frame stale the
    // instant the operator scrolled.
    //
    // ★★ A SELECTED anchor that has been scrolled off screen is not drawn
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
            painter.rect(
                rect,
                CornerRadius::ZERO,
                visuals.selection.stroke.color,
                stroke,
                StrokeKind::Middle,
            );
        } else {
            // ★★★ **FILLED, not hollow** — `OPERATOR_REQUESTS.md` O69, and the
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
            // ★ So the distinction between picked and unpicked becomes the
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

    // ★★ Published so a driven check can aim at an anchor — the same argument
    // `SELECTION_OUTLINE_REGION`'s comment makes about the grips, and it is
    // stronger here: an anchor's position is a fact about the *decomposition*,
    // which no harness can compute without re-implementing the page walk.
    // ★★ The first selected anchor, and the first few DRAWN ones, published so
    // a driven check can aim at them.
    //
    // The selected one alone was not enough, and driving it is what showed
    // that: a check that has descended to the Part rung has selected *no*
    // anchor yet, so there was nothing to aim at and it could never reach the
    // Node rung at all. An anchor's screen position is a fact about the page's
    // decomposition — no harness can compute it without re-implementing the
    // content walk — so if the application does not say where they are, they
    // are undrivable.
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
        // ★ The CULLED set, so the regions name dots that are on screen —
        // O69. Publishing a region for an anchor scrolled out of view was the
        // trap `D:\devag\egui` records twice: the harness resolves a rect,
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
///
/// ★ Slightly larger than an anchor mark and **round** where anchors are
/// square, which is the vector-editor idiom every tool this operator has used
/// shares — Illustrator, Inkscape, Figma and the old shell all draw an on-curve
/// point as a square and a control point as a circle. It is not decoration: the
/// two are different kinds of thing and the shape is what says so at a glance,
/// with no legend and no hover.
pub const HANDLE_PX: f32 = 7.0;

/// Paint the Bézier handles of the selected anchors, each tethered to its
/// anchor.
///
/// # ★ Why the tether is not optional
///
/// A control point with no line back to the anchor it governs is an unexplained
/// dot floating beside a curve — and on a path with two selected anchors, four
/// such dots are ambiguous about which belongs to which. The tether is the only
/// thing that says *this handle steers that point*, and every editor draws it
/// for that reason.
///
/// It is drawn **thin and in the selection colour**, not dashed: a dashed line
/// on a CAD drawing competes with the drawing's own dashed linework, which is
/// the class of collision `DEFECTS.md` D12 records for the guide overlay.
///
/// # This is the CURSOR, not content
///
/// Rule 4's welcome list — "snap indicators, hover highlights, rubber-bands and
/// selection handles are the cursor". These vanish when the rung is left and
/// change nothing about how the page renders, so a screenshot of this canvas
/// and one of the same document saved and reopened differ only in the cursor.
pub fn draw_handles(
    painter: &Painter,
    visuals: &Visuals,
    mapping: &PageMapping,
    handles: &[(usize, pdfcer_core::vector::Handle, egui::Pos2)],
    anchors: &[(usize, egui::Pos2)],
) {
    if handles.is_empty() {
        return;
    }
    let colour = visuals.selection.stroke.color;
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
///
/// Six: enough for a driven check to aim at one and find a neighbour, few
/// enough that a subpath with two hundred anchors does not put two hundred
/// lines in the trace every time the layout moves.
pub const PUBLISHED_ANCHORS: usize = 6;

/// The region name for the `n`th drawn anchor.
///
/// A fixed set of `&'static str`s rather than a `format!`, because
/// `crate::diag::ui_rect` takes a `&'static str` by design — a region name is
/// part of the application's published vocabulary, not a runtime string, and
/// the harness's `driving::declared` matches on it exactly.
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
