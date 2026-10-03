//! # `text::clipboard` — what the clipboard verbs say on the status row
//!
//!
//! 1. **The object clipboard's four refusals** ([`refusal`]), which are about
//!    copying *within* pdfcer. These are the original contents of this file and
//!    the paragraphs below are about them.
//! 2. **The vector copy-out's disclosure and refusals**
//!    ([`copied_as_vector`], [`copy_out_refusal`]), which are about copying
//!    *out* of pdfcer — `OPERATOR_REQUESTS.md` O120. The copy-out is the one
//!    clipboard verb that says something on **success** as well, because it
//!    alone has two possible operands and the button cannot show which was
//!    taken.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/clipboard.md`.

use crate::refusals::clipboard::Refusal;
// Aliased. Two different refusals share the word in this crate — the object
// clipboard's, imported above, and the vector copy-out's — and importing both
// under one name would be a compile error while importing the second
// unqualified would make `refusal` and `copy_out_refusal` look like two arms of
// one function. The alias says which is which at every use site.
use crate::clipboard::place::Refusal as CopyOut;

/// The sentence for a refusal.
#[must_use]
/// Returns an owned `String` since 2026-08-29, not `&'static str`.
pub fn refusal(reason: Refusal) -> String {
    match reason {
        Refusal::NothingSelected => {
            "Nothing is selected. Click something on the page first.".to_owned()
        }
        // **The cut's delete half would be refused, so its copy half did
        // not run either** — and the sentence comes from `annotdelete`'s
        // catalog rather than being written again here.
        //
        // One fact, one wording, four surfaces: the Format tab withholds its
        // Delete on it, the canvas menu withholds its own, the Properties panel
        // draws this sentence beside the selection, and now `Ctrl+X` puts it on
        // the status row. A second phrasing here would be the divergence
        // `UnembedBlocker::reason` delegating to `Removability::reason` exists
        // to prevent.
        //
        // It does not say *"nothing was copied"*, though nothing was: the
        // operator pressed cut, and what they need is the reason the document
        // will not allow it, not an inventory of what did not happen.
        Refusal::DeleteRefused(why) => why.line().to_owned(),
        // Owned, so this function returns `String` rather than
        // `&'static str` -- the subtype is data and the sentence is built
        // around it. See `cut_would_not_survive`.
        Refusal::CutWouldNotSurvive(subtype) => cut_would_not_survive(subtype),
        //
        // It read: *"That is page content — a line, a shape or a piece of text.
        // pdfcer can copy comments and markup, but it cannot yet put page
        // content back onto a page, so copying one would offer a paste that
        // could never happen."*
        //
        // Every word of that was true and carefully chosen — it named the
        // boundary and did not apologise for it, because *"an operator who
        // reads 'pdfcer cannot copy page content yet' stops trying; one who
        // reads 'copy failed' tries four more shapes."* `Pass 120.0` shipped
        // the object clipboard and made it false.
        //
        // Kept in the comment rather than deleted with the string, because
        // this is the **third** refusal in two days to expire the week it was
        // written — after `NotAPath` and `ManyObjects` on the resize. The
        // pattern is worth naming: **a refusal is a claim with a date on it**,
        // and the ones that age worst are the carefully-argued ones, because
        // the care makes them read as permanent.
        //
        // What replaces it is a genuinely different fact, and it is the
        // engine's rather than ours: a clip it could not assemble. Kept
        // deliberately general — it does not guess which of the engine's
        // reasons applied, because the engine words each of them and
        // `vector_edit` carries that sentence to the same status row.
        Refusal::EngineRefused => {
            "pdfcer could not copy what is selected. Some things on a page are drawn in a way it \
             cannot lift off and put back, and it will not offer you a paste that would not \
             work."
                .to_owned()
        }
        //
        // Three subtypes, three genuinely different reasons, and the operator's
        // next move differs for each — which is why this is a `match` on the
        // subtype rather than one sentence about "some annotations".
        // `canvas::cutgate` already words the same three for the CUT half and
        // this reuses that catalog rather than writing a second phrasing of one
        // fact, which is the divergence `UnembedBlocker::reason` delegating to
        // `Removability::reason` exists to prevent.
        //
        // The list is joined rather than reported one at a time because a
        // future multi-annotation selection can refuse several at once, and
        // *"and 2 others"* is the shape that makes an operator go looking for
        // the other two.
        Refusal::CannotCarry(subtypes) => cannot_carry(&subtypes),
        //
        // It read: *"That annotation is not one pdfcer authors — a link, a form
        // field or an attachment — so there is nothing for it to copy."* Every
        // word was true of a clipboard that copied by re-authoring from a
        // `MarkupSpec`. A link copies now. An attachment copies. A sticky note
        // and a stamp copy with their baked appearances, because
        // `copy_selection` carries the dictionary rather than the model.
        //
        // What replaces it is the one job the variant has left, and it is a
        // different fact with a different next move: the selected annotation is
        // no longer on the page. See `Refusal::Unreadable`.
        Refusal::Unreadable => {
            "pdfcer cannot find that comment on the page any more — it may have been changed or \
             removed since you selected it. Click it again."
                .to_owned()
        }
        Refusal::NothingCopied => {
            "Nothing has been copied yet. Select something on the page and press Ctrl+C first."
                .to_owned()
        }
    }
}

