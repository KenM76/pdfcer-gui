//! # `app::cache` — the three derived values a document is worth keeping, and why they live on it
//!
//! ## What is in here
//!
//! Three caches, and the [`OpenDoc`] methods that read them:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/cache.md`.

pub(crate) mod provenance;

use std::cell::{Cell, Ref, RefCell};
use std::time::Instant;

use pdfcer_core::annot::PageLinks;
use pdfcer_core::fontinfo::FontInventory;
use pdfcer_core::outline::DestinationReader;
use pdfcer_core::text_extract::PageText;

use crate::app::settings::SettingsExt;

use crate::app::state::OpenDoc;
use crate::panels::objects::provider::ObjectModelProvider;

/// The page decomposition, held for as long as the document is open.
#[derive(Default)]
pub(in crate::app) struct PageObjectCache {
    /// The `(page index, edit epoch)` the decomposition below describes, or
    /// `None` before the first attempt.
    ///
    /// Both halves are needed, and for the same reason
    /// `OpenDoc::objects_traced_for` needs both: a decomposition is a property
    /// of **this page** in **this revision**. Paging away and back must
    /// rebuild (different content), and an edit must rebuild (the objects
    /// moved).
    ///
    /// Notice what is *not* in it: any document identity. There is none to
    /// carry, because opening a document constructs a whole new [`OpenDoc`]
    /// and this cache dies with the old one. That is the point of the move —
    /// see [`OpenDoc::page_objects`].
    pub(in crate::app) built_for: Cell<Option<(usize, u64)>>,
    /// The decomposition, or the reason the page would not decode.
    ///
    /// `None` means "not attempted". `Some(Err(_))` means "attempted and
    /// failed", which is a **different state**: the failure is deterministic
    /// (same bytes, same code), so a page whose content will not decode must
    /// not be re-decomposed on every frame. That is the same reasoning the
    /// render-error hold in `PdfcerApp::settle_and_rasterize` uses.
    provider: RefCell<Option<Result<ObjectModelProvider, String>>>,
}

/// The current page's **extracted text**, held for as long as the document is
/// open.
///
/// # Why this cache exists
///
/// `crate::find`'s header records the trap in its own words:
///
/// > `EditSession::find_text_with` runs `text_extract::extract_document_view`
/// > over the **whole document** on every call […] There is no cache in
/// > `pdfcer-core` and none here.
///
/// That costs a measurable fraction of a second on the project's benchmark
/// sheet — `crate::find`'s own `find … ms=` trace line is how it is measured
/// — which is why Find never searches on a keystroke. Canvas text selection
/// cannot pay that: a drag is sixty frames a second, and each frame has to
/// know which glyphs the pointer has swept over.
///
/// Two things make it affordable, and both are choices rather than luck:
///
/// 1. **One page, not the document.** `pdfcer_core::text_extract` publishes a
///    per-page twin of every entry point — [`extract_page_view`] beside
///    `extract_document_view` — and it exists for exactly this consumer: its
///    own doc comment names *"the GUI's in-place text-edit model and Copy
///    Text"*. A selection is a range on **one** page (see
///    `crate::canvas::textsel`'s header §4 on why it does not cross pages), so
///    the whole-document walk is not merely expensive here, it is answering a
///    question nobody asked.
/// 2. **Keyed on `(page, edit epoch)`**, exactly as [`PageObjectCache`] is, so
///    a drag pays for the first frame and hits the cache for the rest of the
///    gesture. Panning, zooming and scrolling never touch it at all, because
///    nothing asks for it unless a text gesture is live — see
///    `canvas::interact`'s step 4a.
///
/// [`extract_page_view`]: pdfcer_core::text_extract::extract_page_view
///
/// # The revision is the SESSION's, and that is not the same choice Find made
///
/// `extract_page_view` takes a `DocumentView`, and core makes the revision the
/// caller's explicit decision (decision 018 §8) precisely because
/// the two consumers want different answers: *"What does this FILE say?"*
/// against the base document, *"What does the page IN FRONT OF ME say?"*
/// against the session.
///
/// This is the second question. The operator is dragging across glyphs they can
/// **see**, and after one accepted edit the base revision describes a page that
/// is no longer on screen — a selection resolved against it would highlight one
/// set of glyphs and copy another. So this passes `self.session.view()`, which
/// is the same choice [`OpenDoc::page_objects`] makes and for the same stated
/// reason (decision 018).
///
/// # Why the failure is kept rather than collapsed to `None`
///
/// Same argument as [`PageObjectCache::provider`]: `None` means *not
/// attempted*, `Some(Err(_))` means *attempted and failed*, and the failure is
/// deterministic — same bytes, same code — so a page whose content stream will
/// not tokenize must not be re-walked sixty times a second.
#[derive(Default)]
pub(in crate::app) struct PageTextCache {
    /// The `(page index, edit epoch)` the extraction below describes, or
    /// `None` before the first attempt. See [`PageObjectCache::built_for`] for
    /// the borrow argument this `Cell` is half of — it is the same two-field
    /// shape for the same reason.
    pub(in crate::app) built_for: Cell<Option<(usize, u64)>>,
    /// The extraction, or the engine's own reason it would not run.
    pub(in crate::app) text: RefCell<Option<Result<PageText, String>>>,
}

