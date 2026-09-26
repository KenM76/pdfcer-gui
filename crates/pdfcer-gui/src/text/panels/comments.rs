//! # `text::panels::comments` — every string the Comments panel shows
//!
//! The copy for [`crate::panels::comments`], which lists **every annotation
//! in the document** — the comment list a reviewer works through. One module
//! per panel surface, as [`super`]'s header lays out; `crate::panels::comments`
//! is the sole consumer.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/panels/comments.md`.

/// The count line, above the list.
#[must_use]
pub fn comments_count(total: usize) -> String {
    format!("{total} note(s) and markup item(s).")
}

/// Shown when the document carries no listable annotation.
#[must_use]
pub fn comments_none() -> &'static str {
    "No notes or markup on this document. Form fields and pop-up windows are not listed here — form fields have their own panel."
}

/// **What this panel filtered out, in numbers** — `None` when it filtered
/// nothing.
#[must_use]
pub fn comments_excluded(widgets: usize, popups: usize, trap_nets: usize) -> Option<String> {
    let mut clauses: Vec<String> = Vec::new();
    if widgets > 0 {
        clauses.push(format!(
            "{widgets} form field(s), which the Forms panel lists"
        ));
    }
    if popups > 0 {
        clauses.push(format!(
            "{popups} pop-up window(s), which belong to the annotations they hang off rather than being annotations in their own right"
        ));
    }
    if trap_nets > 0 {
        clauses.push(format!(
            "{trap_nets} trapping record(s), which are prepress output state written by a RIP rather than anything a person wrote"
        ));
    }
    if clauses.is_empty() {
        return None;
    }
    Some(format!("Not listed here: {}.", clauses.join("; ")))
}

/// Shown when EVERY listed annotation lacks `/Contents`.
#[must_use]
pub fn comments_all_without_notes() -> &'static str {
    "None of these carry note text. Shapes drawn in pdfcer do not have a note attached to them yet, so this is expected rather than missing data."
}

/// One row's heading — what it is and which page it is on.
#[must_use]
pub fn comment_row_heading(subtype: &str, page_number: usize) -> String {
    format!("{subtype} — p. {page_number}")
}

/// A **ce dimension**'s row heading, which names it as one.
#[must_use]
pub fn comment_row_ce_dimension_heading(subtype: &str, page_number: usize) -> String {
    format!("ce dimension ({subtype}) — p. {page_number}")
}

/// A row's byline — its author and its modification date, when it has them.
#[must_use]
pub fn comment_row_byline(author: Option<&str>, modified: Option<&str>) -> Option<String> {
    match (author, modified) {
        (Some(a), Some(m)) => Some(format!("by {a} · modified {m}")),
        (Some(a), None) => Some(format!("by {a}")),
        (None, Some(m)) => Some(format!("modified {m}")),
        (None, None) => None,
    }
}

/// Why a modification date can look like machine output.
#[must_use]
pub fn comment_row_modified_tooltip() -> &'static str {
    "Shown exactly as the file wrote it. The standard lets this be a date or any text at all, and requires a reader to accept whatever is there, so pdfcer does not reformat it."
}

/// A row's note body.
#[must_use]
pub fn comment_row_body(text: &str) -> String {
    text.to_owned()
}

/// A row's caption when the annotation has no `/Contents`.
#[must_use]
pub fn comment_row_no_note() -> &'static str {
    "No note text on this markup."
}

/// A **ce dimension**'s caption when it has no `/Contents`.
#[must_use]
pub fn comment_row_ce_dimension_no_note() -> &'static str {
    "No note text. A ce dimension carries its measurement in its own appearance on the page rather than as a note."
}

/// The caption under `/Contents` on a subtype that does not display text.
#[must_use]
pub fn comment_row_description_caption() -> &'static str {
    "This is the annotation's accessibility description, not a note somebody wrote — this kind of annotation displays no text of its own."
}

/// A row whose annotation the file says not to show on screen.
#[must_use]
pub fn comment_row_hidden() -> &'static str {
    "The document marks this one as not shown on screen, so it will not be on the page when you get there."
}

