//! # `pagetree::tests` — and the one shape that would make all of them vacuous
//!
//! **A test on a FLAT page tree defeats this entire module.**
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/pagetree/tests.md`.

// The INNER `#![cfg(test)]` is redundant — the module is declared
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
#[test]
fn a_healthy_nested_tree_is_consistent() {
    let g = three_level(4, 4, 2, &[10, 11]);
    let audit = audit(&g);
    assert!(audit.walked);
    assert_eq!(audit.reachable_pages, 4);
    assert_eq!(audit.declared_pages, Some(4));
    assert!(audit.is_consistent(), "{audit:?}");
}

/// **A stale ROOT above a CORRECT immediate parent is caught.**
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

/// **A node whose whole subtree was removed is caught** — the case §5 says
/// `PageSlot::ancestors` structurally cannot see.
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
    // THE FIXTURE MUST BE NESTED. Seven `/Pages` nodes, three levels. A
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

/// **The real corrupt file is caught** — the positive control that is not
/// a hand-made imitation.
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

    //
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
// Which refusal is owed — `refusal_origin`
// ==========================================================================

/// **A file that arrived broken is classed as pre-existing**, which is the
/// refusal whose sentence withholds Ctrl+Z: undo cannot fix what pdfcer did
/// not do, and an operator who empties his undo stack against it loses his
/// work as well as his time.
///
/// The base file is the fixture with one digit changed: a real, openable PDF
/// whose root declares one page more than it has.
#[test]
fn a_document_that_arrived_broken_is_refused_as_pre_existing() {
    let bytes = std::fs::read(fixture("nested-page-tree.pdf")).expect("the fixture is readable");
    const OLD: &[u8] = b"/Count 12";
    let at = bytes
        .windows(OLD.len())
        .position(|w| w == OLD)
        .expect("the plant must land: the root is the only node declaring 12 pages");
    let mut damaged = bytes.clone();
    damaged[at..at + OLD.len()].copy_from_slice(b"/Count 13");
    assert_eq!(damaged.len(), bytes.len(), "the plant must not move a byte");

    let dir =
        std::env::temp_dir().join(format!("pdfcer-gui-pagetree-tests-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the temporary directory must be creatable");
    let path = dir.join("arrived-broken.pdf");
    std::fs::write(&path, &damaged).expect("the damaged copy is writable");

    let audit = audit_saved_bytes(&damaged);
    assert!(!audit.is_consistent(), "{audit:?}");

    assert_eq!(
        refusal_origin(&audit, Some(path.as_path())),
        RefusalOrigin::PreExisting {
            declared: 13,
            reachable: 12
        }
    );
    // The control: the same audit with no base file to consult has no
    // evidence the file arrived broken, so it is the session's.
    assert_eq!(
        refusal_origin(&audit, None),
        RefusalOrigin::Root {
            declared: 13,
            reachable: 12
        }
    );

    let _ = std::fs::remove_file(&path);
}

/// **A healthy base file leaves the refusal with the session.** A build that
/// mis-read a healthy base as broken would withhold Ctrl+Z when one press
/// would have fixed it — comfortable rather than loud, so asserted.
#[test]
fn a_healthy_base_file_leaves_the_blame_where_it_belongs() {
    let path = fixture("nested-page-tree.pdf");
    // An audit that disagrees, over a base file that does not.
    let g = three_level(4, 4, 1, &[10]);
    let audit = audit(&g);
    assert!(!audit.is_consistent());
    assert!(
        !matches!(
            refusal_origin(&audit, Some(path.as_path())),
            RefusalOrigin::PreExisting { .. }
        ),
        "the base file is HEALTHY, so this refusal must not be classed as pre-existing"
    );
}

/// **The depth a flat tree reports is 2, and a three-level one reports 4.**
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
