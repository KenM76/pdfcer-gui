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
//! `Reflow` does not. `EditSession::reflow_block` plans against the **base**
//! document — it re-extracts and re-recognises the page to get provenance the
//! staging buffer does not carry — and therefore **refuses a page this session
//! has already rewritten**, by name, rather than mis-splicing base-relative byte
//! offsets into a stream that has moved.
//!
//! ⇒ A reader who assumes the three behave alike will wire a reflow after an
//! edit and meet a refusal that looks like a bug. It is a correctness property
//! with a real remedy — **save and reopen** — and the sentence saying so is as
//! much the feature as the wrapping is.

/// The verbs that re-shape a page's own text.
#[derive(Debug, Clone, PartialEq)]
pub enum TextAction {
    /// ★★★ **Re-wrap a paragraph to its own box.** `OPERATOR_REQUESTS.md`
    /// **O54**.
    ///
    /// Raised by `edit.reflow_block` and by nothing else.
    /// **`canvas::textedit::reflow`'s header is the argument** — the short of it
    /// is that this verb does NOT accumulate like its neighbours: it is planned
    /// against the *base* document and refuses a page this session has already
    /// rewritten, by name, rather than mis-splicing. One typed character trips
    /// it, and the remedy is to save and reopen.
    ///
    /// ★ It carries a BLOCK index, not a run. That mapping is made in exactly
    /// one place, against the caret's own block recognition rather than the
    /// engine's relaxed one, because the two segment a page differently.
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
}
