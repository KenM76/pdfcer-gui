//! # `canvas::shapes` — **the shape itself, following your hand**
//!
//! The live geometry preview. `OPERATOR_REQUESTS.md` **O63**.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/shapes.md`.

use egui::{Painter, Pos2, Stroke};
use pdfcer_core::vector::{Matrix, PaintStyle, Point, Segment, Subpath, VectorObject};

use crate::canvas::mapping::PageMapping;
use crate::panels::objects::provider::{ObjectModelProvider, TargetId};

/// How many objects a preview will carry before it gives up.
const MAX_OBJECTS: usize = 64;

/// How many segments across the whole preview before it gives up.
const MAX_SEGMENTS: usize = 8_000;

/// One object's geometry, in **page space**, ready to be mapped and painted.
#[derive(Debug, Clone, PartialEq)]
pub struct PreviewShape {
    /// Page-space subpaths, already transformed by whatever the gesture is
    /// doing.
    pub subpaths: Vec<Subpath>,
    /// Fill/stroke disposition at paint time (§8.5.3 Table 60).
    pub style: PaintStyle,
    /// Stroke width in **page-space** units.
    pub line_width: f64,
}

/// Everything one gesture is about to change, as geometry.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShapePreview {
    /// The shapes, in paint order, **at their new position**.
    pub shapes: Vec<PreviewShape>,
    /// The same shapes **where they still are** — the footprint to erase.
    ///
    /// # Why an erase list exists at all
    ///
    /// The page raster underneath is stale: it still shows the object where it
    /// was, and it cannot be re-rendered in under ~0.7 s on the operator's own
    /// drawing (`BENCHMARK.md` — a *two-pixel* region render costs 691 ms
    /// because ~99 % of render cost is content-stream interpretation, not fill).
    ///
    /// So without this the operator sees the object **twice**: once where it
    /// was, painted into the raster, and once where their pointer is. That is
    /// worse than the bounding box this feature replaced.
    ///
    /// # The footprint, not the bounding box — and that is the whole
    /// difference between acceptable and not
    ///
    /// **Ken, 2026-08-30:** *"yeah do both"*, accepting that erasing the old
    /// position would take whatever was underneath with it.
    ///
    /// It takes much less than he agreed to. Because the shell has the real
    /// geometry, the erase is the object's **own outline** — stroked at its own
    /// width, filled where it was filled — rather than a rectangle over it. On a
    /// CAD sheet a bounding box would blank a title-block cell; a stroked
    /// polyline blanks a line's own width.
    ///
    /// ⇒ What is still a lie, stated plainly: anything drawn *underneath the
    /// object's own footprint* disappears for as long as the stale raster is up,
    /// and so does anything drawn *on top* of it there. Bounded to the object's
    /// own ink, transitional, and it ends when the raster lands.
    pub erase: Vec<PreviewShape>,
    /// Whether a cap above stopped this being the whole selection.
    ///
    /// Carried rather than dropped so the painter can decide what to do about
    /// it, and so a check can assert that a big selection produced a *bounded*
    /// preview rather than no preview.
    pub capped: bool,
}

impl ShapePreview {
    /// Whether there is anything to draw.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }

    /// How many segments this preview will paint — the cost, published.
    #[must_use]
    pub fn segment_count(&self) -> usize {
        self.shapes
            .iter()
            .map(|s| s.subpaths.iter().map(|p| p.segments.len()).sum::<usize>())
            .sum()
    }
}

