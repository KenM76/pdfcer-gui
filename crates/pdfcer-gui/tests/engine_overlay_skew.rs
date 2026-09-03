//! # `engine_overlay_skew` — the shell's model of a page and the engine's must
//! describe the same page, and these tests are what notices when they stop
//!
//! ## ★★★ STATUS, 2026-08-31: FIXED BY THE ENGINE, AND THESE TESTS INVERTED
//!
//! This file was written as a **reproduction** — three tests that passed on
//! the broken engine and were built to fail on the fixed one, each carrying
//! its own instruction to whoever saw it go red. All three fired within the
//! day: `pdfcer-core` **Pass 186.0** made every content verb resolve its page
//! against the session overlay instead of the base document, and the second
//! symptom in the request — the one nobody had reported and which this file
//! discovered — was reproduced by the engine team before being fixed.
//!
//! So each assertion has been turned round to state the CORRECT behaviour,
//! and the file changes job: it stopped being a claim about somebody else's
//! crate and became this project's regression net under an engine it does not
//! control and updates weekly. The history below is kept deliberately — a
//! reader who finds one of these red needs the whole story, not the verdict.
//!
//! ★ Two of the three now assert an **outcome**, not merely a `Ok`/`Err`: a
//! verb that transformed nothing and reported success, or two models that had
//! each missed the same object, would satisfy the naive shape of every one of
//! these tests while the operator's gesture still did nothing.
//!
//! ## The history: why this test existed, and why in the SHELL's test tree
//!
//! `OPERATOR_REQUESTS.md` row **O64**, in the operator's words:
//!
//! > *"When I add a new image to a pdf I can't edit it unless I save the
//! > document first … I assume this probably affects more than just images."*
//!
//! He is right, and the cause is not in this crate. `EditSession` keeps two
//! page-tree readers side by side:
//!
//! | reader | what it sees | who uses it |
//! |---|---|---|
//! | `EditSession::pages()` = `page_tree::pages_in(&self.graph())` | the **overlay** — the document as this session has edited it | every *authoring* verb, and this shell |
//! | `page_tree::pages(&self.base)` | the **base** — the document as it was on disk | every *content-editing* verb, and `EditSession::page_objects` |
//!
//! `add_image` appends a **new content stream** and a new `/XObject`
//! resource, and both writes land in the session overlay. So after an insert
//! the shell's decomposition (taken from `session.view()`, overlay-aware) has
//! N+1 objects while the engine's own model — the one every geometry verb
//! resolves an index against — still has N. The shell selects the new image
//! at index N and asks the engine to transform it; the engine answers
//! `ObjectOutOfRange`, the funnel traces `-refused`, and the operator sees a
//! gesture that does nothing. Saving flattens the overlay into a new base,
//! which is exactly why his workaround works.
//!
//! ## Why it is a test rather than a paragraph in a request
//!
//! Because `D:\Dev\pdfcer` is READ-ONLY to this project, the fix is not mine
//! to make — it is a feature request. A request that asserts a defect in
//! somebody else's crate had better carry a reproduction, and this project's
//! standing rule is that **a backlog row is a record, not evidence**. Three
//! documents in this repository have previously stated an absence that was
//! false. So the claim in the request is this file, and this file is run by
//! `cargo test --workspace` on every commit.
//!
//! ## The shape they were written in, and why it was right
//!
//! Each test was written to **pass on the broken engine and fail on the fixed
//! one**, saying so in its own assertion message. That was the only honest
//! shape available: a test asserting the *correct* behaviour would have been a
//! red test in a green repository for as long as the request stayed open, and
//! would have been muted within the week.
//!
//! ★★ The value of that shape is now measured rather than argued. All three
//! went red on the day the engine landed the fix, each printed the sentence
//! telling the reader what to do, and inverting them took minutes rather than
//! an investigation. **A tripwire that names its own deletion is worth more
//! than a comment saying the same thing**, because only one of the two is
//! executed.
//!
//! ## Scope — the operator's generalisation, stated as code
//!
//! `add_image` is the sample, not the specification. The defect belongs to
//! **every path that adds page content as a new content stream**, which the
//! engine's own source says is exactly four: `add_image`, `add_text`,
//! `paste_objects` and `flatten_fields`. Annotations are **not** affected:
//! they are addressed by `ObjId` through the overlay-aware `self.value()`,
//! which is why a markup, a ce dimension, a redaction mark and a form field
//! are all editable the instant they are authored. Only the first test here
//! is cheap enough to write without a fixture factory; the rest are named in
//! the request.

