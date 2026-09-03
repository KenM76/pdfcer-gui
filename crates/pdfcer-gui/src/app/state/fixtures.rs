//! # `app::state::fixtures` — **how a test opens a document, and why there are two roots**
//!
//! Two functions, `#[cfg(test)]` only. Split out of `app::state` under **R2** on
//! 2026-08-30: that file sat at exactly 1,500 lines — the ceiling — so it had
//! room for nothing at all, and test-only helpers are the part of it that is not
//! in the shipped binary.
//!
//! ## ★★ The distinction the two functions exist to make un-gettable-wrong
//!
//! There are two fixture corpora and they mean different things:
//!
//! | root | what it is |
//! |---|---|
//! | `D:\Dev\pdfcerixtures` | the **engine's** own corpus. READ-ONLY, per this project's governing rule |
//! | `fixtures/` here | the pages this shell had to author because no engine fixture exercised the condition — right-aligned text, a node-draggable polyline, an image-only scan, a page of rotated strings, a button that submits to a web server |
//!
//! ⇒ Two named functions rather than one taking a root or a flag. A single
//! function with a boolean would let a call site pick the wrong tree by getting
//! the boolean backwards, and the failure would be *"the fixture is missing"* on
//! a machine where both trees exist — a message pointing at the wrong problem.
//! Two functions cannot be got backwards; they can only be got *wrong*, loudly.

use super::OpenDoc;
use pdfcer_core::document::Document;
use pdfcer_core::edit::EditSession;

/// Open a fixture the way [`PdfcerApp::open_path`] does, without a frame —
/// the same three calls in the same order, so what is under test is the state
/// machine rather than an approximation of it.
///
/// At module level rather than inside `mod tests`, and `pub(crate)`, because
/// three other modules' tests need the identical starting point:
/// [`crate::app::cache`]'s assert against caches whose fields are declared on
/// [`OpenDoc`], `crate::app::status`'s drive the bar over a real document, and
/// `crate::find`'s run a real search and a real reveal against real page
/// geometry. A second fixture opener would be a second way to assemble an
/// `OpenDoc` — exactly what [`OpenDoc::new`]'s own docs argue against — so the
/// visibility widens rather than the function being copied.
#[cfg(test)]
pub(crate) fn open_fixture(rel: &str) -> OpenDoc {
    let path = crate::panels::objects::test_support::engine_fixture(rel);
    let doc = Document::load(&path).expect("the fixture loads");
    let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
    OpenDoc::new(path, EditSession::new(doc), pages)
}

/// Open a fixture from **this** repository's `fixtures/`, the same way
/// [`open_fixture`] opens one of the engine's.
///
/// Two openers rather than one taking a root, because the two roots mean
/// different things and the difference is the project's governing rule:
/// `D:\Dev\pdfcer\fixtures` is READ-ONLY and is the engine's own corpus, while
/// `fixtures/` here holds the pages this shell had to author because no engine
/// fixture exercised the condition — right-aligned text, a node-draggable
/// polyline, an image-only scan, and now a page of rotated strings. A single
/// function with a flag would let a call site pick the wrong tree by getting a
/// boolean backwards; two named functions cannot.
#[cfg(test)]
pub(crate) fn open_local_fixture(rel: &str) -> OpenDoc {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(rel);
    assert!(
        path.exists(),
        "this repository's fixture {rel} is missing at {}",
        path.display()
    );
    let doc = Document::load(&path).expect("the fixture loads");
    let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
    OpenDoc::new(path, EditSession::new(doc), pages)
}

// ---------------------------------------------------------------------------
// The fixture NAMES, moved here 2026-09-01 under R2
// ---------------------------------------------------------------------------
//
// ★ They lived at the bottom of `state.rs` and belong beside the opener that
// resolves them: a constant naming a file is only meaningful with the function
// that says which of the two fixture roots it is relative to, and that
// function is in this file. Moving them is what took `state.rs` back under the
// gate when the decomposition cache's form-edit counter pushed it over.

#[cfg(test)]
pub(crate) const FOUR_PAGES: &str = "pageops/four-pages.pdf";

/// One page carrying the same words at 0°, 90°, 180°, 270° and 30° — the page
/// `canvas::textsel`'s §8 rules are asserted on. See
/// [`crate::canvas::textsel::fixture`] for what each string is for and why it
/// is set in capitals.
#[cfg(test)]
pub(crate) const ROTATED_TEXT: &str = "rotated-text.pdf";
/// Four optional-content groups: 4 and 7 on by default, 5 and 6 off.
#[cfg(test)]
pub(crate) const PAINTED_LAYERS: &str = "layers/painted-layers.pdf";
/// **Two pages, one approval signature.** Built by
/// `tools/gen-signed-fixture.py`, whose header carries why the engine's own
/// signature fixtures (all one page, none to spare) could not be used; its
/// three load-bearing properties are asserted in `crate::dialogs::signature`.
#[cfg(test)]
pub(crate) const SIGNED_TWO_PAGES: &str = "signed-two-pages.pdf";