/// **Which of the current page's runs have no show operator of their own** -
/// the editability answer, cached because the question is asked on every click
/// that lands on text.
#[derive(Default)]
pub(in crate::app) struct LinkCache {
    /// The edit epoch [`Self::reader`] was built at.
    pub(in crate::app) reader_for: Cell<Option<u64>>,
    /// The document-wide destination resolver. See the type's docs.
    pub(in crate::app) reader: RefCell<Option<DestinationReader>>,
    /// The `(page index, edit epoch)` [`Self::links`] describes.
    pub(in crate::app) built_for: Cell<Option<(usize, u64)>>,
    /// That page's links, resolved.
    pub(in crate::app) links: RefCell<Option<PageLinks>>,
}

#[derive(Default)]
pub(in crate::app) struct FormRunCache {
    /// The `(page index, edit epoch)` the flags below describe.
    pub(in crate::app) built_for: Cell<Option<(usize, u64)>>,
    /// One flag per run: `true` when that run has **no show operator of its
    /// own** and therefore nothing for the text surgery to anchor on.
    ///
    /// **It does not mean "inside a form XObject".** Form content is
    /// editable, and reading this flag as the form set refuses a caret on 99 %
    /// of the text on a CAD drawing — the operator's own estimate.
    ///
    /// What it means is the `/ActualText` case: the producer supplied a
    /// replacement string for a span of glyphs, so the run covers no show
    /// operator a pinned span could name.
    ///
    /// `None` means the extraction did not run or provenance was unavailable -
    /// which the caller must read as **"not measured"**, never as "yes". A
    /// refusal on an unmeasured answer would block text editing everywhere on
    /// a guess, and would look exactly like the feature having been removed.
    pub(in crate::app) flags: RefCell<Option<Vec<bool>>>,
}

/// The document's font inventory, held for as long as the document is open.
#[derive(Default)]
pub(in crate::app) struct FontCache {
    /// The `edit_epoch` the inventory below describes, or `None` before the
    /// first build. See [`PageObjectCache::built_for`] for the borrow
    /// argument this `Cell` is half of.
    pub(in crate::app) built_for: Cell<Option<u64>>,
    /// The inventory. `pdfcer_core::fontinfo::inventory` is **infallible** —
    /// it reports problems in its `diagnostics` rather than in a `Result`
    /// (core API trap T-9.8) — so there is no error arm here, and an empty
    /// inventory does not mean a clean document.
    inventory: RefCell<Option<FontInventory>>,
}

