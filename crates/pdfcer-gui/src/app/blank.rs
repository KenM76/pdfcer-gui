//! # `app::blank` — where a new document comes from, and why it is a file
//!
//! `file.new` (`RIBBON_IA.md` §5.1, the File ▸ File band) makes a blank
//! document. This module holds the 443 bytes it makes it *out of*, the
//! decisions behind them, and nothing else — the lifetime transition itself is
//! [`crate::app::PdfcerApp::new_document`]'s, beside `open_path` and
//! `close_document`, because that is one subject and this is another.
//!
//! ## ★ 1. The engine cannot create a document, and that is deliberate
//!
//! Every constructor on `pdfcer_core::document::Document` — `load`,
//! `from_bytes`, and their password and options variants — **parses existing
//! PDF bytes**. `EditSession` can rotate, delete and reorder pages
//! (`rotate_pages`, `delete_pages`, `reorder_pages`) and has no verb that
//! *creates* one. `pageops::insert` and `pageops::merge` take pages only from
//! an already-loaded `DocumentView`. There is no path anywhere in the engine
//! that conjures a page from nothing.
//!
//! The obvious response is to ask pdfcer for a `Document::blank(…)`. **Do not
//! file that request.** The engine's `document` module header states the
//! reason as a named, permanent invariant:
//!
//! > `Document` is simultaneously the parse result AND […] the write source
//! > […]. **No separate builder/generation model may ever be introduced** —
//! > the audited prior art shows exactly how that bifurcation forecloses
//! > round-trip editing.
//!
//! A blank-document constructor is a generation model. Asking for one is
//! asking the engine to break the invariant that decides its architecture, and
//! it would be refused — which is the standing rule in miniature: **verify a
//! claim about the engine against the engine's source before filing it**, and
//! the verification is worth as much when it stops a filing as when it
//! corrects one.
//!
//! The engine's own tests reach a blank document by writing minimal PDF bytes
//! and parsing them back (`edit.rs`'s `blank_page_doc`, over
//! `pageops::tests_support::build_pdf_bytes`). That builder is `#[cfg(test)]`
//! and `pub(crate)`, with its own note saying why: *"a builder that produces
//! deliberately-minimal PDFs is a testing tool, not part of the engine's API,
//! and exposing it would invite production code to construct documents outside
//! the one object model."*
//!
//! ## ★ 2. So New opens a file, which is the thing this shell already does
//!
//! [`TEMPLATE`] is a real PDF, authored once, checked in, and compiled into the
//! binary. `file.new` hands it to `Document::from_bytes` and marks the result
//! as having no path. **The shell authors no PDF bytes in code**, the engine
//! gains no verb, and the whole of New's implementation is the open path with
//! the source swapped from a disk read to a slice.
//!
//! That matters beyond convenience. A hand-built byte string in a `const` is a
//! second PDF writer living inside the GUI, which is exactly what the engine's
//! invariant refuses on the other side of the boundary. A file is inspectable,
//! diffable, openable in Acrobat, and covered by
//! `tools/gates/check-shipped-assets.py`; a `const [u8]` is none of those.
//!
//! The asset ships under `assets/PROVENANCE.md` as **own work under MIT**,
//! which exempts it from that gate's notice surfaces (checks 4 and 5) and from
//! nothing else. Read that note before touching the bytes; the cross-reference
//! table stores absolute offsets and the file is not hand-editable.
//!
//! ## ★ 3. The page is A4, and Letter was rejected on the evidence
//!
//! The standing instruction: *match what Inkscape, Acrobat and SolidWorks do
//! — but first ask which of them actually has the surface.*
//!
//! **All three have this surface**, which is unusual, so the head-count is
//! worth having:
//!
//! | application | what its New does about size |
//! |---|---|
//! | **Acrobat** | creates the blank page immediately at a **locale default** — A4 under ISO locales, Letter under US ones. It does not ask. |
//! | **Inkscape** | `Ctrl+N` creates from the **default template**, which ships as **A4**. It does not ask. A size chooser exists and is a *different command*, `Ctrl+Alt+N`. |
//! | **SolidWorks** | `Ctrl+N` **does** ask — but what it asks is *which kind of document* (part / assembly / drawing). The sheet-size question comes second and only for drawings. |
//!
//! Two decisions fall out of that table and they are separate:
//!
//! **New does not ask.** Two of the three create immediately from a default;
//! the third asks a question — *what kind of document is this* — that pdfcer has
//! no analogue for, because every pdfcer document is the same kind. A dialog
//! offering one control would be SolidWorks' shape with SolidWorks' content
//! removed. Inkscape's split is the model followed: the plain verb makes a
//! document, and choosing a size is a **separate command** for later. See
//! `crate::shell::manifest::PLANNED`'s `file.new_from_template` row.
//!
//! **The default is A4.** Inkscape ships A4. Acrobat produces A4 on any metric
//! locale and Letter only on the US branch. And the operator's own documents —
//! the test set this project is measured against — are **A3 and A1 SolidWorks
//! drawing sheets**, which is A-series evidence, not Letter evidence: an
//! operator whose sheets are A3 and A1 has an A-series drawer, an A-series
//! plotter and A-series habits. Letter is reachable only through the US-locale
//! branch of one of the three, and this shell has no locale question to ask.
//!
//! So A4 wins on two of three plus the operator's own corpus, and it is
//! recorded here rather than assumed. What is *not* claimed is that A4 is the
//! right size for this operator's next new sheet — it very plausibly is not.
//! That is what the size picker is for, and it is a follow-up row rather than a
//! silent guess dressed up as a default.
//!
//! ### ★ 3a. The size picker is one asset and one engine verb, never ten assets
//!
//! §2 forbids authoring PDF bytes at runtime, so a shell that cannot ask the
//! engine to resize a page has exactly one implementation open to it: **one
//! checked-in template asset per size**. Counted honestly that is A0–A4 = five,
//! doubled to **ten** because a drafting sheet is landscape, more again for
//! ANSI — **and a custom size is impossible at any count.** An operator who
//! needs 900 × 600 is not served by twenty assets.
//!
//! ⇒ **A half-capability that forecloses the real fix is worse than no
//! capability**, and that is the reusable half of this argument. The answer is
//! an engine verb: `EditSession::set_media_box` and `set_media_boxes`, with
//! `pdfcer_core::paper` for the named sizes, which buys one asset and one
//! dialog — every size, both orientations, custom included, with [`TEMPLATE`]
//! unchanged. [`document_sized`] is that implementation.
//!
//! The semantics the shell needs are deliberately narrow — whether content
//! moves, whether `/CropBox` follows, whether shrinking below content is a
//! refusal are the engine's questions, and the narrow answer (*"only on a page
//! with no content"*) is **entirely sufficient** here: `file.new`'s page is
//! empty by construction.
//!
//! ## ★ 4. What a document with no file is, and what must not happen to it
//!
//! `crate::app::state::OpenDoc::path` means *where this came from*. A created
//! document came from nowhere, so its path is a **name** —
//! `crate::text::files::untitled` — and
//! [`crate::app::state::OpenDoc::stored_under`] is the one predicate that tells
//! the two apart. Three things consult it, and each would be a real defect
//! without it:
//!
//! - the **recent list** must not gain a row for a file that does not exist;
//! - the **remembered page display** must be neither read nor written under a
//!   fabricated path;
//! - the **guides store** likewise.
//!
//! Everything else in the shell treats `path` as an identity or a label — the
//! forms cache key, the Pages panel caption, the trace — and all of those are
//! correct for a name. See the field's own documentation.
//!
//! ## ★ 5. A new document CAN be saved
//!
//! **Save a copy is a shell task, not an engine gap.** Both engine write verbs
//! take `&self`, so an `Arc<EditSession>` can call either;
//! `crate::app::files::pick_save_path` supplies the picker and its diagnostic
//! seam; and the one decision left is which writer — incremental preserves
//! superseded content and any existing signature, a full rewrite destroys the
//! signature.
//!
//! `file.save_copy` takes the **incremental** writer, because the command's own
//! shipped tooltip promises it in words on an operator-visible surface.
//! `crate::app::save` §1 carries the argument.
//!
//! What that means for New specifically: a created document saves like any
//! other, and the copy it writes is the 443-byte template plus an appended
//! revision carrying whatever the operator authored on it. Two things about it
//! are New's own and are argued at `crate::app::save::suggested_path`:
//!
//! * the suggested name is the document's own — `Untitled 1.pdf`, with **no**
//!   `-copy` suffix, because there is no original to avoid overwriting;
//! * saving does **not** give the document a file. `path` stays `Untitled
//!   1.pdf`, `origin` stays [`crate::app::state::Origin::Created`], no Recent
//!   row appears, and no per-document preference is stored — because this is
//!   Save a *copy*, not Save As. `OpenDoc::origin`'s own note that *"a created
//!   document that gains a file gains it through a save"* still stands, and
//!   still refers to a `file.save_as` this build does not have.
//!
//! What is still absent is in-place `file.save`, blocked on autosave and crash
//! recovery in `crate::shell::manifest::PLANNED`.

