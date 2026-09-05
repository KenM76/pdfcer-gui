//! # `app::state` — what is open
//!
//! One thing lives here: [`Status`] and [`OpenDoc`] — the shape of "what, if
//! anything, is open", and everything one open document owns.
//!
//! ## What is NOT here, and where it went
//!
//! The **raster bookkeeping** — the per-frame decision about whether the
//! cached page texture is still a picture of what the operator is looking at,
//! and if not whether to re-rasterize now or wait for a zoom gesture to settle
//! — was the second half of this file until Phase 4 and now lives in
//! [`crate::render::settle`]. Phase 4 made it considerably larger (one texture
//! became a texture plus a bounded strip cache, and one staleness question
//! became two), and the seam is the one this header had already named:
//! everything here answers *"what is open, and what is the operator looking
//! at?"*, and everything there answers *"what does the picture need to be, and
//! what should be done about it this frame?"*. Only the second belongs in
//! `render/`, beside the worker it schedules.
//!
//! The **fields** it reads are still declared below, because that is what
//! bounds their lifetime and is the whole point of their living on the
//! document; only the methods moved.
//!
//! The two **derived caches** an open document owns — the page decomposition
//! and the font inventory — live in [`crate::app::cache`], along with the
//! four methods that read them. Their fields are still declared on [`OpenDoc`]
//! below, because that is what bounds their lifetime and is the whole point of
//! their move onto the document; only the types, the accessors and their
//! argument moved out.
//!
//! The seam is a real one rather than a cut made to satisfy rule R2's
//! 1,500-line gate: everything left in this file answers *"what is open, and
//! what is the operator looking at?"*, while everything in `cache.rs` answers
//! *"what expensive derived value do several surfaces need, and how is it
//! computed once?"* — a different question, with its own shared hazard
//! (staleness against [`OpenDoc::edit_epoch`]), its own shared device (a `Cell`
//! key beside a `RefCell` payload) and its own documented exemption from the
//! `&mut`-only mutation rule. `cache.rs`'s header carries that argument in
//! full. Splitting anywhere else would have left half an explanation on each
//! side of the cut.
//!
//! ## Three ways to fail, three ways to say so
//!
//! [`Status`] distinguishes what most viewers conflate, and the
//! distinction is carried across from the old shell because it is one of
//! the things pdfcer does that its competitors do not:
//!
//! - [`Status::Failed`] — the *file* is wrong: damaged, truncated, not a
//!   PDF. "Something is wrong with your document."
//! - [`Status::Unsupported`] — the file is fine and **pdfcer** is not
//!   finished. `pdfcer-core` detects such a file and refuses it cleanly
//!   rather than misparsing it into plausible-looking garbage. Presenting
//!   that as "failed to open" would tell the operator a lie about their own
//!   file.
//! - [`Status::NeedsPassword`] — a third thing again: pdfcer *can* decrypt
//!   this document and has not been told how.
//!
//! The branch between them is made on **structured error data** from
//! `pdfcer-core`, never by matching on a message string. That is exactly what
//! "core errors are stable, structured diagnostics" is *for*, and it is what
//! makes the distinction reliable rather than a heuristic that decays as error
//! prose is edited.
//!
//! ★ The branch itself now lives in [`crate::app::lifecycle`], with
//! `is_unsupported_structure` and the two methods that move `Status` between
//! these variants. This file kept the **shape** of the answer (the enum, and
//! why it has these variants); that one has *when each one is produced*. See
//! that module's header for the seam.
//!
//! ## Rendering happens on state change, never per frame
//!
//! egui redraws continuously; rasterizing a PDF page at 60 Hz would be
//! absurd. The canvas holds one cached [`PageTexture`] for the **current
//! page**, plus — under a continuous page-display mode only — a bounded cache
//! of the *other* visible pages ([`OpenDoc::strip_rasters`]). Each texture
//! carries the [`RenderKey`] it was rendered from — page, raster scale,
//! annotation visibility, layer-override generation — and staleness is that
//! key compared against the one the view wants, with deliberately no second
//! field list to keep in step with it (see [`RenderKey`]'s own docs; a key
//! compared on one side and not the other is a control that ticks and redraws
//! nothing).
//!
//! **The comparison, the zoom debounce and the strip's scheduling all live in
//! [`crate::render::settle`]**, which carries the full argument for both
//! staleness policies. What lives here is the state they read.
//!
//! [`OpenDoc`] also carries the page decomposition and the font inventory,
//! moved off `crate::panels::PanelsState` at S4 so the document's own
//! lifetime bounds them and no identity key is needed — see
//! [`OpenDoc::page_objects`] — and, from the same stage, the **canvas
//! selection**, moved off `egui::Memory` for the same reason. See
//! [`OpenDoc::selection`].

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use pdfcer_core::edit::EditSession;
use pdfcer_core::object::ObjId;
use pdfcer_core::page_tree::Page;

use crate::app::cache::{FontCache, PageObjectCache, PageTextCache};
use crate::canvas::selection::SelectionState;
use crate::render::raster::PageTexture;
use crate::render::worker::{RenderKey, RenderRequest, RenderWorker};
use crate::viewer::{self, ViewState};

/// What, if anything, is open.
///
/// `Box`ed in the `Open` arm because [`OpenDoc`] is much larger than the
/// error arms and an un-boxed enum would make every `Status` move copy it.
#[derive(Default)]
pub enum Status {
    /// Nothing open. The start-up state when no path was given on the
    /// command line.
    #[default]
    Empty,
    /// A document is open.
    Open(Box<OpenDoc>),
    /// The file is damaged, truncated, or not a PDF.
    Failed { path: PathBuf, message: String },
    /// The file is well-formed and uses something pdfcer does not implement.
    Unsupported { path: PathBuf, message: String },
    /// The file is encrypted and pdfcer has not been given the password.
    NeedsPassword { path: PathBuf },
}

/// Where the pointer was over the page when a Ctrl+wheel arrived.
///
/// ★ **Re-exported, not declared here.** The type moved to [`crate::viewer`]
/// when the rulers landed — R2's ceiling forced a split out of this file, and
/// a zoom fact belongs beside `ViewState::zoom` and `ZOOM_LADDER`; that
/// module's own docs carry the argument. The re-export keeps it a *move*
/// rather than a rename, so `canvas::zoom` still names it by this path.
pub use crate::viewer::ZoomAnchor;

/// Which optional-content groups the operator has hidden, if any.
///
/// # ★ `None` is not "hide nothing"
///
/// `pdfcer_render::LayerVisibility` **replaces** the document's own default
/// configuration rather than merging with it (core API trap T-12.9). So:
///
/// | state | meaning |
/// |---|---|
/// | `hidden: None` | obey the document's `/D` configuration (§8.11.4.3) |
/// | `hidden: Some({})` | show **every** layer, including ones the document turns off |
/// | `hidden: Some({…})` | exactly these are hidden |
///
/// Collapsing the first two would silently reveal every layer a document had
/// turned off, which on a drawing with a "Confidential" watermark layer is a
/// disclosure defect rather than a cosmetic one.
///
/// That is also why a set is stored rather than operator *deltas*: the
/// renderer wants the complete answer, so the complete answer is what is
/// held. A delta would have to be resolved against the document's defaults at
/// render time, in a second place, with the merge rules the engine
/// deliberately refused to define.
///
/// # ★ The operator's toggle is session-only, and nothing here can save it
///
/// §8.11.2.1 puts the live state outside the document entirely: the toggle is
/// *"session-only state, held nowhere the save path can see it"*, lost on
/// reopen. That is a property of the format rather than a gap in this build,
/// it is what `crate::text::panels::layers_session_only_note` discloses, and
/// it is why changing it must not bump [`OpenDoc::edit_epoch`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct LayerOverride {
    /// The complete hidden set, or `None` to obey the document.
    pub(super) hidden: Option<BTreeSet<ObjId>>,
    /// How many times the above has changed.
    ///
    /// The render staleness key — see
    /// [`crate::render::worker::RenderKey`], whose own docs explain why a
    /// counter beats comparing the set on every frame. `0` is the
    /// never-touched state, which is exactly `hidden: None`.
    pub(super) generation: u64,
}

mod identity;
/// **What the render tier needs to know about a page's colour** — one method,
/// split out 2026-09-01 under R2. Its header carries why the ink question is a
/// different subject from the document model around it.
mod ink;
pub mod pageepoch;

