//! # `text::ocr` — every word the Recognise-text surface says
//!
//! Consumed by [`crate::dialogs::ocr`] (the dialog that runs recognition and
//! reports what it inferred) and by [`crate::find::bar`] (the offer that
//! appears when a search found nothing on a page that has no text to find).
//!
//! ## ★ Why this catalog is unusually careful, and it is not house style
//!
//! **OCR is the single largest inference pdfcer makes.** `pdfcer-core`'s own
//! `ocr::layer` header says it in those words — *"every word here is a
//! guess"* — and project rule 4 (*"fuzzy, never sneaky"*) therefore binds this
//! surface harder than any other in the program. Two of its clauses bite here
//! and they pull in different directions, which is why the copy is written the
//! way it is:
//!
//! 1. **The result must look normal.** The operator asked for exactly that:
//!    *"I want OCRed stuff to look normal when the command is executed too."*
//!    Mode 3 is not a compromise, it is the whole mechanism — nothing visible
//!    is added, the page renders pixel-identically, and there is **no**
//!    highlighting of doubtful words baked into the document. So none of the
//!    copy below promises a visible mark, and none of it should ever grow one.
//! 2. **The uncertainty must be stated anyway**, off-canvas, before the
//!    recognition becomes a file. That is what this dialog is for.
//!
//! ## ★ The one fact this surface exists to carry
//!
//! **`ocrs` reports no confidence at all.** Not "low confidence", not
//! "confidence pending" — its output type is a character and a rectangle, and
//! there is no score on a character, a word, a line or the page.
//! `OcrsEngine::reports_confidence()` returns `false` and
//! `pdfcer_core::ocr::engine_ocrs`'s header is explicit that this is *"a fact
//! about the world"* rather than a stub awaiting improvement.
//!
//! The consequence for copy is sharp, and it is the reason [`no_confidence`]
//! is worded as a negation of a specific wrong reading rather than as a
//! neutral note: **an absent score and a high score must never look the
//! same.** A dialog that reported "0 words need review" would be true of an
//! engine that scores nothing and would read as a clean bill of health. So
//! this surface never says that, and `OcrPage::words_needing_review` already
//! encodes the same principle on the other side by counting an unscored word
//! as needing review.
//!
//! ## Where the *engine's* sentences come from, and why they are not here
//!
//! [`crate::dialogs::ocr`] renders `OcrLayerReport::disclosures()` — a
//! `Vec<String>` built inside `pdfcer-core` — as a list, verbatim, beneath the
//! headings below. That is deliberate and it is the engine's own instruction:
//! the disclosures are built *"here rather than at each call site so the GUI
//! and the CLI cannot disagree about what was disclosed."*
//!
//! They are therefore **data at run time**, not literals in this crate, and
//! `tools/gates/check-ui-strings.sh` is untouched by them. What lives here is
//! the shell's own framing — the headings, the buttons, the refusals — which
//! is exactly the split rule R1 is about: a catalog owns the words this
//! program chose, not the words another crate reported.
//!
//! ## Conventions
//!
//! [`crate::text`]'s, unchanged: sentence case and no trailing period on a
//! label, full sentences with punctuation for prose, an ellipsis on a control
//! that asks a question before acting. One addition — **no sentence here
//! makes a claim about accuracy.** pdfcer has never measured this engine
//! against a real scan (`FEATURES.md` records that its only test documents are
//! vector PDFs that already contain text), so "accurate", "reliable" and
//! "high quality" are words this surface is not entitled to.

// ---------------------------------------------------------------------------
// The dialog
// ---------------------------------------------------------------------------

/// ★★★ **The run ended where the operator asked it to** — said on the outcome
/// screen, above the reassurance that the words are in the document.
///
/// A stopped run is a success and an incomplete one at the same time, and this
/// is the sentence that stops the first half hiding the second. Somebody who
/// ends a 200-page recognition at page 40 must not walk away believing the
/// document is done; they find out otherwise months later, searching for a word
/// on page 150 that is not in the layer.
///
/// ★ It names both numbers. "Stopped early" alone leaves them to guess how much
/// they have, and the answer is the whole point of having pressed Stop rather
/// than Cancel.
#[must_use]
pub fn stopped_early(attempted: usize, of: usize) -> String {
    format!(
        "You stopped after {attempted} of {of} pages. The rest of the document has not been recognised; what was done is kept."
    )
}

