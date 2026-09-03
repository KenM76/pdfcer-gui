//! # `canvas::marquee` — **what a rubber-band takes, and why the direction
//! decides it**
//!
//! Two pure functions and their tests, split out of [`crate::canvas::interact`]
//! on 2026-09-02 under R2 when that file crossed the 1,500-line ceiling. The
//! seam is a real subject rather than a line count: everything here answers one
//! question — *given a band and the direction it was dragged, which targets does
//! it select?* — and none of it needs a frame, a `Ui` or a document.
//!
//! ## ★★★ The operator's report, `OPERATOR_REQUESTS.md` O88
//!
//! > *"I can't box select the tables in the left or right top corners using the
//! > mouse — it only picks up the lines of each table, so I can't drag the
//! > entire thing and move it somewhere else, or cut/copy and paste it
//! > elsewhere."*
//!
//! It was never a hit test that excluded text. Both tables sit hard against the
//! sheet edge, and this shell asked for `MarqueeMode::Enclosed` everywhere — an
//! object counts only if the band **completely surrounds** it. To surround a
//! table at the sheet edge the band has to start **outside the page**, and at
//! fit zoom there is barely a pixel of margin to start in. So the only band that
//! can actually be drawn is one *inside* the table, which surrounds a few short
//! rules and nothing else.
//!
//! ⇒ **"It only picks up the lines" is what an enclosing band returns when it
//! cannot be drawn big enough.**
//!
//! ## The remedy is a convention, not an invention
//!
//! AutoCAD's direction-sensitive band, which SolidWorks drawings use too:
//!
//! | drag | AutoCAD's name | takes |
//! |---|---|---|
//! | left → right | a **window** | only what it completely surrounds |
//! | right → left | a **crossing window** | anything it touches |
//!
//! No modifier key, nothing new to learn, and it is the behaviour a
//! drawing-office hand already has. The standing instruction is to use the
//! conventional interaction rather than invent one, and the two alternatives are
//! both inventions here: Illustrator touches always, Inkscape puts touch on
//! `Alt`. The direction rule is the drawing-office one, and this is a drawing
//! program.
//!
//! ★ The enclosing band's answer does **not** change. `Enclosed` remains what a
//! left-to-right drag does and remains the right default on a dense sheet —
//! decision 011's reasoning is untouched. What was wrong was that it was the
//! only answer available.
//!
//! ## ★★ The half that was found by a failing test rather than by thinking
//!
//! See [`without_page_wrappers`]. A crossing band touches a page-sized form
//! XObject on **every** drag, so on a wrapped drawing every crossing selection
//! would have quietly included the whole sheet — and the operator's next gesture
//! moves it. Under `Enclosed` that could not happen, which is why it is new.

use pdfcer_core::vector::MarqueeMode;

use crate::canvas::target::TargetId;

/// Which rule a band dragged in this direction selects by.
///
/// One function rather than an `if` at the call site, so the convention is
/// stated once. A second spelling of it somewhere else is how a band that
/// *paints* as a crossing window comes to *select* as a window.
#[must_use]
pub const fn mode_for(crossing: bool) -> MarqueeMode {
    if crossing {
        MarqueeMode::Touched
    } else {
        MarqueeMode::Enclosed
    }
}

