//! # `text::annotpopup` — every string the note pop-up on the canvas shows
//!
//! The copy for [`crate::canvas::notepopup`] — the window that opens when an
//! operator clicks a comment on the page, and the tooltip that appears when
//! they hover one.
//!
//! ## ★★★ The surface this exists for, and the report that commissioned it
//!
//! The operator, 2026-09-05:
//!
//! > *"I could add a yellow sticky note but even in read mode I don't think I
//! > could figure out how to read it. the review features should look and act
//! > the same as they do in Acrobat Reader."*
//!
//! Before this catalog there was **no string anywhere in the crate that
//! displayed a note's `/Contents` on the canvas**, in any mode. The only route
//! to a comment's words was the Comments panel, on a tab Read is not shown.
//!
//! ## ★★ Why this is not part of [`crate::text::panels::comments`]
//!
//! Because they are two surfaces answering two questions, and the wording
//! follows the question rather than the data.
//!
//! The panel is a **work list**: its rows are headed by subtype and page
//! because a reviewer scanning forty of them needs to tell two clouds on sheet
//! three apart, and every caption on a row is a *disclosure about the list*.
//! The pop-up is **one comment, beside the thing it is about**: the operator
//! already knows which annotation they clicked, so a heading that repeated the
//! page number would be answering a question the click just settled.
//!
//! ⇒ Which is why, for instance, [`popup_heading`] takes no page number and
//! `comment_row_heading` does.
//!
//! ## ★ What is shared rather than restated
//!
//! **The byline.** `crate::text::panels::comments::comment_row_byline` is
//! called directly by the pop-up rather than copied here, and that is
//! deliberate: it carries a settled ruling about `/M` — §12.5.2 gives its type
//! as *"date **or** text string"* and requires a reader to accept any format,
//! so pdfcer shows it verbatim rather than writing a parser whose failure mode
//! is rejecting a legal value. Two surfaces showing one comment must not show
//! two different dates for it, and the only way that cannot happen is one
//! function.
//!
//! ## ★★★ R9 governs what is ABSENT here, and two absences are deliberate
//!
//! *"An unavailable capability renders nothing, not a disabled stub. Greying
//! is reserved for temporarily unavailable, and must explain on hover."*
//!
//! 1. **There is no Reply control and no string for one.** `pdfcer-core`
//!    v0.38.0 reads `/IRT` and `/RT` and has no verb of any kind that writes
//!    either — audited 2026-09-05; the only write-side occurrences in the
//!    crate are two *destructive* ones (the deletion cascade at
//!    `edit.rs:24969` and the clipboard key-strip at `edit.rs:10673`). A
//!    greyed Reply button would promise a state of the program that does not
//!    exist. Filed as `request_a_reply_can_be_read_and_never_written.md`.
//! 2. **There is no Accepted/Rejected control and no string for one.**
//!    `/State` and `/StateModel` (§12.5.6.4 Table 171) are **absent from the
//!    engine entirely** — zero occurrences, read or write. Filed as
//!    `request_review_status_is_not_modelled_at_all.md`.
//!
//! ⇒ The one place this catalog *does* speak about a missing capability is
//! [`popup_read_only`], and the difference is the rule: Read mode's inability
//! to edit is **temporary in the operator's own hands** — the mode selector is
//! two clicks away — so it is exactly the case R9 permits to be explained.
//!
//! ## Conventions, restated from [`crate::text`] because they bind here
//!
//! - **Sentence case, no trailing period on labels; full sentences with
//!   punctuation for prose.**
//! - **The operator's own words are never decorated.** [`popup_body`] is a
//!   passthrough for the same reason `comment_row_body` is: the operator is
//!   reading somebody else's remark, and a catalog entry that framed it would
//!   be putting pdfcer's voice inside a quotation.
//! - **Rule 15**: never a bare *dimension*. [`tests::no_string_here_says_a_bare_dimension`]
//!   sweeps every entry, exactly as the Comments catalog does — this is a
//!   catalog, which is the kind of file where a bare noun slips in during a
//!   late reword.

/// The pop-up's heading: what kind of annotation this is.
///
/// No page number, unlike the panel's row heading — see the module header. The
/// subtype is the file's own spelling (`Text`, `Square`, `Line`), because that
/// is the word the operator used when they placed it and the word every other
/// surface in this shell uses for it.
#[must_use]
pub fn popup_heading(subtype: &str) -> String {
    format!("{subtype} comment")
}

