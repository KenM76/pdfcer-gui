//! Several annotations selected at once: the primary in `annot`, the rest in
//! `annot_others`, all on the primary's page.
//!
//! The primary is the one every single-annotation surface reads through
//! [`SelectionState::annot`] — grips, Properties, Delete. The others are
//! outlined without grips and are what Align and Distribute add to it.

use pdfcer_core::annot::Annotation;
use pdfcer_core::page_tree::Page;
use pdfcer_core::view::DocumentView;

use super::SelectionState;
use crate::annotselection::AnnotSelection;
use crate::fieldset::BandMerge;

impl SelectionState {
    /// Every selected annotation: the others in the order they joined, then
    /// the primary. Empty when no annotation is selected.
    #[must_use]
    pub fn annots(&self) -> Vec<&AnnotSelection> {
        let Some(primary) = self.annot.as_ref() else {
            return Vec::new();
        };
        self.annot_others.iter().chain(Some(primary)).collect()
    }

    /// Add `hit` to the annotation selection, or take it out if it is in.
    /// Removing the primary promotes the most recently added other. A hit on
    /// another page than the selection's starts a new selection: Align acts
    /// on one page.
    pub fn toggle_annot(&mut self, hit: AnnotSelection) {
        let mut all: Vec<AnnotSelection> = self.annots().into_iter().cloned().collect();
        if all.iter().any(|a| a.target.page != hit.target.page) {
            all.clear();
        }
        if let Some(at) = all.iter().position(|a| a.target.id == hit.target.id) {
            all.remove(at);
        } else {
            all.push(hit);
        }
        self.set_annots(all);
    }

    /// Combine a marquee's annotations, all on one page, into the annotation
    /// selection.
    pub fn band_annots(&mut self, hits: Vec<AnnotSelection>, merge: BandMerge) {
        let page = hits.first().map(|h| h.target.page);
        let mut all: Vec<AnnotSelection> = match merge {
            BandMerge::Replace => Vec::new(),
            BandMerge::Add | BandMerge::Subtract => self
                .annots()
                .into_iter()
                .filter(|a| page.is_none_or(|p| a.target.page == p))
                .cloned()
                .collect(),
        };
        match merge {
            BandMerge::Subtract => {
                all.retain(|a| !hits.iter().any(|h| h.target.id == a.target.id));
            }
            BandMerge::Replace | BandMerge::Add => {
                for h in hits {
                    if !all.iter().any(|a| a.target.id == h.target.id) {
                        all.push(h);
                    }
                }
            }
        }
        self.set_annots(all);
    }

    /// Make the last of `all` the primary and the rest the others; an empty
    /// list clears the annotation selection.
    fn set_annots(&mut self, mut all: Vec<AnnotSelection>) {
        match all.pop() {
            Some(primary) => {
                self.select_annot(primary);
                self.annot_others = all;
            }
            None => {
                self.clear_annot();
            }
        }
    }
}

/// Re-read `selected`'s outline, lock flag and angle from `found`, the
/// annotations of `page`. Returns whether it is still on the page.
pub(super) fn refresh(
    view: &DocumentView<'_>,
    page: &Page,
    found: &[Annotation],
    selected: &mut AnnotSelection,
) -> bool {
    let Some(found) = found.iter().find(|a| a.id == Some(selected.target.id)) else {
        return false;
    };
    let Some(rect) = found.rect else {
        return true;
    };
    let Some(outline) =
        crate::canvasmapping::annot_canvas_rect([rect.llx, rect.lly, rect.urx, rect.ury], page)
    else {
        return true;
    };
    selected.outline = outline;
    // The `/F` bit 8 flag is re-read too. A lock applied while the mark is
    // selected must take the grips away on the next frame, not on the next
    // click — `Grabbable`'s annotation arm reads `target.locked` and would
    // otherwise go on offering nine handles the file forbids.
    selected.target.locked = found.flags.locked();
    selected.oriented = crate::annotquad::oriented(view, found)
        .filter(|q| !q.is_upright())
        .and_then(|q| crate::canvasmapping::oriented_canvas_quad(q.corners, page));
    true
}
