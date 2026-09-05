//! # `canvas::selection::annot` — clicking the things pdfcer itself put on the page
//!
//! ## ★ The gap this closes, and how long it was open
//!
//! `FEATURES.md` recorded it on 2026-08-17, under the Format contextual tab:
//!
//! > *"**The canvas selection cannot address an annotation** — `Selection` is
//! > `page + object + subpath + node`, four integers naming a paint-order
//! > index into page *content*, which is what makes it immune to zoom and also
//! > means a markup or dimension **is not selectable at all**. The second is
//! > ours; the first is filed."*
//!
//! Both halves of that are now discharged. The engine's half — no verb that
//! modifies an annotation — cleared on 2026-08-18 with `set_markup_style`.
//! This is ours.
//!
//! The operator's report is what it cost: *"How do I edit a stamp I've
//! applied?"*, *"I still can't get to edit dimension groups when I click on
//! it"*, and *"it feels like nothing is moving forward on these things"* —
//! three symptoms of one missing capability. A stamp placed in the wrong spot
//! could not be moved, restyled, or even **deleted**, except by `Ctrl+Z`
//! immediately afterwards.
//!
//! ## Why this is a sibling of [`super::Selection`] and not a variant of it
//!
//! They look similar and are structurally different in four ways, every one of
//! which would have to be special-cased if they shared a type:
//!
//! | | page content | annotation |
//! |---|---|---|
//! | identity | a **paint-order index** — position in `PageObjects::objects` | an **`ObjId`**, stable across edits and across saves |
//! | arity | multi-select, built up over several clicks | one at a time |
//! | structure | a ladder — object ▸ subpath ▸ node, because one CAD path can hold 1,194 subpaths | flat; an annotation has no parts in pdfcer's model |
//! | geometry | needs `decompose_page`, which resolves and walks every content stream | `/Rect`, read straight off the dictionary |
//!
//! The last row is why annotation selection needs no cache and no
//! `resolved_for` epoch key: the rectangle is four numbers in the annotation
//! dictionary, and asking for it costs a dictionary lookup rather than a
//! content-stream walk.
//!
//! **They are still mutually exclusive**, and [`super::SelectionState`]
//! enforces that in one place rather than by convention — see its `annot`
//! field. One canvas, one selection; `panels::ObjectTreeUi::focus`' refusal of
//! *"a second selection"* stands.
//!
//! ## ★ Why the KIND is in the type
//!
//! [`AnnotKind`] distinguishes a **ce dimension** from ordinary markup, and it
//! is carried on the target rather than re-derived where it is needed. That is
//! not tidiness — it is the shell's half of a refusal the engine makes by
//! name.
//!
//! A ce dimension is a `/Line` annotation with `/IT /LineDimension`. It passes
//! every *"is this markup pdfcer can author?"* test, and restyling one through
//! `set_markup_style` would regenerate its appearance as a **bare line, with
//! its label and witness lines gone** — from an operator who asked only to
//! recolour it. `pdfcer-core` refuses it by name
//! (`EditError::AnnotationIsCeDimension`) and points at `set_dimension_style`,
//! and the reply that shipped the verb said so in as many words: *"Your Format
//! tab must route ce dimensions there."*
//!
//! Carrying the kind on the target makes that routing a `match` the compiler
//! checks, rather than a condition somebody has to remember at each of the
//! places a style is applied. The engine's refusal stays as the backstop; this
//! is what stops it being reached.
//!
//! ## Rule 4: this draws nothing on the page that a save would not
//!
//! A selection outline is **the cursor**, which the rule permits by name — the
//! same class as a snap indicator, a rubber band or a resize handle, and the
//! same treatment content selection already gets. Nothing here tints, badges
//! or flags an annotation, and the one-line test still passes: a screenshot of
//! the canvas with a stamp selected differs from a screenshot of the saved
//! file only by the marching outline, which is where the pointer is and not
//! what the document says.
//!
//! ## conventions: click-selects
//!
//! Corpus: `ui-conventions/click-selects.md`.
//!
//! - C1 shape-not-box: a candidate may carry its drawn segments, and where it
//!   does they are what is tested. Added 2026-08-20 on the operator's report;
//!   see [`hit`]'s header for the whole argument.
//! - C2 unfilled-interior: **GAP** — only ce dimensions supply a shape today. A
//!   `/Square` with no `/IC` still claims its interior, so a large empty callout
//!   box remains un-clickable-through. The mechanism to fix it is already here:
//!   give that subtype a shape.
//! - C3 topmost-wins: `.rev()` over `/Annots`, which is paint order.
//! - C4 tolerance: none for a rect — the engine bakes the pen half-width into
//!   `/Rect` at authoring time, so a second one would double-count — and the
//!   canvas click tolerance for a segment, which has no width at all. Both
//!   stated at the call site.
//! - C5 segment-not-line: `distance_to_segment` clamps to the ends. Without it a
//!   short dimension line would claim a stripe across the sheet.
//! - C6 miss-deselects: owned by `canvas::interact`, which clears the annotation
//!   selection when a click in a mode that could have hit one did not.
//! - C7 drawn-equals-live: the ink is the target and the `/Rect` is the outline
//!   drawn AFTER selection, which is a different thing from a hover affordance —
//!   nothing here is painted as targetable that is not. If annotation hover
//!   highlighting is ever added it must highlight the shape, not the box.
//! - C8 stated-precedence: `gesture::press_kind` holds the whole order in one
//!   place, and an annotation click sits below every armed tool by construction.