impl OpenDoc {
    /// The current page's decomposition, building it on first use.
    #[must_use]
    pub fn page_objects(&self) -> Option<Ref<'_, ObjectModelProvider>> {
        self.ensure_page_objects();
        Ref::filter_map(self.page_objects.provider.borrow(), |slot| {
            slot.as_ref().and_then(|built| built.as_ref().ok())
        })
        .ok()
    }

    /// **Where the ink on this page actually reaches**, in PDF user space —
    /// or [`None`] if nobody has decomposed the page yet.
    ///
    /// `PageObjects::page_bbox()` unioned over the whole page, which
    /// `crate::render::offpage`'s `the_content_union_is_available_and_non_empty`
    /// asserts includes geometry **outside** the crop box. That is the input
    /// [`crate::render::halo::region`] turns into "how big a raster do I need
    /// so the operator can see what he placed off the sheet" — O23's "see"
    /// half.
    ///
    /// # It PEEKS. It does not build, and that is the whole point
    ///
    /// [`Self::page_objects`] builds on first use, and on the operator's own
    /// benchmark sheet that build is **469 ms**:
    ///
    /// ```text
    /// page-objects-built page=0 objects=129758 leaves=10256 ms=469
    /// ```
    ///
    /// `OpenDoc::trace_object_count` is gated behind `PDFCER_DIAG` for exactly
    /// that reason — *"nothing is built with tracing off"* — and the canvas
    /// runs on every frame. A halo that forced a decomposition from the render
    /// path would pay that half-second again after **every content edit**,
    /// which is `OPERATOR_REQUESTS.md` O74 at its most expensive point:
    /// *"the last thing that should matter is updating the preview."*
    ///
    /// So the canvas reads what is already there, and
    /// [`Self::ensure_content_bounds`] is called separately from
    /// `render::settle`, **after** the picture has landed. The consequence is
    /// visible and is the honest one: on the first frame after opening a huge
    /// drawing there is no halo, and one frame later there is. A picture that
    /// arrives a moment late beats an edit that stalls.
    ///
    /// The staleness check is the same `(page, content generation)` key
    /// [`Self::ensure_page_objects`] builds against, so a model built for the
    /// previous page — or from before a content edit — reads as "not known"
    /// rather than as a bounding box for the wrong picture.
    #[must_use]
    pub fn content_bounds_if_known(&self) -> Option<pdfcer_core::page_tree::Rect> {
        if self.page_objects.built_for.get()
            != Some((self.view.page_index, self.page_objects_revision()))
        {
            return None;
        }
        let slot = self.page_objects.provider.borrow();
        let bounds = slot.as_ref()?.as_ref().ok()?.page_objects().page_bbox();
        Some(pdfcer_core::page_tree::Rect::from_corners(
            bounds.min.x,
            bounds.min.y,
            bounds.max.x,
            bounds.max.y,
        ))
    }

    /// Build the decomposition now, so that [`Self::content_bounds_if_known`]
    /// can answer on the next frame.
    pub fn ensure_content_bounds(&self) {
        self.ensure_page_objects();
    }

    /// Why the current page would not decompose, if it would not.
    pub(in crate::app) fn page_objects_failure(&self) -> Option<Ref<'_, String>> {
        self.ensure_page_objects();
        Ref::filter_map(self.page_objects.provider.borrow(), |slot| {
            slot.as_ref().and_then(|built| built.as_ref().err())
        })
        .ok()
    }

    /// **What this page's decomposition is keyed on**, and it is not the edit
    /// epoch.
    ///
    /// # The 469 ms this removes, measured on the operator's own drawing
    ///
    /// ```text
    /// page-objects-built page=0 objects=129758 leaves=10256 ms=469
    /// ```
    ///
    /// That is one decomposition of the benchmark CAD sheet. Keyed on
    /// `edit_epoch` — which every mutating action bumps — it would be paid
    /// again after **every** edit, including edits that cannot have touched
    /// page content at all. Authoring a form field is an `/Annots` change; so
    /// is placing a stamp, a note, or a dimension. Each would freeze the
    /// window for about half a second to rebuild a model that had not changed,
    /// which is `OPERATOR_REQUESTS.md` O74 at its most expensive point:
    /// *"the last thing that should matter is updating the preview."*
    ///
    /// `EditSession::page_content_generation` is the engine's own digest of
    /// that page's content dependencies — page id, every `/Contents` entry with
    /// its staged span, the effective `/Resources`. It moves when content moves
    /// and holds still for an annotation, which is exactly the distinction the
    /// epoch cannot make.
    ///
    /// # …and its coverage is measured, because it is not obvious
    ///
    /// **Measured, not assumed** —
    /// `crates/pdfcer-gui/tests/page_generation_covers.rs`, three tests, one per
    /// dependency class:
    ///
    /// | class | generation |
    /// |---|---|
    /// | a content edit (`move_objects`) | moves |
    /// | an annotation edit (`add_markup`) | holds still ✓ the win |
    /// | **an edit inside a form XObject** (`move_node_in_form`) | moves |
    ///
    /// The third row is the one worth measuring rather than assuming: the
    /// digest folds in the descended-form set, which its name does not say.
    /// `PageObjects` addresses content **by index**, so a model that held
    /// still across a form edit would make the next drag edit whatever that
    /// index names in the *wrong* model — the engine's own phrase is *"silent
    /// corruption of the operator's drawing, reported as success"*.
    ///
    /// **The fallback is the epoch**, not a constant. If the generation
    /// cannot be read — a page index the session does not have, a document
    /// mid-close — this returns the epoch, which rebuilds on every edit exactly
    /// as before. Slow is the safe direction; a constant would freeze the model.
    fn page_objects_revision(&self) -> u64 {
        match self.content_generation.get() {
            // The digest, but ONLY while the epoch it was measured at is
            // still current. See `OpenDoc::content_generation`: a measurement
            // can be missed (the render worker holds the other `Arc` handle),
            // and a digest taken before an edit describes a page that has
            // since changed. The epoch stamp turns that from a silent
            // wrong-index edit into an extra rebuild.
            Some((page, epoch, generation))
                if page == self.view.page_index && epoch == self.edit_epoch =>
            {
                generation
            }
            _ => self.edit_epoch,
        }
    }

    /// **Measure the engine's content digest for the current page**, on a frame
    /// where this shell holds the session exclusively.
    pub(in crate::app) fn refresh_content_generation(&mut self) {
        let page = self.view.page_index;
        let epoch = self.edit_epoch;
        let Some(session) = std::sync::Arc::get_mut(&mut self.session) else {
            return;
        };
        if let Ok(generation) = session.page_content_generation(page) {
            self.content_generation.set(Some((page, epoch, generation)));
        }
    }

    /// Decompose the current page if the cache does not already describe it.
    fn ensure_page_objects(&self) {
        let key = (self.view.page_index, self.page_objects_revision());
        if self.page_objects.built_for.get() == Some(key) {
            return;
        }
        self.page_objects.built_for.set(Some(key));
        let built = self.current_page().map(|page| {
            ObjectModelProvider::build_or_reason(
                // The SESSION view, never the base document's: the session
                // view is the edited state, which is the state the operator
                // is looking at and the state the canvas is drawing.
                // Decomposing the base revision would list objects the
                // operator has already removed and miss ones they have added
                // (decision 018).
                &self.session.view(),
                page,
                self.view.page_index,
            )
        });
        // A document with no such page is "no decomposition and no failure":
        // there is nothing to report a reason about. `page_objects` and
        // `page_objects_failure` both return `None`, and the caller above
        // already handles an empty document.
        *self.page_objects.provider.borrow_mut() = built;
    }

    /// **The current page's extracted text**, building it on first use.
    ///
    /// The one extraction. Canvas text selection resolves its range against
    /// this, the highlight's quads are derived from it, the copied string is
    /// sliced out of it, and `file.copy_page_text` writes it to the clipboard —
    /// all from *this* value, for the same reason [`Self::page_objects`] is the
    /// one decomposition. Two extractions of one page would be two chances for
    /// what is shown and what is copied to disagree, which is the single defect
    /// this feature was most likely to ship.
    ///
    /// # Returns
    ///
    /// `None` when there is no such page, or when the page's content stream
    /// cannot be walked. A caller says so rather than presenting an empty
    /// selection, because a page with no text and a page that would not parse
    /// need different responses from the operator. The reason is kept for the
    /// trace channel; see [`Self::page_text_failure`].
    ///
    /// # Cost, and how to re-measure it
    ///
    /// Every *build* writes one `PDFCER_DIAG` line:
    ///
    /// ```text
    /// pdfcer-diag page-text page=0 runs=412 chars=3106 ms=27 status=ok
    /// ```
    ///
    /// A cache **hit** writes nothing, so the number of these lines in a run is
    /// the number of extractions the session actually paid for — which is the
    /// measurement that matters, and the one a prose claim in this file would
    /// otherwise drift from. `crate::find`'s `find … ms=` line is the
    /// whole-document comparison to read it against.
    ///
    /// # Holding the `Ref`
    ///
    /// As [`Self::page_objects`]: the return keeps a shared borrow of `*self`
    /// alive, so a caller that needs `&mut OpenDoc` afterwards must drop it
    /// first. `canvas::interact` does, and says why.
    #[must_use]
    pub fn page_text(&self) -> Option<Ref<'_, PageText>> {
        self.ensure_page_text();
        Ref::filter_map(self.page_text.text.borrow(), |slot| {
            slot.as_ref().and_then(|built| built.as_ref().ok())
        })
        .ok()
    }

    /// **Every clickable link on `page_index`, and where each one goes.**
    #[must_use]
    pub fn page_links(&self, page_index: usize) -> Option<Ref<'_, PageLinks>> {
        self.ensure_page_links(page_index);
        Ref::filter_map(self.links.links.borrow(), Option::as_ref).ok()
    }

    /// Build [`Self::page_links`] for `(page_index, epoch)` if it is not
    /// already built. Idempotent, and two comparisons on every call after the
    /// first.
    fn ensure_page_links(&self, page_index: usize) {
        // The reader FIRST and on its own key. It is the O(document) half and
        // it survives a page turn; the links below do not. See [`LinkCache`].
        if self.links.reader_for.get() != Some(self.edit_epoch) {
            self.links.reader_for.set(Some(self.edit_epoch));
            // The SESSION view, never the base document's — the same rule
            // `ensure_page_objects` states at length. A reader built from the
            // base revision would resolve against the page order before the
            // operator's page deletes, which is precisely the stale-snapshot
            // failure `DestinationReader`'s own docs warn about.
            *self.links.reader.borrow_mut() = Some(DestinationReader::new(&self.session.view()));
            // Force a rebuild of the page list too: it was resolved against the
            // reader that has just been replaced.
            self.links.built_for.set(None);
        }
        let key = (page_index, self.edit_epoch);
        if self.links.built_for.get() == Some(key) {
            return;
        }
        // Recorded BEFORE the work, exactly as `ensure_page_text` does: a page
        // whose `/Annots` will not resolve fails deterministically, and
        // retrying it every frame would burn a core to learn the same thing.
        self.links.built_for.set(Some(key));
        let built = self.pages.get(page_index).and_then(|page| {
            let reader = self.links.reader.borrow();
            reader.as_ref().map(|reader| {
                pdfcer_core::annot::page_link_destinations(&self.session.view(), page.id, reader)
            })
        });
        if let Some(links) = &built {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // Emitted on a BUILD, never on a cache hit — so the number of
                // these lines in a run is the number of resolutions actually
                // paid for. That is the measurement a prose claim about cost
                // would otherwise drift from, and this file has the same note
                // on `page_text` for the same reason.
                format!(
                    "page-links page={page_index} links={} unresolvable={} named={}",
                    links.links.len(),
                    links.links_without_destination,
                    self.links
                        .reader
                        .borrow()
                        .as_ref()
                        .map_or(0, DestinationReader::named_destination_count),
                )
            });
        }
        *self.links.links.borrow_mut() = built;
    }

    /// **Does `run` on the current page have no show operator to anchor on?**
    #[must_use]
    pub fn run_has_no_anchor(&self, run: usize) -> Option<bool> {
        self.ensure_form_runs();
        let flags = self.form_runs.flags.borrow();
        flags.as_ref()?.get(run).copied()
    }

    /// Build [`Self::form_runs`] for the current `(page, epoch)` if it is not
    /// already built. Idempotent, and cheap on every call after the first.
    fn ensure_form_runs(&self) {
        let key = (self.view.page_index, self.edit_epoch);
        if self.form_runs.built_for.get() == Some(key) {
            return;
        }
        // Recorded here, before the work — but **the order is not what makes
        // a failed extraction cheap**, and it cannot be: moving the `set`
        // below the extraction changes nothing.
        //
        // The store at the bottom of this function runs whether the
        // extraction succeeded or not, so the attempt is recorded either
        // way and the next frame is a `Cell` read either way. The property
        // that stops a third of a second per frame being spent re-learning a
        // deterministic failure is that **the key is recorded on the failure
        // arm at all** — which an early `return` there would quietly undo.
        // `cache::provenance::ProvenanceTextCache::built_for` carries the
        // measurement and the test that catches it.
        self.form_runs.built_for.set(Some(key));
        let started = Instant::now();
        // **The shared extraction, not a private one.** A private
        // `with_provenance(true)` extract here would make a single click on a
        // text run with the Properties panel open pay for the same `PageText`
        // twice — once here, once inside `pin::inspect` — and throw one away.
        // `crate::app::cache::provenance` carries the measurement (392 ms
        // each, on the operator's benchmark sheet) and the argument for why
        // the run indices are identical either way.
        let built = self.provenance_page_text(self.view.page_index).map(|text| {
            // **The engine's own query**, `TextRun::editability`, rather than
            // a hand-rolled match on `GlyphProvenance::content_stream`. A
            // hand-rolled match is a shell encoding a fact about the surgery's
            // internals — precisely the workaround this project's own request
            // warned would outlive its bug:
            //
            // > *"the day form editing lands, my guard silently keeps refusing
            // > until I notice and delete it."*
            //
            // `editability()` answers `Editable` for form content, so the
            // shell tracks the engine rather than guessing at it; a guard of
            // its own would go on refusing carets on 99 % of the text on a CAD
            // drawing until somebody noticed. What is left is the case a
            // hand-rolled match cannot see at all: `NoAnchor`, an
            // `/ActualText` run covering no show operators, which has nothing
            // for the surgery to anchor on.
            //
            // `Unknown` is NOT treated as "no". It is the state a caller
            // reaches by default — provenance not captured — and the engine
            // made the type an enum rather than a `bool` specifically so it
            // cannot be confused with a measured refusal. We asked with
            // provenance on, so `Unknown` here means something went wrong with
            // the ask rather than with the run, and refusing every caret on it
            // would block text editing document-wide for a reason nobody
            // measured.
            use pdfcer_core::text_extract::Editability;
            text.runs
                .iter()
                .map(|run| matches!(run.editability(), Editability::NoAnchor))
                .collect::<Vec<bool>>()
        });
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            //
            // The count of these lines IS the measurement, as it is for
            // `page-text` - one line per extraction, so a harness can tell a
            // cache that works from one that does not. `no_anchor` beside
            // `runs` is what makes the refused set visible on a real
            // document.
            format!(
                "form-runs page={} ms={} runs={} no_anchor={}",
                self.view.page_index,
                started.elapsed().as_millis(),
                built.as_ref().map_or(0, Vec::len),
                built
                    .as_ref()
                    .map_or(0, |f| f.iter().filter(|b| **b).count()),
            )
        });
        *self.form_runs.flags.borrow_mut() = built;
    }

    /// **Does this page carry any extractable text at all?**
    #[must_use]
    pub fn page_has_extractable_text(&self) -> bool {
        self.page_text()
            .is_some_and(|text| !text.plain_text().trim().is_empty())
    }

    /// Why the current page's text would not be extracted, if it would not.
    pub(in crate::app) fn page_text_failure(&self) -> Option<Ref<'_, String>> {
        self.ensure_page_text();
        Ref::filter_map(self.page_text.text.borrow(), |slot| {
            slot.as_ref().and_then(|built| built.as_ref().err())
        })
        .ok()
    }

    /// Extract the current page's text if the cache does not already describe
    /// it.
    fn ensure_page_text(&self) {
        let key = (self.view.page_index, self.edit_epoch);
        if self.page_text.built_for.get() == Some(key) {
            return;
        }
        self.page_text.built_for.set(Some(key));
        let started = Instant::now();
        let built = self.current_page().map(|page| {
            // The SESSION view, never the base document's — see
            // `PageTextCache`'s header. The operator is dragging across glyphs
            // they can see, and the base revision may no longer describe them.
            //
            // Through the funnel, and NOT `ExtractOptions::default()`.
            //
            // This call site is why `crate::app::settings::SettingsExt` exists:
            // it is the extraction the canvas's text selection, the find bar
            // and `file.copy_page_text` all read, and a bare `::default()` here
            // silently discards three of the operator's settings — the word
            // gap, the unmappable sentinel and the replacement-text
            // precedence.
            //
            // `capture_provenance` stays off: it is the substrate for *editing*
            // text and this feature only reads it. `canvas::textedit` turns it
            // on with `.with_provenance(true)` **on top of** the funnel's
            // output, which is a modifier rather than a second construction.
            pdfcer_core::text_extract::extract_page_view(
                &self.session.view(),
                page,
                self.view.page_index,
                &self.settings.extract_options(),
            )
            .map_err(|e| e.to_string())
        });
        let elapsed = started.elapsed();
        if let Some(built) = &built {
            // Not de-duplicated through `trace_changed`: two extractions are
            // two events, and the count of these lines IS the measurement (see
            // `page_text`'s docs). A gate that silenced the second would make a
            // harness unable to tell a cache that works from one that does not.
            crate::diag::trace(|| match built {
                Ok(text) => format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "page-text page={} runs={} chars={} ms={} status=ok",
                    self.view.page_index,
                    text.runs.len(),
                    text.plain_text().len(),
                    elapsed.as_millis(),
                ),
                Err(reason) => format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "page-text page={} ms={} status=failed reason={reason:?}",
                    self.view.page_index,
                    elapsed.as_millis(),
                ),
            });
        }
        *self.page_text.text.borrow_mut() = built;
    }

    /// The document's font inventory, building it on first use.
    #[must_use]
    pub fn font_inventory(&self) -> Ref<'_, FontInventory> {
        if self.fonts.built_for.get() != Some(self.edit_epoch) {
            self.fonts.built_for.set(Some(self.edit_epoch));
            *self.fonts.inventory.borrow_mut() =
                Some(pdfcer_core::fontinfo::inventory(&self.session.view()));
        }
        Ref::map(self.fonts.inventory.borrow(), |slot| {
            // `inventory` is infallible and the block above has just filled
            // the slot for this epoch, so `None` is not a reachable state.
            slot.as_ref().expect("just built for this epoch") // ui-text-exempt: panic message, never displayed
        })
    }
}

