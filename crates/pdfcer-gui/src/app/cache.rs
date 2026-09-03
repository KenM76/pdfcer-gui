//! # `app::cache` — the three derived values a document is worth keeping, and why they live on it
//!
//! ## What is in here
//!
//! Three caches, and the [`OpenDoc`] methods that read them:
//!
//! | cache | what it holds | keyed on | read by |
//! |---|---|---|---|
//! | [`PageObjectCache`] | the current page's decomposition | `(page index, edit epoch)` | the Objects panel, the Properties panel, the canvas hit test, the `objects n=` trace |
//! | [`FontCache`] | the document's font inventory | `edit epoch` | the Fonts panel, the Properties panel |
//! | [`PageTextCache`] | the current page's **extracted text** | `(page index, edit epoch)` | canvas text selection, `file.copy_page_text` |
//!
//! ## ★ Why this is a module of its own — the seam, stated
//!
//! `app/state.rs` reached 1,468 lines against the 1,500-line gate (rule R2),
//! and the S4 selection move was about to add to it. A size gate is only
//! useful if the split it forces is a **real** seam rather than an arbitrary
//! cut at line 750, so the question was which of `state.rs`'s subjects is
//! separable without leaving a dangling half-explanation behind.
//!
//! These two are. Everything else in `state.rs` answers *"what is open, and
//! what is the operator looking at?"* — [`crate::app::state::Status`]'s
//! three-way failure distinction, [`OpenDoc`]'s view fields, the raster
//! bookkeeping that keeps the page texture honest. These two answer a
//! different question: *"what expensive thing derived from the document do
//! several surfaces need, and how do we compute it once?"* They share one
//! argument (the cost of `pdfcer-core` recomputation), one hazard (staleness
//! against `edit_epoch`), and one structural device (a `Cell` key beside a
//! `RefCell` payload, for the borrow reason below). None of that is shared
//! with anything left behind.
//!
//! The seam is also the one the caches themselves already implied: they were
//! moved off `crate::panels::PanelsState` onto `OpenDoc` earlier in this same
//! stage, and the whole argument for that move — *a cache should be bounded by
//! the lifetime of what it describes* — is a statement about caches as a
//! class, not about any one of them.
//!
//! ## ★ Why interior mutability, and why that is not a smell here
//!
//! A panel body is handed `&OpenDoc`, never `&mut` — that is the
//! actions-not-mutations invariant, and it is not negotiable
//! (`PROJECT_PLAN.md` §3). A lazily-built cache behind a shared reference is
//! precisely what [`RefCell`] is for: the cache is *derived*, so filling it
//! changes nothing an observer could see, and the alternative — building it
//! eagerly on every page change whether or not any surface asked — would cost
//! a decomposition per page step with the Objects panel closed.
//!
//! It applies to **caches only**, which is the other reason they are worth
//! collecting in one file: the exemption has a visible boundary. State that
//! decides what appears on the page (the layer override, the annotation flag,
//! the selection) stays behind `&mut self` over in `state.rs` and reaches it
//! through an [`crate::app::actions::Action`], or *"what can change what is
//! drawn?"* stops having a complete answer.
//!
//! ## ★ Why neither cache can panic on a double borrow
//!
//! The `RefCell` hazard is a `borrow_mut` taken while a `Ref` is still alive.
//! It is unreachable here by a borrow-checker argument rather than by care,
//! and the argument is the same for both caches:
//!
//! 1. The validity key is a [`Cell`], **outside** the `RefCell`, so the
//!    already-built path reads it and takes only a shared `borrow()`.
//! 2. `borrow_mut` is reached only when the key has *moved*.
//! 3. A live `Ref<'a, …>` borrows `&'a OpenDoc`, so while one exists nothing
//!    can take `&mut OpenDoc` — and `view.page_index` and `edit_epoch` change
//!    only through `&mut self`. The key therefore **cannot** move while a
//!    `Ref` is outstanding.
//!
//! Keeping the key in a `Cell` rather than inside the `RefCell` is what makes
//! step 1 true, and is the entire reason each cache is two fields rather than
//! one. [`tests::a_second_reader_shares_the_decomposition_rather_than_rebuilding_it`]
//! holds two `Ref`s at once, so the property is exercised and not merely
//! argued.

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
///
/// # Why this is a cache at all
///
/// `pdfcer_core::vector::decompose_page` resolves every `/Contents` stream,
/// inflates it, concatenates, tokenizes and walks the whole token stream
/// resolving fonts as it goes, and there is **no cache anywhere in
/// `pdfcer-core`**. On a CAD sheet that is a frame's worth of work; doing it
/// per frame at 60 Hz is not an option, and doing it *twice* per frame — once
/// for the Objects panel and once for a canvas hit test — is the *"two
/// decompositions quietly diverge"* failure decision 011 names.
///
/// So there is exactly one, and it lives here: on the document, whose
/// lifetime bounds it exactly. See the module docs for the borrow argument
/// the two-field shape exists to satisfy.
///
/// `pub(in crate::app)` rather than private because [`OpenDoc`] declares the
/// field and lives in the sibling module `crate::app::state`. The type is not
/// part of the crate's surface: nothing outside `crate::app` can name it, and
/// every read goes through [`OpenDoc::page_objects`].
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
/// # ★ Why this cache exists, and the measurement that forced it
///
/// `crate::find`'s header records the trap in its own words:
///
/// > `EditSession::find_text_with` runs `text_extract::extract_document_view`
/// > over the **whole document** on every call […] There is no cache in
/// > `pdfcer-core` and none here.
///
/// That was measured at **331–449 ms per search** on the project's fixtures,
/// which is why Find never searches on a keystroke. Canvas text selection
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
/// # ★ The revision is the SESSION's, and that is not the same choice Find made
///
/// `extract_page_view` takes a `DocumentView`, and core made the revision the
/// caller's explicit decision (Pass 17.1, decision 018 §8) precisely because
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

