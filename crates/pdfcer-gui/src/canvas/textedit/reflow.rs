//! # `canvas::textedit::reflow` — which paragraph the caret is in
//!
//! One question, asked of the document: *the operator's caret is on run N; which
//! **block** is that, and is it one `reflow_block` can act on?*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/reflow.md`.

use pdfcer_core::text_edit::{
    EditableTextModel, TextPosition, detect_cell_regions, reflow_recognition_options,
};

use crate::app::state::OpenDoc;

/// The block index the caret's run belongs to, **in the engine's numbering**,
/// or `None`. The model is the engine's: cell-aware, so a ruled table's cells
/// number after the page's paragraphs; a page whose cells cannot be read is
/// one `reflow_block` refuses too.
#[must_use]
pub fn block_of_run(doc: &OpenDoc, page_index: usize, run: usize) -> Option<usize> {
    // `with_provenance(true)`, which `reflow_block` requires by name — it
    // answers `ReflowApplyError::NoProvenance` without it. The extraction
    // options are otherwise the operator's own, so the runs this addresses are
    // segmented exactly as the runs the canvas paints. Both facts are now
    // properties of the shared cache rather than of this function; see
    // `crate::app::cache::provenance`.
    let text = doc.provenance_page_text(page_index)?;
    let cells = detect_cell_regions(&doc.session.view(), page_index).ok()?;
    let model =
        EditableTextModel::recognize_with_cells(&text, &reflow_recognition_options(), &cells);
    model.block_at(TextPosition::new(run, 0))
}

#[cfg(test)]
mod tests {
    /// **The recognition is the one the ENGINE will index the answer in.**
    #[test]
    fn the_block_is_numbered_the_way_the_engine_will_read_it() {
        let source = include_str!("reflow.rs");
        // The needle is BUILT rather than written, and it has to be: a
        // literal here appears in this very file, so the scan would match its
        // own assertion and pass against wrong code. **A source scan cannot
        // contain its own needle** — the same trap `typing-guard-exempt:
        // SELF-REFERENTIAL` names one module along.
        let engines = format!(
            "recognize_with_cells(&text, &{}(), &cells)",
            "reflow_recognition_options"
        );
        assert!(
            source.contains(&engines),
            "the block lookup no longer numbers its answer the way `reflow_block` reads it"
        );
        // The negative targets the CALL, not the name. The name appears in
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

    /// **THE BEHAVIOURAL ONE: the index this returns must be an index the
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
    /// The literals `3` and `2` are asserted rather than derived. They are
    /// the calibration: if the engine's recogniser changes so the two configs
    /// agree on this fixture, this test goes red saying so, instead of quietly
    /// becoming a test that cannot distinguish the answers — which is exactly
    /// what `ui-verify`'s `reflowing_a_paragraph_rewraps_it` had become on
    /// `fixtures/paragraph.pdf`, where both recognitions say *block 0 of 1*.
    ///
    /// The right-hand side is built from [`reflow_recognition_options`], the
    /// engine's own published function — the same one `reflow_block` calls
    /// internally. An oracle taken from the system under test needs
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
        // Three independent sessions over one file: the `OpenDoc` the shell
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

    /// On a ruled table the engine numbers each cell as its own block, after
    /// the page's paragraphs; the lookup must name the cell, not the row the
    /// plain recognition reads.
    #[test]
    fn a_ruled_tables_cell_is_named_in_the_engines_cell_aware_numbering() {
        use pdfcer_core::text_edit::{
            EditableTextModel, TextPosition, detect_cell_regions, reflow_recognition_options,
        };

        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/ruled-table.pdf");
        let document = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let pages = pdfcer_core::page_tree::pages(&document).expect("a page tree");
        let doc = crate::app::state::OpenDoc::new(
            path.clone(),
            pdfcer_core::edit::EditSession::new(document),
            pages,
        );
        let text = doc.provenance_page_text(0).expect("page 0 extracts");
        let regions = detect_cell_regions(&doc.session.view(), 0).expect("cells are read");
        assert_eq!(
            regions.len(),
            9,
            "the fixture's 3x3 table is no longer found by its rules"
        );
        let plain = EditableTextModel::recognize(&text, &reflow_recognition_options());
        let celled =
            EditableTextModel::recognize_with_cells(&text, &reflow_recognition_options(), &regions);
        let mut differs = 0;
        for run in 0..text.runs.len() {
            let at = TextPosition::new(run, 0);
            assert_eq!(
                super::block_of_run(&doc, 0, run),
                celled.block_at(at),
                "run {run} is named in a numbering `reflow_block` does not use"
            );
            differs += usize::from(plain.block_at(at) != celled.block_at(at));
        }
        assert!(
            differs > 0,
            "the plain and cell-aware numberings agree on every run, so this fixture cannot \
             tell them apart"
        );
    }
}
