//! # `canvas::marquee` — **what a rubber-band takes, and why the direction
//! decides it**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/marquee.md`.

use pdfcer_core::vector::{FormMarquee, MarqueeMode};

use crate::app::state::OpenDoc;
use crate::canvas::selection::{SelectionLevel, SelectionState};
use crate::canvas::target::TargetId;

/// Which rule a band dragged in this direction selects by.
#[must_use]
pub const fn mode_for(crossing: bool) -> MarqueeMode {
    if crossing {
        MarqueeMode::Touched
    } else {
        MarqueeMode::Enclosed
    }
}

/// **Drop the page's own wrapper from a crossing selection.**
#[must_use]
pub fn without_page_wrappers(
    hits: Vec<TargetId>,
    container_of: impl Fn(TargetId) -> Option<TargetId>,
    worth_selecting: impl Fn(TargetId) -> bool,
) -> Vec<TargetId> {
    let wrappers: std::collections::BTreeSet<TargetId> =
        hits.iter().filter_map(|h| container_of(*h)).collect();
    hits.into_iter()
        .filter(|h| !wrappers.contains(h) || worth_selecting(*h))
        .collect()
}

/// **Resolve a completed select-band into a new selection.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combine {
    /// No modifier: the band's hits become the selection.
    Replace,
    /// Shift: the hits are added to what is already selected.
    Add,
    /// Ctrl: the hits are taken OUT of what is already selected.
    ///
    /// Ctrl rather than Shift, even though AutoCAD spells subtract with
    /// Shift. Shift-adds is already shipped here and in every vector editor
    /// this shell's operators also use, so re-pointing it would break a gesture
    /// they have — and it would contradict `Ctrl+click`, which now means
    /// "toggle out". One modifier, one meaning: **Ctrl takes things out of a
    /// selection wherever you use it.**
    Subtract,
}

/// Every variant must appear in [`Combine::ALL`].
const _: () = {
    const fn _combine_is_listed_in_all(combine: Combine) {
        match combine {
            Combine::Replace | Combine::Add | Combine::Subtract => (),
        }
    }
};

impl Combine {
    /// **Every combine there is**, in the order a modifier reaches for them:
    /// no modifier, Shift, Ctrl.
    pub const ALL: [Self; 3] = [Self::Replace, Self::Add, Self::Subtract];

    /// The word the diagnostic trace spells this with.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            // ui-text-exempt: diagnostic trace vocabulary, never displayed.
            Self::Replace => "replace",
            // ui-text-exempt: diagnostic trace vocabulary, never displayed.
            Self::Add => "add",
            // ui-text-exempt: diagnostic trace vocabulary, never displayed.
            Self::Subtract => "subtract",
        }
    }
}

/// Which [`Combine`] a pair of modifiers asks for.
#[must_use]
pub const fn mode(shift: bool, ctrl: bool) -> Combine {
    if shift {
        Combine::Add
    } else if ctrl {
        Combine::Subtract
    } else {
        Combine::Replace
    }
}

/// **A released band, described once**: where it was drawn, which way, and what
/// it is to do with what it reached.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Band {
    /// The rectangle swept, in **canvas** space — the space the chunk boxes
    /// and `CanvasTargetProvider::bounds` are already in.
    pub rect: egui::Rect,
    /// Right to left, so the band takes whatever it **touches**. Left to right
    /// encloses. See this function's own header for the operator's report.
    pub crossing: bool,
    /// What to do with what was reached.
    pub combine: Combine,
}

/// **Everything a released selection band does**, so `canvas::interact` holds
/// the wiring and this module holds the behaviour.
pub fn on_release(
    ctx: &egui::Context,
    doc: &OpenDoc,
    targets: Option<&dyn crate::canvas::target::CanvasTargetProvider>,
    page_index: usize,
    band: Band,
    selection: &mut SelectionState,
) {
    // Inside a text block, the band takes CHUNKS — O215 ask 4. See
    // [`take_chunks`] for the whole of that decision, including what it does
    // when the band reaches none.
    if take_chunks(ctx, doc, page_index, band, selection) {
        return;
    }
    select_with(
        targets,
        page_index,
        band.rect,
        band.crossing,
        band.combine,
        selection,
    );
    crate::canvas::trace::selection_event(
        selection,
        // ui-text-exempt: a diagnostic slot name, never displayed.
        "pv.marquee",
        band.combine != Combine::Replace,
    );
}

/// **A band released inside a text block takes that block's chunks**, and
/// whether it did.
fn take_chunks(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page_index: usize,
    band: Band,
    selection: &mut SelectionState,
) -> bool {
    let Band {
        rect,
        crossing,
        combine,
    } = band;
    if selection.level() != SelectionLevel::Part {
        return false;
    }
    let Some(entry) = selection.entered_object() else {
        return false;
    };
    if entry.page != page_index || !crate::canvas::chunks::boxed(ctx, doc, entry.object) {
        return false;
    }
    let reached = crate::canvas::chunks::within(doc, entry.object, rect, crossing);
    if reached.is_empty() && combine == Combine::Replace {
        return false;
    }
    let held = selection.selected_parts_on(page_index, entry.object);
    let parts = combined(&held, &reached, combine);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // `reached` and `kept` are both on the line because they answer
        // different failures: a band that reached nothing is an aim or a
        // geometry fault, and a band that reached three while keeping one is
        // the combining arm.
        format!(
            "marquee-parts page={page_index} object={} mode={} reached={} kept={} combine={}",
            entry.object.raw(),
            if crossing { "touched" } else { "enclosed" },
            reached.len(),
            parts.len(),
            combine.label()
        )
    });
    // ui-text-exempt: the `via=` word on a diagnostic line, never displayed.
    selection.select_parts(page_index, entry.object, &parts, "band");
    true
}