/// **Why a cut was refused: the thing selected cannot survive the round trip.**
#[must_use]
pub fn cut_would_not_survive(subtype: &str) -> String {
    match subtype {
        "Redact" => "A redaction mark cannot be cut, because pasting one would arm a redaction \
             nobody had reviewed. Copy it if you want it elsewhere, or press Delete to remove it \
             from here."
            .to_owned(),
        "Widget" => "A form field cannot be cut this way \u{2014} it has its own clipboard. Click \
             the field and press Ctrl+X."
            .to_owned(),
        "Popup" => "A comment's pop-up window cannot be cut on its own. Cut the comment it \
             belongs to and the pop-up goes with it."
            .to_owned(),
        // A named catch-all, not a guess. The engine may refuse a subtype
        // this shell has never seen -- a ce dimension whose sidecar record is
        // missing is the documented fourth case -- and the honest answer names
        // what it was rather than inventing a reason for it.
        other => format!(
            "That {other} cannot be cut: pdfcer could not put it back afterwards. Copy it \
             instead, or press Delete to remove it."
        ),
    }
}

/// **Why a copy could not carry what was selected**, naming the subtypes.
fn cannot_carry(subtypes: &[String]) -> String {
    let mut clauses: Vec<String> = subtypes
        .iter()
        .map(|subtype| match subtype.as_str() {
            "Widget" => "a form field, which is copied from Edit mode so pdfcer can ask what to \
                         call it in the document you paste it into"
                .to_owned(),
            "Popup" => "a comment's pop-up window, which cannot be copied on its own — copy the \
                        comment it belongs to and the pop-up goes with it"
                .to_owned(),
            "Redact" => "a redaction mark, which pdfcer deliberately will not copy: pasting one \
                         would arm a redaction in a document nobody has reviewed"
                .to_owned(),
            other => format!("a {other}, which pdfcer could not put back afterwards"),
        })
        .collect();
    clauses.dedup();
    format!(
        "Nothing was copied. You selected {}.",
        clauses.join("; and ")
    )
}

/// **What a copy took, and what it did not** — the one sentence a partial copy
/// owes before the operator finds out by pasting.
#[must_use]
pub fn partial_copy(left_behind: &[String], thin: usize) -> String {
    let mut parts = Vec::new();
    if !left_behind.is_empty() {
        parts.push(
            cannot_carry(left_behind).replace("Nothing was copied. You selected", "It left behind"),
        );
    }
    if thin > 0 {
        let what = if thin == 1 {
            "One comment".to_owned()
        } else {
            format!("{thin} comments")
        };
        parts.push(format!(
            "{what} will paste without the author, the date, the note text or the see-through \
             setting — pdfcer rebuilds that kind of mark from its shape, and those are not part \
             of its shape."
        ));
    }
    format!("Copied, but not all of it. {}", parts.join(" "))
}

/// What a content copy leaves on the **operating system's** clipboard.
#[must_use]
pub fn os_marker(objects: usize, comments: usize) -> String {
    let what = match (objects, comments) {
        (0, 1) => "1 comment".to_owned(),
        (0, n) => format!("{n} comments"),
        (1, 0) => "1 object".to_owned(),
        (n, 0) => format!("{n} objects"),
        (1, 1) => "1 object and 1 comment".to_owned(),
        (1, c) => format!("1 object and {c} comments"),
        (o, 1) => format!("{o} objects and 1 comment"),
        (o, c) => format!("{o} objects and {c} comments"),
    };
    if objects + comments == 1 {
        format!("{what} copied from pdfcer. Paste it back into pdfcer to place it.")
    } else {
        format!("{what} copied from pdfcer. Paste them back into pdfcer to place them.")
    }
}

