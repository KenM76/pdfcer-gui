//! `panels::layers::highlight` — which layer the current selection is on.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/layers/highlight.md`.

use pdfcer_core::vector::PageObjects;

use crate::app::state::OpenDoc;
use crate::canvas::target::TargetId;

pub use pdfcer_gui_base::layermembership::{Membership, Unresolved, for_leaf, for_object};

/// The answer for one selected target, resolved against a page's model.
fn for_target(model: &PageObjects, target: TargetId) -> Membership {
    let page_malformed = model.diagnostics.oc_unresolved > 0;
    match target {
        TargetId::Object(i) => match usize::try_from(i).ok().and_then(|i| model.objects.get(i)) {
            Some(object) => for_object(object.oc(), page_malformed),
            None => Membership::Unknown(Unresolved::Stale),
        },
        TargetId::Leaf(i) => match usize::try_from(i).ok().and_then(|i| model.leaves.get(i)) {
            Some(leaf) => for_leaf(
                leaf.oc(),
                leaf.containment.len(),
                model
                    .objects
                    .get(leaf.paint_order)
                    .and_then(pdfcer_core::vector::VectorObject::oc),
                page_malformed,
            ),
            None => Membership::Unknown(Unresolved::Stale),
        },
    }
}

/// **Which layer is the current selection on?**
#[must_use]
pub fn resolve(doc: &OpenDoc) -> Membership {
    let Some(annot) = doc.selection.annot() else {
        return resolve_content(doc);
    };
    let view = doc.session.view();
    // `pages_in` and not `EditSession::pages()`: the panel holds a shared
    // `&OpenDoc` and the session's own accessor takes `&mut self`. This is the
    // same call `panels::comments` makes over the same view.
    let Ok(pages) = pdfcer_core::page_tree::pages_in(&view) else {
        // The page tree would not resolve. Emphatically not `None`: a document
        // we cannot read the structure of is the exact case the absent variant
        // exists for.
        return Membership::Unknown(Unresolved::PageNotDecomposed);
    };
    let Some(page) = pages.get(annot.target.page) else {
        // The selection names a page the current revision does not have —
        // reachable for one frame after a page delete, before the selection is
        // re-resolved.
        return Membership::Unknown(Unresolved::Stale);
    };
    match pdfcer_core::annot::page_annotations(&view, page.id)
        .into_iter()
        .find(|a| a.id == Some(annot.target.id))
    {
        Some(a) => match a.oc {
            Some(oc) => Membership::Group(oc),
            None => Membership::None,
        },
        // Selected, but no longer in the page's `/Annots`. Same reasoning as
        // the missing page.
        None => Membership::Unknown(Unresolved::Stale),
    }
}

/// The content half of [`resolve`], split out so the annotation arm stays
/// readable and so the fold has somewhere to be commented.
fn resolve_content(doc: &OpenDoc) -> Membership {
    if doc.selection.is_empty() {
        return Membership::NothingSelected;
    }
    let page = doc.view.page_index;
    let targets = doc.selection.targets_on(page);
    // See the doc comment: entries on another page are a real state, and
    // folding only the current page's would report an empty answer about a
    // non-empty selection.
    let elsewhere = doc.selection.entries().iter().any(|e| e.page != page);
    let off_page = if elsewhere {
        Membership::Unknown(Unresolved::OtherPage)
    } else {
        Membership::NothingSelected
    };

    let Some(provider) = doc.page_objects() else {
        // The page would not decompose. The Objects panel says so in words on
        // the same frame; this line is why the layer row is not lit.
        return Membership::Unknown(Unresolved::PageNotDecomposed).join(off_page);
    };
    let model = provider.page_objects();
    targets
        .into_iter()
        .map(|t| for_target(model, t))
        .fold(off_page, Membership::join)
}

