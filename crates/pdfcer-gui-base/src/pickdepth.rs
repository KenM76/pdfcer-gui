//! # `pickdepth` — how deep the last click reached, and how deep it could have
//!
//! Two numbers, remembered from the last selecting click: **which** candidate
//! was taken, and **how many** there were under the pointer.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/pickdepth.md`.

/// The `egui::Memory` slot the pair lives in.
const KEY: &str = "pdfcer-canvas-depth"; // ui-text-exempt: internal memory id, never displayed

/// Which candidate the last selecting click took, how many there were, and
/// **which object it was about**.
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
    /// A [`TargetId`](crate::canvastarget::TargetId) rather than a bare
    /// index, and that is load-bearing rather than tidy: a page has **two**
    /// index spaces now — the page's own objects and the leaves inside its
    /// form XObjects — and `7` occurs in both. A bare number would let a depth
    /// measured for the seventh leaf be claimed by a selection of the seventh
    /// page object, which is exactly the mis-attribution this field exists to
    /// prevent.
    object: crate::canvastarget::TargetId,
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
    object: crate::canvastarget::TargetId,
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

/// **What the last click chose, but only if it was about THIS selection.**
#[must_use]
pub fn taken(
    ctx: &egui::Context,
    page: usize,
    object: crate::canvastarget::TargetId,
) -> Option<Depth> {
    ctx.data_mut(|d| d.get_temp::<Depth>(egui::Id::new(KEY)))
        .filter(|d| d.of > 1 && d.page == page && d.object == object)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvastarget::TargetId;

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

    /// **A selection this depth was not measured for claims nothing** —
    /// and nobody had to remember to clear it.
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
