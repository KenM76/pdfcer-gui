//! # `app::state::fixtures` — how a test opens a document, and the fixture names
//!
//! `#[cfg(test)]` only: the two openers, and the constants naming the files they
//! resolve. A constant naming a fixture is meaningless without the function that
//! says which of the two roots it is relative to, so the two live together.
//!
//! ★ There are two fixture corpora and they mean different things. The engine's
//! own corpus under `D:\Dev\pdfcer\fixtures` is READ-ONLY here; `fixtures/` in
//! this repository holds the pages this shell had to author because no engine
//! fixture exercised the condition. Hence two named openers rather than one
//! taking a root or a flag: a boolean can be got backwards, and the failure
//! would read *"the fixture is missing"* on a machine where both trees exist —
//! a message pointing at the wrong problem.

use super::OpenDoc;
use pdfcer_core::document::Document;
use pdfcer_core::edit::EditSession;

/// Open a fixture the way [`PdfcerApp::open_path`] does, without a frame —
/// the same three calls in the same order, so what is under test is the state
/// machine rather than an approximation of it.
///
/// At module level rather than inside `mod tests`, and `pub(crate)`, because
/// tests all over the crate need the identical starting point. A second fixture
/// opener would be a second way to assemble an `OpenDoc` — exactly what
/// [`OpenDoc::new`]'s own docs argue against — so the visibility widens rather
/// than the function being copied.
#[cfg(test)]
pub(crate) fn open_fixture(rel: &str) -> OpenDoc {
    let path = crate::panels::objects::test_support::engine_fixture(rel);
    let doc = Document::load(&path).expect("the fixture loads");
    let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
    OpenDoc::new(path, EditSession::new(doc), pages)
}

/// Open a fixture from **this** repository's `fixtures/`, the same way
/// [`open_fixture`] opens one of the engine's.
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
// The fixture names
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) const FOUR_PAGES: &str = "pageops/four-pages.pdf";

/// **One page, one `/Widget` annotation that no `/AcroForm` field owns** —
/// the input `EditSession::adopt_widget` exists for, and the only fixture in
/// either corpus that has the shape.
///
/// ★★ Hand-authored from ISO 32000-1, generator checked in as
/// `fixtures/orphan-widget.PROVENANCE.py`, because **no pdfcer verb can
/// produce this file**: every field-creating verb registers its widget in
/// `/AcroForm /Fields` in the same commit. An unregistered widget is what a
/// damaged or third-party document looks like — a form flattened by a tool
/// that dropped `/AcroForm` and left the annotations, or a page extracted
/// from a form without its field tree. ⇒ *a fixture produced by the code
/// under test measures that code's agreement with itself.*
///
/// ★ The widget carries its own `/T (Orphan)` and `/FT /Tx` deliberately.
/// That makes it the **merged field-widget** — a widget that IS its own
/// field and was simply never registered, which is the recoverable case. A
/// bare kid with no `/T` refuses with `WidgetHasNoFieldIdentity` before any
/// name is examined, which would make a name-refusal test pass for the wrong
/// reason. The generator's header lists `adopt_plan`'s preconditions and the
/// byte that clears each.
#[cfg(test)]
pub(crate) const ORPHAN_WIDGET: &str = "orphan-widget.pdf";

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
/// load-bearing properties are asserted in `crate::dialogs::signature`.
#[cfg(test)]
pub(crate) const SIGNED_TWO_PAGES: &str = "signed-two-pages.pdf";
/// **A document whose catalog names `/PageMode` twice, with two different
/// values** — hand-authored because nothing else in either corpus reaches
/// `Document::load_anomalies()`.
///
/// ★ Both of the facts that make it worth its bytes are asserted rather than
/// assumed: it **loads**, and it produces **exactly one** anomaly carrying the
/// kept and discarded values the operator would see.
/// `fixtures/contradicts-itself.PROVENANCE.py` is the generator, and says why
/// its xref is deliberately sound.
#[cfg(test)]
pub(crate) const CONTRADICTS_ITSELF: &str = "contradicts-itself.pdf";

/// **A document with no cross-reference table at all**, so pdfcer rebuilds one
/// by scanning — and whose scan finds two objects it cannot keep.
///
/// ★ It is the only file in either corpus that reaches
/// `RecoveryReport::objects_dropped`. Every other fixture either has a sound
/// index (so `Document::recovery()` is `None` and there is no report to read)
/// or, like [`CONTRADICTS_ITSELF`], deliberately keeps its index sound in order
/// to test the anomaly path without lighting this one.
///
/// ★★ The two drops are the two different stories that share one reason code:
/// object 9 does not exist (the bytes `9 0 obj` appear inside the content
/// stream's own text, which the scan is obliged to try), and object 8 is real
/// and truncated. The disclosure has to be readable for both without making the
/// first sound like the second. `fixtures/recovered-with-losses.PROVENANCE.py`
/// carries the whole account, including why `DropReason::IdMismatch` cannot be
/// produced from a hand-written file.
#[cfg(test)]
pub(crate) const RECOVERED_WITH_LOSSES: &str = "recovered-with-losses.pdf";

/// **The control for [`RECOVERED_WITH_LOSSES`]**: the same damage, the same
/// recovery path, and nothing the scan could not keep.
///
/// ★ It is a RECOVERED file rather than a sound one, and that is the whole
/// point of it. A driven check that opened a *sound* document to prove the
/// dropped-object block is absent would also pass if the panel never opened or
/// the document was never recovered, so it would measure nothing. This file
/// differs from its sibling in exactly one property, so the two launches
/// isolate exactly one variable.
///
/// Its property is asserted through the engine in `crate::panels::docprops`,
/// in the suite that runs on every `cargo test`, so a fixture that stopped
/// being a control surfaces there rather than as a red driven check blaming
/// the application. `fixtures/recovered-no-losses.PROVENANCE.py` carries the
/// rest.
#[cfg(test)]
pub(crate) const RECOVERED_NO_LOSSES: &str = "recovered-no-losses.pdf";