/// **How many parts the one selected object holds**, when that number is a
/// reason to distrust the word "selected".
#[must_use]
pub fn parts_in_selected_object(doc: &OpenDoc) -> Option<usize> {
    if doc.selection.annot().is_some() {
        return None;
    }
    let page = doc.view.page_index;
    let targets = doc.selection.targets_on(page);
    let [target] = targets.as_slice() else {
        return None;
    };
    if doc.selection.entries().iter().any(|e| e.page != page) {
        return None;
    }
    let provider = doc.page_objects()?;
    let model = provider.page_objects();
    let object = match *target {
        TargetId::Object(i) => model.objects.get(usize::try_from(i).ok()?)?,
        TargetId::Leaf(i) => &model.leaves.get(usize::try_from(i).ok()?)?.object,
    };
    match object {
        pdfcer_core::vector::VectorObject::Path(p) if p.subpaths.len() > 1 => {
            Some(p.subpaths.len())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::OpenDoc;
    use crate::panels::objects::test_support::engine_fixture;
    use pdfcer_core::object::ObjId;

    fn oc(n: u32) -> ObjId {
        ObjId::new(n, 0)
    }

    /// The one fixture in either corpus that can falsify this feature.
    ///
    /// `layers/painted-layers.pdf` is fourteen objects of hand-written syntax
    /// carrying **four** optional-content groups and, critically, **an object
    /// painted after every `EMC`** — so it holds both halves of the relation:
    ///
    /// ```text
    /// /OC /L1 BDC  0 0 0 rg 60 60 120 120 re f  EMC     <- "Visible Box"
    /// /OC /L2 BDC  0 0 0 rg 400 60 120 120 re f
    ///   /OC /L4 BDC 0 0 0 rg 400 220 120 120 re f EMC   <- "Nested Inner", innermost wins
    /// EMC
    /// /OC /L3 BDC  0 0 300 792 re W n EMC
    /// 0.5 g 0 600 612 60 re f                           <- on NO layer
    /// ```
    ///
    /// A fixture whose every object shares one layer would make *"the
    /// answer follows the selection"* true of a build that ignores the
    /// selection entirely. That is this project's vacuous-pass shape, and it
    /// is why these tests use two objects with different answers rather than
    /// one with a right answer.
    fn painted_layers() -> OpenDoc {
        let path = engine_fixture("layers/painted-layers.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
        OpenDoc::new(path, pdfcer_core::edit::EditSession::new(doc), pages)
    }

    /// The paint-order index of the object whose page bbox is **exactly**
    /// this rectangle.
    fn index_of_bbox(doc: &OpenDoc, min: (f64, f64), max: (f64, f64)) -> TargetId {
        let provider = doc.page_objects().expect("the fixture decomposes");
        let model = provider.page_objects();
        let near = |a: f64, b: f64| (a - b).abs() < 0.01;
        let mut found = model.objects.iter().enumerate().filter(|(_, o)| {
            let b = o.page_bbox();
            near(b.min.x, min.0)
                && near(b.min.y, min.1)
                && near(b.max.x, max.0)
                && near(b.max.y, max.1)
        });
        let (i, _) = found.next().unwrap_or_else(|| {
            panic!("no object has bounds {min:?}..{max:?} — the fixture has changed")
        });
        assert!(
            found.next().is_none(),
            "two objects share the bounds {min:?}..{max:?}, so this helper names neither of them"
        );
        TargetId::Object(u64::try_from(i).expect("an index fits"))
    }

    /// The layer name `resolve` lands on, resolved the way the panel resolves
    /// it — through the document's own `/OCProperties` list.
    fn resolved_name(doc: &OpenDoc) -> Option<String> {
        let read = pdfcer_core::layers::read_layers(&doc.session.view());
        resolve(doc)
            .highlighted()
            .and_then(|id| crate::panels::layers::layer_name_for(&read, id))
    }

    /// **Selecting a page object names the layer it is painted on.**
    #[test]
    fn selecting_a_page_object_names_the_layer_it_is_painted_on() {
        let mut open = painted_layers();
        let target = index_of_bbox(&open, (60.0, 60.0), (180.0, 180.0));
        open.selection.select_only(0, target, "test");
        assert_eq!(
            resolved_name(&open).as_deref(),
            Some("Visible Box"),
            "the square at (60,60)-(180,180) is painted inside `/OC /L1`, whose /Name is \
             'Visible Box'"
        );
    }

    /// **…and an object painted outside every section is reported as on
    /// no layer, not as unknown and not as somebody else's layer.**
    #[test]
    fn an_object_outside_every_section_is_on_no_layer() {
        let mut open = painted_layers();
        let target = index_of_bbox(&open, (0.0, 600.0), (612.0, 660.0));
        open.selection.select_only(0, target, "test");
        assert_eq!(
            resolve(&open),
            Membership::None,
            "the grey bar is painted after every EMC, so it is on no optional-content group — \
             and `None` here is a POSITIVE fact, distinct from `Unknown`"
        );
        assert_eq!(resolved_name(&open), None, "nothing may be highlighted");
    }

    /// **The innermost `/OC` wins**, which is what `current_oc` resolves
    /// and what the renderer honours.
    #[test]
    fn the_innermost_section_is_the_layer_not_the_outer_one() {
        let mut open = painted_layers();
        let target = index_of_bbox(&open, (400.0, 220.0), (520.0, 340.0));
        open.selection.select_only(0, target, "test");
        assert_eq!(resolved_name(&open).as_deref(), Some("Nested Inner"));
    }

    /// **The granularity line stays silent on an ordinary object.**
    #[test]
    fn a_one_part_object_says_nothing_about_its_parts() {
        let mut open = painted_layers();
        let target = index_of_bbox(&open, (60.0, 60.0), (180.0, 180.0));
        open.selection.select_only(0, target, "test");
        assert_eq!(parts_in_selected_object(&open), None);
    }

    /// **The two points `ui-verify selecting_an_object_names_its_layer`
    /// aims at, pinned headlessly.**
    #[test]
    fn the_driven_checks_two_aim_points_land_where_it_thinks() {
        let open = painted_layers();
        let provider = open.page_objects().expect("the fixture decomposes");
        let model = provider.page_objects();
        let read = pdfcer_core::layers::read_layers(&open.session.view());

        let layer_at = |x: f64, y: f64| -> Option<String> {
            let hits = pdfcer_core::vector::hit_test_point_deep(
                model,
                pdfcer_core::vector::Point::new(x, y),
                3.0,
            );
            let first = hits.first().copied().expect("the point must hit something");
            let i = match first {
                pdfcer_core::vector::HitTarget::Object(i) => i,
                pdfcer_core::vector::HitTarget::Leaf(_) => {
                    panic!("this fixture has no forms, so a leaf hit means the fixture changed")
                }
            };
            model.objects[i]
                .oc()
                .and_then(|id| crate::panels::layers::layer_name_for(&read, id))
        };

        assert_eq!(
            layer_at(120.0, 120.0).as_deref(),
            Some("Visible Box"),
            "the check's first click must land on the square inside `/OC /L1`"
        );
        assert_eq!(
            layer_at(150.0, 630.0),
            None,
            "the check's second click must land on the grey bar, which is on NO layer, and NOT on the `Clip Only` path whose bbox also covers this point"
        );
    }

    /// **Nothing selected is nothing selected**, on a layered document — the
    /// state that must not be confused with "on no layer".
    #[test]
    fn an_empty_selection_answers_the_empty_answer() {
        let open = painted_layers();
        assert_eq!(resolve(&open), Membership::NothingSelected);
    }

    /// **`Unknown` and `None` are different values**, which is the whole
    /// reason this type exists rather than an `Option<ObjId>`.
    #[test]
    fn not_on_a_layer_is_not_the_same_answer_as_cannot_tell() {
        assert_ne!(Membership::None, Membership::Unknown(Unresolved::Stale));
        assert_ne!(
            Membership::NothingSelected,
            Membership::Unknown(Unresolved::Stale)
        );
        assert_ne!(Membership::None, Membership::NothingSelected);
        assert_ne!(Membership::None, Membership::Mixed);
    }

    /// **The reason is part of the answer.**
    #[test]
    fn two_reasons_are_two_answers() {
        assert_ne!(
            Membership::Unknown(Unresolved::NestedForm),
            Membership::Unknown(Unresolved::Malformed)
        );
    }

    /// **Only a known group highlights a row.**
    #[test]
    fn only_a_known_group_highlights_anything() {
        assert_eq!(Membership::Group(oc(7)).highlighted(), Some(oc(7)));
        assert_eq!(Membership::None.highlighted(), None);
        assert_eq!(
            Membership::Unknown(Unresolved::NestedForm).highlighted(),
            None
        );
        assert_eq!(Membership::NothingSelected.highlighted(), None);
        assert_eq!(Membership::Mixed.highlighted(), None);
    }

    /// **The trace vocabulary is one word per state, and no word is
    /// empty.**
    #[test]
    fn the_trace_vocabulary_separates_every_state() {
        let words = [
            Membership::NothingSelected.kind(),
            Membership::Group(oc(1)).kind(),
            Membership::None.kind(),
            Membership::Unknown(Unresolved::Stale).kind(),
            Membership::Mixed.kind(),
        ];
        for (i, a) in words.iter().enumerate() {
            assert!(!a.is_empty());
            assert!(!a.contains(' '), "a trace word with a space in it: {a}");
            for b in words.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
        let reasons = [
            Unresolved::PageNotDecomposed,
            Unresolved::Malformed,
            Unresolved::NestedForm,
            Unresolved::Stale,
            Unresolved::OtherPage,
        ];
        for (i, a) in reasons.iter().enumerate() {
            assert_eq!(Membership::Unknown(*a).reason(), a.key());
            for b in reasons.iter().skip(i + 1) {
                assert_ne!(a.key(), b.key());
            }
        }
        // Every state that has no reason still prints one, so the line's shape
        // does not vary with the answer.
        for m in [
            Membership::NothingSelected,
            Membership::Group(oc(1)),
            Membership::None,
            Membership::Mixed,
        ] {
            assert!(!m.reason().is_empty(), "{m:?} prints an empty reason");
        }
    }

    /// **A group id round-trips**, so the panel's row lookup is a comparison
    /// rather than a translation.
    #[test]
    fn the_group_id_is_the_one_the_layers_list_speaks() {
        let id = oc(42);
        assert_eq!(Membership::Group(id).highlighted(), Some(id));
    }

    // -------------------------------------------------------------------
    // The per-object rule.
    // -------------------------------------------------------------------

    /// **A resolved `/OC` is the layer, and a page's malformation elsewhere
    /// does not taint it.**
    #[test]
    fn a_named_group_is_the_answer_even_on_a_malformed_page() {
        assert_eq!(for_object(Some(oc(4)), false), Membership::Group(oc(4)));
        assert_eq!(for_object(Some(oc(4)), true), Membership::Group(oc(4)));
    }

    /// **`None` means "on no layer" only while the page's `/OC` sections
    /// all resolved.**
    #[test]
    fn an_unresolvable_section_demotes_no_layer_to_cannot_tell() {
        assert_eq!(for_object(None, false), Membership::None);
        assert_eq!(
            for_object(None, true),
            Membership::Unknown(Unresolved::Malformed)
        );
    }

    // -------------------------------------------------------------------
    // The form-leaf rule — divergence D1.
    // -------------------------------------------------------------------

    /// **A leaf's own `/OC` wins at any depth.**
    #[test]
    fn a_leafs_own_group_wins_over_the_form_it_is_in() {
        assert_eq!(
            for_leaf(Some(oc(9)), 1, Some(oc(4)), false),
            Membership::Group(oc(9))
        );
        assert_eq!(
            for_leaf(Some(oc(9)), 3, Some(oc(4)), false),
            Membership::Group(oc(9))
        );
    }

    /// **D1, repaired: a leaf one form deep inherits the layer its `Do`
    /// was painted under.**
    #[test]
    fn a_leaf_one_form_deep_inherits_the_forms_layer() {
        assert_eq!(
            for_leaf(None, 1, Some(oc(4)), false),
            Membership::Group(oc(4))
        );
    }

    /// **…and a leaf in an unlayered form is genuinely on no layer.**
    ///
    /// The other direction of the same arm, and the one that stops the repair
    /// from becoming "everything inside a form is on a layer".
    #[test]
    fn a_leaf_in_an_unlayered_form_is_on_no_layer() {
        assert_eq!(for_leaf(None, 1, None, false), Membership::None);
    }

    /// **Deeper than one form, pdfcer says so rather than guessing.**
    #[test]
    fn a_leaf_two_forms_deep_is_not_guessed_from_the_outer_one() {
        assert_eq!(
            for_leaf(None, 2, Some(oc(4)), false),
            Membership::Unknown(Unresolved::NestedForm)
        );
        assert_eq!(
            for_leaf(None, 2, None, false),
            Membership::Unknown(Unresolved::NestedForm)
        );
    }

    /// **The nesting reason outranks the malformation reason**, because it is
    /// the more specific of the two and the one the operator can act on.
    #[test]
    fn the_nesting_reason_is_the_one_reported() {
        assert_eq!(
            for_leaf(None, 4, None, true),
            Membership::Unknown(Unresolved::NestedForm)
        );
    }

    // -------------------------------------------------------------------
    // The fold.
    // -------------------------------------------------------------------

    /// **Nothing selected is the identity**, so a fold needs no empty case.
    #[test]
    fn nothing_selected_is_the_folds_identity() {
        let g = Membership::Group(oc(1));
        assert_eq!(Membership::NothingSelected.join(g), g);
        assert_eq!(g.join(Membership::NothingSelected), g);
        assert_eq!(
            Membership::NothingSelected.join(Membership::NothingSelected),
            Membership::NothingSelected
        );
    }

    /// **Agreement survives; disagreement becomes `Mixed`.**
    #[test]
    fn two_objects_on_one_layer_stay_on_one_layer() {
        let a = Membership::Group(oc(1));
        let b = Membership::Group(oc(2));
        assert_eq!(a.join(a), a);
        assert_eq!(a.join(b), Membership::Mixed);
        assert_eq!(b.join(a), Membership::Mixed);
    }

    /// **A layered object and an unlayered one are `Mixed`, not the
    /// layer.**
    #[test]
    fn a_layered_and_an_unlayered_object_are_mixed() {
        assert_eq!(
            Membership::Group(oc(1)).join(Membership::None),
            Membership::Mixed
        );
        assert_eq!(
            Membership::None.join(Membership::Group(oc(1))),
            Membership::Mixed
        );
        assert_eq!(Membership::None.join(Membership::None), Membership::None);
    }

    /// **One unanswerable member makes the whole answer unanswerable.**
    ///
    /// The highlight is read as a claim about the *selection*, so it may not
    /// survive a member nobody could resolve.
    #[test]
    fn an_unknown_member_withholds_the_whole_answer() {
        let u = Membership::Unknown(Unresolved::NestedForm);
        assert_eq!(Membership::Group(oc(1)).join(u), u);
        assert_eq!(u.join(Membership::Group(oc(1))), u);
        assert_eq!(Membership::None.join(u), u);
    }

    /// **…including over an established disagreement, and this assertion
    /// is INVERTED from the one that was written first.**
    #[test]
    fn an_unknown_outranks_even_an_established_disagreement() {
        let u = Membership::Unknown(Unresolved::Stale);
        assert_eq!(Membership::Mixed.join(u), u);
        assert_eq!(u.join(Membership::Mixed), u);
    }

    /// **Two unanswerable members merge by PRIORITY, not by position.**
    #[test]
    fn two_reasons_merge_by_priority_rather_than_by_position() {
        let read = Membership::Unknown(Unresolved::PageNotDecomposed);
        let stale = Membership::Unknown(Unresolved::Stale);
        assert_eq!(read.join(stale), read);
        assert_eq!(stale.join(read), read);
        assert!(
            Unresolved::PageNotDecomposed < Unresolved::Stale,
            "the declaration order IS the priority order — see `Unresolved`"
        );
    }

    /// **The join is commutative and associative**, which is what makes the
    /// answer independent of the order `targets_on` happens to return.
    #[test]
    fn the_fold_does_not_depend_on_selection_order() {
        // TWO different `Unknown`s, deliberately. The first version of
        // this array held one, and with one the commutativity of
        // `Unknown ⊔ Unknown` is unobservable — a "first operand wins" rule
        // would have passed. The associativity failure it DID catch was found
        // by the `Mixed` entry sitting beside a `Group` pair; both entries are
        // load-bearing and neither is decoration.
        let all = [
            Membership::NothingSelected,
            Membership::Group(oc(1)),
            Membership::Group(oc(2)),
            Membership::None,
            Membership::Unknown(Unresolved::PageNotDecomposed),
            Membership::Unknown(Unresolved::Stale),
            Membership::Mixed,
        ];
        for a in all {
            for b in all {
                assert_eq!(a.join(b), b.join(a), "join is not commutative: {a:?} {b:?}");
                for c in all {
                    assert_eq!(
                        a.join(b).join(c),
                        a.join(b.join(c)),
                        "join is not associative: {a:?} {b:?} {c:?}"
                    );
                }
            }
        }
    }
}
