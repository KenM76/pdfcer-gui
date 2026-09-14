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
//! ## ★★★ One of these three is not like the others, and it is the whole reason
//! ## this module has prose
//!
//! `CommitTextEdit` and `TextStyle` **accumulate**: they stage onto the session,
//! and a page may take twenty of them.
//!
//! `Reflow` does not, and **the reason changed under us on 2026-09-06** — this
//! paragraph was re-measured against engine `Pass 257.0` on 2026-09-14 and what
//! it used to say is no longer true.
//!
//! It used to say: `reflow_block` plans against the **base** document, so it
//! refuses a page this session has already rewritten, and one typed character
//! trips it. `Pass 257.0` moved the planner onto the **session view** — the
//! same graph every other read uses — and both "save and reopen" refusals went
//! with it. An ordinary text edit no longer blocks a later reflow.
//!
//! **What still makes `Reflow` unlike its neighbours** is narrower and is worth
//! stating precisely. `reflow_block` re-emits the page's FIRST content stream
//! and the commit sweep empties every other one. `add_text` puts new text in a
//! new stream. ★★★ So reflow refuses a page carrying a non-empty extra content
//! stream — `ReflowApplyError::PageEditedThisSession` — because committing
//! would silently delete text the operator can see. **Adding** text trips it;
//! **editing** existing text does not, because that edit's own sweep has
//! already consolidated the page.
//!
//! ★★ And the same guard fires on a page NO ONE edited, because the condition
//! is structural rather than provenance-based: a producer that splits page
//! content across streams (SOLIDWORKS does; ISO 32000-1 7.8.2 permits it) is
//! read as "text was added this session". Filed as `request_G015`; see O198.
//! Until it lands, the remedy sentence this shell shows is the engine's and is
//! wrong on that one class of page, and there is nothing honest to substitute.
//!
//! ⇒ A reader who assumes the three behave alike will wire a reflow after an
//! **add** and meet a refusal that looks like a bug. It is a correctness
//! property, and the sentence saying so is as much the feature as the wrapping
//! is.

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
