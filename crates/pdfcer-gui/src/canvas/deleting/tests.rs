//! # `canvas::deleting::tests` — which verb each rung reaches, asserted
//!
//! Split out of [`super`] the way `canvas::moving::tests` and
//! `canvas::selection::tests` were, and for their reason: the module above is
//! one subject — *which delete verb does this selection reach, and what does it
//! refuse* — and the assertions are a different subject with a different reader.
//!
//! ## ★★★ What these are actually protecting, and it is not the routing
//!
//! The routing is five arms and a match; a reviewer can read it. What cannot be
//! read is the property the whole feature exists for, and it is the one a wrong
//! build gets wrong **while looking right**:
//!
//! > **A Part-rung delete on a text object must reach `delete_text_run` and
//! > never `delete_objects`.**
//!
//! On the operator's SolidWorks export one text object holds **all 237 pdf
//! dimension labels** and one path object holds **1,194 subpaths**. A build that
//! borrowed the Object rung's verb for the Part rung deletes every label on the
//! sheet, or a whole drawing view, in answer to *"remove this line"* — and it
//! reports success. [`selecting_one_label_deletes_that_label_and_not_the_object`]
//! is the assertion that forbids it, and it is deliberately written as *which
//! variant*, not *did something happen*.
//!
//! ## `#![cfg(test)]` at the top, and why it is the marker rather than the name
//!
//! `check-ui-strings.sh` and `check-theme-colors.sh` both recognise the inner
//! attribute as meaning *"none of this is in the shipped binary"*, and both
//! state why they match on that rather than on a filename: the property that
//! earns the exemption is not being in the binary. Without it every `assert!`
//! message below is reported as un-catalogued operator copy.
//!
//! ★ `check-file-size.sh` still counts these lines. This is the split R2 asks
//! for, not a way of hiding from it.

#![cfg(test)]

use super::{DeleteSubject, Refusal, action, subject};
use crate::app::actions::VectorAction;
use crate::canvas::selection::{ClickHit, SelectionLevel, SelectionState};
use crate::canvas::target::TargetId;
use crate::panels::objects::provider::ObjectModelProvider;
use pdfcer_core::content::ContentStream;
use pdfcer_core::vector::{Matrix, NoXObjects, decompose};
// ★ The renderer's own transform type, named through the crate that defines it
// rather than through `crate::viewer` — that module imports it privately, so the
// path a reader would guess does not resolve.
use pdfcer_render::tiny_skia::Transform;

/// Build a provider over one content stream, at the identity transform.
///
/// The seam `provider::node_rung_tests` uses, and for its reason: the adapter
/// logic is proven without a live `Document` or an egui frame.
fn provider(src: &[u8]) -> ObjectModelProvider {
    let cs = ContentStream::parse(src.to_vec()).expect("the fixture stream parses");
    let objects = decompose(&cs, Matrix::IDENTITY, &NoXObjects);
    ObjectModelProvider::from_parts(0, objects, Transform::identity())
}

/// ★★★ **A TEXT FIXTURE HAS TO BE A REAL DOCUMENT, and finding that out is
/// worth the paragraph.**
///
/// The obvious spelling — `decompose` over a hand-written `BT /F1 10 Tf …
/// (12.5) Tj … ET` — produces a `TextObject` with **zero runs**, silently.
/// `close_text_run` drops a run whose `current_run` bounds are empty, and
/// bounds come from glyph widths, which come from a resolved font; a bare
/// `ContentStream` has no `/Resources` and therefore no font.
///
/// That is not a curiosity, it is a trap with teeth: `text_run_count` then
/// answers `0`, `text_run_delete_would_move_next` answers `false` because it
/// requires `runs.len() > 1`, and a test written that way **passes for the
/// wrong reason** — it asserts a routing decision made over an object that has
/// no parts. `provider::tests::part_kind_and_part_count_answer_for_every_object_kind`
/// hits the same wall and works around it by asserting
/// `part_count(0) == text_run_count(0)` rather than a number.
///
/// So the text rungs are exercised against documents on disk. The path rungs
/// are not, because path geometry needs no font.
///
/// **Six runs, all explicitly placed** — one text object holding several
/// labels, which is what makes "delete a label" and "delete the text object"
/// two different acts.
const SIX_LABELS: &str = "paragraph.pdf";

