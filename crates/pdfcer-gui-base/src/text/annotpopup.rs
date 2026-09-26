//! # `text::annotpopup` — every string the note pop-up on the canvas shows
//!
//! The copy for `pdfcer_gui::canvas::notepopup` — the window that opens when an
//! operator clicks a comment on the page, and the tooltip that appears when
//! they hover one.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/annotpopup.md`.

/// The pop-up's heading: what kind of annotation this is.
#[must_use]
pub fn popup_heading(subtype: &str) -> String {
    format!("{subtype} comment")
}

/// The heading for a **ce dimension**'s pop-up.
#[must_use]
pub fn popup_ce_dimension_heading(subtype: &str) -> String {
    format!("ce dimension comment [{subtype}]")
}

/// The note's own words, undecorated.
#[must_use]
pub fn popup_body(text: &str) -> String {
    text.to_owned()
}

/// Shown in place of the body when the annotation carries no `/Contents`.
#[must_use]
pub fn popup_no_note() -> &'static str {
    "No note has been written on this markup."
}

/// Shown in place of the body when the annotation is a ce dimension.
#[must_use]
pub fn popup_ce_dimension_note() -> &'static str {
    "This text is the measurement pdfcer wrote, not a note. Editing it here would be discarded the next time the ce dimension is redrawn."
}

/// The close control on the pop-up's title row.
///
/// A multiplication sign rather than the letter x: it is the glyph every
/// window close affordance in the class uses, and the letter reads as content.
#[must_use]
pub fn popup_close() -> &'static str {
    "\u{00d7}"
}

/// What the close control does — on hover, because the glyph alone is
/// conventional enough not to need a caption on the row.
#[must_use]
pub fn popup_close_tooltip() -> &'static str {
    "Hide this note on screen. The file's own open-or-closed setting is unchanged; use Open by default to record it."
}

/// The control that opens the editor inside the pop-up.
#[must_use]
pub fn popup_edit() -> &'static str {
    "Edit note"
}

/// The same control when the annotation has no note yet.
#[must_use]
pub fn popup_add() -> &'static str {
    "Add note"
}

/// Commit the edited note.
#[must_use]
pub fn popup_save() -> &'static str {
    "Save note"
}

/// Abandon the edit.
#[must_use]
pub fn popup_cancel() -> &'static str {
    "Cancel"
}

/// Remove the note's text, keeping the annotation.
#[must_use]
pub fn popup_remove() -> &'static str {
    "Remove note"
}

/// What *Remove note* does, and what it does not.
#[must_use]
pub fn popup_remove_tooltip() -> &'static str {
    "Delete the words and keep the markup on the page."
}

/// Remove the whole annotation.
#[must_use]
pub fn popup_delete() -> &'static str {
    "Delete comment"
}

/// What *Delete comment* does, including the part that is not obvious.
#[must_use]
pub fn popup_delete_tooltip() -> &'static str {
    "Remove this markup and its note from the page. This is not redaction: saving without rewriting the whole file leaves the previous revision in place."
}

/// The heading above a comment's replies.
#[must_use]
pub fn popup_replies(count: usize) -> String {
    format!("{count} repl(y/ies)")
}

/// Beside a reply that is a §12.5.6.2 **group member** rather than an ordinary
/// reply.
#[must_use]
pub fn popup_reply_is_group_member() -> &'static str {
    "Grouped with the comment above. Other readers show the group's text here instead of this."
}

/// Shown in place of a reply's body when it carries no `/Contents`.
#[must_use]
pub fn popup_reply_no_note() -> &'static str {
    "No text."
}

/// Why there is no editor in Read mode.
#[must_use]
pub fn popup_read_only() -> &'static str {
    "Read mode shows comments and does not change them. Switch to Review to edit this note."
}

/// Why there is no editor on an annotation the file has locked.
#[must_use]
pub fn popup_locked() -> &'static str {
    "The document locks this comment, so its note cannot be changed here."
}

