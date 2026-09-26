//! # `text::constrain` — the sentence a held Shift puts on the status row
//!
//! Three strings, for [`crate::canvas::constrain`].
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/constrain.md`.

use crate::canvas::constrain::{Axis, Lock};

/// The sentence for a live constraint.
///
/// One function over the enum rather than one per variant, for the reason
/// [`crate::text::resizing::refusal`] gives for the same shape: a variant added
/// to [`Lock`] becomes a compile error here instead of a constraint that
/// silently announces nothing.
#[must_use]
pub const fn caption(lock: Lock) -> &'static str {
    match lock {
        // "Left and right", not "the X axis" and not "horizontally". The
        // operator can see left and right; X is the file format's word, and
        // "horizontally" is an adverb doing the work of a picture.
        Lock::Axis(Axis::Horizontal) => "Shift: locked to left and right",
        Lock::Axis(Axis::Vertical) => "Shift: locked to up and down",
        // "Keeping its proportions", not "aspect ratio locked". The first is
        // what the operator wanted; the second is what a program does about it.
        // It also states the *consequence* — the shape does not distort — which
        // is the fact that makes the key worth reaching for.
        Lock::Aspect => "Shift: keeping its proportions",
        // It names the STEP, because that is the fact an operator acts on —
        // "constrained" tells them a rule is in force and not what it will let
        // them have. Fifteen degrees is what makes the four right angles and
        // the four diagonals reachable, and saying the number is how they find
        // that out without counting.
        Lock::Angle => "Shift: turning in steps of 15°",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant has words, and the two axes do not share one sentence.
    #[test]
    fn each_lock_has_its_own_sentence() {
        let h = caption(Lock::Axis(Axis::Horizontal));
        let v = caption(Lock::Axis(Axis::Vertical));
        let a = caption(Lock::Aspect);
        assert_ne!(h, v);
        assert_ne!(h, a);
        assert_ne!(v, a);
    }

    /// Every sentence names the key, because the caption is also how the
    /// feature is discovered.
    #[test]
    fn every_sentence_names_the_key() {
        for lock in [
            Lock::Axis(Axis::Horizontal),
            Lock::Axis(Axis::Vertical),
            Lock::Aspect,
        ] {
            assert!(
                caption(lock).contains("Shift"),
                "a caption that does not name the key teaches nothing: {}",
                caption(lock)
            );
        }
    }
}