/// **Drop the page's own wrapper from a crossing selection.**
///
/// # ★★★ Why this exists, and it was measured rather than anticipated
///
/// The first cut of the direction-sensitive band failed
/// `a_marquee_encloses_objects_inside_a_form` with `[Object(0), Leaf(1)]` where
/// only the leaf was wanted. That test's fixture is a page-sized form XObject
/// with squares inside it — the shape a CAD exporter produces, and the shape
/// `ncored-benchmark-cad-drawing.pdf` has.
///
/// A crossing band **touches** a page-sized wrapper wherever it is drawn. So
/// without this, every right-to-left drag on a wrapped drawing would silently
/// include the whole sheet in the selection, and the operator's next gesture —
/// a move, a delete, a cut — would act on all of it.
///
/// ★★ Under `Enclosed` this could not happen and that is exactly why it is new:
/// a band that *surrounds* a page-sized form has to surround the page, which
/// cannot be drawn. Touching one is unavoidable.
///
/// # The rule is the shell's existing one
///
/// `CanvasTargetProvider::container_is_worth_selecting` already answers *"is
/// this container really just the sheet?"* — measured against the page extent at
/// `COVERS_EVERYTHING`, with its own argument about why the threshold is
/// generous — and `canvas::smart` already applies it to the **click** ladder.
/// Reusing it here is the consistency argument the provider's own
/// `hit_test_rect` makes at length: two gestures that both mean *"select this"*
/// must not disagree about what is selectable.
///
/// Nothing new is measured here and no second threshold exists.
///
/// # ★ Only a hit that CONTAINS another hit is tested
///
/// A lone path covering the whole sheet — a drawing border, which is on almost
/// every sheet this program is for — is **not** a container and must stay
/// selectable. Asking the container question about it would drop it, which
/// would be a second defect wearing the first one's fix.
///
/// So the container set is derived from the hits themselves: an id is a wrapper
/// only if some *other* hit in the same band reports it as its containing form.
/// Nothing is asked of the provider that the click path does not already ask.
///
/// # Parameters, and why they are closures
///
/// `container_of` and `worth_selecting` are the two provider queries, passed as
/// functions so that this rule is testable without a provider, a page or a
/// decomposition. The rule is the thing worth pinning; the queries are already
/// under test where they live.
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
///
/// The whole of the marquee arm's body, lifted out of
/// [`crate::canvas::interact`] on 2026-09-02 so that file could come back under
/// the 1,500-line R2 ceiling — and it belongs here anyway: every line of it is
/// about what a band takes, which is this module's one subject.
///
/// `targets` is `None` when the page has no decomposition, in which case the
/// band selects nothing. That is not an error and must not clear the selection
/// by a different route than a genuine empty band does: `SelectionState::marquee`
/// with an empty slice is the one path, and it is reached the same way either
/// way.
/// **What a band does to the selection it lands on** — `OPERATOR_REQUESTS.md`
/// O104.
///
/// Until 2026-09-03 a band could only replace or add, which is half of the
/// operator's report *"I can't unselect things once I have selected them"*. On
/// a CAD sheet with hundreds of overlapping strokes, taking one object back out
/// by clicking it precisely is often not practical; a band is how the work is
/// actually done, and ours had no way to remove.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combine {
    /// No modifier: the band's hits become the selection.
    Replace,
    /// Shift: the hits are added to what is already selected.
    Add,
    /// Ctrl: the hits are taken OUT of what is already selected.
    ///
    /// ★ Ctrl rather than Shift, even though AutoCAD spells subtract with
    /// Shift. Shift-adds is already shipped here and in every vector editor
    /// this shell's operators also use, so re-pointing it would break a gesture
    /// they have — and it would contradict `Ctrl+click`, which now means
    /// "toggle out". One modifier, one meaning: **Ctrl takes things out of a
    /// selection wherever you use it.**
    Subtract,
}

/// Which [`Combine`] a pair of modifiers asks for.
///
/// Shift wins when both are held, because adding is the non-destructive answer
/// and a band held with every modifier at once is an operator who has not
/// decided yet.
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