/// **Everything was thrown away**, which is what Cancel means.
///
/// ★ It says the document is untouched, because that is the fact the operator
/// is actually checking for — a half-written layer is the thing they pressed
/// Cancel to avoid, and silence about it leaves them to wonder.
#[must_use]
pub fn cancelled(attempted: usize) -> String {
    if attempted == 0 {
        "Cancelled. Nothing was recognised and your document is unchanged.".to_owned()
    } else {
        format!(
            "Cancelled after {attempted} page(s). That work was thrown away and your document is unchanged."
        )
    }
}

/// **What the recogniser is doing right now.**
///
/// Operator request, 2026-09-01: *"so that the user can see that it is doing
/// something and hasn't frozen on large documents."*
///
/// ★★ Three moving numbers, and each answers a different worry. The page count
/// answers *"how far"*; the character count answers *"is it still alive"* —
/// it moves on a dense sheet where the word count barely does; and naming the
/// page it is ON rather than only the count tells an operator whose scan is bad
/// exactly which sheet to look at afterwards.
#[must_use]
pub fn working_progress(attempted: usize, of: usize, words: usize, chars: usize) -> String {
    format!("Page {attempted} of {of} — {words} words, {chars} characters so far")
}

/// The control that finishes the page in hand and keeps everything.
#[must_use]
pub fn stop_button() -> String {
    "Stop".to_owned()
}

/// What Stop does, said in full because it is the half of the pair that keeps
/// work and the operator must not have to guess which is which.
#[must_use]
pub fn stop_tooltip() -> String {
    "Finishes the page it is working on, then stops. Everything recognised so far is kept."
        .to_owned()
}

/// The control that abandons the run.
#[must_use]
pub fn cancel_button() -> String {
    "Cancel".to_owned()
}

/// What Cancel does. The word "thrown away" is deliberate and is not softened:
/// it is the difference between the two buttons, and a euphemism here would put
/// the operator one click from losing a long run.
#[must_use]
pub fn cancel_tooltip() -> String {
    "Stops straight away and throws away everything recognised so far. Your document is left unchanged."
        .to_owned()
}

/// The dialog's title.
#[must_use]
pub fn title() -> &'static str {
    "Recognise text"
}

/// The sentence at the top of the dialog, before anything has been run.
///
/// Says what the operation *does to the page*, because that is the first
/// question an operator has about a tool that rewrites a document they may
/// have to defend the provenance of. The answer — nothing visible changes, the
/// image is not re-encoded — is `ocr::layer`'s own guarantee and is worth
/// leading with rather than burying under a progress bar.
#[must_use]
pub fn intro() -> &'static str {
    "Reads the words in the page image and adds them as invisible text behind it, so Find and copy work. The page still looks exactly the same, and the scan itself is never re-encoded."
}

/// The label on the control that starts recognition.
///
/// ★ **No longer "Recognise this page".** It said that because that was all it
/// could do, and the operator's 2026-08-26 report — *"how do I OCR more than
/// one page? Why does the tool stop at one?"* — was as much about the label as
/// about the capability: a button naming one page is a button that has already
/// answered the question, wrongly.
#[must_use]
pub fn run() -> &'static str {
    "Recognise"
}

// ---------------------------------------------------------------------------
// Page scope
//
// The group of choices that answers "which pages". Every surveyed recogniser
// has one; this program had none, and the absence was the operator's loudest
// complaint about the whole dialog.
// ---------------------------------------------------------------------------

/// The heading above the scope choices.
#[must_use]
pub fn scope_heading() -> &'static str {
    "Pages"
}

/// Every page of the document — the default.
///
/// ★ First in the list **and** pre-selected, which are two decisions and both
/// deliberate. First because the surveyed tools put it first; pre-selected
/// because recognising a scan means recognising the scan, not one sheet of it.
/// The old behaviour is the second option and one click away.
#[must_use]
pub fn scope_all() -> &'static str {
    "All pages"
}