/// The heading for a **ce dimension**'s pop-up.
///
/// Project rule 15 at the point of use, and the same shape
/// `comment_row_ce_dimension_heading` takes: a **ce dimension** is a `/Line`
/// annotation, and the bracketed subtype is not decoration — the reason a
/// dimension appears on this surface at all is that it *is* one, and a heading
/// that hid that would quietly contradict the argument that let it in.
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
///
/// Worded as a fact about the document rather than as missing data. On markup
/// pdfcer itself drew this is the **expected** state: `MarkupSpec` has no
/// contents field on any variant, deliberately, so a shape this shell authored
/// has no note until somebody writes one.
#[must_use]
pub fn popup_no_note() -> &'static str {
    "No note has been written on this markup."
}

/// Shown in place of the body when the annotation is a ce dimension.
///
/// Its `/Contents` is **regenerated from the measurement** by
/// `author_dimension`, so it is never a remark somebody wrote and a note typed
/// over it would be silently thrown away. Rule 15 and R9 together: the
/// capability is not withheld with a greyed control, it is explained.
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
///
/// ★ It says **screen**, and that is the honest half. Closing a pop-up is
/// interface state and is not written back to the file: `pdfcer-core` v0.38.0
/// has no verb that can change an existing annotation's `/Open`. Saying
/// "close" alone would let an operator conclude they had changed the document.
#[must_use]
pub fn popup_close_tooltip() -> &'static str {
    "Hide this note on screen. The file's own open-or-closed setting is unchanged."
}

/// The control that opens the editor inside the pop-up.
#[must_use]
pub fn popup_edit() -> &'static str {
    "Edit note"
}

/// The same control when the annotation has no note yet.
///
/// Two labels rather than one, because *Add* and *Edit* are different acts and
/// a reviewer scanning a sheet's pop-ups can tell at a glance which comments
/// have been written on.
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
///
/// The distinction the engine draws by having two verbs:
/// `clear_markup_note` *"does **not** delete the annotation — the shape stays
/// and undo restores the words"*, while `delete_annotation` is the other
/// thing. Both controls are on this pop-up, so the difference has to be
/// legible without pressing either.
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
///
/// ★★ Three things, and each is required by `docs/core-api/03-capabilities.md`
/// §3.4: what it removes, that **delete is not redaction**, and — implied by
/// the second — that a previous revision of the file may still hold it.
#[must_use]
pub fn popup_delete_tooltip() -> &'static str {
    "Remove this markup and its note from the page. This is not redaction: saving without rewriting the whole file leaves the previous revision in place."
}

/// The heading above a comment's replies.
///
/// The count is in the heading rather than left to be counted, because a
/// thread scrolled past its third entry is one an operator cannot count by
/// eye — and the number is what tells them there is more below the fold.
#[must_use]
pub fn popup_replies(count: usize) -> String {
    format!("{count} repl(y/ies)")
}

/// Beside a reply that is a §12.5.6.2 **group member** rather than an ordinary
/// reply.
///
/// ★★ Rule 4, and it is the same disclosure `comment_row_is_group_member`
/// makes for the same reason: for a group subordinate the standard says its
/// own `/Contents`, `/M`, `/T` and the rest *"shall be ignored"* in favour of
/// the group primary's. `pdfcer-core` deliberately does not apply that rule,
/// so what is shown here is the raw dictionary value — and another conforming
/// reader will legitimately show something else.
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
///
/// # ★★★ The one place this catalog explains an absence, and why R9 allows it
///
/// R9 reserves an explanation for a capability that is **temporarily**
/// unavailable, and Read mode is the purest example of that in the whole
/// program: the capability is not missing, the operator has *chosen a stance*,
/// and the control that changes it is a labelled three-position selector on
/// the ribbon. Saying so is not a placeholder; it is the answer to *"why can I
/// read this and not fix the typo?"*, which has exactly one correct answer and
/// it is short.
///
/// ★ It names the mode to switch to rather than the mode you are in. *"You are
/// in Read mode"* is a fact the badge already states; *"Review lets you edit
/// comments"* is the sentence that gets the operator to the thing they wanted.
#[must_use]
pub fn popup_read_only() -> &'static str {
    "Read mode shows comments and does not change them. Switch to Review to edit this note."
}

/// Why there is no editor on an annotation the file has locked.
///
/// §12.5.3 Table 165 bit 8: the file says the user interface *"shall not"*
/// allow the annotation's properties to be changed. R83 — the controls are
/// omitted rather than offered and refused — and this sentence is why they are
/// not there, because otherwise a locked comment is indistinguishable from a
/// broken pop-up.
#[must_use]
pub fn popup_locked() -> &'static str {
    "The document locks this comment, so its note cannot be changed here."
}

