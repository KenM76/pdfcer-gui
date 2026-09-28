//! # `text::ocr` — every word the Recognise-text surface says
//!
//! Consumed by `pdfcer_gui::dialogs::ocr` (the dialog that runs recognition and
//! reports what it inferred) and by `find::bar` (the offer that
//! appears when a search found nothing on a page that has no text to find).
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/ocr.md`.

// ---------------------------------------------------------------------------
// The dialog
// ---------------------------------------------------------------------------

/// **The run ended where the operator asked it to** — said on the outcome
/// screen, above the reassurance that the words are in the document.
#[must_use]
pub fn stopped_early(attempted: usize, of: usize) -> String {
    format!(
        "You stopped after {attempted} of {of} pages. The rest of the document has not been recognised; what was done is kept."
    )
}

/// **Everything was thrown away**, which is what Cancel means.
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
#[must_use]
pub fn intro() -> &'static str {
    "Reads the words in the page image and adds them as invisible text behind it, so Find and copy work. The page still looks exactly the same, and the scan itself is never re-encoded."
}

/// The label on the control that starts recognition.
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
#[must_use]
pub fn scope_all() -> &'static str {
    "All pages"
}

/// Only the page the dialog opened on. `page` is one-based, for display.
#[must_use]
pub fn scope_current(page: usize) -> String {
    format!("This page only (page {page})")
}

/// **The pages picked in the thumbnail rail** —
/// `OPERATOR_REQUESTS.md` O79.
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
#[must_use]
pub fn scope_range_hint() -> &'static str {
    "e.g. 1-4, 7, 9-12"
}

/// Said under the range field when what was typed names no page.
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
#[must_use]
pub fn skip_pages_with_text_tooltip() -> &'static str {
    "Recognising a page whose text came from another program adds a second invisible copy of it, so Find matches and copied text come out doubled. Text pdfcer recognised earlier is replaced instead. Turn this off only if you know a page's existing text is wrong, or to redo pdfcer's own recognition."
}

/// Its tooltip.
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

/// **The confidence sentence, and the most load-bearing string here.**
#[must_use]
pub fn no_confidence() -> &'static str {
    "This recogniser reports no confidence score for any word, so nothing here has been checked — that is not the same as everything being right. Read the text before you rely on it."
}

/// The confidence sentence for a recogniser that scores every word.
///
/// Says what the score is — the recogniser's estimate of its own reading —
/// and that it is not a check, so a high score is not read as a verified word.
#[must_use]
pub fn scored_confidence() -> &'static str {
    "Every word carries the recogniser's own confidence score. That score is its estimate of its own reading, not a check — read the text before you rely on it."
}

/// The heading over the recogniser choice. Drawn only when a build offers
/// more than one.
#[must_use]
pub fn engine_heading() -> &'static str {
    "Recogniser"
}

/// A recogniser's name as the choice shows it.
#[must_use]
pub const fn engine_label(engine: crate::ocr::EngineId) -> &'static str {
    match engine {
        crate::ocr::EngineId::Ocrs => "ocrs",
        crate::ocr::EngineId::Ocrcer => "OCRcer",
        crate::ocr::EngineId::Paddle => "PaddleOCR",
    }
}

/// What choosing a recogniser changes, on hover.
#[must_use]
pub const fn engine_tooltip(engine: crate::ocr::EngineId) -> &'static str {
    match engine {
        crate::ocr::EngineId::Ocrs => {
            "The ocrs recogniser: two neural networks. It gives no confidence score."
        }
        crate::ocr::EngineId::Ocrcer => {
            "The OCRcer recogniser: matches each character against its model and gives every word a confidence score."
        }
        crate::ocr::EngineId::Paddle => {
            "The PaddleOCR recogniser: runs PP-OCR models you supply in the models\\paddle folder, and gives every word a confidence score."
        }
    }
}

/// **The sentence that replaced the whole save apparatus.**
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
#[must_use]
pub fn models_missing(shipped: bool, searched: &[String]) -> String {
    let list = searched.join(", ");
    if shipped {
        format!(
            "The recognition models are not installed. They ship in the models folder beside \
             pdfcer-gui.exe; this build looked in: {list}"
        )
    } else {
        format!(
            "No PaddleOCR models were found. pdfcer does not include them: place PP-OCR ONNX \
             exports named det.onnx and rec.onnx (and optionally dict.txt) in one of: {list}"
        )
    }
}

/// The run read characters through a dictionary file.
#[must_use]
pub fn dictionary_file(path: &str) -> String {
    format!(
        "Characters were read through the dictionary in {path}. If it does not match the \
         recognition model, the text will be confident nonsense."
    )
}

/// The run read characters through the model's own character list.
#[must_use]
pub fn dictionary_embedded() -> &'static str {
    "Characters were read through the character list built into the recognition model."
}

/// This build was compiled without the recogniser.
#[must_use]
pub fn engine_absent() -> &'static str {
    "This build was made without the text recogniser, so it cannot read words from an image. A standard pdfcer build can."
}

/// Recognition ran and found no word it could place.
#[must_use]
pub fn nothing_recognised() -> &'static str {
    "No text was recognised on this page. There may be nothing readable on it, or the image may be too small or too faint."
}

/// Every page in the run already had text, so nothing was recognised.
#[must_use]
pub fn already_has_text() -> &'static str {
    "Every page selected already has text, so none were recognised. Turn off \u{201c}Skip pages that already have text\u{201d} to recognise them anyway."
}

/// What a multi-page run did, in pages.
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
#[must_use]
pub fn failed(reason: &str) -> String {
    format!("Recognition did not finish: {reason}")
}

// ---------------------------------------------------------------------------
// The Find offer
// ---------------------------------------------------------------------------

/// **The sentence the Find bar shows when the page has no text at all.**
#[must_use]
pub fn offer() -> &'static str {
    "This page has no text on it — only an image."
}

/// The control beside it.
#[must_use]
pub fn offer_action() -> &'static str {
    "Recognise text…"
}

/// Its tooltip.
#[must_use]
pub fn offer_tooltip() -> &'static str {
    "Opens Recognise text, which reads the words in the page image and adds them as invisible text so Find can see them."
}

/// The View ▸ Display blend control, which fades between the scan and the
/// recognised text drawn over it.
#[must_use]
pub fn layer_blend_label() -> &'static str {
    "Text layer"
}

/// Its tooltip.
#[must_use]
pub fn layer_blend_tooltip() -> &'static str {
    "Fades between the scanned page and the recognised text drawn over it. All the way left is the page alone; all the way right is the recognised text alone, on blank paper."
}

/// The unit on the blend control's number.
#[must_use]
pub fn layer_blend_suffix() -> &'static str {
    "%"
}

// ---------------------------------------------------------------------------
// Earlier OCR text: replaced by a re-run, or removed on request
// ---------------------------------------------------------------------------

/// Added to the edit's disclosures when a recognition replaced text an earlier
/// run wrote. `layers` is the total across the run; `pages` how many pages had one.
#[must_use]
pub fn layers_replaced(layers: usize, pages: usize) -> String {
    format!(
        "The earlier recognised text was replaced, not added to: {layers} old OCR text layer(s) on {pages} page(s) came off, so each word is in the document once."
    )
}

/// The disclosure after File ▸ Remove OCR text succeeded.
#[must_use]
pub fn layers_removed(layers: usize, pages: usize) -> String {
    format!(
        "Removed {layers} OCR text layer(s) from {pages} page(s). The pages look the same; Find and copy no longer see that text. Press Ctrl+Z to put it back."
    )
}

/// The disclosure when some layers came off and a later one was refused.
#[must_use]
pub fn layers_removed_partly(removed: usize, of: usize) -> String {
    format!(
        "Removed {removed} of {of} OCR text layers; pdfcer could not remove the rest and left them as they were. Press Ctrl+Z to put back what was removed."
    )
}

/// Why an OCR-layer write or removal did nothing — the status-bar decline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcrLayerRefusal {
    /// `OcrLayerError::LayerPresent`: the page already has pdfcer OCR text and
    /// the run was told not to replace it.
    AlreadyPresent,
    /// `OcrLayerError::LayerNotFound`: the layer changed under the command.
    LayerGone,
    /// Remove OCR text found no layer pdfcer wrote.
    NoneFound,
}

impl OcrLayerRefusal {
    /// The sentence.
    #[must_use]
    pub const fn line(self) -> &'static str {
        match self {
            Self::AlreadyPresent => {
                "These pages already carry text pdfcer recognised earlier, so nothing was added. Remove the OCR text first, or run recognition again to replace it."
            }
            Self::LayerGone => {
                "That OCR text was no longer where pdfcer found it, so nothing was removed. Run Remove OCR text again."
            }
            // States the limit: text another program recognised is not marked
            // and is left alone.
            Self::NoneFound => {
                "This document has no OCR text written by pdfcer, so nothing was removed. Text recognised by other programs is left alone."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **This surface never claims accuracy.**
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
            scored_confidence().to_owned(),
            engine_heading().to_owned(),
            engine_tooltip(crate::ocr::EngineId::Ocrs).to_owned(),
            engine_tooltip(crate::ocr::EngineId::Ocrcer).to_owned(),
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
            layers_replaced(2, 1),
            layers_removed(2, 1),
            layers_removed_partly(1, 2),
            OcrLayerRefusal::AlreadyPresent.line().to_owned(),
            OcrLayerRefusal::LayerGone.line().to_owned(),
            OcrLayerRefusal::NoneFound.line().to_owned(),
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

    /// **The confidence sentence says the absence is not a clean bill.**
    #[test]
    fn the_scored_sentence_says_the_score_is_not_a_check() {
        let text = scored_confidence().to_lowercase();
        assert!(text.contains("not a check"), "{text}");
        assert!(text.contains("before you rely"), "{text}");
    }

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

    /// **The outcome sentence names the document, the save and the undo.**
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

    /// **The two "nothing happened" sentences are not interchangeable.**
    #[test]
    fn a_skipped_page_and_an_unreadable_one_say_different_things() {
        assert_ne!(already_has_text(), nothing_recognised());
        assert!(
            already_has_text().to_lowercase().contains("already"),
            "the skip sentence must name the reason it skipped"
        );
    }

    /// **The Find offer talks about the page, not about the search.**
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
        assert_ne!(engine_absent(), models_missing(true, &["x".to_owned()]));
        assert!(
            models_missing(true, &["C:\\a".to_owned(), "C:\\b".to_owned()])
                .contains("C:\\a, C:\\b"),
            "the searched paths are the actionable half and must survive into the message — \
             and so must the separator between them, which is why this function joins the \
             list rather than taking one already joined"
        );
    }
}