/// Only the page the dialog opened on. `page` is one-based, for display.
///
/// It names the number rather than saying *"the current page"* because the
/// operator can page the document while this window is up, and by the time
/// they read the label "current" may no longer mean what the run will do. The
/// number cannot drift.
#[must_use]
pub fn scope_current(page: usize) -> String {
    format!("This page only (page {page})")
}

/// ★★★ **The pages picked in the thumbnail rail** —
/// `OPERATOR_REQUESTS.md` O79.
///
/// The operator: *"I should have options to do the whole document, or the
/// pages I have selected in the thumbnails."*
///
/// `count` is how many are picked, so the label states the operand rather than
/// naming a place the operator then has to go and count. *"Selected pages"*
/// alone would be a promise whose size is invisible from the dialog — and this
/// is a run that can take minutes, so the number is the part that decides
/// whether he presses the button.
///
/// # ★ Why it is drawn only when something is picked
///
/// R9. With an empty rail selection this option has no operand at all, and a
/// greyed radio saying *"Selected pages (0)"* would be a control explaining
/// its own uselessness in a window that already has three working answers. The
/// remedy is not on this surface — it is *go and pick some pages* — so there
/// is nothing a hover could usefully say either.
///
/// The plural is written out for the same reason every count in this crate is:
/// *"1 pages"* costs credibility on a surface whose whole job is being
/// believed.
#[must_use]
pub fn scope_picked(count: usize) -> String {
    if count == 1 {
        "The 1 page picked in the thumbnails".to_owned()
    } else {
        format!("The {count} pages picked in the thumbnails")
    }
}

/// The pages the operator types.
#[must_use]
pub fn scope_range() -> &'static str {
    "Pages"
}

/// The hint beside the range field.
///
/// Shows the syntax by example rather than describing it, because the syntax
/// is `dialogs::print::tabs::parse_page_range`'s and an example is both shorter
/// and harder to get subtly wrong than a description of it.
#[must_use]
pub fn scope_range_hint() -> &'static str {
    "e.g. 1-4, 7, 9-12"
}

/// Said under the range field when what was typed names no page.
///
/// ★ Not an error — a **status**. A half-typed `1-` is an ordinary state of a
/// text field the operator is in the middle of using, and colouring it red or
/// popping a message would be scolding them for typing. The Recognise button is
/// simply not available until the range resolves, and this says why.
#[must_use]
pub fn scope_range_unresolved() -> &'static str {
    "Type page numbers to recognise, like 1-4 or 2, 5, 9."
}

/// The label on the skip-existing-text toggle.
#[must_use]
pub fn skip_pages_with_text() -> &'static str {
    "Skip pages that already have text"
}

/// Its tooltip — the measured reason it is on by default.
///
/// ★★ This is not a preference, it is a **hazard guard**, and the measurement
/// is worth keeping in the tooltip rather than only in a comment: running the
/// recogniser twice over one page was measured on 2026-08-26 to take a page
/// from 427 character codes to 854. Nothing looks different, because the layer
/// is invisible both times — but every Find hit is doubled and every copy comes
/// out twice.
#[must_use]
pub fn skip_pages_with_text_tooltip() -> &'static str {
    "Recognising a page that already has text adds a second invisible copy of it, so Find matches and copied text come out doubled. Turn this off only if you know a page's existing text is wrong."
}

/// Its tooltip.
///
/// Names the cost in the operator's terms. There is no measured figure to
/// quote — see the module header on what this surface is not entitled to
/// claim — so it says *seconds* and says which page, which are both true and
/// checkable.
#[must_use]
pub fn run_tooltip() -> &'static str {
    "Runs the recogniser over the pages you chose. It takes a few seconds per page, and the window will not respond while it does."
}

/// Shown while the recogniser is working.
#[must_use]
pub fn working() -> &'static str {
    "Recognising…"
}

/// The heading above the engine's own disclosure lines.
#[must_use]
pub fn what_was_inferred() -> &'static str {
    "What was recognised, and what that is worth"
}