/// The hover tooltip over a comment on the page.
#[must_use]
pub fn popup_tooltip(author: Option<&str>, contents: Option<&str>) -> String {
    let words = match contents.map(str::trim).filter(|t| !t.is_empty()) {
        Some(text) => truncate(text),
        None => popup_no_note().to_owned(),
    };
    match author.map(str::trim).filter(|a| !a.is_empty()) {
        Some(author) => format!("{author}\n{words}"),
        None => words,
    }
}

/// The hint under the pop-up's editor.
#[must_use]
pub fn popup_note_hint() -> &'static str {
    "Save note and Escape both write the note. Cancel leaves it as it was."
}

// ===========================================================================
// THE FILE'S OWN `/Open` — `EditSession::set_annotation_open`, `Pass 253.3`
// ===========================================================================
//
// Two states with the same name, and the whole of this group exists to
// keep them apart in the operator's head:
//
// | | who owns it | undo | survives closing the document |
// |---|---|---|---|
// | **is this bubble showing right now** | `canvas::notepopup::open` — interface state, keyed by document path | none | no |
// | **does this comment open when the file is opened** | the document's `/Open` (§12.5.6.4 Table 172, §12.5.6.14 Table 183) | one entry per press | yes |
//
// The ✕ moves the first. [`popup_open_default`] moves the second. An operator
// who could not tell them apart would either believe every click was editing
// their file, or believe none of them could.

/// The control that records this comment's window state **in the document**.
#[must_use]
pub fn popup_open_default() -> &'static str {
    "Open by default"
}

/// What ticking it does, on hover — and the one thing about it that costs
/// something.
#[must_use]
pub fn popup_open_default_tooltip() -> &'static str {
    "Write this into the file, so the comment opens the same way for the next reader. This is a document change and can be undone."
}

/// **What the engine actually wrote**, for the case where it wrote
/// nothing.
#[must_use]
pub fn open_state_written(annotation: bool, popup: bool) -> Option<&'static str> {
    (!annotation && !popup).then_some(
        "This markup has no pop-up window, so there was nothing to record. The document was not changed.",
    )
}

/// How many characters of a note the tooltip shows before eliding.
///
/// 160 — about two lines at the tooltip's natural width, which is enough to
/// recognise a comment you wrote and short enough not to cover the sheet.
const TOOLTIP_CHARS: usize = 160;

