//! # `fieldset` — the form fields selected **beside** the primary one
//!
//! `OpenDoc::selected_field` stays the one field every single-field surface
//! reads (properties, grips, Delete, Copy). This set holds the others a
//! Shift/Ctrl-click or a marquee added, so Align and Distribute can act on
//! several fields while nothing that reads the primary has to change.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/fieldset.md`.

use crate::docidentity::SelectedField;
pub use crate::multiset::BandMerge;

/// **The extra fields of a multi-field selection.**
///
/// Valid only while [`Self::anchor`] equals the document's primary field:
/// any other writer of `selected_field` (a plain click, delete, paste, a
/// rename) replaces the primary without knowing this set exists, and that
/// replacement is exactly what invalidates it. [`Self::all`] enforces this, so
/// no writer has to remember to clear the set.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FieldSet {
    /// The primary field this set was built against.
    anchor: Option<SelectedField>,
    /// The other selected fields, in the order they were added. Never holds
    /// the anchor.
    others: Vec<SelectedField>,
}

impl FieldSet {
    /// Every selected field — the others in the order they were added, then
    /// the primary. Empty when there is no primary; the primary alone when the
    /// set was built against a different one.
    #[must_use]
    pub fn all(&self, primary: Option<&SelectedField>) -> Vec<SelectedField> {
        let Some(primary) = primary else {
            return Vec::new();
        };
        let mut out = if self.anchor.as_ref() == Some(primary) {
            self.others.clone()
        } else {
            Vec::new()
        };
        out.push(primary.clone());
        out
    }

    /// Whether `field` is one of the selected fields.
    #[must_use]
    pub fn contains(&self, primary: Option<&SelectedField>, field: &SelectedField) -> bool {
        self.all(primary).contains(field)
    }

    /// Add `field` if absent, remove it if present; returns the new primary.
    ///
    /// Removing the primary promotes the most recently added other field, so
    /// the properties panel keeps showing a field that is still selected.
    pub fn toggle(
        &mut self,
        primary: Option<&SelectedField>,
        field: SelectedField,
    ) -> Option<SelectedField> {
        let mut all = self.all(primary);
        crate::multiset::toggle(&mut all, field, PartialEq::eq);
        self.rebuild(all)
    }

    /// Combine a marquee's `fields` into the selection; returns the new
    /// primary.
    pub fn band(
        &mut self,
        primary: Option<&SelectedField>,
        fields: Vec<SelectedField>,
        merge: BandMerge,
    ) -> Option<SelectedField> {
        let all = crate::multiset::band(self.all(primary), fields, merge, PartialEq::eq);
        self.rebuild(all)
    }

    /// Forget every other field.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Make the last of `all` the primary and the rest the others.
    fn rebuild(&mut self, mut all: Vec<SelectedField>) -> Option<SelectedField> {
        let primary = all.pop();
        self.anchor = primary.clone();
        self.others = all;
        primary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(name: &str) -> SelectedField {
        SelectedField {
            field: name.to_owned(),
            widget: 0,
            page: 0,
        }
    }

    #[test]
    fn toggle_adds_then_removes_and_promotes() {
        let mut set = FieldSet::default();
        let a = f("a");
        let p = set.toggle(Some(&a), f("b"));
        assert_eq!(p, Some(f("b")));
        assert_eq!(set.all(p.as_ref()), vec![f("a"), f("b")]);
        let p = set.toggle(p.as_ref(), f("b"));
        assert_eq!(p, Some(f("a")));
        assert_eq!(set.all(p.as_ref()), vec![f("a")]);
    }

    #[test]
    fn a_replaced_primary_invalidates_the_others() {
        let mut set = FieldSet::default();
        let p = set.toggle(Some(&f("a")), f("b"));
        assert_eq!(set.all(p.as_ref()).len(), 2);
        assert_eq!(set.all(Some(&f("c"))), vec![f("c")]);
        assert!(set.all(None).is_empty());
    }

    #[test]
    fn band_merges() {
        let mut set = FieldSet::default();
        let p = set.band(Some(&f("a")), vec![f("b"), f("c")], BandMerge::Add);
        assert_eq!(set.all(p.as_ref()), vec![f("a"), f("b"), f("c")]);
        let p = set.band(p.as_ref(), vec![f("b")], BandMerge::Subtract);
        assert_eq!(set.all(p.as_ref()), vec![f("a"), f("c")]);
        let p = set.band(p.as_ref(), vec![f("d")], BandMerge::Replace);
        assert_eq!(set.all(p.as_ref()), vec![f("d")]);
        let p = set.band(p.as_ref(), vec![f("d")], BandMerge::Subtract);
        assert_eq!(p, None);
    }
}