/// What the band leaves selected: `held` combined with `reached` under
/// `combine`, ascending and unique.
fn combined(held: &[usize], reached: &[usize], combine: Combine) -> Vec<usize> {
    let mut parts: Vec<usize> = match combine {
        Combine::Replace => reached.to_vec(),
        Combine::Add => held.iter().chain(reached.iter()).copied().collect(),
        Combine::Subtract => held
            .iter()
            .copied()
            .filter(|p| !reached.contains(p))
            .collect(),
    };
    parts.sort_unstable();
    parts.dedup();
    parts
}

/// [`select_with`] with the pre-O104 signature: `shift` means add.
///
/// Kept because several callers and every existing test say `shift`, and the
/// two-value question they are asking is still a real one.
pub fn select(
    targets: Option<&dyn crate::canvas::target::CanvasTargetProvider>,
    page_index: usize,
    rect: egui::Rect,
    crossing: bool,
    shift: bool,
    selection: &mut crate::canvas::selection::SelectionState,
) {
    select_with(
        targets,
        page_index,
        rect,
        crossing,
        mode(shift, false),
        selection,
    );
}

/// See [`select`], plus [`Combine`] for what the band does to what was already
/// selected.
pub fn select_with(
    targets: Option<&dyn crate::canvas::target::CanvasTargetProvider>,
    page_index: usize,
    rect: egui::Rect,
    crossing: bool,
    combine: Combine,
    selection: &mut crate::canvas::selection::SelectionState,
) {
    let mode = mode_for(crossing);
    // `Include` — the container comes back alongside its leaves. The long
    // argument is on the live provider's `hit_test_rect`; the short one is
    // that a leaf is not an edit operand in this shell and the form is, so a
    // band that returned leaves alone would select things nothing can move.
    // The page-sized-wrapper case that would make this obnoxious is dropped
    // two lines down, by this shell's own rule rather than by the hit test.
    let mut hits = targets.map_or_else(Vec::new, |t| {
        t.hit_test_rect(page_index, rect, mode, FormMarquee::Include)
    });
    if let (true, Some(t)) = (crossing, targets) {
        hits = without_page_wrappers(
            hits,
            |h| t.containing_form(page_index, h),
            |h| t.container_is_worth_selecting(page_index, h),
        );
    }
    match combine {
        Combine::Subtract => selection.marquee_remove(page_index, &hits),
        Combine::Add => selection.marquee(page_index, &hits, true),
        Combine::Replace => selection.marquee(page_index, &hits, false),
    }
    //
    // `a_marquee_over_a_table_takes_its_text_as_well_as_its_lines` asserted a
    // COUNT, reasoning that *"a table's rules are one path object per line and
    // its words are one text object per cell, so a band over this table should
    // return well into double figures"*. Measured on the operator's own sheet:
    // that whole drawing — two tables, a title block, an isometric view,
    // dozens of labels — decomposes into **25 objects, 19 paths and 6 text**.
    // A band returning 3 is a substantial fraction of the sheet, and the
    // threshold was rejecting a correct result.
    //
    // His complaint is about a KIND being missing — *"it only picks up the
    // lines of each table"* — and a count cannot express that at any
    // threshold. One path and one text is a pass; nine paths and no text is the
    // defect, and the count ranks them the wrong way round.
    //
    // `object_class` is the provider's own classifier, the same one the pick
    // filter reads, so what this line reports and what a filter would exclude
    // cannot disagree.
    let (mut paths, mut text, mut other) = (0usize, 0usize, 0usize);
    for hit in &hits {
        match targets.and_then(|t| t.object_class(page_index, *hit)) {
            Some(crate::canvas::pick::PickClass::Path) => paths += 1,
            Some(crate::canvas::pick::PickClass::Text) => text += 1,
            _ => other += 1,
        }
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // The MODE is on the line, not only the hit count. A crossing band
        // and a window band over the same rect differ only in what they
        // return, so a count alone cannot tell a working crossing window from
        // a window that happened to enclose everything -- which is exactly the
        // pair a driven check has to distinguish.
        format!(
            "marquee-mode crossing={crossing} mode={} hits={} paths={paths} text={text} other={other}",
            if crossing { "touched" } else { "enclosed" },
            hits.len()
        )
    });
}

#[cfg(test)]
mod tests {

    /// **Ctrl subtracts, Shift adds, neither replaces** —
    /// `OPERATOR_REQUESTS.md` O104.
    #[test]
    fn the_three_band_modes_are_distinct() {
        assert_eq!(mode(false, false), Combine::Replace);
        assert_eq!(mode(true, false), Combine::Add);
        assert_eq!(mode(false, true), Combine::Subtract);
    }