/// **The engine's own §9.4.2 fixture**: run 1 is `RunPositioning::Inherited`,
/// so deleting run 0 would slide it. `crates/pdfcer-core/tests/text_run_delete.rs`
/// asserts that property of this file, which is what makes it safe to build on.
const INHERITING_LABEL: &str = "text/runs-inherited.pdf";

/// One path object holding **two lines**, the shape of a CAD view in miniature.
const TWO_LINES: &[u8] = b"0 0 m 10 0 l 20 0 l h 40 40 m 60 40 l 60 60 l S";

/// Ask [`subject`] against a document's real page-0 object model, and report
/// which object was used so a failure names the thing that was asked about.
///
/// Takes a closure rather than returning the provider because `page_objects`
/// hands back a `Ref` borrowed from the `OpenDoc`, and a helper returning one
/// would have to return the document too.
fn against(
    doc: &crate::app::state::OpenDoc,
    selection: &SelectionState,
) -> Result<DeleteSubject, Refusal> {
    let provider = doc.page_objects().expect("the fixture page decomposes");
    subject(selection, 0, Some(&provider))
}

/// The paint-order index of the first text object on page 0 that has at least
/// `runs` runs, so a fixture edit that renumbers the page does not silently
/// re-aim the assertions at a different object.
fn text_object_with(doc: &crate::app::state::OpenDoc, runs: usize) -> usize {
    let provider = doc.page_objects().expect("the fixture page decomposes");
    (0..provider.page_objects().objects.len())
        .find(|&i| provider.text_run_count(i) >= runs)
        .unwrap_or_else(|| panic!("no text object on page 0 holds {runs} runs"))
}

/// A selection sitting at the Part rung on `object`, part `part`.
fn at_part(object: u64, part: usize) -> SelectionState {
    let mut selection = SelectionState::default();
    // Two double-clicks would descend to the Node rung; one enters the Part.
    selection.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(object)),
            part: Some(part),
            node: None,
        },
        false,
        true,
    );
    assert_eq!(selection.level(), SelectionLevel::Part);
    selection
}

/// A selection sitting at the Node rung on `object`, anchor `node`.
fn at_node(object: u64, part: usize, node: usize) -> SelectionState {
    let mut selection = at_part(object, part);
    selection.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(object)),
            part: Some(part),
            node: Some(node),
        },
        false,
        true,
    );
    assert_eq!(selection.level(), SelectionLevel::Node);
    selection
}

/// ★★★ **THE ASSERTION `Pass 32.0` EXISTS FOR.**
///
/// A label is selected — the Part rung on a text object — and Delete must reach
/// `delete_text_run` with that run's index. The variant is the whole claim: a
/// build that answered [`DeleteSubject::Objects`] here would pass any test that
/// only asked whether *something* was deleted, and would remove all 237 labels
/// on the operator's sheet.
#[test]
fn selecting_one_label_deletes_that_label_and_not_the_object() {
    let doc = crate::app::state::open_local_fixture(SIX_LABELS);
    let object = text_object_with(&doc, 2);
    let selection = at_part(object as u64, 1);
    assert_eq!(
        against(&doc, &selection),
        Ok(DeleteSubject::TextRun {
            page: 0,
            object,
            run: 1,
        }),
        "the Part rung on a text object must reach `delete_text_run`, not the \
         whole-object verb"
    );
    assert_eq!(
        action(DeleteSubject::TextRun {
            page: 0,
            object,
            run: 1,
        }),
        VectorAction::DeleteTextRun {
            page: 0,
            object,
            run: 1,
        }
    );
}

