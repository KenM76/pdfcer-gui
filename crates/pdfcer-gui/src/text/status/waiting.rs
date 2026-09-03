//! # `text::status::waiting` — **sentences about the program being busy**
//!
//! One function today. It is its own module rather than a paragraph in
//! [`super`] because it is a different **species** of sentence from everything
//! that catalog holds, and the distinction is the one `app::status`'s own header
//! spends forty lines making.
//!
//! Every other line in that half of the bar describes **an event**: what a fill
//! inferred, what a move had to change, what a command declined to do. Each is
//! keyed on the document's edit epoch and retires when the document moves past
//! it.
//!
//! This describes **a state** — *the picture you are looking at is not the
//! answer yet* — which is live for as long as its condition holds, needs no
//! retirement rule, and can appear for one edit and not the next depending only
//! on how hard the page was to draw.
//!
//! ★ Filed apart so that the second sentence of this species, when it comes, is
//! written beside the first and under the same rules rather than filed by
//! whichever group looked closest.

/// **The page is still being redrawn** — `OPERATOR_REQUESTS.md` O63.
///
/// # ★★★ The two ways this sentence could be wrong, and they are opposite
///
/// **Too alarming** and it reads as a fault. Nothing is wrong: the edit
/// happened, the document is correct, and a picture is being made of it. A
/// sentence that sounded like a warning would teach the operator to distrust an
/// edit that worked.
///
/// **Too quiet** and it fails at its one job. This exists because on a dense
/// drawing the picture is a second or two behind the document, and an operator
/// who cannot tell *"the program is drawing"* from *"the program ignored me"*
/// presses the button again — which is a second edit, on top of the first,
/// neither of which they wanted.
///
/// ⇒ So it states the fact in the operator's own terms and puts the reassurance
/// first: **the change is already made**. The redraw is described as something
/// happening to the *picture*, never to the document.
///
/// # ★★ Why it does not name a duration or show a progress bar
///
/// Because nothing here knows one. A page's render time is a property of its
/// content — 8.97 ms for a text page against 877 ms for a CAD sheet — and a
/// number invented for the sake of having one would be wrong on most documents.
/// A bar that filled at a rate nobody could predict would be worse than a
/// sentence, because it promises a shape of wait it cannot keep.
#[must_use]
pub const fn page_catching_up() -> &'static str {
    "Your change is made \u{2014} the picture of it is still being drawn."
}
