//! The merge rules every multi-selection shares: a click toggles one item, a
//! marquee replaces, adds or subtracts its catch. Form fields (`fieldset`)
//! and annotations (`selectionstate::annotset`) both keep their selection as
//! an ordered list whose last item is the primary, and both go through here.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/multiset.md`.

/// How a marquee's catch combines with what is already selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandMerge {
    /// The catch becomes the selection.
    Replace,
    /// The catch joins it.
    Add,
    /// The catch leaves it.
    Subtract,
}

/// Remove `item` from `all` if `same` finds it, append it otherwise. The
/// last item is the primary, so removing the primary promotes the most
/// recently added one.
pub fn toggle<T>(all: &mut Vec<T>, item: T, same: impl Fn(&T, &T) -> bool) {
    if let Some(at) = all.iter().position(|a| same(a, &item)) {
        all.remove(at);
    } else {
        all.push(item);
    }
}

/// Combine a marquee's `hits` with `all`, the current selection, under
/// `merge`. Order is kept: existing items first, new hits after.
#[must_use]
pub fn band<T>(
    mut all: Vec<T>,
    hits: Vec<T>,
    merge: BandMerge,
    same: impl Fn(&T, &T) -> bool,
) -> Vec<T> {
    match merge {
        BandMerge::Replace => all.clear(),
        BandMerge::Add | BandMerge::Subtract => {}
    }
    match merge {
        BandMerge::Subtract => all.retain(|a| !hits.iter().any(|h| same(a, h))),
        BandMerge::Replace | BandMerge::Add => {
            for h in hits {
                if !all.iter().any(|a| same(a, &h)) {
                    all.push(h);
                }
            }
        }
    }
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_appends_then_removes() {
        let mut all = vec![1, 2];
        toggle(&mut all, 3, |a, b| a == b);
        assert_eq!(all, [1, 2, 3]);
        toggle(&mut all, 3, |a, b| a == b);
        assert_eq!(all, [1, 2]);
    }

    #[test]
    fn band_replaces_adds_and_subtracts_without_repeats() {
        let eq = |a: &i32, b: &i32| a == b;
        assert_eq!(band(vec![1], vec![2, 2, 3], BandMerge::Replace, eq), [2, 3]);
        assert_eq!(band(vec![1, 2], vec![2, 3], BandMerge::Add, eq), [1, 2, 3]);
        assert_eq!(
            band(vec![1, 2, 3], vec![2], BandMerge::Subtract, eq),
            [1, 3]
        );
    }
}
