//! # `app::actions::text` — the verbs that re-shape a page's own text
//!
//! Every variant here has the page's existing text as its subject: the caret
//! commit, the free-text commit, the restyle and the reflow.
//!
//! ## The verbs compose, and that is asserted rather than stated
//!
//! `CommitTextEdit`, `TextStyle` and `Reflow` all **accumulate**: they stage
//! onto the session, and a page may take twenty of them. Add text to a page,
//! then re-wrap a paragraph on it, and the add survives.
//!
//! ⚠ **Do not restate that as a prose limitation.** Whether reflow composes
//! with the other verbs is a property of a branch dependency, so a sentence
//! here is a citation of something that moves with no command run in this
//! repository — and a citation nobody re-measures is a claim about an engine
//! that has changed underneath it. `the_three_text_verbs_compose` below is the
//! re-measurement: it adds a run to `fixtures/paragraph.pdf`, reflows the
//! paragraph on the same page in the same session, and asserts both that the
//! reflow is accepted and that the added run is still on the page. A guard
//! reintroduced upstream turns that test red in the commit that bumps the pin.
//!
//! ## What is worth knowing about `Reflow`'s shape
//!
//! `reflow_block` re-emits the page's **first** content object, and the commit
//! sweep empties every other one — that is by design, not a defect. It is safe
//! because the plan is read from the session's graph and
//! `ContentStream::from_page` concatenates every `/Contents` entry, so an
//! appended run is inside the plan's source and is carried through verbatim
//! rather than dropped. The engine discloses the collapse in its own report —
//! *"multi-stream page: N additional /Contents stream(s) were collapsed into
//! the first"* — and `app::actions::textstyle::reflow` forwards that
//! disclosure to the status line verbatim, which is the whole of what the
//! operator is owed about it.
//!
//! ⚠ One refusal a reader will meet on a CAD sheet, unrelated to any of the
//! above: a paragraph set in a **composite (Type 0 / CIDFont)** face is refused
//! by name, and [`crate::text::textedit::ReflowRefusal::FontIsComposite`] words
//! it. That is an engine feature not yet built rather than a guard, and it is
//! the answer to most reflows on a SOLIDWORKS-exported drawing.

/// The verbs that re-shape a page's own text.
#[derive(Debug, Clone, PartialEq)]
pub enum TextAction {
    /// **Re-wrap a paragraph to its own box.** `OPERATOR_REQUESTS.md`
    /// **O54**.
    ///
    /// Raised by `edit.reflow_block` and by nothing else. It accumulates like
    /// its neighbours — see this module's header, and the test that keeps that
    /// claim honest.
    ///
    /// It carries a **block** index, not a run, and the index is numbered in
    /// the engine's *relaxed* recognition — `reflow_recognition_options()`, the
    /// list `reflow_block` itself builds — never the caret's default one. The
    /// two recognitions group the same page's runs into different numbers of
    /// paragraphs, so passing a default-recognition index here re-wraps the
    /// wrong paragraph. The mapping is made in exactly one place,
    /// `canvas::textedit::reflow::block_of_run`; do not re-derive it.
    Reflow {
        /// The 0-based page.
        page: usize,
        /// Which paragraph on it.
        block: usize,
    },
    /// **Enter was pressed where a line break cannot go** —
    /// `OPERATOR_REQUESTS.md` **O127**, defect 2.
    ///
    /// Raised by `canvas::textedit::keys` when the caret is in an existing show
    /// operator, and by nothing else. It changes **no document**: it exists
    /// solely to carry a sentence from a keystroke handler to the status bar.
    ///
    /// # Why a keystroke needs an `Action` to say something
    ///
    /// Because of a module boundary that is worth keeping. `app::status::decline`
    /// is `pub(super)` inside `crate::app`, with its own note saying why:
    ///
    /// > *"a decline is written by the one dispatcher and read by the one
    /// > bar."*
    ///
    /// `canvas::textedit::keys` is neither, and widening that visibility so a
    /// keystroke could reach the store directly would trade a real invariant
    /// for two saved lines. An `Action` is the channel this shell already has
    /// for *"something in the canvas happened and `crate::app` must react"* —
    /// the same one every commit, every markup and every move travels on.
    ///
    /// It carries no fields, and that is the honest shape: there is exactly
    /// one thing to say, the sentence is in the catalog, and an anchor or a run
    /// index here would be data nobody reads.
    EnterCannotSplit,
    /// **A key was pressed that the caret's run cannot spell** — the
    /// pre-commit half of `OPERATOR_REQUESTS.md` **O140/O141**, 2026-09-09.
    ///
    /// Raised by `canvas::textedit::keys` when
    /// `canvas::textedit::repertoire::sieve` drops a character, and by nothing
    /// else. Like [`Self::EnterCannotSplit`] it changes **no document**: the
    /// keystroke has already been declined by the time this is raised, and this
    /// exists solely to carry the sentence — and the offer — across the
    /// `crate::app` boundary. The same module-visibility argument applies
    /// verbatim; see that variant's docs.
    ///
    /// # Why this one carries fields when its neighbour carries none
    ///
    /// Because there are two surfaces to feed, not one. The status bar gets a
    /// sentence that names the character; `panels::properties::refusedchar`
    /// gets the character *and* the face it was refused against, and offers the
    /// faces that could type it. `EnterCannotSplit` has one thing to say and no
    /// remedy to offer, so it carries nothing.
    ///
    /// `base_font` is the run's `/BaseFont` — the font's own name, `SUBSET+…`
    /// tag and all — because that is what the chooser is replacing and what its
    /// candidate list is measured against. It is **not** the `/Resources /Font`
    /// key.
    ///
    /// # The one thing this variant must never become
    ///
    /// A mark on the page. The draft is left alive and the text in it renders
    /// exactly as it did before the refused key — no red, no strike, no
    /// placeholder glyph, nothing standing in for the character that did not
    /// arrive. **R8b rule 4**: applied content renders as saved content will,
    /// and the disclosure lives off-canvas. What the operator sees is that the
    /// letter did not appear and a sentence in the bar saying why, which is the
    /// same shape every other refusal in this shell takes.
    KeyRefused {
        /// The 0-based page the draft is on.
        page: usize,
        /// The run the caret is in — the same index the repertoire was measured
        /// for, so the offer and the measurement cannot disagree.
        run: usize,
        /// The first character the run's font cannot spell.
        character: char,
        /// The run's `/BaseFont`, which is the face the offer replaces.
        base_font: String,
    },
}

