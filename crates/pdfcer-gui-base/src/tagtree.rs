//! # `tagtree` — the document's structure tree, for the Tags panel and the
//! on-page outline of the element picked in it.
//!
//! The tree is read once per edit epoch, through [`OpenDoc::tag_tree`], and
//! only when something asks: reading it extracts the whole document's text,
//! which an untagged file never pays because [`has_structure_tree`] reads the
//! catalog alone. The pick is a set of boxes in default user space, keyed by
//! the epoch it was read at, so an edit hides an outline that may no longer
//! describe the page.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/tagtree.md`.

use std::cell::{Cell, Ref, RefCell};

use crate::settings::SettingsExt;
use pdfcer_core::page_tree::Rect;
use pdfcer_core::structure_tree::{
    StructKid, StructureTree, has_structure_tree, read_structure_tree,
};

use crate::opendoc::OpenDoc;

/// What reading the structure tree found.
#[derive(Debug)]
pub enum TagRead {
    /// The catalog has no `/StructTreeRoot`.
    Untagged,
    /// The page tree could not be walked; the engine's message.
    Failed(String),
    /// The tree, with the engine's diagnostics.
    Read(Box<StructureTree>),
}

/// The structure tree of the edit epoch it was read at.
#[derive(Debug, Default)]
pub struct TagCache {
    built_for: Cell<Option<u64>>,
    read: RefCell<Option<TagRead>>,
}

/// The element picked in the Tags panel, and where it is drawn.
#[derive(Debug, Clone, PartialEq)]
pub struct TagPick {
    /// Index into [`StructureTree::elements`].
    pub element: usize,
    /// The edit epoch the boxes were read at.
    pub epoch: u64,
    /// One box per page it spans, in page order, in default user space.
    pub boxes: Vec<(usize, Rect)>,
}

impl TagPick {
    /// The boxes on `page`, or none when an edit has happened since.
    pub fn boxes_on(&self, page: usize, epoch: u64) -> impl Iterator<Item = Rect> + '_ {
        let live = self.epoch == epoch;
        self.boxes
            .iter()
            .filter(move |(p, _)| live && *p == page)
            .map(|(_, r)| *r)
    }
}

/// Why an element has no box, from the facts the engine joined to its kids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoBox {
    /// Neither it nor any descendant names content.
    NoContent,
    /// Content names a page that could not be resolved.
    PageUnresolved,
    /// Marked content names an MCID no page declares.
    McidNotFound,
    /// Its content paints no text, and no annotation rectangle: the outline
    /// in this build follows text runs and `/OBJR` rectangles only.
    NothingTextual,
}

/// Why `index` has no box. Walks kids only forward (`c > i`), the engine's
/// own termination rule, so a cyclic `/K` ends.
#[must_use]
pub fn no_box_reason(tree: &StructureTree, index: usize) -> NoBox {
    let (mut content, mut unresolved, mut undeclared) = (false, false, false);
    let mut stack = vec![index];
    while let Some(i) = stack.pop() {
        let Some(e) = tree.elements.get(i) else {
            continue;
        };
        for kid in &e.kids {
            match kid {
                StructKid::Element(c) if *c > i => stack.push(*c),
                StructKid::MarkedContent {
                    page_index,
                    declared,
                    ..
                } => {
                    content = true;
                    unresolved |= page_index.is_none();
                    undeclared |= page_index.is_some() && !declared;
                }
                StructKid::Object { page_index, .. } => {
                    content = true;
                    unresolved |= page_index.is_none();
                }
                _ => {}
            }
        }
    }
    match (content, unresolved, undeclared) {
        (false, ..) => NoBox::NoContent,
        (_, true, _) => NoBox::PageUnresolved,
        (_, _, true) => NoBox::McidNotFound,
        _ => NoBox::NothingTextual,
    }
}

impl OpenDoc {
    /// The structure tree, read on first use after each edit.
    pub fn tag_tree(&self) -> Ref<'_, TagRead> {
        if self.tags.built_for.get() != Some(self.edit_epoch) {
            self.tags.built_for.set(Some(self.edit_epoch));
            let view = self.session.view();
            let read = if has_structure_tree(&view) {
                match read_structure_tree(&view, &self.settings.extract_options()) {
                    Ok(tree) => TagRead::Read(Box::new(tree)),
                    Err(e) => TagRead::Failed(e.to_string()),
                }
            } else {
                TagRead::Untagged
            };
            *self.tags.read.borrow_mut() = Some(read);
        }
        Ref::map(self.tags.read.borrow(), |slot| {
            slot.as_ref().expect("just built for this epoch") // ui-text-exempt: panic message, never displayed
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pick_draws_on_its_own_pages_until_an_edit() {
        let r = Rect::from_corners(0.0, 0.0, 10.0, 10.0);
        let pick = TagPick {
            element: 3,
            epoch: 2,
            boxes: vec![(0, r), (1, r)],
        };
        assert_eq!(pick.boxes_on(1, 2).count(), 1);
        assert_eq!(pick.boxes_on(2, 2).count(), 0);
        assert_eq!(pick.boxes_on(1, 3).count(), 0, "an edit hides it");
    }
}
