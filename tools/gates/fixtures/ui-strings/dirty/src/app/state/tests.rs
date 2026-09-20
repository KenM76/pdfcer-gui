//! FIXTURE — the cfg-gated file that must NOT be reported.
//!
//! `state.rs` declares this module `#[cfg(test)] mod tests;`, so the compiler
//! never emits it into a release build and no literal here can reach the
//! operator. It sits in the DIRTY fixture on purpose: the self-test asserts on
//! the exact hit list, so if the test-only exclusion breaks, the extra lines
//! from this file make that list wrong and the assertion says which file they
//! came from.
//!
//! Not compiled by anything.

#[test]
fn the_label_is_not_a_bare_literal() {
    assert_eq!(
        super::delete_label(),
        "Delete selected object",
        "a test comparing against the expected label is not operator copy"
    );
}
