//! **This module's documentation lives in `OVERVIEW.md`** beside this file,
//! pulled in below with `include_str!`.
//!
//! What is left in the `.rs` file is the declaration list, the re-exports and
//! the disclosure wiring; the vocabulary itself is [`action`]. R2's gate says
//! *"split the module along its seams — one subject per file — rather than
//! raising the limit"*, and the seam that remains here is not one more `mod`:
//! it is prose against code. The prose is the map of the whole `actions` tree,
//! and prose has a file format.
//!
//! Nothing is lost and nothing is hidden. `include_str!` puts the text back
//! into the rendered docs verbatim, the file sits in the same directory, and
//! `cargo doc` and a reader browsing the source both see the same text. R5
//! asks that the documentation be *complete and adjacent*; it does not ask
//! that it be in a `.rs` file.
#![doc = include_str!("OVERVIEW.md")]

/// Turning a bookmark's destination into the moves that arrive at it.
pub mod destination;
/// **The verbs whose subject is a PREFERENCE rather than a document.**
///
/// Its header carries the four properties every member shares, and the
/// first of them is why the family exists at all: a preference needs **no
/// open document**, so its arm is matched above the `Status::Open` guard
/// that every other arm in [`apply`] lives under. `pub` rather than
/// private because the surfaces that raise these verbs are outside `app`
/// — `find::bar` and `panels::pages::previews` both name
/// [`prefs::PrefAction`] to build one.
pub mod prefs;
/// **Reordering a page's annotations** — O99. Its header carries why the
/// disclosures are the interesting part rather than the call.
pub(super) mod reorder;
/// **The three saves.** All three ask a signature question before the document
/// guard and hand off to
/// `lifecycle`; its header carries why Save As asks the COPY question rather
/// than the in-place one, which reads like a mistake and is not.
mod saving;
/// The verbs that move the operator rather than the document.
mod view;

/// **Placing NEW page text, and the width question** —
/// `OPERATOR_REQUESTS.md` **O127**, defect 2.
///
/// Beside [`funnel`], along that file's seam: it routes, and this one
/// **decides**. A PDF has no paragraph, so a multi-line add needs a width to
/// wrap against — and once Enter makes a line
/// break at a *clicked* caret, which has no extent, somebody has to answer
/// where the second line ends. Its header carries the answer and, more
/// importantly, why the answer is read off the operator's own sheet rather than
/// invented.
mod addtext;
/// The verbs that change an annotation — delete today, the Format tab's
/// restyles next. Its header carries the seam and the ce-dimension routing
/// obligation every future verb here inherits.
mod annots;
/// What applying an [`Action`] does — the interpreter half of this module.
///
/// Split out under **R2**; see its own header for the seam. Not `pub`: nothing
/// outside `app` applies an action, and the two entry points it adds are
/// inherent methods on [`crate::app::PdfcerApp`] rather than free functions, so
/// they are reachable exactly where they were before the split.
mod apply;
/// The three verbs whose subject is a whole **file living inside the
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
/// The three verbs whose subject is one entry in the document's outline —
/// add, rename, and delete-with-its-subtree.
///
/// Its header carries the property that makes them a family rather than a
/// size-driven
/// cut — **every one of them addresses its operand by `ObjId`, never by a
/// position in the tree**, because an outline is renumbered by every edit to
/// it — and the §12.3.3 `/Count` table the engine sent this shell unprompted.
///
/// `pub` rather than private, unlike [`apply`], because the surface that raises
/// these verbs is outside `app`: `panels::bookmarks::add` and
/// `panels::bookmarks::edit` both name [`bookmarks::BookmarkAction`] to build
/// one.
pub mod bookmarks;
/// `ViewChrome` — which piece of View ▸ Display an action is about.
///
/// Its own file because it is not an action: it is the one *operand* in this
/// vocabulary with a type of its own. Re-exported below, so no call site has
/// to name this module.
mod chrome;
/// **Placing one of the operator's OWN stamps** — O172's second half.
///
/// Beside [`textannot`] because both end in a `/Stamp` annotation, and NOT
/// inside it because a standard stamp is a *name* and a custom stamp is a
/// *document*: this one opens a second PDF off his disk, imports an object
/// graph, and comes back with four disclosures about what did and did not
/// travel. Its header carries the argument in full.
mod customstamp;
/// Extracting pages into a new file — the one page verb that writes a file
/// rather than changing the open document. Its header carries the seam
/// against [`pages`].
mod extract;
/// Combine several PDFs into a new file — `OPERATOR_REQUESTS.md` O68.
///
/// Beside [`extract`] rather than in `pages`, because the two share the
/// property that decides where they live: both produce **new file bytes** and
/// touch neither the session nor the undo log. See its header.
pub(crate) mod merge;
/// Author an Acrobat **stamp collection** from this document's pages —
/// `OPERATOR_REQUESTS.md` O169.
///
/// Beside [`extract`] and [`merge`] for the property all three share: they
/// produce **new file bytes** and touch neither the session nor the undo log.
/// Its own header carries why it is not an arm in [`export`] — the difference
/// is not size, it is that this one authors a document whose *structure* means
/// something to a second application. What goes IN the file lives in
/// `crate::stamps`, which needs no picker to test.
mod stamps;
/// Authoring the annotations that carry WORDS — the sticky note, the text box
/// and the stamp. Its header carries the seam, which is *composes rather
/// than routes*.
mod textannot;
/// **Committing an edit to text that is already on the page** — the body of
/// `Action::CommitTextEdit`. Its header carries why that body computes rather
/// than routes, and why the two neighbouring commit verbs are not here.
mod textcommit;

