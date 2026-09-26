//! # `text::panels::face` — every string the face chooser shows
//!
//! One control, two surfaces, one catalog. The Properties panel's *This text*
//! section and the ribbon's Format ▸ Font group draw the **same** face chooser
//! through `pdfcer_gui::panels::properties::face`, so its wording lives in its own
//! module rather than inside [`super::properties`]: the face chooser is the
//! largest single subject either surface has, and it owes a disclosure neither
//! of the others does.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/panels/face.md`.

/// Label for the face chooser, in the Properties panel.
#[must_use]
pub const fn text_face_label() -> &'static str {
    "Font"
}

/// Shown in the face chooser when **nothing at all** can be offered for this
/// run.
#[must_use]
pub const fn text_face_none() -> &'static str {
    "No other font can show these characters — not the ones on this page, and not the standard \
     fourteen."
}

/// Hover for a face whose `/BaseFont` is shared by a second resource.
#[must_use]
pub const fn text_face_ambiguous() -> &'static str {
    "This page carries two fonts with this name — two subsets of one face. Choosing this \
     row uses this one."
}

/// The heading over the rows the **page already carries**.
#[must_use]
pub const fn face_group_on_page() -> &'static str {
    "On this page"
}

/// The heading over the rows pdfcer would **add to the document**.
#[must_use]
pub const fn face_group_addable() -> &'static str {
    "pdfcer can add these"
}

/// **The disclosure this feature owes**, said once, where the choice is made.
#[must_use]
pub const fn face_addable_disclosure() -> &'static str {
    "Choosing one of these adds it to the document. pdfcer writes the face's name and its letter \
     widths, not the font program, so the text is drawn with each reader's own copy of that \
     face. Every PDF reader carries these fourteen, so it will always show; on another machine \
     the letters may be set a little differently from what you see here."
}

// ===========================================================================
// O141 — the offer that turns a refused character into a face that has it
// ===========================================================================
//
// The operator: *"if the character isn't available in a pdf are we able to
// change to a different font?"*
//
// Yes. The engine refuses by name, the refusal carries the character, and this
// chooser offers the standard fourteen; these five strings are what connects
// the refusal to the chooser, and the surface is
// `pdfcer_gui::panels::properties::refusedchar`.
//
// They live in THIS module rather than in `crate::text::textedit` because
// the surface they belong to *is* a face chooser: it draws the same two-group
// popup through `panels::properties::face::popup_body` and owes the same
// disclosure. Splitting the offer's words from the chooser's words is how two
// wordings of one act grow up beside each other, which is the divergence this
// module's own header exists to record.

/// The heading over the offer block.
#[must_use]
pub const fn refused_char_heading() -> &'static str {
    "A character this font cannot type"
}

/// **The sentence that names the character** — the half `Declined::line`
/// structurally cannot say.
#[must_use]
pub fn refused_char_named(character: char, font: &str) -> String {
    format!(
        "The “{character}” is not one of the letters {font} carries. Fonts inside a PDF usually \
         hold only the letters your page already prints, and pdfcer cannot add one to a font that \
         is already in the file."
    )
}

/// The instruction under [`refused_char_named`], and the label on the chooser.
#[must_use]
pub fn refused_char_offer(character: char) -> String {
    format!("Pick a font that has the “{character}” and pdfcer will put your change in with it:")
}

/// **No face pdfcer can author will take this character either** — the honest
/// dead end, named rather than drawn as an empty list.
#[must_use]
pub fn refused_char_no_face(character: char) -> String {
    format!(
        "None of the fonts pdfcer can add to this page can write “{character}” either — they \
         cover the Western European alphabets only. Your document has not been changed. A font \
         that has this character must already be in the file, or come from a program that can \
         embed one."
    )
}

/// **What the block says on the frame the face swap lands** — the second
/// half of the route, now carried out rather than described.
#[must_use]
pub fn refused_char_swapped(character: char, font: &str) -> String {
    format!("This text is now set in {font}, and pdfcer is putting your “{character}” in with it.")
}

/// **The one state that asks the operator to type it again**, and the only
/// one in this module that does.
#[must_use]
pub fn refused_char_swapped_type_again(character: char, font: &str) -> String {
    format!(
        "This text is now set in {font}. Click in it and type the \u{201c}{character}\u{201d} \
         again, and it will go in."
    )
}