use std::collections::{BTreeMap, BTreeSet};

use egui::{Pos2, Rect};
use pdfcer_core::annot::page_annotations;
use pdfcer_core::graph::ObjectGraph;
use pdfcer_core::object::ObjId;
use pdfcer_core::page_tree::Page;

use crate::canvas::mapping::annot_canvas_rect;

/// Which family an annotation belongs to, and therefore **which verb may
/// restyle it**.
///
/// Two variants, and the distinction is load-bearing rather than descriptive —
/// see the module header. Deliberately not `is_ce_dimension: bool` on a struct:
/// a bool is a fact a caller may forget to read, while a variant is one the
/// compiler makes them handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnnotKind {
    /// Ordinary markup — a shape, a note, a stamp, a text markup.
    /// `EditSession::set_markup_style` is its verb.
    Markup,
    /// A **ce dimension**: a `/Line` carrying `/IT /LineDimension` and a record
    /// in the document's `/PieceInfo` sidecar.
    ///
    /// `set_dimension_style` is its verb. Handing one to `set_markup_style`
    /// regenerates it as a bare line and loses its label and witness lines,
    /// which is why the engine refuses that by name.
    CeDimension,
}

/// One annotation, addressed the way the engine addresses it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotTarget {
    /// The page it lives on.
    ///
    /// Carried for the same reason [`super::Selection::page`] is: it lets a
    /// selection survive navigating away and back, and Phase 4 puts several
    /// pages on screen at once.
    pub page: usize,
    /// The annotation's object id — **stable**, unlike a content object's
    /// paint-order index.
    ///
    /// This is what every `EditSession` annotation verb takes, so a selection
    /// made here can be acted on without a second lookup that could resolve
    /// differently.
    pub id: ObjId,
    /// Which verb may restyle it. See [`AnnotKind`].
    pub kind: AnnotKind,
    /// `/Subtype`, as the file spells it — `Stamp`, `Square`, `Line`, `Text`.
    ///
    /// Operator-facing, through [`crate::text`]: the status line and the
    /// Format tab both say *what* is selected, and "Stamp" is the word the
    /// operator used when they placed it.
    pub subtype: String,
    /// §12.5.3 Table 165 bit 8 — the file says the user interface may not
    /// change this annotation's properties.
    ///
    /// Carried on the target rather than checked at each verb, so a surface
    /// can **omit** the controls it governs rather than offer them and let the
    /// engine refuse. That is R83: an affordance that cannot be honoured is
    /// not drawn.
    pub locked: bool,
}

/// A selected annotation, with the outline to draw for it.
#[derive(Debug, Clone, PartialEq)]
pub struct AnnotSelection {
    /// What is selected.
    pub target: AnnotTarget,
    /// Its `/Rect`, in **canvas space** — the zoom-independent space the
    /// content selection's outlines are also cached in, so a zoom or a pan
    /// moves where this is drawn without changing what it is.
    pub outline: Rect,
}

