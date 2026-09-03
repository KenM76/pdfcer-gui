//! # `canvas::depth` — how deep the last click reached, and how deep it could have
//!
//! Two numbers, remembered from the last selecting click: **which** candidate
//! was taken, and **how many** there were under the pointer.
//!
//! ## Why they are worth remembering at all
//!
//! The operator, 2026-08-26: *"when I click on one of the objects all I get is
//! the page selected."*
//!
//! Two things were true. The engine did not enter form XObjects, so most of
//! what he could see was not in the object model — that is filed as an engine
//! request and is not this module's business. And **this shell threw away the
//! rest of the list**: `hit_test_all` returns every candidate under a point,
//! front to back, and the pick called `.find()` on it. Anything underneath
//! anything was unreachable.
//!
//! `Alt`+click now walks the stack ([`crate::canvas::clicking`]'s
//! `CycleCursor`). But a cycling gesture with no readout is a gesture nobody
//! discovers: the operator clicks, gets the wrong object, and has no way to
//! learn that there were four more behind it. **The count is what turns a
//! mystery into a diagnosis** — *"1 of 5 here"* says both that this is not the
//! only answer and that there is a way to ask for the others.
//!
//! ## Why a memory slot rather than a field on `OpenDoc`
//!
//! Because it is a fact about the last **gesture**, not about the document. It
//! does not survive a document change, it must not be persisted, and nothing
//! that re-derives the selection should re-derive it — a selection restored
//! after an edit was not clicked for, and claiming *"1 of 5"* about it would be
//! describing a click that never happened.
//!
//! `egui::Memory` is where per-frame and per-gesture UI state lives in this
//! shell, and it drops on its own when the context does.
//!
//! ## ★ It is deliberately NOT part of the selection
//!
//! `SelectionState` is the answer to *"what is being worked on"*, and it is
//! read by the overlay, by every transform verb and by two panels. A depth is
//! the answer to *"how did we get here"*, which is a different question with a
//! different lifetime — it is stale the instant the selection changes by any
//! route other than a click.
//!
//! Putting it on the selection would make every consumer carry a field none of
//! them can use, and would make a restored or programmatic selection have to
//! decide what to claim about a click that did not occur. Keeping it apart lets
//! the honest answer be *nothing*: [`taken`] returns `None` and the status line
//! says nothing rather than something untrue.

/// The `egui::Memory` slot the pair lives in.
const KEY: &str = "pdfcer-canvas-depth"; // ui-text-exempt: internal memory id, never displayed

/// Which candidate the last selecting click took, how many there were, and
/// **which object it was about**.
///
/// The third field is what makes this self-invalidating — see [`taken`].
/// ★ **No `Default`, deliberately.** A defaulted `Depth` would have to name
/// some target, and every number in `TargetId`'s two index spaces is a real,
/// addressable object — so the default would be a claim about `objects[0]`
/// rather than an absence. `taken` already answers `None` for "nothing to
/// say", which is the honest shape and the only one any caller uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Depth {
    /// How many candidates the click skipped. `0` for a plain click.
    pub taken: usize,
    /// How many candidates were under the pointer, after the pick filter.
    pub of: usize,
    /// The page the click was on.
    page: usize,
    /// The target the click selected.
    ///
    /// ★ A [`TargetId`](crate::canvas::target::TargetId) rather than a bare
    /// index, and that is load-bearing rather than tidy: a page has **two**
    /// index spaces now — the page's own objects and the leaves inside its
    /// form XObjects — and `7` occurs in both. A bare number would let a depth
    /// measured for the seventh leaf be claimed by a selection of the seventh
    /// page object, which is exactly the mis-attribution this field exists to
    /// prevent.
    object: crate::canvas::target::TargetId,
}

/// Record what the last selecting click chose, and about what.
///
/// Called from the one place a click resolves to an object. A click that hit
/// nothing records `of = 0`, which [`taken`] reports as nothing to say.
pub fn remember(
    ctx: &egui::Context,
    taken: usize,
    of: usize,
    page: usize,
    object: crate::canvas::target::TargetId,
) {
    ctx.data_mut(|d| {
        d.insert_temp(
            egui::Id::new(KEY),
            Depth {
                taken,
                of,
                page,
                object,
            },
        );
    });
}

/// ★★ **What the last click chose, but only if it was about THIS selection.**
///
/// # Why it validates rather than trusting a caller to forget
///
/// A depth describes a *gesture*, and the selection can change by four routes
/// that are not a click: an edit re-resolving it, Escape, a row click in the
/// Objects panel, and a placement selecting what it just made. Attributing
/// *"1 of 5 here"* to any of those describes a click that did not happen — a
/// small lie, told confidently, on the surface the operator is now relying on
/// to explain the program to them.
///
/// The first design had a `forget()` for callers to call. That is the shape of
/// a guard that is correct until somebody adds a fifth route — and this module
/// exists **because** a value with no owner drifted from the thing it described.
/// Repeating that mistake one file later would have been hard to defend.
///
/// So the record carries what it is about, and answering compares. A selection
/// this depth was not measured for gets `None` without anyone having to
/// remember anything.
///
/// Also `None` when fewer than two candidates were under the pointer: there is
/// no stack, and *"1 of 1"* is noise on a bar that has to earn every character.
#[must_use]
pub fn taken(
    ctx: &egui::Context,
    page: usize,
    object: crate::canvas::target::TargetId,
) -> Option<Depth> {
    ctx.data_mut(|d| d.get_temp::<Depth>(egui::Id::new(KEY)))
        .filter(|d| d.of > 1 && d.page == page && d.object == object)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::target::TargetId;

    /// A lone candidate is not a stack, and saying *"1 of 1"* would be noise.
    #[test]
    fn a_single_candidate_reports_nothing() {
        let ctx = egui::Context::default();
        remember(&ctx, 0, 1, 0, TargetId::Object(7));
        assert_eq!(taken(&ctx, 0, TargetId::Object(7)), None);
        remember(&ctx, 0, 0, 0, TargetId::Object(7));
        assert_eq!(
            taken(&ctx, 0, TargetId::Object(7)),
            None,
            "a click that hit nothing has nothing to say"
        );
    }

    /// A stack reports which one was taken and how many there were.
    #[test]
    fn a_stack_reports_which_of_how_many() {
        let ctx = egui::Context::default();
        remember(&ctx, 2, 5, 3, TargetId::Object(11));
        let got = taken(&ctx, 3, TargetId::Object(11)).expect("the depth is about this selection");
        assert_eq!((got.taken, got.of), (2, 5));
    }

    /// ★★★ **A selection this depth was not measured for claims nothing** —
    /// and nobody had to remember to clear it.
    ///
    /// The four routes that change a selection without a click are an edit
    /// re-resolving it, Escape, an Objects-panel row click, and a placement.
    /// Each lands on a different object, or on the same object on a different
    /// page, and either way the record stops matching. That is the whole
    /// mechanism: no `forget()` to call, so no fifth route to forget to add it
    /// to.
    #[test]
    fn a_depth_measured_for_another_selection_is_not_claimed() {
        let ctx = egui::Context::default();
        remember(&ctx, 3, 9, 0, TargetId::Object(4));
        assert!(
            taken(&ctx, 0, TargetId::Object(4)).is_some(),
            "about this one, so it speaks"
        );
        assert_eq!(
            taken(&ctx, 0, TargetId::Object(5)),
            None,
            "a different object on the same page"
        );
        assert_eq!(
            taken(&ctx, 1, TargetId::Object(4)),
            None,
            "the same index on a different page"
        );
    }
}
