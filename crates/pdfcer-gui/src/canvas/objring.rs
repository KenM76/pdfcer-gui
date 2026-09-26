//! # `canvas::objring` — Tab walks the objects on the page
//!
//! `OPERATOR_REQUESTS.md` O204, the object half:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/objring.md`.

use crate::canvas::pick::{PickClass, PickFilter};
use crate::canvas::selection::SelectionState;
use crate::canvas::tabnav;
use crate::canvas::target::CanvasTargetProvider;
use crate::panels::objects::provider::{ObjectModelProvider, TargetId};

/// O204 decision 3: the object ring stays on its page.
const CROSS_PAGES: bool = false;

/// The `why` a Tab-driven selection carries into the trace.
const WHY_TAB: &str = "tab-object"; // ui-text-exempt: diagnostic token, never displayed

/// **The ring for the space `anchor` is standing in**, in paint order.
///
/// `anchor` is the current selection on this page, or `None` when there is
/// none.
#[must_use]
pub fn stops(
    provider: &ObjectModelProvider,
    page_index: usize,
    pick: PickFilter,
    anchor: Option<TargetId>,
) -> Vec<TargetId> {
    let model = provider.page_objects();
    // `None` means the provider cannot classify, and the filter's contract for
    // that is to let the candidate through — see `object_class`.
    let allowed = |target: TargetId| {
        provider
            .object_class(page_index, target)
            .is_none_or(|class| pick.allows(class))
    };
    // Asked of form XObjects only. The predicate is a bounds-against-page
    // ratio, so asking it of an ordinary object would drop a full-page
    // background rectangle — which a click selects perfectly well — and would
    // put one trace line on the wire per object on the sheet.
    let worth = |target: TargetId| {
        provider.object_class(page_index, target) != Some(PickClass::FormXObject)
            || provider.container_is_worth_selecting(page_index, target)
    };
    let leaves_of = |containment: &[pdfcer_core::object::ObjId]| -> Vec<TargetId> {
        model
            .leaves
            .iter()
            .enumerate()
            .filter(|(_, leaf)| leaf.containment == containment)
            .map(|(k, _)| TargetId::Leaf(k as u64))
            .filter(|target| allowed(*target))
            .collect()
    };

    if let Some(TargetId::Leaf(i)) = anchor
        && let Some(here) = model.leaves.get(usize::try_from(i).unwrap_or(usize::MAX))
    {
        return leaves_of(&here.containment);
    }

    let page_ring: Vec<TargetId> = (0..model.objects.len())
        .map(|k| TargetId::Object(k as u64))
        .filter(|target| allowed(*target))
        .filter(|target| worth(*target))
        .collect();
    if !page_ring.is_empty() {
        return page_ring;
    }

    // A sheet whose whole body is one page-sized wrapper. Its own paint order
    // offers nothing selectable, so the ring is the wrapper's contents — the
    // same objects a click anywhere on that sheet lands on.
    let Some(first) = model.leaves.iter().find(|leaf| leaf.containment.len() == 1) else {
        return Vec::new();
    };
    leaves_of(&first.containment)
}

/// **Spend this frame's Tab press**, if one was claimed for the object ring.
pub(super) fn advance(
    ctx: &egui::Context,
    page_index: usize,
    provider: Option<&ObjectModelProvider>,
    pick: PickFilter,
    selection: &mut SelectionState,
) {
    let Some(request) = tabnav::take(ctx, tabnav::Scope::Object) else {
        return;
    };
    let Some(provider) = provider else {
        // The page holds focus but this frame has no decomposition — the model
        // is still being built, or the page declined to decompose. Traced
        // rather than ignored: the symptom is a Tab that does nothing, which
        // nobody can debug from outside the process.
        tabnav::discard(ctx);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("tab-object-unmodelled page={page_index}")
        });
        return;
    };

    let anchor = selection
        .entries()
        .first()
        .filter(|entry| entry.page == page_index)
        .map(|entry| entry.object);
    let ring = stops(provider, page_index, pick, anchor);
    if ring.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("tab-object-empty page={page_index}")
        });
        return;
    }

    let next = match anchor.and_then(|a| ring.iter().position(|t| *t == a)) {
        Some(at) => {
            let lens = [(page_index, ring.len())];
            let Some((_, pos)) =
                tabnav::step(&lens, (page_index, at), request.backwards, CROSS_PAGES)
            else {
                return;
            };
            pos
        }
        // Nothing selected, or a selection that is on no ring — an undo removed
        // it, or the filter excludes its class. Either way the press means
        // "start here", and which end it starts from is the direction it came
        // in: a Shift+Tab into a page lands on the last stop, as it does in
        // every program that tabs backwards into a surface.
        None if request.backwards => ring.len() - 1,
        None => 0,
    };
    let Some(target) = ring.get(next).copied() else {
        return;
    };
    selection.select_only(page_index, target, WHY_TAB);
    let stops = ring.len();
    let backwards = request.backwards;
    crate::diag::trace(move || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        let list = if target.is_leaf() { "leaf" } else { "object" };
        format!(
            "tab-object page={page_index} backwards={backwards} stops={stops} \
             to={list}:{}",
            target.raw()
        )
    });
}

