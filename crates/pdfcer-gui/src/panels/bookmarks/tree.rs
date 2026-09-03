//! # `panels::bookmarks::tree` — the two questions this panel asks of an
//! outline, in the one place they can be tested
//!
//! Split out of [`super::add`] on 2026-08-28, when [`super::edit`] arrived and
//! needed both of them. Before that they were private helpers of the add row;
//! promoting them to a shared module is what stops the delete surface growing a
//! second, subtly different depth-first walk — which is exactly how two callers
//! come to disagree about what "the selected bookmark" means.
//!
//! ## ★★ Generic over the tree, because `OutlineItem` is `#[non_exhaustive]`
//!
//! This crate **cannot construct a `pdfcer_core::outline::OutlineItem`**. A
//! recursion written directly over one is therefore a recursion no unit test in
//! this crate can reach — and the recursion is the only part of this module
//! with anything to get wrong.
//!
//! Both walks here are split in two: a one-line wrapper that names the real
//! type, and a generic worker that takes *the two things a tree is* — how to
//! read a node's identity, and how to read its children. The worker can be
//! exercised against a tree built here in a test.
//!
//! That is the fourth remedy in `D:/dev/rag/rust/`'s `#[non_exhaustive]`
//! finding — restructure so the logic does not touch the unconstructible type —
//! and it is the third time in this codebase that the constraint pushed toward
//! the better shape rather than merely around it (`dialogs::insert_image`'s
//! arithmetic and `add`'s original `find_in` were the first two).
//!
//! ## Why depth-first, and why the order is load-bearing
//!
//! A breadth-first walk would return a shallower item carrying a duplicate id
//! before a deeper one. `ObjId`s are unique in a well-formed document — and an
//! outline that made them not so is exactly the malformed case
//! `read_outline`'s cycle-breaking exists for, which means it is a case that
//! reaches this code rather than one that cannot.
//!
//! ## ★ What [`descendants`] counts, and what it deliberately does not
//!
//! It counts **the nodes below a node in the tree the panel drew**. It does
//! *not* read `/Count`, and the distinction is the whole §12.3.3 trap the
//! engine warned this shell about:
//!
//! | | root `/Outlines` | an item |
//! |---|---|---|
//! | `/Count` counts | all visible items, including the top level | visible **descendants**, excluding itself |
//! | sign | cannot be negative | **positive = open, negative = closed** |
//!
//! A **closed** item contributes exactly **1** to its ancestors however large
//! its subtree, so `/Count` on a collapsed heading is not the number of things
//! a delete would take. `declared_count` is carried on `OutlineItem`
//! *"verbatim … Do not use this to size anything"*, in core's own words, and
//! this module obeys that. Walking the children the reader already resolved
//! gives the number the operator needs, which is *how many bookmarks go if I
//! press this*.
//!
//! It is still the **shell's** number rather than the engine's, and the two are
//! allowed to differ: `read_outline` gives up part-way on a cycle, on excessive
//! depth, or on exhausting its item budget. See
//! `crate::text::panels::bookmark_deleted` for why the delete therefore reports
//! the engine's count afterwards as well as this one beforehand.

use pdfcer_core::object::ObjId;
use pdfcer_core::outline::OutlineItem;

/// Find an outline item by id, anywhere in the tree.
///
/// A depth-first walk rather than an index, for [`super::BookmarksUi::selected`]'s
/// reason: an id survives an edit and a position does not. The engine hit that
/// in its own CLI — *"the indices shift after every add … I got this wrong
/// myself while driving the command and nested something two levels deeper than
/// intended, and the output looked entirely plausible."*
///
/// One line, because the recursion — the part with something to get wrong —
/// lives in [`find_in`], which **can** be tested. See the module header.
#[must_use]
pub fn find(items: &[OutlineItem], id: ObjId) -> Option<&OutlineItem> {
    find_in(items, id, |item| item.id, |item| item.children.as_slice())
}