impl OpenDoc {
    /// **Drop every value derived from a text extraction.**
    pub(crate) fn invalidate_derived_text(&mut self) {
        self.page_text.built_for.set(None);
        *self.page_text.text.borrow_mut() = None;
    }
}

#[cfg(test)]
mod tests {
    use crate::app::state::{FOUR_PAGES, PAINTED_LAYERS, open_fixture};

    // =======================================================================
    // The cache move — what replaced `panels::DocKey`
    // =======================================================================

    /// **The decomposition cache carries NO document identity, and does
    /// not need one.**
    #[test]
    fn a_documents_decomposition_cannot_outlive_the_document() {
        let mut doc = open_fixture(FOUR_PAGES);
        // The frame's own first step, performed here because this test is
        // about the KEY and the key's second half is measured there.
        //
        // `page_objects_revision` reads a digest that `app::frame` takes once
        // per frame at the one `&mut` point it has — a unit test that skipped
        // it would exercise the epoch fallback and assert nothing about the
        // digest, which is what this assertion is for.
        doc.refresh_content_generation();
        assert_eq!(doc.page_objects().expect("page 0").page_index(), 0);
        // The page, not the whole key: the second half is a content digest
        // and is a different number per fixture, which is exactly what this
        // test is about — that the key does not carry over to another
        // document.
        let first = doc.page_objects.built_for.get();
        assert_eq!(first.map(|(page, _)| page), Some(0));

        doc = open_fixture(PAINTED_LAYERS);
        doc.refresh_content_generation();
        assert_eq!(
            doc.page_objects().expect("the layer fixture").page_index(),
            0
        );
        let second = doc.page_objects.built_for.get();
        assert_eq!(
            second.map(|(page, _)| page),
            Some(0),
            "a fresh document starts un-built, whatever address it landed on"
        );
        // And the two documents' keys DIFFER, which is the property that
        // makes the previous line safe. Two documents whose page 0 happened to
        // share a key would serve one's decomposition for the other — the
        // failure this test is named for — and a page index alone cannot rule
        // it out.
        assert_ne!(
            first, second,
            "a second document must not inherit the first's cache entry"
        );
        assert_eq!(doc.pages.len(), 1, "and it is this document's page tree");
    }

