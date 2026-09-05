//! # `text::clipboard` — what the clipboard verbs say on the status row
//!
//! ★ Two families live here as of 2026-09-04, and they are kept in one file
//! because they are one operator-facing subject — *what happened to my copy* —
//! and splitting them would invite two wordings of the same idea:
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
//! Each refusal exists because the
//! alternative is a keystroke that does nothing and says nothing. That is how
//! the operator experienced the absence of cut, copy and paste in the first
//! place — *"the standard copy/paste … aren't implemented"* — and a build that
//! implemented them and stayed silent when it could not act would read
//! identically.
//!
//! ## ★ Why two of the four name the ENGINE and two name the selection
//!
//! Because they are different kinds of "no" and an operator's next move differs:
//!
//! - *nothing selected* / *nothing copied* — **select something, or copy
//!   something.** The operator's own next act fixes it.
//! - *a path is selected* — **nothing the operator can do fixes it today.**
//!   `EditSession` has no verb that puts page content back on a page, so there
//!   is no sequence of clicks that would make the copy work.
//!
//! The second kind has to say so, or the operator spends the afternoon trying
//! different objects. `NO_SURFACE.md` §1c's rule about dated citations applies
//! to the sentence as much as to the code comment: it says *what pdfcer cannot
//! do*, not *that something went wrong*.

use crate::canvas::clipboard::Refusal;
// ★ Aliased. Two different refusals share the word in this crate — the object
// clipboard's, imported above, and the vector copy-out's — and importing both
// under one name would be a compile error while importing the second
// unqualified would make `refusal` and `copy_out_refusal` look like two arms of
// one function. The alias says which is which at every use site.
use crate::clipboard::place::Refusal as CopyOut;

/// The sentence for a refusal.
#[must_use]
/// ★ Returns an owned `String` since 2026-08-29, not `&'static str`.
///
/// One variant — [`Refusal::CutWouldNotSurvive`] — carries a **subtype**, and
/// its sentence is built around it. The alternative was a second function for
/// the one data-carrying case, which would put two clipboard refusals on two
/// surfaces and invite them to word the same idea differently.
///
/// Every other arm is still a literal, so `check-ui-strings` still sees them
/// all in this file.
pub fn refusal(reason: Refusal) -> String {
    match reason {
        Refusal::NothingSelected => {
            "Nothing is selected. Click something on the page first.".to_owned()
        }
        // ★★★ **The cut's delete half would be refused, so its copy half did
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
        // ★ It does not say *"nothing was copied"*, though nothing was: the
        // operator pressed cut, and what they need is the reason the document
        // will not allow it, not an inventory of what did not happen.
        Refusal::DeleteRefused(why) => why.line().to_owned(),
        // ★ Owned, so this function returns `String` rather than
        // `&'static str` -- the subtype is data and the sentence is built
        // around it. See `cut_would_not_survive`.
        Refusal::CutWouldNotSurvive(subtype) => cut_would_not_survive(subtype),
        // ★★★ THIS SENTENCE WAS RETIRED ON 2026-08-20, AND IT WAS THE
        // OPERATOR'S OLDEST OPEN REQUEST.
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
        // ★ Kept in the comment rather than deleted with the string, because
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
        Refusal::Unreadable => {
            "That annotation is not one pdfcer authors — a link, a form field or an attachment — \
             so there is nothing for it to copy."
                .to_owned()
        }
        Refusal::NothingCopied => {
            "Nothing has been copied yet. Select something on the page and press Ctrl+C first."
                .to_owned()
        }
    }
}

/// **Why a cut was refused: the thing selected cannot survive the round trip.**
///
/// `pdfcer-core`'s `CutWouldNotSurvive { subtype }`, in the operator's terms.
///
/// # ★★★ Why this exists even though the button is greyed
///
/// Because **a chord is not a button.** `Ctrl+X` is dispatched through the
/// keymap without consulting command enablement, so it reaches the handler
/// whatever the ribbon is showing. Greying the control removes the *invitation*;
/// this removes the *silence*.
///
/// ⇒ And it carries what greying cannot: **which** thing. A greyed button has
/// one static tooltip and the operator may have several things selected.
///
/// # Why each subtype earns its own sentence
///
/// A generic *"that cannot be cut"* is true and useless: the operator's next
/// move differs completely between the three, and only one of them is a
/// limitation at all.
///
/// ★ The `Redact` case is the one that will actually happen, and it is not an
/// apology. Refusing to put a redaction mark on the clipboard is pdfcer
/// protecting them from arming a destructive operation somewhere they did not
/// review — so the sentence says what it is *for*, and offers Delete, which is
/// almost certainly what they wanted.
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
        // ★ A named catch-all, not a guess. The engine may refuse a subtype
        // this shell has never seen -- a ce dimension whose sidecar record is
        // missing is the documented fourth case -- and the honest answer names
        // what it was rather than inventing a reason for it.
        other => format!(
            "That {other} cannot be cut: pdfcer could not put it back afterwards. Copy it \
             instead, or press Delete to remove it."
        ),
    }
}