pub use identity::{Origin, SelectedField};
/// One open document and everything the shell knows about looking at it.
pub struct OpenDoc {
    /// ★ **The operator's configuration, as this document's derived data was
    /// computed under it.**
    ///
    /// # Why a snapshot lives here at all, when the live answer is on `PdfcerApp`
    ///
    /// Because everything this type caches was produced under *some* settings,
    /// and the question every consumer actually asks is not "what are the
    /// settings?" but "what were the settings when this cache was filled?".
    /// Five of the thirteen change what a rasterization looks like and three
    /// change what an extraction produces, so a page texture, a strip entry and
    /// a page-text cache are all derived values with a configuration baked into
    /// them.
    ///
    /// Keeping the snapshot beside them makes that explicit and makes the
    /// invalidation one act rather than two: `PdfcerApp::adopt_settings` writes
    /// this field **and** drops every cache in the same function, so there is
    /// no state in which the snapshot and the derived data disagree. A
    /// threaded `&Settings` would give the *renderer* the right answer while
    /// leaving a cached texture that was drawn under the old one — the same
    /// answer arriving by two routes, which is the shape of bug this project
    /// keeps finding.
    ///
    /// # It starts as the shipped defaults, and that is not the operator's
    ///
    /// `assemble` cannot reach `PdfcerApp`. Every real open path calls
    /// `adopt_settings` immediately afterwards, and
    /// `PdfcerApp::tests::opening_a_document_gives_it_the_operators_settings`
    /// is what stops a fourth open path forgetting to.
    pub(crate) settings: pdfcer_core::settings::Settings,
    /// ★ The shell's own preferences, snapshotted for the same reason
    /// [`Self::settings`] is and updated in the same one function.
    ///
    /// `render_quality` multiplies the raster scale, so it is baked into every
    /// cached texture exactly as the engine's five rendering settings are —
    /// which makes it the same kind of value and gives it the same home. A
    /// second mechanism for a second store would be two things to keep in step
    /// with one set of caches.
    pub(crate) prefs: crate::app::prefs::Prefs,
    /// Where it came from — or, for a created document, what it is called.
    ///
    /// Read by [`PdfcerApp::open_path`], which hands it to
    /// [`crate::app::recent::RecentFiles::remember`] — so the recent list is
    /// built from the path the document was *actually* opened from rather
    /// than from whatever the caller happened to have, which is the same
    /// distinction that makes the recording conditional on the open having
    /// succeeded. Still also what the window title becomes (`<file> — pdfcer`)
    /// and what a document switcher will need.
    ///
    /// ★ **It is not always a location.** `file.new` sets it to a name, and
    /// [`Self::origin`] is what says which of the two this is. Every consumer
    /// that treats it as an *identity* or a *label* — the forms cache key, the
    /// Pages panel caption, the trace — is correct either way. Every consumer
    /// that treats it as a **file** must go through [`Self::stored_under`].
    pub path: PathBuf,
    /// Whether [`Self::path`] names a file, or only names the document.
    ///
    /// Set once at construction and never changed: a created document that
    /// gains a file gains it through a save, and this build has no save that
    /// gives it one.
    ///
    /// ★ **`file.save_copy` landed on 2026-08-14 and is deliberately not that
    /// save.** *Save a copy* writes the document somewhere and leaves the
    /// document alone — Inkscape's verb, and the only one of the three
    /// reference applications that has it — so a created document saved to
    /// `D:\jobs\sheet.pdf` is still called `Untitled 1.pdf` afterwards and still
    /// reads [`Origin::Created`] here. The save that would write this field is
    /// `file.save_as`, which does not exist. See [`crate::app::save`] §3.4,
    /// which carries the argument and the reason writing it here would rename
    /// the operator's open document because they asked for a copy.
    pub origin: Origin,
    /// The edit session — the single owner of the document, through which
    /// every future mutation will pass.
    ///
    /// **An `Arc` for one specific reason**: a render worker holds a clone
    /// for as long as it rasterizes, so the borrow can cross the thread
    /// boundary and `session.view()` can be called on the far side. That is
    /// also why every future mutation must go through a path that first
    /// calls [`RenderWorker::cancel_and_wait`] — `Arc::get_mut` fails while
    /// a render is running, and the alternatives were rejected with numbers
    /// (see that method).
    pub session: Arc<EditSession>,
    /// The flattened page vector, resolved once at open.
    pub pages: Vec<Page>,
    /// Which page, at what zoom, chosen how.
    pub view: ViewState,
    /// The **current page's** cached raster, or `None` before the first one
    /// arrives.
    ///
    /// Its meaning is unchanged by Phase 4 and deliberately so: three surfaces
    /// outside this module depend on this field and on this spelling —
    /// `crate::app::status` reads its `diagnostics`, and both
    /// `crate::app::actions`' `vector_edit` and `crate::panels::forms::edit`
    /// invalidate it by assigning `None`. None is about a strip. The rule that
    /// split buys, enforced rather than remembered: **the current page is never
    /// in [`Self::strip_rasters`]** — see
    /// [`crate::render::strip::StripRasters`], which carries the argument.
    pub page_texture: Option<PageTexture>,
    /// The [`Self::edit_epoch`] [`Self::page_texture`] is a picture of.
    ///
    /// # ★ Why this exists: the blank flash after every edit
    ///
    /// `RenderKey` compares page index, raster scale, annotation stance and
    /// layer generation — **not the edit epoch**, because an edit changes none
    /// of them. So until 2026-08-18 the only way an edit could make the canvas
    /// re-render was to assign `page_texture = None`, and every writer did
    /// exactly that.
    ///
    /// Nulling it is also what put a **blank page on screen between the edit
    /// and the next raster**. The operator: *"the page goes blank and flashes
    /// after every change instead of just writing and rendering the change."*
    ///
    /// The strip cache never had this problem — `StripRasters` keys on
    /// `(page, key, epoch)` and has since it was written. This field is the
    /// current page's missing third term, so the same question can be asked
    /// the same way: *is the picture on screen a picture of the revision the
    /// operator is looking at?* A "no" now requests a fresh raster **and keeps
    /// showing the old one until it arrives**, which is what
    /// `OpenDoc::rasterize`'s own docs already promised for the slow-render
    /// case: *"the previous texture staying on screen meanwhile."*
    ///
    /// # What still drops the texture, and why that is not this
    ///
    /// A **page-set change** — delete, reorder, insert. There the stale raster
    /// is a picture of a *different sheet*, not an older revision of the same
    /// one, so showing it would be wrong rather than merely late.
    /// `actions::pages::resync` drops it on exactly that condition, beside the
    /// strip cache and the selection it drops for the same reason.
    pub page_texture_epoch: u64,
    /// ★★★ **The preview that outlives the gesture**, held until the page
    /// catches up (`OPERATOR_REQUESTS.md` O63, third piece).
    ///
    /// The whole argument — what it removes, why holding a picture of a
    /// committed edit is honest rather than optimistic, and the three-clause
    /// liveness rule with its two wall-clock bounds — lives in
    /// [`heldpreview`](super::state::heldpreview), beside the code that
    /// enforces it. Do not restate it here; one fact, one place.
    pub(crate) held_preview: Option<HeldPreview>,
    /// When the last edit bumped [`Self::edit_epoch`].
    ///
    /// ★ For one question and one only: **has the picture been behind long
    /// enough to be worth saying so?** See
    /// [`page_is_catching_up`](Self::page_is_catching_up); the argument is in
    /// [`heldpreview`](super::state::heldpreview)'s header beside the rest of
    /// O63's timing.
    pub(crate) last_edit_at: Option<std::time::Instant>,
    /// **The other visible pages' rasters**, under a continuous mode.
    ///
    /// Empty for the whole of a single-page session — which is the mechanical
    /// form of "continuous is an option, not a replacement": the default path
    /// allocates nothing here and runs the code it ran before Phase 4. Bounded
    /// by [`crate::render::strip::MAX_CACHED_TEXELS`] and pruned to the
    /// visible set every frame by [`crate::render::settle`].
    pub strip_rasters: crate::render::strip::StripRasters,
    /// **Which pages the canvas drew this frame**, nearest the viewport
    /// centre first.
    ///
    /// Published by [`crate::canvas::show`] during layout and read by
    /// [`crate::render::settle`] after the frame, because "which pages are on
    /// screen" is only knowable once the scroll area has settled — the same
    /// reason [`Self::last_scroll_offset`] is stored rather than derived. It is
    /// the **complete** input to the strip's scheduling: what to keep, what to
    /// evict and what to render next all come from this list and the current
    /// page index. Empty means single page, and the strip pass returns at once.
    pub strip_visible: Vec<usize>,
    /// **The page index the canvas last derived from the scroll position.**
    ///
    /// The one piece of state that tells a *navigation* apart from a *scroll*
    /// under a continuous mode, which matters because both write
    /// [`crate::viewer::ViewState::page_index`]: the canvas writes it every
    /// frame from where the operator has scrolled to and records the same value
    /// here, while a page **command** writes it and not this. So
    /// `page_index != tracked_page` means exactly one thing — something other
    /// than the scroll asked for a different page. See
    /// [`crate::canvas::strip::page_scroll_offset`], which is its only reader
    /// and carries the full argument.
    pub tracked_page: usize,