    /// **A page step rebuilds the decomposition; so does an edit.**
    #[test]
    fn the_decomposition_is_rebuilt_when_the_page_or_the_revision_moves() {
        let mut doc = open_fixture(FOUR_PAGES);
        assert_eq!(doc.page_objects().expect("page 0").page_index(), 0);

        doc.view.page_index = 2;
        assert_eq!(
            doc.page_objects().expect("page 2").page_index(),
            2,
            "a page step must rebuild, or the panel lists another page's objects"
        );

        // An edit renumbers objects without moving page, so the revision is
        // the other half.
        //
        // Driven through a REAL content edit rather than by setting
        // `edit_epoch` by hand: the key does not read the epoch, so a test
        // that bumped it would assert nothing at all — it would pass on a
        // build whose cache never invalidated. `move_objects` is the cheapest
        // content edit there is.
        let before = doc.page_objects.built_for.get();
        let objects = doc
            .page_objects()
            .expect("page 2 decomposes")
            .page_objects()
            .objects
            .len();
        assert!(objects > 0, "the fixture's page 3 must carry an object");
        // `Arc::get_mut`, the shape this crate uses everywhere a test needs
        // the session mutably: the session is shared with the render worker
        // through an `Arc`, and a test that cloned it would be editing a copy.
        std::sync::Arc::get_mut(&mut doc.session)
            .expect("the test holds the only handle")
            .move_objects(2, &[0], 1.0, 0.0)
            .expect("moving one object by a point must succeed");
        doc.edit_epoch += 1;
        let _ = doc.page_objects();
        let after = doc.page_objects.built_for.get();
        assert_ne!(
            before, after,
            "an edit must rebuild, or the panel lists the pre-edit object set"
        );
        assert_eq!(
            after.map(|(page, _)| page),
            Some(2),
            "…and it must still be THIS page's objects"
        );
    }

