//! Content on a hidden layer is not on the canvas, so no click, band or
//! Select All may take it.
//!
//! Contract: [`ObjectModelProvider::hide_layers`] records, for one layer
//! state, every object and form leaf whose `/OC` membership is hidden; the
//! hit tests drop them. A leaf is hidden when its own membership is or when
//! the page object it is painted through is. The engine reports only the
//! innermost `/OC` of nested marked content, so content inside a visible
//! inner group whose outer group is hidden stays selectable.

use std::collections::{BTreeMap, BTreeSet};

use pdfcer_core::object::ObjId;

use super::{ObjectModelProvider, TargetId};

/// The targets one layer state hides, and which state that was.
#[derive(Debug, Default)]
pub(super) struct Unseen {
    generation: Option<u64>,
    targets: BTreeSet<TargetId>,
}

impl ObjectModelProvider {
    /// Whether [`Self::hide_layers`] has run for layer state `generation`.
    #[must_use]
    pub fn hides_for(&self, generation: u64) -> bool {
        self.unseen_lock().generation == Some(generation)
    }

    /// Hide every target whose layer `is_hidden` answers true for, as layer
    /// state `generation`. `is_hidden` is asked once per distinct `/OC`.
    pub fn hide_layers(&self, generation: u64, is_hidden: impl Fn(ObjId) -> bool) {
        let mut answers: BTreeMap<ObjId, bool> = BTreeMap::new();
        let mut hidden = |oc: Option<ObjId>| {
            oc.is_some_and(|oc| *answers.entry(oc).or_insert_with(|| is_hidden(oc)))
        };
        let mut targets = BTreeSet::new();
        let objects: Vec<bool> = self
            .objects
            .objects
            .iter()
            .map(|o| hidden(o.oc()))
            .collect();
        for (i, gone) in objects.iter().enumerate() {
            if *gone {
                targets.insert(TargetId::Object(i as u64));
            }
        }
        for (i, leaf) in self.objects.leaves.iter().enumerate() {
            if objects.get(leaf.paint_order).copied().unwrap_or(false) || hidden(leaf.oc()) {
                targets.insert(TargetId::Leaf(i as u64));
            }
        }
        let count = targets.len();
        *self.unseen_lock() = Unseen {
            generation: Some(generation),
            targets,
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("hidden-targets page={} n={count}", self.page_index)
        });
    }

    /// `hits` without the targets on a hidden layer.
    pub(super) fn visible_only(&self, mut hits: Vec<TargetId>) -> Vec<TargetId> {
        let unseen = self.unseen_lock();
        if !unseen.targets.is_empty() {
            hits.retain(|t| !unseen.targets.contains(t));
        }
        hits
    }

    fn unseen_lock(&self) -> std::sync::MutexGuard<'_, Unseen> {
        self.unseen
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
