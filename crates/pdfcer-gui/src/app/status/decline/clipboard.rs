//! # `app::status::decline::clipboard` — the decline a MODE raises on a cut or
//! a paste
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/status/decline/clipboard.md`.

use super::{Declined, LAST};
use crate::text::clipboard::ModeRefusal;

/// **Record that the active mode does not do this clipboard verb.**
pub(crate) fn record_mode_refusal(why: ModeRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::ClipboardMode(why)));
}
