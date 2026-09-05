//! # `app::status::decline::textedit` — the two declines the TEXT CARET raises
//!
//! `OPERATOR_REQUESTS.md` **O127**, defects 2 and 3. Two recording functions,
//! and the argument they share.
//!
//! ## Why it is its own file
//!
//! **R2.** [`super`] crossed 1,500 lines the day these two arrived — the second
//! split from it, after `decline/floor.rs` — and the seam is a real one rather
//! than a size-driven cut. Everything else in that file answers *"what is a
//! decline, and how long does it owe its sentence?"*; this answers *"what does
//! the text caret decline, and who says so?"*, which is a subject with two
//! call-site families and an argument about **channels** that nothing else on
//! that surface shares.
//!
//! ## ★★★ The argument, once, for both: a sentence in the wrong slot is silence
//!
//! Every cause below was **already being reported** before O127. Four of the
//! reflow refusals went through `crate::app::actions::record_note`, which the
//! bar draws under **`⚑ About your last edit:`**; four more collapsed into
//! [`super::Declined::EditRefused`]'s nine cause-free words; and Enter in an
//! existing run said nothing at all, because it quietly committed instead.
//!
//! The operator's verdict on the first family was *"I haven't seen the reflow
//! option actually work with anything when I press it."* **He was answered
//! every single time.** In a slot whose whole contract is *an edit happened;
//! here is the part you cannot see* — for a press where nothing happened — in
//! the past tense, about an earlier gesture, truncated to 45 % of the bar
//! (`NOTES_WIDTH_FRACTION`).
//!
//! [`super`]'s own header had already ruled on this exact swap, for two other
//! sentences, in these words:
//!
//! > *"an operator who reads 'About your last edit' after a gesture that did
//! > nothing has been told a small lie confidently."*
//!
//! ⇒ So the fix for two of O127's three defects is **not new copy**. It is
//! these two functions, which put existing sentences in the slot that wears
//! `⊗` and means *nothing happened*. That is worth a file of prose because it
//! is the second time this project has proved the same thing: **a control that
//! answers in the wrong place is indistinguishable, from the operator's chair,
//! from a control that does not answer at all.**
//!
//! ## ★★ Written unconditionally, overwriting whatever was live
//!
//! [`super::record_text_style`]'s rule, and it matters more here: reflow is a
//! control an operator presses **twice** when nothing appears to happen. The
//! second press must produce the second press's sentence, and `LAST` is a slot
//! rather than a queue precisely so the most recent answer is the visible one.

use super::{Declined, LAST};

/// **Record that a paragraph reflow did not happen, and why** —
/// `OPERATOR_REQUESTS.md` **O127**, defect 3.
///
/// ★★ Called from **two positions**, which is deliberate rather than untidy:
/// `app::dispatch::text` resolves the caret and can decline before any verb
/// runs, and `app::actions::textstyle` maps the engine's own refusal after one
/// has. Both name a cause the operator can act on, and neither can see the
/// other's — so merging them would mean one of the two speaking for a condition
/// it cannot observe.
///
/// ★ The `textstyle` call site is what makes the engine's half legible at all.
/// It writes through `Result::inspect_err` **inside** the funnel's closure, and
/// the ordering is what makes that work: `vector_edit` takes the decline floor
/// *before* running the closure, and `BeforeTheVerb::refused` fills the slot
/// only `if slot.is_none()` — so this specific sentence survives and the
/// generic *"that change was refused"* stands aside. That is the floor's
/// documented purpose; reflow is the first verb to use it.
pub(crate) fn record_reflow(why: crate::text::textedit::ReflowRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::Reflow(why)));
}

/// **Record that Enter could not make a line break here** —
/// `OPERATOR_REQUESTS.md` **O127**, defect 2.
///
/// ★★ The one decline in this module raised by a **keystroke** rather than by a
/// command or a verb, and it belongs here for exactly the reason the others do:
/// the operator did something, nothing happened, and the slot that says so
/// wears `⊗`. A key that quietly does something else is the same defect class
/// as a button that quietly does nothing — this project's founding one.
///
/// ★ It arrives by `Action`, not by a direct call. `canvas::textedit::keys` is
/// outside `crate::app`, and [`super`] is `pub(super)` there on purpose — *"a
/// decline is written by the one dispatcher and read by the one bar"*. So the
/// keystroke raises `TextAction::EnterCannotSplit` and the apply arm calls
/// this. Widening the module's visibility so a keystroke handler could reach
/// the store would have traded a real invariant for two saved lines.
///
/// ★ The draft is left **alive** afterwards, which matters to the wording as
/// well as to the gesture: the sentence is about the key just pressed, the
/// operator is still in the text it is about, and *"press Ctrl+Enter to finish
/// this edit"* therefore names something they can do right now.
pub(crate) fn record_enter_cannot_split() {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::EnterCannotSplit));
}
