//! # `canvas::shapes` — **the shape itself, following your hand**
//!
//! The live geometry preview. `OPERATOR_REQUESTS.md` **O63**.
//!
//! ## What this replaces, and the convention it overrules
//!
//! **Ken, 2026-08-30:** *"if I moved the end of a line, it didn't show me the
//! shape change of the line, it just had a perimeter box around it. this goes
//! for anything I change right now. there isn't a real preview like there is in
//! inkscape."*
//!
//! He is right, and it was **deliberate**. `canvas/handledrag.rs` states the
//! rule this module exists to reverse:
//!
//! > *"a preview shows the cursor, the render shows the document."*
//!
//! That was a defensible position while the alternative looked like a second
//! rendering path. ★★★ **It is overruled by operator ruling, by name, against a
//! named comparison** — Inkscape shows the line bend while you drag its end, and
//! so must this. Recorded as *reversed* rather than quietly contradicted,
//! because the sentence is repeated across several modules and the next session
//! would otherwise re-derive it and delete this file.
//!
//! ## ★★ Why this is cheap, when the rest of O63 is not
//!
//! The measurement that framed O63 says a **rasterised** preview is impossible:
//! on the operator's CAD drawing a *two-pixel* render costs 691 ms, because ~99 %
//! of render cost is content-stream interpretation rather than fill. Anything
//! that goes through `pdfcer-render` is a second away.
//!
//! **This does not go through `pdfcer-render`.** `vector::decompose_page` has
//! already produced the real geometry — `PathObject::page_subpaths()` gives
//! page-space `Line` and `Cubic` segments with control points resolved, plus the
//! paint style, the line width and both colours — and the shell already caches
//! it (`app::cache::page_objects`, keyed on `(page, edit_epoch)`).
//!
//! ⇒ Transform that in memory and hand it to egui's painter. No engine call, no
//! raster, no decomposition. **Pointer speed, and exact for geometry** — this is
//! not the "fuzzy" half of O63 at all.
//!
//! ## Rule 4, which this is on the right side of
//!
//! A pre-commit affordance is *the cursor*, and the cursor is explicitly
//! permitted: snap indicators, rubber bands and selection handles are all
//! welcome. What is forbidden is styling **applied** content as though it were
//! provisional.
//!
//! This draws a shape that has not been applied yet, in the selection stroke, and
//! it disappears the moment the real one is rendered. Nothing already in the
//! document is marked, tinted, badged or outlined because of it.
//!
//! ★ And it is **derived from the commit**, which is this canvas's standing
//! convention D2: the transform painted here is the *same* transform the release
//! hands to `EditSession`, so the operator cannot be shown one shape and given
//! another.
//!
//! ## What it deliberately does not draw
//!
//! **Text, images and form XObjects.** A `PathObject` carries its own geometry;
//! a text run carries glyph provenance and an image carries a bounding box, and
//! neither can be drawn by this shell without becoming a second renderer. Those
//! keep the bounding outline they have today — which is honest, because a
//! rectangle *is* all the shell knows about where an image is going.
//!
//! ⇒ So the preview is **exact where it exists and absent where it does not**,
//! rather than approximate everywhere. A half-right glyph is worse than no glyph.

use egui::{Painter, Pos2, Stroke};
use pdfcer_core::vector::{Matrix, PaintStyle, Point, Segment, Subpath, VectorObject};

use crate::canvas::mapping::PageMapping;
use crate::panels::objects::provider::{ObjectModelProvider, TargetId};

/// How many objects a preview will carry before it gives up.
///
/// # ★★ Why there is a cap at all, and why it is disclosed rather than silent
///
/// A marquee across a CAD sheet can select thousands of paths, each with
/// thousands of segments. Painting all of them every frame would turn the
/// gesture this feature exists to make smooth into the slowest thing in the
/// program — the exact inversion of the point.
///
/// Past the cap the preview is **absent**, and the bounding outline that was
/// there before this module existed is what the operator sees. That is a
/// graceful floor rather than a failure: it is what the shell did yesterday.
///
/// ★ `canvas-shape-preview` traces `capped=1` when it fires, because an absence
/// with no account of itself is indistinguishable from a defect — the lesson
/// `painting.rs`'s anchor census already carries.
const MAX_OBJECTS: usize = 64;