    /// **Asking twice does not decompose twice** — the point of a cache, and
    /// the case that would panic if the validity key lived *inside* the
    /// `RefCell` instead of beside it: the second call would take
    /// `borrow_mut` while the first call's `Ref` was still alive. Holding the
    /// first borrow across the second call is the assertion, not an accident
    /// of how the test is written.
    #[test]
    fn a_second_reader_shares_the_decomposition_rather_than_rebuilding_it() {
        let doc = open_fixture(FOUR_PAGES);
        let first = doc.page_objects().expect("page 0 decomposes");
        let second = doc.page_objects().expect("…and again");
        assert_eq!(first.page_objects().objects.len(), 3);
        assert_eq!(
            second.page_objects().objects.len(),
            first.page_objects().objects.len()
        );
    }

    /// **A page that is not there yields no decomposition and no invented
    /// reason.**
    #[test]
    fn a_missing_page_yields_no_decomposition_and_no_invented_reason() {
        let mut doc = open_fixture(FOUR_PAGES);
        doc.view.page_index = 99;
        assert!(doc.page_objects().is_none());
        assert!(
            doc.page_objects_failure().is_none(),
            "there is no page, so there is no decode failure to report"
        );
        assert_eq!(
            doc.page_objects.built_for.get(),
            Some((99, 0)),
            "the attempt is still recorded, or it is retried every frame"
        );
    }