/// **What a vector copy-out put on the clipboard**, said on the status row.
#[must_use]
pub fn copied_as_vector(selection: bool, formats: usize) -> String {
    let what = if selection {
        "the selection"
    } else {
        "this page"
    };
    format!(
        "Copied {what} to the clipboard in {formats} formats, vectors first. Paste into Word, \
         PowerPoint or Inkscape and the line-work is still editable."
    )
}

/// Why a vector copy-out did not happen.
#[must_use]
pub fn copy_out_refusal(reason: &CopyOut) -> String {
    match reason {
        CopyOut::NoPage => "There is no page to copy. Open a document first \u{2014} whatever was \
             on the clipboard is still there."
            .to_owned(),
        // The engine's own message is carried rather than paraphrased, for
        // `text::export_image`'s reason: it names the numbers, and a shell
        // rewording is a second account of a failure only the engine saw.
        CopyOut::Render(why) => format!(
            "This could not be recorded as vectors: {why}. Nothing was put on the clipboard."
        ),
        CopyOut::WouldDegrade => {
            "The vector form could not be made, so nothing was copied. A picture-only copy \
             would paste into Word as a flat image that cannot be scaled or recoloured, which \
             is not what this command promises."
                .to_owned()
        }
        CopyOut::Clipboard(err) => clipboard_refusal(err),
    }
}

/// The sentence for the operating system's own refusal.
fn clipboard_refusal(err: &native_clipboard::PlaceError) -> String {
    match err {
        native_clipboard::PlaceError::Open => {
            "Another program is holding the clipboard. Try the copy again in a moment \
             \u{2014} nothing has changed on it yet."
                .to_owned()
        }
        native_clipboard::PlaceError::Set(_) => {
            "Windows refused part of the copy. The clipboard now holds only some of the \
             formats, so paste the result before relying on it, or copy again."
                .to_owned()
        }
        // Register, Stage, Nothing and Unsupported. None of them is expected on
        // a Windows build with a page to copy, so the sentence says what it can
        // honestly say — nothing was placed — rather than inventing a cause.
        _ => "Windows would not take this copy, so nothing was placed. Whatever was on the \
             clipboard is still there."
            .to_owned(),
    }
}

//
// A third family, and it is deliberately not folded into `refusal` above. The
// two are different kinds of "no" and the operator's next move differs, which
// is this file's own stated rule for splitting a refusal out:
//
// - `Refusal` is the **clipboard's** answer — *nothing is selected*, *nothing
//   was copied*, *this cut would not survive*. It is about the operand.
// - `ModeRefusal` is the **stance's** answer — *this mode does not do that*.
//   Nothing is wrong with the operand at all; the operator has told pdfcer
//   which interface they want and this verb is not in it. The remedy is a
//   two-inch move of the mode selector, and the sentence has to say so or the
//   press looks broken.
//
// ⇒ It reaches the operator through `app::status::decline`, which wears `⊗`
// and means *nothing happened* — NOT through `app::actions::record_note`,
// which draws under `⚑ About your last edit:` and would report a press where
// nothing happened as an edit. `app::status::decline::textedit`'s header
// carries the full argument for that distinction; this is its fourth
// application.

/// **Which clipboard verb the active mode refused, and what it was about.**
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModeRefusal {
    /// `edit.paste` with **page content** on the clipboard, in a mode that does
    /// not author page content.
    PasteContent,
    /// `edit.paste` with a **comment or markup** on the clipboard — or with
    /// nothing on it — in a mode that authors no markup.
    ///
    /// The empty clipboard lands here on purpose: in Read, *"nothing has been
    /// copied"* would be true and useless, because copying something would not
    /// help. `app::dispatch::clipboard`'s paste gate says the same thing in the
    /// same words at the point it makes the choice.
    PasteMarkup,
    /// `edit.paste` with a **form field** on the clipboard, in a mode that does
    /// not author page content. A field is part of the document rather than a
    /// comment on it.
    PasteField,
    /// `edit.paste` with **another program's picture** on the clipboard, in a
    /// mode that does not author page content.
    PastePicture,
    /// `edit.paste` with **another program's text** on the clipboard, in a mode
    /// that adds nothing to the page.
    PasteText,
    /// `edit.paste` with **a drawing copied from a PDF** on the clipboard, in a
    /// mode that authors no markup.
    PasteDrawing,
    /// A picture **dropped** on the page, in a mode that does not change page
    /// content.
    DropPicture,
    /// `edit.cut` over **page content**, in a mode that does not change it.
    CutContent,
    /// `edit.cut` over a **comment or markup**, in a mode that authors none.
    CutMarkup,
    /// `edit.cut` over a **form field**, in a mode that does not change page
    /// content.
    CutField,
    /// `edit.duplicate` over a **comment or markup**, in a mode that authors
    /// none. 2026-09-06.
    ///
    /// # Why this is a seventh variant and not [`Self::PasteMarkup`] reused
    ///
    /// Because that sentence says *"Switch to Review to **paste** this"*, and
    /// nothing was pasted. The operator pressed `Ctrl+D` over a comment they
    /// can see, with a clipboard that may hold something else entirely — a
    /// sentence about pasting sends them to inspect a clipboard that has
    /// nothing to do with what they just did, which is the *"describing a
    /// different world than the one on screen"* shape `DEFECTS.md` D4a names.
    ///
    /// The remedy is identical to [`Self::PasteMarkup`]'s and the sentence
    /// still has to be its own, which is this enum's founding argument turned
    /// round: the variants are distinguished by **what the operator did**, not
    /// by which mode fixes it. Two acts that share a remedy still owe two
    /// sentences, because the first half of each sentence is what tells the
    /// operator pdfcer understood the gesture.
    DuplicateMarkup,
}

