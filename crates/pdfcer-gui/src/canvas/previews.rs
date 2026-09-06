//! # `canvas::previews` — **the fourteen things one frame might be showing you**
//!
//! Every *pre-commit affordance* the canvas can draw, in one place, each with
//! the argument for why it is its own value rather than a variant of another.
//!
//! ## Why this is a module and not a paragraph in `interact`
//!
//! It was thirteen `let mut` bindings at the top of `canvas::interact::interact`
//! carrying seventy lines of doc comment between them, and it pushed that file
//! past **R2**'s 1,500-line ceiling on 2026-08-30 when O63 added a fourteenth.
//!
//! The gate's own message says *"split the module along its seams — one subject
//! per file — rather than raising the limit"*, and this is a seam rather than a
//! convenient cut: `interact` answers *"what does this frame's pointer mean?"*
//! and these answer *"what is drawn while the answer is still provisional?"*.
//! The painter reads them; `interact` only fills them in.
//!
//! ## ★★★ The one argument every field here shares, stated once
//!
//! **They are separate values and not one `enum`.** The reasoning was repeated
//! nine times in the source this was extracted from, and it is worth keeping
//! once rather than nine times:
//!
//! > The painter reads each independently. Folding them together would put a
//! > branch inside the paint loop for a value that is `None` on every frame
//! > nobody is dragging — and it would make *"which kind of thing is this
//! > rectangle about?"* answerable only by looking at the selection.
//!
//! Several really are the same Rust type — [`Slots::marquee`],
//! [`Slots::annot_ghost`] and [`Slots::zoom_region`] are all `Option<Rect>` —
//! and they are still separate, deliberately. **Two names is one fewer place to
//! be wrong.**
//!
//! ## Rule 4, which every field here is on the right side of
//!
//! All of this is **the cursor**, not the document. A pre-commit affordance —
//! a rubber band, a snap marker, a ghost, the O63 shape preview — is explicitly
//! permitted; what is forbidden is styling content *already applied* as though
//! it were provisional. Every value here describes something that has not
//! happened yet and disappears when it does.

use pdfcer_core::vector::{Handle, Point, snap::SnapCandidate};

use crate::canvas::handles::Grip;
use crate::canvas::shapes::ShapePreview;

/// Everything one frame of the canvas might be drawing that is not the document.
///
/// Built empty at the top of `interact`, filled in by whichever gesture arm
/// runs, and read by `canvas::painting::draw`.
#[derive(Default)]
pub struct Slots {
    /// The rubber-band selection rectangle, in canvas space.
    pub marquee: Option<egui::Rect>,
    /// A content move's canvas-space displacement — the **bounding** ghost.
    ///
    /// This is the *selection* indicator: it says which thing is moving. What
    /// it will look like is [`Self::shape`]'s job.
    pub ghost: Option<egui::Vec2>,
    /// ★★★ **The selection's own geometry at its new position**, in page space
    /// (`OPERATOR_REQUESTS.md` O63).
    ///
    /// **Ken, 2026-08-30:** *"if I moved the end of a line, it didn't show me
    /// the shape change of the line, it just had a perimeter box around it …
    /// there isn't a real preview like there is in inkscape."*
    ///
    /// Beside [`Self::ghost`] rather than replacing it, and the two are `Some`
    /// together on most rungs. They answer different questions — *where is it
    /// going* against *what will it look like* — and this one is `None` on every
    /// rung the shell cannot draw honestly (a text run, an image, a form
    /// XObject, a page that will not decompose, a selection past the cap) while
    /// the other is not. On those rungs the outline alone is the whole answer,
    /// exactly as it was before this field existed.
    pub shape: Option<ShapePreview>,
    /// A resize in flight: the grip being dragged and its two scale factors.
    ///
    /// A move ghost is one displacement and a resize is a grip plus two factors.
    pub resize_ghost: Option<(Grip, (f32, f32))>,
    /// The angle a rotate drag has turned through, in **screen** space.
    pub rotate_ghost: Option<f32>,
    /// The Bézier handle being dragged: its anchor, its side, and where it now
    /// sits in canvas space.
    pub handle: Option<(usize, Handle, egui::Pos2)>,
    /// A ce dimension being dragged to a new placement, as the **page-space**
    /// segments it would be drawn as on release.
    ///
    /// ★ Not an outline of an existing shape at all — it is the dimension
    /// redrawn from its own geometry, because moving a dimension line *stretches
    /// its extension lines* rather than translating a box. A ghost offset by a
    /// delta would draw the wrong picture entirely.
    pub dimension: Option<Vec<(Point, Point)>>,
    /// ★★★ **A markup shape redrawn from its nodes' new positions**, as
    /// page-space segments — `Pass 255.0`, and the operator's *"I also can't
    /// edit or delete nodes of a markup shape once it is drawn."*
    ///
    /// Beside [`Self::dimension`] rather than sharing it, even though the two
    /// are the same shape of value and can never both be `Some` on one frame.
    /// They describe two different subjects reaching two different engine verb
    /// families — R8b rule 15's distinction between a **ce dimension** and a
    /// comment shape — and one `Vec` whose meaning depends on which selection
    /// is live is a value the paint loop has to interrogate. `Self::annot_ghost`
    /// makes the identical choice against `Self::ghost` for the identical
    /// reason.
    ///
    /// ★ Not a ghosted outline: moving one node **stretches two segments**, so
    /// a bounding box translated by a delta would draw a picture the release
    /// does not commit.
    pub markup_nodes: Option<Vec<(Point, Point)>>,
    /// What a dragged node is snapping to — a ce dimension's corner or a markup
    /// shape's node.
    ///
    /// ★ Separate from [`Self::dimension`] for the reason `dimdrag::VertexDrag`
    /// gives: the polyline is page-space geometry and this is one screen-space
    /// glyph, drawn by a different painter at a different moment.
    pub vertex_snap: Option<SnapCandidate>,
    /// A markup annotation being dragged, as the **canvas-space** rectangle it
    /// would occupy on release.
    ///
    /// ★★ Not [`Self::ghost`], even though it is the same shape and the two can
    /// never both be `Some` on one frame. Sharing would work and would make the
    /// painter's question *"which kind of thing is this rectangle about?"*
    /// answerable only by looking at the selection.
    pub annot_ghost: Option<egui::Rect>,
    /// The quads a text-following highlight would cover, in canvas space.
    ///
    /// ★ A **list** of rectangles — one per line the drag crosses — where a band
    /// is two points. The painter draws them with the same wash the band uses so
    /// the two gestures look like one feature.
    pub text_marks: Option<Vec<egui::Rect>>,
    /// A markup band being drawn — its shape kind and its two canvas-space
    /// points, as `markup::band` reports them.
    pub band: Option<crate::canvas::markup::band::Preview>,
    /// The freehand trail, already simplified, in canvas space.
    ///
    /// ★ Beside [`Self::band`] rather than a variant of it: a band is two points
    /// and a shape rule, a trail is a polyline of however many points survived
    /// `markup::ink::simplify`. Folding them would put a `Vec` in a value the
    /// band path copies per frame for no benefit.
    pub ink_trail: Option<Vec<egui::Pos2>>,
    /// The zoom-to-region rectangle being dragged, in canvas space.
    pub zoom_region: Option<egui::Rect>,
}
