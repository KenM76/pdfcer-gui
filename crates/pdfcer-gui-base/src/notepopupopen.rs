//! # `notepopupopen` — which pop-ups are showing, and who decided
//!
//! One subject: **the open/closed state of every note pop-up**, and the rule
//! that the file gets the first word and the operator gets the last.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/notepopupopen.md`.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use pdfcer_core::object::ObjId;

/// The operator's explicit open/closed decisions for one document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Overrides(BTreeMap<ObjId, bool>);

impl Overrides {
    /// **Is this note's pop-up showing?**
    #[must_use]
    pub fn is_open(&self, id: ObjId, authored: bool) -> bool {
        self.0.get(&id).copied().unwrap_or(authored)
    }

    /// Record a decision.
    pub fn set(&mut self, id: ObjId, open: bool) {
        self.0.insert(id, open);
    }

    /// **Has the operator spoken about this note?**
    #[must_use]
    pub fn touched(&self, id: ObjId) -> bool {
        self.0.contains_key(&id)
    }

    /// How many decisions have been recorded — for the trace, and for the test
    /// that asserts a click is recorded rather than merely appearing to work.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether nothing has been decided. Present because clippy requires it
    /// beside [`Self::len`], and it is the honest name for "the document's own
    /// state is the whole answer right now".
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// The egui id this document's overrides are stored under.
fn key(path: &Path) -> egui::Id {
    egui::Id::new(("pdfcer-note-popup-open", path)) // ui-text-exempt: internal widget id, never displayed
}

/// Read this document's overrides.
#[must_use]
pub fn load(ctx: &egui::Context, path: &Path) -> Arc<Overrides> {
    ctx.data_mut(|d| d.get_temp::<Arc<Overrides>>(key(path)).unwrap_or_default())
}

/// Record one decision, in place.
pub fn set(ctx: &egui::Context, path: &Path, id: ObjId, open: bool) {
    ctx.data_mut(|d| {
        let key = key(path);
        let mut overrides = (*d.get_temp::<Arc<Overrides>>(key).unwrap_or_default()).clone();
        overrides.set(id, open);
        d.insert_temp(key, Arc::new(overrides));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(num: u32) -> ObjId {
        ObjId::new(num, 0)
    }

    /// **An untouched note reads its state out of the file.**
    #[test]
    fn an_untouched_note_takes_the_files_word() {
        let overrides = Overrides::default();
        assert!(overrides.is_open(id(7), true));
        assert!(!overrides.is_open(id(7), false));
    }

    /// The operator outranks the file, in **both** directions.
    #[test]
    fn an_override_wins_either_way() {
        let mut overrides = Overrides::default();
        overrides.set(id(7), false);
        assert!(!overrides.is_open(id(7), true));
        overrides.set(id(7), true);
        assert!(overrides.is_open(id(7), false));
    }

    /// One note's decision says nothing about another's. Trivial of a map and
    /// asserted anyway, because the failure — one click opening every pop-up
    /// on the sheet — is the exact shape of a store keyed by something coarser
    /// than the annotation.
    #[test]
    fn a_decision_is_per_note() {
        let mut overrides = Overrides::default();
        overrides.set(id(7), true);
        assert!(overrides.is_open(id(7), false));
        assert!(!overrides.is_open(id(8), false));
    }
}