use pdfcer_core::edit::EditSession;

/// A one-page document with a little content, built from a fixture that
/// already ships with this repository.
///
/// `a1-titleblock.pdf` is used rather than a synthetic blank because a page
/// with **zero** existing objects would make the skew assertion trivially
/// true (0 vs 1 proves nothing about indexing) and because a real page is
/// what the operator has.
fn fixture() -> pdfcer_core::document::Document {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/a1-titleblock.pdf"
    );
    pdfcer_core::document::Document::load(std::path::Path::new(path))
        .expect("fixture a1-titleblock.pdf must load")
}

/// A 2x2 opaque PNG, encoded inline so this test needs no image fixture.
///
/// The bytes are a minimal valid PNG: signature, IHDR, a single IDAT holding
/// two zlib-stored scanlines, and IEND. Written out rather than generated so
/// the test has no dependency on an encoder crate.
fn tiny_png() -> Vec<u8> {
    // Built at test time with the `image` crate if available would be
    // simpler, but this crate does not depend on it for tests. A 1x1 grey
    // PNG, byte for byte.
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR length + type
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1 x 1
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE, // bitdepth 8, RGB
        0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, // IDAT length + type
        0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D,
        0xB0, // IDAT crc
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82, // IEND
    ]
}

/// ★★★ **The two models of one page, stated with no gesture at all.**
///
/// After `add_image` the shell's decomposition — what the canvas hit-tests and
/// what the Objects panel lists — and `EditSession::page_objects` — the
/// engine's own model, against which every geometry verb resolves an index —
/// must describe the same page.
///
/// For the life of row O64 they did not: the shell gained the image and the
/// engine did not, so the shell selected index N of an (N+1)-object model and
/// the engine refused it as out of range. No pointer, no raster, no race —
/// the inequality WAS the defect, and the equality is now the guarantee.
#[test]
fn the_engine_sees_the_content_this_session_added() {
    let doc = fixture();
    let mut session = EditSession::new(doc);

    let before = session
        .page_objects(0)
        .expect("page 0 decomposes before the edit")
        .objects
        .len();

    let png = tiny_png();
    let image = pdfcer_core::image_import::import(&png).expect("a 1x1 PNG must import");
    let rect = pdfcer_core::page_tree::Rect {
        llx: 100.0,
        lly: 100.0,
        urx: 200.0,
        ury: 200.0,
    };
    let spec = pdfcer_core::edit::NewImage::new(0, rect, &image);
    session.add_image(&spec).expect("add_image must succeed");

    // The shell's view: overlay-aware, and this is exactly the call
    // `crate::app::cache::ensure_page_objects` makes.
    let shell_count = {
        let overlay_pages = session.pages().expect("the overlay page tree walks");
        let page = overlay_pages.first().expect("page 0 exists in the overlay");
        let view = session.view();
        pdfcer_core::vector::decompose_page(
            &view,
            page,
            pdfcer_core::vector::Matrix::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
        )
        .expect("the overlay page decomposes")
        .objects
        .len()
    };

    // The engine's view: base-derived, and this is what every geometry verb
    // resolves `object_index` against.
    let engine_count = session
        .page_objects(0)
        .expect("page 0 still decomposes after the edit")
        .objects
        .len();

    assert_eq!(
        shell_count,
        before + 1,
        "the SHELL must see the image it just inserted — if this fails the \
         defect has moved and O64's diagnosis is wrong"
    );

    assert_eq!(
        shell_count, engine_count,
        "★★★ THE SKEW IS BACK. These two models must describe the same page: \
         the shell selects an index in ITS model and hands it to a verb that \
         resolves the index in the ENGINE's, so a disagreement of one is an \
         edit applied to the wrong object or refused as out of range. Fixed \
         by the engine in Pass 186.0, 2026-08-31."
    );

    assert_eq!(
        engine_count,
        before + 1,
        "and both must SEE the image, not merely agree with each other — two \
         models that had each missed it would satisfy the equality above and \
         the operator would still be unable to move what he just placed"
    );
}

