//! # `testsupport` — fixtures for this crate's tests
//!
//! Test-only. Resolves fixtures under the engine's synthetic fixture tree,
//! which this workspace builds against by path, so it is always present.

pub use crate::opendoc::fixtures::engine_fixture;