/// Depth-first search of a tree, given the two things a tree is.
///
/// Generic for the reason the module header gives: `OutlineItem` is
/// `#[non_exhaustive]` and cannot be constructed in this crate, so a search
/// written over it directly is a search no test here can reach.
pub fn find_in<'a, T>(
    items: &'a [T],
    id: ObjId,
    id_of: impl Fn(&T) -> ObjId + Copy,
    children: impl Fn(&'a T) -> &'a [T] + Copy,
) -> Option<&'a T> {
    for item in items {
        if id_of(item) == id {
            return Some(item);
        }
        if let Some(found) = find_in(children(item), id, id_of, children) {
            return Some(found);
        }
    }
    None
}

/// How many bookmarks are filed **under** `item`, at every depth.
///
/// Excludes `item` itself, which is the number the delete disclosure wants: the
/// operator can see the row they clicked, and what they cannot see is what else
/// goes with it. `EditSession::delete_outline_item` returns the *inclusive*
/// count afterwards, and `crate::text::panels::bookmark_deleted` subtracts one
/// to speak about the same quantity this does.
///
/// Reads the tree, never `/Count` — see the module header's table for why the
/// two are different numbers on a collapsed item.
#[must_use]
pub fn descendants(item: &OutlineItem) -> usize {
    descendants_in(item, |i| i.children.as_slice())
}

/// Count every node below `node`, given how to read a node's children.
///
/// Generic for [`find_in`]'s reason. The arithmetic is *one per child plus that
/// child's own descendants* — trivial, and trivially wrong in the two ways this
/// module's tests pin: counting the node itself, and counting only one level.
pub fn descendants_in<'a, T>(node: &'a T, children: impl Fn(&'a T) -> &'a [T] + Copy) -> usize {
    children(node)
        .iter()
        .map(|child| 1 + descendants_in(child, children))
        .sum()
}