/// What a content copy leaves on the **operating system's** clipboard.
///
/// ★★ It exists because of a toolkit constraint rather than a design wish:
/// `egui-winit` synthesises `Event::Paste` only when the OS clipboard holds
/// non-empty text, and swallows the `Ctrl+V` keystroke entirely otherwise — so
/// without something here, whether paste works depends on what the operator
/// last copied in another application. `canvas::clipboard::copy_content`
/// carries the full account.
///
/// # The wording
///
/// It is for a human who pastes into a text editor and wonders what they got,
/// so it says **what was copied and by what**, and does not pretend to be the
/// data. Naming pdfcer matters more than usual here: the paste may land in an
/// email, days later, with no other context.
///
/// Singular and plural are spelled out rather than `{n} object(s)`, because a
/// parenthesised plural is the tell of a program that could not be bothered —
/// and this string's whole job is to be read by somebody who did not expect it.
#[must_use]
pub fn os_marker(count: usize) -> String {
    if count == 1 {
        "1 object copied from pdfcer. Paste it back into pdfcer to place it.".to_owned()
    } else {
        format!("{count} objects copied from pdfcer. Paste them back into pdfcer to place them.")
    }
}

/// **What a vector copy-out put on the clipboard**, said on the status row.
///
/// ★★★ It names the OPERAND, and that is the whole reason this is a sentence
/// rather than a silence. `edit.copy_as_vector` copies the selection when there
/// is one and the whole page when there is not, and those two outcomes look
/// identical from the button — the operator finds out which they got when they
/// paste, in another application, possibly minutes later. A copy that quietly
/// took the sheet when three parts were selected is exactly the kind of thing
/// `DEFECTS.md` D4a calls *a sentence describing a different world than the one
/// on screen*, one step removed.
///
/// ★★ It does **not** list the clipboard format names. `image/svg+xml`,
/// `CF_ENHMETAFILE`, `CF_DIBV5` are wire identifiers — they belong in the trace,
/// where a developer looks, and `crate::clipboard::ClipFormat::name` is where
/// they live. What an operator can act on is *how many ways the receiving
/// program may read it*, and above all the promise that at least one of them is
/// vector, which is what the count and the second clause carry between them.
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
///
/// ★★★ The [`CopyOut::WouldDegrade`] arm is the one this whole feature is built
/// around, and it is the reason a refusal is better than a success here. Placing
/// only the raster formats would produce a paste that **works**: Word accepts it,
/// it looks right at 100%, and it is a flat picture that cannot be scaled,
/// recoloured or taken apart. The operator would discover that days later and
/// report it as *"pdfcer's copy doesn't paste as vectors"* — indistinguishable
/// from the feature not existing, except that it cost them the time to find out.
///
/// ⇒ So the sentence says the vector form could not be made **and** that nothing
/// was copied, in that order: the cause first, because the operator's next move
/// (try a different page, or export to SVG and place the file) depends on it.
///
/// ★ Every arm ends by saying what is still on the clipboard. `native-clipboard`
/// stages every handle before it opens the clipboard, so all of these except the
/// partial-placement case leave the previous contents intact — which is a real
/// reassurance and not a platitude, because the operator may have had something
/// there that took work to produce.
#[must_use]
pub fn copy_out_refusal(reason: &CopyOut) -> String {
    match reason {
        CopyOut::NoPage => "There is no page to copy. Open a document first \u{2014} whatever was \
             on the clipboard is still there."
            .to_owned(),
        // ★ The engine's own message is carried rather than paraphrased, for
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
///
/// ★★ Split out so the four Win32 outcomes get four different next moves rather
/// than one shrug. *Another program is holding it* is transient and the answer is
/// to press the button again; a **partial** placement is the one case where the
/// clipboard has genuinely changed, and the operator needs to know that what is
/// there now is neither the old contents nor the whole copy.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **Every refusal says what to do next, or why there is nothing to do.**
    ///
    /// Asserted as a length floor rather than by matching words, because the
    /// property is *"this is a sentence, not a label"*. A four-word refusal is
    /// the failure this whole module exists to prevent, and it is the shape a
    /// future edit would most plausibly introduce while "tidying".
    #[test]
    fn every_refusal_is_a_sentence() {
        for reason in [
            Refusal::NothingSelected,
            Refusal::EngineRefused,
            Refusal::Unreadable,
            Refusal::NothingCopied,
        ] {
            let s = refusal(reason);
            assert!(
                s.len() > 40,
                "{reason:?} is too short to be an explanation: {s:?}"
            );
            assert!(s.ends_with('.'), "{reason:?} must be a sentence: {s:?}");
        }
    }
}