/// **Pages dragged out of one open document and into another.**
///
/// The only edit in the application that reads two documents at once, which is
/// why it is a file of its own rather than a sixth member of [`pages`]. Its
/// header carries the argument for the drop being a *copy* — it is about undo,
/// not about caution.
mod crossdoc;
/// Everything the **ce-dimension** feature asks the document to do — the
/// groups, their scales, standards and style defaults, and the per-ce-dimension
/// overrides.
///
/// A sibling of [`annots`] and [`pages`], drawn along the same seam they are:
/// *what class of thing does this verb act on?* Its own header carries the one
/// fact a reader needs first — that some of its verbs regenerate every member
/// of a group, on every page, and the rest touch exactly one annotation.
///
/// `pub` rather than private, unlike [`apply`], because the surfaces that raise
/// these verbs are outside `app`: `dialogs::scale`, `dialogs::dimension_groups`,
/// `panels::dimension` and `canvas::measure` all name
/// [`dimensions::DimensionAction`] to build one.
pub mod dimensions;
/// The sentences one edit owed, and the epoch rule that keeps them honest.
/// Its own header carries the seam.
pub mod disclosure;
/// The four actions that replace the open document — Open, New, NewSized,
/// Close — and the two guards all four share.
///
/// Its header carries the guard table and why the two guards are two
/// predicates rather than one. The second exists because without it all four
/// destroy every edit made since the file was opened, silently, while
/// `file.close`'s tooltip promises otherwise.
mod document;
/// What leaves the document — DXF today, and the sixth sibling of [`apply`].
///
/// Its header carries the property that makes it a subject rather than a
/// size-driven cut: **no verb in it changes the document at all**, so every
/// rule the mutation funnel enforces is irrelevant to them and every rule about
/// file handling applies instead.
pub mod export;
/// The document-level font verbs — embedding the programs a document names but
/// does not carry. Its header carries the one thing a reader must not move:
/// the shell owns the honesty of the donor match, and the engine will not
/// check it.
mod fonts;
/// **Everything done to a form FIELD** — fill, place, author, rename, delete,
/// and registering a control the document draws that no field claims. Its
/// header carries the property that makes the family a family: every verb
/// addresses a control by fully qualified name or by widget `ObjId`, never by
/// a paint-order index.
pub mod forms;
/// Stepping the command log, in both directions — `Direction`, its four
/// per-direction answers, and `history_step`.
///
/// A sibling of [`apply`] rather than an arm in it: that module answers *what
/// does this verb do to the document*, and undo and redo describe **no edit at
/// all** —
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
/// **Changing the paper an open drawing sits on** — `set_media_boxes`, and
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
// Crate-visible rather than `pub`, and re-exported here rather than reached
// through `disclosure::` at every call site: the split was an R2 move and it
// must not change what any caller can see or how they spell it. Widening these
// to `pub` to make one `pub use` compile would have made a private recording
// path part of the crate's surface as a side effect of a file split.
pub(crate) use disclosure::{record_edit_disclosure, record_note, record_notes};

