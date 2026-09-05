//! # `pagetree::tests` — and the one shape that would make all of them vacuous
//!
//! ★★★ **A test on a FLAT page tree defeats this entire module.**
//!
//! On a flat tree the immediate parent *is* the root, so `/Count` and the leaf
//! tally can only disagree at one node — and an implementation that compared
//! the root's `/Count` against `pages().len()` and looked no further would pass
//! every such test while being blind to the defect this module exists for. The
//! defect is *an ancestor above the parent going stale*, and that state cannot
//! be constructed on a one-level tree at all.
//!
//! ⇒ So the positive controls here are **nested**, and the load-bearing one is
//! not hand-built: [`the_real_corrupt_file_is_caught`] runs against a document
//! produced by **the engine's own CLI performing the operator's own operation**
//! — `pdfcer.exe delete-pages --pages 2` on `fixtures/nested-page-tree.pdf` —
//! so what is under test is the real defect rather than a hand-made imitation
//! of it. A hand-built graph can only assert that the walk does what its author
//! thought; a file the writer produced asserts that the walk catches what the
//! writer does.
//!
//! ## What each control is for
//!
//! | test | proves |
//! |---|---|
//! | [`a_healthy_nested_tree_is_consistent`] | the guard does not fire on good files — without it the guard could be `false` |
//! | [`a_stale_root_above_a_correct_parent_is_caught`] | the walk goes **above the immediate parent**; this is the assertion a flat fixture cannot make |
//! | [`a_stale_middle_node_is_caught_as_well_as_the_root`] | it reports **every** bad node, not just the root |
//! | [`the_real_corrupt_file_is_caught`] | ★★★ end to end, against bytes the engine wrote |
//! | [`the_real_clean_file_is_consistent`] | the same file **before** the delete, so the test above cannot pass by always refusing |
//! | [`a_flat_tree_is_still_checked`] | the flat case is not skipped — it is merely unable to exhibit the defect |
//! | [`an_absent_count_is_not_a_disagreement`] | §6 — malformed is counted, not refused |
//! | [`a_document_that_cannot_be_walked_is_not_refused`] | §6 — a skip narrows evidence rather than fabricating it |
//! | [`a_kids_cycle_terminates`] | the walk cannot loop on legal-but-hostile syntax |
//! | [`a_node_whose_whole_subtree_was_removed_is_caught`] | the case `PageSlot::ancestors` structurally cannot see (§5) |

// ★ The INNER `#![cfg(test)]` is redundant — the module is declared
// `#[cfg(test)] mod tests;` — and it is here for two gates that recognise a
// test-only FILE by exactly this attribute rather than by its name:
// `tools/gates/check-ui-strings.sh` exclusion 2, and
// `app::settings::tests::no_call_site_builds_its_own_options`, which otherwise
// reads the `SaveOptions::default()` below as a shipped call site discarding
// the operator's configuration. `app/save/tests.rs` carries the same attribute
// for the same reason. Both gates state, in their own words, that the property
// earning the exemption is "not in the shipped binary" — which a filename
// merely restates and this attribute actually asserts.
#![cfg(test)]

use super::*;
use pdfcer_core::document::Document;
use pdfcer_core::object::{Dict, Name};

/// A graph of loose objects with a trailer — enough for a page tree and
/// nothing more.
///
/// Hand-built rather than loaded from a file for the three tests whose subject
/// is one *shape* (an absent `/Count`, a cycle, an emptied subtree). Those
/// shapes are awkward or impossible to obtain from a real writer, and building
/// them by hand is the only way to assert the branch at all. Every test whose
/// subject is the **defect** uses the real file instead — see the header.
struct Objects {
    objects: Vec<(ObjId, Object)>,
    root: Object,
}

impl ObjectGraph for Objects {
    fn value(&self, id: ObjId) -> Option<&Object> {
        self.objects.iter().find(|(o, _)| *o == id).map(|(_, v)| v)
    }
    fn trailer_entry(&self, key: &[u8]) -> Option<&Object> {
        (key == b"Root").then_some(&self.root)
    }
}