/// How many segments across the whole preview before it gives up.
///
/// The second half of the same guard, and the one that actually fires on this
/// operator's drawings: `SW41177.pdf` carries a single object with **4,972
/// anchors**. One object is under [`MAX_OBJECTS`] and would still cost five
/// thousand line segments a frame.
const MAX_SEGMENTS: usize = 8_000;

/// One object's geometry, in **page space**, ready to be mapped and painted.
///
/// A copy rather than a borrow, deliberately: the provider is behind a `Ref`
/// that must be dropped before anything is painted (`painting.rs` says so at its
/// anchor draw), and a preview that borrowed it would hold that `Ref` across the
/// paint.
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
    /// # ★★★ Why an erase list exists at all
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
    /// # ★★ The footprint, not the bounding box — and that is the whole
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
    /// ★ Carried rather than dropped so the painter can decide what to do about
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
///
/// The builder for move, resize and rotate — the three gestures whose whole
/// effect *is* a matrix, and which reach `EditSession::transform_objects` or
/// `move_objects` with exactly this transform.
///
/// # ★★★ The matrix is the COMMIT's matrix, not a second one
///
/// Convention D2 on this canvas: the preview is derived from the value the
/// release will hand to the engine. `moving::action`, `resizing::action` and
/// `rotating`'s `TransformObjects` all build a page-space `Matrix`; this takes
/// that same value. Building a *parallel* transform here — "translate by the
/// canvas delta, converted again" — is how a preview and a commit come to
/// disagree by a rounding step, and the disagreement is invisible until an
/// operator lines something up against a guide.
///
/// # Returns
///
/// `None` when there is no decomposition this frame — a document still loading,
/// or a page that would not decompose. **Not an error**: the caller falls back
/// to the bounding outline, which is what it drew before this module existed.
#[must_use]
pub fn transformed(
    provider: &ObjectModelProvider,
    targets: &[TargetId],
    m: Matrix,
) -> Option<ShapePreview> {
    let mut out = ShapePreview::default();
    let mut segments = 0_usize;

    for &target in targets.iter().take(MAX_OBJECTS) {
        // ★ Only paths. See the module header: a text run and an image carry no
        // geometry this shell may draw, and drawing them approximately would be
        // worse than leaving them to the outline.
        //
        // ★★ `object_for` rather than `model.objects.get`, since 2026-09-01
        // (`OPERATOR_REQUESTS.md` O70): it resolves either index space, so a
        // drag inside a form XObject previews its geometry instead of falling
        // back to the outline ghost. That fallback was correct while a leaf
        // could not be edited at all and became a visible gap the moment it
        // could — the operator drags a line inside a title block and watches a
        // box move.
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
        // ★ The untransformed twin, for the erase pass. Built from the same
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
            // ★★ The width is scaled by the matrix, because a resize scales the
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
///
/// The builder for the gesture the operator actually named: *"if I moved the end
/// of a line, it didn't show me the shape change of the line"*.
///
/// # ★★★ The anchor indices are OBJECT-SCOPED, and the walk order is the
/// provider's, not a second one
///
/// `move_node` / `move_nodes` address an anchor by an index that counts across
/// the whole object, flattening its subpaths — the space
/// `ObjectModelProvider::object_node_points` reports and `pdfcer node-move`
/// speaks.
///
/// This walks the same order: `page_subpaths()`, then within a subpath the
/// `start` anchor followed by each segment's end, which is exactly
/// `Subpath::anchors()`. **That agreement is asserted by a test rather than
/// assumed** — see `the_walk_agrees_with_the_provider`. R74's rule is that a
/// matching rule must not be re-derived in the shell, and index arithmetic that
/// *must* match another module's is the same hazard wearing a smaller hat.
///
/// # ★★ Handles move with their anchor, and that is a choice
///
/// Displacing an on-curve anchor leaves its two control points where they were,
/// which is Inkscape's *"corner"* behaviour and produces a visibly different
/// curve from Inkscape's default *"smooth"* drag. **This preview does whatever
/// `EditSession::move_node` does**, and what it does is move the anchor alone —
/// so the preview is right about pdfcer even where it would be wrong about
/// Inkscape. Getting this backwards would produce the one failure this whole
/// module must not have: a preview that is prettier than the commit.
#[must_use]
pub fn with_nodes_moved(
    provider: &ObjectModelProvider,
    target: TargetId,
    nodes: &std::collections::BTreeSet<usize>,
    dx: f64,
    dy: f64,
) -> Option<ShapePreview> {
    // ★ Either index space — see [`transformed`]'s own note. The anchor
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
///
/// One function, so that the mapping from *the verb the release will call* to
/// *the shape the operator sees* lives in one readable table and a sixth rung
/// cannot be added without appearing in it.
///
/// ★★★ It takes the [`MoveSubject`] the commit will use, not the selection.
/// That is convention D2 — *derived from commit* — enforced by the type rather
/// than by discipline: there is no way to reach this function without having
/// already computed what the release is going to do, so a preview cannot be
/// drawn for a gesture that would then refuse.
///
/// `dx`/`dy` are **PDF user-space** and Y is up, exactly as
/// [`crate::canvas::moving::PageDelta`] carries them.
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
        // ★★★ **And the same preview for a drag inside a container** — O70,
        // 2026-09-01. One line, because `transformed` stopped caring which
        // list an object came from.
        MoveSubject::LeavesInForm { leaves, .. } => {
            let targets: Vec<TargetId> = leaves.iter().map(|&i| TargetId::Leaf(i as u64)).collect();
            transformed(provider, &targets, Matrix::translate(dx, dy))
        }
        // ★ A subpath move is every anchor in that subpath, and the anchor list
        // comes from the PROVIDER rather than from a walk here — it already
        // reports the object-scoped indices for one subpath, offsets included,
        // and re-deriving that arithmetic is the hazard `with_nodes_moved`'s
        // header describes.
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
        // ★★★ THE ONE THE OPERATOR NAMED: *"if I moved the end of a line, it
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
///
/// The builder for a delete: an erase list and **no** shapes, so the preview is
/// pure subtraction.
///
/// # ★★★ Why a delete needs a preview at all, when nothing is being drawn
///
/// Because the raster underneath does not know. The operator presses Delete,
/// the object is gone from the document — and it stays on screen for one to two
/// seconds on a dense drawing, because that is how long the page takes to
/// redraw. There is no gesture in flight to explain the wait, so what they see
/// is *a delete that did nothing*, and the natural response is to press Delete
/// again, which deletes something else.
///
/// ⇒ This makes the object disappear at the moment it is deleted, which is what
/// every operator on earth expects and what the program was already doing to
/// the document. The picture simply catches up with it.
///
/// # ★★ It must be built BEFORE the commit
///
/// `app::cache::page_objects` is keyed on `(page, edit_epoch)` and the commit
/// bumps the epoch, so the geometry this needs is thrown away by the very edit
/// it describes. Called from the apply arm with the pre-edit model still in
/// hand; a caller that reached for it afterwards would find the objects gone and
/// silently hold nothing, which is the old behaviour wearing a new name.
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
    // ★ `is_empty()` asks about `shapes`, which a delete never has — so the
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
///
/// `hypot` per axis rather than reading `a` and `d`, because a rotation puts
/// scale into `b` and `c` and reading the diagonal alone would report a rotated
/// object as having shrunk to zero width at 90°.
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
///
/// ★ Written on **every** build, including the empty one. An absent preview and
/// a preview nobody asked for are different states and a trace that only spoke
/// when there was something to say could not tell them apart — the lesson
/// `painting.rs`'s anchor census carries, applied before it can bite here.
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

/// **Paint the preview**, in the selection stroke, over the page.
///
/// # ★★ Stroke only, never fill — and this is the one place the preview is
/// deliberately *less* than the truth
///
/// A filled shape following the pointer would hide what is under it, and what is
/// under it is the page the operator is aligning against. Every drawing program
/// that previews a transform previews it as an outline for exactly that reason.
///
/// ⇒ So a filled path previews as its **boundary**. That is fuzzy in the
/// permitted sense — less than the truth, never different in meaning — and it is
/// the same compromise `backdrop.rs` documents for the low-resolution page.
///
/// # Why the selection colour and not the object's own
///
/// Because it is the *selection* moving. Painting a shape in its own stroke
/// colour would make the preview indistinguishable from committed content, and
/// there would then be two of it on screen — the stale raster still holds the
/// object where it was. One of the two has to be legible as "this is the one
/// following your hand".
pub fn draw(
    painter: &Painter,
    preview: &ShapePreview,
    page: &pdfcer_core::page_tree::Page,
    map: &PageMapping,
    colour: egui::Color32,
    scale: f32,
) {
    // ★★ The census: what actually reached the PAINTER.
    //
    // Distinct from `canvas-shape-preview`, which says what was BUILT, and the
    // distinction is the whole reason there are two lines. A preview that is
    // built and never painted — a `None` on the way through `interact`, a
    // painter arm never reached, a page index that does not match — looks
    // exactly like a preview that was never built, to anything reading one
    // trace. Two lines make "built but not drawn" a state a check can name.
    //
    // ★ Written only when there is something to draw, so it costs nothing on
    // the frames nobody is dragging — which is almost all of them.
    if !preview.shapes.is_empty() || !preview.erase.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "canvas-shape-drawn shapes={} segments={} erased={}",
                preview.shapes.len(),
                preview.segment_count(),
                preview.erase.len()
            )
        });
    }
    // ★★★ THE ERASE PASS — the object's old footprint, in paper.
    //
    // See [`ShapePreview::erase`] for why this is necessary and what it costs.
    // In short: the raster underneath still shows the object where it was and
    // cannot be redrawn inside a second, so without this the operator sees the
    // thing twice.
    //
    // ★★ Painted **1.5 points wider** than the object's own stroke. An erase
    // exactly as wide as the line leaves a hairline of the original visible
    // down both sides, because the raster's antialiasing spread the ink half a
    // pixel further than the geometry says. A visible outline of where the
    // object *used* to be is the one outcome worse than not erasing at all — it
    // reads as a rendering artefact rather than as a preview.
    for shape in &preview.erase {
        let width = ((shape.line_width as f32) * scale).max(1.0) + 1.5;
        stroke_shape(painter, shape, page, map, Stroke::new(width, paper()));
    }
    for shape in &preview.shapes {
        // ★ A minimum of one logical point. A hairline (`0 w`, §8.4.3.2) is one
        // *device* pixel and would vanish under egui's antialiasing; and a
        // preview nobody can see is the same as no preview.
        let width = ((shape.line_width as f32) * scale).max(1.0);
        stroke_shape(painter, shape, page, map, Stroke::new(width, colour));
    }
}

