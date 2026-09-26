//! # `pagetree` — **does this document's page tree still agree with itself?**
//!
//!
//! > *"I tested deleting pages from a pdf. when I open the document in Acrobat
//! > there are blank pages at the end of the document equalling the number of
//! > pages I deleted."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/pagetree/mod.md`.

use pdfcer_core::document::Document;
use pdfcer_core::graph::ObjectGraph;
use pdfcer_core::object::{ObjId, Object};
use pdfcer_core::page_tree::MAX_TREE_DEPTH;
use std::collections::HashSet;

/// One `/Pages` node whose `/Count` is not the number of leaves beneath it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Disagreement {
    /// The `/Pages` node.
    pub node: ObjId,
    /// What its `/Count` says.
    pub declared: i64,
    /// How many `/Page` leaves are actually reachable beneath it through
    /// `/Kids`.
    pub reachable: usize,
    /// Whether this node is the page-tree **root** — the one the catalog's
    /// `/Pages` names, and the one whose `/Count` a reader takes as the
    /// document's page count.
    ///
    /// Separated from the rest because it is the only node whose disagreement
    /// has a symptom the operator can describe: blank pages at the end of the
    /// document, or pages missing from it. A stale *intermediate* node is just
    /// as much corruption, but which page a reader lands on is then reader-
    /// specific, so the sentence for it does not promise a particular symptom.
    pub root: bool,
}

/// What one walk of a document's page tree found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Audit {
    /// Whether a page-tree root was found and walked at all.
    ///
    /// `false` means the bytes did not parse, the trailer had no `/Root`, the
    /// catalog had no `/Pages`, or what it named was not a dictionary. The
    /// audit then says nothing about the file in either direction, and the save
    /// proceeds — §6.
    pub walked: bool,
    /// Every `/Page` leaf reachable from the root through `/Kids`. The number
    /// this shell, and any `/Kids`-walking reader, will show.
    pub reachable_pages: usize,
    /// The root's own `/Count`, when it has one. The number Acrobat will show.
    pub declared_pages: Option<i64>,
    /// Every node whose declaration does not match its structure, in the order
    /// the walk completed them (deepest first, root last).
    pub disagreements: Vec<Disagreement>,
    /// `/Pages` nodes carrying no `/Count` at all — malformed under §7.7.3.2,
    /// counted rather than refused. See §6.
    pub nodes_without_count: usize,
    /// Nodes reached a second time — a `/Kids` cycle. Counted so that a
    /// `reachable_pages` produced by a truncated walk is recognisable as a
    /// floor rather than a total.
    pub cycles: usize,
    /// Whether the walk hit [`MAX_TREE_DEPTH`] and stopped descending. Same
    /// meaning as `cycles` for the same reason.
    pub too_deep: bool,
    /// **How many levels the deepest leaf hangs below the root**, counting
    /// the root as level 1 and the leaf as a level of its own. A flat page
    /// tree — root plus leaves — is `2`; the three-level fixture is `4`.
    ///
    /// It is a diagnostic, not part of the verdict, and it exists because of a
    /// falsification: a test written against a **flat** fixture passes for a
    /// build with no upward walk at all, and can even print *"the engine has
    /// been fixed"* while it does. This is the number that lets any consumer —
    /// a test, a check, a reader of a trace — assert that the document it is
    /// reasoning about could exhibit the defect in the first place. See
    /// §2 and `app::save::tests`.
    pub depth: usize,
}

impl Audit {
    /// **May these bytes be written?**
    ///
    /// The one question the save path asks. `true` when nothing disagrees —
    /// which includes every document the walk could not run on, deliberately.
    #[must_use]
    pub fn is_consistent(&self) -> bool {
        self.disagreements.is_empty()
    }

    /// The **root's** disagreement, when the root is one of them.
    #[must_use]
    pub fn root_disagreement(&self) -> Option<Disagreement> {
        self.disagreements.iter().copied().find(|d| d.root)
    }
}

/// **Parse the bytes a save is about to write, walk their page tree, and say
/// what the walk found.**
#[must_use]
pub fn audit_saved_bytes(bytes: &[u8]) -> Audit {
    // `bytes.to_vec()` — `Document::from_bytes` takes ownership and the caller
    // still needs the buffer for `std::fs::write`. Measured at §9: a memcpy of
    // a few megabytes is well under the parse it feeds, and the alternative
    // (hand the buffer over and recover it from `Document::bytes()`) loses it
    // entirely on the parse-failure path, which is the one path that must stay
    // recoverable.
    Document::from_bytes(bytes.to_vec())
        .map(|written| audit(&written))
        .unwrap_or_default()
}

