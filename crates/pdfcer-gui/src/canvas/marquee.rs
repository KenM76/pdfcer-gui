//! # `canvas::marquee` — **what a rubber-band takes, and why the direction
//! decides it**
//!
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

use pdfcer_core::vector::{FormMarquee, MarqueeMode};

use crate::app::state::OpenDoc;
use crate::canvas::selection::{SelectionLevel, SelectionState};
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
///
/// `targets` is `None` when the page has no decomposition, in which case the
/// band selects nothing. That is not an error and must not clear the selection
/// by a different route than a genuine empty band does: `SelectionState::marquee`
/// with an empty slice is the one path, and it is reached the same way either
/// way.
/// **What a band does to the selection it lands on** — `OPERATOR_REQUESTS.md`
/// O104.
///
/// A band subtracts as well as adds, because of the operator's report *"I can't
/// unselect things once I have selected them"*. On a CAD sheet with hundreds of
/// overlapping strokes, taking one object back out by clicking it precisely is
/// often not practical; a band is how the work is actually done.
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

/// Every variant must appear in [`Combine::ALL`].
///
/// The arms are counted by the compiler, so adding a variant to [`Combine`]
/// stops this crate compiling and names the variant it is missing. `ALL` is
/// the declaration immediately below, which is where the answer goes.
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
    ///
    /// The one authoritative list. Anything that has to visit each combine
    /// walks this rather than writing its own copy, because a private copy
    /// cannot go red when the set grows and the guard above only watches this
    /// one.
    pub const ALL: [Self; 3] = [Self::Replace, Self::Add, Self::Subtract];

    /// The word the diagnostic trace spells this with.
    ///
    /// A word rather than the derived `Debug` form: a check reads this field by
    /// equality, and a `Debug` spelling is a rename away from breaking one
    /// silently.
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

/// **A released band, described once**: where it was drawn, which way, and what
/// it is to do with what it reached.
///
/// The three travel together through every arm of the release and are decided
/// in one place, by the gesture. Bundling them is what keeps a rung's entry
/// point from growing an argument list nobody can read a call site of, and it
/// is also the seam that makes the two rungs' bands provably the same gesture:
/// the chunk arm and the object arm are handed the same value, so neither can
/// come to disagree about direction or about which modifier means what.
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
///
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
    ctx: &egui::Context,
    doc: &OpenDoc,
    targets: Option<&dyn crate::canvas::target::CanvasTargetProvider>,
    page_index: usize,
    band: Band,
    selection: &mut SelectionState,
) {
    // ★★★ Inside a text block, the band takes CHUNKS — O215 ask 4. See
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
///
/// `OPERATOR_REQUESTS.md` O215 ask 4 asks for the usual three multi-select
/// gestures on chunks. Shift-click and Ctrl-click were the click path's; this
/// is the band, and without it a rubber-band drawn across four lines of a note
/// ascended out of the block and selected the whole block instead — because
/// [`SelectionState::marquee`] resolves to the Object rung by construction.
///
/// # Why this is not a contradiction of that function's reasoning
///
/// Its argument is that *a region of the page contains objects*, and that
/// "every subpath of some other object this box happens to cover" has no
/// sensible reading. Both still hold. This is the different case those words
/// were not about: the operator has **entered** one object, and that object's
/// chunks are **drawn as boxes on the canvas**. The band is then a region over
/// a set of visible rectangles belonging to the one thing being worked on,
/// which is what every isolation mode — Illustrator's, Inkscape's,
/// PowerPoint's — does with a band drawn inside a group.
///
/// ⇒ The gate is `chunks::boxed`, so the rung is offered exactly where the
/// boxes are. A band can never take a unit the operator cannot see.
///
/// # What a band that reaches no chunk does
///
/// With **no modifier**, nothing here claims it and the object-rung band runs:
/// a plain band over empty paper clears, which is the convention and is what
/// leaving a text block ought to feel like. With **Shift or Ctrl held** it is
/// claimed and changes nothing — a modifier says *refine what I have*, and a
/// refinement that reached nothing must not instead throw the set away.
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
///
/// Split out with no borrows in it so the three arms can be tested directly.
/// The arms are [`Combine`]'s and mean there exactly what they mean at the
/// Object rung — the rung changes what a hit *is*, never what a modifier
/// *does*.
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
    // ★ `Include` — the container comes back alongside its leaves. The long
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

    /// **A plain band at the chunk rung replaces**, and what was held before it
    /// has no say.
    ///
    /// The arm a driven check cannot see on its own: a build that added instead
    /// of replacing still ends up with the band's chunks selected, so the drag
    /// that follows still moves several and the check still passes — while an
    /// operator banding a second group of lines silently keeps the first.
    #[test]
    fn a_plain_chunk_band_replaces_what_was_held() {
        assert_eq!(
            combined(&[7, 9], &[0, 1, 2], Combine::Replace),
            vec![0, 1, 2]
        );
        assert_eq!(combined(&[], &[3], Combine::Replace), vec![3]);
    }

    /// **Shift adds**, and the result is ascending and holds no chunk twice.
    ///
    /// The overlap is the point: a band dragged across lines the operator
    /// already had must not select them twice, because every Part-rung verb
    /// loops the entries and a duplicate would move one line the distance twice.
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
    ///
    /// The same answer Ctrl-clicking the last held chunk gives, and the
    /// alternative — silently keeping one, or falling back to the whole block —
    /// would be the program overruling an explicit gesture.
    #[test]
    fn subtracting_everything_leaves_nothing() {
        assert!(combined(&[0, 1], &[0, 1], Combine::Subtract).is_empty());
    }

    /// The three trace words are distinct.
    ///
    /// A check reads `combine=` by equality, so two arms spelled the same would
    /// collapse *the band added* and *the band replaced* into one answer — the
    /// pair the tests above exist to tell apart.
    #[test]
    fn every_combine_label_is_its_own_word() {
        let mut words: Vec<&str> = Combine::ALL.iter().map(|c| c.label()).collect();
        let before = words.len();
        words.sort_unstable();
        words.dedup();
        assert_eq!(words.len(), before);
    }
}