/// **The selection's own geometry, transformed by a page-space matrix.**
#[must_use]
pub fn transformed(
    provider: &ObjectModelProvider,
    targets: &[TargetId],
    m: Matrix,
) -> Option<ShapePreview> {
    let mut out = ShapePreview::default();
    let mut segments = 0_usize;

    for &target in targets.iter().take(MAX_OBJECTS) {
        // Only paths. See the module header: a text run and an image carry no
        // geometry this shell may draw, and drawing them approximately would be
        // worse than leaving them to the outline.
        //
        let Some(VectorObject::Path(path)) = provider.object_for(target) else {
            continue;
        };
        let subpaths: Vec<Subpath> = path
            .page_subpaths()
            .into_iter()
            .map(|sp| sp.transformed(m))
            .collect();
        segments += subpaths.iter().map(|sp| sp.segments.len()).sum::<usize>();
        if segments > MAX_SEGMENTS {
            out.capped = true;
            break;
        }
        // The untransformed twin, for the erase pass. Built from the same
        // `page_subpaths()` walk rather than from a second read of the model:
        // two walks could disagree if the cache were invalidated between them,
        // and an erase that does not match the shape it is erasing leaves a
        // ghost of the object behind.
        out.erase.push(PreviewShape {
            subpaths: path.page_subpaths(),
            style: path.style,
            line_width: path.line_width,
        });
        out.shapes.push(PreviewShape {
            subpaths,
            style: path.style,
            // The width is scaled by the matrix, because a resize scales the
            // stroke. Approximated by the average of the two axis scales rather
            // than solved properly: a stroke under a non-uniform scale is an
            // ellipse-pen, PDF has no such thing, and the engine's own resize
            // does not produce one either. The number is used for ONE frame of
            // preview and the honest alternative — refusing to preview a
            // non-uniform resize — would remove the feature from the gesture
            // that needs it most.
            line_width: path.line_width * average_scale(m),
        });
    }
    if targets.len() > MAX_OBJECTS {
        out.capped = true;
    }
    trace(&out, targets.len());
    Some(out)
}

/// **One object's geometry with some of its anchors displaced.**
#[must_use]
pub fn with_nodes_moved(
    provider: &ObjectModelProvider,
    target: TargetId,
    nodes: &std::collections::BTreeSet<usize>,
    dx: f64,
    dy: f64,
) -> Option<ShapePreview> {
    // Either index space — see [`transformed`]'s own note. The anchor
    // numbering below is object-scoped and identical in both, because
    // `provider::geometry` computes it with the same running-offset walk.
    let VectorObject::Path(path) = provider.object_for(target)? else {
        return None;
    };

    let mut subpaths = path.page_subpaths();
    let mut scoped = 0_usize;
    for subpath in &mut subpaths {
        // Anchor `scoped` is this subpath's start; `scoped + 1 + i` is the end
        // of its `i`th segment. Same enumeration as `Subpath::anchors`.
        if nodes.contains(&scoped) {
            subpath.start = shift(subpath.start, dx, dy);
        }
        for (i, segment) in subpath.segments.iter_mut().enumerate() {
            if nodes.contains(&(scoped + 1 + i)) {
                *segment = shift_end(*segment, dx, dy);
            }
        }
        scoped += 1 + subpath.segments.len();
    }

    let capped = subpaths.iter().map(|sp| sp.segments.len()).sum::<usize>() > MAX_SEGMENTS;
    let out = if capped {
        ShapePreview {
            shapes: Vec::new(),
            erase: Vec::new(),
            capped: true,
        }
    } else {
        ShapePreview {
            capped: false,
            erase: vec![PreviewShape {
                subpaths: path.page_subpaths(),
                style: path.style,
                line_width: path.line_width,
            }],
            shapes: vec![PreviewShape {
                subpaths,
                style: path.style,
                line_width: path.line_width,
            }],
        }
    };
    trace(&out, 1);
    Some(out)
}