/// ★ **The confidence sentence, and the most load-bearing string here.**
///
/// Worded to refuse a specific wrong reading rather than to state a neutral
/// fact, because the wrong reading is the one a reader arrives with: a page of
/// recognised text with no warnings on it looks checked. It is not checked. It
/// was never scored either way.
///
/// The engine emits its own version of this through
/// `OcrLayerReport::disclosures()`, and the two are deliberately both present:
/// that one appears in the list of disclosures beside the counts, this one is
/// the dialog's own heading-level statement, and the operator reads the second
/// before they read the list. Duplication is the point — this is the one fact
/// that must not be missed by someone who skims.
#[must_use]
pub fn no_confidence() -> &'static str {
    "This recogniser reports no confidence score for any word, so nothing here has been checked — that is not the same as everything being right. Read the text before you rely on it."
}

/// ★★★ **The sentence that replaced the whole save apparatus.**
///
/// It says three things in one line, and each was a separate control before:
/// the words are *in the document*, an ordinary Save writes them, and an
/// ordinary Undo removes them.
///
/// The operator, 2026-08-26: *"Why do I have to save a copy instead of just go
/// back into my pdf and save over it or save from there?"* The answer was that
/// `add_ocr_layer` took an immutable document and handed back a whole file, so
/// this shell had nothing to put the layer *into*. The engine's Pass 135.0
/// (2026-08-27) made recognition an edit, and the honest sentence is now the
/// short one.
#[must_use]
pub fn applied_to_document() -> &'static str {
    "The text is now in this document. Save when you are ready, or press Ctrl+Z to take it back out."
}

/// The title on the system file-save dialog.
#[must_use]
pub fn save_dialog_title() -> &'static str {
    "Save recognised copy"
}

/// The suffix appended to the original file's stem to suggest a name.
///
/// A suggestion, not a rule — the operator can type anything. It exists so the
/// default answer is never the file they opened, which is the same protection
/// the label spells out in words.
#[must_use]
pub fn suggested_suffix() -> &'static str {
    "-recognised"
}

/// The button that closes the dialog.
#[must_use]
pub fn close() -> &'static str {
    "Close"
}

// ---------------------------------------------------------------------------
// Named refusals
//
// Every one of these is a specific, actionable cause. The engine's own error
// type refuses by name for the same reason, and folding them into one
// "OCR failed" would throw away the half of the message the operator can act
// on.
// ---------------------------------------------------------------------------

/// The models are not where this build looks for them.
///
/// `searched` is the engine's own list of every directory it tried, in order.
/// It is part of the message rather than a detail: *"models not found"* is
/// unactionable, and the list is what tells an operator either where to put
/// the files or — just as often — that they put them somewhere pdfcer never
/// looks.
///
/// ★ It takes a **list**, not a pre-joined string, and the separator below is
/// why: a comma and a space between two paths is punctuation an operator reads,
/// so it is copy and belongs in this file rather than at the call site.
/// `tools/gates/check-ui-strings.sh` caught exactly that `", "` sitting in
/// `dialogs::ocr::sentence`, and it was right to.
#[must_use]
pub fn models_missing(searched: &[String]) -> String {
    let list = searched.join(", ");
    format!(
        "The recognition models are not installed. They ship in the models\\ocrs folder beside \
         pdfcer-gui.exe; this build looked in: {list}"
    )
}

/// This build was compiled without the recogniser.
///
/// A named refusal rather than a greyed control, and distinct from
/// [`models_missing`] on purpose: *"cannot look for text"* and *"could not
/// find the files to look with"* call for completely different actions, and
/// the engine's own feature block insists the two never collapse into one
/// answer.
#[must_use]
pub fn engine_absent() -> &'static str {
    "This build was made without the text recogniser, so it cannot read words from an image. A standard pdfcer build can."
}

/// Recognition ran and found no word it could place.
///
/// Distinct from a failure: the engine worked, the page simply had nothing on
/// it a recogniser could read. Blank paper and a photograph of a wall both
/// land here, and so does a page whose ink is too faint.
#[must_use]
pub fn nothing_recognised() -> &'static str {
    "No text was recognised on this page. There may be nothing readable on it, or the image may be too small or too faint."
}