    /// How many frames the canvas has drawn for this document, saturating.
    /// O23: with a pasteboard, egui's initial zero is the content's origin
    /// rather than the strip's, so the view is seeded once.
    pub canvas_frames: u8,
    /// **Wheel travel not yet spent on a page turn**, in logical points —
    /// `OPERATOR_REQUESTS.md` O30.
    ///
    /// Only ever non-zero while [`crate::app::prefs::WheelPaging::FlipPages`]
    /// is on and the display mode is not continuous.
    ///
    /// ★ An accumulator rather than "one event, one page", because the two
    /// devices that produce a wheel do not agree on what an event is. A mouse
    /// delivers one detent as one large delta; a trackpad delivers a swipe as
    /// dozens of small ones. Counting events would turn a single trackpad
    /// gesture into forty page turns, and thresholding an instantaneous delta
    /// would make a slow, deliberate scroll do nothing at all. Travel is the
    /// quantity both devices agree on.
    pub wheel_travel: f32,
    /// What the render worker was rendering when this frame's poll took its
    /// slot.
    ///
    /// Read *before* [`crate::render::worker::RenderWorker::poll`] and
    /// consumed by the absorb, for one specific reason: a render **failure**
    /// arrives as a bare message with no [`RenderKey`], so the page it is
    /// about is only knowable from the slot it came out of. Without this a
    /// strip page that would not draw would be attributed to the current page
    /// and blank the whole canvas.
    pub render_in_flight: Option<RenderKey>,
    /// Why the current page would not draw, if it would not.
    ///
    /// Held rather than propagated: the document is still open and the
    /// operator can still navigate away from a page that will not draw.
    ///
    /// **The current page's**, and only its. A strip page that will not draw
    /// records its refusal in [`Self::strip_rasters`] and says so in its own
    /// rect, because one bad sheet in a forty-page set must not replace the
    /// other thirty-nine with a message.
    pub render_error: Option<String>,
    /// **How many fonts the last mark-by-search in this document could not
    /// read**, or 0 — Pass 127.1's disclosure.
    ///
    /// ★★★ Why a redaction owes this. An empty match list has two causes with
    /// one appearance: the term is not in the document, or the document's text
    /// was never recoverable as Unicode so no term could ever have matched it.
    /// For a search that ambiguity wastes a minute. **For a redaction it fails
    /// in the direction nobody catches** — the operator asked for every
    /// occurrence of a name to be removed, the run reported success, the file
    /// still contains it, and then they send it. Both populations render
    /// perfectly, which is exactly what makes it invisible.
    ///
    /// ★ On the DOCUMENT rather than on the redact panel, and that placement is
    /// the decision rather than a convenience. It is a fact about *this file's
    /// fonts*, so it must follow the file: parked with it, restored with it,
    /// and — the case that decides it — **not shown against a different
    /// document** when the operator switches tabs. Panel state would have
    /// leaked one document's warning onto another's.
    pub last_redaction_unreadable_fonts: u64,
    /// ★★★ **The strings a redaction ARMED on this document claims it will
    /// remove** — `RedactionReport::redacted_text`, empty on every document
    /// that has no removal armed.
    ///
    /// Added 2026-09-04 with the collapsing apply; **re-based 2026-09-05** onto
    /// `Action::PendingRedaction(Staging::Stage)` when `pdfcer-core`
    /// `Pass 250.2` replaced that route with an undo-preserving one. The old
    /// sentence said *applied INTO this document*, which is no longer what
    /// happens: nothing is removed until the save.
    ///
    /// It exists for one consumer: `crate::app::save::write_copy` greps the
    /// bytes it is about to write for these strings and **refuses the save** if
    /// any of them survives in a decoded stream. That is the shell's
    /// independent absence proof, kept alive on the deferred route.
    ///
    /// ★ Why it has to be carried rather than re-derived. The write-now routes
    /// prove the buffer inside `crate::redact::PreparedRedaction::write_to`,
    /// one statement from the syscall. The deferred route has no such buffer at
    /// arming time — the bytes are built minutes later, by whichever save verb
    /// the operator reaches for, possibly after further edits — so the only
    /// thing that can travel from the arming to the save is *what will be
    /// removed*. `EditSession::has_pending_redaction()` says **that** a removal
    /// is armed and cannot say **what**, which is not enough to prove anything.
    ///
    /// ★★ On the DOCUMENT rather than in a global, for
    /// [`Self::last_redaction_unreadable_fonts`]'s reason and more sharply: it
    /// is a fact about this file's content, and proving one document's claims
    /// against another document's bytes would be a refusal to save a perfectly
    /// good file — or, in the direction that matters, a clean pass on a file
    /// nobody checked.
    ///
    /// ★★★ **It is not cleared by a save, and IS cleared by a cancel** — the
    /// second half is new on 2026-09-05 and it is not tidiness. These strings
    /// are this shell's statement that every file it writes for this document
    /// has that text removed from it, and after
    /// `Staging::Cancel` that statement is false: the content is deliberately
    /// still there. Leaving them set would make the next ordinary save refuse
    /// itself, correctly, over a removal the operator called off on purpose —
    /// with no way out but to close the document.
    pub redaction_absence_claims: Vec<String>,
    /// The single-slot background rasterizer.
    pub render_worker: RenderWorker,
    /// The zoom seen at the end of the previous frame, used to detect that
    /// the zoom changed at all.
    pub observed_zoom: f32,
    /// The earliest instant at which the current zoom may be committed to a
    /// real rasterization — the [`ZOOM_SETTLE`] debounce deadline.
    pub zoom_commit_at: Instant,
    /// Set by any *discrete* zoom command during this frame's action
    /// dispatch, and consumed at the end of the frame. It is what
    /// distinguishes "the operator pressed Ctrl+0" (commit at once) from
    /// "the operator is mid-wheel-gesture" (wait for the gesture to settle).
    pub zoom_commanded: bool,
    /// See [`ZoomAnchor`]. Written by the canvas, consumed by the canvas on
    /// the following frame.
    pub zoom_anchor: Option<ZoomAnchor>,
    /// **A fit command is waiting to have its view placed** —
    /// `OPERATOR_REQUESTS.md` O28.
    ///
    /// Set by `Action::Fit` for any mode that pins an axis, consumed by
    /// `canvas::show` on the next frame, when the re-fitted zoom has landed
    /// and the page's new drawn size is known. Exactly the shape
    /// [`ZoomAnchor`] uses, and for the same reason: the new zoom is not
    /// known when the command is raised.
    ///
    /// ★ It is a **separate** one-shot from the zoom anchor rather than an
    /// anchor of its own, because the two are answering different questions.
    /// An anchor says *"hold this page point where it is"*; a fit says
    /// *"decide where the page goes"*, and on a pinned axis there is no
    /// previous position worth holding — that is what pinning means. Folding
    /// it into `ZoomAnchor` would need a per-axis "ignore the anchor" flag,
    /// which is a second mechanism wearing the first one's name.
    ///
    /// ★★ It **outranks** a pending zoom anchor and spends it. A fit is the
    /// operator's most recent explicit instruction about the view, and a
    /// wheel anchor armed a frame earlier describes a position the fit has
    /// just superseded. See the offset-decision chain in `canvas::show`.
    pub fit_placement: Option<crate::viewer::FitMode>,
    /// **Where a bookmark asked the view to land.** A one-shot, parked on
    /// `fit_placement`'s pattern; see `canvas::destination`.
    pub pending_destination: Option<crate::canvas::destination::PendingDestination>,
    /// **The viewport the active fit was last placed against**, so a resize
    /// can re-place the page and nothing else can — `OPERATOR_REQUESTS.md`
    /// O55.
    ///
    /// ★ The argument for a remembered SIZE rather than re-placing every frame
    /// lives in [`crate::canvas::fit::placement`], beside the comparison that
    /// reads it: re-placing per frame pins the wheel under Fit page and makes
    /// a continuous document unscrollable, which is a fact about that
    /// function's behaviour rather than about this field's shape.
    ///
    /// `None` until the first placement, which is what a fresh document gets,
    /// so the first placement always happens.
    pub view_viewport: Option<(f32, f32)>,
    /// **A search hit that has been navigated to and is waiting to be
    /// scrolled into view.** See [`crate::find::Reveal`].
    ///
    /// Sits here beside [`Self::zoom_anchor`] because it is the same *kind* of
    /// thing for the same reason: per-document **view** bookkeeping that has
    /// to span two frames, because the page it targets has not been navigated
    /// to yet on the frame the request is made. Written by
    /// `crate::find::apply` during the apply phase, consumed by
    /// `crate::canvas::show` on the first frame that is actually showing the
    /// hit's page, and abandoned if that frame never comes — the same
    /// hold/solve/drop shape `zoom::anchor_step` documents, minus the zoom.
    ///
    /// On `OpenDoc` rather than on `FindState` — which owns everything else
    /// about Find — by this struct's own rule: state that dies with the
    /// document lives here. A pending scroll position is a fact about *this*
    /// document's pages and means nothing in another one, and putting it here
    /// makes forgetting it free rather than another `forget_document` line
    /// somebody has to remember.
    pub find_reveal: Option<crate::find::Reveal>,
    /// Bumped by **every action that changes the document's content**.
    ///
    /// Not a document version in any meaningful sense — it is a cheap "has
    /// anything been edited since I last looked?" token, and the only thing
    /// that reads it is [`OpenDoc::trace_object_count`]. It exists because
    /// the object count that `PROJECT_PLAN.md` §4.3 requirement 3 asks for is
    /// **expensive to compute** (`decompose_page` fully decodes, tokenizes
    /// and walks every content stream on the page, with no cache anywhere in
    /// `pdfcer-core`), so it must not be recomputed per frame, and the only
    /// honest cheap test for "is my last count still true?" is "has an edit
    /// happened".
    ///
    /// **Stage S2 never bumps it**, because stage S2 has no editing action —
    /// `Action` is zoom and page navigation, and both leave the content
    /// untouched. This field is therefore the documented *seam*: the first
    /// mutating arm added to [`crate::app::actions::PdfcerApp::apply`] must
    /// bump it, and the count then re-traces on the following frame with no
    /// further work. A mutating action that forgets is a stale `objects=`
    /// line, which is why the bump belongs in the funnel every mutation is
    /// already required to pass through rather than at each verb.
    pub edit_epoch: u64,
    /// **The engine's content digest for one page, measured on a frame when
    /// this shell held the session exclusively.**
    ///
    /// `(page, edit_epoch_at_the_time, generation)`.
    ///
    /// # ★★★ Why all three, and why the middle one is the safety property
    ///
    /// `EditSession::page_content_generation` became `&mut self` on 2026-09-01
    /// — unavoidably, and for the reason this shell itself gave the engine:
    /// the number has to walk the page to know which forms it descends into,
    /// and walking populates a memo. But `crate::app::cache`'s decomposition is
    /// built from `&self`, behind an `Arc` the render worker also holds, so the
    /// read cannot happen there.
    ///
    /// ⇒ So it is measured once per frame at a `&mut` point and read from here.
    /// `Arc::get_mut` can fail — a render in flight holds a second handle — so
    /// the measurement can be **missed**, and a missed measurement after an
    /// edit would leave a stale digest describing a page that has changed.
    /// `PageObjects` addresses content by index, so serving a model built from
    /// it would edit whatever that index names in the wrong model.
    ///
    /// **The epoch stamp is what makes that unrepresentable.** The digest is
    /// used only while the epoch it was measured at is still the current one;
    /// any edit bumps the epoch, so a digest taken before it is ignored and the
    /// key falls back to the epoch — which rebuilds, exactly as this shell did
    /// before any of this existed. Slow is the safe direction, and it is the
    /// only direction reachable when the measurement is missed.
    pub content_generation: std::cell::Cell<Option<(usize, u64, u64)>>,
    /// ★ **The same question, asked per page** — `OPERATOR_REQUESTS.md` O74.
    ///
    /// [`edit_epoch`](Self::edit_epoch) above says *something changed*; this
    /// says *what changed*, for the caches that hold one entry per page and
    /// were throwing all of them away on every edit. It is laid **beside**
    /// `edit_epoch`, not in place of it — every existing reader of that field
    /// is unaffected. `crate::app::state::pageepoch` carries the measurements,
    /// the three caches involved, and the argument for why `bump_all` is the
    /// default and precision is opted into per verb.
    pub page_epochs: pageepoch::PageEpochs,
    /// The `(page index, edit epoch)` the last `objects` line was traced for,
    /// or `None` before the first one.
    ///
    /// Both halves are needed: the count is a property of *this page* in
    /// *this revision*, so paging away and back must re-trace (the count is
    /// different) and an edit must re-trace (the count may be different).
    pub objects_traced_for: Option<(usize, u64)>,
    /// The scroll offset the canvas settled on at the end of the last frame.
    ///
    /// Kept because middle-drag panning has to compute "where the view
    /// should be now" BEFORE the scroll area is built, and the area's own
    /// state is only readable after. Storing last frame's settled value
    /// lets the pan be applied in the same frame as the movement rather
    /// than a frame late — which is the difference between panning that
    /// tracks the hand and panning that lags it.
    pub last_scroll_offset: egui::Vec2,

