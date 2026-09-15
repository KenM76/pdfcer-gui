//! # `canvas::textedit::reflow` — which paragraph the caret is in
//!
//! One question, asked of the document: *the operator's caret is on run N; which
//! **block** is that, and is it one `reflow_block` can act on?*
//!
//! ## ★★★ Why this is a module and not two lines at the call site
//!
//! `OPERATOR_REQUESTS.md` **O54(b)**: *"I think the paragraph reflow was
//! implemented ages ago in the pdfcer core, so we should have that option too."*
//! He was right, and the re-derivation turned up the part that decides the whole
//! design — the integer this module returns is **not a description of a
//! paragraph**. It is an **index into a list the engine rebuilds for itself**,
//! and it means whatever that list says it means.
//!
//! ## ★★★ THE RECOGNITION IS THE ENGINE'S, AND THIS MODULE HAD IT BACKWARDS
//!
//!
//! > The block recognition must match the one the caret was placed against —
//! > `BlockRecognitionOptions::default()`, the same as [`super::plan`]'s. *"The
//! > question is how did the thing the operator clicked get segmented, and
//! > asking it of a differently-recognised model would answer about a different
//! > segmentation."* ★ Not `reflow_recognition_options()` … using it here would
//! > name a block index the operator's caret never pointed at.
//!
//! Every sentence of that is true about *paragraphs* and false about *indices*,
//! and the difference is the whole defect. `EditSession::reflow_block` does not
//! receive a paragraph. It receives an integer, throws away everything the
//! caller knew, re-extracts the page itself, and re-recognises it with
//! [`pdfcer_core::text_edit::reflow_recognition_options`] —
//! `reflow_apply.rs:465` — a **relaxed** config that pushes `indent_ratio` out
//! of reach so ragged-left lines merge into one block instead of fragmenting
//! into one block per line. Then it indexes *that* list.
//!
//! ⇒ **So the only correct thing to send is an index into that list.** Asking
//! the caret's own recognition still answers *"which paragraph did he click
//! in"* — but numbers it in a list the engine will never build, and hands over
//! a subscript into a different array. The engine's own doc comment states the
//! contract in as many words: *"the GUI's caret-block resolution … call THIS
//! function, so the block index the GUI targets means the SAME block the engine
//! previews and the surgery re-emits"* (`reflow.rs:144`). It believed we did.
//!
//! ### What it measured, on `SW41177.pdf` page 0, run 100
//!
//! ```text
//! caret recognition : block 106 of 144
//! reflow recognition: block  49 of  70
//! ```
//!
//! Two failure modes come out of that, and the quiet one is the bad one:
//!
//! 1. **Index ≥ the engine's block count** — 74 of the caret list's 144 indices
//!    on this page — is refused outright: *"block index 106 is out of range (70
//!    block(s) recognised)"*. The operator presses Reflow on a paragraph in
//!    plain sight and is told it does not exist. This is O198's *"seems the
//!    reflow works with each line but …"*: it works low on the page, where the
//!    two lists have not yet drifted apart, and stops working further down.
//! 2. **Index < the engine's block count but past the first disagreement** —
//!    silently re-wraps **a different paragraph**. Nothing refuses, nothing is
//!    marked, the wrong text moves. No test in this crate could see it, because
//!    both option sets produce a valid model and a valid index.
//!
//! ★ The two lists agree at index 0 and diverge at the first place the relaxed
//! config merges what the default split — which on a business letter is never,
//! and on a CAD sheet is almost immediately. That is why this shipped: the only
//! driven check of the feature, `ui-verify`'s `ReflowingAParagraphRewrapsIt`,
//! runs on `fixtures/paragraph.pdf`, a flush-left six-line paragraph that is
//! block 0 of 1 under **both** recognitions. A check whose fixture cannot
//! distinguish the two answers is not a check of which one shipped.
//!
//! ### The trade-off is real, and it is disclosed rather than hidden
//!
//! The relaxed recognition merges paragraphs the default separates, so the
//! block the engine re-wraps can be **larger than the line the operator
//! clicked**. He is owed that fact and he already gets it: the engine's
//! [`ReflowApplyReport`](pdfcer_core::text_edit::ReflowApplyReport) reports
//! `lines_before`/`lines_after` and `crate::app::actions::textstyle::reflow`
//! forwards them to the status line verbatim. A second sentence written here
//! would be a second author for one fact — the rule that module already keeps.
//!
//! ⇒ Nothing is drawn on the canvas about it. R8b rule 4: applied content
//! renders exactly as saved content will; the extent goes off-canvas, in words.

