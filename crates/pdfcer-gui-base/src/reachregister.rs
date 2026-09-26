//! # `reachregister` — the allow-list, and only the allow-list
//!
//! **The DATA half of [`super`]**, kept apart from the check that reads it.
//!
//! ## The seam is real, and it is the one the file kept re-discovering
//!
//! `reach.rs` does two things that change for entirely different reasons:
//!
//! | half | what it is | changes when |
//! |---|---|---|
//! | **this file** | the register — every registered command with no dispatch arm, and *why* | a command is wired, deferred, or its reason expires |
//! | `mod.rs` | the CHECK — a `syn` parse of the dispatcher's `match`, the guard evaluation, and the tests | the dispatcher's shape changes, or the check gets sharper |
//!
//! The second is machinery and is nearly static. The first is a **living
//! document** that grows a paragraph every time somebody explains why a
//! control is inert and shrinks by an entry every time somebody fixes one —
//! and every one of those paragraphs is prose, so it is the half that pushes
//! the line count.
//!
//! ⚠ **The pressure that produces is the reason for the split, and trimming a
//! reason to get back under R2's ceiling is the worst available response**,
//! because the reason is the entry's whole value. R2 says so in as many words:
//! *"when a file approaches the limit, that is the signal to find the seam,
//! not to raise the limit."*
//!
//! ## The counts live with the data
//!
//! `super::tests::the_p3_tension_is_counted` pins both figures and reads them
//! from here through the ordinary path, so the number quoted in `super`'s
//! header and the length of the list below move together or fail.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/reachregister.md`.

// ===========================================================================
// THE ALLOW-LIST
// ===========================================================================

/// **Registered, deliberately without a dispatch arm, and why.**
pub const SCAFFOLDED: &[(&str, &str)] = &[
    // Empty. Every id that stood here has been wired, unregistered, or
    // deleted; the reasons that were transferable are in this constant's doc
    // comment, and the rest were the entries themselves.
];

/// **The mirror defect: a literal arm that no token can reach, and why each is
/// tolerated.**
pub const UNREACHED_ARMS: &[(&str, &str)] = &[];