/// A row whose appearance state pdfcer could not resolve.
#[must_use]
pub fn comment_row_appearance_unresolved() -> &'static str {
    "pdfcer could not work out which appearance this annotation should use, so it draws nothing for it. That is pdfcer declining to guess, not a fault in the page."
}

/// A row that is a **reply** to another annotation (`/IRT` with `/RT /R`).
#[must_use]
pub fn comment_row_is_reply() -> &'static str {
    "A reply to another annotation on this document."
}

/// A row that is a `/RT /Group` **subordinate**.
#[must_use]
pub fn comment_row_is_group_member() -> &'static str {
    "Part of a group. The standard says a reader should show the group's main annotation's note here instead; pdfcer shows what this one actually says, so another viewer may show something different."
}

/// The go-to-page button on a row.
///
/// Salvaged verbatim.
#[must_use]
pub fn comment_row_goto() -> &'static str {
    "Go to"
}

/// Its tooltip — names the page, because the button sits in a list of many.
///
/// Salvaged verbatim.
#[must_use]
pub fn comment_row_goto_tooltip(page_number: usize) -> String {
    format!("Show page {page_number}, where this is")
}

/// **The control that opens the note editor on a row that has no note.**
#[must_use]
pub fn comment_row_add_note() -> &'static str {
    "Add note"
}

/// The same control on a row that already carries note text.
#[must_use]
pub fn comment_row_edit_note() -> &'static str {
    "Edit note"
}

/// The editor's Save.
#[must_use]
pub fn comment_row_note_save() -> &'static str {
    "Save note"
}

/// The editor's Cancel — abandons the draft and writes nothing.
#[must_use]
pub fn comment_row_note_cancel() -> &'static str {
    "Cancel"
}

/// **Remove the note entirely**, leaving the shape on the page.
#[must_use]
pub fn comment_row_note_remove() -> &'static str {
    "Remove note"
}

/// Its tooltip — says what survives, because the button sits next to a Delete
/// in the operator's mental model even though it is not one.
#[must_use]
pub fn comment_row_note_remove_tooltip() -> &'static str {
    "Remove the words, the author and the date. The markup itself stays on the page."
}

/// The hint under the editor while it is open.
#[must_use]
pub fn comment_row_note_hint() -> &'static str {
    "Enter starts a new line. Save note and Escape both write it. Cancel leaves the note as it was."
}

/// **What the editor will write into `/T`, disclosed before it is written.**
#[must_use]
pub fn comment_row_note_signature() -> &'static str {
    "Saving signs this with the name in Settings > Comments, and dates it. Leave that name blank to comment anonymously."
}

/// The same disclosure on a row that **already has an author** — what is
/// preserved, rather than what is written.
#[must_use]
pub fn comment_row_note_signature_kept(author: &str) -> String {
    format!("This note stays credited to {author}. Saving updates its date.")
}

/// The caption on a row whose note cannot be edited here, and where its text
/// actually lives.
#[must_use]
pub fn comment_row_note_not_editable_ce_dimension() -> &'static str {
    "A ce dimension's text comes from its measurement. Use the Measure tools to change it."
}

/// The caption on a row whose annotation is written as a **direct dictionary**
/// and therefore has no object id to name.
#[must_use]
pub fn comment_row_note_no_handle() -> &'static str {
    "This annotation is written into the page rather than as its own object, so pdfcer cannot address it."
}

/// **The heading of the row whose annotation is selected on the canvas.**
#[must_use]
pub fn comment_row_selected_heading(heading: &str) -> String {
    format!("> {heading} — selected on the page")
}

// ---------------------------------------------------------------------------
// The filter strip, and Delete
// ---------------------------------------------------------------------------

/// **The disclosure a filtered list owes**, above the rows.
#[must_use]
pub fn comments_filtered(shown: usize, total: usize) -> String {
    format!("Showing {shown} of {total}. A filter is hiding the rest.")
}

/// The control that puts every row back.
#[must_use]
pub fn comment_filter_clear() -> &'static str {
    "Show all"
}

