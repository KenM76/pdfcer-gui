//! # `canvas::handles::name` — a grip's stable spelling for the trace
//!
//! One function, in its own file, and the reason is R2 rather than taste:
//! `handles.rs` was **1,499 lines** when this was written — one line under the
//! ceiling — so the alternative was splitting its two `#[cfg(test)]` modules
//! out to make room for twenty lines of `match`.
//!
//! ⚠ That split was attempted first and reverted. `handles.rs` carries **two**
//! test modules, not one, so a cut at the first `#[cfg(test)]` swept up both
//! and left their braces unbalanced. A seam that has to be found by counting
//! braces is not a seam.
//!
//! ⇒ This is the smaller, truer cut: the function has one job, no dependency
//! on anything else in the module, and nothing else wants to live beside it.

use super::Grip;

impl Grip {
    /// **The grip's stable name, for the trace** — and deliberately not
    /// `{:?}`.
    ///
    /// # Why a hand-written function rather than the derive
    ///
    /// It goes into a `key=value` line that a driven check parses, and a
    /// `Debug` spelling in a parsed field is banned in this tree. That is not
    /// a style rule: `{:?}` on a domain value has already produced **two**
    /// false failure reports here, one of which reported the opposite of the
    /// truth while quoting the truth in its own message.
    ///
    /// # The abbreviations
    ///
    /// Compass points are abbreviated and the two non-compass grips are not.
    /// `NorthWest` earns nothing over `NW` on a line carrying eleven other
    /// fields — but `M` and `R` beside eight compass points would read as two
    /// more directions, so [`Grip::Move`] and [`Grip::Rotate`] keep their
    /// words.
    ///
    /// ⚠ Exhaustive with no wildcard, so a tenth grip is a compile error here
    /// rather than a press that traces as something it is not.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::NorthWest => "NW",
            Self::North => "N",
            Self::NorthEast => "NE",
            Self::East => "E",
            Self::SouthEast => "SE",
            Self::South => "S",
            Self::SouthWest => "SW",
            Self::West => "W",
            Self::Move => "Move",
            Self::Rotate => "Rotate",
        }
    }
}