fn id(num: u32) -> ObjId {
    ObjId::new(num, 0)
}

fn dict(entries: &[(&[u8], Object)]) -> Object {
    let mut d = Dict::new();
    for (k, v) in entries {
        d.insert(Name((*k).to_vec()), v.clone());
    }
    Object::Dict(d)
}

fn refs(ids: &[u32]) -> Object {
    Object::Array(ids.iter().map(|n| Object::Reference(id(*n))).collect())
}

/// A `/Pages` node with a `/Count` and `/Kids`.
fn node(count: i64, kids: &[u32]) -> Object {
    dict(&[
        (b"Type", Object::Name(Name(b"Pages".to_vec()))),
        (b"Count", Object::Integer(count)),
        (b"Kids", refs(kids)),
    ])
}

/// A `/Page` leaf.
fn leaf() -> Object {
    dict(&[(b"Type", Object::Name(Name(b"Page".to_vec())))])
}

/// A graph whose catalog is object 100 and whose page-tree root is object 1.
fn graph(objects: Vec<(ObjId, Object)>) -> Objects {
    let mut all = objects;
    all.push((id(100), dict(&[(b"Pages", Object::Reference(id(1)))])));
    Objects {
        objects: all,
        root: Object::Reference(id(100)),
    }
}

/// The three-level shape of `fixtures/nested-page-tree.pdf`, in miniature:
/// root(1) -> A(2) -> {A1(3), A2(4)}, each bottom node holding two leaves.
///
/// `root_count`, `a_count` and `a1_count` are the declarations, so a test can
/// make exactly one of them wrong and assert that the walk found that one.
fn three_level(root_count: i64, a_count: i64, a1_count: i64, a1_leaves: &[u32]) -> Objects {
    let mut objects = vec![
        (id(1), node(root_count, &[2])),
        (id(2), node(a_count, &[3, 4])),
        (id(3), node(a1_count, a1_leaves)),
        (id(4), node(2, &[12, 13])),
        (id(12), leaf()),
        (id(13), leaf()),
    ];
    for n in a1_leaves {
        objects.push((id(*n), leaf()));
    }
    graph(objects)
}

/// **A healthy nested tree passes.**
///
/// The control that makes every other test here mean something: without it,
/// `is_consistent` returning `false` unconditionally would satisfy all the
/// positive controls, and the guard would refuse every save in the program.
#[test]
fn a_healthy_nested_tree_is_consistent() {
    let g = three_level(4, 4, 2, &[10, 11]);
    let audit = audit(&g);
    assert!(audit.walked);
    assert_eq!(audit.reachable_pages, 4);
    assert_eq!(audit.declared_pages, Some(4));
    assert!(audit.is_consistent(), "{audit:?}");
}

/// ★★★ **A stale ROOT above a CORRECT immediate parent is caught.**
///
/// The single assertion this module exists for, and the one a flat fixture is
/// structurally incapable of making. `A1` declares 1 and holds 1 — correct, as
/// the engine leaves it. `A` and the root still declare the pre-delete numbers.
/// An implementation that checked only the node holding the changed page would
/// pass a clean bill on this graph.
#[test]
fn a_stale_root_above_a_correct_parent_is_caught() {
    // A1 correctly says 1 over its one leaf; A still says 2; root still says 4.
    let g = three_level(4, 4, 1, &[10]);
    let audit = audit(&g);
    assert_eq!(audit.reachable_pages, 3);
    assert_eq!(audit.declared_pages, Some(4));
    assert!(!audit.is_consistent());
    let root = audit.root_disagreement().expect("the root disagrees");
    assert_eq!((root.declared, root.reachable), (4, 3));
    // And the correct node is NOT reported: a guard that flagged healthy nodes
    // would refuse every save in the program on its second run.
    assert!(
        !audit.disagreements.iter().any(|d| d.node == id(3)),
        "the immediate parent is correct and must not be reported: {audit:?}"
    );
}