/// Every page in the run already had text, so nothing was recognised.
///
/// ★ Distinct from [`nothing_recognised`], which reports that the recogniser
/// looked and found nothing. This reports that it **declined to look**, which
/// is a different fact with a different remedy — one is "there is nothing
/// readable here", the other is "there is already text here and I did not want
/// to double it". Collapsing them would leave the operator with no way to tell
/// a blank scan from a document that was already recognised last week.
#[must_use]
pub fn already_has_text() -> &'static str {
    "Every page selected already has text, so none were recognised. Turn off \u{201c}Skip pages that already have text\u{201d} to recognise them anyway."
}

/// What a multi-page run did, in pages.
///
/// Only shown when the run covered more than one page — a one-page run reports
/// its words and nothing else, because *"1 page recognised"* is a sentence that
/// tells the operator only what they already did.
#[must_use]
pub fn pages_outcome(written: usize, skipped: usize) -> String {
    let pages = if written == 1 { "page" } else { "pages" };
    if skipped == 0 {
        format!("{written} {pages} recognised.")
    } else {
        let skipped_pages = if skipped == 1 { "page" } else { "pages" };
        format!(
            "{written} {pages} recognised; {skipped} {skipped_pages} skipped because they already had text."
        )
    }
}

/// The recogniser or the layer writer refused, carrying the engine's reason.
///
/// The engine's own sentence is appended rather than replaced. `pdfcer-core`'s
/// error types name specific causes — an encrypted document, a page index past
/// the end, a model file the runtime rejected — and paraphrasing them here
/// would produce a second, vaguer account of a diagnosis that was already
/// precise.
#[must_use]
pub fn failed(reason: &str) -> String {
    format!("Recognition did not finish: {reason}")
}

// ---------------------------------------------------------------------------
// The Find offer
// ---------------------------------------------------------------------------

/// ★ **The sentence the Find bar shows when the page has no text at all.**
///
/// It reports the *page*, not the search. That distinction is the whole rule
/// and the operator stated it: the trigger is *"this document is images"*, and
/// it is **not** *"this search had no matches"*. A search for a word that
/// simply is not in a text PDF is an ordinary empty result, and offering to
/// recognise it would be nonsense — so this sentence says what was actually
/// established, which is that there is no text on this page for any search to
/// have found.
#[must_use]
pub fn offer() -> &'static str {
    "This page has no text on it — only an image."
}

/// The control beside it.
///
/// Ellipsis, because it opens the dialog rather than recognising on the spot.
/// A search bar is the wrong place to start several seconds of work from a
/// single click.
#[must_use]
pub fn offer_action() -> &'static str {
    "Recognise text…"
}