use pdfcer_core::text_edit::{EditableTextModel, TextPosition, reflow_recognition_options};

use crate::app::state::OpenDoc;

/// The block index the caret's run belongs to, **in the engine's numbering**,
/// or `None`.
///
/// `None` for a page whose text cannot be extracted, a run the model does not
/// place in a block, or a caret that is not on a run at all — three states that
/// are one answer here (*"there is no paragraph to reflow"*) and are told apart
/// by the caller only insofar as it says so.
///
/// ★ The run index is the SAME integer in both recognitions — both recognise
/// one extraction, and `BlockRecognitionOptions` groups runs into blocks
/// without renumbering the runs. So asking the relaxed model
/// `block_at(run)` still asks *"which paragraph is the operator's run in"*.
/// Only the answer's numbering changes, and its numbering is the one the
/// engine will read it in.
#[must_use]
pub fn block_of_run(doc: &OpenDoc, page_index: usize, run: usize) -> Option<usize> {
    // ★ `with_provenance(true)`, which `reflow_block` requires by name — it
    // answers `ReflowApplyError::NoProvenance` without it. The extraction
    // options are otherwise the operator's own, so the runs this addresses are
    // segmented exactly as the runs the canvas paints. Both facts are now
    // properties of the shared cache rather than of this function; see
    // `crate::app::cache::provenance`.
    let text = doc.provenance_page_text(page_index)?;
    let model = EditableTextModel::recognize(&text, &reflow_recognition_options());
    model.block_at(TextPosition::new(run, 0))
}

#[cfg(test)]
mod tests {
    /// ★★ **The recognition is the one the ENGINE will index the answer in.**
    ///
    /// A source assertion, and the WEAKER of the two instruments in this module
    /// — the behavioural one below is the real check and should be read first.
    /// This one survives because it fails with a sentence naming the intent,
    /// where a behavioural failure names only a number: both option sets
    /// produce a valid model and a valid block index, so a build using the
    /// wrong one either re-wraps **a different paragraph** than the operator
    /// clicked in — silently, correctly — or is refused for an index out of
    /// range on a paragraph plainly on the screen. Measured on the operator's
    /// own drawing: run 100 is block 106 of 144 to the caret's recognition and
    /// block 49 of 70 to the engine's.
    ///
    /// ⇒ This assertion ran green for the whole time the WRONG option set was
    /// in place — it was written to hold the opposite, on reasoning about
    /// paragraphs rather than about subscripts. It is inverted here rather than
    /// deleted, because the mistake is symmetrical and the next author will
    /// reach for the caret's recognition for exactly the reason the last one
    /// did.
    #[test]
    fn the_block_is_numbered_the_way_the_engine_will_read_it() {
        let source = include_str!("reflow.rs");
        // ★ The needle is BUILT rather than written, and it has to be: a
        // literal here appears in this very file, so the scan would match its
        // own assertion and pass against wrong code. **A source scan cannot
        // contain its own needle** — the same trap `typing-guard-exempt:
        // SELF-REFERENTIAL` names one module along.
        let engines = format!("recognize(&text, &{}())", "reflow_recognition_options");
        assert!(
            source.contains(&engines),
            "the block lookup no longer numbers its answer the way `reflow_block` reads it"
        );
        // ★★ The negative targets the CALL, not the name. The name appears in
        // this module's own header, in the quoted argument that got this wrong
        // — an assertion on the bare string would fail on the documentation
        // that prevents the mistake, which is the shape where a test punishes
        // its own fix.
        let carets = format!(
            "recognize(&text, &{}::default())",
            "BlockRecognitionOptions"
        );
        assert!(
            !source.contains(&carets),
            "the block lookup numbers its answer in the CARET's recognition, which the engine \
             never builds — the index names a different paragraph, or none"
        );
    }