use pdfcer_core::document::Document;
use pdfcer_core::page_tree::Page;

/// **The blank document, as bytes.**
///
/// 443 bytes: one A4 page with an empty content stream, a classic
/// cross-reference table, and nothing else. `assets/PROVENANCE.md` documents
/// every object in it and why each is shaped the way it is.
///
/// `include_bytes!` rather than a read at start-up, for two reasons that both
/// bite in the field: a portable folder whose template file was deleted would
/// produce a New that fails on a machine nobody can see, and a template that
/// can be replaced on disk is a template whose bytes are not the bytes the
/// tests pinned.
pub const TEMPLATE: &[u8] = include_bytes!("assets/blank-a4.pdf");

/// The template page's width in PDF units. ISO 216 A4: 210 mm at 72/inch.
///
/// Public so the test below can assert the *asset* matches the *decision*
/// rather than merely matching itself. A constant compared against nothing is
/// documentation; compared against the parsed `MediaBox` it is a check.
pub const WIDTH_PT: f64 = 595.276;

/// The template page's height in PDF units. ISO 216 A4: 297 mm at 72/inch.
pub const HEIGHT_PT: f64 = 841.89;

/// **Parse [`TEMPLATE`] into a document and its page vector.**
///
/// The same two steps `PdfcerApp::open_path` performs on a file, in the same
/// order, so a created document reaches `OpenDoc` through the identical
/// pipeline an opened one does. Nothing here is a shortcut around the engine.
///
/// # Errors
///
/// The engine's own message, ready to be shown by
/// `crate::text::open_failed`. **Unreachable in a correct build** — the bytes
/// are compiled in and [`tests::the_template_parses_and_holds_exactly_one_page`]
/// pins that they parse — but returned rather than unwrapped, because the
/// state it would describe is "this binary was built with a corrupt asset",
/// and an operator meeting that deserves a sentence rather than a stack trace.
pub fn document() -> Result<(Document, Vec<Page>), String> {
    let doc = Document::from_bytes(TEMPLATE.to_vec()).map_err(|err| err.to_string())?;
    let pages = pdfcer_core::page_tree::pages(&doc).map_err(|err| err.to_string())?;
    Ok((doc, pages))
}