/// Every annotation on `page_index` that a click may select, topmost last.
///
/// # What is excluded, and why each one
///
/// The same four exclusions [`crate::panels::comments`] makes, for the same
/// reasons, plus one this surface needs that the panel does not:
///
/// | excluded | why |
/// |---|---|
/// | `/Widget` | the form field surface owns it — a click there focuses an editor, and two owners of one press is how a field becomes unfillable |
/// | `/Popup` | §12.5.6.14 is a `shall`: a pop-up *"shall not appear alone but is associated with a markup annotation"*. It is a reader-UI window, not content |
/// | `/Link`, `/Movie`, `/PrinterMark`, `/TrapNet` | not authored by the operator and not restylable. `/TrapNet` in particular is prepress output state |
/// | **hidden** (§12.5.3 bit 2) | ★ **this surface's own**, and it is not shared with the panel |
///
/// The hidden case is the one worth stating. The Comments panel *lists* a
/// hidden annotation, deliberately — it is on the page and the operator has a
/// right to know. The canvas must not **select** one, because nothing is drawn
/// there: a click on blank paper would produce a selection outline around
/// nothing, and a Delete would remove something the operator cannot see. The
/// panel is where a hidden annotation is reached, which is exactly the split
/// the forms surface already makes for an undrawn field.
///
/// # Ordering
///
/// `/Annots` order, which is paint order — later entries draw on top. The
/// caller takes the **last** match, so the topmost annotation wins a click,
/// which is the rule page content already follows.
///
/// # Cost
///
/// One `/Annots` walk and one dictionary read per entry, bounded by
/// `pdfcer_core::annot::MAX_ANNOTS_PER_PAGE`. No decomposition, no content
/// stream, no cache — see the module header's table.
pub fn selectable_on<G: ObjectGraph + ?Sized>(
    graph: &G,
    page: &Page,
    page_index: usize,
    ce_dimensions: &BTreeSet<ObjId>,
    shapes: &BTreeMap<ObjId, Vec<(Pos2, Pos2)>>,
) -> Vec<Candidate> {
    let mut out = Vec::new();
    for annot in page_annotations(graph, page.id) {
        // ★★★ **`suppressed_on_screen`, not `hidden`** — corrected 2026-09-05.
        //
        // `hidden()` is `/F` bit 2 alone. `suppressed_on_screen()` is the
        // engine's own screen predicate, `hidden() || no_view()` (§12.5.3,
        // Table 165), and it is what the RENDERER asks before painting.
        //
        // ⇒ Until this line changed, a `/NoView` annotation was **selectable
        // with nothing drawn under the pointer**: an outline appeared around
        // blank paper, handles and all, on a mark the operator cannot see and
        // did not know was there. Found by the note pop-up track, which uses
        // `suppressed_on_screen` correctly and so **disagreed with the
        // selection layer about which annotations exist on screen** — reported
        // rather than fixed at the time because this file belonged to another
        // track that afternoon.
        //
        // ★★ The rule is that **the selection layer must ask the same question
        // the painter asked.** Two predicates over the same flags is exactly
        // the shape this project has been bitten by repeatedly: each half is
        // self-consistent, so no test of either half can see the disagreement.
        // Calling the engine's own predicate rather than spelling
        // `hidden() || no_view()` here is what keeps them from drifting again
        // when Table 165 gains a third bit.
        //
        // ⚠ A `/NoView` annotation is **not** invisible to the operator
        // altogether, and that is why this is a correction and not a
        // concealment: it still prints, the Comments panel still lists it, and
        // `app::status::notes` still counts it. R50's rule holds — *"a page
        // carrying content the operator cannot see is a fact they are entitled
        // to know"*. What it must not be is **clickable on a canvas that is not
        // drawing it.**
        if annot.is_widget() || annot.is_popup || annot.flags.suppressed_on_screen() {
            continue;
        }
        let subtype = String::from_utf8_lossy(&annot.subtype).into_owned();
        if matches!(
            subtype.as_str(),
            "Link" | "Movie" | "PrinterMark" | "TrapNet"
        ) {
            continue;
        }
        // No id means no verb can name it, so selecting it could only ever
        // lead to a refusal — R83 again, at the earliest point it can be
        // applied. `page_annotations` reports an inline (direct) annotation
        // this way; the Comments panel lists those and says so.
        let Some(id) = annot.id else { continue };
        let Some(rect) = annot.rect else { continue };
        let Some(outline) = annot_canvas_rect([rect.llx, rect.lly, rect.urx, rect.ury], page)
        else {
            continue;
        };
        let kind = if ce_dimensions.contains(&id) {
            AnnotKind::CeDimension
        } else {
            AnnotKind::Markup
        };
        out.push(Candidate {
            target: AnnotTarget {
                page: page_index,
                id,
                kind,
                subtype,
                locked: annot.flags.locked(),
            },
            outline,
            // ★ `filter` rather than `unwrap_or_default`: an EMPTY shape would
            // claim nothing and make the annotation unselectable, which is a
            // worse failure than claiming too much. Absent means "not known",
            // and not-known falls back to the rectangle.
            shape: shapes.get(&id).filter(|s| !s.is_empty()).cloned(),
        });
    }
    out
}

