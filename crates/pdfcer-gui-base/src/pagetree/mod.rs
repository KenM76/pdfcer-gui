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
///
/// Carries the node's identity as well as the two numbers, because a document
/// with several stale nodes is a different diagnosis from one with a single
/// stale root — the first says a whole subtree was rebuilt wrongly, the second
/// says an ancestor walk stopped one level early, which is the defect actually
/// observed. The trace prints the ids; the operator is never shown one.
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
///
/// Always constructible — there is no error variant — because every way the
/// walk can fail to run is a way it must **not** refuse a save (§6), and an
/// error type would make "could not look" and "looked and found nothing"
/// interchangeable at the call site. [`Self::walked`] keeps them apart.
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
    ///
    /// What the operator-facing sentence is built from, because it is the only
    /// pair of numbers with a symptom he can be promised: `declared` is what
    /// Acrobat will list, `reachable` is what is really there, and the
    /// difference is the number of blank pages he will find. An intermediate
    /// node's disagreement still refuses the save; it just gets the sentence
    /// that does not name a page count.
    #[must_use]
    pub fn root_disagreement(&self) -> Option<Disagreement> {
        self.disagreements.iter().copied().find(|d| d.root)
    }
}

/// **Parse the bytes a save is about to write, walk their page tree, and say
/// what the walk found.**
///
/// The save path's whole call — see §8. It is here rather than inlined at the
/// call site so that the argument for *what a failure to parse means* lives
/// beside the code that decides it, which is the same placement
/// `redact::prove_saved_bytes` takes for the identical reason.
///
/// # Bytes rather than the live session, and it is not a free choice
///
/// The session's own graph carries the same disagreement — `delete_pages`
/// rewrites the parent in place and leaves the ancestors alone, so the corrupt
/// state exists before serialization — and auditing it would cost no re-parse
/// at all. It is not what this does, because **the artifact is what the
/// operator receives**, and a guard that checked the state a writer was asked
/// to serialize rather than the bytes it produced would be blind to any defect
/// introduced by the writer itself. That is the same posture, and the same
/// sentence, `app::save::write_copy` uses for the absence proof: the
/// guarantee must not depend on how the value was constructed.
///
/// # An unparsable buffer returns a DEFAULT audit, not an error
///
/// [`Audit::walked`] is `false` and [`Audit::is_consistent`] is `true`, so the
/// save proceeds. §6 carries the argument;
/// `redact::proof::decoded_streams_of` carries the precedent and the sentence:
/// *"a skip narrows the evidence rather than fabricating it."* These are bytes
/// pdfcer itself just wrote, so a buffer that will not re-parse is a **writer**
/// defect of a different kind, and blocking the operator's only route to a file
/// over the guard's own inability to look is the failure mode this project has
/// already shipped once, in the redaction proof, and corrected.
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
///
/// Returns the **structural** answer — what `/Kids` actually holds — never the
/// declared one. That direction is the whole of §1: a walk that short-circuited
/// on `/Count` would be reading the field it is here to check.
///
/// The disagreement is recorded **after** the children are counted, so
/// [`Audit::disagreements`] comes out deepest-first with the root last, which
/// is the order a reader of the trace wants: the first entry is the innermost
/// node that is wrong.
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
///
/// `as_int` rather than `as_number`: §7.7.3.2 says integer, and a `/Count 3.0`
/// is a file this guard declines to judge rather than one it refuses — the same
/// posture §6 takes for an absent count, for the same reason.
fn count_of<G: ObjectGraph + ?Sized>(graph: &G, id: ObjId) -> Option<i64> {
    graph
        .resolved(id)
        .as_dict()?
        .get(b"Count")
        .map(|o| graph.resolve(o))
        .and_then(Object::as_int)
}

/// **Which sentence a refused save owes the operator.**
///
///
/// `base` is the file the document was opened from, or `None` for a document
/// that has never been on disk (`file.new`).
///
/// # The question this exists to ask: was it already like this when he
/// opened it?
///
/// `pdfcer-gui`'s `text::pagetree::save_refused_root` and `save_refused_interior`
/// both end *"undo the page removal (Ctrl+Z)"*. That is the correct remedy
/// exactly when pdfcer caused the damage — and useless when the file arrived
/// broken. An operator who empties his undo stack against a refusal his own
/// tool told him undo would fix has been sent in a circle by it, which is worse
/// than an unexplained refusal because it costs him his work as well as his
/// time.
///
/// So the base file is walked again and [`RefusalOrigin::PreExisting`] is
/// returned when it was already inconsistent; its sentence,
/// `text::pagetree::save_refused_pre_existing`, names a different remedy and does not
/// claim the fault is pdfcer's.
///
/// # It is paid only on the refusal path
///
/// One extra parse of the original file, inside a function that runs only after
/// a save has already failed. An ordinary save never reaches it, so it is
/// outside §9's budget entirely.
///
/// # A base file it cannot read or walk falls through to the ordinary
/// sentences
///
/// Deliberately. `Audit::walked` is `false` and `is_consistent` is `true` for
/// an unreadable file, and *"I could not check the original"* is not evidence
/// that the original was fine. Erring the other way would tell the operator a
/// defect is not pdfcer's on the strength of nobody having looked, and this
/// project has a standing rule against exactly that shape.
///
/// # ⚠ One residual, named rather than papered over
///
/// A file that arrived with an **interior-only** disagreement gets the interior
/// sentence, which says *"this is a fault in pdfcer"* — and is wrong. The
/// pre-existing sentence needs the root's two numbers to say anything useful
/// and there is no root disagreement to take them from. A fourth string is not
/// written for a state no measurement has ever produced; if one appears, this
/// is the paragraph that predicted it.
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