/// The hover tooltip over a comment on the page.
///
/// # ★★ Why the tooltip exists when a click opens the whole window
///
/// Because it is the cheap half of the same affordance and every reader in the
/// class has it: hovering answers *"what is this?"* without committing to
/// opening anything, which is what a reviewer skimming a sheet of forty marks
/// is doing. Acrobat shows author and text on hover; so does this.
///
/// # ★ It truncates, and the truncation is visible
///
/// A tooltip that grew to a paragraph would cover the drawing it is about — a
/// note is arbitrary operator text and can be a page of it. The ellipsis is
/// the disclosure: it says there is more, and clicking is how you get it. The
/// pop-up itself never truncates.
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
///
/// Escape rather than the Cancel button, because a reviewer typing has their
/// hands on the keyboard and the key is the faster route — and because a
/// keyboard route that nothing announces is a keyboard route nobody finds.
#[must_use]
pub fn popup_note_hint() -> &'static str {
    "Press Escape to leave the note unchanged."
}

/// How many characters of a note the tooltip shows before eliding.
///
/// 160 — about two lines at the tooltip's natural width, which is enough to
/// recognise a comment you wrote and short enough not to cover the sheet.
const TOOLTIP_CHARS: usize = 160;

/// Cut `text` to [`TOOLTIP_CHARS`] characters, appending an ellipsis when it
/// had to.
///
/// **Characters, not bytes** — slicing a `String` by byte index panics in the
/// middle of a multi-byte character, and a note is arbitrary operator text
/// that may be in any script. Newlines are collapsed to spaces for the same
/// reason the length is bounded: a tooltip is one gesture's worth of
/// information, and a note's own paragraph breaks would make it a document.
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

    /// ★★★ **Rule 15: no string here says a bare "dimension".**
    ///
    /// The same sweep `crate::text::panels::comments` runs, over this catalog,
    /// and for its reason: **ce dimensions** are the ones pdfcer authors and
    /// **pdf dimensions** are CAD-exported page content, they have opposite
    /// properties, and the ambiguity has already sent one investigation down
    /// the wrong path. A catalog is exactly the kind of file where a bare noun
    /// slips in during a late reword, so this is swept rather than reviewed.
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
    ///
    /// The ellipsis is the disclosure — rule 4's *"an inference the operator
    /// cannot see still owes a report"* in its smallest form. Without it a
    /// note truncated mid-sentence reads as a note that ends mid-sentence.
    #[test]
    fn a_long_note_is_cut_and_says_so() {
        let long = "w".repeat(TOOLTIP_CHARS * 2);
        let tip = popup_tooltip(None, Some(&long));
        assert!(tip.ends_with('\u{2026}'), "{tip}");
        assert_eq!(tip.chars().count(), TOOLTIP_CHARS + 1, "{tip}");
    }

    /// ★★ **The tooltip does not panic on a multi-byte note.**
    ///
    /// The failure this guards is not cosmetic: slicing a `String` by byte
    /// index inside a character panics, and the panic would be *in the frame
    /// that is drawing the tooltip* — the worst available outcome, on a
    /// document whose only fault is being written in a script this project's
    /// tests do not otherwise use.
    #[test]
    fn a_multibyte_note_is_cut_safely() {
        let long = "\u{6f22}".repeat(TOOLTIP_CHARS * 2);
        let tip = popup_tooltip(None, Some(&long));
        assert_eq!(tip.chars().count(), TOOLTIP_CHARS + 1, "{tip}");
    }

    /// **An anonymous note shows its words and claims no author.**
    ///
    /// `/T` is legitimately absent — it means *anonymous*, never *unknown* —
    /// so a placeholder byline would turn a correct fact about the file into a
    /// claim about a person. `crate::text::panels::comments::comment_row_byline`
    /// makes the identical ruling and this is it holding on the second
    /// surface.
    #[test]
    fn an_anonymous_note_gets_no_byline() {
        let tip = popup_tooltip(None, Some("words"));
        assert_eq!(tip, "words");
        // …and whitespace counts as absent, exactly as it does for the panel's
        // byline: `/T ( )` is a byline nobody wrote.
        assert_eq!(popup_tooltip(Some("  "), Some("words")), "words");
    }

    /// **A note with no words still gets a tooltip**, saying so.
    ///
    /// The hover must answer *something* — a comment icon that produces no
    /// tooltip is indistinguishable from one the hover missed, which is the
    /// exact ambiguity this feature exists to remove.
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
    ///
    /// Asserting both directions, because a heading function that returned the
    /// ce-dimension wording for everything would pass a one-sided check and
    /// would relabel every `/Line` markup an operator drew.
    #[test]
    fn only_a_ce_dimension_heading_says_ce_dimension() {
        let ce = popup_ce_dimension_heading("Line");
        assert!(ce.contains("ce dimension"), "{ce}");
        assert!(ce.contains("Line"), "{ce}");
        let plain = popup_heading("Line");
        assert!(!plain.contains("dimension"), "{plain}");
        assert!(plain.contains("Line"), "{plain}");
    }
}