/// The annotation under `point`, or `None`.
///
/// `point` is **canvas space**, the same space `selectable_on` returns and the
/// same space the content hit test works in.
///
/// # ★★★ A RECTANGLE IS NOT ALWAYS THE SHAPE, and assuming it was cost the
/// # operator the ability to select anything under a dimension
///
/// This function tested `rect.contains(point)` and nothing else. The reasoning
/// below about tolerance was careful and correct — and it never asked the prior
/// question, which is *is the rectangle the thing?*
///
/// For a stamp, a highlight or a sticky note, yes: the `/Rect` **is** the mark.
/// For a **ce dimension** it is emphatically not. A dimension is two thin
/// witness lines, a dimension line, two arrowheads and a small label — and its
/// `/Rect` is the box around all of that, which for anything but a perfectly
/// horizontal dimension is mostly empty air. A perimeter traced round a
/// building is worse still: its rectangle covers the entire footprint and its
/// ink is the outline.
///
/// So clicking inside that box selected the dimension, and the operator could
/// not reach the drawing underneath:
///
/// > *"selecting space not actually occupied by the lines or text of the
/// > dimension still selects it if I am selecting within the box area it
/// > occupies — I can't select objects underneath it. Where did you learn that
/// > behaviour? It's not in any program I've seen."*
///
/// He is right, and the convention is universal: **a click selects what is
/// under the cursor, not what merely encompasses it.** An unfilled shape's
/// interior belongs to whatever is behind it — every drawing program, every CAD
/// package, every vector editor. A bounding box is what a MARQUEE tests
/// against, and a marquee is a different gesture.
///
/// A candidate may therefore carry a precise `shape` — its drawn segments in
/// canvas space — and where it does, that is what is tested. Where it does not,
/// the rectangle stands, because for those kinds it is the truth.
///
/// # ★ Tolerance: none for a rect, and necessarily some for a segment
///
/// The engine's argument for testing `/Rect` bare:
///
/// > *"`bounds_of` applies the pen half-width at **authoring** time, so the
/// > stored `/Rect` already contains it. A shell hit-testing `/Rect` is
/// > already correct today."*
///
/// The rectangle is the geometry **plus** the margin a tolerance would add, so
/// adding a second would make two adjacent markups claim each other's clicks.
///
/// A **segment** is the opposite case: it is a mathematical line with no width
/// at all, and without a tolerance nothing could ever be clicked. So the shape
/// path takes one, and it is the same click tolerance the content hit test uses
/// — which is what makes a dimension line as easy to hit as the drawing line
/// beside it, rather than easier or harder.
///
/// # Topmost wins
///
/// The **last** match in `/Annots` order, which is the last one painted. A
/// stamp dropped on top of a rectangle is the thing the operator sees and
/// therefore the thing they mean.
#[must_use]
pub fn hit(candidates: &[Candidate], point: Pos2, tolerance: f32) -> Option<AnnotSelection> {
    candidates
        .iter()
        .rev()
        .find(|c| c.claims(point, tolerance))
        .map(|c| AnnotSelection {
            target: c.target.clone(),
            outline: c.outline,
        })
}

/// One selectable annotation: what it is, the box to draw round it, and — when
/// it is known — the geometry it actually occupies.
#[derive(Debug, Clone)]
pub struct Candidate {
    /// What a verb would name.
    pub target: AnnotTarget,
    /// The `/Rect` in canvas space. **Always the outline that is drawn**, even
    /// when [`Self::shape`] is what decides the click — the operator needs to
    /// see the extent of what they selected, and a marching outline round a
    /// perimeter's ink would be the ink again.
    pub outline: Rect,
    /// The drawn segments, canvas space, when this annotation's ink is known
    /// precisely. `None` means *not known*, and the rectangle stands.
    ///
    /// Never `Some(vec![])`: an empty shape would claim nothing and make the
    /// annotation unselectable, which is a worse failure than claiming too
    /// much. The builder drops to `None` instead.
    pub shape: Option<Vec<(Pos2, Pos2)>>,
}

