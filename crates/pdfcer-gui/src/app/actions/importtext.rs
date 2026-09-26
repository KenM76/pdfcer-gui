//! # `app::actions::importtext` — a text file becomes pages, and the receipt
//! says what the import decided
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/importtext.md`.

use pdfcer_core::text_edit::{PlaceTextError, PlaceTextReport};

use crate::app::state::OpenDoc;
use crate::text::importtext as t;

/// **What a File-tab command asks the document to do**, when the answer is an
/// edit rather than a write.
///
/// # Why this enum exists, and it is R2 arriving on time for once
///
/// `Action` already carries four sub-enums — `Annot`, `Vector`, `Field` and
/// the write family — and adds a fifth here. The pattern is the same one
/// `AnnotAction`'s own note argues: a family of related verbs shares one
/// dispatch arm, one apply arm and one module, so the shared files carry a line
/// each and the family's reasoning lives with the family.
///
/// **It has one member, and that is the honest shape rather than premature
/// structure.** Spelled directly on `Action` instead, one feature's argument
/// would be written **three times in three files nobody owns** — `action.rs`,
/// `apply.rs` and `dispatch.rs`, each already at R2's 1,500-line ceiling. The
/// line count is the symptom; the duplication is the defect, and R2's rule is
/// *"when a file approaches the limit, that is the signal to find the seam"*.
/// This is the seam.
#[derive(Debug, Clone, PartialEq)]
pub enum FileAction {
    /// **Turn a plain text file into new pages** — `file.import_text`.
    ///
    /// Raised by [`crate::dialogs::import_text`]'s Import button and by nothing
    /// else. This module's header carries why the file is read here rather than
    /// in the window.
    ImportText {
        /// The text file to read. UTF-8 is assumed and a failure to decode is
        /// reported by [`import`], not guessed at in the dialog.
        path: std::path::PathBuf,
        /// The sheet, margins, face and size the operator chose.
        ///
        /// **Boxed**, and clippy is not the reason: `PageTemplate` carries a
        /// rect, four margins, a face, a size, an optional leading, an
        /// alignment, a colour and a policy — the largest thing any `Action`
        /// variant holds — and every action in the queue is as large as the
        /// largest. Boxing keeps a queue of a hundred selection changes from
        /// carrying a page template each.
        template: Box<pdfcer_core::text_edit::PageTemplate>,
        /// Where the new pages land, in the engine's own vocabulary — the same
        /// field [`crate::app::actions::pages::PageAction::InsertPagesFromFile`]
        /// carries, verbatim and for the same reason.
        position: pdfcer_core::pageops::InsertPosition,
    },
}

/// **Route one File-tab action to its body.**
///
/// One arm today. It exists so that the second one is a line here rather than a
/// line in `apply.rs`, which is the whole point of the family split above.
pub(super) fn apply_action(doc: &mut OpenDoc, action: FileAction) {
    match action {
        FileAction::ImportText {
            path,
            template,
            position,
        } => import(doc, &path, &template, position),
    }
}

/// **Read `path` and place it as new pages.**
fn import(
    doc: &mut OpenDoc,
    path: &std::path::Path,
    template: &pdfcer_core::text_edit::PageTemplate,
    position: pdfcer_core::pageops::InsertPosition,
) {
    // Read as BYTES and decoded explicitly, rather than `read_to_string`.
    //
    // The two failures are different facts and the operator needs to be told
    // which: *"I could not open that file"* names a path, a permission or a
    // missing disk, and *"that file is not text I can read"* names an encoding
    // — a Windows-1252 register saved out of an old system is the realistic
    // case, and it is fixable by re-saving as UTF-8. `read_to_string` folds
    // both into one `io::Error` whose `InvalidData` kind an operator would meet
    // as *"the file is invalid"*.
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(why) => {
            super::record_note(doc.edit_epoch, t::unreadable_file(&why.to_string()));
            return;
        }
    };
    let text = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(_) => {
            super::record_note(doc.edit_epoch, t::not_utf8());
            return;
        }
    };

    // The page count BEFORE, so the receipt can say where the new sheets
    // landed in terms of the document the operator is looking at. Read here
    // rather than after, because after is the whole point of the sentence.
    let before = doc.pages.len();

    // The refusal is caught in a cell rather than recorded directly, because
    // `record_note` needs `doc.edit_epoch` and `doc` is borrowed mutably by
    // `vector_edit` for the length of the closure. One `Option`, written inside
    // and read after — which is also what makes the ordering above provable
    // rather than a claim about when a side effect ran.
    let mut notes: Option<String> = None;

    super::apply::vector_edit(doc, "import-text", 0, 1, |session| {
        session
            .place_text(&text, template, position)
            .inspect_err(|error| {
                // `record_note`, not the `decline` channel, and the reason is
                // a property of the TYPE rather than a preference: `Declined` is
                // `Copy` and its `line()` returns `&'static str`, so it cannot
                // carry a character listing, a page count or an `io::Error`.
                // Every refusal this verb produces is specific to the operator's
                // own file, and a static sentence would have to drop exactly the
                // part he needs.
                //
                // It is recorded from INSIDE the closure, so it is in the slot
                // before `vector_edit`'s generic arm runs — the precedence
                // `decline::floor` states: a decline the verb can name always
                // beats one it cannot.
                notes = Some(refusal_for(error));
            })
            .map(|report| {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    //
                    // It carries `pages=`, `coalesced=` and `undo=`, which
                    // are the three a wrong build gets wrong: a build that
                    // placed nothing, a build that promised one undo it cannot
                    // deliver, and a build whose fold silently stopped working.
                    format!(
                        "import-text-applied pages={} coalesced={} undo={} split={} \
                         tabs={} breaks={} overlong={} overflow={}",
                        report.pages_created,
                        u8::from(report.coalesced),
                        report.undo_entries,
                        report.paragraphs_split_across_pages,
                        report.tabs_collapsed,
                        report.explicit_page_breaks,
                        report.overlong_words,
                        report.box_overflow_lines,
                    )
                });
                disclosures(&report, before)
            })
    });

    if let Some(why) = notes {
        super::record_note(doc.edit_epoch, why);
    }
}