impl ModeRefusal {
    /// The sentence for this refusal.
    #[must_use]
    pub fn line(self) -> &'static str {
        match self {
            Self::PasteContent => {
                "The clipboard holds page content — a line, a shape or a piece of text — and this mode does not change what is on the page. Switch to Edit to paste it."
            }
            Self::PasteMarkup => {
                "This mode does not add comments or markup to a document. Switch to Review to paste this, or to Edit."
            }
            Self::PasteField => {
                "The clipboard holds a form field, which is part of the document rather than a comment on it, and this mode does not change what is on the page. Switch to Edit to paste it."
            }
            Self::PastePicture => {
                "The clipboard holds a picture from another program, and this mode adds nothing to the page. Switch to Edit to paste it as page content, or to Review to paste it as a stamp."
            }
            Self::PasteText => {
                "The clipboard holds text from another program, and this mode adds nothing to the page. Switch to Edit to paste it as page text, or to Review to paste it as a comment."
            }
            Self::PasteDrawing => {
                "The clipboard holds a drawing copied from a PDF, and this mode adds nothing to the page. Switch to Review or Edit to paste it as a stamp."
            }
            Self::DropPicture => {
                "A dropped picture goes onto the page, and this mode does not change what is on the page. Switch to Edit and drop it again."
            }
            Self::CutContent => {
                "That is page content, and this mode does not change what is on the page. Nothing has been removed — switch to Edit to cut it."
            }
            Self::CutMarkup => {
                "This mode does not remove comments or markup. Nothing has been removed — switch to Review to cut this, or to Edit."
            }
            Self::CutField => {
                "That is a form field, which is part of the document rather than a comment on it, and this mode does not change what is on the page. Nothing has been removed — switch to Edit to cut it."
            }
            // *"Nothing has been added"*, on the three cut sentences' own
            // rule inverted. A duplicate that is refused after the operator has
            // watched a selected comment sit there is a case where they may
            // reasonably wonder whether a second one landed off-screen, and
            // rule 4 puts the disclosure where the doubt is.
            Self::DuplicateMarkup => {
                "This mode does not add comments or markup to a document. Nothing has been added — switch to Review to duplicate this, or to Edit."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every refusal says what to do next, or why there is nothing to do.**
    #[test]
    fn every_refusal_is_a_sentence() {
        //
        // `.clone()` because `Refusal` stopped being `Copy` when that
        // variant arrived carrying the subtypes. Cloning in a test costs
        // nothing and is preferable to taking `&Refusal` in the signature —
        // every production call site owns its refusal and moves it, and a
        // by-reference signature would make each of them borrow something they
        // are about to drop.
        for reason in [
            Refusal::NothingSelected,
            Refusal::EngineRefused,
            Refusal::Unreadable,
            Refusal::NothingCopied,
            Refusal::CannotCarry(vec!["Redact".to_owned()]),
            Refusal::CannotCarry(vec!["Widget".to_owned(), "Popup".to_owned()]),
            // A subtype this shell has never seen — the catch-all must still
            // produce a sentence rather than a fragment naming nothing.
            Refusal::CannotCarry(vec!["Movie".to_owned()]),
        ] {
            let s = refusal(reason.clone());
            assert!(
                s.len() > 40,
                "{reason:?} is too short to be an explanation: {s:?}"
            );
            assert!(s.ends_with('.'), "{reason:?} must be a sentence: {s:?}");
        }
    }

    /// **A partial copy's sentence says BOTH what was left behind and what
    /// arrived thin**, and never claims the copy failed.
    #[test]
    fn a_partial_copy_says_what_arrived_and_what_did_not() {
        let both = partial_copy(&["Redact".to_owned()], 2);
        assert!(
            both.starts_with("Copied, but not all of it."),
            "★ it must say the copy HAPPENED first — an operator who reads a failure retries a \
             gesture that worked: {both:?}"
        );
        assert!(
            both.contains("redaction mark"),
            "the refused subtype must be named: {both:?}"
        );
        assert!(
            both.contains("2 comments"),
            "and so must the count that will paste thin: {both:?}"
        );
        let thin_only = partial_copy(&[], 1);
        assert!(
            thin_only.contains("One comment") && !thin_only.contains("left behind"),
            "★ with nothing refused it must not invent a second clause: {thin_only:?}"
        );
    }

    /// The OS marker names comments as comments, and a mixed copy as both.
    #[test]
    fn the_os_marker_counts_objects_and_comments_separately() {
        assert!(os_marker(0, 1).starts_with("1 comment copied"));
        assert!(os_marker(1, 0).starts_with("1 object copied"));
        assert!(os_marker(2, 1).starts_with("2 objects and 1 comment copied"));
        assert!(os_marker(1, 3).starts_with("1 object and 3 comments copied"));
        // The trailing pronoun follows the TOTAL, not either count: "Paste
        // it back" over two things is the tell of a template nobody read.
        assert!(os_marker(0, 1).contains("Paste it back"));
        assert!(os_marker(1, 1).contains("Paste them back"));
        // The substring `clipboard_text`'s driven check greps for must survive
        // every one of these branches, or that check goes permanently green.
        for (o, c) in [(0, 1), (1, 0), (3, 0), (0, 4), (2, 2)] {
            assert!(
                os_marker(o, c).contains("copied from pdfcer"),
                "★ ui-verify's `ctrl_c_copies_text_to_the_os_clipboard` greps for this exact \
                 substring to tell an object copy from a text copy; a branch without it makes \
                 that check unable to detect the defect it exists for"
            );
        }
    }

    /// **Every mode refusal names a mode the selector actually offers.**
    #[test]
    fn every_mode_refusal_names_a_mode_the_selector_actually_offers() {
        use crate::text::ribbon;
        let edit = ribbon::mode_edit();
        let review = ribbon::mode_review();
        for (why, must_name) in [
            (ModeRefusal::PasteContent, edit),
            (ModeRefusal::PasteMarkup, review),
            (ModeRefusal::PasteField, edit),
            (ModeRefusal::PastePicture, edit),
            (ModeRefusal::PastePicture, review),
            (ModeRefusal::PasteText, edit),
            (ModeRefusal::PasteText, review),
            (ModeRefusal::PasteDrawing, edit),
            (ModeRefusal::PasteDrawing, review),
            (ModeRefusal::DropPicture, edit),
            (ModeRefusal::CutContent, edit),
            (ModeRefusal::CutMarkup, review),
            (ModeRefusal::CutField, edit),
            (ModeRefusal::DuplicateMarkup, review),
        ] {
            let line = why.line();
            assert!(
                line.contains(must_name),
                "{why:?} must send the operator to `{must_name}`: {line}"
            );
        }
    }

    /// **Every refusal is a distinct sentence**, and the three that
    /// follow a gesture over a visible operand say the document is unchanged.
    #[test]
    fn the_mode_refusals_are_distinct_sentences_and_the_gestures_reassure() {
        let all = [
            ModeRefusal::PasteContent,
            ModeRefusal::PasteMarkup,
            ModeRefusal::PasteField,
            ModeRefusal::PastePicture,
            ModeRefusal::PasteText,
            ModeRefusal::PasteDrawing,
            ModeRefusal::DropPicture,
            ModeRefusal::CutContent,
            ModeRefusal::CutMarkup,
            ModeRefusal::CutField,
            ModeRefusal::DuplicateMarkup,
        ];
        let mut lines: Vec<&str> = all.iter().map(|w| w.line()).collect();
        lines.sort_unstable();
        let distinct = lines.len();
        lines.dedup();
        assert_eq!(distinct, lines.len(), "two refusals share a sentence");

        for why in [
            ModeRefusal::CutContent,
            ModeRefusal::CutMarkup,
            ModeRefusal::CutField,
        ] {
            assert!(
                why.line().contains("Nothing has been removed"),
                "a refused cut must say the document is unchanged: {}",
                why.line()
            );
        }

        assert!(
            ModeRefusal::DuplicateMarkup
                .line()
                .contains("Nothing has been added"),
            "a refused duplicate must say no second copy landed: {}",
            ModeRefusal::DuplicateMarkup.line()
        );
    }
}