    // =======================================================================
    // The page-text cache
    // =======================================================================

    /// **The page's text is extracted once per `(page, epoch)`**, and asking
    /// twice does not extract twice.
    #[test]
    fn a_pages_text_is_extracted_once_and_shared() {
        let doc = open_fixture(FOUR_PAGES);
        let first = doc.page_text().expect("page 0 has extractable text");
        let second = doc.page_text().expect("…and again, from the cache");
        assert!(
            !first.plain_text().is_empty(),
            "the fixture must actually contain text, or every assertion below is vacuous"
        );
        assert_eq!(second.plain_text(), first.plain_text());
        assert_eq!(doc.page_text.built_for.get(), Some((0, 0)));
    }

    /// **A page step re-extracts; so does an edit.**
    #[test]
    fn the_page_text_is_rebuilt_when_the_page_or_the_revision_moves() {
        let mut doc = open_fixture(FOUR_PAGES);
        let first = doc.page_text().expect("page 0").plain_text();

        doc.view.page_index = 2;
        let third = doc.page_text().expect("page 2").plain_text();
        assert_eq!(doc.page_text.built_for.get(), Some((2, 0)));
        assert_ne!(
            first, third,
            "the fixture stamps a page number on each sheet, so two pages must not \
             produce the same text — if they do, this test cannot see a stale cache"
        );

        doc.edit_epoch = 1;
        let _ = doc.page_text();
        assert_eq!(
            doc.page_text.built_for.get(),
            Some((2, 1)),
            "an edit renumbers runs, so a selection resolved against the pre-edit \
             extraction would name the wrong glyphs"
        );
    }

    /// **A page that is not there yields no text and no invented reason.**
    #[test]
    fn a_missing_page_yields_no_text_and_no_invented_reason() {
        let mut doc = open_fixture(FOUR_PAGES);
        doc.view.page_index = 99;
        assert!(doc.page_text().is_none());
        assert!(
            doc.page_text_failure().is_none(),
            "there is no page, so there is no extraction failure to report"
        );
        assert_eq!(doc.page_text.built_for.get(), Some((99, 0)));
    }

    /// **The font inventory survives a page step and is dropped by an edit.**
    #[test]
    fn the_font_inventory_is_kept_across_pages_and_dropped_by_an_edit() {
        let mut doc = open_fixture(FOUR_PAGES);
        let _ = doc.font_inventory();
        assert_eq!(doc.fonts.built_for.get(), Some(0));

        doc.view.page_index = 3;
        let _ = doc.font_inventory();
        assert_eq!(
            doc.fonts.built_for.get(),
            Some(0),
            "a page step must NOT drop it — the inventory is document-scoped"
        );

        doc.edit_epoch = 1;
        let _ = doc.font_inventory();
        assert_eq!(doc.fonts.built_for.get(), Some(1), "an edit must drop it");
    }
}