/// **The preview for whatever `moving::drag` decided this gesture is.**
#[must_use]
pub fn for_move_subject(
    provider: &ObjectModelProvider,
    subject: &crate::canvas::moving::MoveSubject,
    dx: f64,
    dy: f64,
) -> Option<ShapePreview> {
    use crate::canvas::moving::MoveSubject;
    match subject {
        // A whole-object move IS a translation, under both rungs. The two rungs
        // differ in which engine verb they are entitled to call, not in what the
        // operator sees, so they share a preview.
        MoveSubject::Transform { objects, .. } | MoveSubject::Objects { objects, .. } => {
            let targets: Vec<TargetId> = objects
                .iter()
                .map(|&i| TargetId::Object(i as u64))
                .collect();
            transformed(provider, &targets, Matrix::translate(dx, dy))
        }
        MoveSubject::LeavesInForm { leaves, .. } => {
            let targets: Vec<TargetId> = leaves.iter().map(|&i| TargetId::Leaf(i as u64)).collect();
            transformed(provider, &targets, Matrix::translate(dx, dy))
        }
        // A subpath move is every anchor in that subpath, and the anchor list
        // comes from the PROVIDER rather than from a walk here — it already
        // reports the object-scoped indices for one subpath, offsets included,
        // and re-deriving that arithmetic is the hazard `with_nodes_moved`'s
        // header describes.
        // **A text run gets the bounding ghost and NOTHING else, and that
        // is the honest preview rather than a gap.**
        //
        // `ShapePreview` is path geometry — anchors and segments moved by a
        // delta — and a show operator has none. There is nothing to draw
        // that would be more informative than the outline `MovePreview::ghost`
        // already carries, and drawing a rectangle here and calling it a shape
        // would be the same rectangle twice.
        //
        // On a rung the shape preview cannot serve — a line of text, an
        // image, a form XObject — the outline is the whole answer.
        //
        // The plural pair joins them for the same reason and gets the same
        // answer, which is better than it sounds: the ghost is a displacement
        // the canvas applies to the SELECTION OUTLINES, and a multi-chunk
        // selection already draws one box per chunk. So four selected chunks
        // preview as four boxes travelling together, which is what the gesture
        // is about to do.
        //
        // That last sentence is load-bearing on `overlay::draw_move_ghost`
        // actually drawing at this rung. It withholds the ghost only where the
        // real geometry is already travelling, which for a text line it never
        // is; a gate that withheld on the RUNG instead would leave every
        // sentence above describing feedback nobody receives.
        // `dragging_a_chunk_shows_where_it_is_going` is what keeps the two in
        // step.
        MoveSubject::TextLine { .. }
        | MoveSubject::TextLineInForm { .. }
        | MoveSubject::TextLines { .. }
        | MoveSubject::TextLinesInForm { .. } => None,
        MoveSubject::Subpath {
            object, subpath, ..
        } => {
            let target = TargetId::Object(*object as u64);
            let nodes: std::collections::BTreeSet<usize> = provider
                .subpath_node_points_of(target, *subpath)
                .into_iter()
                .map(|(index, _)| index)
                .collect();
            with_nodes_moved(provider, target, &nodes, dx, dy)
        }
        MoveSubject::SubpathInForm { leaf, subpath, .. } => {
            let target = TargetId::Leaf(*leaf as u64);
            let nodes: std::collections::BTreeSet<usize> = provider
                .subpath_node_points_of(target, *subpath)
                .into_iter()
                .map(|(index, _)| index)
                .collect();
            with_nodes_moved(provider, target, &nodes, dx, dy)
        }
        // THE ONE THE OPERATOR NAMED: *"if I moved the end of a line, it
        // didn't show me the shape change of the line."*
        MoveSubject::Node { object, node, .. } => {
            let mut only = std::collections::BTreeSet::new();
            only.insert(*node);
            with_nodes_moved(provider, TargetId::Object(*object as u64), &only, dx, dy)
        }
        MoveSubject::NodeInForm { leaf, node, .. } => {
            let mut only = std::collections::BTreeSet::new();
            only.insert(*node);
            with_nodes_moved(provider, TargetId::Leaf(*leaf as u64), &only, dx, dy)
        }
        MoveSubject::Nodes { object, nodes, .. } => with_nodes_moved(
            provider,
            TargetId::Object(*object as u64),
            &nodes.iter().copied().collect(),
            dx,
            dy,
        ),
        MoveSubject::NodesInForm { leaf, nodes, .. } => with_nodes_moved(
            provider,
            TargetId::Leaf(*leaf as u64),
            &nodes.iter().copied().collect(),
            dx,
            dy,
        ),
    }
}

/// **The footprint of objects that are about to stop existing.**
#[must_use]
pub fn erased(provider: &ObjectModelProvider, objects: &[usize]) -> Option<ShapePreview> {
    let model = provider.page_objects();
    let mut out = ShapePreview::default();
    let mut segments = 0_usize;
    for &index in objects.iter().take(MAX_OBJECTS) {
        let Some(VectorObject::Path(path)) = model.objects.get(index) else {
            continue;
        };
        let subpaths = path.page_subpaths();
        segments += subpaths.iter().map(|sp| sp.segments.len()).sum::<usize>();
        if segments > MAX_SEGMENTS {
            out.capped = true;
            break;
        }
        out.erase.push(PreviewShape {
            subpaths,
            style: path.style,
            line_width: path.line_width,
        });
    }
    if objects.len() > MAX_OBJECTS {
        out.capped = true;
    }
    trace(&out, objects.len());
    // `is_empty()` asks about `shapes`, which a delete never has — so the
    // emptiness test here is about the ERASE list, and using the wrong one would
    // discard every delete preview ever built.
    (!out.erase.is_empty()).then_some(out)
}

