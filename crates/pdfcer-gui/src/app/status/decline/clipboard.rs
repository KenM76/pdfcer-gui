//! # `app::status::decline::clipboard` — the decline a MODE raises on a cut or
//! a paste
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/status/decline/clipboard.md`.

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
/// It takes the [`ModeRefusal`] rather than deriving one from a command id
/// and a `Capabilities`, because the caller is the only place that knows
/// **both** the verb and the operand — the dispatcher has just matched on what
/// is on the clipboard in order to choose the gate, and asking it to hand over
/// the answer it already computed is what stops a second derivation growing up
/// here and disagreeing with the gate about which sentence applies.
pub(crate) fn record_mode_refusal(why: ModeRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::ClipboardMode(why)));
}
