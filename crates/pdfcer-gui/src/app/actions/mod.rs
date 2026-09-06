//! ★ **This module's documentation lives in `OVERVIEW.md`** beside this file,
//! pulled in below with `include_str!`.
//!
//! It was moved there on 2026-08-19 when `Action` gained `MoveHandle` and the
//! file crossed R2's 1,500-line ceiling. The gate's own message says *"split
//! the module along its seams — one subject per file — rather than raising the
//! limit"*, and this file has exactly one seam and it is not where the lines
//! are: the whole body is **one enum**, which cannot be split without inventing
//! a nested variant and rewriting every match arm in the crate. The bulk is
//! prose, and prose has a file format.
//!
//! Nothing is lost and nothing is hidden. `include_str!` puts the text back
//! into the rendered docs verbatim, the file sits in the same directory, and
//! `cargo doc` and a reader browsing the source both see the same thing they
//! did before. R5 asks that the documentation be *complete and adjacent*; it
//! does not ask that it be in a `.rs` file.
#![doc = include_str!("OVERVIEW.md")]

pub mod destination;
/// ★ **Reordering a page's annotations** — O99. Split out of [`forms`] on
/// 2026-09-02 under R2; its header carries why the disclosures are the
/// interesting part rather than the call.
pub(super) mod reorder;
/// ★ **The three saves**, split out of [`apply`] on 2026-09-02 under R2. All
/// three ask a signature question before the document guard and hand off to
/// `lifecycle`; its header carries why Save As asks the COPY question rather
/// than the in-place one, which reads like a mistake and is not.
mod saving;
/// The sentences one edit owed, and the epoch rule that keeps them honest.
/// Split out of this file under R2 when annotation selection needed the room;
/// its own header carries the seam.
/// The verbs that change an annotation — delete today, the Format tab's
/// restyles next. Split out of `apply` under R2; its header carries the seam
/// and the ce-dimension routing obligation every future verb here inherits.
/// Turning a bookmark's destination into the moves that arrive at it.
/// The verbs that move the operator rather than the document.
mod view;

/// ★★★ **Placing NEW page text, and the width question** —
/// `OPERATOR_REQUESTS.md` **O127**, defect 2.
///
/// Split out of [`apply`] under R2 on 2026-09-04, along [`funnel`]'s seam: that
/// file routes, and this one **decides**. A PDF has no paragraph, so a
/// multi-line add needs a width to wrap against — and once Enter makes a line
/// break at a *clicked* caret, which has no extent, somebody has to answer
/// where the second line ends. Its header carries the answer and, more
/// importantly, why the answer is read off the operator's own sheet rather than
/// invented.
mod addtext;
mod annots;
/// What applying an [`Action`] does — the interpreter half of this module.
///
/// Split out under **R2**; see its own header for the seam. Not `pub`: nothing
/// outside `app` applies an action, and the two entry points it adds are
/// inherent methods on [`crate::app::PdfcerApp`] rather than free functions, so
/// they are reachable exactly where they were before the split.
mod apply;
/// ★★ The three verbs whose subject is a whole **file living inside the
/// document** — attach, remove, and save one out (ISO 32000-1 §7.11.4.1).
///
/// Its header carries the property that makes them a family rather than a
/// subject label: **every one of them opens a native file dialog**, so all
/// three are `Action`s for the reason [`write`]'s three are *as well as* for
/// the funnel's own — and **nothing any of them does is visible on the
/// canvas**, so every one of them owes a sentence to `app::status`, which is
/// the exact inverse of [`bookmarks`]' deliberately-silent rename.
///
/// `pub` rather than private, unlike [`apply`], because the surface that raises
/// these verbs is outside `app`: `panels::attachments` names
/// [`attachments::AttachmentAction`] and [`attachments::AttachmentRef`] to
/// build one.
pub mod attachments;
/// ★ The three verbs whose subject is one entry in the document's outline —
/// add, rename, and delete-with-its-subtree.
///
/// Split out of [`action`] under R2 on 2026-08-28, the day `pdfcer-core`
/// `Pass 156.0` turned a one-verb family into a three-verb one. Its header
/// carries the property that makes them a family rather than a size-driven
/// cut — **every one of them addresses its operand by `ObjId`, never by a
/// position in the tree**, because an outline is renumbered by every edit to
/// it — and the §12.3.3 `/Count` table the engine sent this shell unprompted.
///
/// `pub` rather than private, unlike [`apply`], because the surface that raises
/// these verbs is outside `app`: `panels::bookmarks::add` and
/// `panels::bookmarks::edit` both name [`bookmarks::BookmarkAction`] to build
/// one.
pub mod bookmarks;
/// ★ `ViewChrome` — which piece of View ▸ Display an action is about.
///
/// Split out on 2026-08-19: it is not an action, it is the one *operand* in
/// this vocabulary with a type of its own. Re-exported below, so no call site
/// learns that it moved.
mod chrome;
/// Extracting pages into a new file — the one page verb that writes a file
/// rather than changing the open document. Split out of `pages` under R2 on
/// 2026-08-28; its header carries the seam.
mod extract;
/// ★ Combine several PDFs into a new file — `OPERATOR_REQUESTS.md` O68.
///
/// Beside [`extract`] rather than in `pages`, because the two share the
/// property that decides where they live: both produce **new file bytes** and
/// touch neither the session nor the undo log. See its header.
pub(crate) mod merge;
/// Authoring the annotations that carry WORDS — the sticky note, the text box
/// and the stamp. Split out of `apply` under R2 on 2026-08-28; its header
/// carries the seam, which is *composes rather than routes*.
mod textannot;

