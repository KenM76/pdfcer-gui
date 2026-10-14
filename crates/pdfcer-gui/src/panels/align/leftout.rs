//! # `panels::align::leftout` — what a selection holds that Align does not move
//!
//! Align and Distribute act on the current page at the Object rung. Anything
//! else selected is named off-canvas instead of dropped in silence: entries on
//! other pages, a part whose own drawing is selected too (it rides with the
//! drawing), and a selection made at the Part rung. Parts of different placed
//! drawings are grouped here so each drawing gets one engine verb.

use pdfcer_core::vector::PageObjects;

use crate::app::state::OpenDoc;
use crate::canvas::selection::SelectionLevel;
use crate::text::panels::align as t;

/// The selected things an Align or Distribute leaves where they are.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LeftOut {
    /// Parts of a placed drawing whose drawing is selected too.
    pub carried: usize,
    /// Content, form fields and annotations selected on other pages.
    pub elsewhere: usize,
}

impl LeftOut {
    /// One sentence per kind left out, for the panel and the status line.
    #[must_use]
    pub fn notes(self) -> Vec<String> {
        let mut out = Vec::new();
        if self.carried > 0 {
            out.push(t::parts_ride_with_drawing(self.carried));
        }
        if self.elsewhere > 0 {
            out.push(t::left_on_other_pages(self.elsewhere));
        }
        out
    }
}

/// `leaves` without those inside one of the selected page `objects` (moving
/// the drawing moves them already), and how many were dropped.
#[must_use]
pub fn uncarried(
    objects: &PageObjects,
    page_objects: &[usize],
    leaves: Vec<usize>,
) -> (Vec<usize>, usize) {
    let total = leaves.len();
    let kept: Vec<usize> = leaves
        .into_iter()
        .filter(|&i| {
            objects
                .leaves
                .get(i)
                .is_none_or(|leaf| !page_objects.contains(&leaf.paint_order))
        })
        .collect();
    let carried = total - kept.len();
    (kept, carried)
}

/// Leaf `moves` grouped by the placed drawing each leaf is directly inside,
/// in first-seen order: the engine moves the parts of one drawing per call.
#[must_use]
pub fn by_drawing(
    objects: &PageObjects,
    moves: Vec<(usize, f64, f64)>,
) -> Vec<Vec<(usize, f64, f64)>> {
    let mut keys = Vec::new();
    let mut groups: Vec<Vec<(usize, f64, f64)>> = Vec::new();
    for m in moves {
        let key = objects
            .leaves
            .get(m.0)
            .map(|l| (l.paint_order, l.containment.clone()));
        if let Some(at) = keys.iter().position(|k| *k == key) {
            groups[at].push(m);
        } else {
            keys.push(key);
            groups.push(vec![m]);
        }
    }
    groups
}

/// How many selected things are on a page other than `page`.
#[must_use]
pub fn elsewhere(doc: &OpenDoc, page: usize) -> usize {
    let content = if doc.selection.level() == SelectionLevel::Object {
        doc.selection
            .entries()
            .iter()
            .filter(|e| e.page != page)
            .count()
    } else {
        0
    };
    let fields = doc
        .field_set
        .all(doc.selected_field.as_ref())
        .into_iter()
        .filter(|f| f.page != page)
        .count();
    let annots = doc
        .selection
        .annots()
        .into_iter()
        .filter(|a| a.target.page != page)
        .count();
    content + fields + annots
}

/// Why nothing on the current page was measured, as a trace reason and the
/// sentence the operator reads.
#[must_use]
pub fn unmeasured(doc: &OpenDoc) -> (&'static str, String) {
    if doc.selection.level() == SelectionLevel::Part && !doc.selection.is_empty() {
        return ("part-rung", t::parts_rung().to_owned()); // ui-text-exempt: diagnostic trace token
    }
    let n = elsewhere(doc, doc.view.page_index);
    if n > 0 {
        return ("other-pages", t::all_on_other_pages(n)); // ui-text-exempt: diagnostic trace token
    }
    ("nothing-selected", t::nothing_selected().to_owned()) // ui-text-exempt: diagnostic trace token
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::vector::Matrix;

    /// `fixtures/parts-beside-objects.pdf`: a page rectangle, drawing Fm0
    /// holding two parts, drawing Fm1 holding one.
    fn fixture() -> PageObjects {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parts-beside-objects.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
        pdfcer_core::vector::decompose_page(&doc.view(), &pages[0], Matrix::IDENTITY)
            .expect("the page decomposes")
    }

    #[test]
    fn a_part_whose_drawing_is_selected_is_carried() {
        let objects = fixture();
        assert_eq!(objects.leaves.len(), 3);
        let drawing = objects.leaves[0].paint_order;
        let inside: Vec<usize> = (0..3)
            .filter(|&i| objects.leaves[i].paint_order == drawing)
            .collect();
        let (kept, carried) = uncarried(&objects, &[drawing], vec![0, 1, 2]);
        assert_eq!(carried, inside.len());
        assert!(kept.iter().all(|i| !inside.contains(i)), "{kept:?}");
        assert_eq!(kept.len(), 3 - inside.len());
    }

    #[test]
    fn parts_of_two_drawings_are_two_groups_in_first_seen_order() {
        let objects = fixture();
        let moves: Vec<(usize, f64, f64)> = (0..3).rev().map(|i| (i, 1.0, 0.0)).collect();
        let groups = by_drawing(&objects, moves);
        assert_eq!(groups.len(), 2, "{groups:?}");
        assert_eq!(groups[0][0].0, 2, "the first-seen part leads");
        for group in &groups {
            let first = &objects.leaves[group[0].0];
            assert!(group.iter().all(|m| {
                let leaf = &objects.leaves[m.0];
                leaf.paint_order == first.paint_order && leaf.containment == first.containment
            }));
        }
    }
}