/// **Everything the import decided, as sentences.**
fn disclosures(report: &PlaceTextReport, pages_before: usize) -> Vec<String> {
    let mut out = Vec::new();
    out.push(t::pages_created(report.pages_created, pages_before));

    // Only when the promise cannot be kept. A sentence saying *"this can be
    // undone in one press"* on every import would be the third-time-unread
    // disclosure this project keeps deleting.
    if !report.coalesced {
        out.push(t::many_undo_steps(report.undo_entries));
    }

    // The six judgements, each only when it happened.
    if report.paragraphs_split_across_pages > 0 {
        out.push(t::paragraphs_split(report.paragraphs_split_across_pages));
    }
    if report.tabs_collapsed > 0 {
        out.push(t::tabs_collapsed(report.tabs_collapsed));
    }
    if report.explicit_page_breaks > 0 {
        out.push(t::page_breaks(report.explicit_page_breaks));
    }
    if report.overlong_words > 0 {
        out.push(t::overlong_words(report.overlong_words));
    }
    if report.chars_dropped_control > 0 {
        out.push(t::control_chars(report.chars_dropped_control));
    }
    if report.chars_dropped_unmappable > 0 {
        out.push(t::unmappable_dropped(report.chars_dropped_unmappable));
    }

    // LAST, and it is not one of the six. `box_overflow_lines` is the
    // engine's own self-check — its doc says it *must* be 0 — so a non-zero
    // here is a defect in the placer, not a judgement about the operator's
    // file. It is worded as one, because an operator who reads it needs to know
    // it is worth reporting rather than worth accepting.
    if report.box_overflow_lines > 0 {
        out.push(t::overflowed(report.box_overflow_lines));
    }
    out
}

/// **One refusal, as a sentence.**
fn refusal_for(error: &PlaceTextError) -> String {
    match error {
        // Both of these are the SAME operator problem seen from two sides —
        // the column has no width, or the column has no height — and both are
        // answerable in the window that is still open behind this message. So
        // both name the two controls rather than the two clauses of §9.
        PlaceTextError::NoColumn { .. } => t::no_column(),
        PlaceTextError::PageTooShort { .. } => t::page_too_short(),
        // The command is greyed on `doc.pages`, so reaching this means the
        // document lost its last page between the press and the drain — which
        // is rare, real, and worth saying plainly rather than generically.
        PlaceTextError::NoPageToInsertBeside => t::no_page_to_insert_beside(),
        // The refusal a real text file is most likely to meet — an em
        // dash, a curly quote, an accented name — and the one this window's
        // `face_note` warns about before the press. The engine's LISTING is
        // carried verbatim because the operator has to find those characters in
        // his own file and no rewording improves `U+2014 '—' ×12`.
        PlaceTextError::Unmappable {
            base_font,
            total,
            listing,
            ..
        } => t::unmappable_refused(base_font, *total, listing),
        other => t::refused(&other.to_string()),
    }
}

#[cfg(test)]
mod tests;