    /// ★★ **Which page-space rectangle to rasterize, and for which page** —
    /// `OPERATOR_REQUESTS.md` O24's region tier.
    ///
    /// `None` is the whole-page path this shell has always taken. `Some` is
    /// set by [`crate::canvas::show`] once the page's own raster would
    /// exceed `MAX_PIXMAP_EDGE` — the failure the operator hit at 2382 %:
    ///
    /// > *"requested raster size 14580x18868 is empty or exceeds
    /// > MAX_PIXMAP_EDGE"*
    ///
    /// ★ Written by the canvas rather than derived here, for the same reason
    /// [`Self::last_scroll_offset`] is: only the canvas knows where the
    /// operator is looking, and that is what decides the rectangle.
    ///
    /// ★★ It carries its **page index**, and that is not decoration. A
    /// region is in one page's own coordinate space, so applying it to a
    /// neighbouring page in a continuous strip would rasterize the wrong
    /// part of it — silently, because both are valid rectangles. The index
    /// makes the mismatch impossible rather than merely unlikely.
    pub raster_region: Option<(usize, pdfcer_core::page_tree::Rect)>,

    /// ★★ **Where the view is, once the scroll offset can no longer say** —
    /// O24 tier 3.
    ///
    /// `None` below the sub-pixel content extent, where `egui::ScrollArea`'s
    /// own `f32` offset is authoritative and nothing about the canvas differs
    /// from before this feature. `Some` above it, where the position is a page
    /// point in `f64` and the screen pixel it sits under.
    ///
    /// ★ Seeded on the way in from wherever the scroll area had settled, and
    /// cleared on the way out — so crossing the threshold in either direction
    /// does not move the page under the operator, and re-entering starts from
    /// the truth rather than from a stale anchor.
    pub deep_anchor: Option<crate::viewer::deep::DeepAnchor>,
    /// The zoom [`Self::deep_anchor`] was last valid at, or `None` outside the
    /// deep tier.
    ///
    /// ★★ **What makes zoom-to-cursor possible above the threshold.**
    /// `DeepAnchor::zoomed_about` needs the zoom the anchor was written at, so
    /// it can read which page point sits under the cursor *before* re-stating
    /// the anchor at the new scale. The anchor itself deliberately does not
    /// carry a zoom — it is a statement about page space and screen space, and
    /// baking a scale into it would make it stale rather than merely
    /// unfashionable. So the canvas remembers the scale beside it.
    ///
    /// `OPERATOR_REQUESTS.md` O24f: without this the anchor never moved on a
    /// zoom, the anchored page point stayed nailed to the viewport's top-left,
    /// and everything the operator was looking at expanded off the screen.
    /// Cleared on leaving the tier so the first frame back inside seeds from
    /// the scroll area rather than re-anchoring against a scale from minutes
    /// ago.
    pub deep_zoom: Option<f64>,
    /// ★ **What the operator has selected on the canvas.**
    ///
    /// # Why it is a field of the document rather than a value in `egui::Memory`
    ///
    /// It was the latter until this stage, and the move is the same argument
    /// the decomposition cache's move made, applied to state rather than to a
    /// cache. A selection is **document-scoped**: closing a document must
    /// forget it, and a selection retained across an open would name paint-order
    /// indices in a file that no longer exists.
    ///
    /// `egui::Memory` outlives documents, so the canvas had to *detect* the
    /// change — with a `DocumentToken` built from the `Arc<EditSession>`'s
    /// allocation address mixed with the page count, compared once per frame,
    /// forgetting everything when it moved. That is the identical shape, and
    /// the identical hazard, as the `panels::DocKey` this stage deleted: an
    /// address is not an identity, a freed allocation can be reused, and
    /// holding an `Arc` or a `Weak` to make it one would disable editing
    /// outright (`Arc::get_mut` fails while any other strong **or weak**
    /// reference exists — see [`Self::session`]).
    ///
    /// Here the question does not arise. [`Self::new`]'s own doc comment is the
    /// whole proof: *"opening a document constructs a whole new `OpenDoc`, so a
    /// cached texture or a page index can never refer to a page from a previous
    /// file."* A selection **inside** that structure inherits the guarantee for
    /// free, by construction rather than by comparison, on every frame, at zero
    /// cost. So `DocumentToken` and `SelectionState::sync_document` were
    /// deleted rather than repaired.
    ///
    /// # ★ Public, and why that does not breach actions-not-mutations
    ///
    /// `crate::app::actions`' invariant is that **no code path runs from a
    /// widget to a *document***. A selection is not the document: it names
    /// parts of it and changes nothing that a save would write. It sits with
    /// [`Self::last_scroll_offset`] and [`Self::zoom_anchor`] as per-document
    /// *view* state the canvas is permitted to write directly, and for the same
    /// reason they are — it is settled during the frame, from input that only
    /// exists during the frame, and deferring it would make a click land one
    /// frame after the operator made it.
    ///
    /// The **edit** that a selection leads to is still an action, and that is
    /// the line: Delete does not remove anything here. It raises
    /// [`crate::app::actions::VectorAction::DeleteSelection`] carrying the operand
    /// list, applied after the frame through the one funnel, exactly as before
    /// the move. Nothing that touches `EditSession` moved.
    ///
    /// # Who writes it
    ///
    /// `crate::canvas` — the click, marquee, Escape and Delete gestures — and
    /// nothing else. Everything outside the canvas **reads**: the ribbon's
    /// `selection.any` condition, the Format tab's Delete, and (from the next
    /// stage) the Properties panel. A second writer would be a second
    /// selection model, which is precisely what
    /// `panels::PanelsState::focus`'s docs refuse to become.
    pub selection: SelectionState,
    /// ★★ **The form field selected for editing its properties**, if any.
    ///
    /// The operator, 2026-08-26: *"when I click on an existing form field on
    /// the page its properties should come up in our side pane for editing its
    /// properties."* This is what a click on a widget in Edit mode sets, and
    /// what `panels::properties::formfield` reads.
    ///
    /// # ★ Why it is NOT part of [`SelectionState`]
    ///
    /// Because it is not the same kind of thing, and merging them would make
    /// three surfaces lie. `SelectionState` holds a **page object or an
    /// annotation** — something with paint-order indices, a bounding box, drag
    /// handles, a Format tab and a Delete that means "remove this drawing". A
    /// form field is none of those: it is a document-level entry in `/AcroForm`
    /// that may have several widgets on several pages, its identity is a
    /// **name** rather than an index, and deleting it is a different verb with
    /// a different meaning.
    ///
    /// Folding it in would arm the Format tab's Delete over a field, hand the
    /// resize grips a rectangle that `move_nodes` cannot move, and make
    /// `selection.any` true for something none of the selection-scoped commands
    /// can act on. Separate field, separate surface, no overlap.
    ///
    /// # ★★ It is cleared by an edit, not merely allowed to go stale
    ///
    /// The handle is a fully-qualified name, and a rename or a delete changes
    /// which field that name reaches — or whether it reaches one at all. See
    /// `app::actions::forms`'s selection arm: every verb that touches the form
    /// clears this, so the panel cannot go on describing a field that no longer
    /// exists under that name.
    /// ★★★ **The backdrop — the last small whole-page raster, kept so the page
    /// is never blank while a sharper one is on its way.**
    ///
    /// The operator, 2026-08-26: *"the screen should never be blank while
    /// waiting to render when zooming out — there should be at least a low
    /// resolution zoom of the newly panned or zoomed out area instead of just
    /// remaining blank while the higher definition render occurs."*
    ///
    /// Measured before it was built: zooming out from 3590 % held
    /// `canvas-coverage covered=0.000` for about twenty frames. The held
    /// texture is a picture of a small region of a big sheet, and once the view
    /// moves off that region there is nothing behind it.
    ///
    /// # ★★ Why this costs no extra render, which is the whole design
    ///
    /// It is not a new rasterisation. It is **the whole-page texture the shell
    /// already made**, kept instead of dropped when a sharper one replaces it.
    /// A document opens at a fit zoom, so the first raster of every page is
    /// whole-page and small; that one becomes the backdrop and stays until the
    /// page or the edit changes.
    ///
    /// The pixel budget is what makes that safe: a whole-page raster is only
    /// promoted while it is under [`BASE_MAX_PIXELS`], so the enormous
    /// whole-page rasters near the region tier are never retained. In the
    /// common case the backdrop and the live texture are the **same handle**
    /// and cost nothing at all, and they diverge only once the operator has
    /// zoomed past it.
    ///
    /// # ★ Why it is dropped on an edit rather than shown stale
    ///
    /// A backdrop is a stand-in and staleness in *sharpness* is exactly what it
    /// is for. Staleness in *content* is a different thing: showing an object
    /// the operator has just deleted, in the part of the page a sharp raster
    /// does not cover, is a picture that contradicts the document. Rule 4's
    /// line is that pdfcer may be fuzzy and may not be sneaky, and an out-of-date
    /// backdrop is the second.
    ///
    /// So it is keyed to the edit epoch and dropped when that moves. The cost
    /// is that an edit made while zoomed deep leaves no backdrop until the
    /// operator zooms out far enough to make a small whole-page raster again —
    /// which is a narrower gap than the one this closes, and an honest one.
    pub base_texture: Option<crate::render::raster::PageTexture>,
    /// The edit epoch [`Self::base_texture`] was rasterised under.
    pub base_texture_epoch: u64,
    /// ★★ **Which pages have been SEEN compositing in ink**, by page index.
    ///
    /// The observation `crate::render::strategy::Ink` rests on, and that type's
    /// docs carry the argument for why it is observed rather than assumed —
    /// including the measurement that made assuming it unacceptable: on the
    /// operator's own D-size sheet the ink ceiling falls at 263 % zoom, and
    /// that sheet has no transparency on it at all.
    ///
    /// Written in exactly one place, [`Self::absorb_render`], from the
    /// renderer's own `cmyk_buffer_engaged` / `cmyk_buffer_refused` counters.
    /// **Either** being non-zero is the signal: engaged means the buffer was
    /// used, refused means it was wanted and too large, and both mean *this
    /// page asked to be blended in ink*.
    ///
    /// # It only ever grows, and that is correct
    ///
    /// A page's blending space is a property of its `/Group` dictionary, not of
    /// the zoom, so an observation cannot become false by scrolling or
    /// magnifying. An **edit** could in principle change a page group, which is
    /// why this is a fact about the open document rather than a persisted one —
    /// it is rebuilt from scratch on the next open, and a stale entry costs at
    /// worst one region raster where a whole-page one would have done.
    ///
    /// A `HashSet` rather than a per-page flag on a struct: it is empty for
    /// virtually every document, since about 0.4 % of real files declare a
    /// subtractive page group.
    pub ink_pages: std::collections::HashSet<usize>,
    /// ★★★ **The [`Self::edit_epoch`] the file on disk currently holds** — i.e.
    /// the revision a successful *Save* last wrote over the operator's own file.
    ///
    /// `edit_epoch == saved_epoch` means *everything the operator has done is on
    /// disk*. It is deliberately NOT the same question as `edit_epoch == 0`,
    /// which asks *has anything ever been edited* and is what the OCR preflight
    /// used to ask.
    ///
    /// # The defect this exists for
    ///
    /// `dialogs::ocr` refused recognition whenever `edit_epoch != 0`, with the
    /// reason **UnsavedEdits**. The refusal protects something real —
    /// `add_ocr_layer` reads the session's *base* revision, so a recognised copy
    /// taken over unsaved edits would silently omit them, and a plausible,
    /// working, wrong file is the worst failure shape this subsystem has.
    ///
    /// But `edit_epoch` never comes back down. `app::save`'s own header argues
    /// at length that a **save-a-copy** must not reset it, and is right: those
    /// edits are saved *somewhere else*, and the document in front of the
    /// operator is exactly as unsaved as it was. That argument was then applied
    /// to save-**in-place**, where it does not hold: an in-place save writes the
    /// operator's own file, and afterwards the document in front of them **is**
    /// what is on disk.
    ///
    /// So OCR died for the rest of the session the first time anyone edited and
    /// saved anything, and told them *"unsaved edits"* on the way out — which by
    /// then was false.
    ///
    /// # Why a second number rather than resetting `edit_epoch`
    ///
    /// Because `edit_epoch` means *"which revision is on the screen"* and is the
    /// cache key for the decomposition, the page-text extraction, the texture
    /// and every live disclosure. Resetting it would throw all of those away to
    /// record that nothing changed, which is the argument `app::save` §3.1 makes
    /// and it is still correct. Two numbers, one question each.
    ///
    /// ★ Moved by exactly one thing: a successful `save::save_in_place`.
    /// Save-a-copy does not touch it, for the reason above — the copy is not
    /// this document.
    pub saved_epoch: u64,
    pub selected_field: Option<SelectedField>,
    /// ★ **The operator's guide lines**, per page, in canvas space.
    ///
    /// View state, not document content — a guide changes nothing a save would
    /// write — so it sits here beside [`Self::selection`] rather than anywhere
    /// near `EditSession`. It is nevertheless **remembered per document**, in
    /// `guides.txt`, because a guide is *work the operator did* rather than a
    /// switch they flicked; [`crate::canvas::guides`]' header §2 carries the
    /// argument, including why that is a fourth store beside `layout.ron`,
    /// `recent.txt` and `page-display.txt`. Changed through
    /// [`crate::app::actions::Action::SetGuides`], which is what makes the
    /// file write happen once per gesture rather than once per frame of a drag.
    pub guides: crate::canvas::guides::Guides,
    /// The current page's decomposition. Read through
    /// [`Self::page_objects`]; see [`crate::app::cache`] for why it is a cache,
    /// why it is behind a `RefCell`, and why the accessors live there while the
    /// field lives here.
    pub(super) page_objects: PageObjectCache,
    /// The current page's font inventory. Read through
    /// [`Self::font_inventory`]; see [`crate::app::cache`].
    pub(super) fonts: FontCache,
    /// The current page's extracted text. Read through [`Self::page_text`];
    /// see [`crate::app::cache`] for the cost measurement that made it a cache
    /// and for why it is per-page where a search is per-document.
    pub(super) page_text: PageTextCache,
    /// Which runs on the current page are inside a form XObject, and therefore
    /// cannot be edited by this cut of `pdfcer-core`. See `FormRunCache`.
    pub(super) form_runs: crate::app::cache::FormRunCache,
    /// **The current page's clickable links**, and the document-wide reader
    /// that resolves where they go. Read through [`Self::page_links`]; see
    /// [`crate::app::cache::LinkCache`] for why it is two caches with two keys
    /// and why the reader going stale would be silent.
    pub(super) links: crate::app::cache::LinkCache,
    /// **What text the operator has selected on the canvas**, if any.
    ///
    /// Beside [`Self::selection`] rather than inside it, and the separation is
    /// the design: that one names *page content objects* and is gated on
    /// `Capabilities::edit_content`, this one names *a range of characters* and
    /// is offered precisely where the other is not. `crate::canvas::textsel`'s
    /// header §3 carries the whole argument, including why this needed no new
    /// capability and why the two can never both be non-empty.
    ///
    /// Document-scoped for the same reason the object selection is: a range of
    /// characters on page 3 of this file means nothing in the next one, and a
    /// fresh [`OpenDoc`] per document is what makes that true by construction
    /// rather than by a `forget_document` somebody has to call.
    pub text_selection: Option<crate::canvas::textsel::TextSelection>,
    /// Whether annotation appearances (`/AP` `/N`, §12.5) are painted over
    /// the page content.
    ///
    /// `true` at open, because that is what a reader does with a file it was
    /// handed and what [`pdfcer_render::RenderOptions`] defaults to. Read
    /// through [`Self::annotations_visible`], changed through
    /// [`Self::set_annotations_visible`].
    ///
    /// **This is view state, not document state.** It changes what is drawn
    /// and nothing that is saved, which is why it lives beside `view` and
    /// dies with the document rather than bumping [`Self::edit_epoch`].
    annotations: bool,
    /// The operator's optional-content override. See [`LayerOverride`].
    ///
    /// ★ `pub(super)` rather than private since 2026-09-01: the four methods
    /// that read and write it live in [`crate::app::layers`], split out of this
    /// file under R2 when it reached the 1,500-line ceiling. Still module-scoped
    /// — nothing outside `app` may reach past `hidden_layers` and its three
    /// companions, because the three-state `Option` they enforce is exactly what
    /// a direct write would get wrong.
    pub(super) layers: LayerOverride,
}