/// **The edit funnel** — `vector_edit`, the four-step protocol every verb that
/// changes a document passes through. Its header carries why a router and a
/// protocol are two subjects.
mod funnel;
/// The three arms that mark content for removal. Its header carries the seam
/// argument and names the one thing deliberately absent from it.
mod redact;
pub mod redactimg;
/// **Redact what is selected on the page** — the third marking route, and the
/// first that does not go through text. Its header carries why the search box
/// could not reach a vector title block, a stamp or a logo.
mod redactsel;
/// **Record a comment's review status** — `/State` and `/StateModel`,
/// §12.5.6.3. Its own file rather than a place in [`annots`], because that
/// module is *"what happens to a thing that already exists"* and this one adds
/// a separate annotation and changes nothing about the comment it names.
///
/// `pub` because [`Action::RecordReviewState`] carries
/// [`reviewstate::RecordStatus`] as its payload — the pattern `annot`,
/// `forms` and `vector` already use, and the one that keeps `action.rs` under
/// R2's ceiling.
pub mod reviewstate;
/// The one arm that signs a document — `Action::SignDocument`'s body, split
/// on the seam `saving`, `redact` and `destination` already occupy.
/// `#[cfg]` for `crate::sign`'s reason: without the capability there is
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

/// **An instrument, not a feature** — how long the engine takes to accept one
/// edit, measured rather than reasoned. `#[cfg(test)]` and `#[ignore]`d; it
/// exists because `OPERATOR_REQUESTS.md` O63's whole design turns on whether
/// the delay an operator feels is the commit or the raster, and `BENCHMARK.md`
/// exists because the last time this project answered that from architecture it
/// was wrong.
#[cfg(test)]
mod latency;
/// **Changing how EXISTING text looks** — size, colour, face, weight, slant.
///
/// `pub` because [`action::Action::TextStyle`] names its `StyleChange` and the
/// Properties panel constructs one.
pub mod textstyle;

/// **Everything that changes page geometry** — delete, the four move verbs,
/// the Bézier handle and the transform. Its header carries the one property
/// every variant shares (they all address paint-order indices into one content
/// stream) and the argument for why there are two verbs that both "move
/// things".
pub mod vector;

// O122 — the two halves of handing the document to Acrobat: the arm that
// raises the question, and the drain that saves, launches and then closes. Its
// header carries the save→launch→close ordering and why the other order loses
// the operator's document off their screen when a `spawn` fails.
mod acrobat;
/// **The action vocabulary**, one variant per operator intent — the type this
/// module's `OVERVIEW.md` describes. It cannot be split internally: it is one
/// enum, and a nested variant would rewrite every match arm in the crate. What
/// grows out of it instead is a sub-enum per family, the shape `PageAction`,
/// `DimensionAction` and `RedactAction` already have.
mod action;
/// The verbs whose subject is a whole annotation — move, resize, remove. Its
/// header carries what makes them a family: all three find their operand by
/// stable object id, so none needs a page to locate one.
pub mod annot;
/// The verbs that re-shape a page's own text. Its header carries the reason
/// reflow is not like its neighbours: it re-emits the page's FIRST content
/// stream and the commit sweep empties the rest, so it refuses a page carrying
/// a non-empty extra stream.
pub mod text;
/// The three verbs that exist only to move a native file picker out of the
/// layout pass — DXF, form data and a compacted copy. Its header carries the
/// property they share and the reason a SAVE is filed with two exports.
pub mod write;
/// The verbs whose subject is a **form XObject** — the shared drawing a CAD
/// producer invokes from every sheet (§8.10.1).
///
/// One verb today, `unshare_form`. Its header
/// carries the property that makes this a family rather than a stray: **the
/// operand is a stream object paired with the page that invokes it**, a
/// `(usize, ObjId)` whose halves are not independent, and no other family in
/// this crate addresses anything of that shape.
///
/// It also carries the one fact a reader must not get wrong — the granularity
/// is **one page, not one invocation**, which is the engine's decision — and
/// the reason every one of the verb's refusals is worded rather than collapsed
/// into a shrug: after a refusal here the page looks exactly as
/// it does after a success, so silence reads as *"it worked"* and sends the
/// operator on to edit content they still share.
pub mod xobject;