/// Move a point.
const fn shift(p: Point, dx: f64, dy: f64) -> Point {
    Point {
        x: p.x + dx,
        y: p.y + dy,
    }
}

/// Move a segment's **end anchor only**, leaving its control points alone.
///
/// See [`with_nodes_moved`]'s note: this mirrors `EditSession::move_node`, not
/// a smoothing rule of the shell's own invention.
const fn shift_end(segment: Segment, dx: f64, dy: f64) -> Segment {
    match segment {
        Segment::Line { to } => Segment::Line {
            to: shift(to, dx, dy),
        },
        Segment::Cubic { c1, c2, to } => Segment::Cubic {
            c1,
            c2,
            to: shift(to, dx, dy),
        },
    }
}

/// The mean of a matrix's two axis scales, for the stroke width.
fn average_scale(m: Matrix) -> f64 {
    let sx = m.a.hypot(m.b);
    let sy = m.c.hypot(m.d);
    let mean = (sx + sy) / 2.0;
    if mean.is_finite() && mean > 0.0 {
        mean
    } else {
        1.0
    }
}

/// Publish what the preview cost and whether it was capped.
fn trace(preview: &ShapePreview, asked: usize) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "canvas-shape-preview asked={asked} shapes={} segments={} capped={}",
            preview.shapes.len(),
            preview.segment_count(),
            u8::from(preview.capped)
        )
    });
}

/// **How wide a preview stroke is drawn on glass** - `OPERATOR_REQUESTS.md`
/// **O184**, and the one place in this module where the answer is NOT "whatever
/// the document says".
#[derive(Clone, Copy, Debug)]
pub struct StrokeRule {
    /// Device pixels per page point, at the zoom currently on screen.
    ///
    /// Derived by mapping a unit page vector rather than read off the mapping's
    /// private zoom field - `coords`' standing rule is that a coordinate is
    /// produced by exactly one conversion in exactly one place, and a length is
    /// a coordinate.
    pub zoom: f32,
    /// `view.line_weights` - **true** when strokes are drawn at the widths the
    /// file states, **false** when every stroke is one device pixel (O137).
    pub real_widths: bool,
}

impl StrokeRule {
    /// The width, in device pixels, of the **preview** outline for a stroke the
    /// file states as `line_width` points.
    pub fn preview_px(self, line_width: f64) -> f32 {
        if !self.real_widths {
            // One device pixel, because that is what the renderer drew.
            return 1.0;
        }
        (line_width as f32).max(1.0)
    }

    /// The width, in device pixels, of the **erase** band that covers the same
    /// stroke's footprint on the raster underneath.
    pub fn erase_px(self, line_width: f64) -> f32 {
        let ink = if self.real_widths {
            (line_width as f32) * self.zoom
        } else {
            // The hairline view put one device pixel on the texture whatever
            // the file said, so that - and not the file's width - is what has
            // to be covered.
            1.0
        };
        ink.max(1.0) + 1.5
    }
}