impl OpenDoc {
    /// Build the state for a freshly opened document.
    ///
    /// Everything starts fresh, deliberately: opening a document
    /// constructs a whole new `OpenDoc`, so a cached texture or a page
    /// index can never refer to a page from a previous file. A `reset()`
    /// method would be a second, weaker way to achieve the same thing and
    /// an invitation to reuse an `OpenDoc` across documents — which is
    /// exactly the stale-state bug that constructing fresh state prevents
    /// by design.
    ///
    /// `pub(crate)` rather than private so a panel's own test can build the
    /// document state its body reads, through the **same** constructor
    /// [`PdfcerApp::open_path`] uses. A test-only alternative constructor
    /// would be a second way to assemble an `OpenDoc`, which is precisely
    /// what this function's own argument says not to have.
    pub(crate) fn new(path: PathBuf, session: EditSession, pages: Vec<Page>) -> Self {
        Self::assemble(path, Origin::Opened, session, pages)
    }

    /// Build the state for a document `file.new` has just **created**.
    ///
    /// `name` is what the document is called, not where it is —
    /// `crate::text::files::untitled`. See [`Origin::Created`].
    ///
    /// A sibling of [`Self::new`] rather than a flag on it, because the two
    /// read differently at the call site and one of them is rare: `open_path`
    /// and `new_document` each say which they mean, and neither passes a
    /// boolean whose meaning a reader has to look up.
    pub(crate) fn created(name: PathBuf, session: EditSession, pages: Vec<Page>) -> Self {
        Self::assemble(name, Origin::Created, session, pages)
    }