/// **Every bad node is reported, not only the root.**
///
/// The middle node `A` and the root are both stale here — the shape the engine
/// actually produces on a three-level tree — and the walk must name both.
/// Reporting only the root would still refuse the save, so this is not about
/// the verdict; it is about the trace being able to say *how far up the rot
/// goes*, which is what distinguishes "the walk stopped one short" from "there
/// is no walk".
#[test]
fn a_stale_middle_node_is_caught_as_well_as_the_root() {
    let g = three_level(4, 4, 1, &[10]);
    let audit = audit(&g);
    let nodes: Vec<u32> = audit.disagreements.iter().map(|d| d.node.num).collect();
    assert_eq!(nodes, vec![2, 1], "deepest first, root last: {audit:?}");
    assert!(!audit.disagreements[0].root);
    assert!(audit.disagreements[1].root);
}

/// **A flat tree is walked, not skipped.**
///
/// The guard has no special case for a one-level document and must not grow
/// one: a flat tree simply cannot exhibit the defect, which is a property of
/// the document rather than a reason to look away. If a writer ever does leave
/// a flat root stale, this catches it.
#[test]
fn a_flat_tree_is_still_checked() {
    let good = graph(vec![
        (id(1), node(2, &[10, 11])),
        (id(10), leaf()),
        (id(11), leaf()),
    ]);
    assert!(audit(&good).is_consistent());

    let bad = graph(vec![
        (id(1), node(3, &[10, 11])),
        (id(10), leaf()),
        (id(11), leaf()),
    ]);
    let audit = audit(&bad);
    assert!(!audit.is_consistent());
    assert!(audit.root_disagreement().is_some());
}

/// **A `/Pages` node with no `/Count` is counted, not refused** — §6.
///
/// §7.7.3.2 requires the key, so the node is malformed; but no pdfcer verb
/// produces one, it arrives on files pdfcer merely opened, and refusing it
/// would block a save on damage pdfcer did not do. `redact::proof`'s header
/// argues the same case at length for the same reason: a false refusal after
/// the operator has done the work is worse than the thing it guards against.
#[test]
fn an_absent_count_is_not_a_disagreement() {
    let g = graph(vec![
        (
            id(1),
            dict(&[
                (b"Type", Object::Name(Name(b"Pages".to_vec()))),
                (b"Kids", refs(&[10])),
            ]),
        ),
        (id(10), leaf()),
    ]);
    let audit = audit(&g);
    assert!(audit.walked);
    assert_eq!(audit.reachable_pages, 1);
    assert_eq!(audit.declared_pages, None);
    assert_eq!(audit.nodes_without_count, 1);
    assert!(audit.is_consistent(), "counted, not refused: {audit:?}");
}

/// **A document with no page-tree root is not refused, and says it was not
/// walked** — §6.
///
/// The two halves matter separately. Not refusing is the posture; `walked ==
/// false` is what stops a skipped audit and a clean audit being the same
/// value, which is this project's most-repeated failure shape.
#[test]
fn a_document_that_cannot_be_walked_is_not_refused() {
    let g = Objects {
        objects: vec![(id(100), dict(&[]))],
        root: Object::Reference(id(100)),
    };
    let audit = audit(&g);
    assert!(!audit.walked);
    assert!(audit.is_consistent());
    assert_eq!(audit.reachable_pages, 0);
}

/// **A `/Kids` cycle terminates and is counted.**
///
/// `1 0 obj << /Kids [1 0 R] >>` is legal syntax. Without the visited set this
/// recurses until the stack ends, and `pdfcer-core`'s panic-free policy on
/// untrusted input forbids that outcome as firmly as it forbids an `unwrap`.
/// The count is asserted as well as the termination, because a walk that
/// silently truncated would report a leaf tally that is a floor rather than a
/// total and would then refuse a save over its own truncation.
#[test]
fn a_kids_cycle_terminates() {
    let g = graph(vec![
        (id(1), node(1, &[2])),
        (id(2), node(1, &[1, 10])),
        (id(10), leaf()),
    ]);
    let audit = audit(&g);
    assert!(audit.walked);
    assert_eq!(audit.cycles, 1);
    assert_eq!(audit.reachable_pages, 1);
}