/// ★★ **R83, asked before the press.**
///
/// The second label has no position of its own, so removing the first would
/// slide it. The engine refuses with `DeleteWouldMoveNextRun`; this refuses
/// first, from the same `positioned_by` flag, so the operator gets the remedy
/// instead of a cause-less decline.
#[test]
fn deleting_a_label_that_would_move_the_next_one_is_refused_by_name() {
    let doc = crate::app::state::open_fixture(INHERITING_LABEL);
    let object = text_object_with(&doc, 2);
    let selection = at_part(object as u64, 0);
    assert_eq!(
        against(&doc, &selection),
        Err(Refusal::RunWouldMoveNext(0)),
        "§9.4.2: the following run inherits its position, so this delete must be \
         refused before it is raised"
    );
}

/// …and the LAST label of the same object is deletable, because nothing follows
/// it to be moved. The pair matters: a guard that refused both would be an
/// over-broad refusal, which teaches an operator that the tool says no for no
/// reason.
#[test]
fn the_last_label_is_deletable_even_when_the_earlier_one_is_not() {
    let doc = crate::app::state::open_fixture(INHERITING_LABEL);
    let object = text_object_with(&doc, 2);
    let selection = at_part(object as u64, 1);
    assert_eq!(
        against(&doc, &selection),
        Ok(DeleteSubject::TextRun {
            page: 0,
            object,
            run: 1,
        })
    );
}

/// ★★★ **One line out of a path that holds many.**
///
/// The twin of the label case, and the one whose move verb has been wired since
/// Pass 28.0 — so until 2026-09-05 this exact selection could be **dragged** and
/// not removed.
#[test]
fn selecting_one_line_deletes_that_line_and_not_the_drawing_view() {
    let provider = provider(TWO_LINES);
    let selection = at_part(0, 1);
    assert_eq!(
        subject(&selection, 0, Some(&provider)),
        Ok(DeleteSubject::Subpath {
            page: 0,
            object: 0,
            subpath: 1,
        }),
        "the Part rung on a path must reach `delete_subpath`, not the whole-object \
         verb — one measured export holds a whole isometric view in one object"
    );
}

/// ★★★ **One corner point.**
#[test]
fn selecting_one_point_deletes_that_point() {
    let provider = provider(TWO_LINES);
    let selection = at_node(0, 0, 1);
    assert_eq!(
        subject(&selection, 0, Some(&provider)),
        Ok(DeleteSubject::Node {
            page: 0,
            object: 0,
            node: 1,
        })
    );
}

/// ★★ **Several anchors refuse rather than removing one of them.**
///
/// `move_nodes` takes a slice and `delete_node` is singular, so a multi-anchor
/// delete would be N commands and N undo entries — and each excision renumbers,
/// so the second index would be planned against offsets the first invalidated.
/// Acting on the entered one alone is the `selected_nodes_on` defect: four
/// anchors highlighted, one removed, nothing said.
#[test]
fn several_selected_points_refuse_and_say_how_many() {
    let provider = provider(TWO_LINES);
    let mut selection = at_node(0, 0, 1);
    selection.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(0),
            node: Some(2),
        },
        true,
        false,
    );
    assert_eq!(
        subject(&selection, 0, Some(&provider)),
        Err(Refusal::ManyNodes(2))
    );
}

/// The Node rung on **text** has no verb: a glyph is not an anchor, and
/// `pdfcer-core` has nothing that removes one character from a show operator.
#[test]
fn the_point_rung_on_text_declines_by_name() {
    let doc = crate::app::state::open_local_fixture(SIX_LABELS);
    let object = text_object_with(&doc, 2);
    let mut selection = at_part(object as u64, 0);
    // A second descent enters the Node rung even with no anchor under the
    // pointer — "inside this part, nothing picked yet" is a real state — so the
    // entry is given a node explicitly to reach the arm under test.
    selection.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(object as u64)),
            part: Some(0),
            node: Some(0),
        },
        false,
        true,
    );
    assert_eq!(against(&doc, &selection), Err(Refusal::NoNodeVerbForText));
}