    /// The one place an `OpenDoc` is assembled, for both origins.
    ///
    /// [`Self::new`]'s own argument — *"a `reset()` method would be a second,
    /// weaker way to achieve the same thing"* — applies with equal force to a
    /// second struct literal. Two constructors that each listed thirty fields
    /// would drift the moment one of them gained a field, and the drift would
    /// be invisible: the compiler is satisfied by both.
    fn assemble(path: PathBuf, origin: Origin, session: EditSession, pages: Vec<Page>) -> Self {
        // ★ The one field read from disk here rather than started empty, and
        // the one `ViewState` default it overrides. `canvas::guides::opening`
        // owns both halves and the rule joining them.
        //
        // ★ …and it is read only for a document that **has** a file. A created
        // document's `path` is a name, so `guides::recall` would absolutize it
        // against the working directory and look up a location nothing is at —
        // usually finding nothing, and finding somebody else's guides on the
        // day an operator really does have `Untitled 1.pdf` in the folder they
        // launched pdfcer from. Both outcomes are wrong for the same reason:
        // per-document state belongs to a document that can be identified, and
        // a name is not an identity.
        let (guides, view) = match origin {
            Origin::Opened => crate::canvas::guides::opening(&path),
            Origin::Created => (
                crate::canvas::guides::Guides::default(),
                ViewState::default(),
            ),
        };
        Self {
            // The shipped defaults, replaced immediately by
            // `PdfcerApp::adopt_settings` on every path that can reach the
            // operator's own. `assemble` cannot see `PdfcerApp`, and giving it
            // an argument for this would mean every test constructing an
            // `OpenDoc` had to state a configuration it does not care about.
            settings: pdfcer_core::settings::Settings::default(),
            prefs: crate::app::prefs::Prefs::default(),
            // ★ Default rather than sized to `pages.len()` here: an empty set
            // answers the same floor for every index (see `PageEpochs::get`),
            // so a document is uniform before `resync` first sizes it — which
            // is the correct starting state and needs no argument at the two
            // dozen call sites that build an `OpenDoc`.
            page_epochs: crate::app::state::pageepoch::PageEpochs::default(),
            path,
            origin,
            session: Arc::new(session),
            pages,
            observed_zoom: view.zoom,
            view,
            page_texture: None,
            page_texture_epoch: 0,
            held_preview: None,
            last_edit_at: None,
            // Empty by construction, like everything else here. A strip cache
            // carried across an open would hold textures of another file's
            // pages under this file's indices.
            strip_rasters: crate::render::strip::StripRasters::default(),
            strip_visible: Vec::new(),
            // Equal to `view.page_index`, so a freshly opened document is not
            // mistaken for one whose page was navigated to before the first
            // frame — which would scroll a continuous strip on open for no
            // reason the operator asked for.
            tracked_page: 0,
            canvas_frames: 0,
            wheel_travel: 0.0,
            render_in_flight: None,
            render_error: None,
            // Nothing has been marked yet, so there is nothing to disclose.
            last_redaction_unreadable_fonts: 0,
            // Nothing has been redacted into this document, so there is nothing
            // for a save to prove the absence of.
            redaction_absence_claims: Vec::new(),
            render_worker: RenderWorker::default(),
            // In the past, so the first zoom change commits at once rather
            // than waiting out a debounce nobody started.
            zoom_commit_at: Instant::now(),
            zoom_commanded: false,
            zoom_anchor: None,
            fit_placement: None,
            pending_destination: None,
            view_viewport: None,
            // Nothing to reveal on a document nobody has searched yet — and,
            // like every other field here, fresh by construction rather than
            // by a reset somebody has to call.
            find_reveal: None,
            edit_epoch: 0,
            content_generation: std::cell::Cell::new(None),
            objects_traced_for: None,
            last_scroll_offset: egui::Vec2::ZERO,
            // Whole page until the canvas says otherwise.
            raster_region: None,
            deep_anchor: None,
            deep_zoom: None,
            // Empty, like everything else here — and that is the entire
            // mechanism by which a selection can never refer to a previous
            // file. See the field's own docs.
            selection: SelectionState::default(),
            selected_field: None,
            base_texture: None,
            base_texture_epoch: 0,
            ink_pages: std::collections::HashSet::new(),
            saved_epoch: 0,
            // Read above, before `path` was moved into the struct.
            guides,
            page_objects: PageObjectCache::default(),
            fonts: FontCache::default(),
            page_text: PageTextCache::default(),
            form_runs: crate::app::cache::FormRunCache::default(),
            links: crate::app::cache::LinkCache::default(),
            // Empty, like every other derived field here — see `selection`.
            text_selection: None,
            // What a reader shows. `pdfcer_render::RenderOptions` defaults the
            // same way, and agreeing with it means a document opened here and
            // a page rendered by `pdfcer` start from the same picture.
            annotations: true,
            // `None`, meaning "obey the document's own default configuration"
            // — which is a distinct state from "hide nothing". See
            // `LayerOverride`.
            layers: LayerOverride::default(),
        }
    }

