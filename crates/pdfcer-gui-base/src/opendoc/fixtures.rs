//! # `app::state::fixtures` — how a test opens a document, and the fixture names
//!
//! `#[cfg(test)]` only: the two openers, and the constants naming the files they
//! resolve. A constant naming a fixture is meaningless without the function that
//! says which of the two roots it is relative to, so the two live together.
//!
//! There are two fixture corpora and they mean different things. The engine's
//! own corpus under `D:\Dev\pdfcer\fixtures` is READ-ONLY here; `fixtures/` in
//! this repository holds the pages this shell had to author because no engine
//! fixture exercised the condition. Hence two named openers rather than one
//! taking a root or a flag: a boolean can be got backwards, and the failure
//! would read *"the fixture is missing"* on a machine where both trees exist —
//! a message pointing at the wrong problem.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/opendoc/fixtures.md`.

use std::path::PathBuf;

use super::OpenDoc;
use pdfcer_core::document::Document;
use pdfcer_core::edit::EditSession;

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

/// Open a fixture the way [`PdfcerApp::open_path`] does, without a frame —
/// the same three calls in the same order, so what is under test is the state
/// machine rather than an approximation of it.
pub fn open_fixture(rel: &str) -> OpenDoc {
    let path = engine_fixture(rel);
    let doc = Document::load(&path).expect("the fixture loads");
    let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
    OpenDoc::new(path, EditSession::new(doc), pages)
}

/// Open a fixture from **this** repository's `fixtures/`, the same way
/// [`open_fixture`] opens one of the engine's.
pub fn open_local_fixture(rel: &str) -> OpenDoc {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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
// The fixture names
// ---------------------------------------------------------------------------

pub const FOUR_PAGES: &str = "pageops/four-pages.pdf";

/// **One page, one `/Widget` annotation that no `/AcroForm` field owns** —
/// the input `EditSession::adopt_widget` exists for, and the only fixture in
/// either corpus that has the shape.
pub const ORPHAN_WIDGET: &str = "orphan-widget.pdf";

/// **One page, THREE text widgets, every one of them with a drawn `/AP`.**
pub const THREE_TEXT_FIELDS: &str = "three-text-fields.pdf";

/// One page carrying the same words at 0°, 90°, 180°, 270° and 30° — the page
/// `canvas::textsel`'s §8 rules are asserted on. See
/// [`crate::canvas::textsel::fixture`] for what each string is for and why it
/// is set in capitals.
pub const ROTATED_TEXT: &str = "rotated-text.pdf";
/// Four optional-content groups: 4 and 7 on by default, 5 and 6 off.
pub const PAINTED_LAYERS: &str = "layers/painted-layers.pdf";
/// **Two pages, one approval signature.** Built by
/// `tools/gen-signed-fixture.py`, whose header carries why the engine's own
/// signature fixtures (all one page, none to spare) could not be used; its
/// load-bearing properties are asserted in `crate::dialogs::signature`.
pub const SIGNED_TWO_PAGES: &str = "signed-two-pages.pdf";
/// **A document whose catalog names `/PageMode` twice, with two different
/// values** — hand-authored because nothing else in either corpus reaches
/// `Document::load_anomalies()`.
pub const CONTRADICTS_ITSELF: &str = "contradicts-itself.pdf";

/// **A document with no cross-reference table at all**, so pdfcer rebuilds one
/// by scanning — and whose scan finds two objects it cannot keep.
pub const RECOVERED_WITH_LOSSES: &str = "recovered-with-losses.pdf";

/// **The control for [`RECOVERED_WITH_LOSSES`]**: the same damage, the same
/// recovery path, and nothing the scan could not keep.
pub const RECOVERED_NO_LOSSES: &str = "recovered-no-losses.pdf";