/// **Which of the current page's runs are drawn from inside a form XObject** -
/// the editability answer, cached because the question is asked on every click
/// that lands on text.
///
/// # ★★ Why this exists at all
///
/// `pdfcer-core` edits text in the **page's own content stream** and not inside
/// a `Do`-invoked form XObject - a named non-goal of that cut
/// (`pdfcer-core/src/text_edit/edit.rs:79`). The only published way to tell the
/// two apart is `GlyphProvenance::content_stream`, and provenance is only
/// populated when the extraction asked for it, which [`PageTextCache`]
/// deliberately does not.
///
/// So answering *"can this run be edited?"* costs **a second extraction of the
/// whole page, with provenance on**.
///
/// ★ **Measured on two documents, and the range is the point.** The operator
/// pointed out - correctly - that testing on the densest sheet available makes
/// everything look slow:
///
/// | document | runs | extraction |
/// |---|---|---|
/// | a 6-page scanned note | 0 | **1 ms** |
/// | the benchmark CAD site plan, 129,758 objects | 4,655 | **336 ms** |
///
/// So this is not "text extraction is slow". It is *"text extraction is
/// proportional to how much text is on the page, and on the documents this
/// operator actually works on there is a great deal of it."* Both numbers are
/// recorded because a single alarming one invites the wrong fix.
///
/// The cache is still the right answer, and the dense end is why: 336 ms
/// inside the click handler froze the UI thread for a third of a second on
/// every click that landed on text - a visible hitch on exactly the documents
/// this application exists for, and one that made a driven check flake because
/// the trace it was waiting on had not been written by the time the settle
/// window closed. A performance defect presenting as harness flakiness is a
/// shape this project has been caught by before.
///
/// # What is cached, and why it is a `Vec<bool>` rather than the extraction
///
/// The **answer**, not the evidence. A second `PageText` for a dense sheet is
/// megabytes held for the life of the page; a bit per run is 4,655 bytes. The
/// commit path still does its own provenance extraction, because it needs the
/// byte spans rather than the verdict and a commit is a rare, already-expensive
/// act.
///
/// Keyed on `(page index, edit epoch)` like every other cache here, so an edit
/// invalidates it - which matters, because an edit can change how many runs a
/// page has and a stale `Vec` indexed by run number would answer confidently
/// about the wrong run.
/// **Every clickable `/Link` on the current page, and the reader that resolves
/// where each one goes** (ISO 32000-1 §12.5.6.5, §12.3.2).
///
/// # ★★★ Why this is TWO caches with two different keys
///
/// They have genuinely different lifetimes, and collapsing them would make the
/// expensive one page-scoped:
///
/// | | keyed on | cost | rebuilt when |
/// |---|---|---|---|
/// | [`Self::reader`] | the **edit epoch** | **O(document)** | any edit |
/// | [`Self::links`] | `(page, epoch)` | O(annots on the page) | the page changes, or any edit |
///
/// `pdfcer_core::outline::DestinationReader` has to flatten two document-wide
/// tables before it can answer anything: the page-object → index map, and both
/// §12.3.2.3 named-destination namespaces. The engine's own reply on shipping
/// it put the point plainly — a proposed per-call signature *"would have
/// rebuilt both on every call. A page with 200 links would have walked the page
/// tree 200 times, and you would have discovered it as 'links are slow on big
/// documents' months from now with no obvious cause."*
///
/// So the reader is held for as long as the document's structure is unchanged,
/// and the per-page link list is rebuilt beside it whenever the operator turns
/// a page.
///
/// # ★★ The reader is a SNAPSHOT and going stale is silent
///
/// It resolves against the page order it was built with. A page delete, a page
/// insert or a new named destination invalidates it — and a stale one does not
/// error, it answers *confidently and wrongly*, sending a link to the page that
/// used to be at that index.
///
/// The epoch key is what prevents that, and it is why the reader is keyed on
/// the epoch rather than held for the document's lifetime: **every** edit bumps
/// `OpenDoc::edit_epoch`, so the reader cannot survive one. That is coarser
/// than necessary — recolouring a path invalidates a reader nothing structural
/// touched — and coarse in the safe direction. Rebuilding it is a page-tree
/// walk; getting it wrong is a link that navigates somewhere plausible and
/// false, which is the one failure this feature could ship that nobody would
/// report as a bug.
///
/// # Why the whole `PageLinks` is kept, not just the navigable ones
///
/// Because `PageLinks::links_without_destination` is the count of `/Link`
/// annotations carrying **neither** `/Dest` nor `/A` — clickable boxes that
/// Table 173 gives no way to act. A caller that only saw the resolved list
/// could not tell a page with no links from a page whose links are all broken,
/// and those two want opposite sentences from the program.
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
    /// ★★ **This used to mean "inside a form XObject", and it stopped meaning
    /// that on 2026-08-20** when `Pass 119.0` made form content editable. The
    /// old reading refused a caret on 99 % of the text on a CAD drawing — the
    /// operator's own estimate — so the change is recorded here rather than
    /// left to be inferred from a renamed field.
    ///
    /// What is left is the case that was always unreachable: an `/ActualText`
    /// run, where the producer supplied a replacement string for a span of
    /// glyphs, so the run covers no operator a pinned span could name.
    ///
    /// `None` means the extraction did not run or provenance was unavailable -
    /// which the caller must read as **"not measured"**, never as "yes". A
    /// refusal on an unmeasured answer would block text editing everywhere on
    /// a guess, and would look exactly like the feature having been removed.
    pub(in crate::app) flags: RefCell<Option<Vec<bool>>>,
}

