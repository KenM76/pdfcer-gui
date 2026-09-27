//! # `testsupport` — fixtures for this crate's tests
//!
//! Test-only. Resolves fixtures under the engine's synthetic fixture tree,
//! which this workspace builds against by path, so it is always present.

use std::path::PathBuf;

/// The path of `rel` under the engine's `fixtures/synthetic`, asserted to exist.
pub fn engine_fixture(rel: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../pdfcer/fixtures/synthetic")
        .join(rel);
    assert!(
        path.exists(),
        "the engine fixture {rel} is missing at {}",
        path.display()
    );
    path
}