/// The author chooser's label.
#[must_use]
pub fn comment_filter_author() -> &'static str {
    "Author"
}

/// The type chooser's label. *Type* rather than *Subtype*, because the values
/// under it are the file's own spellings and the operator does not need the
/// dictionary key's name to use them.
#[must_use]
pub fn comment_filter_type() -> &'static str {
    "Type"
}

/// The "no filter" entry in either chooser.
#[must_use]
pub fn comment_filter_all() -> &'static str {
    "All"
}

/// The switch that hides rows carrying no note text.
#[must_use]
pub fn comment_filter_with_note() -> &'static str {
    "With text only"
}

/// The ordering chooser's label.
#[must_use]
pub fn comment_sort_label() -> &'static str {
    "Order"
}

/// The default ordering — page order, then `/Annots` order.
///
/// Named *By page* rather than *Document order*, because the operator's
/// question is *"where is it"* and the sheet number is the answer they act on.
#[must_use]
pub fn comment_sort_document() -> &'static str {
    "By page"
}

/// Ordering by `/T`.
#[must_use]
pub fn comment_sort_author() -> &'static str {
    "By author"
}

/// Ordering by `/Subtype`.
#[must_use]
pub fn comment_sort_subtype() -> &'static str {
    "By type"
}

/// **Delete this comment** — the control this panel spent its whole life
/// without.
#[must_use]
pub fn comment_row_delete() -> &'static str {
    "Delete comment"
}

/// Its tooltip, carrying the three things `docs/core-api/03-capabilities.md`
/// §3.4 requires a delete to disclose.
#[must_use]
pub fn comment_row_delete_tooltip() -> &'static str {
    "Remove this markup and its note from the page. This is not redaction: saving without rewriting the whole file leaves the previous revision in place."
}

// ===========================================================================
// ANSWERING A COMMENT — `EditSession::add_reply`
// ===========================================================================
//
// These strings exist because the engine gained the verb, and the shape
// worth remembering is the one they were blocked on: `pdfcer-core` MODELLED
// `/IRT` and `/RT` long before it could WRITE either, so this panel could
// display a conversation and not continue one. R9 forbids a greyed Reply
// button for a capability no state of the program can reach, so nothing was
// drawn and nothing was worded.
//
// ⇒ **An absence with a reason is still an absence with a shelf life.** A
// catalog that had said "there is no Reply and there cannot be" would have
// gone on saying it the afternoon the verb landed, with nothing in this file
// changed and nothing to notice.

/// The control that opens the reply editor on a row.
#[must_use]
pub fn comment_row_reply() -> &'static str {
    "Reply"
}

/// What Reply does, on hover — and the one fact about it an operator cannot
/// see.
#[must_use]
pub fn comment_row_reply_tooltip() -> &'static str {
    "Write a new comment that answers this one. The comment you are answering is not changed."
}

/// The reply editor's commit.
#[must_use]
pub fn comment_row_reply_save() -> &'static str {
    "Post reply"
}

/// The hint under the reply editor.
#[must_use]
pub fn comment_row_reply_hint() -> &'static str {
    "Enter starts a new line. Post reply and Escape both send it. Cancel discards it."
}

/// **What the reply will be signed with**, disclosed before it is written.
#[must_use]
pub fn comment_row_reply_signature() -> &'static str {
    "Your reply is signed with the name in Settings > Comments and dated now. Leave that name blank to reply anonymously."
}

/// **Shown when the row being answered is itself a reply** — the
/// threading-depth decision, said out loud where it is made.
#[must_use]
pub fn comment_row_reply_to_a_reply() -> &'static str {
    "You are answering a reply. Your answer joins the same conversation and is listed with it, rather than nested under it."
}