    /// **The file this document's per-document preferences belong to**, or
    /// `None` when it has no file for a preference to belong to.
    ///
    /// The single predicate that separates [`Origin::Opened`] from
    /// [`Origin::Created`] at every site that cares, and it is deliberately
    /// shaped as *"give me the path if there is one"* rather than as
    /// `is_created()`: the three call sites all want the path, so a boolean
    /// would leave each of them reaching for `self.path` afterwards and one of
    /// them eventually forgetting the test.
    ///
    /// Its three readers, and what each would do wrong without it:
    ///
    /// | site | without this |
    /// |---|---|
    /// | `PdfcerApp::open_path` → `RecentFiles::remember` | the Recent menu gains a row for a file that does not exist, whose whole promise is *"this worked before"* |
    /// | `viewer::remembered` (read at open, written by `SetPageDisplay`) | a page-display choice stored against a fabricated path, and inherited by the next document that happens to be called the same thing |
    /// | `canvas::guides` (read in [`Self::assemble`], written by `SetGuides`) | the same, for guide positions |
    ///
    /// It is **not** consulted by the forms cache key, the Pages panel caption
    /// or the trace, and that is correct rather than an omission: those want an
    /// identity or a label, and a name is both.
    #[must_use]
    pub fn stored_under(&self) -> Option<&std::path::Path> {
        match self.origin {
            Origin::Opened => Some(&self.path),
            Origin::Created => None,
        }
    }

    /// The page currently being viewed, if the index is in range.
    #[must_use]
    pub fn current_page(&self) -> Option<&Page> {
        self.pages.get(self.view.page_index)
    }

    /// The current page's on-screen extent in PDF user-space units, with
    /// `/Rotate` applied.
    ///
    /// Falls back to a US Letter shape for a document with no pages, so the
    /// fit arithmetic has something finite to divide by. Nothing is drawn
    /// in that state — the canvas shows [`crate::text::canvas_no_pages`] —
    /// so the value is never seen; it exists so the arithmetic upstream of
    /// the check does not have to special-case an empty document as well.
    #[must_use]
    pub fn current_extent(&self) -> (f32, f32) {
        self.current_page()
            .map_or((612.0, 792.0), viewer::page_extent_pts)
    }

    /// Whether annotation appearances are painted over the page content.
    #[must_use]
    pub fn annotations_visible(&self) -> bool {
        self.annotations
    }

    /// Show or hide annotation appearances (§12.5).
    ///
    /// A staleness key, so changing it makes the cached texture stale and the
    /// page re-rasterizes on the next frame — see [`RenderKey`]. That is the
    /// whole difference between this being a control and being a bool nobody
    /// can see.
    ///
    /// **Deliberately does NOT bump [`Self::edit_epoch`]**: nothing about the
    /// document has changed, only what is drawn of it. Bumping would throw
    /// away the decomposition and the font inventory to no purpose, and would
    /// make an `objects n=` line re-trace as though an edit had happened.
    pub fn set_annotations_visible(&mut self, visible: bool) {
        self.annotations = visible;
    }

    /// What a render of the current view would be *of*.
    ///
    /// The staleness key the shell wants, built from the same constructor the
    /// worker labels its output with — see
    /// [`crate::render::worker::RenderKey::new`]. One arithmetic path, so
    /// "what I want" and "what I have" cannot disagree about how a key is
    /// spelled.
    ///
    /// `pub(crate)` for one reason: a panel whose control is blocked on
    /// something else needs to be able to assert that *its* input reaches the
    /// key. `crate::panels::layers` does exactly that — see
    /// `the_render_key_no_longer_blocks_a_layer_toggle` — which is how the
    /// next person to restore that checkbox learns which of its three
    /// preconditions is still open without re-deriving the answer.
    pub(crate) fn render_key(&self, raster_scale: f32) -> RenderKey {
        self.render_key_for(self.view.page_index, raster_scale)
    }

    /// What a render of **any** page in this view's current settings would be
    /// *of*.
    ///
    /// [`Self::render_key`]'s general form, and the one a continuous strip
    /// needs: every visible page is rendered with the same scale, annotation
    /// stance and layer override, so the only thing that varies between them
    /// is the page index. Written as one function with the current page as a
    /// special case, rather than two, because two would be two places for the
    /// annotation stance to be forgotten — and a key that omitted it would
    /// leave the strip's pages showing annotations after the operator turned
    /// them off, while the current page obeyed.
    pub(crate) fn render_key_for(&self, page_index: usize, raster_scale: f32) -> RenderKey {
        RenderKey::new(
            page_index,
            raster_scale,
            self.annotations,
            self.layers.generation,
            // ★★★ O137. Through `ViewState::stroke_display`, which is also what
            // `render_request_for` hands the worker — one conversion, so "what
            // I want" and "what I have" cannot disagree about whether line
            // weights were on. Omit it and the toggle looks inert: the cache
            // reports a hit and serves the picture drawn under the other
            // answer.
            self.view.stroke_display(),
        )
        .with_region(self.region_for(page_index))
    }

    /// The region to rasterize for `page_index`, if the canvas set one **for
    /// that page**.
    ///
    /// ★ The page check is the whole of this method's job. Without it a
    /// region computed for page 4 would be applied to page 5 as well, and
    /// both rectangles are valid — so the wrong part of the neighbour would
    /// be rasterized with nothing reporting an error.
    #[must_use]
    pub(crate) fn region_for(&self, page_index: usize) -> Option<pdfcer_core::page_tree::Rect> {
        self.raster_region
            .filter(|(page, _)| *page == page_index)
            .map(|(_, rect)| rect)
    }

    /// Everything a worker needs to rasterize `page_index`, or `None` if there
    /// is no such page.
    ///
    /// The one constructor for a [`RenderRequest`], so the current page and a
    /// strip page cannot be rendered with different options. It exists here,
    /// on the document, rather than in [`crate::render::settle`] because the
    /// annotation stance and the layer override are **private** fields of this
    /// type — and they should stay private: they are changed through
    /// [`Self::set_annotations_visible`] and [`Self::set_hidden_layers`],
    /// which are the methods that keep the staleness keys moving.
    pub(crate) fn render_request_for(
        &self,
        page_index: usize,
        raster_scale: f32,
    ) -> Option<RenderRequest> {
        let page = self.pages.get(page_index)?;
        Some(RenderRequest {
            // ★ O24's region tier, live since 2026-08-22. `None` below the
            // pixmap ceiling — which is every zoom that can render whole-page,
            // so panning there is unchanged — and `Some` above it, where the
            // alternative is the operator's `MAX_PIXMAP_EDGE` failure.
            region: self.region_for(page_index),
            // The `Arc` is handed over rather than a `DocumentView`, which is
            // what lets the borrow stay local to the worker thread.
            session: Arc::clone(&self.session),
            page: page.clone(),
            page_index,
            raster_scale,
            annotations: self.annotations,
            // ★★★ O137 — `view.line_weights`, canvas only. The same conversion
            // the key above uses; see `ViewState::stroke_display` for why it is
            // a function and not two `if`s. `render_on_worker` is the only
            // place this is read, and no export or print path builds a
            // `RenderRequest` at all.
            stroke_display: self.view.stroke_display(),
            layers: self.layer_visibility(),
            layers_generation: self.layers.generation,
            // ★ The SNAPSHOT, not a live read — see the field's own docs.
            //
            // The worker runs on another thread and may finish after the
            // operator has changed a setting, so what it must be given is the
            // configuration this document's caches are keyed to. Handing it a
            // live value would produce a texture drawn under settings that no
            // cached neighbour shares, and nothing would notice: the render key
            // does not carry the settings, because `adopt_settings` drops every
            // cache instead, which is the more direct mechanism and the visible
            // one.
            //
            // Cloned rather than shared: one `String` and twelve `Copy` fields,
            // paid once per render request, against a rasterization measured in
            // tens of milliseconds.
            settings: self.settings.clone(),
        })
    }

