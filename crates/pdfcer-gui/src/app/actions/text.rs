//! # `app::actions::text` — the verbs that re-shape a page's own text
//!
//! Split out of [`super::action`] under **R2** on 2026-08-28, when paragraph
//! reflow landed and took that file past 1,500 lines for the fourth time.
//!
//! ## ★★ Why THIS family, when the file's own header names markup
//!
//! [`super::action`]'s header pre-measured the next sub-enum and named
//! **markup** — some 370 lines, still the largest. The rule it stated is *"the
//! next family of variants to **grow**"*, and today that was this one: reflow is
//! the fourth verb whose subject is the page's existing text, after the caret
//! commit, the free-text commit and the restyle.
//!
//! ⇒ The markup measurement stands and is still the answer the day markup grows.
//! Taking a different family because it is bigger would be re-deciding a
//! decision on a criterion nobody chose.
//!
//! ## ★★★ The three verbs DO compose now, and the history of this paragraph
//! ## is the reason it is an assertion rather than a sentence
//!
//! `CommitTextEdit` and `TextStyle` **accumulate**: they stage onto the
//! session, and a page may take twenty of them. **So does `Reflow`, as of
//! engine `025d703d`** — add text to a page, then re-wrap a paragraph on it,
//! and the add survives.
//!
//! ⚠⚠⚠ **This paragraph has said the opposite twice, and both times it was
//! right when written and wrong by the time it was read.**
//!
//! | version | claim | true at | falsified by |
//! |---|---|---|---|
//! | 1 | reflow plans against the BASE document, so one typed character blocks it | engine `Pass 251.0` | `Pass 257.0` (2026-09-06) moved the planner onto the session view |
//! | 2 | reflow still refuses a page carrying a non-empty EXTRA content stream, so **adding** text blocks it even though editing does not | engine `7378c838` | `G015` (2026-09-14) deleted the guard |
//!
//! Version 2 was written on 2026-09-14 while correcting version 1, from the
//! engine source, and it was accurate for about six hours. The guard it
//! described was already known to be wrong — its condition was **structural**
//! (*does `contents[1..]` hold a non-empty stream*), which ISO 32000-1 §7.8.2
//! permits a producer to author and which SOLIDWORKS does routinely, so it
//! fired on pages nobody had edited. That was `request_G015`, filed from
//! `SW41177.pdf`, whose title sheet carries eight producer-authored streams and
//! was refused with *"text was added to this page this session"* on a freshly
//! opened file. The engine agreed and removed it.
//!
//! ⇒ **Version 3 is a test, not a sentence.** `the_three_text_verbs_compose`
//! below adds a run to `fixtures/paragraph.pdf` and then reflows the paragraph
//! on the same page, in that order, and asserts the reflow is accepted and the
//! added run is still on the page afterwards. The day the engine reintroduces a
//! guard of either shape, that test goes red in the same commit that bumps the
//! pin — which is the only mechanism that has ever caught this. *A limitation
//! sentence is a citation, and a citation nobody re-measures is a claim about
//! an engine that has moved.*
//!
//! ## ★★ What is still worth knowing about `Reflow`'s shape
//!
//! `reflow_block` re-emits the page's **first** content object, and the commit
//! sweep empties every other one — that has not changed and is not a defect.
//! What changed is where the plan is read FROM: since `Pass 257.0` it is the
//! session's graph, and `ContentStream::from_page` concatenates every
//! `/Contents` entry, so an appended run is inside the plan's source and is
//! carried through verbatim rather than dropped. The engine discloses the
//! collapse in its own report — *"multi-stream page: N additional /Contents
//! stream(s) were collapsed into the first"* — and
//! `app::actions::textstyle::reflow` forwards that disclosure to the status
//! line verbatim, which is the whole of what the operator is owed about it.
//!
//! ⚠ One refusal a reader WILL still meet on a CAD sheet, and it is unrelated
//! to any of the above: a paragraph set in a **composite (Type 0 / CIDFont)**
//! face is refused by name (`R-INV-4`, FF-E), and
//! [`crate::text::textedit::ReflowRefusal::FontIsComposite`] words it. That is
//! a deferred engine feature rather than a guard, and it is the answer to every
//! reflow on `SW41177.pdf`.