/// ★★ **A node whose whole subtree was removed is caught** — the case §5 says
/// `PageSlot::ancestors` structurally cannot see.
///
/// `A1` has an empty `/Kids` and still declares 2. It appears in no
/// `PageSlot`, because a `PageSlot` exists only per surviving leaf, so an
/// implementation built on `page_slots` would report this document clean. It is
/// exactly the state a page deletion that emptied a subtree produces.
#[test]
fn a_node_whose_whole_subtree_was_removed_is_caught() {
    let g = graph(vec![
        (id(1), node(4, &[2])),
        (id(2), node(4, &[3, 4])),
        (id(3), node(2, &[])),
        (id(4), node(2, &[12, 13])),
        (id(12), leaf()),
        (id(13), leaf()),
    ]);
    let audit = audit(&g);
    let nodes: Vec<u32> = audit.disagreements.iter().map(|d| d.node.num).collect();
    assert_eq!(nodes, vec![3, 2, 1], "{audit:?}");
}

// ==========================================================================
// The end-to-end controls: real bytes, written by the engine's own writer.
// ==========================================================================

/// The repository's `fixtures/` directory, from this crate's manifest.
fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name)
}

/// **The clean fixture is clean.**
///
/// The negative control for [`the_real_corrupt_file_is_caught`], and it is not
/// optional: a guard that refused every document would pass that test. It also
/// asserts the fixture's shape, so a regenerated or flattened fixture fails
/// here rather than silently making the positive control vacuous.
#[test]
fn the_real_clean_file_is_consistent() {
    let bytes = std::fs::read(fixture("nested-page-tree.pdf"))
        .expect("fixtures/nested-page-tree.pdf — run tools/gen-nested-page-tree-fixture.py");
    let doc = Document::from_bytes(bytes).expect("the fixture parses");
    let audit = audit(&doc);
    assert!(audit.walked);
    assert_eq!(audit.reachable_pages, 12);
    assert_eq!(audit.declared_pages, Some(12));
    assert!(audit.is_consistent(), "{audit:?}");
    // ★★★ THE FIXTURE MUST BE NESTED. Seven `/Pages` nodes, three levels. A
    // flat replacement would leave every positive control in this file passing
    // against a build whose walk never goes above the immediate parent, which
    // is the exact defect. Asserted here rather than trusted from the
    // generator's docstring.
    assert!(
        audit.nodes_without_count == 0,
        "every node in the fixture must declare a /Count: {audit:?}"
    );
    assert_eq!(
        nodes_walked(&doc),
        7,
        "the fixture must have 7 /Pages nodes"
    );
}

/// How many `/Pages` nodes the document has, counted the same way [`audit`]
/// counts them — used only to assert the fixture's own shape.
fn nodes_walked(doc: &Document) -> usize {
    let mut n = 0;
    for object in doc.objects() {
        if object
            .value
            .as_dict()
            .and_then(|d| d.get(b"Type"))
            .and_then(Object::as_name)
            .is_some_and(|t| t.as_bytes() == b"Pages")
        {
            n += 1;
        }
    }
    n
}