/// ★★ **And the consequence, driven through the verb the operator's drag
/// actually calls.**
///
/// Moving a placed image goes through `MoveSubject::Transform` →
/// `VectorAction::TransformObjects` → `EditSession::transform_objects`. This
/// asserts that call refuses, and names the error, so the request can quote
/// it rather than describe it.
#[test]
fn a_just_inserted_image_can_be_transformed() {
    let doc = fixture();
    let mut session = EditSession::new(doc);

    let before = session
        .page_objects(0)
        .expect("page 0 decomposes before the edit")
        .objects
        .len();

    let png = tiny_png();
    let image = pdfcer_core::image_import::import(&png).expect("a 1x1 PNG must import");
    let rect = pdfcer_core::page_tree::Rect {
        llx: 100.0,
        lly: 100.0,
        urx: 200.0,
        ury: 200.0,
    };
    let spec = pdfcer_core::edit::NewImage::new(0, rect, &image);
    session.add_image(&spec).expect("add_image must succeed");

    // `before` is the paint-order index the shell computes for the new image:
    // it selects `objects.len() - 1` of its own (N+1)-object model.
    let outcome = session.transform_objects(
        0,
        &[before],
        pdfcer_core::vector::Matrix::translate(10.0, 0.0),
        pdfcer_core::vector::TransformOptions::default(),
    );

    let outcome = outcome.expect(
        "★★★ MOVING A JUST-INSERTED IMAGE MUST WORK — the operator's own \
         report, and it was an engine defect rather than a shell one: the \
         verb resolved its page against the base document instead of the \
         session overlay, so the object the shell had just added did not \
         exist as far as the verb was concerned. Fixed in Pass 186.0, \
         2026-08-31. A refusal here means it has come back",
    );

    // ★ The COUNT, not merely the `Ok`. A verb that transformed nothing and
    // reported success would satisfy `is_ok()` and would be the same defect
    // wearing another face: the operator drags the image and it does not
    // move.
    assert_eq!(
        outcome.objects_transformed, 1,
        "one object was named and one must have moved: {outcome:?}"
    );
}

/// ★★★ **The half nobody had reported, and the one with teeth: after a page
/// is deleted, the engine's content verbs address a DIFFERENT SHEET.**
///
/// `delete_pages` commits into the overlay, so `EditSession::pages()` returns
/// three pages while `page_tree::pages(&self.base)` — which every geometry
/// verb and `EditSession::page_objects` read — still returns four. The shell
/// computes a page index against the overlay and hands it to a verb that
/// resolves it against the base.
///
/// Two consequences, and the second is why this is filed as urgent rather
/// than as a nuisance:
///
/// 1. **An index the document no longer has still resolves.** Asking for
///    page 3 of a three-page document must be `PageOutOfRange`. It is not.
/// 2. **Therefore an index the document DOES have resolves to the wrong
///    sheet.** Delete page 0, then move or delete an object on what the
///    operator sees as page 0, and the verb edits the page that used to be
///    page 0 — a different sheet — and returns `Ok`. Nothing refuses, nothing
///    discloses, and the wrong drawing is changed.
///
/// This test asserts (1), because it is the crisp one: a count mismatch needs
/// no fixture with distinguishable content and cannot be argued with. (2)
/// follows from it arithmetically and is stated in the request.
#[test]
fn after_a_page_is_deleted_the_old_page_set_is_gone() {
    let doc = fixture_four_pages();
    let mut session = EditSession::new(doc);

    let before = session.pages().expect("the page tree walks").len();
    assert_eq!(before, 4, "the fixture is a four-page document");

    session
        .delete_pages(&[0])
        .expect("deleting the first page must succeed");

    let overlay = session.pages().expect("the page tree walks").len();
    assert_eq!(overlay, 3, "the OVERLAY correctly has three pages left");

    // The engine's own content model, asked for a page the document no
    // longer has. It must refuse. On the current engine it does not, because
    // it is looking at the base document, which still has four.
    let out_of_range = session.page_objects(3);

    let Err(refusal) = out_of_range else {
        panic!(
            "★★★ THE WRONG-SHEET DEFECT IS BACK, and it is the one with \
             teeth. Page 3 of a THREE-page document resolved. If an index the \
             document no longer has still resolves, then every index it DOES \
             have resolves to the sheet that used to carry it — so an edit \
             made after a page deletion changes a different drawing, returns \
             `Ok`, and discloses nothing. Fixed by the engine in Pass 186.0."
        );
    };
    let text = format!("{refusal:?}");
    assert!(
        text.contains("PageOutOfRange"),
        "the refusal must be about the PAGE INDEX. Any other refusal means \
         the page resolved and something else went wrong, which is a \
         different subject and must not be read as this one being fixed. \
         Got: {text}"
    );
}

/// The four-page fixture, used only by the page-index test above.
fn fixture_four_pages() -> pdfcer_core::document::Document {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/four-pages.pdf");
    pdfcer_core::document::Document::load(std::path::Path::new(path))
        .expect("fixture four-pages.pdf must load")
}