/// Its tooltip.
#[must_use]
pub fn offer_tooltip() -> &'static str {
    "Opens Recognise text, which reads the words in the page image and adds them as invisible text so Find can see them."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **This surface never claims accuracy.**
    ///
    /// The module header's rule, asserted rather than trusted. pdfcer has never
    /// run this engine against a real scan, so any adjective implying measured
    /// quality would be a claim with nothing behind it — and marketing
    /// adjectives are exactly what a copy pass adds without thinking.
    #[test]
    fn nothing_here_claims_the_recognition_is_accurate() {
        let forbidden = [
            "accurate",
            "accuracy",
            "reliable",
            "reliably",
            "high quality",
            "high-quality",
            "precise",
            "correctly",
            "perfect",
        ];
        let prose: Vec<String> = vec![
            title().to_owned(),
            intro().to_owned(),
            run().to_owned(),
            run_tooltip().to_owned(),
            working().to_owned(),
            what_was_inferred().to_owned(),
            no_confidence().to_owned(),
            applied_to_document().to_owned(),
            scope_heading().to_owned(),
            scope_all().to_owned(),
            scope_current(1),
            scope_range().to_owned(),
            scope_range_hint().to_owned(),
            scope_range_unresolved().to_owned(),
            skip_pages_with_text().to_owned(),
            skip_pages_with_text_tooltip().to_owned(),
            engine_absent().to_owned(),
            already_has_text().to_owned(),
            nothing_recognised().to_owned(),
            offer().to_owned(),
            offer_action().to_owned(),
            offer_tooltip().to_owned(),
        ];
        for line in &prose {
            let lower = line.to_lowercase();
            for word in forbidden {
                assert!(
                    !lower.contains(word),
                    "`{line}` claims {word:?}; this surface has no measurement behind such a claim"
                );
            }
        }
    }

    /// ★ **The confidence sentence says the absence is not a clean bill.**
    ///
    /// The one string on this surface that must not be softened. It is here as
    /// a test rather than only as a doc comment because "no confidence
    /// reported" reads as neutral, and a future copy pass tidying it into
    /// something neutral would delete the disclosure while leaving a sentence
    /// in its place.
    #[test]
    fn the_confidence_sentence_refuses_the_wrong_reading() {
        let text = no_confidence().to_lowercase();
        assert!(
            text.contains("no confidence"),
            "it must state the absence outright: {text}"
        );
        assert!(
            text.contains("not the same"),
            "…and must say that the absence is not the same as everything being right, which is \
             the reading it exists to refuse: {text}"
        );
    }

    /// ★★★ **The outcome sentence names the document, the save and the undo.**
    ///
    /// This replaced a test called
    /// `the_write_control_offers_a_new_file_and_never_an_overwrite`, which
    /// asserted that the only way out of this dialog was a Save-as. That was
    /// true, it was enforced, and it was the thing the operator objected to:
    /// *"Why do I have to save a copy instead of just go back into my pdf and
    /// save over it?"*
    ///
    /// It was never a policy. `ocr::layer::add_ocr_layer` took an immutable
    /// document and returned a whole file, so a Save-as was the only shape
    /// available. The engine's Pass 135.0 made recognition an edit, and the
    /// three facts below are what the operator now needs to be told instead.
    #[test]
    fn the_outcome_says_where_the_text_went_and_how_to_undo_it() {
        let text = applied_to_document().to_lowercase();
        assert!(
            text.contains("in this document"),
            "it must say the words are in the OPEN document, not in a file somewhere: {text}"
        );
        assert!(
            text.contains("save"),
            "…that an ordinary save writes them: {text}"
        );
        assert!(
            text.contains("ctrl+z") || text.contains("undo"),
            "…and that they can be taken back out: {text}"
        );
    }

    /// ★★ **The two "nothing happened" sentences are not interchangeable.**
    ///
    /// `nothing_recognised` means the recogniser looked and found nothing;
    /// `already_has_text` means it declined to look. Different facts, different
    /// remedies — one is "there is nothing readable here", the other is "there
    /// is already text here and doubling it would break Find". Collapsing them
    /// would leave the operator unable to tell a blank scan from a document
    /// that was recognised last week.
    #[test]
    fn a_skipped_page_and_an_unreadable_one_say_different_things() {
        assert_ne!(already_has_text(), nothing_recognised());
        assert!(
            already_has_text().to_lowercase().contains("already"),
            "the skip sentence must name the reason it skipped"
        );
    }

    /// ★ **The Find offer talks about the page, not about the search.**
    ///
    /// The trap the operator named, pinned. A sentence mentioning matches
    /// would be the collapse of *"the document is images"* into *"this search
    /// found nothing"* — the two the specification insists must not be one.
    #[test]
    fn the_find_offer_reports_the_page_rather_than_the_search() {
        let text = offer().to_lowercase();
        assert!(
            text.contains("no text") && text.contains("page"),
            "the offer must state what is true of the page: {text}"
        );
        for word in ["match", "search", "result", "found"] {
            assert!(
                !text.contains(word),
                "the offer must not mention {word:?} — the trigger is that the page is an image, \
                 not that a search came back empty: {text}"
            );
        }
    }

    /// Two different absences produce two different sentences.
    #[test]
    fn a_missing_engine_and_missing_models_are_not_the_same_message() {
        assert_ne!(engine_absent(), models_missing(&["x".to_owned()]));
        assert!(
            models_missing(&["C:\\a".to_owned(), "C:\\b".to_owned()]).contains("C:\\a, C:\\b"),
            "the searched paths are the actionable half and must survive into the message — \
             and so must the separator between them, which is why this function joins the \
             list rather than taking one already joined"
        );
    }
}