#[cfg(test)]
mod tests {
    //! The ring's shape over the engine's own form fixtures.
    //!
    //! `stops` is the whole of what a Tab decides, and it is pure over a
    //! provider and a filter — so the cases below need no egui frame and no
    //! keyboard. What they cannot see is whether `advance` is called at all;
    //! that is `canvas::keys`'s wiring, and the driven check's job.

    use super::*;
    use crate::panels::objects::test_support::engine_fixture;

    /// One page-sized form holding three separate 40 × 40 squares: **one**
    /// page object, **three** leaves.
    const PAGE_SIZED_FORM: &str = "forms-xobject/page-sized-form.pdf";

    /// A provider over page 1, with the page's own device transform — the
    /// same map `build_or_reason` gives the running program, so
    /// `container_is_worth_selecting` is answered against a real page extent
    /// rather than the `None` a from-parts fixture would supply.
    fn provider_over(fixture: &str) -> ObjectModelProvider {
        let bytes = std::fs::read(engine_fixture(fixture)).expect("the fixture is readable");
        let doc = pdfcer_core::document::Document::from_bytes(bytes).expect("the fixture parses");
        let view = doc.view();
        let pages = pdfcer_core::page_tree::pages(&doc).expect("the fixture has a page tree");
        ObjectModelProvider::build_or_reason(&view, &pages[0], 0).expect("the page decomposes")
    }

    /// The fixture really has the shape every case below assumes.
    #[test]
    fn the_fixture_is_one_page_sized_wrapper_over_three_squares() {
        let p = provider_over(PAGE_SIZED_FORM);
        let model = p.page_objects();
        assert_eq!(
            model.objects.len(),
            1,
            "the page paints one thing: the form"
        );
        assert_eq!(
            model.leaves.len(),
            3,
            "and three squares come from inside it"
        );
    }

    /// **THE CASE THE MODULE EXISTS FOR.** With nothing selected on a
    /// sheet whose whole body is one page-sized wrapper, the ring is the
    /// wrapper's contents — not the wrapper.
    #[test]
    fn a_page_sized_wrapper_is_skipped_and_the_ring_is_its_contents() {
        let p = provider_over(PAGE_SIZED_FORM);
        let ring = stops(&p, 0, PickFilter::all(), None);
        assert_eq!(
            ring,
            vec![TargetId::Leaf(0), TargetId::Leaf(1), TargetId::Leaf(2)],
            "three squares, in paint order, and the wrapper is not among them"
        );
    }

    /// A leaf anchor rings the leaves beside it — the siblings inside the
    /// same form, which is *"whatever space I have clicked on"* one level in.
    #[test]
    fn a_selected_leaf_rings_its_own_containment() {
        let p = provider_over(PAGE_SIZED_FORM);
        let ring = stops(&p, 0, PickFilter::all(), Some(TargetId::Leaf(1)));
        assert_eq!(ring.len(), 3, "all three squares share one wrapper");
        assert!(
            ring.contains(&TargetId::Leaf(1)),
            "the anchor is ON its own ring — `advance` steps from a position, \
             so a ring that excluded the current stop could not be stepped"
        );
    }

    /// **The filter narrows the ring, and a filter that excludes
    /// everything empties it rather than falling back to something the next
    /// click could not select.**
    #[test]
    fn a_filter_that_excludes_the_contents_leaves_no_ring() {
        let p = provider_over(PAGE_SIZED_FORM);
        let text_only = PickFilter::all().with(PickClass::Path, false);
        assert!(
            stops(&p, 0, text_only, None).is_empty(),
            "no path is on the table, so there is nothing to tab to"
        );
        assert!(
            stops(&p, 0, text_only, Some(TargetId::Leaf(0))).is_empty(),
            "and the leaf-anchored branch filters too, rather than trusting \
             the anchor to mean the class is allowed"
        );
    }

    /// A query for a page this provider does not answer for returns nothing
    /// selectable rather than the wrong page's objects.
    #[test]
    fn every_stop_indexes_into_this_providers_own_lists() {
        let p = provider_over(PAGE_SIZED_FORM);
        let model = p.page_objects();
        for target in stops(&p, 0, PickFilter::all(), None) {
            match target {
                TargetId::Object(i) => assert!((i as usize) < model.objects.len()),
                TargetId::Leaf(i) => assert!((i as usize) < model.leaves.len()),
            }
        }
    }
}