/// ★★★ **The real corrupt file is caught** — the positive control that is not
/// a hand-made imitation.
///
/// The bytes under test are produced here, at test time, by
/// `pdfcer_core::EditSession::delete_pages` — **the same code path the CLI ran
/// when the defect was measured**, and the same one the GUI's delete-pages arm
/// reaches. So this test asserts that the guard catches what the writer
/// actually does, not what this module's author believed it does.
///
/// ★★ It is written to **skip loudly rather than fail** if the engine is ever
/// fixed. The day `pdfcer-core` walks the ancestor chain, this document comes
/// out consistent and the assertion below would go red on a *repaired* engine —
/// turning the fix into a broken build. The skip prints the fact instead, and
/// [`a_stale_root_above_a_correct_parent_is_caught`] keeps the guard itself
/// under assertion with a graph that does not depend on any engine behaviour.
/// That is the same split `check-stale-blockers` exists to enforce: a claim
/// about what the engine cannot do is a dated citation, and where it can be an
/// assertion it should be one — but not an assertion that inverts on good news.
#[test]
fn the_real_corrupt_file_is_caught() {
    let bytes = std::fs::read(fixture("nested-page-tree.pdf"))
        .expect("fixtures/nested-page-tree.pdf — run tools/gen-nested-page-tree-fixture.py");
    let doc = Document::from_bytes(bytes).expect("the fixture parses");
    let mut session = pdfcer_core::edit::EditSession::new(doc);
    session
        .delete_pages(&[1])
        .expect("page 2 (0-based 1) deletes");
    let (out, _report) = session
        .to_incremental_bytes(&pdfcer_core::writer::SaveOptions::default())
        .expect("the update serializes");

    let written = Document::from_bytes(out).expect("what the writer produced parses");
    let audit = audit(&written);
    assert!(audit.walked);
    assert_eq!(
        audit.reachable_pages, 11,
        "one page was removed from the structure: {audit:?}"
    );

    // ★★★ **THE SKIP FIRED, AND IS NOW AN ASSERTION — 2026-09-05.**
    //
    // This branch used to `println!("SKIP: …")` and return the moment the engine
    // began updating every ancestor. It did, within hours of being reported —
    // `Pass 251.1`, `e4cefcd` — and on the bump to `pdfcer-core b1033ab` this
    // test walked the **real written bytes** and found every node consistent.
    //
    // ⇒ So the hopeful skip becomes a **standing assertion that the fix is
    // still there.** A `println!` inside a passing test is not evidence of
    // anything, and this project's own rule says why: *a SKIP is not red, so a
    // check can stop running unnoticed.* A skip that has served its purpose is
    // the clearest case of it — left alone, an engine regression would re-open
    // his defect while this test reported `ok` and printed a sentence nobody
    // reads.
    //
    // What it pins is his own reported symptom, end to end: delete one page
    // from a **three-level** tree through the real `EditSession`, serialise it,
    // re-read the bytes, and every `/Pages` node agrees with the leaves beneath
    // it. Two levels cannot make this claim — with the parent's parent being
    // the root, "no upward walk" and "a walk that stops one short" are the same
    // observation, which is why the fixture is three deep.
    //
    // ⚠ The broken shape, recorded because the numbers are the diagnosis and
    // the file that carried them is not in git: on v0.38.0 `b01964f` the
    // immediate parent was correct and **every node above it was stale** — the
    // root declaring **12 against 11 reachable**, i.e. one blank page at the end
    // in Acrobat, which is exactly what he reported. `audit.disagreements` held
    // at least two, because there was no upward walk at all rather than one that
    // stopped short.
    assert!(
        audit.is_consistent(),
        "★★★ REGRESSION: pdfcer-core has stopped decrementing /Count on every \
         page-tree ancestor. That is the defect the operator reported on \
         2026-09-05 — `blank pages at the end of the document equalling the \
         number of pages I deleted` — fixed in Pass 251.1 and now back. \
         {audit:?}"
    );
}

// ==========================================================================
// Which sentence a refusal owes — `refusal_sentence`
// ==========================================================================