/// **Pages dragged out of one open document and into another.**
///
/// The only edit in the application that reads two documents at once, which is
/// why it is a file of its own rather than a sixth member of [`pages`]. Its
/// header carries the argument for the drop being a *copy* — it is about undo,
/// not about caution.
mod crossdoc;
/// ★ Everything the **ce-dimension** feature asks the document to do — the
/// groups, their scales, standards and style defaults, and the per-ce-dimension
/// overrides.
///
/// A sibling of [`annots`] and [`pages`], drawn along the same seam they are:
/// *what class of thing does this verb act on?* Its own header carries the one
/// fact a reader needs first — that four of its eight verbs regenerate every
/// member of a group, on every page, and four touch exactly one annotation.
///
/// `pub` rather than private, unlike [`apply`], because the surfaces that raise
/// these verbs are outside `app`: `dialogs::scale`, `dialogs::dimension_groups`,
/// `panels::dimension` and `canvas::measure` all name
/// [`dimensions::DimensionAction`] to build one.
pub mod dimensions;
pub mod disclosure;
/// ★★ The four actions that replace the open document — Open, New, NewSized,
/// Close — and the two guards all four share.
///
/// Split out of [`apply`] on 2026-08-19 along a seam that file had already
/// described in prose. Its header carries the guard table, why the two guards
/// are two predicates and not one, and the defect the second of them closed:
/// until that day all four destroyed every edit made since the file was
/// opened, silently, while `file.close`'s tooltip promised otherwise.
mod document;
/// ★ What leaves the document — DXF today, and the sixth sibling of [`apply`].
///
/// Its header carries the property that makes it a subject rather than a
/// size-driven cut: **no verb in it changes the document at all**, so every
/// rule the mutation funnel enforces is irrelevant to them and every rule about
/// file handling applies instead.
pub mod export;
/// Registering a form control the document draws but no field claims — one
/// verb, whose refusal and disclosure wording is the substantial part.
/// The document-level font verbs - embedding the programs a document names but
/// does not carry. Its header carries the one thing a reader must not move:
/// the shell owns the honesty of the donor match, and the engine will not
/// check it.
mod fonts;
pub mod forms;
/// ★ Stepping the command log, in both directions — `Direction`, its four
/// per-direction answers, and `history_step`.
///
/// Split out of [`apply`] on 2026-08-19: that module answers *what does this
/// verb do to the document*, and undo and redo describe **no edit at all** —
/// they ask the session to replay one it has already recorded.
mod history;
pub use chrome::ViewChrome;
/// The four page verbs' bodies, and the structural resync every edit owes.
///
/// A sibling of [`apply`] rather than part of it, on rule R2's own reasoning:
/// that file's subject is the cancel–mutate–bump–invalidate protocol, and this
/// one's is *a page index is a position, not an identity*. See its header for
/// the table of what each kind of page edit invalidates.
pub mod pages;
/// ★★★ **Changing the paper an open drawing sits on** — `set_media_boxes`, and
/// the pre-commit survey that tells the operator whether he is about to crop
/// his drawing or leave it alone.
///
/// A sibling of [`pages`] rather than part of it, and the seam is a real one:
/// that file's subject is the resync a **structural** edit owes, and a media
/// box change adds, removes and renumbers nothing — every row of its table is
/// "unchanged" for this verb. Its header carries the measured answer to the
/// question every other page-size control in the world gets wrong: the paper
/// changes and the drawing does not move.
pub mod pagesize;

