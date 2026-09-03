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
//! design — `EditSession::reflow_block`'s own words:
//!
//! > The reflow is planned against the **base** document's content (it extracts
//! > + recognises the page fresh, needing provenance the staging buffer does not
//! > carry), so — unlike the accumulating `edit_text`/`format_text` — it
//! > **refuses when the page's content object was already rewritten this
//! > session** (a prior text or format edit). Save and reopen to reflow after an
//! > in-session edit of the same page. This is a clean, named refusal, never a
//! > silent mis-splice.
//!
//! ⇒ **So this verb is not like its neighbours and the shell must not pretend it
//! is.** Every other editing verb in this crate accumulates; this one is
//! base-relative, and an operator who has typed one character on a page has
//! already made it refuse. That is a real limit with a real remedy — save and
//! reopen — and the sentence saying so is the feature as much as the wrap is.
//!
//! ## ★★ The block recognition must match the one the caret was placed against
//!
//! `BlockRecognitionOptions::default()`, the same as
//! [`super::plan`]'s — *"the question is how did the thing the operator clicked
//! get segmented, and asking it of a differently-recognised model would answer
//! about a different segmentation."*
//!
//! ★ Not `reflow_recognition_options()`, which the engine also publishes. That
//! one is a **relaxed** recognition used to detect alignment on blocks the
//! default splits apart, and `cost.rs` and `plan` both use it for exactly that.
//! Using it here would name a block index the operator's caret never pointed at.

use pdfcer_core::text_edit::{BlockRecognitionOptions, EditableTextModel, TextPosition};

use crate::app::state::OpenDoc;

/// The block index the caret's run belongs to, or `None`.
///
/// `None` for a page whose text cannot be extracted, a run the model does not
/// place in a block, or a caret that is not on a run at all — three states that
/// are one answer here (*"there is no paragraph to reflow"*) and are told apart
/// by the caller only insofar as it says so.
#[must_use]
pub fn block_of_run(doc: &OpenDoc, page_index: usize, run: usize) -> Option<usize> {
    use crate::app::settings::SettingsExt;
    let page_ref = doc.pages.get(page_index)?;
    // ★ `with_provenance(true)`, which `reflow_block` requires by name — it
    // answers `ReflowApplyError::NoProvenance` without it. The extraction
    // options are otherwise the operator's own, so the runs this addresses are
    // segmented exactly as the runs the canvas paints.
    let opts = doc.settings.extract_options().with_provenance(true);
    let text = pdfcer_core::text_extract::extract_page_view(
        &doc.session.view(),
        page_ref,
        page_index,
        &opts,
    )
    .ok()?;
    let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
    model.block_at(TextPosition::new(run, 0))
}

#[cfg(test)]
mod tests {
    /// ★★ **The recognition options are the caret's, not the reflow engine's.**
    ///
    /// A source assertion rather than a behavioural one, and it is the only
    /// instrument available: both option sets produce a valid model and a valid
    /// block index, so a build using the wrong one reflows **a different
    /// paragraph** than the operator clicked in — silently, correctly, and on
    /// the documents where the two recognitions disagree, which are exactly the
    /// multi-column and ragged-right ones this feature is for.
    ///
    /// ⇒ The engine publishes `reflow_recognition_options()` and it is the
    /// obvious thing to reach for here. It is wrong here and right two modules
    /// away, which is why the mistake is worth a test rather than a comment.
    #[test]
    fn the_block_is_found_with_the_carets_own_recognition() {
        let source = include_str!("reflow.rs");
        assert!(
            source.contains("BlockRecognitionOptions::default()"),
            "the block lookup no longer uses the caret's own recognition"
        );
        // ★ The negative targets the CALL, not the name. The name appears in
        // this module's own header, warning against exactly this — an assertion
        // on the bare string would fail on the documentation that prevents the
        // mistake, which is the shape where a test punishes its own fix.
        // ★★ The needle is BUILT rather than written, and it has to be: a
        // literal here appears in this very file, so the scan matched its own
        // assertion and failed against correct code. **A source scan cannot
        // contain its own needle** — the same trap `typing-guard-exempt:
        // SELF-REFERENTIAL` names one module along.
        let relaxed = format!("recognize(&text, &{}())", "reflow_recognition_options");
        assert!(
            !source.contains(&relaxed),
            "the block lookup uses the RELAXED recognition, which segments the page \
             differently from the model the caret was placed against"
        );
    }
}