/// **What the block says when the swap landed and the character still would not
/// go in** — the third state, and the one that names **no cause at all**.
#[must_use]
pub fn refused_char_blocked(character: char, font: &str) -> String {
    format!(
        "This text is now set in {font} — that part worked and is in your document. The \
         “{character}” still would not go in, and pdfcer has not been told why. Nothing was \
         damaged: the text is exactly as it was, in the new face. Try typing the \
         “{character}” again, and if it is still refused the reason will be named in the \
         status bar."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The disclosure carries all three facts it exists to carry.**
    #[test]
    fn the_disclosure_states_the_act_the_omission_and_the_consequence() {
        let line = face_addable_disclosure();
        assert!(line.contains("adds it to the document"), "{line}");
        assert!(line.contains("not the font program"), "{line}");
        assert!(line.contains("reader's own copy"), "{line}");
    }

    /// **The two group headings are not paraphrases of each other.**
    #[test]
    fn the_two_group_headings_say_different_things() {
        assert_ne!(face_group_on_page(), face_group_addable());
        // The addable heading must read as an act pdfcer performs. "Standard
        // fonts" would pass the inequality above and fail the operator.
        assert!(
            face_group_addable().contains("add"),
            "the addable heading must name the act: {}",
            face_group_addable()
        );
    }

    /// **The empty-list sentence must account for BOTH sources** — the page's
    /// own fonts and the standard fourteen.
    #[test]
    fn the_empty_sentence_accounts_for_the_standard_fourteen() {
        let line = text_face_none();
        assert!(line.contains("this page"), "{line}");
        assert!(line.contains("fourteen"), "{line}");
    }

    /// **The offer names the character, every time, in every sentence that
    /// mentions it** — `OPERATOR_REQUESTS.md` O141.
    #[test]
    fn every_sentence_in_the_offer_names_the_character_itself() {
        for line in [
            refused_char_named('€', "Arimo-Bold"),
            refused_char_offer('€'),
            refused_char_no_face('€'),
            refused_char_swapped('€', "Helvetica-Bold"),
        ] {
            assert!(
                line.contains('€'),
                "the character is the one fact the status bar cannot carry: {line}"
            );
            assert!(
                !line.contains("20ac") && !line.contains("20AC"),
                "a code point is not what the operator typed: {line}"
            );
        }
    }

    /// **Both font-naming sentences name the font**, and they name two
    /// different ones.
    #[test]
    fn the_offer_names_the_font_that_refused_and_the_font_that_replaced_it() {
        let refused = refused_char_named('q', "AAAAAA+Arimo-Bold");
        assert!(refused.contains("AAAAAA+Arimo-Bold"), "{refused}");
        let swapped = refused_char_swapped('q', "Helvetica-Bold");
        assert!(swapped.contains("Helvetica-Bold"), "{swapped}");
        assert!(
            !swapped.contains("cannot"),
            "the follow-up reports a success and must not read like a second refusal: {swapped}"
        );
    }

    /// **The offer promises that pdfcer will finish the job, and neither
    /// sentence tells the operator to retype anything.**
    #[test]
    fn the_offer_promises_pdfcer_finishes_the_job_and_never_asks_for_a_retype() {
        let offer = refused_char_offer('%');
        assert!(
            offer.contains("pdfcer will put your change in"),
            "the operator must know his edit is coming with the swap, or the font list \
             reads as a route that ends in a list: {offer}"
        );
        let swapped = refused_char_swapped('%', "Courier");
        assert!(
            swapped.contains("putting your “%” in"),
            "the follow-up must report what is happening to his edit: {swapped}"
        );
        for line in [offer, swapped] {
            assert!(
                !line.contains("again"),
                "asking for a retype after the retype has been made produces a second \
                 copy of the character: {line}"
            );
        }
    }

    /// **The third state names the face that IS in force and no cause at all.**
    #[test]
    fn the_blocked_sentence_names_no_cause_it_cannot_see() {
        let line = refused_char_blocked('%', "Courier");
        assert!(
            line.contains("Courier"),
            "the face that IS in force, because it really is in force: {line}"
        );
        assert!(
            line.contains("not been told why"),
            "the block cannot see the refusal — it infers one from the edit epoch not \
             moving — so the sentence must say so rather than pick a cause: {line}"
        );
        assert!(
            !line.contains("saved and opened again") && !line.contains("Save this document"),
            "★ save-and-reopen was the remedy for the base-revision font resolution the \
             engine fixed in v0.41.0 (Pass 257.0). Prescribing it now costs the operator \
             two gestures to learn it does nothing: {line}"
        );
        assert!(
            !line.contains("cannot type into a font it has just added"),
            "★★ and it must not name that cause either. It was measured, it was true, \
             and it stopped being true — which is why this assertion is here rather \
             than a comment: {line}"
        );
        assert!(
            line.contains("Nothing was damaged"),
            "the swap DID reach his document, so an operator reading a failure notice \
             must be told what survived or he will go looking for damage: {line}"
        );
    }

    /// **The dead end says the document survived it, and says what the
    /// operator can still do.**
    #[test]
    fn the_dead_end_says_the_document_is_unchanged_and_what_is_still_possible() {
        let line = refused_char_no_face('€');
        assert!(
            line.contains("has not been changed"),
            "a dead end that does not say the document survived it reads as a failure the \
             operator must go and verify: {line}"
        );
        assert!(
            line.contains("already be in the file") || line.contains("embed"),
            "it must name what is still possible, or it is only a complaint: {line}"
        );
        // ⚠ It must NOT keep the old caveat's hedge. The list is exact now, and
        // a sentence implying otherwise teaches the operator to distrust it.
        assert!(
            !line.contains("has not checked"),
            "the untested caveat's wording must not survive its subject: {line}"
        );
    }
}