/// ★★★ **A file that arrived broken is NOT told to press Ctrl+Z.**
///
/// The assertion `refusal_sentence` exists for. Both ordinary sentences name
/// undo as the way out, which is right when pdfcer caused the damage and a
/// circle when the file came in that way — and an operator who empties his undo
/// stack against a refusal his own tool promised undo would fix has lost his
/// work as well as his time.
///
/// The base file here is the fixture with one digit changed, which is the same
/// plant `app::save::tests` uses and for the same reason: it is a real,
/// openable PDF whose root declares one page more than it has.
#[test]
fn a_document_that_arrived_broken_gets_the_sentence_that_does_not_offer_undo() {
    let bytes = std::fs::read(fixture("nested-page-tree.pdf")).expect("the fixture is readable");
    const OLD: &[u8] = b"/Count 12";
    let at = bytes
        .windows(OLD.len())
        .position(|w| w == OLD)
        .expect("★ THE PLANT MUST LAND: the root is the only node declaring 12 pages");
    let mut damaged = bytes.clone();
    damaged[at..at + OLD.len()].copy_from_slice(b"/Count 13");
    assert_eq!(damaged.len(), bytes.len(), "the plant must not move a byte");

    let dir = std::env::temp_dir().join("pdfcer-gui-pagetree-tests");
    std::fs::create_dir_all(&dir).expect("the temporary directory must be creatable");
    let path = dir.join("arrived-broken.pdf");
    std::fs::write(&path, &damaged).expect("the damaged copy is writable");

    let audit = audit_saved_bytes(&damaged);
    assert!(!audit.is_consistent(), "{audit:?}");

    let said = refusal_sentence("arrived-broken.pdf", &audit, Some(path.as_path()));
    assert!(
        said.contains("already disagreed with itself when you opened it"),
        "{said}"
    );
    assert!(!said.contains("Ctrl+Z"), "★★★ undo cannot fix this: {said}");

    // ★ And the control that makes it mean something: the SAME audit with no
    // base file to blame produces the ordinary sentence, which does offer undo.
    // Without this, a `refusal_sentence` that returned the pre-existing wording
    // unconditionally would pass the assertion above.
    let blamed = refusal_sentence("x.pdf", &audit, None);
    assert!(blamed.contains("Ctrl+Z"), "{blamed}");
    assert!(blamed.contains("fault in pdfcer"), "{blamed}");

    let _ = std::fs::remove_file(&path);
}

/// **A base file that is fine gets the ordinary sentence.**
///
/// The other control. `refusal_sentence` reads the file on disk, and a build
/// that mis-read a healthy base as broken would tell the operator every pdfcer
/// defect was somebody else's fault — the failure that is comfortable rather
/// than loud, and therefore the one to assert against.
#[test]
fn a_healthy_base_file_leaves_the_blame_where_it_belongs() {
    let path = fixture("nested-page-tree.pdf");
    // An audit that disagrees, over a base file that does not.
    let g = three_level(4, 4, 1, &[10]);
    let audit = audit(&g);
    assert!(!audit.is_consistent());
    let said = refusal_sentence("nested-page-tree.pdf", &audit, Some(path.as_path()));
    assert!(said.contains("Ctrl+Z"), "{said}");
    assert!(said.contains("fault in pdfcer"), "{said}");
}

/// **The depth a flat tree reports is 2, and a three-level one reports 4.**
///
/// `Audit::depth` is a diagnostic and it is asserted because a *consumer*
/// depends on it: `app::save::tests` refuses to run its engine-half assertion
/// on anything shallower than three levels, and the driven check reads
/// `levels=` off the trace and does the same. Both were added after a
/// falsification in which a flat fixture made a test print *"the engine has
/// been fixed"* about a build carrying the defect in full. A `depth` that was
/// silently always 0 would switch both guards off.
#[test]
fn depth_distinguishes_a_flat_tree_from_a_nested_one() {
    let flat = graph(vec![
        (id(1), node(2, &[10, 11])),
        (id(10), leaf()),
        (id(11), leaf()),
    ]);
    assert_eq!(audit(&flat).depth, 2, "root + leaves");

    let bytes = std::fs::read(fixture("nested-page-tree.pdf")).expect("the fixture is readable");
    let deep = audit_saved_bytes(&bytes);
    assert_eq!(
        deep.depth, 4,
        "★★★ root + A + A1 + leaf. If this ever reads 2 the fixture has been \
         flattened and every assertion that depends on it has quietly stopped \
         being able to fail: {deep:?}"
    );
}