/// **Everything a released selection band does**, so `canvas::interact` holds
/// the wiring and this module holds the behaviour.
///
/// Extracted 2026-09-03 when O104's Ctrl handling pushed `interact.rs` past
/// R2's 1,500-line ceiling — the same seam `canvas::previews` was found at on
/// 2026-08-30, and found the same way.
///
/// ★★★ **THE DIRECTION DECIDES WHAT THE BAND TAKES** (O88): left to right
/// encloses, right to left touches — AutoCAD's window / crossing-window rule.
/// This module's header carries the operator's report, why the fix is geometric
/// rather than about hit tests, and the page-wrapper hazard a crossing band
/// introduces.
///
/// ★★ And [`Combine`] decides what it does to what was already selected. The
/// two are independent: *what the band reaches* and *what it then does with
/// it*, which is why they are separate arguments rather than one flag.
pub fn on_release(
    targets: Option<&dyn crate::canvas::target::CanvasTargetProvider>,
    page_index: usize,
    rect: egui::Rect,
    crossing: bool,
    combine: Combine,
    selection: &mut crate::canvas::selection::SelectionState,
) {
    select_with(targets, page_index, rect, crossing, combine, selection);
    crate::canvas::trace::selection_event(
        selection,
        // ui-text-exempt: a diagnostic slot name, never displayed.
        "pv.marquee",
        combine != Combine::Replace,
    );
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
    let mut hits = targets.map_or_else(Vec::new, |t| t.hit_test_rect(page_index, rect, mode));
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
    // ★★★ **THE KINDS, NOT ONLY THE COUNT** — added 2026-09-02, and the
    // reason is a check whose oracle turned out to be an assumption.
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
    // ★★ His complaint is about a KIND being missing — *"it only picks up the
    // lines of each table"* — and a count cannot express that at any
    // threshold. One path and one text is a pass; nine paths and no text is the
    // defect, and the count ranks them the wrong way round.
    //
    // ★ `object_class` is the provider's own classifier, the same one the pick
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
        // ★ The MODE is on the line, not only the hit count. A crossing band
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

    /// **★★ Ctrl subtracts, Shift adds, neither replaces** —
    /// `OPERATOR_REQUESTS.md` O104.
    ///
    /// Pinned as a table rather than three separate tests because the value of
    /// the rule is that the three answers are *distinct*: a modifier scheme
    /// where two of them coincide is one an operator cannot use to mean two
    /// different things.
    #[test]
    fn the_three_band_modes_are_distinct() {
        assert_eq!(mode(false, false), Combine::Replace);
        assert_eq!(mode(true, false), Combine::Add);
        assert_eq!(mode(false, true), Combine::Subtract);
    }

    /// **★ Shift wins when both are held.**
    ///
    /// Adding is the non-destructive answer, and a band dragged with every
    /// modifier at once is an operator who has not decided yet — so the
    /// tie-break is the one that cannot lose them a selection they built.
    #[test]
    fn holding_both_modifiers_adds_rather_than_subtracts() {
        assert_eq!(mode(true, true), Combine::Add);
    }
    use super::*;

    /// The two directions map to the two engine modes, and not to one of them
    /// twice.
    ///
    /// ★ Trivial, and pinned anyway: the whole feature is one boolean choosing
    /// between two enum variants, and a build in which both arms returned
    /// `Enclosed` would behave exactly as this shell did before the change —
    /// which is to say it would look like the feature had never been merged,
    /// with every other test still green.
    #[test]
    fn the_direction_chooses_the_mode() {
        assert_eq!(mode_for(false), MarqueeMode::Enclosed);
        assert_eq!(mode_for(true), MarqueeMode::Touched);
        assert_ne!(mode_for(false), mode_for(true));
    }

    /// ★★★ **A page-sized wrapper is dropped; its contents are kept.**
    ///
    /// The case the failing test surfaced. `Object(0)` wraps `Leaf(1)`, and
    /// `Object(0)` covers the page — so a crossing band takes the leaf and not
    /// the sheet.
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

    /// ★★ **A container that is NOT the whole sheet survives.**
    ///
    /// The falsifying half, and the one that stops this from being "drop every
    /// container". A 320×220 form on a 400×300 page is a real object an
    /// operator selects on purpose — this project has a driven check that
    /// demands exactly that on the click path, and a crossing band must not
    /// disagree with it.
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

    /// ★★★ **A lone page-covering object that contains nothing is KEPT.**
    ///
    /// The drawing border, and the reason the container set is derived from the
    /// hits rather than by asking `worth_selecting` of everything. A border
    /// covers the sheet and is not worth selecting *as a container* — but it is
    /// not a container at all, it is a path the operator may well be reaching
    /// for, and dropping it would be a second defect wearing the first one's
    /// fix.
    ///
    /// This is the assertion that would fail against the obvious simpler
    /// implementation (`hits.retain(worth_selecting)`), which is why it is
    /// here.
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

    /// ★ Order is preserved.
    ///
    /// The selection's paint order is what the ladder and the Objects panel
    /// both read, and a filter that reordered would change which object a
    /// subsequent double-click descends into.
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
}