/// Cut `text` to [`TOOLTIP_CHARS`] characters, appending an ellipsis when it
/// had to.
fn truncate(text: &str) -> String {
    let flat: String = text
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let flat = flat.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= TOOLTIP_CHARS {
        return flat;
    }
    let head: String = flat.chars().take(TOOLTIP_CHARS).collect();
    format!("{head}\u{2026}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Rule 15: no string here says a bare "dimension".**
    #[test]
    fn no_string_here_says_a_bare_dimension() {
        let strings = [
            popup_heading("Text"),
            popup_ce_dimension_heading("Line"),
            popup_no_note().to_owned(),
            popup_ce_dimension_note().to_owned(),
            popup_close_tooltip().to_owned(),
            popup_edit().to_owned(),
            popup_add().to_owned(),
            popup_save().to_owned(),
            popup_cancel().to_owned(),
            popup_remove().to_owned(),
            popup_remove_tooltip().to_owned(),
            popup_delete().to_owned(),
            popup_delete_tooltip().to_owned(),
            popup_replies(2),
            popup_reply_is_group_member().to_owned(),
            popup_reply_no_note().to_owned(),
            popup_read_only().to_owned(),
            popup_locked().to_owned(),
            popup_note_hint().to_owned(),
            popup_open_default().to_owned(),
            popup_open_default_tooltip().to_owned(),
            open_state_written(false, false)
                .expect("the no-op case has a sentence")
                .to_owned(),
        ];
        for s in &strings {
            let lower = s.to_lowercase();
            for (at, _) in lower.match_indices("dimension") {
                let before = lower[..at].trim_end();
                assert!(
                    before.ends_with("ce") || before.ends_with("pdf"),
                    "rule 15: `{s}` says a bare \"dimension\". Write `ce dimension` \
                     (the ones pdfcer authors) or `pdf dimension` (CAD-exported \
                     page content) — never the bare noun."
                );
            }
        }
    }

    /// **A short note is shown whole.**
    ///
    /// The half that makes the truncation test mean something: a tooltip that
    /// elided everything would also pass an "it ends with an ellipsis" check.
    #[test]
    fn a_short_note_is_not_truncated() {
        let tip = popup_tooltip(Some("Ken Mantle"), Some("Check this weld"));
        assert!(tip.contains("Ken Mantle"), "{tip}");
        assert!(tip.contains("Check this weld"), "{tip}");
        assert!(!tip.contains('\u{2026}'), "{tip}");
    }

    /// **A long note is cut, and says that it was.**
    #[test]
    fn a_long_note_is_cut_and_says_so() {
        let long = "w".repeat(TOOLTIP_CHARS * 2);
        let tip = popup_tooltip(None, Some(&long));
        assert!(tip.ends_with('\u{2026}'), "{tip}");
        assert_eq!(tip.chars().count(), TOOLTIP_CHARS + 1, "{tip}");
    }

    /// **The tooltip does not panic on a multi-byte note.**
    #[test]
    fn a_multibyte_note_is_cut_safely() {
        let long = "\u{6f22}".repeat(TOOLTIP_CHARS * 2);
        let tip = popup_tooltip(None, Some(&long));
        assert_eq!(tip.chars().count(), TOOLTIP_CHARS + 1, "{tip}");
    }

    /// **An anonymous note shows its words and claims no author.**
    #[test]
    fn an_anonymous_note_gets_no_byline() {
        let tip = popup_tooltip(None, Some("words"));
        assert_eq!(tip, "words");
        // …and whitespace counts as absent, exactly as it does for the panel's
        // byline: `/T ( )` is a byline nobody wrote.
        assert_eq!(popup_tooltip(Some("  "), Some("words")), "words");
    }

    /// **A note with no words still gets a tooltip**, saying so.
    #[test]
    fn a_note_with_no_words_still_says_something() {
        let tip = popup_tooltip(Some("Ken"), None);
        assert!(tip.contains("Ken"), "{tip}");
        assert!(tip.contains(popup_no_note()), "{tip}");
        // An empty string is the same case as an absent one: a producer
        // writing `/Contents ()` has written no note.
        assert!(popup_tooltip(None, Some("   ")).contains(popup_no_note()));
    }

    /// The two heading forms differ, and only the ce-dimension one names a ce
    /// dimension.
    #[test]
    fn only_a_ce_dimension_heading_says_ce_dimension() {
        let ce = popup_ce_dimension_heading("Line");
        assert!(ce.contains("ce dimension"), "{ce}");
        assert!(ce.contains("Line"), "{ce}");
        let plain = popup_heading("Line");
        assert!(!plain.contains("dimension"), "{plain}");
        assert!(plain.contains("Line"), "{plain}");
    }

    /// **The no-op sentence fires only when the engine wrote nothing.**
    #[test]
    fn only_a_write_that_reached_nothing_says_so() {
        // Neither object carried the key: the call succeeded, changed nothing
        // and pushed no undo entry. The operator is owed the sentence, because
        // a control that did nothing and said nothing is indistinguishable
        // from a broken build.
        let nothing = open_state_written(false, false).expect("the no-op case has a sentence");
        assert!(
            nothing.contains("not changed"),
            "the no-op sentence must say the document was not changed: {nothing}"
        );
        assert!(
            !nothing.to_lowercase().contains("error") && !nothing.to_lowercase().contains("fail"),
            "a shape with no pop-up is an ordinary document, not a fault: {nothing}"
        );

        // The three shapes of success say nothing at all.
        assert_eq!(open_state_written(true, true), None);
        assert_eq!(open_state_written(true, false), None);
        assert_eq!(open_state_written(false, true), None);
    }
}