/// **Paint the preview**, in the selection stroke, over the page.
pub fn draw(
    painter: &Painter,
    preview: &ShapePreview,
    page: &pdfcer_core::page_tree::Page,
    map: &PageMapping,
    colour: egui::Color32,
    rule: StrokeRule,
) {
    // The census: what actually reached the PAINTER.
    //
    // Distinct from `canvas-shape-preview`, which says what was BUILT, and the
    // distinction is the whole reason there are two lines. A preview that is
    // built and never painted — a `None` on the way through `interact`, a
    // painter arm never reached, a page index that does not match — looks
    // exactly like a preview that was never built, to anything reading one
    // trace. Two lines make "built but not drawn" a state a check can name.
    //
    // Written only when there is something to draw, so it costs nothing on
    // the frames nobody is dragging — which is almost all of them.
    if !preview.shapes.is_empty() || !preview.erase.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "canvas-shape-drawn shapes={} segments={} erased={} zoom={:.3} real_widths={} widest_px={:.2}",
                preview.shapes.len(),
                preview.segment_count(),
                preview.erase.len(),
                rule.zoom,
                // Printed as `true`/`false` rather than 1/0 so it reads the same
                // way as `view-chrome LineWeights on=`, which is the other place
                // this one field is published. Two spellings of one fact is one
                // more thing a reader has to hold.
                rule.real_widths,
                // O184 - the widest preview stroke this frame, IN THE
                // TRACE, because zoom-invariance is a claim about two frames at
                // two zooms and no screenshot of one frame can carry it. A
                // driven check reads this line at 100 % and again at 800 % and
                // asserts the number did not move.
                preview
                    .shapes
                    .iter()
                    .map(|s| rule.preview_px(s.line_width))
                    .fold(0.0_f32, f32::max)
            )
        });
    }
    // THE ERASE PASS — the object's old footprint, in paper.
    //
    // See [`ShapePreview::erase`] for why this is necessary and what it costs.
    // In short: the raster underneath still shows the object where it was and
    // cannot be redrawn inside a second, so without this the operator sees the
    // thing twice.
    //
    // Sized by [`StrokeRule::erase_px`], which is the ZOOM-SCALED one of the
    // pair: this band covers ink a renderer actually put on the texture, and
    // that ink scaled with the zoom. See the rule's header for why the erase
    // and the preview deliberately answer to different spaces.
    for shape in &preview.erase {
        stroke_shape(
            painter,
            shape,
            page,
            map,
            Stroke::new(rule.erase_px(shape.line_width), paper()),
        );
    }
    for shape in &preview.shapes {
        // Sized by [`StrokeRule::preview_px`], which zoom does not enter -
        // O184. The preview is the cursor, and a cursor does not grow when the
        // document is magnified.
        stroke_shape(
            painter,
            shape,
            page,
            map,
            Stroke::new(rule.preview_px(shape.line_width), colour),
        );
    }
}

/// **The colour the renderer's own page backdrop is.**
pub(super) const fn paper() -> egui::Color32 {
    // DOCUMENT COLOUR: this is the renderer's own page backdrop, not chrome.
    // §11.4.7 composites a page group onto an opaque white backdrop when the
    // document does not say otherwise, and `render_page` produced the raster
    // this is painted over using exactly that value. A theme must never move it:
    // restyling the application would change what an erased footprint looks
    // like against a raster the theme has no say in, and the two would stop
    // matching.
    egui::Color32::WHITE
}

/// Walk one shape's subpaths and stroke them.
fn stroke_shape(
    painter: &Painter,
    shape: &PreviewShape,
    page: &pdfcer_core::page_tree::Page,
    map: &PageMapping,
    stroke: Stroke,
) {
    {
        for subpath in &shape.subpaths {
            let Some(start) = screen(subpath.start, page, map) else {
                continue;
            };
            let mut cursor = start;
            for segment in &subpath.segments {
                match *segment {
                    Segment::Line { to } => {
                        if let Some(end) = screen(to, page, map) {
                            painter.line_segment([cursor, end], stroke);
                            cursor = end;
                        }
                    }
                    Segment::Cubic { c1, c2, to } => {
                        let (Some(p1), Some(p2), Some(end)) = (
                            screen(c1, page, map),
                            screen(c2, page, map),
                            screen(to, page, map),
                        ) else {
                            continue;
                        };
                        painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
                            [cursor, p1, p2, end],
                            false,
                            egui::Color32::TRANSPARENT,
                            stroke,
                        ));
                        cursor = end;
                    }
                }
            }
            if subpath.closed && cursor != start {
                painter.line_segment([cursor, start], stroke);
            }
        }
    }
}

/// Page space → screen, through the one function entitled to do it.
fn screen(p: Point, page: &pdfcer_core::page_tree::Page, map: &PageMapping) -> Option<Pos2> {
    crate::canvas::measure::page_to_screen(p, page, map)
}

#[cfg(test)]
mod tests;