/// **What the engine did that no surface in this program shows** — the
/// disclosure after a reply lands.
#[must_use]
pub fn reply_posted(has_popup: bool) -> Option<&'static str> {
    has_popup.then_some(
        "Your reply was added to the conversation. It also carries a pop-up window of its own, which pdfcer shows inside the comment you answered and other readers may show separately.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every fixed sentence in this module, for the sweeps below.
    fn all_fixed() -> [&'static str; 15] {
        [
            comment_row_reply_tooltip(),
            comment_row_reply_hint(),
            comment_row_reply_signature(),
            comment_row_reply_to_a_reply(),
            comments_none(),
            comments_all_without_notes(),
            comment_row_modified_tooltip(),
            comment_row_no_note(),
            comment_row_ce_dimension_no_note(),
            comment_row_description_caption(),
            comment_row_hidden(),
            comment_row_appearance_unresolved(),
            comment_row_is_reply(),
            comment_row_is_group_member(),
            comment_row_goto(),
        ]
    }

    /// **Rule 15 — no string here writes a bare "dimension".**
    ///
    /// *"**ce dimensions** are the ones pdfcer authors; **pdf dimensions** are
    /// CAD-exported page content pdfcer reads and must not silently alter. They
    /// have opposite properties and the ambiguity has already sent one
    /// investigation down the wrong path."*
    ///
    /// Swept rather than reviewed because a catalog is exactly the kind of
    /// file where a bare noun arrives in a late reword — someone shortens a
    /// sentence that ran long in a screenshot, and the qualifier is the first
    /// thing to go. The two headings that legitimately contain the word are
    /// checked for their qualifier rather than exempted, so the test still
    /// bites if one of them loses it.
    #[test]
    fn no_string_here_says_a_bare_dimension() {
        let qualified = |s: &str| {
            // Every occurrence must be preceded by "ce " or "pdf ".
            let lower = s.to_lowercase();
            let mut at = 0;
            while let Some(found) = lower[at..].find("dimension") {
                let start = at + found;
                let before = &lower[..start];
                if !(before.ends_with("ce ") || before.ends_with("pdf ")) {
                    return false;
                }
                at = start + "dimension".len();
            }
            true
        };

        for s in all_fixed() {
            assert!(
                qualified(s),
                "this string writes a bare \"dimension\": {s}. Rule 15 — say \
                 \"ce dimension\" or \"pdf dimension\"."
            );
        }
        // The two formatted entries that DO name the concept, checked rather
        // than exempted.
        let heading = comment_row_ce_dimension_heading("Line", 3);
        assert!(qualified(&heading), "{heading}");
        assert!(qualified(comment_row_ce_dimension_no_note()));
        // …and the sweep can actually fail, which is the half a green run
        // never proves.
        assert!(
            !qualified("the dimension is wrong"),
            "the rule-15 check cannot detect its own violation"
        );
    }

    /// **The exclusion line is `None` when nothing was excluded.**
    #[test]
    fn nothing_excluded_draws_nothing() {
        assert_eq!(comments_excluded(0, 0, 0), None);
    }

    /// …and it names every kind that was excluded, and only those.
    #[test]
    fn the_exclusion_line_names_where_each_kind_went() {
        let only_widgets = comments_excluded(12, 0, 0).expect("something was excluded");
        assert!(only_widgets.contains("12"), "{only_widgets}");
        assert!(only_widgets.contains("Forms panel"), "{only_widgets}");
        assert!(
            !only_widgets.contains("pop-up"),
            "a kind that was not excluded must not be mentioned: {only_widgets}"
        );

        let all_three = comments_excluded(1, 2, 3).expect("something was excluded");
        for n in ["1", "2", "3"] {
            assert!(all_three.contains(n), "{all_three}");
        }
        assert!(all_three.contains("pop-up"), "{all_three}");
        assert!(all_three.contains("prepress"), "{all_three}");
    }

    /// **The byline is `None` only when the annotation has neither half.**
    #[test]
    fn a_byline_with_neither_half_is_not_drawn() {
        assert_eq!(comment_row_byline(None, None), None);
        assert_eq!(
            comment_row_byline(Some("Ken"), None).as_deref(),
            Some("by Ken")
        );
        let modified = comment_row_byline(None, Some("D:20240117093000Z"))
            .expect("a date alone is still a byline");
        assert!(modified.contains("D:20240117093000Z"), "{modified}");
        assert!(
            !modified.contains("by "),
            "an absent author must not produce the word \"by\": {modified}"
        );
        let both = comment_row_byline(Some("Ken"), Some("D:20240117093000Z"))
            .expect("both halves is still a byline");
        assert!(both.contains("Ken") && both.contains("D:2024"), "{both}");
    }

    /// **The date is passed through byte for byte.**
    #[test]
    fn a_modification_date_is_never_reformatted() {
        for raw in [
            "D:20240117093000Z",
            "D:19980223085612-04'00",
            "last Tuesday",
            "",
        ] {
            let line = comment_row_byline(None, Some(raw));
            match line {
                Some(line) => assert!(line.contains(raw), "{raw} was altered: {line}"),
                None => panic!("a present /M must produce a byline, even an odd one"),
            }
        }
    }

    /// **The "no note" caption reads as a fact, not as an error.**
    #[test]
    fn an_absent_note_is_never_described_as_missing_or_broken() {
        for caption in [comment_row_no_note(), comment_row_ce_dimension_no_note()] {
            let lower = caption.to_lowercase();
            for alarm in ["missing", "error", "fail", "invalid", "corrupt", "warning"] {
                assert!(
                    !lower.contains(alarm),
                    "`{caption}` describes an ordinary state with the word \
                     \"{alarm}\", which reads as damage"
                );
            }
        }
        // And the document-wide disclosure says the absence is expected, in
        // that word — `03-capabilities.md:1085` asks for it in these terms.
        assert!(
            comments_all_without_notes().contains("expected"),
            "{}",
            comments_all_without_notes()
        );
    }

    /// **Every fixed sentence is prose or a label, never both halves of one.**
    #[test]
    fn prose_is_punctuated_and_the_one_label_is_not() {
        for s in all_fixed() {
            assert!(!s.is_empty(), "an empty catalog entry");
            if s == comment_row_goto() {
                assert!(!s.ends_with('.'), "`{s}` is a button label, not a sentence");
            } else {
                assert!(
                    s.ends_with('.'),
                    "`{s}` is prose and must be punctuated as a sentence"
                );
            }
        }
    }

    /// **No two of these sentences are the same.**
    #[test]
    fn every_row_state_says_something_different() {
        let mut seen: Vec<&str> = Vec::new();
        for s in all_fixed() {
            assert!(!seen.contains(&s), "two states share the sentence `{s}`");
            seen.push(s);
        }
    }

    /// The two page-numbered entries print the number they were given.
    #[test]
    fn a_page_number_reaches_the_string_unchanged() {
        assert!(comment_row_heading("Circle", 7).contains('7'));
        assert!(comment_row_ce_dimension_heading("Line", 7).contains('7'));
        assert!(comment_row_goto_tooltip(7).contains('7'));
        // The subtype survives too, including the malformed-annotation label
        // core hands over when `/Subtype` is absent.
        assert!(comment_row_heading("(no Subtype)", 1).contains("(no Subtype)"));
    }

    /// A note body is returned byte for byte.
    ///
    /// The operator is reading somebody else's words; a catalog entry that
    /// decorated them would put pdfcer's voice inside a quotation.
    #[test]
    fn a_note_body_is_not_decorated() {
        for text in ["Check this weld", "  leading space", "", "多行\ntext"] {
            assert_eq!(comment_row_body(text), text);
        }
    }

    /// **A reply's own pop-up is disclosed, and only when it has one.**
    #[test]
    fn a_replys_own_popup_is_disclosed_only_when_the_engine_reports_one() {
        let told = reply_posted(true).expect("a reply with a pop-up owes a sentence");
        assert!(
            told.contains("pop-up"),
            "the sentence must name the thing it is disclosing: {told}"
        );
        assert!(
            told.contains("other readers") || told.contains("others"),
            "the whole point is that another reader draws it: {told}"
        );
        assert_eq!(
            reply_posted(false),
            None,
            "a reply with no pop-up has nothing to disclose, and a sentence \
             about a window that is not there is an invention"
        );
    }
}