/// A bookmark's title, or the stand-in for one that has none.
///
/// An untitled bookmark is **legal** — `OutlineItem::title`'s own doc says a
/// file may legitimately carry one — so naming it in a sentence needs the same
/// stand-in the row does rather than an empty gap.
#[must_use]
pub fn display_title(title: &str) -> String {
    if title.trim().is_empty() {
        crate::text::panels::bookmark_untitled().to_owned()
    } else {
        title.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tree this crate CAN build, standing in for the engine's.
    ///
    /// `OutlineItem` is `#[non_exhaustive]`, so the real one cannot be
    /// constructed here — which is why the walks above take accessors rather
    /// than the type. This is the tree they are exercised against.
    struct Node {
        id: ObjId,
        open: bool,
        children: Vec<Node>,
    }

    fn node(num: u32, open: bool, children: Vec<Node>) -> Node {
        Node {
            id: ObjId::new(num, 0),
            open,
            children,
        }
    }

    fn kids(n: &Node) -> &[Node] {
        n.children.as_slice()
    }

    fn find_node(items: &[Node], num: u32) -> Option<&Node> {
        find_in(items, ObjId::new(num, 0), |n| n.id, kids)
    }

    /// ★ A bookmark is found at any depth.
    ///
    /// Depth is the point. The hazard this search replaces is an **index**, and
    /// an index is wrong precisely for the nested case — which is the one the
    /// engine hit in its own CLI: *"I got this wrong myself while driving the
    /// command and nested something two levels deeper than intended, and the
    /// output looked entirely plausible."*
    #[test]
    fn an_item_is_found_at_any_depth() {
        let tree = vec![
            node(1, true, vec![]),
            node(2, true, vec![node(3, false, vec![node(4, true, vec![])])]),
        ];
        assert_eq!(
            find_node(&tree, 4).map(|n| n.id.num),
            Some(4),
            "three levels down"
        );
        assert_eq!(
            find_node(&tree, 1).map(|n| n.id.num),
            Some(1),
            "the first sibling"
        );
        assert!(find_node(&tree, 99).is_none(), "an id that is not there");
    }

    /// ★ A collapsed item is readable, which is what makes the add row's
    /// disclosure possible at all.
    ///
    /// `open` is the shell's read of the **sign** on `/Count` — §12.3.3 defines
    /// no `/Open` key, so the sign is the only carrier — and it is the one
    /// field that decides whether an operator will be able to see what they
    /// just added.
    #[test]
    fn a_collapsed_item_is_visible_to_the_disclosure() {
        let tree = vec![node(2, true, vec![node(3, false, vec![])])];
        assert!(!find_node(&tree, 3).expect("present").open);
        assert!(find_node(&tree, 2).expect("present").open);
    }

    /// ★★ **The subtree count is the whole subtree, and it excludes the node
    /// itself.**
    ///
    /// This is the number the delete disclosure quotes before the press, so
    /// both of its plausible errors are worth pinning:
    ///
    /// * counting the node itself would over-report by one and read as *"and
    ///   the 3 bookmarks under it"* for a parent with two children;
    /// * counting only the immediate children would under-report, and would
    ///   under-report **most** on exactly the deep heading where the operator
    ///   can least see what they are about to lose.
    ///
    /// The fixture is deliberately three levels deep and lopsided so the two
    /// wrong answers (3 and 5) differ from the right one (6) and from each
    /// other. That is the discipline the engine's `Pass 156.0` note asks for:
    /// *"when you assert that A and B differ, check your fixture can tell them
    /// apart"* — its own delete test passed against every sabotage because it
    /// only asserted the list got shorter.
    #[test]
    fn the_subtree_count_is_every_level_and_excludes_the_node() {
        // 1
        //  ├── 2
        //  │    ├── 3
        //  │    └── 4
        //  │         └── 5
        //  └── 6
        //       └── 7
        let tree = node(
            1,
            true,
            vec![
                node(
                    2,
                    true,
                    vec![
                        node(3, true, vec![]),
                        node(4, true, vec![node(5, true, vec![])]),
                    ],
                ),
                node(6, false, vec![node(7, true, vec![])]),
            ],
        );
        assert_eq!(descendants_in(&tree, kids), 6, "every node below the root");
        assert_ne!(
            descendants_in(&tree, kids),
            7,
            "the node itself must NOT be counted"
        );
        assert_ne!(
            descendants_in(&tree, kids),
            2,
            "the immediate children alone are not the answer"
        );

        let branch = find_node(std::slice::from_ref(&tree), 2).expect("present");
        assert_eq!(descendants_in(branch, kids), 3, "3, 4 and 5");
    }

    /// ★ A **collapsed** node's subtree is counted in full.
    ///
    /// The case the §12.3.3 trap would get wrong. `/Count` on a closed item is
    /// negative and its magnitude is not a subtree size — core's own doc says
    /// *"Do not use this to size anything"* — and a closed item contributes
    /// exactly **1** to its ancestors however large it is. So a disclosure
    /// built from `/Count` would tell an operator that removing a collapsed
    /// chapter takes one bookmark when it takes twenty.
    ///
    /// The fixture makes the two answers different: the collapsed node holds
    /// two levels, so a `/Count`-shaped answer (1) and the true one (3) cannot
    /// be confused.
    #[test]
    fn a_collapsed_nodes_subtree_is_counted_in_full() {
        let collapsed = node(
            6,
            false,
            vec![node(
                7,
                false,
                vec![node(8, true, vec![]), node(9, true, vec![])],
            )],
        );
        assert_eq!(descendants_in(&collapsed, kids), 3);
        assert!(!collapsed.open, "the fixture must actually be closed");
    }

    /// A leaf has no descendants, which is the case that gets its own sentence.
    #[test]
    fn a_leaf_has_no_descendants() {
        assert_eq!(descendants_in(&node(1, true, vec![]), kids), 0);
    }

    /// An untitled bookmark is named by the stand-in, not by a gap.
    #[test]
    fn an_untitled_item_is_still_nameable() {
        assert_eq!(
            display_title("   "),
            crate::text::panels::bookmark_untitled()
        );
        assert_eq!(display_title(""), crate::text::panels::bookmark_untitled());
        assert_eq!(display_title("Chapter 3"), "Chapter 3");
    }
}
