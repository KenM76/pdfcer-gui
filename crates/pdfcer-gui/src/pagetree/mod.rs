//! # `pagetree` — **does this document's page tree still agree with itself?**
//!
//! One question, asked of the bytes a save is about to hand to the file system,
//! and answered by walking the page tree the way a *stranger's* reader walks
//! it. It exists because on 2026-09-05 the operator opened a document pdfcer
//! had just written and found pages in it that pdfcer did not believe were
//! there:
//!
//! > *"I tested deleting pages from a pdf. when I open the document in Acrobat
//! > there are blank pages at the end of the document equalling the number of
//! > pages I deleted."*
//!
//! ## ★★★ 1. The lesson this module exists to carry: **a GUI that checks its
//! own work with its own parser cannot see this class of defect at all**
//!
//! This is the sentence to read before changing anything below, because every
//! design decision here follows from it.
//!
//! ISO 32000-1 §7.7.3.2 gives a page tree **two** independent descriptions of
//! how many pages it has, and requires them to agree:
//!
//! * `/Kids` — the children, walked recursively to the `/Page` leaves. **This
//!   is the structure.**
//! * `/Count` — on every `/Pages` node, *"the number of leaf nodes that are
//!   descendants of this node"*; on the root, the document's page count.
//!   **This is the declaration.**
//!
//! A well-formed file has them equal at every node, so a reader may use either.
//! Readers therefore split into two camps, and the split is invisible until the
//! two disagree:
//!
//! | reader | what it builds the page list from | what it saw after `delete-pages --pages 2,3` on a 36-page nested document |
//! |---|---|---|
//! | `pdfcer_core::page_tree::pages` — and therefore this whole shell | `/Kids`, walked | **34 pages. A healthy document.** |
//! | Acrobat | the root `/Count` | **36 pages, the last two blank** |
//!
//! ⇒ Every unit test this project could have written about page deletion, every
//! panel, every driven check, and the engine's own reader **all agree the file
//! is fine**, because they all read the same half of a contract whose other
//! half is the broken one. Two thousand five hundred passing tests could not
//! have caught it and did not. **The operator caught it.**
//!
//! ⇒ So this module does the one thing none of those do: it reads **`/Count`
//! raw off each node's dictionary** and compares it against the leaves actually
//! hanging below that node. It never asks `page_tree::pages` anything. A
//! rewrite of this module that starts by calling `pages()` and comparing its
//! length to something would be **vacuous** — it would compare the walked
//! structure against itself and pass on every corrupt file in existence.
//!
//! ## ★★★ 2. And it must be a NESTED tree, or the question is unfalsifiable
//!
//! The second reason the defect shipped. On a **flat** page tree — one root
//! whose `/Kids` are all `/Page` leaves — the immediate parent *is* the root,
//! so a writer that updates only the immediate parent updates the root by
//! accident and the file is correct. The bug **cannot occur**. Reproduced
//! against `fixtures/four-pages.pdf` first, and it was clean.
//!
//! Every synthetic fixture in either corpus had a one-level tree. Real CAD
//! exports do not: SolidWorks nests, and so does every producer that writes its
//! page tree in balanced chunks. So the defect was invisible to the corpus and
//! present on every document the operator actually works with.
//!
//! ⇒ `fixtures/nested-page-tree.pdf` (`tools/gen-nested-page-tree-fixture.py`)
//! exists for this module and its header states, in as many words, that the
//! nesting **is the point** and that swapping it for a flat document would make
//! every assertion here pass against a build whose walk never goes above the
//! immediate parent.
//!
//! ## 3. What this module is NOT: a repair
//!
//! It never writes. It has no `&mut` anywhere and takes its graph by shared
//! reference. The temptation — *"we know the right `/Count`, just fix it"* — is
//! refused on the boundary argument this project applies everywhere:
//!
//! `pdfcer-core` **owns** the page tree. It is the only writer of `/Count`
//! (`edit.rs:31918`, `:32636-32647`, `:33296-33304`,
//! `pageops/assemble.rs:540`). A shell that silently patched the same key would
//! be a **second writer of one structure**, and the two would drift — pdfcer
//! would then be repairing files against a rule the engine had since changed,
//! on documents nobody was looking at, with the operator told nothing. The
//! defect would move from *"Acrobat shows blank pages"* to *"pdfcer edits your
//! page tree behind your back and is sometimes wrong about it"*, which is
//! strictly worse because it is no longer visible.
//!
//! ⇒ **Refuse, name what is wrong, say what it cost, and file the defect
//! upstream.** Filed as
//! `request_delete_pages_leaves_ancestor_count_stale_on_a_nested_page_tree.md`,
//! whose closing offer stands: *"If `pdfcer-core` would rather own that check —
//! a `validate_page_tree()` a writer calls before it commits — we would use it
//! and delete ours the same day."*
//!
//! ## 4. ★★ Why the raw-dictionary read is legitimate here, and where it drifts
//!
//! Reading `/Count` off a dictionary through [`ObjectGraph`] is a seam, and
//! this project has one standing precedent for it — `canvas::notepopup::model`'s
//! `open_flag`, which reads `/Open` the same way and admits in its own header
//! that it is a workaround. The same admission is owed here and is made:
//!
//! ⚠ **`pdfcer-core`'s read model cannot see an intermediate node's `/Count` at
//! all.** Audited 2026-09-05 against v0.38.0 (`b01964f`):
//! `page_tree::pages` / `pages_in` return `Vec<Page>` — **leaves only**;
//! intermediate nodes are traversed and discarded (`page_tree.rs:283`, `:300`).
//! [`page_slots`](pdfcer_core::page_tree::page_slots) (`page_tree.rs:417`) is
//! the only public type that leaks an intermediate node's *identity*, via
//! `PageSlot::ancestors`, and it exposes neither that node's `/Count` nor its
//! `/Kids`. `b"Count"` does not occur in `page_tree.rs` at all; every
//! occurrence in the crate is either an outline `/Count` or a **write** site in
//! the rebalancer. There is no public read of a page-tree `/Count` anywhere,
//! and there is no `validate_page_tree`.
//!
//! ⇒ So there is no non-raw way to ask this question, and the raw way is a
//! **read of a key ISO 32000-1 §7.7.3.2 defines, through the crate's own public
//! graph**. Nothing is guessed and nothing is written. The day `pdfcer-core`
//! models a page-tree node, [`audit`] becomes a loop over that type.
//!
//! ## 5. ★ Why it walks itself instead of using `PageSlot::ancestors`
//!
//! `page_slots` would have been fewer lines, and
//! `panels::forms::tab_order::tabs` sets the precedent for reading a raw key
//! off the nodes it names. It is not used, for one reason that matters and one
//! that does:
//!
//! * **It cannot see a node with no leaves under it.** A `/Pages` node appears
//!   in `PageSlot::ancestors` only when some leaf is beneath it, so a subtree
//!   whose every page was deleted is invisible to it — and *"every page under
//!   this node was removed"* is precisely a state a page deletion produces. The
//!   walk below sees that node, with `reachable: 0`, and can refuse on it.
//! * It answers the wrong shape: a per-leaf ancestor list has to be inverted
//!   into a per-node tally anyway, and the inversion is the same recursion.
//!
//! ★★ Worth recording, because it is the engine stating the very contract it
//! then broke: `PageSlot::ancestors`' own doc comment
//! (`page_tree.rs:346-348`) reads *"Every ancestor `Pages` node, root first,
//! excluding the page itself. **A delete must decrement `/Count` on all of
//! them.**"* The requirement is written into the type. It is `delete_pages`
//! that does not do it.
//!
//! ## 6. The verdict this module can and cannot return
//!
//! | state of the file | [`Audit::disagreements`] | the save |
//! |---|---|---|
//! | every node's `/Count` equals its leaf tally | empty | proceeds |
//! | some node declares a number it does not have | one entry per node | **refused** |
//! | a `/Pages` node carries **no** `/Count` at all | empty — see below | proceeds |
//! | the bytes cannot be parsed, or there is no `/Pages` root | empty, with [`Audit::walked`] `false` | proceeds |
//!
//! **An absent `/Count` is deliberately not a disagreement.** §7.7.3.2 requires
//! the key, so a node without one is malformed — but it is a *different*
//! defect, it is not one any pdfcer verb produces, and it arrives most often on
//! a file pdfcer merely opened. A guard that refused it would refuse to save
//! documents pdfcer did not damage, which is the failure mode
//! `redact::proof`'s own header warns about at length: a false refusal after
//! the operator has done the work, with no route to a file. It is counted in
//! [`Audit::nodes_without_count`] so it is visible in the trace and unmeasured
//! by nobody.
//!
//! **An unwalkable document is deliberately not a refusal**, on
//! `redact::proof::decoded_streams_of`'s standing reason, quoted because it is
//! the same argument: *"a document that cannot be re-parsed yields an empty
//! list rather than a panic or an error… a skip narrows the evidence rather
//! than fabricating it."* [`Audit::walked`] is what makes the narrowing
//! visible instead of silent — a clean audit and an audit that never ran are
//! otherwise byte-identical, which is this project's most-repeated failure
//! shape.
//!
//! ## 7. Guards
//!
//! A page tree is operator-supplied and may be hostile or merely broken. The
//! walk is bounded three ways, each mirroring the engine's own walk so that a
//! document `pages_in` accepts is one this accepts:
//!
//! * **cycles** — a `visited` set of [`ObjId`]; a node reached twice
//!   contributes zero and is counted in [`Audit::cycles`].
//! * **depth** — [`pdfcer_core::page_tree::MAX_TREE_DEPTH`], the engine's own
//!   constant rather than a second number that could drift from it.
//! * **recursion** — the walk is recursive, exactly as the engine's is, and the
//!   depth bound is what makes that safe. `pdfcer-core`'s panic-free policy
//!   forbids a stack overflow on untrusted input and this honours it by
//!   borrowing the same ceiling.
//!
//! ## 8. Where it is called from
//!
//! [`crate::app::save::write_copy`] — the funnel every save verb goes through
//! (`file.save_copy`, `file.save_as`, `file.save_in_place`), between the bytes
//! being built and `std::fs::write`. **Not** on the delete-pages arm.
//!
//! That placement is the point, and it was vindicated the same day. The defect
//! is a *writer's* invariant, so all seven page verbs were measured on the
//! three-level fixture rather than assumed:
//!
//! | verb | ancestor `/Count` after |
//! |---|---|
//! | `delete_pages` | ✗ **stale** — only the immediate parent updated |
//! | `page-copy --cut` | ✗ **stale**, byte-for-byte identical output |
//! | `insert_pages`, `extract_pages` | ✓ (they rebuild the tree flat) |
//! | `reorder_pages` | ✓ |
//! | `paste_pages`, `merge_document` | ✓ — and these demonstrably **do** walk to the root |
//!
//! ⇒ A guard on the delete-pages arm would have passed `page-copy --cut`
//! straight through, and would have to be remembered again for every verb added
//! later. `crate::redact::prove_saved_bytes` sits at the same boundary for the
//! same reason and its argument is the precedent: *"the proof has to be made
//! here or not at all."*
//!
//! ★ And the **sentence** a refusal owes is chosen here too, by
//! [`refusal_sentence`], rather than at the save. That is not tidiness: the
//! choice depends on a **second audit** — of the file the document was opened
//! from, to answer *"was it already like this when he opened it?"* — and this
//! module is the only place equipped to take one. `crate::text::pagetree` still
//! owns every word.
//!
//! ## 9. ★★ What it costs, measured rather than asserted
//!
//! `BENCHMARK.md` exists in this repository because an earlier analysis
//! asserted a performance weakness from architecture and was wrong, so these
//! numbers were taken before the guard was placed on every save. Release build,
//! 2026-09-05, this machine:
//!
//! | document | size | the walk itself | [`audit_saved_bytes`] end to end | `to_incremental_bytes` on the same document |
//! |---|---:|---:|---:|---:|
//! | `D:/Dev/pdfTests/SW41177/SW41177.pdf` — 36 pages, nested, his own drawing set | 1,831,090 B | **16.7 µs** | **1.78 ms** | 1.58 ms |
//! | `D:/Dev/pdfTests/ncored-benchmark-cad-drawing.pdf` — 129,758 objects | 5,724,699 B | **0.6 µs** | **3.51 ms** | 5.41 ms |
//! | `fixtures/nested-page-tree.pdf` | 5,026 B | 4.2 µs | — | — |
//!
//! ★★★ **A first draft of this paragraph claimed the guard was "beneath"
//! `to_incremental_bytes`, and that was wrong.** It was written from the shape
//! of the code rather than from a measurement, which is the exact error
//! `BENCHMARK.md` commemorates — written into the very paragraph citing it. The
//! measured truth: on the operator's drawing set the guard costs **more** than
//! the writer that produced the bytes (1.78 ms against 1.58 ms), roughly
//! doubling a save's CPU; on the large CAD sheet it costs about two thirds of
//! it. The guard is not free and this paragraph will not pretend it is.
//!
//! ⇒ It is placed on every save anyway, and the reason is that ~2–3.5 ms is
//! **imperceptible against the act it is part of**: an operator-initiated save
//! that opens a file dialog and then writes megabytes to disk. Doubling a
//! millisecond inside a gesture that takes a second is not a cost he can
//! observe. The number that would change this decision is tens of milliseconds,
//! and it is an order of magnitude away.
//!
//! ★ **The walk is not where the time goes; the re-parse is.** The walk barely
//! moves with file size — the benchmark sheet is three times the bytes and
//! *faster*, because it has one page — since it is bounded by the page tree
//! rather than by the document. So a future engine that let the guard read the
//! written revision without re-parsing the whole file would make this free
//! rather than cheap.
//!
//! ⇒ So it is **ungated**: run on every save of every document, not only after
//! a page-count change. That is a decision and it has a reason beyond the
//! numbers — a gate on *"has the page count changed this session?"* would be a
//! second source of truth about a fact the bytes already carry, and the
//! direction it would drift in is a save that quietly wrote a damaged file
//! because a shell-side flag disagreed with the writer. The same argument
//! `crate::app::save::write_copy` makes for asking the **session** whether a
//! redaction is staged rather than keeping a flag.

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
    /// ★★ **How many levels the deepest leaf hangs below the root**, counting
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
/// `crate::redact::prove_saved_bytes` takes for the identical reason.
///
/// # ★ Bytes rather than the live session, and it is not a free choice
///
/// The session's own graph carries the same disagreement — `delete_pages`
/// rewrites the parent in place and leaves the ancestors alone, so the corrupt
/// state exists before serialization — and auditing it would cost no re-parse
/// at all. It is not what this does, because **the artifact is what the
/// operator receives**, and a guard that checked the state a writer was asked
/// to serialize rather than the bytes it produced would be blind to any defect
/// introduced by the writer itself. That is the same posture, and the same
/// sentence, `crate::app::save::write_copy` uses for the absence proof: the
/// guarantee must not depend on how the value was constructed.
///
/// # ★ An unparsable buffer returns a DEFAULT audit, not an error
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

    // The engine's own node-kind dispatch (`page_tree::is_pages_node`,
    // `page_tree.rs:531`), reproduced rather than called because it is
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
/// Moved here from [`crate::app::save`] on 2026-09-05 when that file crossed
/// R2's 1,500-line ceiling, and it is the right home rather than a convenient
/// one: choosing between the three sentences is reasoning about an [`Audit`],
/// and the third choice needs a **second audit** that this module is the only
/// place equipped to take. `crate::text::pagetree` still owns every word; this
/// owns only the question of which words apply.
///
/// `base` is the file the document was opened from, or `None` for a document
/// that has never been on disk (`file.new`).
///
/// # ★★★ The question this exists to ask: was it already like this when he
/// opened it?
///
/// [`crate::text::pagetree::save_refused_root`] and `save_refused_interior`
/// both end *"undo the page removal (Ctrl+Z)"*. That is the correct remedy
/// exactly when pdfcer caused the damage — and useless when the file arrived
/// broken. An operator who empties his undo stack against a refusal his own
/// tool told him undo would fix has been sent in a circle by it, which is worse
/// than an unexplained refusal because it costs him his work as well as his
/// time.
///
/// So the base file is walked again and
/// [`crate::text::pagetree::save_refused_pre_existing`] takes over when it was
/// already inconsistent. That sentence names a different remedy and does not
/// claim the fault is pdfcer's.
///
/// # ★ It is paid only on the refusal path
///
/// One extra parse of the original file, inside a function that runs only after
/// a save has already failed. An ordinary save never reaches it, so it is
/// outside §9's budget entirely.
///
/// # ★ A base file it cannot read or walk falls through to the ordinary
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
pub fn refusal_sentence(name: &str, audit: &Audit, base: Option<&std::path::Path>) -> String {
    let pre_existing = base
        .and_then(|path| std::fs::read(path).ok())
        .map(|bytes| audit_saved_bytes(&bytes))
        .is_some_and(|base| base.walked && !base.is_consistent());
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("save-refused-pagetree-origin pre_existing={pre_existing}")
    });
    match (pre_existing, audit.root_disagreement()) {
        (true, Some(root)) => {
            crate::text::pagetree::save_refused_pre_existing(name, root.declared, root.reachable)
        }
        (false, Some(root)) => {
            crate::text::pagetree::save_refused_root(name, root.declared, root.reachable)
        }
        // See ⚠ above for the `(true, None)` half of this arm.
        (_, None) => crate::text::pagetree::save_refused_interior(name, audit.disagreements.len()),
    }
}

#[cfg(test)]
mod tests;
