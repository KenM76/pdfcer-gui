//! # `text::clipboard` — the four sentences the object clipboard can say
//!
//! Each is a **refusal on the status row**, and each exists because the
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