/// **The colour an erased footprint is painted in.**
///
/// # ★★ Why this is a constant and not read from the document
///
/// PDF has no page-background colour. A page is whatever its content paints,
/// and the overwhelming majority of pages paint nothing at all outside their
/// ink — which a viewer composites over **white**, because that is what
/// `render_page`'s own backdrop is (§11.4.7's page group is composited onto an
/// opaque white backdrop when a document does not say otherwise).
///
/// ⇒ So white is not a guess about this document, it is the same value the
/// renderer used to produce the raster this is painted over. Reading a colour
/// out of the texture at that point would be a *measurement* rather than an
/// assumption, and it is a genuine improvement worth making if a coloured
/// drawing ever makes this look wrong — recorded here rather than done now
/// because no such drawing has been seen.
const fn paper() -> egui::Color32 {
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
///
/// Shared by the erase pass and the preview pass so the two cannot disagree
/// about what the shape's outline *is*: an erase that traced a different path
/// from the preview would leave part of the original showing, and the part left
/// showing would look like the program had drawn it on purpose.
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
///
/// ★ `measure::page_to_screen`, not arithmetic here. `coords`' standing rule is
/// that a coordinate is produced by exactly one conversion in exactly one place,
/// and the ce-dimension placement preview already goes through this door.
fn screen(p: Point, page: &pdfcer_core::page_tree::Page, map: &PageMapping) -> Option<Pos2> {
    crate::canvas::measure::page_to_screen(p, page, map)
}

#[cfg(test)]
mod tests;
