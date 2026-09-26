//! # `canvas::selection::annot` — clicking the things pdfcer itself put on the page
//!
//! ## The gap this closes, and how long it was open
//!
//! `FEATURES.md` recorded it on 2026-08-17, under the Format contextual tab:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/selection/annot.md`.
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
use pdfcer_core::object::ObjId;
use pdfcer_core::page_tree::Page;

use crate::canvas::mapping::{annot_canvas_rect, oriented_canvas_quad};

/// Which family an annotation belongs to, and therefore **which verb may
/// restyle it**.
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
    /// **Where the artwork ACTUALLY sits**, when that differs from
    /// [`Self::outline`] — the four corners of the appearance's placed
    /// `/BBox`, canvas space, in the artwork's own frame (see
    /// [`crate::canvas::annotquad`]).
    ///
    /// `None` for the overwhelmingly common unturned case **and** for any
    /// annotation with no usable appearance stream, and both mean the same
    /// thing to a caller: *use the rectangle, it is the truth here.*
    ///
    /// # Why this rides on the selection rather than being re-read at paint
    ///
    /// Because the outline, the grips and the hit test must agree about where
    /// the object is, and this project has been bitten three times by two
    /// surfaces deriving the same geometry independently — most recently when a
    /// dimension's vertex handles were painted from the selection and
    /// hit-tested from a capability check, so they were visible and untouchable
    /// in the very mode that authors dimensions. One value, one decision, every
    /// consumer.
    pub oriented: Option<[Pos2; 4]>,
}

/// Every annotation on `page_index` that a click may select, topmost last.
pub fn selectable_on(
    view: &pdfcer_core::view::DocumentView<'_>,
    page: &Page,
    page_index: usize,
    ce_dimensions: &BTreeSet<ObjId>,
    shapes: &BTreeMap<ObjId, Vec<(Pos2, Pos2)>>,
) -> Vec<Candidate> {
    let mut out = Vec::new();
    for annot in page_annotations(view, page.id) {
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
        // The rule is that **the selection layer must ask the same question
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
            // `filter` rather than `unwrap_or_default`: an EMPTY shape would
            // claim nothing and make the annotation unselectable, which is a
            // worse failure than claiming too much. Absent means "not known",
            // and not-known falls back to the rectangle.
            shape: shapes.get(&id).filter(|s| !s.is_empty()).cloned(),
            // The turned outline, and **only when it is actually turned**.
            //
            // `is_upright` answering `true` collapses to `None` right here
            // rather than at the painter, so every downstream consumer gets one
            // fact — *is there a second frame to honour?* — instead of each one
            // re-deciding what counts as upright. The unturned path is then
            // bit-for-bit the code it has always been, which is why this change
            // cannot regress the 99 % of annotations nobody has rotated.
            oriented: crate::canvas::annotquad::oriented(view, &annot)
                .filter(|q| !q.is_upright())
                .and_then(|q| oriented_canvas_quad(q.corners, page)),
        });
    }
    out
}

/// The annotation under `point`, or `None`.
#[must_use]
pub fn hit(candidates: &[Candidate], point: Pos2, tolerance: f32) -> Option<AnnotSelection> {
    candidates
        .iter()
        .rev()
        .find(|c| c.claims(point, tolerance))
        .map(|c| AnnotSelection {
            target: c.target.clone(),
            outline: c.outline,
            oriented: c.oriented,
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
    /// The artwork's four placed corners, canvas space — see
    /// [`AnnotSelection::oriented`], which this becomes on selection.
    ///
    /// **Carried but NOT hit-tested against**, deliberately. Narrowing the
    /// click target to the turned quad would be more precise and would be the
    /// wrong trade: `/Rect` already contains the pen half-width (the engine's
    /// own argument for testing it bare), and a turned mark is exactly the case
    /// where an operator's aim is least reliable. Selection stays generous;
    /// only the *drawing* gets more honest.
    pub oriented: Option<[Pos2; 4]>,
}

impl Candidate {
    /// Does a click at `point` land on this annotation?
    fn claims(&self, point: Pos2, tolerance: f32) -> bool {
        let Some(shape) = self.shape.as_deref() else {
            return self.outline.contains(point);
        };
        // The rectangle still gates the segment scan. It is a cheap reject
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
#[must_use]
pub fn under_pointer(
    doc: &crate::app::state::OpenDoc,
    page_index: usize,
    point: Pos2,
    map: &crate::canvas::mapping::PageMapping,
) -> Option<AnnotSelection> {
    let page = doc.pages.get(page_index)?;
    let ce = crate::panels::comments::model::ce_dimension_annots(&doc.session);
    // The ce dimensions' ACTUAL INK, so that a click inside a dimension's
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

    /// **The topmost annotation wins, not the first one found.**
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
            oriented: None,
        }
    }

    /// A candidate whose ink is known — a ce dimension.
    fn inked(target: AnnotTarget, outline: Rect, shape: Vec<(Pos2, Pos2)>) -> Candidate {
        Candidate {
            target,
            outline,
            shape: Some(shape),
            oriented: None,
        }
    }

    /// **The operator's report of 2026-08-20, as one test.**
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

    /// The tolerance is what makes a hairline clickable at all, and it is
    /// bounded: a segment is a SEGMENT, not an infinite line.
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
    #[test]
    fn a_click_outside_every_annotation_is_not_a_hit() {
        let candidates = vec![boxed(
            target(1, AnnotKind::Markup),
            rect(0.0, 0.0, 10.0, 10.0),
        )];
        assert!(hit(&candidates, Pos2::new(50.0, 50.0), TOL).is_none());
        assert!(hit(&[], Pos2::new(0.0, 0.0), TOL).is_none());
    }

    /// The kind survives the hit test.
    #[test]
    fn a_ce_dimension_stays_a_ce_dimension() {
        let candidates = vec![boxed(
            target(7, AnnotKind::CeDimension),
            rect(0.0, 0.0, 50.0, 50.0),
        )];
        let hit = hit(&candidates, Pos2::new(10.0, 10.0), TOL).expect("a hit");
        assert_eq!(hit.target.kind, AnnotKind::CeDimension);
    }
    /// **The selection layer asks the SAME question the painter asked.**
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

        // And the identity that keeps the two layers from drifting apart
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

        // **AND THE CALL SITE, which is the half that actually catches a
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
            resources_defaulted: false,
        };

        let view = pdfcer_core::view::DocumentView::new(
            &graph,
            &[],
            pdfcer_core::PdfVersion { major: 1, minor: 7 },
        );
        let ids: Vec<u32> = selectable_on(&view, &page, 0, &BTreeSet::new(), &BTreeMap::new())
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