impl Candidate {
    /// Does a click at `point` land on this annotation?
    fn claims(&self, point: Pos2, tolerance: f32) -> bool {
        let Some(shape) = self.shape.as_deref() else {
            return self.outline.contains(point);
        };
        // ★ The rectangle still gates the segment scan. It is a cheap reject
        // that cannot change the answer — every segment is inside the `/Rect`
        // by construction — and on a sheet carrying hundreds of dimensions it
        // is the difference between one containment test per annotation and a
        // distance calculation per segment per annotation, on every click.
        //
        // Expanded by the tolerance, because a segment ON the boundary is
        // hittable from just outside it.
        if !self.outline.expand(tolerance).contains(point) {
            return false;
        }
        shape
            .iter()
            .any(|(a, b)| distance_to_segment(point, *a, *b) <= tolerance)
    }
}

/// Shortest distance from `p` to the segment `a`–`b`, in canvas units.
///
/// The standard projection-and-clamp. Clamping is what makes it a *segment*
/// rather than an infinite line — without it, a click level with a dimension
/// line but far off its end would still hit, which is exactly the
/// claims-too-much failure this whole path exists to remove.
///
/// A degenerate segment (`a == b`) falls out correctly: the projection divides
/// by a zero length, which is guarded, and the answer becomes the distance to
/// the point. A zero-length segment is a real thing here — a perimeter does not
/// de-duplicate its vertices — so this is a case rather than a defence.
fn distance_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let (abx, aby) = (b.x - a.x, b.y - a.y);
    let len_sq = abx.mul_add(abx, aby * aby);
    if len_sq <= f32::EPSILON {
        return p.distance(a);
    }
    let t = (((p.x - a.x) * abx) + ((p.y - a.y) * aby)) / len_sq;
    let t = t.clamp(0.0, 1.0);
    p.distance(Pos2::new(abx.mul_add(t, a.x), aby.mul_add(t, a.y)))
}

