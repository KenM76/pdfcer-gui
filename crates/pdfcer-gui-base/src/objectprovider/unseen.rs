//! Content on a hidden layer is not on the canvas, so no click, band or
//! Select All may take it.
//!
//! Contract: [`ObjectModelProvider::hide_layers`] records, for one layer
//! state, every object and form leaf the renderer leaves unpainted: one any
//! of whose enclosing `/OC` memberships (`oc_chain`, outermost first) is
//! hidden. The hit tests drop them. A leaf is hidden too when the page object
//! it is painted through is.

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
            .map(|o| o.oc_chain().iter().any(|&oc| hidden(Some(oc))))
            .collect();
        for (i, gone) in objects.iter().enumerate() {
            if *gone {
                targets.insert(TargetId::Object(i as u64));
            }
        }
        for (i, leaf) in self.objects.leaves.iter().enumerate() {
            if objects.get(leaf.paint_order).copied().unwrap_or(false)
                || leaf.oc_chain().iter().any(|&oc| hidden(Some(oc)))
            {
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

#[cfg(test)]
mod tests {
    use super::TargetId;
    use crate::opendoc::OpenDoc;
    use pdfcer_core::document::Document;
    use pdfcer_core::edit::EditSession;
    use pdfcer_core::object::ObjId;

    const OUTER: ObjId = ObjId::new(4, 0);

    /// One page: a square inside `Inner` inside `Outer`, then a square inside
    /// `Inner` alone.
    fn nested() -> OpenDoc {
        let page = "/OC /Outer BDC /OC /Inner BDC 0 g 10 10 20 20 re f EMC EMC \
                    /OC /Inner BDC 0 g 60 10 20 20 re f EMC";
        let bodies = [
            "<< /Type /Catalog /Pages 2 0 R /OCProperties << /OCGs [4 0 R 5 0 R] \
             /D << /OFF [4 0 R] >> >> >>"
                .to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 40] /Contents 6 0 R \
             /Resources << /Properties << /Outer 4 0 R /Inner 5 0 R >> >> >>"
                .to_string(),
            "<< /Type /OCG /Name (Outer) >>".to_string(),
            "<< /Type /OCG /Name (Inner) >>".to_string(),
            format!("<< /Length {} >>\nstream\n{page}\nendstream", page.len()),
        ];
        let mut buf = b"%PDF-1.7\n".to_vec();
        let mut offsets = Vec::new();
        for (i, body) in bodies.iter().enumerate() {
            offsets.push(buf.len());
            buf.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
        }
        let xref = buf.len();
        buf.extend_from_slice(
            format!("xref\n0 {}\n0000000000 65535 f \n", bodies.len() + 1).as_bytes(),
        );
        for o in offsets {
            buf.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
        }
        buf.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
                bodies.len() + 1
            )
            .as_bytes(),
        );
        let doc = Document::from_bytes(buf).expect("the page parses");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
        OpenDoc::new("nested-layers.pdf".into(), EditSession::new(doc), pages)
    }

    /// A shown inner layer inside a hidden outer one is not painted, so it is
    /// not picked; the inner layer alone still is.
    #[test]
    fn content_under_a_hidden_outer_layer_is_not_picked() {
        let mut doc = nested();
        doc.set_hidden_layers(std::iter::once(OUTER).collect());
        let provider = doc.page_objects().expect("the page decomposes");
        assert_eq!(provider.page_objects().objects.len(), 2, "two squares");
        let left = provider.visible_only(vec![TargetId::Object(0), TargetId::Object(1)]);
        assert_eq!(left, vec![TargetId::Object(1)]);
    }
}
