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
        // ★★★ **THE ENGINE REFUSES THAT KIND, AND THE SENTENCE NAMES IT** —
        // 2026-09-05.
        //
        // Three subtypes, three genuinely different reasons, and the operator's
        // next move differs for each — which is why this is a `match` on the
        // subtype rather than one sentence about "some annotations".
        // `canvas::cutgate` already words the same three for the CUT half and
        // this reuses that catalog rather than writing a second phrasing of one
        // fact, which is the divergence `UnembedBlocker::reason` delegating to
        // `Removability::reason` exists to prevent.
        //
        // ★ The list is joined rather than reported one at a time because a
        // future multi-annotation selection can refuse several at once, and
        // *"and 2 others"* is the shape that makes an operator go looking for
        // the other two.
        Refusal::CannotCarry(subtypes) => cannot_carry(&subtypes),
        // ★★★ THIS SENTENCE WAS RETIRED ON 2026-09-05, and it was reporting a
        // LIMIT AS AN ABSENCE — the pattern `RESUME.md` records as this
        // project's most expensive.
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

/// **Why a copy could not carry what was selected**, naming the subtypes.
///
/// # ★★ Why each subtype earns its own clause rather than one general refusal
///
/// Because the operator's next move differs, and in one case the refusal is
/// protecting them rather than admitting a limit:
///
/// | subtype | what they should do instead |
/// |---|---|
/// | `/Widget` | select it in Edit mode — a form field has its own copy, which asks the naming question a blind copy would have to guess at |
/// | `/Popup` | copy the comment it belongs to; the pop-up travels with it |
/// | `/Redact` | nothing — and that is the point. A redaction mark is a **pending destructive operation**, and pasting one arms a redaction in a document nobody reviewed |
///
/// ★ The `/Redact` line is the one that must not be softened into *"pdfcer
/// cannot copy this"*. It can; it declines to, and an operator who reads a
/// capability limit will go looking for a workaround for something that is a
/// safeguard.
///
/// ★★ The catch-all is **named, not guessed**. `pdfcer-core` may refuse a
/// subtype this shell has never seen — a ce dimension whose sidecar record is
/// missing is the documented fourth case — and the honest answer says which it
/// was rather than inventing a reason for it. Same posture as
/// [`cut_would_not_survive`] one function up, and the same reason.
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
///
/// Rule 4, *"fuzzy never sneaky"*: a copy that quietly took three of four
/// selected things, or took a comment without its author and its opacity, looks
/// exactly like one that took everything. Nothing errors and nothing is marked,
/// which is the definition of sneaky.
///
/// # ★★ The two halves are different kinds of loss and are said differently
///
/// * **`left_behind`** — annotations that will not be on the clipboard at all.
///   The operator will notice, eventually, and this is what stops it being a
///   mystery. Worded by [`cannot_carry`], reused rather than re-phrased.
/// * **`thin`** — annotations that *will* paste, and will paste **without their
///   author, date, note text and opacity**. This is the one nobody would ever
///   report: the mark is on the page, it looks right, and what is missing lives
///   in a pop-up this shell does not draw. It is `pdfcer-core`'s limit, not
///   this shell's — `paste_clip_annotations` plants a modelled markup with
///   `add_markup` rather than `add_markup_with` — and the sentence says so
///   plainly, because an operator who believes it is their mistake will retry.
///
/// # ★ Why it is not reachable today, said rather than implied
///
/// A lone modelled markup takes the spec-plus-options route, which carries all
/// four keys, so `thin` is zero for every clip this shell parks; and the three
/// refused subtypes are either routed elsewhere or refuse the whole copy, so
/// `left_behind` is empty. Both become live the day the selection model can
/// hold more than one annotation. The sentence is written now because the
/// alternative is writing it *after* the first silent partial copy.
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
///
/// ★★ It exists because of a toolkit constraint rather than a design wish:
/// `egui-winit` synthesises `Event::Paste` only when the OS clipboard holds
/// non-empty text, and swallows the `Ctrl+V` keystroke entirely otherwise — so
/// without something here, whether paste works depends on what the operator
/// last copied in another application. `canvas::clipboard::copy`
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
///
/// # ★★ It took TWO counts as of 2026-09-05, and the second is not cosmetic
///
/// The clipboard can now carry annotations, and *"1 object copied from
/// pdfcer"* pasted into an email about a revision cloud is a sentence about
/// the wrong thing. The operator's own word for these is **comment** — it is
/// what the panel is called — so that is the word here, rather than the file
/// format's *annotation* or the engine's *markup*.
///
/// ★ The mixed line reads *"2 objects and 1 comment"* rather than *"3
/// items"*, because the two halves came from two different selections in the
/// operator's mind and a total tells them nothing about whether the copy took
/// what they meant.
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
        // ★ `CannotCarry` joined the list on 2026-09-05, and it is the one
        // that most needed adding: it is built at runtime from the engine's
        // payload rather than being a literal in this file, so it is the only
        // arm `check-ui-strings` cannot see and the only one that could become
        // a fragment without anybody noticing.
        //
        // ★★ `.clone()` because `Refusal` stopped being `Copy` when that
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

    /// ★★ **A partial copy's sentence says BOTH what was left behind and what
    /// arrived thin**, and never claims the copy failed.
    ///
    /// The wording trap here is real and one-directional: *"could not be
    /// copied"* over a copy that mostly worked sends the operator back to
    /// press `Ctrl+C` again, which changes nothing and costs them the
    /// afternoon. It has to open by saying the copy happened.
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
    ///
    /// ★ `os_marker(0, 1)` was the case that did not exist before 2026-09-05
    /// and is the one an operator now meets most: a comment copied and pasted
    /// into an email says *"1 comment copied from pdfcer"*, not *"1 object"*.
    #[test]
    fn the_os_marker_counts_objects_and_comments_separately() {
        assert!(os_marker(0, 1).starts_with("1 comment copied"));
        assert!(os_marker(1, 0).starts_with("1 object copied"));
        assert!(os_marker(2, 1).starts_with("2 objects and 1 comment copied"));
        assert!(os_marker(1, 3).starts_with("1 object and 3 comments copied"));
        // ★ The trailing pronoun follows the TOTAL, not either count: "Paste
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
}
