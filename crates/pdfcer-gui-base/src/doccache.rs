//! # `doccache` — The per-document caches an open document carries: each a cell keyed by page and edit epoch, filled on first read.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use pdfcer_core::annot::PageLinks;
use pdfcer_core::fontinfo::FontInventory;
use pdfcer_core::outline::DestinationReader;
use pdfcer_core::text_extract::PageText;

use crate::objectprovider::ObjectModelProvider;

/// The page decomposition, held for as long as the document is open.
#[derive(Default)]
pub struct PageObjectCache {
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
    /// carry, because opening a document constructs a whole new `OpenDoc`
    /// and this cache dies with the old one. That is the point of the move —
    /// see `OpenDoc::page_objects`.
    pub built_for: Cell<Option<(usize, u64)>>,
    /// The decomposition, or the reason the page would not decode.
    ///
    /// `None` means "not attempted". `Some(Err(_))` means "attempted and
    /// failed", which is a **different state**: the failure is deterministic
    /// (same bytes, same code), so a page whose content will not decode must
    /// not be re-decomposed on every frame. That is the same reasoning the
    /// render-error hold in `PdfcerApp::settle_and_rasterize` uses.
    pub provider: RefCell<Option<Result<ObjectModelProvider, String>>>,
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
///    per-page twin of every entry point — `extract_page_view` beside
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
/// `extract_page_view`: pdfcer_core::text_extract::extract_page_view
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
/// is the same choice `OpenDoc::page_objects` makes and for the same stated
/// reason (decision 018).
///
/// # Why the failure is kept rather than collapsed to `None`
///
/// Same argument as [`PageObjectCache::provider`]: `None` means *not
/// attempted*, `Some(Err(_))` means *attempted and failed*, and the failure is
/// deterministic — same bytes, same code — so a page whose content stream will
/// not tokenize must not be re-walked sixty times a second.
#[derive(Default)]
pub struct PageTextCache {
    /// The `(page index, edit epoch)` the extraction below describes, or
    /// `None` before the first attempt. See [`PageObjectCache::built_for`] for
    /// the borrow argument this `Cell` is half of — it is the same two-field
    /// shape for the same reason.
    pub built_for: Cell<Option<(usize, u64)>>,
    /// The extraction, or the engine's own reason it would not run.
    pub text: RefCell<Option<Result<PageText, String>>>,
}

/// **The document's page labels**, rebuilt once per edit epoch.
#[derive(Default)]
pub struct LabelCache {
    /// The edit epoch the fields below describe.
    pub built_for: Cell<Option<u64>>,
    /// Every page's displayed label, in page order; empty when the document
    /// stores no labels or its page tree will not walk.
    pub labels: RefCell<Vec<String>>,
    /// The ranges as stored; empty when there are none.
    pub ranges: RefCell<Vec<pdfcer_core::page_labels::LabelRange>>,
}

/// **Which of the current page's runs have no show operator of their own** -
/// the editability answer, cached because the question is asked on every click
/// that lands on text.
#[derive(Default)]
pub struct LinkCache {
    /// The edit epoch [`Self::reader`] was built at.
    pub reader_for: Cell<Option<u64>>,
    /// The document-wide destination resolver. See the type's docs.
    pub reader: RefCell<Option<DestinationReader>>,
    /// The `(page index, edit epoch)` [`Self::links`] describes.
    pub built_for: Cell<Option<(usize, u64)>>,
    /// That page's links, resolved.
    pub links: RefCell<Option<PageLinks>>,
}

#[derive(Default)]
pub struct FormRunCache {
    /// The `(page index, edit epoch)` the flags below describe.
    pub built_for: Cell<Option<(usize, u64)>>,
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
    pub flags: RefCell<Option<Vec<bool>>>,
}

/// The document's font inventory, held for as long as the document is open.
#[derive(Default)]
pub struct FontCache {
    /// The `edit_epoch` the inventory below describes, or `None` before the
    /// first build. See [`PageObjectCache::built_for`] for the borrow
    /// argument this `Cell` is half of.
    pub built_for: Cell<Option<u64>>,
    /// The inventory. `pdfcer_core::fontinfo::inventory` is **infallible** —
    /// it reports problems in its `diagnostics` rather than in a `Result`
    /// (core API trap T-9.8) — so there is no error arm here, and an empty
    /// inventory does not mean a clean document.
    pub inventory: RefCell<Option<FontInventory>>,
}

/// **The provenance-bearing text of one page**, keyed by `(page, edit epoch)`.
#[derive(Default)]
pub struct ProvenanceTextCache {
    /// The `(page index, edit epoch)` the text below describes, or `None`
    /// before the first build.
    ///
    /// **What actually makes a failed extraction cheap, and it is NOT the
    /// order this line is written in.**
    ///
    ///
    /// It could not. The value store at the end of
    /// `OpenDoc::ensure_provenance_text` is *unconditional* — a failed
    /// extraction stores `None`, and the key is recorded either way — so both
    /// orders record the attempt and both make the next frame a `Cell` read.
    /// What stops the retry is that **the key is recorded at all, on the
    /// failure arm as well as the success arm**. That is the property to
    /// preserve: an early `return` on failure, before the key is set, would
    /// reintroduce the third-of-a-second-per-frame cost the sibling comment
    /// warns about — and would do it while leaving that comment looking
    /// satisfied.
    ///
    /// The order is kept anyway, for a smaller reason that is real: it makes a
    /// re-entrant ask for the same page terminate (answering `None`) rather
    /// than recurse.
    pub built_for: Cell<Option<(usize, u64)>>,
    /// The extraction, or `None` when it failed or the page does not exist.
    ///
    /// `None` here means **"could not be measured"**, never "this page has no
    /// text" — a page with no text extracts successfully to an empty
    /// `PageText`. Every caller reads it as *"there is nothing to edit"*,
    /// which is the correct reading of both, but a caller that ever needs to
    /// tell them apart must not infer one from this field.
    pub text: RefCell<Option<Rc<PageText>>>,
}