/// **Walk `graph`'s page tree and compare every node's `/Count` against the
/// leaves beneath it.**
///
/// Reads only. See the module header for why it reads `/Count` raw, why it does
/// its own recursion rather than using `page_slots`, and why a document it
/// cannot walk is not a refusal.
///
/// Generic over [`ObjectGraph`] so that it runs against a
/// [`pdfcer_core::document::Document`] parsed from the bytes a save is about to
/// write (the production call), against an [`EditSession`]'s graph, and against
/// a hand-rolled graph in a unit test — the same three-way testability
/// `canvas::notepopup::model` gets from the same bound.
///
/// [`EditSession`]: pdfcer_core::edit::EditSession
#[must_use]
pub fn audit<G: ObjectGraph + ?Sized>(graph: &G) -> Audit {
    let mut out = Audit::default();
    let Some(catalog) = graph.catalog_dict() else {
        return out;
    };
    let Some(root) = catalog.get(b"Pages").and_then(Object::as_reference) else {
        // §7.7.3.2 Table 28: `/Pages` "shall be an indirect reference". A
        // direct dictionary here is a file this walk declines to judge rather
        // than one it refuses — the id is what the cycle guard is keyed on and
        // there is none.
        return out;
    };
    if graph.resolved(root).as_dict().is_none() {
        return out;
    }
    out.walked = true;
    let mut visited: HashSet<ObjId> = HashSet::new();
    out.reachable_pages = walk(graph, root, 0, true, &mut visited, &mut out);
    out.declared_pages = count_of(graph, root);
    out
}

/// One node: how many `/Page` leaves are beneath it, recording any
/// disagreement on the way back up.
fn walk<G: ObjectGraph + ?Sized>(
    graph: &G,
    id: ObjId,
    depth: usize,
    root: bool,
    visited: &mut HashSet<ObjId>,
    out: &mut Audit,
) -> usize {
    if !visited.insert(id) {
        out.cycles += 1;
        return 0;
    }
    if depth >= MAX_TREE_DEPTH {
        out.too_deep = true;
        return 0;
    }
    // Levels are counted from 1 at the root, so a leaf at `depth` makes the
    // tree `depth + 1` deep. Recorded for every node, leaf or not, because a
    // `/Pages` node with no kids is still a level.
    out.depth = out.depth.max(depth + 1);
    let Some(dict) = graph.resolved(id).as_dict() else {
        // A `/Kids` entry that resolves to nothing is a dangling reference
        // (§7.3.10 — "shall not be considered an error"), and it is not a page.
        return 0;
    };

    // The engine's own node-kind dispatch (`page_tree::is_pages_node`),
    // reproduced rather than called because it is
    // private. Stated here so a future divergence is visible as a difference
    // between two written rules rather than as a silent one:
    //   /Type /Pages  -> intermediate node
    //   /Type /Page   -> leaf
    //   no /Type      -> intermediate iff it has /Kids
    let type_ = dict.get(b"Type").and_then(Object::as_name);
    let is_node = match type_.map(|n| n.as_bytes().to_vec()) {
        Some(ref t) if t == b"Pages" => true,
        Some(ref t) if t == b"Page" => false,
        _ => dict.contains_key(b"Kids"),
    };
    if !is_node {
        return 1;
    }

    let kids: Vec<ObjId> = dict
        .get(b"Kids")
        .map(|o| graph.resolve(o))
        .and_then(Object::as_array)
        .map(|a| a.iter().filter_map(Object::as_reference).collect())
        .unwrap_or_default();
    let mut reachable = 0;
    for kid in kids {
        reachable += walk(graph, kid, depth + 1, false, visited, out);
    }

    match count_of(graph, id) {
        Some(declared) if usize::try_from(declared).ok() != Some(reachable) => {
            out.disagreements.push(Disagreement {
                node: id,
                declared,
                reachable,
                root,
            });
        }
        Some(_) => {}
        // §6: malformed, counted, not refused.
        None => out.nodes_without_count += 1,
    }
    reachable
}

/// One node's `/Count`, resolved through a reference if it is one.
fn count_of<G: ObjectGraph + ?Sized>(graph: &G, id: ObjId) -> Option<i64> {
    graph
        .resolved(id)
        .as_dict()?
        .get(b"Count")
        .map(|o| graph.resolve(o))
        .and_then(Object::as_int)
}

/// **Which sentence a refused save owes the operator.**
#[must_use]
pub fn refusal_origin(audit: &Audit, base: Option<&std::path::Path>) -> RefusalOrigin {
    let pre_existing = base
        .and_then(|path| std::fs::read(path).ok())
        .map(|bytes| audit_saved_bytes(&bytes))
        .is_some_and(|base| base.walked && !base.is_consistent());
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("save-refused-pagetree-origin pre_existing={pre_existing}")
    });
    match (pre_existing, audit.root_disagreement()) {
        (true, Some(root)) => RefusalOrigin::PreExisting {
            declared: root.declared,
            reachable: root.reachable,
        },
        (false, Some(root)) => RefusalOrigin::Root {
            declared: root.declared,
            reachable: root.reachable,
        },
        // See ⚠ above for the `(true, None)` half of this arm.
        (_, None) => RefusalOrigin::Interior {
            nodes: audit.disagreements.len(),
        },
    }
}

/// Which refusal a page-tree disagreement owes; `pdfcer-gui`'s
/// `text::pagetree::refusal_sentence` words each one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefusalOrigin {
    /// The file on disk already disagreed at the root: undo cannot fix it.
    PreExisting {
        /// The root's `/Count`.
        declared: i64,
        /// The pages actually reachable beneath the root.
        reachable: usize,
    },
    /// The root disagrees and the file on disk did not: this session did it.
    Root {
        /// The root's `/Count`.
        declared: i64,
        /// The pages actually reachable beneath the root.
        reachable: usize,
    },
    /// Only interior nodes disagree.
    Interior {
        /// How many `/Pages` nodes disagree.
        nodes: usize,
    },
}

#[cfg(test)]
mod tests;