pub use disclosure::{EditDisclosure, last_edit_disclosure};
// ★ Crate-visible rather than `pub`, and re-exported here rather than reached
// through `disclosure::` at every call site: the split was an R2 move and it
// must not change what any caller can see or how they spell it. Widening these
// to `pub` to make one `pub use` compile would have made a private recording
// path part of the crate's surface as a side effect of a file split.
pub(crate) use disclosure::{record_edit_disclosure, record_note, record_notes};

/// The three arms that mark content for removal. Split out of `apply` under
/// rule R2; its header carries the seam argument and names the one thing
/// deliberately absent from it.
/// ★ **The edit funnel** — `vector_edit`, the four-step protocol every verb
/// that changes a document passes through. Split out of `apply` under R2; its
/// header carries why a router and a protocol are two subjects.
mod funnel;
mod redact;
pub mod redactimg;
/// ★ **Redact what is selected on the page** — the third marking route, and the
/// first that does not go through text. Its header carries why the search box
/// could not reach a vector title block, a stamp or a logo.
mod redactsel;
/// The one arm that signs a document — `Action::SignDocument`'s body, split
/// out under R2 on the seam `saving`, `redact` and `destination` already
/// occupy. `#[cfg]` for `crate::sign`'s reason: without the capability there is
/// no verb for it to call.
#[cfg(feature = "signing")]
pub mod sign;

// ---------------------------------------------------------------------------
// The edit disclosure — what [`vector_edit`] carries out to `app::status`
//
// See [`vector_edit`]'s "The disclosures" section for what a disclosure IS.
// This block is the answer to the question that section used to leave open:
// *where does an operator read one?*
// ---------------------------------------------------------------------------

/// **The action vocabulary**, one variant per operator intent.
///
/// Split out of this file on 2026-08-20, when it crossed rule R2's ceiling
/// for the second time. The header above records the previous answer — the
/// module's prose moved to `OVERVIEW.md`, because *"the bulk is prose, and
/// prose has a file format"* — and this is the same reasoning applied one
/// step further along. What is left here is the **module**: its declarations,
/// its re-exports and the disclosure wiring. What moved is the **type**.
///
/// That the enum still cannot be split *internally* remains true and remains
/// stated in the header. `action.rs` is now within a hundred lines of the
/// ceiling itself, so the next family of variants to grow is the one that
/// will have to become a sub-enum beside `PageAction` and `DimensionAction`.
/// Measured on 2026-08-20, the candidate is **markup**: `CommitMarkup` (116
/// lines), `PasteMarkup` (69), `CommitTextMarkup` (57), `BeginTextAnnot` (40),
/// `SetMarkupStyle` (39), `CommitTextAnnot` (29) and `DeleteAnnotation` — some
/// 370 lines, whose call sites are concentrated in `canvas::markup` and
/// `app::actions::annots`. Written down here so the next person does not have
/// to re-measure it under deadline.
/// ★ **Everything that changes page geometry** — delete, the four move verbs,
/// the Bézier handle and the transform. Split out of [`action`] under R2 on
/// 2026-08-20; its header carries the one property every variant shares (they
/// all address paint-order indices into one content stream) and the argument
/// for why there are two verbs that both "move things".
/// **Changing how EXISTING text looks** — size, colour, face, weight, slant.
///
/// `pub` because [`action::Action::TextStyle`] names its `StyleChange` and the
/// Properties panel constructs one.
/// **An instrument, not a feature** — how long the engine takes to accept one
/// edit, measured rather than reasoned. `#[cfg(test)]` and `#[ignore]`d; it
/// exists because `OPERATOR_REQUESTS.md` O63's whole design turns on whether
/// the delay an operator feels is the commit or the raster, and `BENCHMARK.md`
/// exists because the last time this project answered that from architecture it
/// was wrong.
#[cfg(test)]
mod latency;
pub mod textstyle;

pub mod vector;