    /// **Shift wins when both are held.**
    #[test]
    fn holding_both_modifiers_adds_rather_than_subtracts() {
        assert_eq!(mode(true, true), Combine::Add);
    }
    use super::*;

    /// The two directions map to the two engine modes, and not to one of them
    /// twice.
    #[test]
    fn the_direction_chooses_the_mode() {
        assert_eq!(mode_for(false), MarqueeMode::Enclosed);
        assert_eq!(mode_for(true), MarqueeMode::Touched);
        assert_ne!(mode_for(false), mode_for(true));
    }

    /// **A page-sized wrapper is dropped; its contents are kept.**
    #[test]
    fn a_wrapper_that_is_the_whole_sheet_is_dropped() {
        let hits = vec![TargetId::Object(0), TargetId::Leaf(1)];
        let kept = without_page_wrappers(
            hits,
            |h| (h == TargetId::Leaf(1)).then_some(TargetId::Object(0)),
            // `Object(0)` covers the page, so it is not worth selecting.
            |h| h != TargetId::Object(0),
        );
        assert_eq!(kept, vec![TargetId::Leaf(1)]);
    }

    /// **A container that is NOT the whole sheet survives.**
    #[test]
    fn a_container_worth_selecting_is_kept() {
        let hits = vec![TargetId::Object(0), TargetId::Leaf(1)];
        let kept = without_page_wrappers(
            hits.clone(),
            |h| (h == TargetId::Leaf(1)).then_some(TargetId::Object(0)),
            // This one IS worth selecting.
            |_| true,
        );
        assert_eq!(kept, hits);
    }

    /// **A lone page-covering object that contains nothing is KEPT.**
    #[test]
    fn a_page_covering_object_that_contains_nothing_is_kept() {
        let hits = vec![TargetId::Object(0), TargetId::Object(1)];
        let kept = without_page_wrappers(
            hits.clone(),
            // Nothing has a container.
            |_| None,
            // …and both would be judged "not worth selecting" if asked.
            |_| false,
        );
        assert_eq!(
            kept, hits,
            "an object that contains none of the other hits is not a wrapper, whatever its size"
        );
    }

    /// An empty band keeps being empty, and a band with no forms is untouched.
    #[test]
    fn the_ordinary_cases_pass_through_unchanged() {
        assert!(without_page_wrappers(Vec::new(), |_| None, |_| true).is_empty());
        let plain = vec![TargetId::Object(3), TargetId::Object(7)];
        assert_eq!(
            without_page_wrappers(plain.clone(), |_| None, |_| false),
            plain,
            "a page with no forms must be unaffected, whatever the size rule would say"
        );
    }

    /// Order is preserved.
    #[test]
    fn the_surviving_order_is_the_order_it_arrived_in() {
        let hits = vec![
            TargetId::Object(5),
            TargetId::Object(0),
            TargetId::Leaf(2),
            TargetId::Object(9),
        ];
        let kept = without_page_wrappers(
            hits,
            |h| (h == TargetId::Leaf(2)).then_some(TargetId::Object(0)),
            |h| h != TargetId::Object(0),
        );
        assert_eq!(
            kept,
            vec![TargetId::Object(5), TargetId::Leaf(2), TargetId::Object(9)]
        );
    }

    /// **A plain band at the chunk rung replaces**, and what was held before it
    /// has no say.
    #[test]
    fn a_plain_chunk_band_replaces_what_was_held() {
        assert_eq!(
            combined(&[7, 9], &[0, 1, 2], Combine::Replace),
            vec![0, 1, 2]
        );
        assert_eq!(combined(&[], &[3], Combine::Replace), vec![3]);
    }

    /// **Shift adds**, and the result is ascending and holds no chunk twice.
    #[test]
    fn a_shift_chunk_band_adds_without_duplicating() {
        assert_eq!(combined(&[2, 0], &[1, 2], Combine::Add), vec![0, 1, 2]);
    }

    /// **Ctrl takes the band's chunks out** and leaves the rest alone.
    #[test]
    fn a_ctrl_chunk_band_subtracts_only_what_it_reached() {
        assert_eq!(
            combined(&[0, 1, 2, 3], &[1, 3], Combine::Subtract),
            vec![0, 2]
        );
        assert_eq!(
            combined(&[0, 1], &[4, 5], Combine::Subtract),
            vec![0, 1],
            "a subtracting band that reached none of the held chunks changes nothing"
        );
    }

    /// **A subtracting band over the whole set empties it**, deliberately.
    #[test]
    fn subtracting_everything_leaves_nothing() {
        assert!(combined(&[0, 1], &[0, 1], Combine::Subtract).is_empty());
    }

    /// The three trace words are distinct.
    #[test]
    fn every_combine_label_is_its_own_word() {
        let mut words: Vec<&str> = Combine::ALL.iter().map(|c| c.label()).collect();
        let before = words.len();
        words.sort_unstable();
        words.dedup();
        assert_eq!(words.len(), before);
    }
}
