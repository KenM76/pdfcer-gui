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
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/handles/name.md`.

use super::Grip;

impl Grip {
    /// **The grip's stable name, for the trace** — and deliberately not
    /// `{:?}`.
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