/// **Which annotation is under `point`** — the whole question, in one call.
///
/// # Why this is a function and not four lines at the call site
///
/// Because answering it takes four collaborators — the annotation list, the set
/// of which ones are ce dimensions, those dimensions' drawn ink, and the click
/// tolerance — and every one of them has to agree with what is on screen. Four
/// lines inline in `canvas::interact` is four lines that can each be got subtly
/// wrong somewhere else, and the "somewhere else" is what this project keeps
/// paying for.
///
/// It also puts the whole hit-testing story in one file with [`hit`]'s
/// argument, which is where a reader will look for it.
///
/// # The tolerance comes from the MAPPING, so it is zoom-invariant
///
/// [`crate::canvas::mapping::PageMapping::tolerance`] is the same click
/// tolerance the content hit test uses. That is deliberate: a dimension line
/// must be exactly as easy to hit as the drawing line beside it, and a
/// separately chosen number here would drift from it the first time either was
/// tuned.
#[must_use]
pub fn under_pointer(
    doc: &crate::app::state::OpenDoc,
    page_index: usize,
    point: Pos2,
    map: &crate::canvas::mapping::PageMapping,
) -> Option<AnnotSelection> {
    let page = doc.pages.get(page_index)?;
    let ce = crate::panels::comments::model::ce_dimension_annots(&doc.session);
    // ★★ The ce dimensions' ACTUAL INK, so that a click inside a dimension's
    // bounding box but not on it reaches the drawing underneath. See [`hit`]'s
    // header for the operator's report and the argument; the shapes come from
    // the same segment function the dimension is DRAWN from, so what is
    // clickable and what is visible cannot drift apart.
    let shapes = crate::canvas::dimdrag::annot_shapes(doc, &ce);
    let view = doc.session.view();
    let candidates = selectable_on(&view, page, page_index, &ce, &shapes);
    #[allow(clippy::cast_possible_truncation)]
    let tolerance = map.tolerance() as f32;
    hit(&candidates, point, tolerance)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(id: u32, kind: AnnotKind) -> AnnotTarget {
        AnnotTarget {
            page: 0,
            id: ObjId::new(id, 0),
            kind,
            subtype: "Square".to_owned(),
            locked: false,
        }
    }

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(Pos2::new(x, y), egui::vec2(w, h))
    }

    /// ★ **The topmost annotation wins, not the first one found.**
    ///
    /// `/Annots` is paint order, so a stamp dropped over a rectangle is drawn
    /// last and is what the operator sees. A hit test that took the first
    /// match would select the thing underneath — which looks like the click
    /// missing entirely, because the outline appears somewhere the operator
    /// was not pointing.
    #[test]
    fn the_last_painted_annotation_takes_the_click() {
        let candidates = vec![
            boxed(target(1, AnnotKind::Markup), rect(0.0, 0.0, 100.0, 100.0)),
            boxed(target(2, AnnotKind::Markup), rect(20.0, 20.0, 40.0, 40.0)),
        ];
        let hit = hit(&candidates, Pos2::new(30.0, 30.0), TOL).expect("the overlap is a hit");
        assert_eq!(hit.target.id, ObjId::new(2, 0), "the topmost must win");

        // …and outside the upper one, the lower one still takes it.
        let hit = hit_outside(&candidates);
        assert_eq!(hit.target.id, ObjId::new(1, 0));
    }

    fn hit_outside(candidates: &[Candidate]) -> AnnotSelection {
        hit(candidates, Pos2::new(5.0, 5.0), TOL).expect("inside the lower one only")
    }

    /// The click tolerance these tests use. Small, and the shape tests are
    /// built to be unambiguous at it rather than to probe its exact value —
    /// a test tuned to a tolerance breaks when the tolerance is retuned, and
    /// says nothing about the rule it was meant to pin.
    const TOL: f32 = 3.0;

    /// A candidate with no known shape: the rectangle is the truth, which is
    /// the right answer for a stamp, a highlight or a sticky note.
    fn boxed(target: AnnotTarget, outline: Rect) -> Candidate {
        Candidate {
            target,
            outline,
            shape: None,
        }
    }

    /// A candidate whose ink is known — a ce dimension.
    fn inked(target: AnnotTarget, outline: Rect, shape: Vec<(Pos2, Pos2)>) -> Candidate {
        Candidate {
            target,
            outline,
            shape: Some(shape),
        }
    }

    /// ★★★ **The operator's report of 2026-08-20, as one test.**
    ///
    /// > *"selecting space not actually occupied by the lines or text of the
    /// > dimension still selects it if I am selecting within the box area it
    /// > occupies — I can't select objects underneath it."*
    ///
    /// An L of two thin segments across a 100×100 box. A click in the middle of
    /// that box is nowhere near either segment, and must miss — which is what
    /// lets the click reach the drawing underneath. Under the old
    /// `rect.contains` test it hit.
    #[test]
    fn a_click_in_a_dimensions_empty_space_does_not_select_it() {
        let l_shape = vec![
            (Pos2::new(0.0, 0.0), Pos2::new(0.0, 100.0)),
            (Pos2::new(0.0, 100.0), Pos2::new(100.0, 100.0)),
        ];
        let candidates = vec![inked(
            target(1, AnnotKind::CeDimension),
            rect(0.0, 0.0, 100.0, 100.0),
            l_shape,
        )];

        assert!(
            hit(&candidates, Pos2::new(60.0, 30.0), TOL).is_none(),
            "the middle of the box is empty air and belongs to whatever is behind it"
        );
        assert!(
            hit(&candidates, Pos2::new(1.0, 50.0), TOL).is_some(),
            "…and the ink itself is still hittable"
        );
        assert!(
            hit(&candidates, Pos2::new(50.0, 99.0), TOL).is_some(),
            "…on every segment, not just the first"
        );
    }

    /// ★ The tolerance is what makes a hairline clickable at all, and it is
    /// bounded: a segment is a SEGMENT, not an infinite line.
    ///
    /// A click level with the horizontal arm but well past its end must miss.
    /// Without the clamp in `distance_to_segment` it would hit, and a dimension
    /// would claim a stripe across the whole sheet.
    #[test]
    fn a_segment_does_not_claim_the_line_it_lies_on() {
        let arm = vec![(Pos2::new(0.0, 50.0), Pos2::new(20.0, 50.0))];
        let candidates = vec![inked(
            target(1, AnnotKind::CeDimension),
            rect(0.0, 0.0, 100.0, 100.0),
            arm,
        )];
        assert!(
            hit(&candidates, Pos2::new(10.0, 50.0), TOL).is_some(),
            "on it"
        );
        assert!(
            hit(&candidates, Pos2::new(80.0, 50.0), TOL).is_none(),
            "level with it, far past its end — a different thing entirely"
        );
    }

    /// A shape-less candidate still behaves exactly as it did. A stamp's
    /// rectangle IS the stamp, and nothing about this change may make one
    /// harder to click.
    #[test]
    fn an_annotation_with_no_known_shape_still_uses_its_rectangle() {
        let candidates = vec![boxed(
            target(1, AnnotKind::Markup),
            rect(0.0, 0.0, 100.0, 100.0),
        )];
        assert!(hit(&candidates, Pos2::new(50.0, 50.0), TOL).is_some());
    }

    /// A click on blank paper selects nothing.
    ///
    /// Stated because the alternative — nearest-match — is a plausible
    /// implementation that would make it impossible to *deselect* by clicking
    /// away, which is the gesture every operator tries first.
    #[test]
    fn a_click_outside_every_annotation_is_not_a_hit() {
        let candidates = vec![boxed(
            target(1, AnnotKind::Markup),
            rect(0.0, 0.0, 10.0, 10.0),
        )];
        assert!(hit(&candidates, Pos2::new(50.0, 50.0), TOL).is_none());
        assert!(hit(&[], Pos2::new(0.0, 0.0), TOL).is_none());
    }

    /// ★ The kind survives the hit test.
    ///
    /// The one property that routes a later restyle to `set_dimension_style`
    /// rather than `set_markup_style`. If it were dropped here and re-derived
    /// downstream, the re-derivation would be the thing that could be
    /// forgotten — and forgetting it turns a recolour into a dimension that
    /// loses its label.
    #[test]
    fn a_ce_dimension_stays_a_ce_dimension() {
        let candidates = vec![boxed(
            target(7, AnnotKind::CeDimension),
            rect(0.0, 0.0, 50.0, 50.0),
        )];
        let hit = hit(&candidates, Pos2::new(10.0, 10.0), TOL).expect("a hit");
        assert_eq!(hit.target.kind, AnnotKind::CeDimension);
    }
    /// ★★★ **The selection layer asks the SAME question the painter asked.**
    ///
    /// # The defect
    ///
    /// `selectable_on` filtered on `flags.hidden()` — `/F` bit 2 alone — while
    /// the renderer and the note pop-up both ask
    /// `AnnotFlags::suppressed_on_screen()`, which is `hidden() || no_view()`
    /// (§12.5.3, Table 165). So a **`/NoView`** annotation was **selectable
    /// with nothing drawn under the pointer**: click blank paper and an outline
    /// appears, with handles, around a mark the operator cannot see and did not
    /// know was there.
    ///
    /// Found by the note-pop-up work, which noticed the two layers disagreeing
    /// about which annotations exist on screen, and reported rather than fixed
    /// because this file belonged to another track that afternoon.
    ///
    /// # Why no existing test could have caught it
    ///
    /// ★★ **Two predicates over the same flags, each self-consistent.** Every
    /// test of the selection layer used the selection layer's own notion of
    /// visible, and every test of the painter used the painter's. A
    /// disagreement between two correct halves is invisible to any test of
    /// either half — which is why this one asserts them **against each other**
    /// rather than against a constant, and why the fix calls the engine's
    /// predicate instead of re-spelling `hidden() || no_view()` here.
    ///
    /// # What it deliberately does NOT assert
    ///
    /// ⚠ That a `/NoView` annotation is unreachable. It is not, and must not be
    /// — it still prints, the Comments panel still lists it, and the page's
    /// notes still count it. R50: *"a page carrying content the operator cannot
    /// see is a fact they are entitled to know."* The claim is narrower and
    /// exact: **it is not clickable on a canvas that is not drawing it.**
    #[test]
    fn an_annotation_the_canvas_does_not_draw_cannot_be_clicked() {
        use pdfcer_core::annot::AnnotFlags;
        use pdfcer_core::object::{Dict, Name, Object};

        // The three states, spelled from the engine's own bit constants so a
        // renumbering cannot leave this test asserting about the wrong flag.
        let plain = AnnotFlags(0);
        let hidden = AnnotFlags(AnnotFlags::HIDDEN);
        let no_view = AnnotFlags(AnnotFlags::NO_VIEW);

        // The positive control. Without it, a filter that rejected EVERYTHING
        // would satisfy the two assertions below and this test would be a
        // statement about nothing.
        assert!(
            !plain.suppressed_on_screen(),
            "an ordinary annotation must remain selectable, or this test is \
             asserting that the canvas selects nothing at all"
        );

        assert!(
            hidden.suppressed_on_screen(),
            "Hidden was already excluded and must stay excluded"
        );
        assert!(
            no_view.suppressed_on_screen(),
            "NoView is drawn by nothing, so a click on it would put an outline \
             and handles around blank paper — this is the case `hidden()` alone \
             let through"
        );

        // ★ And the identity that keeps the two layers from drifting apart
        // again: the predicate this file filters on IS the predicate the
        // painter filters on. Asserted as an equality of derivations rather
        // than by repeating the expression, so a third bit added to Table 165
        // moves both at once.
        for flags in [plain, hidden, no_view] {
            assert_eq!(
                flags.suppressed_on_screen(),
                flags.hidden() || flags.no_view(),
                "the screen predicate must stay the engine's, not a copy"
            );
        }

        // ★★★ **AND THE CALL SITE, which is the half that actually catches a
        // regression here.** Everything above is a contract test on
        // `AnnotFlags`, and every line of it passes on a build where
        // `selectable_on` still filters on `hidden()` alone — which is exactly
        // the vacuous shape this project keeps meeting. So drive the real
        // function over a real `/Annots` list.
        //
        // Hand-built graph rather than a fixture, for `notepopup::model`'s
        // stated reason: the subject is **one flag**, and a fixture would make
        // the assertion depend on a file, a page tree and an `/Annots` walk —
        // three things that can fail for reasons this assertion is not about.
        // There is also no fixture in either corpus carrying `/NoView`, which
        // is the deeper reason the defect survived.
        let square = |num: u32, flags: u32| {
            let mut d = Dict::new();
            d.insert(Name::from(b"Type"), Object::Name(Name::from(b"Annot")));
            d.insert(Name::from(b"Subtype"), Object::Name(Name::from(b"Square")));
            d.insert(Name::from(b"F"), Object::Integer(i64::from(flags)));
            d.insert(
                Name::from(b"Rect"),
                Object::Array(vec![
                    Object::Integer(10),
                    Object::Integer(10),
                    Object::Integer(60),
                    Object::Integer(60),
                ]),
            );
            (ObjId::new(num, 0), Object::Dict(d))
        };

        let page_id = ObjId::new(1, 0);
        let mut page_dict = Dict::new();
        page_dict.insert(Name::from(b"Type"), Object::Name(Name::from(b"Page")));
        page_dict.insert(
            Name::from(b"Annots"),
            Object::Array(vec![
                Object::Reference(ObjId::new(2, 0)),
                Object::Reference(ObjId::new(3, 0)),
            ]),
        );

        let graph = Loose(vec![
            (page_id, Object::Dict(page_dict)),
            square(2, 0),                   // ordinary
            square(3, AnnotFlags::NO_VIEW), // drawn by nothing
        ]);
        let page = Page {
            id: page_id,
            resources: Dict::new(),
            media_box: pdfcer_core::page_tree::Rect::from_corners(0.0, 0.0, 612.0, 792.0),
            crop_box: pdfcer_core::page_tree::Rect::from_corners(0.0, 0.0, 612.0, 792.0),
            rotate: 0,
            contents: Vec::new(),
            contents_flattened: 0,
            contents_unresolved: 0,
        };

        let ids: Vec<u32> = selectable_on(&graph, &page, 0, &BTreeSet::new(), &BTreeMap::new())
            .into_iter()
            .map(|c| c.target.id.num)
            .collect();

        assert_eq!(
            ids,
            vec![2],
            "the ordinary square must be selectable and the /NoView one must \
             not — a click on it would put an outline and handles around blank \
             paper. Got {ids:?}"
        );
    }

    /// A graph of loose objects, for the call-site half of the test above.
    struct Loose(Vec<(ObjId, pdfcer_core::object::Object)>);

    impl pdfcer_core::graph::ObjectGraph for Loose {
        fn value(&self, id: ObjId) -> Option<&pdfcer_core::object::Object> {
            self.0.iter().find(|(o, _)| *o == id).map(|(_, v)| v)
        }
        fn trailer_entry(&self, _key: &[u8]) -> Option<&pdfcer_core::object::Object> {
            None
        }
    }
}