/// The blank document, **resized** to `rect` before anybody sees it.
///
/// # ★ Why this exists at all
///
/// The module header's §3a sets out the alternative and why it is refused: one
/// checked-in template asset per size — ten with landscape, more with ANSI,
/// *and a custom size impossible at any count*. This function is the other
/// implementation, resting on `EditSession::set_media_box` and
/// `pdfcer_core::paper`: one asset and one dialog, every size, both
/// orientations, custom included, with [`TEMPLATE`] unchanged.
///
/// # ★ Why the document is serialized and re-parsed rather than handed over
///
/// The obvious implementation is to build the [`EditSession`], resize page 0,
/// and give the caller the session. **That produces a document that is already
/// modified**: the undo stack holds one command, `save_pending()` is true, and
/// `Ctrl+Z` on a brand-new A1 sheet takes the operator back to A4 — a state
/// they never asked for and cannot name.
///
/// A new document is not an edited document. So the resize happens *before the
/// document exists as far as the shell is concerned*: the bytes are rewritten
/// with [`EditSession::to_full_bytes`] and parsed back, and what the caller
/// receives is an ordinary freshly-parsed `Document` whose page simply is that
/// size. Nothing is pending, nothing is undoable, and
/// [`crate::app::lifecycle`] needs no special case.
///
/// The cost is one save and one parse of a ~450-byte file, which is not
/// measurable. The cost of the alternative is a permanent oddity in the undo
/// stack of every created document.
///
/// # Why `to_full_bytes` rather than `to_incremental_bytes`
///
/// An incremental save appends a revision, so the result would carry the A4
/// original *and* the resize — a two-revision file for a document that has no
/// history worth keeping and no signature to preserve. `to_full_bytes` writes
/// one revision. The engine's warning on it — that it destroys existing
/// digital signatures — cannot apply: the input is [`TEMPLATE`], which has
/// none.
///
/// # ★ `SaveOptions::identity()`, and why this is not a hole in the funnel
///
/// `crate::app::settings`' funnel exists because an option struct built at a
/// call site discards every setting the operator chose, and a test parses this
/// crate's syntax tree to enforce it. This call site is exempt, and the
/// argument is not "it is only a template" — it is that **no operator-visible
/// byte of this rewrite survives**.
///
/// `SaveOptions` has three fields. Two of them, `xref_entry_eol` and
/// `trailing_eol`, are byte-level spellings of the *written file*, and this
/// file is parsed back and discarded within the same statement — the document
/// the operator eventually saves is written by `crate::app::save`, which does
/// read their settings. The third, `producer`, writes `/Producer` into an
/// **existing** `/Info` dictionary and explicitly does not create one; the
/// template has no `/Info` (read it — it is 443 bytes and hand-legible), so
/// the policy is inert here whichever way it is set.
///
/// `identity()` rather than `default()` regardless, because it is the value
/// that promises to change nothing, and a future template that grew an `/Info`
/// would then be preserved rather than silently stamped. That is R41's rule —
/// pdfcer does not write its own identity into a file the operator did not ask
/// it to mark — and a new document's construction is not an act of authorship
/// the operator directed at a file.
///
/// # Errors
///
/// A `String` for the caller to show, in three cases that are all worth
/// telling apart in the message rather than in a type:
///
/// - the template did not parse — a **build defect**, the same unreachable arm
///   [`document`] has;
/// - the size was degenerate — `EditError::MediaBoxDegenerate`, reachable from
///   a custom size the operator typed, which is why the dialog checks before
///   it asks for one;
/// - the rewrite failed — unreachable for a 443-byte unencrypted file with no
///   hybrid cross-reference, and reported rather than unwrapped.
pub fn document_sized(rect: pdfcer_core::page_tree::Rect) -> Result<(Document, Vec<Page>), String> {
    let base = Document::from_bytes(TEMPLATE.to_vec()).map_err(|err| err.to_string())?;
    let mut session = pdfcer_core::edit::EditSession::new(base);
    let change = session
        .set_media_box(0, rect)
        .map_err(|err| err.to_string())?;
    let (bytes, _report) = session
        .to_full_bytes(&pdfcer_core::writer::SaveOptions::identity())
        .map_err(|err| err.to_string())?;

    let written = bytes.len();
    let doc = Document::from_bytes(bytes).map_err(|err| err.to_string())?;
    let pages = pdfcer_core::page_tree::pages(&doc).map_err(|err| err.to_string())?;

    // ★ Traced AFTER the re-parse, and reporting the page as the RE-PARSED
    // document states it rather than the rectangle that was asked for.
    //
    // The distinction is the whole value of the line. A trace of the request
    // says what this function was told; a trace of `pages[0].media_box` says
    // what a reader of the resulting file will see, which is what the operator
    // gets and the only thing worth asserting from outside the process.
    // `ui-verify`'s `new_document_sizes_the_page` reads `result_w`/`result_h`
    // for exactly that reason — a build that recorded the request and wrote
    // nothing would have a perfect `w=`/`h=` and a 595 × 842 page.
    let media = pages.first().map(|page| page.media_box);
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "new-document-sized w={:.2} h={:.2} change={change:?} bytes={} \
             result_w={:.2} result_h={:.2}",
            rect.width(),
            rect.height(),
            written,
            media.map_or(0.0, |m| m.urx - m.llx),
            media.map_or(0.0, |m| m.ury - m.lly),
        )
    });

    Ok((doc, pages))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **The compiled-in template really is a document.**
    ///
    /// The one assertion that makes [`document`]'s error arm unreachable, and
    /// therefore the one that lets `file.new` be described as a command that
    /// cannot fail. Without it the claim would rest on the asset having been
    /// correct on the day it was written.
    #[test]
    fn the_template_parses_and_holds_exactly_one_page() {
        let (_doc, pages) = document().expect("the compiled-in template must parse");
        assert_eq!(pages.len(), 1, "New makes a one-page document");
    }

    /// ★ **The page is A4, to the tenth of a point.**
    ///
    /// This is the decision in §3 of the module header being *checked* rather
    /// than merely written down. A future edit that regenerated the asset at
    /// Letter — 612 × 792, which is what most minimal-PDF recipes on the
    /// internet carry, including the engine's own `blank_page_doc` fixture —
    /// fails here, naming both numbers, rather than shipping a silently
    /// different default.
    #[test]
    fn the_template_page_is_a4() {
        let (_doc, pages) = document().expect("the template parses");
        let media = pages[0].media_box;
        let width = media.urx - media.llx;
        let height = media.ury - media.lly;
        assert!(
            (width - WIDTH_PT).abs() < 0.1 && (height - HEIGHT_PT).abs() < 0.1,
            "the template is {width} x {height} pt; A4 is {WIDTH_PT} x {HEIGHT_PT}. \
             If this default was changed deliberately, change `app::blank`'s header \
             argument with it — the reasoning is what makes the number defensible."
        );
    }

    /// ★ **The page has a content stream, empty though it is.**
    ///
    /// A page with no `/Contents` is legal (§7.7.3.3) and would render
    /// identically — which is exactly why this needs an assertion rather than
    /// an eyeball. Every real producer emits a content stream, so a template
    /// without one would exercise a renderer path no other document in this
    /// project takes, and would prove less than it appears to on the day
    /// somebody uses New to reproduce a rendering defect.
    #[test]
    fn the_template_page_carries_a_content_stream() {
        let (_doc, pages) = document().expect("the template parses");
        assert_eq!(
            pages[0].contents.len(),
            1,
            "the blank page must carry exactly one (empty) content stream"
        );
    }

    /// The asset stays a template rather than becoming a document.
    ///
    /// Not a change-detector: the failure it guards against is somebody
    /// "improving" the template by embedding a font, a logo or a title block,
    /// which would make every new document carry bytes the operator did not
    /// ask for and would quietly move this directory out of the own-work
    /// provenance it is declared under. Two kilobytes is roughly four times
    /// the honest size and nowhere near a single embedded face.
    #[test]
    fn the_template_is_still_a_few_hundred_bytes() {
        assert!(
            TEMPLATE.len() < 2048,
            "the blank template is {} bytes; it was 443. Anything that big is \
             carrying content, and `assets/PROVENANCE.md` describes a file that \
             carries none.",
            TEMPLATE.len()
        );
    }
}
