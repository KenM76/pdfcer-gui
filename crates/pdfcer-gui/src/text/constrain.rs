//! # `text::constrain` — the sentence a held Shift puts on the status row
//!
//! Three strings, for [`crate::canvas::constrain`].
//!
//! ## ★ Why a constraint gets words at all, when the ghost already shows it
//!
//! `ui-conventions/drag-moves.md` D5 states the failure mode in the operator's
//! own position:
//!
//! > *"The operator holds Shift, gets a result they did not expect, and cannot
//! > tell whether the modifier did anything."*
//!
//! The ghost answers *"the object is behaving like this"*. It does not answer
//! *"…because you are holding Shift"*, and those are different questions. An
//! operator whose drag comes out horizontal cannot tell, from the picture
//! alone, whether the tool locked it or whether their hand was simply steady —
//! and the moment they cannot tell, they stop trusting the key and start
//! aligning by eye, which is the whole capability lost.
//!
//! ## The wording rule these three follow
//!
//! **Say what is happening, in the operator's terms, and name the key.** Not
//! *"axis constraint active"* — that is a state, in the program's vocabulary,
//! and it tells someone who does not already know nothing at all. The key is
//! named because the sentence is also how the feature is *discovered*: an
//! operator who reads *"Shift: locked to left and right"* once has learned that
//! Shift constrains, which no amount of correct behaviour teaches on its own.
//!
//! Present tense and no period, matching the other transient in-flight line on
//! this row (`text::doctabs`' drag captions): these are captions on something
//! happening now, not statements about something that happened.

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
        // ★ "Left and right", not "the X axis" and not "horizontally". The
        // operator can see left and right; X is the file format's word, and
        // "horizontally" is an adverb doing the work of a picture.
        Lock::Axis(Axis::Horizontal) => "Shift: locked to left and right",
        Lock::Axis(Axis::Vertical) => "Shift: locked to up and down",
        // ★★ "Keeping its proportions", not "aspect ratio locked". The first is
        // what the operator wanted; the second is what a program does about it.
        // It also states the *consequence* — the shape does not distort — which
        // is the fact that makes the key worth reaching for.
        Lock::Aspect => "Shift: keeping its proportions",
        // ★ It names the STEP, because that is the fact an operator acts on —
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
    ///
    /// The sharing case is the one worth guarding: a copy-paste that left both
    /// axes saying "left and right" would be invisible in review and would tell
    /// the operator the exact opposite of the truth half the time.
    #[test]
    fn each_lock_has_its_own_sentence() {
        let h = caption(Lock::Axis(Axis::Horizontal));
        let v = caption(Lock::Axis(Axis::Vertical));
        let a = caption(Lock::Aspect);
        assert_ne!(h, v);
        assert_ne!(h, a);
        assert_ne!(v, a);
    }

    /// ★ Every sentence names the key, because the caption is also how the
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