pub use action::Action;
// The redaction family's sub-enum, re-exported beside `Action` exactly as
// `VectorAction` is, so a call site writes `actions::RedactAction` rather than
// reaching through the module that happens to hold the bodies. See its own
// header for why this family became a sub-enum before markup did.
pub use redact::RedactAction;
pub use vector::VectorAction;
// The third payload type, and the only one whose bodies live outside
// this module — `Action::DeclineOnCanvas`'s two-armed vocabulary, which is
// declared beside the store it feeds because that is where the argument for
// keeping it short belongs.
//
// It is re-exported here because `crate::app::status::decline` is
// `pub(super)` and `crate::canvas` therefore cannot name that path, while the
// canvas is the only surface that raises this action. That is the same
// relationship `Action` itself has: the store stays shut, the vocabulary for
// asking it for a sentence does not. Callers outside `crate::app` write
// `actions::CanvasDecline`, and nothing outside `crate::app` can reach
// `record_canvas`, which is still `pub(crate)` behind the `pub(super)` path.
pub use crate::app::status::decline::CanvasDecline;

// ---------------------------------------------------------------------------
// EVERYTHING BELOW THIS LINE IS TEST-ONLY, AND THAT IS A GATE REQUIREMENT
//   RATHER THAN A HOUSE STYLE.
//
// `tools/gates/check-ui-strings.sh` truncates each file at its FIRST
// column-0 `#[cfg(test)]` and scans nothing after it — its own header states
// the limit in as many words ("any non-test code placed AFTER the test module
// is invisible to the checker").
//
// So a `#[cfg(test)]` item in the MIDDLE of a file silently disarms rule R1
// for the rest of that file: a violation planted after such a line passes the
// gate, measured rather than assumed. `plant_edit_disclosure_for_test` belongs
// beside the store it plants into, next to `record_edit_disclosure`, and
// putting it there would leave everything below that point unscanned.
//
// Keeping the test-only helper here, below all real code, costs one level of
// distance from the thing it plants into and buys back the rest of the file's
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

/// **What an image export IS** — the format, the pages, the resolution and
/// whether transparency survives — decided as a value before anything is
/// written, and with the one combination pdfcer refuses named as an enum rather
/// than as a `bool`. `OPERATOR_REQUESTS.md` O120.
pub mod imageexport;

/// **What a TEXT export is** — which pages, what goes between them, and how
/// the bytes are encoded — plus the pure parts of making one.
///
/// Its header carries the several different features "import text" could mean,
/// which matters because the operator asked for export and import in one
/// sentence and only one of those meanings is [`importtext`].
pub mod exporttext;
/// **A text file becomes pages** — `Action::ImportText`'s body, on
/// `EditSession::place_text`. Mostly a disclosure: its header lists the
/// judgements `PlaceTextReport` carries about the operator's own file, and why
/// the engine's ready-made sentences are not the ones printed.
pub mod importtext;
/// **The two actions that change what is SELECTED and nothing else.** Its
/// header records why that is a real boundary rather than a size cut — every
/// other `Action` variant asks the document to
/// change, and these two touch only shell state.
pub mod selecting;

#[cfg(test)]
mod tests;