/// ★★ **The Object rung still works with no object model at all.**
///
/// The asymmetry `subject`'s docs argue for: the Object rung is answered from
/// the selection alone, so a page that will not decompose can still have its
/// objects deleted. A signature that demanded a provider would have invented a
/// limit.
#[test]
fn the_object_rung_needs_no_object_model() {
    let mut selection = SelectionState::default();
    selection.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(4)),
            ..ClickHit::default()
        },
        false,
        false,
    );
    assert_eq!(
        subject(&selection, 0, None),
        Ok(DeleteSubject::Objects {
            page: 0,
            objects: vec![4],
        })
    );
}

/// …and the deeper rungs decline by name when it is absent, rather than
/// guessing at a part kind.
#[test]
fn a_deeper_rung_without_an_object_model_declines_by_name() {
    let selection = at_part(0, 1);
    assert_eq!(subject(&selection, 0, None), Err(Refusal::NoObjectModel));
}

/// A selection made entirely of form-interior targets reaches
/// `delete_objects_in_form` at the Object rung — unchanged behaviour, asserted
/// because the routing moved.
#[test]
fn a_form_interior_selection_still_reaches_the_form_delete() {
    let mut selection = SelectionState::default();
    selection.click(
        0,
        ClickHit {
            object: Some(TargetId::Leaf(7)),
            ..ClickHit::default()
        },
        false,
        false,
    );
    assert_eq!(
        subject(&selection, 0, None),
        Ok(DeleteSubject::LeavesInForm {
            page: 0,
            leaves: vec![7],
        })
    );
}

/// ★★ **A LINE inside a form has no delete verb, and says so.**
///
/// Measured against the locked engine rather than assumed: its six
/// form-interior verbs are five moves and one whole-object delete. There is no
/// `delete_subpath_in_form`. Naming it is what turns a dead key into a limit
/// the operator is told about — and the sentence points at what does work.
#[test]
fn a_part_inside_a_form_declines_because_the_engine_has_no_verb() {
    let mut selection = SelectionState::default();
    selection.click(
        0,
        ClickHit {
            object: Some(TargetId::Leaf(7)),
            part: Some(2),
            node: None,
        },
        false,
        true,
    );
    assert_eq!(selection.level(), SelectionLevel::Part);
    assert_eq!(subject(&selection, 0, None), Err(Refusal::NoObjectModel));
    // With a model present the answer is the honest one about the address space
    // rather than about the missing decomposition.
    let provider = provider(TWO_LINES);
    assert_eq!(
        subject(&selection, 0, Some(&provider)),
        Err(Refusal::InsideForm)
    );
}

/// An empty selection deletes nothing and is not narrated. The pair with the
/// arm above is the point: *"you selected nothing"* and *"you selected
/// something this verb cannot reach"* are the two states an operator most needs
/// kept apart.
#[test]
fn nothing_selected_is_its_own_refusal() {
    let selection = SelectionState::default();
    assert_eq!(subject(&selection, 0, None), Err(Refusal::NothingSelected));
    assert!(
        crate::text::deleting::refusal(Refusal::NothingSelected).is_none(),
        "a state the operator can see must not be narrated"
    );
}

/// An entry belonging to another page must never address this page's index
/// space. One comparison rules out a whole class of wrong-sheet edits.
#[test]
fn an_entry_on_another_page_is_not_deletable_here() {
    let provider = provider(TWO_LINES);
    let selection = at_part(0, 1);
    assert_eq!(
        subject(&selection, 3, Some(&provider)),
        Err(Refusal::NothingSelected)
    );
}