// ★★★ O122 — the two halves of handing the document to Acrobat: the arm that
// raises the question, and the drain that saves, launches and then closes. Its
// header carries the save→launch→close ordering and why the other order loses
// the operator's document off their screen when a `spawn` fails.
mod acrobat;
mod action;
/// The three verbs that exist only to move a native file picker out of the
/// layout pass — DXF, form data and a compacted copy. Split out of [`action`]
/// under R2 on 2026-08-28; its header carries the property they share and the
/// reason a SAVE is filed with two exports.
/// The verbs whose subject is a whole annotation — move, resize, remove. Its
/// header carries what makes them a family: all three find their operand by
/// stable object id, so none needs a page to locate one.
pub mod annot;
/// The verbs that re-shape a page's own text. Its header carries the reason
/// reflow is not like its neighbours: it is planned against the BASE document
/// and refuses a page this session has already rewritten.
pub mod text;
pub mod write;
/// ★★ The verbs whose subject is a **form XObject** — the shared drawing a CAD
/// producer invokes from every sheet (§8.10.1).
///
/// One verb today, `unshare_form`, wired 2026-08-28 after `EDITABLE_SURFACES.md`
/// found it implemented in the engine and called by nothing here. Its header
/// carries the property that makes this a family rather than a stray: **the
/// operand is a stream object paired with the page that invokes it**, a
/// `(usize, ObjId)` whose halves are not independent, and no other family in
/// this crate addresses anything of that shape.
///
/// It also carries the one fact a reader must not get wrong — the granularity
/// is **one page, not one invocation**, which is the engine's decision — and
/// the reason every one of the verb's seven refusals is worded when its
/// neighbours word one of six: after a refusal here the page looks exactly as
/// it does after a success, so silence reads as *"it worked"* and sends the
/// operator on to edit content they still share.
pub mod xobject;

pub use action::Action;
// ★ The redaction family's sub-enum, re-exported beside `Action` exactly as
// `VectorAction` is, so a call site writes `actions::RedactAction` rather than
// reaching through the module that happens to hold the bodies. It moved out of
// `Action` on 2026-09-06 under R2 — see its own header for why it went before
// markup, which the written plan had nominated.
pub use redact::RedactAction;
pub use vector::VectorAction;

// ---------------------------------------------------------------------------
// ★ EVERYTHING BELOW THIS LINE IS TEST-ONLY, AND THAT IS A GATE REQUIREMENT
//   RATHER THAN A HOUSE STYLE.
//
// `tools/gates/check-ui-strings.sh` truncates each file at its FIRST
// column-0 `#[cfg(test)]` and scans nothing after it — its own header states
// the limit in as many words ("any non-test code placed AFTER the test module
// is invisible to the checker") and records the day a planted violation
// failed to fire because of it.
//
// So a `#[cfg(test)]` item in the MIDDLE of a file silently disarms rule R1
// for the rest of that file. `plant_edit_disclosure_for_test` was written
// beside the store it plants into, next to `record_edit_disclosure` — which
// would have put the attribute at line 244 of 1,253 and left this module's
// entire `Action` enum, its doc comments and every `format!` in
// `PdfcerApp::apply` unscanned. Measured, not assumed: a violation planted
// after such a line passes the gate.
//
// Keeping the test-only helper here, below all real code, costs one level of
// distance from the thing it plants into and buys back a thousand lines of
// coverage.
// ---------------------------------------------------------------------------

/// Plant a disclosure, for tests in other modules that must draw one.
///
/// `#[cfg(test)]` so it cannot become a second way to record one — the real
/// path is [`record_edit_disclosure`], called from [`vector_edit`] with the
/// epoch the edit produced, and a second entry point is how two callers come
/// to disagree about what "the last edit" means.
///
/// It exists because the status bar draws this and must prove it does not grow
/// the bar while doing so (R128), and that measurement has to happen in
/// `crate::app::status`, which cannot reach a `thread_local` here. Exactly the
/// reason `crate::panels::forms::edit::plant_fill_disclosure_for_test` exists,
/// which is the shape this follows.
#[cfg(test)]
pub(crate) fn plant_edit_disclosure_for_test(disclosure: EditDisclosure) {
    record_edit_disclosure(Some(disclosure));
}

/// ★★★ **What an image export IS** — the format, the pages, the resolution and
/// whether transparency survives — decided as a value before anything is
/// written, and with the one combination pdfcer refuses named as an enum rather
/// than as a `bool`. `OPERATOR_REQUESTS.md` O120.
pub mod imageexport;

/// ★★★ **What a TEXT export is** — which pages, what goes between them, and how
/// the bytes are encoded — plus the pure parts of making one, and the recorded
/// finding that the **import** half the operator asked for in the same sentence
/// does not exist in `pdfcer-core` at all.
///
/// Its header carries the three different features "import text" could mean and
/// the reason none of the three is buildable today. Read it before adding an
/// import control.
pub mod exporttext;

#[cfg(test)]
mod tests;