    /// ★★★ **THE BEHAVIOURAL ONE: the index this returns must be an index the
    /// engine accepts, on a fixture that is in this repository.**
    ///
    ///
    /// `fixtures/tail-alignment.pdf` carries right-aligned text — flush right
    /// edges, ragged left — which is the exact shape the two recognitions
    /// disagree about, and the disagreement is large enough to be fatal:
    ///
    /// ```text
    /// page 0, run 2 : caret recognition -> block 3 of 4
    ///                 reflow recognition -> block 2 of 3
    /// ```
    ///
    /// **Block 3 does not exist in the engine's model.** So a build resolving
    /// the caret's recognition hands `reflow_block` a subscript one past the
    /// end and is refused — `BlockIndexOutOfRange(3, 3)` — for a paragraph the
    /// operator can see. That is asserted here both ways round: the wrong
    /// answer is proved to be rejected, and the right answer is proved not to
    /// be rejected *for that reason*.
    ///
    /// ★ The literals `3` and `2` are asserted rather than derived. They are
    /// the calibration: if the engine's recogniser changes so the two configs
    /// agree on this fixture, this test goes red saying so, instead of quietly
    /// becoming a test that cannot distinguish the answers — which is exactly
    /// what `ui-verify`'s `reflowing_a_paragraph_rewraps_it` had become on
    /// `fixtures/paragraph.pdf`, where both recognitions say *block 0 of 1*.
    ///
    /// ★★ The right-hand side is built from [`reflow_recognition_options`], the
    /// engine's own published function — the same one `reflow_block` calls at
    /// `reflow_apply.rs:465`. An oracle taken from the system under test needs
    /// independent calibration, and the calibration is the round trip below:
    /// the index is handed to the real verb and the real refusal is read.
    #[test]
    fn the_index_is_one_the_engine_accepts_on_a_right_aligned_fixture() {
        use pdfcer_core::text_edit::{
            BlockRecognitionOptions, EditableTextModel, ReflowApplyError, ReflowError,
            ReflowRequest, TextPosition, reflow_recognition_options,
        };

        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tail-alignment.pdf");
        assert!(
            path.exists(),
            "the fixture is missing at {}. Regenerate it: python tools/gen-textedit-fixtures.py",
            path.display()
        );
        // ★ Three independent sessions over one file: the `OpenDoc` the shell
        // reads through, and one fresh session per round trip below. They must
        // not share, because `reflow_block` MUTATES on success and a second
        // question asked of a mutated session is a question about a different
        // document.
        let open = || {
            pdfcer_core::edit::EditSession::new(
                pdfcer_core::document::Document::load(&path).expect("the fixture loads"),
            )
        };
        let document = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let pages = pdfcer_core::page_tree::pages(&document).expect("a page tree");
        let doc = crate::app::state::OpenDoc::new(
            path.clone(),
            pdfcer_core::edit::EditSession::new(document),
            pages,
        );

        // The run the census named. Right-aligned body text, not the heading.
        const RUN: usize = 2;

        // --- the calibration: the two recognitions still disagree here ------
        let extracted = doc
            .provenance_page_text(0)
            .expect("the fixture's page 0 extracts with provenance");
        let carets = EditableTextModel::recognize(&extracted, &BlockRecognitionOptions::default());
        let engines = EditableTextModel::recognize(&extracted, &reflow_recognition_options());
        assert_eq!(
            carets.block_at(TextPosition::new(RUN, 0)),
            Some(3),
            "the caret's recognition no longer puts run {RUN} in block 3 — this fixture can no \
             longer tell the two numberings apart, so the assertions below prove nothing"
        );
        assert_eq!(
            engines.block_at(TextPosition::new(RUN, 0)),
            Some(2),
            "the engine's recognition no longer puts run {RUN} in block 2"
        );
        assert_eq!(engines.blocks().len(), 3, "the engine's block count moved");

        // --- what the shell sends -------------------------------------------
        assert_eq!(
            super::block_of_run(&doc, 0, RUN),
            Some(2),
            "the reflow target is numbered in the caret's recognition, which the engine never \
             builds — so it names a different paragraph, or none at all"
        );

        // --- the round trip, which is what makes the numbers mean something --
        //
        // The wrong answer, handed to the real verb, is refused BY INDEX. This
        // is the operator-visible half of the defect: press Reflow, be told the
        // paragraph under the caret does not exist.
        let mut wrong = open();
        assert!(
            matches!(
                wrong.reflow_block(0, 3, &ReflowRequest::new()),
                Err(ReflowApplyError::Preview(
                    ReflowError::BlockIndexOutOfRange(3, 3)
                ))
            ),
            "block 3 is no longer out of range for the engine, so the caret's numbering is no \
             longer provably fatal on this fixture and this test has stopped measuring"
        );

        // And the right answer is not refused for THAT reason. It may still be
        // refused — `tail-alignment.pdf` exists to carry rotated and
        // right-aligned text and the engine defers some of it — and a refusal
        // about the block's CONTENT is a different fact from a refusal about
        // the caller's arithmetic. Only the arithmetic is this module's.
        let mut right = open();
        assert!(
            !matches!(
                right.reflow_block(0, 2, &ReflowRequest::new()),
                Err(ReflowApplyError::Preview(
                    ReflowError::BlockIndexOutOfRange(..)
                ))
            ),
            "the index this module produces is out of range for the engine"
        );
    }
}
