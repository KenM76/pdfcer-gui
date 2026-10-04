//! The sentence for a style ladder that took an axis OFF (`StyleTarget` with
//! `Some(false)`), and the `text-style-ladder` trace every ladder outcome
//! leaves.
//!
//! Off has its own rungs' meanings: `RealFaceOnPage` and
//! `StandardFourteenSibling` bound the face of the run's own family without
//! the axis, `SynthesisRemoved` undid a stroke or shear, and `AlreadyStyled`
//! means the text never had it.

use pdfcer_core::text_edit::{StyleLadder, StyleRung};

use crate::text::status as t;

/// The sentence for an off ladder, or `None` for an outcome with no sentence
/// here (traced by the caller's `text-style-ladder` line).
pub(super) fn note(ladder: &StyleLadder) -> Option<String> {
    let (bold, italic) = (ladder.removed.bold(), ladder.removed.italic());
    match (ladder.rung, ladder.bound.as_deref()) {
        (StyleRung::RealFaceOnPage, Some(to)) => {
            Some(t::text_style_off_face(bold, italic, to, false))
        }
        (StyleRung::StandardFourteenSibling, Some(to)) => {
            Some(t::text_style_off_face(bold, italic, to, true))
        }
        (StyleRung::SynthesisRemoved, _) => Some(t::text_style_off_unsynthesised(bold, italic)),
        (StyleRung::AlreadyStyled, _) => Some(t::text_style_already_without(bold, italic)),
        _ => None,
    }
}

/// Trace what the ladder decided: its rung, the axes asked on and off, the
/// axes whose synthesis it undid, and the face it bound.
pub(super) fn trace(ladder: &StyleLadder) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "text-style-ladder rung={} requested={} removed={} unsynthesised={} bound={}",
            rung_token(ladder.rung),
            word(ladder.requested.axes()),
            word(ladder.removed.axes()),
            word(ladder.unsynthesised.axes()),
            ladder.bound.as_deref().unwrap_or("-")
        )
    });
}

/// The rung as one trace token; `StyleRung`'s `Display` is prose.
const fn rung_token(rung: StyleRung) -> &'static str {
    // ui-text-exempt: trace tokens, never displayed
    match rung {
        StyleRung::RealFaceOnPage => "page-face",
        StyleRung::StandardFourteenSibling => "standard-14",
        StyleRung::SuppliedFaceEmbedded => "supplied-face",
        StyleRung::Synthetic => "synthetic",
        StyleRung::AlreadyStyled => "already-styled",
        StyleRung::SynthesisRemoved => "synthesis-removed",
        _ => "other",
    }
}

/// One trace token for an axes label: `bold-italic`, or `-` for none.
fn word(axes: &str) -> String {
    if axes == "nothing" {
        "-".to_owned()
    } else {
        axes.replace(' ', "-")
    }
}