    /// **Where every page this view is showing sits**, in one coordinate
    /// space.
    ///
    /// Built from the page vector and the view state, so it cannot disagree
    /// with either. The convenience over calling
    /// [`crate::viewer::strip::Strip::new`] at each site is not brevity: it is
    /// that the three arguments after `pages` are all view state, and a call
    /// site that passed its own idea of the display mode or the zoom would be
    /// laying out a strip the rest of the frame does not agree with.
    #[must_use]
    pub fn strip(&self) -> crate::viewer::strip::Strip {
        crate::viewer::strip::Strip::new(
            &self.pages,
            self.view.display,
            self.view.page_index,
            self.view.zoom,
        )
    }

    /// Report how many objects the current page holds, on the `PDFCER_DIAG`
    /// channel — on document open, on a page change, and after any edit.
    ///
    /// # Why a count and not a deletion event
    ///
    /// `PROJECT_PLAN.md` §4.3 requirement 3, stated in one sentence:
    ///
    /// > Strictly better evidence than a `delete-objects` event: it measures
    /// > the property the check is about rather than the verb that was meant
    /// > to change it.
    ///
    /// The regression test for D1 asks "did pressing Delete remove the
    /// object?". Against a binary with only a deletion event, the available
    /// evidence is the **absence** of that event, and absence is weak: it
    /// cannot distinguish "the deletion path never ran" from "the deletion
    /// path ran and did nothing" from "the event is traced somewhere the code
    /// did not reach". `ui-verify`'s `delete_key` module admits that evidence
    /// exactly once, under three stated conditions, and says that removing
    /// any one of them turns the check into a SKIP.
    ///
    /// A count needs none of that. Read it before, read it after, compare.
    /// It is also robust to a deletion implemented by some future verb that
    /// nobody remembered to add a trace to — the count measures the page, not
    /// the code path.
    ///
    /// # The line
    ///
    /// ```text
    /// pdfcer-diag objects n=412 page=0 paths=380 text=30 images=2 forms=0
    /// ```
    ///
    /// `n=` is the total — the field name `ui-verify`'s vocabulary already
    /// uses for a count — and it is `PageObjects::objects.len()`, the same
    /// index space `EditSession::delete_objects` takes, so "n dropped by one"
    /// and "one object was deleted" are the same statement rather than two
    /// that have to be reconciled. The per-kind breakdown comes free from the
    /// decomposition's own diagnostics and is worth having: a count that
    /// changed by one is more convincing when you can see *which kind* left.
    ///
    /// # Failure is a different event, deliberately
    ///
    /// A page whose content streams will not decode traces
    /// `objects-unavailable page=… reason=…`, **never** `objects` with a
    /// missing or zero `n`. The distinction matters to a consumer: `objects`
    /// is a claim that the count was measured, and a check comparing before
    /// and after must be able to trust that. An `objects` line with no `n`
    /// would read as "the binary stopped reporting", and one with `n=0` would
    /// read as "the page is empty" — a false statement about the document.
    ///
    /// # Cost, and the gate that makes it affordable
    ///
    /// `decompose_page` resolves every `/Contents` stream, inflates it,
    /// concatenates, tokenizes and walks the whole token stream, resolving
    /// fonts as it goes. There is no cache inside `pdfcer-core`; the old GUI
    /// keeps exactly one decomposition per page for precisely this reason.
    /// So this is gated twice: it does nothing at all unless tracing is on,
    /// and even then it runs only when `(page, edit_epoch)` has moved. On a
    /// CAD sheet under `PDFCER_DIAG`, that is one decomposition per page
    /// visited, not one per frame.
    ///
    /// The gate is a stored key rather than [`crate::diag::trace_changed`]
    /// on purpose: `trace_changed` de-duplicates the *rendered line*, which
    /// would still require computing the count in order to render it. The
    /// expensive part is the count itself, so the gate has to sit in front of
    /// it.
    ///
    /// # ★ It counts the SHARED decomposition, not one of its own
    ///
    /// Until S4 this ran its own `decompose_page`, because the only cache was
    /// on the panels and this is not a panel — a second decomposition of the
    /// same page, i.e. the *"two decompositions quietly diverge"* pattern
    /// decision 011 warns about, sitting in the code whose entire job is to
    /// report a trustworthy number about that page. It now reads
    /// [`Self::page_objects`], so `n=` is by construction the count of the
    /// objects the Objects panel lists and the canvas hit-tests. The cost
    /// gate is unchanged: nothing is built with tracing off, and with it on
    /// the page decomposes once per `(page, epoch)` — what the private
    /// decomposition already cost, minus the duplicate.
    pub(crate) fn trace_object_count(&mut self) {
        if !crate::diag::enabled() {
            return;
        }
        let key = (self.view.page_index, self.edit_epoch);
        if self.objects_traced_for == Some(key) {
            return;
        }
        // Recorded BEFORE the work, so a decomposition that fails is not
        // retried on every subsequent frame — the failure is deterministic
        // (same bytes, same code), exactly as `settle_and_rasterize` argues
        // for the render error it holds.
        self.objects_traced_for = Some(key);

        if self.current_page().is_none() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "objects-unavailable page={} pages={} reason=no-such-page",
                    self.view.page_index,
                    self.pages.len()
                )
            });
            return;
        }

        // The shared decomposition — see this method's docs. It is built from
        // `session.view()` and not `document.view()`: the edit session's view
        // is the *edited* state (base plus staged changes), which is the
        // state the operator is looking at and the state a delete is meant to
        // have changed. Counting the base revision would report a number that
        // never moves however many objects are removed.
        if let Some(provider) = self.page_objects() {
            let model = provider.page_objects();
            let d = &model.diagnostics;
            // Read out before the closure so the `Ref` is not held across it.
            let (n, paths, text, images, forms) =
                (model.objects.len(), d.paths, d.text, d.images, d.forms);
            // ★★ **`leaves=` — how many objects are painted from INSIDE those
            // forms**, added 2026-08-27 with form-XObject descent.
            //
            // `n=` counts `PageObjects::objects`, which is the page's own
            // content stream and nothing else. On a wrapped drawing that is a
            // small fraction of what is on screen and of what a click can now
            // select: the industry print-conformance suite's composite page 1
            // reports `n=28 forms=4` and carries **242** leaves;
            // `ncored-benchmark-cad-drawing` page 1 reports `n=129758 forms=1`
            // and carries **10,256**.
            //
            // Without this field, `objects n=` is a half-truth on exactly the
            // documents the operator complained about — and it is the line a
            // driven check reads to answer *"did the page decompose, and how
            // much is there?"*. A harness reading `n=28` on a page with 270
            // selectable things is reading a number that no longer means what
            // its name suggests.
            //
            // ★ `depth_overflow=` and `cycles=` come with it, and they are the
            // half that stops `leaves=` becoming its own half-truth: a non-zero
            // count means the walk did NOT reach everything, so `leaves` is a
            // floor rather than a total. The engine counts them rather than
            // truncating silently for that reason, and a consumer that ignored
            // them would present an incomplete list as complete.
            let (leaves, depth_overflow, cycles) =
                (model.leaves.len(), d.form_depth_overflows, d.form_cycles);
            let page_index = self.view.page_index;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "objects n={n} page={page_index} paths={paths} text={text} images={images} forms={forms} leaves={leaves} depth_overflow={depth_overflow} cycles={cycles}"
                )
            });
            return;
        }

        // A distinct event, deliberately: `objects` is a claim that the count
        // was measured, and a check comparing before against after must be
        // able to trust that. An `objects` line with no `n` would read as
        // "the binary stopped reporting"; one with `n=0` would read as "the
        // page is empty", which is a false statement about the document.
        let detail = self
            .page_objects_failure()
            .map_or_else(|| "unknown".to_owned(), |err| err.clone());
        let page_index = self.view.page_index;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("objects-unavailable page={page_index} reason=decompose-failed detail={detail}")
        });
    }
}

/// How a test opens a document, and why there are **two** fixture roots.
/// `#[cfg(test)]` only; split out under R2 when this file hit its ceiling.
#[cfg(test)]
mod fixtures;
#[cfg(test)]
pub(crate) use fixtures::{
    // ★ The fixture NAMES are re-exported from here rather than moved in the
    // callers' `use` lines, and that is deliberate: forty test modules write
    // `use crate::app::state::{FOUR_PAGES, open_fixture}`, and an R2 split is
    // a change to where code LIVES, not to what the crate offers. A split that
    // rewrote forty imports would be a diff nobody could read.
    FOUR_PAGES,
    PAINTED_LAYERS,
    ROTATED_TEXT,
    SIGNED_TWO_PAGES,
    open_fixture,
    open_local_fixture,
};

/// ★★★ **The preview that outlives the gesture** — `OPERATOR_REQUESTS.md` O63.
///
/// Split out under R2 on 2026-08-30. Its header carries the subject: for how
/// long is a picture of this document still true? Three clauses, two of them
/// bounded by wall-clock time, and the reason each of the two is.
mod heldpreview;
pub(crate) use heldpreview::HeldPreview;

#[cfg(test)]
mod tests;