/// The verbs that re-shape a page's own text.
#[derive(Debug, Clone, PartialEq)]
pub enum TextAction {
    /// ★★★ **Re-wrap a paragraph to its own box.** `OPERATOR_REQUESTS.md`
    /// **O54**.
    ///
    /// Raised by `edit.reflow_block` and by nothing else.
    /// **This module's header is the argument** — the short of it is that this
    /// verb does not accumulate like its neighbours: it refuses a page carrying
    /// a non-empty EXTRA content stream, because it re-emits the first one and
    /// the commit sweep empties the rest. Adding text trips it; editing text
    /// does not. (Corrected 2026-09-14: this said "planned against the base
    /// document" and "one typed character trips it" for eight days after engine
    /// `Pass 257.0` made both false.)
    ///
    /// ★★★ It carries a BLOCK index, not a run, and the index is numbered in
    /// **the engine's relaxed recognition** — `reflow_recognition_options()`,
    /// the list `reflow_block` itself builds — never the caret's. That mapping
    /// is made in exactly one place, `canvas::textedit::reflow::block_of_run`,
    /// whose header carries the measurement: on the operator's own drawing the
    /// two recognitions number the same paragraph 106-of-144 and 49-of-70. This
    /// doc comment asserted the opposite until 2026-09-14, and so did the code.
    Reflow {
        /// The 0-based page.
        page: usize,
        /// Which paragraph on it.
        block: usize,
    },
    /// ★★★ **Enter was pressed where a line break cannot go** —
    /// `OPERATOR_REQUESTS.md` **O127**, defect 2.
    ///
    /// Raised by `canvas::textedit::keys` when the caret is in an existing show
    /// operator, and by nothing else. It changes **no document**: it exists
    /// solely to carry a sentence from a keystroke handler to the status bar.
    ///
    /// # ★★ Why a keystroke needs an `Action` to say something
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
    /// ★ It carries no fields, and that is the honest shape: there is exactly
    /// one thing to say, the sentence is in the catalog, and an anchor or a run
    /// index here would be data nobody reads.
    ///
    /// ★★ Nested under [`TextAction`] rather than added to
    /// `super::action::Action`, per **R2**: that file is at its 1,500-line
    /// ceiling and its own header names this as the remedy — *"nest a domain
    /// enum"*. The subject fits: this variant's subject is the page's own text
    /// and the caret in it, which is what this enum is for.
    EnterCannotSplit,
    /// ★★★ **A key was pressed that the caret's run cannot spell** — the
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
    /// # ★★ Why this one carries fields when its neighbour carries none
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
    /// # ★★★ The one thing this variant must never become
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
    //! ★★★ The header's load-bearing claim, made falsifiable.
    //!
    //! This module exists for one assertion. See the table above for why a
    //! prose claim here is not good enough: the paragraph it replaces has been
    //! wrong twice in nine days, both times because an engine guard was removed
    //! and nothing in this crate could notice.

    /// ★★★ **Add, then reflow: the engine composes them, and the added run
    /// survives.**
    ///
    /// # What this measures, precisely
    ///
    /// 1. `add_text` puts a new run on page 0 of `fixtures/paragraph.pdf`. On a
    ///    single-stream page that necessarily creates a SECOND `/Contents`
    ///    stream, which is exactly the condition the deleted guard tested.
    /// 2. `reflow_block` is then asked to re-wrap block 0 — the fixture's one
    ///    paragraph — in the same session, with no save in between.
    /// 3. The reflow must be **accepted**, and the added text must still be
    ///    extractable from the page afterwards.
    ///
    /// # ★★ Why all three steps, and why the third is not redundant
    ///
    /// Step 3 is the one that matters most and is the easiest to leave out. The
    /// guard that was removed existed to prevent **silent data loss**, not to
    /// prevent an error: its argument was that the plan read the base document,
    /// which did not contain the appended run, and committing that plan ran a
    /// sweep that emptied every extra stream. An engine that accepted the
    /// reflow and dropped the added text would satisfy a test asserting only
    /// `is_ok()` — and would be a far worse defect than the refusal.
    ///
    /// ⇒ So the acceptance and the survival are asserted separately, and the
    /// failure messages say which happened.
    ///
    /// # ★ Why this fixture
    ///
    /// `fixtures/paragraph.pdf` is a flush-left six-line paragraph in
    /// `Helvetica`, a simple (single-byte) face. That matters: a composite face
    /// is refused by name (`R-INV-4`) whatever the stream layout, so a CAD
    /// sheet cannot distinguish "the guard came back" from "the font is out of
    /// scope" and would make this test permanently unable to fail for the
    /// reason it was written.
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