/// The document's font inventory, held for as long as the document is open.
///
/// Cached for the same reason [`PageObjectCache`] is, and the sweep is more
/// expensive: `pdfcer_core::fontinfo::inventory` **decodes every embedded font
/// program**, because that is where the `OS/2` table lives. On a document
/// carrying a megabyte of CJK outlines that is not a per-frame cost.
///
/// Document-scoped rather than page-scoped — paging does not drop it — but
/// **not** revision-scoped-by-accident: an edit can add or remove a font, so
/// the epoch is the key.
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
    ///
    /// # ★ This is THE decomposition — there is deliberately only one
    ///
    /// The Objects panel lists it, the Properties panel describes a row of
    /// it, the diagnostic `objects n=` line counts it, and the canvas
    /// hit-tests against it — all from *this* value. A second
    /// `decompose_page` over the same page is the *"two decompositions
    /// quietly diverge"* pattern decision 011 warns about, and
    /// [`ObjectModelProvider::page_objects`]' own docs call this the shared
    /// escape hatch that exists to prevent it.
    ///
    /// **The canvas's second decomposition is gone.** Until this stage's
    /// wiring pass, `canvas::show` built its own `ObjectModelProvider` per
    /// gesture, because the only cache was on the panels and the canvas had no
    /// route to it. That was one extra full decomposition per click and per
    /// marquee release on the same page the Objects panel had already
    /// decomposed. The canvas now calls this method, so *"what did I click?"*
    /// and *"what is in this list?"* are answered from one value by
    /// construction rather than by two code paths that happen to agree.
    ///
    /// # Why it lives on `OpenDoc` and needs no identity key
    ///
    /// It was on `crate::panels::PanelsState` until S4, guarded by a `DocKey`
    /// built partly from the `Arc<EditSession>`'s **address** — because a
    /// cache hanging off the application outlives the document it describes,
    /// so it has to say *which* document that was, and an address is the only
    /// token that was available. `crate::panels`' own header records that key,
    /// its ABA hazard, and why a `Weak` clone would have been a worse fix
    /// than the bug.
    ///
    /// Moving it here dissolves the question rather than answering it, for
    /// the reason already in [`OpenDoc::new`]'s doc comment: *"opening a
    /// document constructs a whole new `OpenDoc`, so a cached texture or a
    /// page index can never refer to a page from a previous file."* A cache
    /// held **inside** that structure inherits the guarantee for free — there
    /// is no "which document is this?" to get wrong, because the answer is
    /// "the one you are holding". So `DocKey` was deleted rather than
    /// repaired, and what remains is `(page, epoch)`: two plain values, no
    /// address, no ABA. The canvas's `DocumentToken` — the same idea, built
    /// the same way, for the selection — was deleted for the same reason in
    /// the same stage, once the selection moved onto `OpenDoc` beside this.
    ///
    /// # Returns
    ///
    /// `None` when the page's content cannot be decoded — the same failure
    /// the renderer would hit. A caller says so in words rather than showing
    /// an empty list, because a failure state indistinguishable from a
    /// success state is the same defect as no message at all. The reason is
    /// kept for the trace channel; see [`Self::page_objects_failure`].
    ///
    /// # Holding the `Ref`
    ///
    /// The return is a [`Ref`] into the cache, so it keeps a shared borrow of
    /// `*self` alive for as long as the caller holds it — which is exactly
    /// what stops a `borrow_mut` racing it (module docs, step 3). A caller
    /// that needs `&mut OpenDoc` afterwards must let it go first; `Ref`
    /// implements `Drop`, so the borrow does **not** end at its last use and
    /// an explicit `drop` is sometimes required. `canvas::interact` does that
    /// and says why.
    #[must_use]
    pub fn page_objects(&self) -> Option<Ref<'_, ObjectModelProvider>> {
        self.ensure_page_objects();
        Ref::filter_map(self.page_objects.provider.borrow(), |slot| {
            slot.as_ref().and_then(|built| built.as_ref().ok())
        })
        .ok()
    }

    /// Why the current page would not decompose, if it would not.
    ///
    /// Separate from [`Self::page_objects`] because the two audiences differ:
    /// a panel shows the operator a sentence from the text catalog, and the
    /// `PDFCER_DIAG` channel wants the engine's own error text. A harness that
    /// learns only *that* a page failed has to work out *why* by hand.
    ///
    /// `pub(in crate::app)` because the one consumer is
    /// `OpenDoc::trace_object_count`, which stayed in `state.rs` with the rest
    /// of the per-frame bookkeeping.
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
    /// # ★★★ The 469 ms this removes, measured on the operator's own drawing
    ///
    /// ```text
    /// page-objects-built page=0 objects=129758 leaves=10256 ms=469
    /// ```
    ///
    /// That is one decomposition of the benchmark CAD sheet. Keyed on
    /// `edit_epoch` — which every mutating action bumps — it was paid again
    /// after **every** edit, including edits that cannot have touched page
    /// content at all. Authoring a form field is an `/Annots` change; so is
    /// placing a stamp, a note, or a ce dimension. Each of those froze the
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
    /// # ★★ …and a counter of our own, because the digest has one blind spot
    ///
    /// **Measured, not assumed** —
    /// `crates/pdfcer-gui/tests/page_generation_covers.rs`, three tests, one per
    /// dependency class:
    ///
    /// | class | generation |
    /// |---|---|
    /// | a content edit (`move_objects`) | moves |
    /// | an annotation edit (`add_markup`) | holds still ✓ the win |
    /// | **an edit inside a form XObject** (`move_node_in_form`) | ★ **holds still** |
    ///
    /// The third is a real hazard rather than a curiosity: `PageObjects`
    /// addresses content **by index**, so a stale model makes the next drag
    /// edit whatever that index names in the *wrong* model — the engine's own
    /// phrase is *"silent corruption of the operator's drawing, reported as
    /// success"*. It is consistent with its account of the memo: the descended
    /// form set is kept beside the key because it is an **output** of the
    /// decomposition, and this accessor digests the key alone.
    ///
    /// ⇒ **That was true for four hours.** It was filed as a boundary finding
    /// per decision 058 rather than absorbed, the reproduction test asserted
    /// the limitation so it would go red when the engine closed it, and the
    /// engine closed it the same night (`6e2b69e`): the digest now folds in
    /// the descended-form set. The shell-side counter that carried the gap has
    /// been **deleted**, which is what the tripwire existed to trigger.
    ///
    /// ★ The third time in two days this shape has paid out. A test that
    /// asserts a limitation is a request that files its own closure.
    ///
    /// ★ **The fallback is the epoch**, not a constant. If the generation
    /// cannot be read — a page index the session does not have, a document
    /// mid-close — this returns the epoch, which rebuilds on every edit exactly
    /// as before. Slow is the safe direction; a constant would freeze the model.
    fn page_objects_revision(&self) -> u64 {
        match self.content_generation.get() {
            // ★★★ The digest, but ONLY while the epoch it was measured at is
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
    ///
    /// Called once per frame from `app::frame`, before anything draws. Silent
    /// when the session is shared — a render in flight holds the second handle
    /// — and that silence is safe by construction: the stamp stored with the
    /// digest is compared against the live epoch before it is trusted.
    ///
    /// ★ It is `&mut self` because the engine's accessor is, and the engine's
    /// accessor is because *"which forms a page paints is an OUTPUT of the
    /// decomposition"* — this shell's own sentence, quoted back at it in the
    /// reply that shipped the fix. There is no way to fold the form set into
    /// the digest without walking, and walking populates a memo.
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
    ///
    /// The key is recorded **before** the work, so a page whose content will
    /// not decode is not re-decomposed on every frame: the failure is
    /// deterministic, and retrying it sixty times a second would peg a core
    /// producing the same error.
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
    ///
    /// The one resolution. The hover cursor, the click that follows a link and
    /// anything that ever reports a document's broken links all read *this*
    /// value, for the same reason [`Self::page_text`] is the one extraction:
    /// two resolutions of one page are two chances for what the cursor promises
    /// and what the click performs to disagree.
    ///
    /// # Returns
    ///
    /// `None` only when there is no such page. A page with **no** links returns
    /// an empty [`PageLinks`], which is a different answer and a caller may
    /// need the difference — see [`LinkCache`] on why the unresolvable ones are
    /// counted rather than dropped.
    ///
    /// # Cost
    ///
    /// The first call for a `(page, epoch)` pays one `/Annots` walk plus, if
    /// the epoch moved, one page-tree walk and one name-tree flatten for the
    /// [`DestinationReader`]. Every call after it is two comparisons. See
    /// [`LinkCache`] for why those two costs are keyed separately.
    ///
    /// ★ It is called from a **hover**, sixty times a second, which is why the
    /// caching is not optional. The first sketch of this feature resolved links
    /// per frame and would have walked the page tree of a 36-sheet drawing on
    /// every mouse move.
    ///
    /// # Holding the `Ref`
    ///
    /// As [`Self::page_text`]: the return keeps a shared borrow of `*self`
    /// alive, so a caller that needs `&mut OpenDoc` afterwards must drop it
    /// first — or clone the one link it cares about, which is what
    /// `crate::canvas::links` does.
    #[must_use]
    pub fn page_links(&self, page_index: usize) -> Option<Ref<'_, PageLinks>> {
        self.ensure_page_links(page_index);
        Ref::filter_map(self.links.links.borrow(), Option::as_ref).ok()
    }

    /// Build [`Self::page_links`] for `(page_index, epoch)` if it is not
    /// already built. Idempotent, and two comparisons on every call after the
    /// first.
    fn ensure_page_links(&self, page_index: usize) {
        // ★ The reader FIRST and on its own key. It is the O(document) half and
        // it survives a page turn; the links below do not. See [`LinkCache`].
        if self.links.reader_for.get() != Some(self.edit_epoch) {
            self.links.reader_for.set(Some(self.edit_epoch));
            // ★ The SESSION view, never the base document's — the same rule
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
                // ★ Emitted on a BUILD, never on a cache hit — so the number of
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
    ///
    /// `Some(true)` / `Some(false)`, or `None` when the question could not be
    /// answered - see [`FormRunCache::flags`] for why `None` must never be read
    /// as `true`, and for what this question used to be.
    ///
    /// The first call for a `(page, epoch)` pays one provenance-bearing
    /// extraction; every call after it is a vector index. See
    /// [`FormRunCache`] for the measurement that made the cache necessary.
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
        // Set BEFORE the work, exactly as `ensure_page_text` does: a failed
        // extraction is deterministic for these bytes, so re-attempting it
        // sixty times a second would burn a third of a second per frame to
        // learn the same thing.
        self.form_runs.built_for.set(Some(key));
        let started = Instant::now();
        let built = self.current_page().and_then(|page| {
            use crate::app::settings::SettingsExt;
            // ★ The funnel's output MODIFIED, never `ExtractOptions::default()`
            // - the same rule and the same reason as `ensure_page_text`. The run
            // indices here must agree with the ones the canvas hit-tests and the
            // ones the commit pins, and two extractions under two configurations
            // segment differently.
            let opts = self.settings.extract_options().with_provenance(true);
            let text = pdfcer_core::text_extract::extract_page_view(
                &self.session.view(),
                page,
                self.view.page_index,
                &opts,
            )
            .ok()?;
            // ★★★ THE ENGINE'S OWN QUERY, since `Pass 118.0` — and the whole
            // point of it was proved on 2026-08-20.
            //
            // This matched on `GlyphProvenance::content_stream` by hand until
            // that morning — a shell encoding a fact about the surgery's
            // internals, which is precisely the workaround this project's own
            // request warned would outlive its bug:
            //
            // > *"the day form editing lands, my guard silently keeps refusing
            // > until I notice and delete it."*
            //
            // `TextRun::editability` shipped that afternoon. **`Pass 119.0`
            // landed form editing that evening**, `editability()` began
            // answering `Editable` for form content, and the entire cost to
            // this shell was deleting one arm that a `#[deprecated]` attribute
            // pointed straight at. The hand-rolled guard would have gone on
            // refusing carets on 99 % of the text on a CAD drawing until
            // somebody noticed.
            //
            // What is left is the case the hand-rolled version could not see at
            // all: `NoAnchor`, an `/ActualText` run covering no show operators,
            // which has nothing for the surgery to anchor on and was being
            // offered a caret.
            //
            // ★ `Unknown` is NOT treated as "no". It is the state a caller
            // reaches by default — provenance not captured — and the engine
            // made the type an enum rather than a `bool` specifically so it
            // cannot be confused with a measured refusal. We asked with
            // provenance on, so `Unknown` here means something went wrong with
            // the ask rather than with the run, and refusing every caret on it
            // would block text editing document-wide for a reason nobody
            // measured.
            use pdfcer_core::text_extract::Editability;
            Some(
                text.runs
                    .iter()
                    .map(|run| matches!(run.editability(), Editability::NoAnchor))
                    .collect::<Vec<bool>>(),
            )
        });
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            //
            // ★ The count of these lines IS the measurement, as it is for
            // `page-text` - one line per extraction, so a harness can tell a
            // cache that works from one that does not. `in_form` beside `runs`
            // is what makes the engine's boundary visible on a real document:
            // on the benchmark CAD sheet it is most of them.
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

    /// ★ **Does this page carry any extractable text at all?**
    ///
    /// The question *"is this page an image rather than a document"*, answered
    /// as a **cache read** rather than as an extraction. Read by
    /// [`crate::find::bar`] to decide whether to offer OCR when a search comes
    /// back empty, and it is the whole reason that offer is affordable.
    ///
    /// # ★ Why this is not "the search found nothing"
    ///
    /// The operator's rule for the Find offer, and the trap inside it: the
    /// trigger is *"this document is images"*, **not** *"this search had no
    /// matches"*. A search for `flange` that finds nothing on a text PDF is an
    /// ordinary empty result and offering to recognise it would be nonsense —
    /// the words are there, that one just is not among them.
    ///
    /// This function is what tells the two apart, and it does so by asking
    /// about the **page** rather than about the query. `false` means the
    /// extractor walked this page's content streams and found no character on
    /// it: there is nothing here for *any* search to have matched.
    ///
    /// # `false` covers two different states, on purpose
    ///
    /// A page with no text, and a page whose content stream will not walk, both
    /// answer `false`. [`Self::page_text`] separates them and
    /// [`Self::page_text_failure`] carries the reason; this predicate
    /// deliberately does not, because the *offer* is right in both cases —
    /// a page whose stream pdfcer cannot read is exactly a page where
    /// recognising the pixels is the remaining route to its words.
    ///
    /// # Cost
    ///
    /// One extraction per `(page, edit epoch)`, shared with canvas text
    /// selection and `file.copy_page_text` — see [`Self::page_text`]'s cost
    /// section. The first caller on a page pays it and the rest are a `Cell`
    /// comparison, so this is affordable **as long as it is asked at a moment
    /// the operator caused**. `crate::find::bar` asks it only when the readout
    /// is already `Empty`, i.e. after a committed search has run a
    /// whole-document extraction — which is strictly more expensive than this
    /// and has just been paid. Calling it every frame the bar is open would be
    /// `HANDOFF.md` §2's defect 9 in miniature: the right work, charged at the
    /// wrong moment.
    ///
    /// Whitespace does not count as text. A page carrying one space is an
    /// image page with a stray operator on it, and an offer suppressed by that
    /// would be suppressed on exactly the scans most in need of it.
    #[must_use]
    pub fn page_has_extractable_text(&self) -> bool {
        self.page_text()
            .is_some_and(|text| !text.plain_text().trim().is_empty())
    }

    /// Why the current page's text would not be extracted, if it would not.
    ///
    /// Separate from [`Self::page_text`] for the reason
    /// [`Self::page_objects_failure`] is separate: the `PDFCER_DIAG` channel
    /// wants the engine's own error text, and a consumer that learns only
    /// *that* extraction failed has to work out *why* by hand.
    ///
    /// Read by the `file.copy_page_text` dispatch arm, which uses it to tell
    /// three states apart — *the content stream would not walk*, *there is no
    /// such page*, and *the page has no text on it* — rather than reporting one
    /// "unavailable" for all three.
    pub(in crate::app) fn page_text_failure(&self) -> Option<Ref<'_, String>> {
        self.ensure_page_text();
        Ref::filter_map(self.page_text.text.borrow(), |slot| {
            slot.as_ref().and_then(|built| built.as_ref().err())
        })
        .ok()
    }

    /// Extract the current page's text if the cache does not already describe
    /// it.
    ///
    /// The key is recorded **before** the work, for the reason
    /// [`Self::ensure_page_objects`] records its own: the failure is
    /// deterministic, and retrying it every frame would peg a core producing
    /// the same error.
    fn ensure_page_text(&self) {
        let key = (self.view.page_index, self.edit_epoch);
        if self.page_text.built_for.get() == Some(key) {
            return;
        }
        self.page_text.built_for.set(Some(key));
        let started = Instant::now();
        let built = self.current_page().map(|page| {
            // ★ The SESSION view, never the base document's — see
            // `PageTextCache`'s header. The operator is dragging across glyphs
            // they can see, and the base revision may no longer describe them.
            //
            // ★ Through the funnel, and NOT `ExtractOptions::default()`.
            //
            // This call site is why `crate::app::settings::SettingsExt` exists:
            // it is the extraction the canvas's text selection, the find bar
            // and `file.copy_page_text` all read, and a bare `::default()` here
            // silently discarded three of the operator's settings — the word
            // gap, the unmappable sentinel and the replacement-text precedence.
            // The old shell had exactly this line and exactly that consequence.
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
            // ★ Not de-duplicated through `trace_changed`: two extractions are
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
    ///
    /// Moved here from `crate::panels::PanelsState` at S4 for exactly the
    /// reason [`Self::page_objects`] was, and it is the cheaper half of the
    /// argument to state: the inventory decodes every embedded font program,
    /// and it is read by two panels (Fonts lists it; Properties joins one
    /// object's `/BaseFont` against it). Two inventories over one document
    /// would be two sweeps and two chances to disagree.
    ///
    /// `pdfcer_core::fontinfo::inventory` is **infallible** — it reports
    /// problems in its `diagnostics` rather than in a `Result` (core API trap
    /// T-9.8) — so there is no error path here, and an empty inventory does
    /// not mean a clean document. The Fonts panel reads the diagnostics.
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
    /// ★ **Drop every value derived from a text extraction.**
    ///
    /// Called from `PdfcerApp::adopt_settings` and from nowhere else, because
    /// there is exactly one thing that invalidates these without also
    /// invalidating everything: the operator changing a setting.
    ///
    /// # Why the ordinary staleness keys do not cover this
    ///
    /// Every cache in this module is keyed on document state — a page index, an
    /// edit epoch. Those keys are complete for the question they were built to
    /// answer, which is *"has the document changed under this?"*. They are
    /// silent on *"has the configuration this was computed under changed?"*,
    /// and three settings change what an extraction produces:
    ///
    /// | setting | what moves |
    /// |---|---|
    /// | `word_gap_ratio` | where spaces appear between words |
    /// | `unmappable_code` | what stands in for undecodable text — **and whether a whole run survives at all** |
    /// | `actual_text` | whether a document's own replacement text wins over the glyphs |
    ///
    /// The middle row is why this is not cosmetic. Under *Leave it out*, a run
    /// whose codes are all unmappable **disappears entirely**, so a stale
    /// extraction is not merely differently spaced — it can be missing content
    /// that a find, a text selection or a redaction-by-pattern would then fail
    /// to see. A cache holding one of those after the setting has changed is a
    /// surface confidently reporting the wrong answer.
    ///
    /// # Why the keys are not extended instead
    ///
    /// Adding the settings to `built_for` would mean hashing a
    /// `#[non_exhaustive]` struct from another crate into every key, and it
    /// would spread the answer across three tuples that must each be updated
    /// when a fourteenth setting arrives. Clearing at the one moment the
    /// configuration changes is both cheaper and visible in one place — the
    /// same reasoning `render::settle` applies to the raster keys.
    ///
    /// # What is NOT cleared, and why
    ///
    /// [`PageObjectCache`] and [`FontCache`] hold structure rather than text:
    /// the object model's paint-order inventory and the document's font list.
    /// No setting in the window changes either. They are left alone rather than
    /// swept along for tidiness, because clearing a cache nothing invalidated
    /// makes the next frame pay for a rebuild that changes nothing — and on the
    /// benchmark sheet the object model is the expensive one.
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

    /// **★ The decomposition cache carries NO document identity, and does
    /// not need one.**
    ///
    /// The `DocKey` deletion, asserted rather than argued. That key existed
    /// because the cache hung off the *application* and outlived the document
    /// it described, so it had to say **which** document — and the only token
    /// available was an `Arc` address, which is not an identity (see
    /// `OpenDoc::page_objects`).
    ///
    /// This replaces one document with another **in the same binding**, the
    /// sequence that would exercise an address reuse. There is nothing to get
    /// wrong: the second document's cache is a field of the second document.
    /// The remaining key is `(page, epoch)`, and it is asserted here so that
    /// putting an address, a pointer or a `Weak` back into it is a test
    /// failure rather than a review finding.
    #[test]
    fn a_documents_decomposition_cannot_outlive_the_document() {
        let mut doc = open_fixture(FOUR_PAGES);
        // ★★ The frame's own first step, performed here because this test is
        // about the KEY and the key's second half is measured there.
        //
        // `page_objects_revision` reads a digest that `app::frame` takes once
        // per frame at the one `&mut` point it has — a unit test that skipped
        // it would exercise the epoch fallback and assert nothing about the
        // digest, which is what this assertion is for.
        doc.refresh_content_generation();
        assert_eq!(doc.page_objects().expect("page 0").page_index(), 0);
        // ★ The page, not the whole key: the second half is a content digest
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
        // ★★ And the two documents' keys DIFFER, which is the property that
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
    ///
    /// Both halves of the key, one at a time. Serving page 0's objects while
    /// the operator is on page 1 would make every index in the Objects panel
    /// address the wrong object.
    ///
    /// ★ It asserts that the key **CHANGED**, not what it changed to. The
    /// second half stopped being `edit_epoch` on 2026-08-31 — it is now the
    /// engine's content digest, mixed with this shell's form-edit counter (see
    /// [`super::OpenDoc::page_objects_revision`]) — and a test pinning the
    /// literal would have had to be rewritten for a change it exists to be
    /// indifferent to. What matters is that a rebuild happened, which is what
    /// a changed key means and all it means.
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
        // ★★ Driven through a REAL content edit rather than by setting
        // `edit_epoch` by hand, and that change is the point of the swap: the
        // key no longer reads the epoch, so a test that bumped it would now
        // assert nothing at all — it would pass on a build whose cache never
        // invalidated. `move_objects` is the cheapest content edit there is.
        let before = doc.page_objects.built_for.get();
        let objects = doc
            .page_objects()
            .expect("page 2 decomposes")
            .page_objects()
            .objects
            .len();
        assert!(objects > 0, "the fixture's page 3 must carry an object");
        // ★ `Arc::get_mut`, the shape this crate uses everywhere a test needs
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
    ///
    /// The attempt is recorded either way, so it is not retried sixty times a
    /// second. But "there is no such page" must not be reported as a decode
    /// failure: the trace channel distinguishes `reason=no-such-page` from
    /// `reason=decompose-failed`, and a consumer is entitled to that.
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

    /// ★ **The page's text is extracted once per `(page, epoch)`**, and asking
    /// twice does not extract twice.
    ///
    /// The property the whole feature's affordability rests on: a text drag
    /// asks for this on every frame of the gesture, and `crate::find`'s header
    /// records what an unconditional extraction costs — 331–449 ms, for the
    /// *document*. Holding the first `Ref` across the second call is the
    /// assertion rather than an accident of how the test is written: it is also
    /// the case that would panic if the validity key lived inside the `RefCell`
    /// instead of beside it.
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
    ///
    /// Both halves of the `(page, epoch)` key, one at a time — the same two
    /// failures [`the_decomposition_is_rebuilt_when_the_page_or_the_revision_moves`]
    /// guards, and sharper here: a stale `PageText` does not merely list the
    /// wrong objects, it makes every `TextPosition` in a live selection name a
    /// run that has moved, so the highlight would be drawn over the wrong
    /// glyphs and the copy would carry them.
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
    ///
    /// The attempt is still recorded, or it is retried every frame — the same
    /// rule [`a_missing_page_yields_no_decomposition_and_no_invented_reason`]
    /// states for the decomposition.
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
    ///
    /// It decodes every embedded font program, so rebuilding it per page is a
    /// large cost for a value that cannot have changed — and an edit *can*
    /// add or remove a font, so keeping it across one reports a font list the
    /// document no longer has.
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
