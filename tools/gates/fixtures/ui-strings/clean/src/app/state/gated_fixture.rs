//! FIXTURE — a module declared `#[cfg(test)] mod gated_fixture;`.
//!
//! Every literal below is whitespace-bearing, operator-shaped, outside the
//! catalog and carries no exemption marker. It is nonetheless legal, because
//! the compiler never emits this file into a release build: the declaration in
//! `state.rs` is cfg-gated, and Rust permits exactly one declaration per module
//! path, so there is no second route by which this file enters the binary.
//!
//! This file therefore belongs to the CLEAN fixture. If the gate's test-only
//! exclusion ever breaks, `--self-test` assertion A turns red here rather than
//! the failure surfacing months later as noise on the real tree.
//!
//! Not compiled by anything.

/// Builds the bytes of a throwaway document for a test to parse.
pub fn synthetic_source() -> &'static str {
    "<< /Type /Catalog /Pages 2 0 R >>"
}

/// A panic message on a path a test drives deliberately.
pub fn must_have_a_page(pages: usize) {
    assert!(pages >= 1, "a document needs at least one page");
}
