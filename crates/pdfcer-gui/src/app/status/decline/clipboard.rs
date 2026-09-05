//! # `app::status::decline::clipboard` — the decline a MODE raises on a cut or
//! a paste
//!
//! One recording function, and the argument for why it exists at all.
//!
//! ## ★★★ It is the second half of a fix, and without it the fix is a
//! regression
//!
//! The driven sweep of 2026-09-05 found, as its first failure:
//!
//! ```text
//! chord-command      chord="Ctrl+C" id=edit.copy  via=clipboard-event
//! clipboard-copy     kind=selection page=0 objects=0 annots=1 thin=0 bytes=395
//! chord-command      chord="Ctrl+V" id=edit.paste via=clipboard-event
//! chord-not-offered  id=edit.paste mode=review
//! ```
//!
//! **Copy was offered in Review and paste was not**, so an operator in the mode
//! whose entire purpose is marking up somebody else's drawing could copy a
//! comment and had nowhere to put it. The cause was
//! [`crate::app::modes::capability::offers_command`]: Paste lives on the Edit
//! tab, Review is not shown that tab, and the chord was refused before the
//! dispatcher — which had been gating the effect **correctly** on the
//! clipboard's contents since 2026-09-05 and was never reached.
//!
//! ⇒ The chord is now pushed through blind, on this project's standing pattern
//! (*"push the chord blind, gate the effect in dispatch"*, stated at
//! `shell::commands::catalog::view`'s cut registration). And that pattern has a
//! debt attached which is easy to miss: **a chord that is refused at the gate
//! at least traces `chord-not-offered`. A chord that reaches a dispatcher which
//! silently returns traces nothing at all.** `app::dispatch::clipboard`'s two
//! mode gates did exactly that — `command-declined … reason=mode-cannot-paste-here`
//! on the diagnostic channel and **nothing on any surface**.
//!
//! So this file exists because the first half of the fix would otherwise have
//! moved the defect rather than closed it: from *a key that does nothing in
//! Review* to *a key that does nothing in Read and Review and says less about
//! it*.
//!
//! ## ★★ Why `decline` and emphatically not `record_note`
//!
//! `crate::app::actions::record_note` draws under **`⚑ About your last edit:`**
//! — a slot whose contract is *an edit happened; here is the part you cannot
//! see*. Nothing happened here. `decline/textedit.rs`'s header carries the full
//! argument, in [`super`]'s own words:
//!
//! > *"an operator who reads 'About your last edit' after a gesture that did
//! > nothing has been told a small lie confidently."*
//!
//! This is the fourth application of that ruling and the first outside the text
//! caret. The `⊗` slot means *this did not happen*, which is exactly what a
//! mode refusal is.
//!
//! ⚠ **The clipboard's OTHER refusals still go to `record_note`**, and that is
//! left alone rather than swept up: `crate::canvas::clipboard::Refusal` is
//! about the operand (*nothing is selected*, *nothing was copied*) and several
//! driven checks read the note slot for them. Moving them is its own change
//! with its own falsification, and doing it inside this one would have made a
//! two-line gate fix into a re-baseline of the status bar. Named here so the
//! next session finds it stated rather than inferring that the split was
//! principled.
//!
//! ## Written unconditionally, overwriting whatever was live
//!
//! [`super::record_text_style`]'s rule, and it matters for the same reason it
//! matters for reflow: this is a refusal an operator meets by **pressing the
//! chord again**, having read the sentence and not yet moved the selector. The
//! second press must produce the second press's sentence, and `LAST` is a slot
//! rather than a queue precisely so the most recent answer is the visible one.

use super::{Declined, LAST};
use crate::text::clipboard::ModeRefusal;

/// **Record that the active mode does not do this clipboard verb.**
///
/// Called from `app::dispatch::clipboard`, in the **dispatch** phase: the
/// refusal is knowable before any action is raised, because it is a fact about
/// the mode and the clipboard rather than about the document. That is
/// [`super::record`]'s call site, not [`super::record_save_failure`]'s, and the
/// distinction is the one those two functions' docs already draw.
///
/// ★ It takes the [`ModeRefusal`] rather than deriving one from a command id
/// and a `Capabilities`, because the caller is the only place that knows
/// **both** the verb and the operand — the dispatcher has just matched on what
/// is on the clipboard in order to choose the gate, and asking it to hand over
/// the answer it already computed is what stops a second derivation growing up
/// here and disagreeing with the gate about which sentence applies.
pub(crate) fn record_mode_refusal(why: ModeRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::ClipboardMode(why)));
}