#[cfg(test)]
mod tests {
    //! The header's load-bearing claim, made falsifiable.
    //!
    //! This module exists for one assertion, because the claim is about a
    //! branch dependency: prose here cannot notice the engine changing, and a
    //! test run against the pinned engine can.

    /// **Add, then reflow: the engine composes them, and the added run
    /// survives.**
    ///
    /// # What this measures, precisely
    ///
    /// 1. `add_text` puts a new run on page 0 of `fixtures/paragraph.pdf`. On a
    ///    single-stream page that necessarily creates a **second** `/Contents`
    ///    stream, which is the structural condition a reinstated guard would
    ///    test.
    /// 2. `reflow_block` is then asked to re-wrap block 0 — the fixture's one
    ///    paragraph — in the same session, with no save in between.
    /// 3. The reflow must be **accepted**, and the added text must still be
    ///    extractable from the page afterwards.
    ///
    /// # Why the third step is not redundant
    ///
    /// Step 3 matters most and is the easiest to leave out. The hazard the
    /// composition claim guards against is **silent data loss**, not an error:
    /// reflow's commit sweep empties every `/Contents` stream but the first, so
    /// an engine that accepted the reflow and dropped the added run would
    /// satisfy a test asserting only `is_ok()` — and would be far worse than a
    /// refusal, because nothing would tell the operator. Acceptance and
    /// survival are therefore asserted separately, and the failure messages say
    /// which happened.
    ///
    /// # Why this fixture
    ///
    /// `fixtures/paragraph.pdf` is a flush-left six-line paragraph in
    /// `Helvetica`, a simple (single-byte) face. That matters: a composite face
    /// is refused by name whatever the stream layout, so a CAD sheet cannot
    /// distinguish "a guard came back" from "the font is out of scope" and
    /// would make this test permanently unable to fail for the reason it was
    /// written.
    #[test]
    fn the_three_text_verbs_compose() {
        use pdfcer_core::text_edit::{AddTextRequest, ReflowRequest};

        const ADDED: &str = "composition probe";

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/paragraph.pdf");
        let doc = pdfcer_core::document::Document::load(std::path::Path::new(path))
            .expect("fixtures/paragraph.pdf must load");
        let mut session = pdfcer_core::edit::EditSession::new(doc);

        // --- 1. add a run, which creates the extra content stream -----------
        let add = AddTextRequest::new(0, (72.0, 72.0), ADDED);
        session
            .add_text(&add)
            .expect("adding a 12-pt Helvetica run to a plain fixture must be accepted");

        // --- 2. reflow the fixture's one paragraph, same session -------------
        let reflow = session.reflow_block(0, 0, &ReflowRequest::new());
        let report = match reflow {
            Ok(report) => report,
            Err(e) => panic!(
                "THE GUARD IS BACK, or a new one is. `reflow_block` refused a page that had \
                 text added to it this session: {e}\n\
                 \n\
                 This is the exact behaviour `request_G015` had removed at engine 025d703d, and \
                 the module header above states its absence as a property of the program. If \
                 the engine has deliberately reinstated it, this test and that header are what \
                 must change — together, and with the pin named."
            ),
        };

        // --- 3. the added run must still be on the page ---------------------
        //
        // Read back through the session's own view rather than a saved file:
        // the claim is about what the operator sees in an unsaved session,
        // which is where the loss would have happened.
        let view = session.view();
        let pages = pdfcer_core::page_tree::pages_in(&view).expect("a page tree");
        let text = pdfcer_core::text_extract::extract_page_view(
            &view,
            &pages[0],
            0,
            &pdfcer_core::text_extract::ExtractOptions::default(),
        )
        .expect("the page must still extract after a reflow");
        let survived = text.runs.iter().any(|r| r.text.contains(ADDED));
        assert!(
            survived,
            "SILENT DATA LOSS. The reflow was ACCEPTED (lines {} -> {}) and the run added \
             before it is no longer on the page. That is the failure the removed guard \
             existed to prevent, and it is worse than the refusal: nothing told the \
             operator. Runs now on the page: {:?}",
            report.lines_before,
            report.lines_after,
            text.runs
                .iter()
                .map(|r| r.text.as_str())
                .collect::<Vec<_>>()
        );
    }
}
