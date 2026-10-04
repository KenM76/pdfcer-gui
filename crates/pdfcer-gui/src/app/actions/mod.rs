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
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/mod.md`.
#![doc = include_str!("OVERVIEW.md")]

/// Turning a bookmark's destination into the moves that arrive at it.
pub mod destination;
/// **The verbs whose subject is a PREFERENCE rather than a document.**
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
mod addtext;
/// The verbs that change an annotation — delete today, the Format tab's
/// restyles next. Its header carries the seam and the ce-dimension routing
/// obligation every future verb here inherits.
pub(crate) mod annots;
/// What applying an [`Action`] does — the interpreter half of this module.
mod apply;
/// The three verbs whose subject is a whole **file living inside the
/// document** — attach, remove, and save one out (ISO 32000-1 §7.11.4.1).
pub mod attachments;
/// The three verbs whose subject is one entry in the document's outline —
/// add, rename, and delete-with-its-subtree.
pub mod bookmarks;
/// A push button's picture: the picker, then `edit_widget`.
mod buttonicon;
/// Where a Review mark lands: an annotation, or the page's own content.
mod markupdest;
/// Saving an embedded 3D model out to a file.
pub(crate) mod models;
/// Placing a picture or drawing, and selecting what was placed.
mod picture;
/// `ViewChrome` — which piece of View ▸ Display an action is about.
use pdfcer_gui_base::displaypiece as chrome;
/// Committing a paragraph's whole text, from a draft Enter opened on it.
mod blocktext;
/// **Placing one of the operator's OWN stamps** — O172's second half.
mod customstamp;
/// Extracting pages into a new file — the one page verb that writes a file
/// rather than changing the open document. Its header carries the seam
/// against [`pages`].
mod extract;
/// Combine several PDFs into a new file — `OPERATOR_REQUESTS.md` O68.
pub(crate) mod merge;
/// **Committing an edit to text that is already on the page** — the body of
/// `Action::CommitTextEdit`. Its header carries why that body computes rather
/// than routes, and why the two neighbouring commit verbs are not here.
mod reface;
/// Author an Acrobat **stamp collection** from this document's pages —
/// `OPERATOR_REQUESTS.md` O169.
mod stamps;
/// Authoring the annotations that carry WORDS — the sticky note, the text box
/// and the stamp. Its header carries the seam, which is *composes rather
/// than routes*.
mod textannot;
mod textcommit;

/// **Pages dragged out of one open document and into another.**
mod crossdoc;
/// Everything the **ce-dimension** feature asks the document to do — the
/// groups, their scales, standards and style defaults, and the per-ce-dimension
/// overrides.
pub mod dimensions;
/// The sentences one edit owed, and the epoch rule that keeps them honest.
/// Its own header carries the seam.
pub use pdfcer_gui_base::editdisclosure as disclosure;
/// The four actions that replace the open document — Open, New, NewSized,
/// Close — and the two guards all four share.
mod document;
/// What leaves the document — DXF today, and the sixth sibling of [`apply`].
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
/// A hand-drawn signature written into the page inside a signature box.
mod handsign;
/// Stepping the command log, in both directions — `Direction`, its four
/// per-direction answers, and `history_step`.
mod history;
pub use chrome::ViewChrome;
/// The four page verbs' bodies, and the structural resync every edit owes.
pub mod pages;
/// **Changing the paper an open drawing sits on** — `set_media_boxes`, and
/// the pre-commit survey that tells the operator whether he is about to crop
/// his drawing or leave it alone.
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
/// The Find bar's Replace and Replace all.
mod replace;
/// **Record a comment's review status** — `/State` and `/StateModel`,
/// §12.5.6.3. Its own file rather than a place in [`annots`], because that
/// module is *"what happens to a thing that already exists"* and this one adds
/// a separate annotation and changes nothing about the comment it names.
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

/// **Changing how EXISTING text looks** — size, colour, face, weight, slant.
///
/// `pub` because [`Action::TextStyle`] names its `StyleChange` and the
/// Properties panel constructs one.
pub mod textstyle;

/// **Everything that changes page geometry** — delete, the four move verbs,
/// the Bézier handle and the transform. Its header carries the one property
/// every variant shares (they all address paint-order indices into one content
/// stream) and the argument for why there are two verbs that both "move
/// things".
pub mod vector;

/// File ▸ Security ▸ Add archive time-stamp….
#[cfg(feature = "timestamp")]
mod archive;
/// File ▸ Security ▸ Add validation evidence….
#[cfg(feature = "signing")]
mod evidence;
/// Edit ▸ Forms ▸ Repair fonts.
mod formfonts;
/// Create, change and delete a layer: `Action::Layer`.
mod layers;
/// OCR text layers pdfcer wrote: the re-run policy and Remove OCR text.
mod ocrlayers;
// O122 — the two halves of handing the document to Acrobat: the arm that
// raises the question, and the drain that saves, launches and then closes. Its
// header carries the save→launch→close ordering and why the other order loses
// the operator's document off their screen when a `spawn` fails.
mod acrobat;
/// The verbs whose subject is a whole annotation — move, resize, remove. Its
/// header carries what makes them a family: all three find their operand by
/// stable object id, so none needs a page to locate one.
pub mod annot;
/// The verbs that re-shape a page's own text. Its header carries the reason
/// reflow is not like its neighbours: it re-emits the page's FIRST content
/// stream and the commit sweep empties the rest, so it refuses a page carrying
/// a non-empty extra stream.
pub use pdfcer_gui_base::textverbs as text;
/// The three verbs that exist only to move a native file picker out of the
/// layout pass — DXF, form data and a compacted copy. Its header carries the
/// property they share and the reason a SAVE is filed with two exports.
pub use pdfcer_gui_base::writeaction as write;
/// The verbs whose subject is a **form XObject** — the shared drawing a CAD
/// producer invokes from every sheet (§8.10.1).
pub mod xobject;

pub use pdfcer_gui_base::appaction::Action;
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
// Everything below this line is test-only.
//
// A reading convention, not a constraint: `check-ui-strings.sh` skips a
// `#[cfg(test)]` item and resumes at its closing brace, so a mid-file test
// module no longer hides the rest of the file from rule R1.
// ---------------------------------------------------------------------------

/// Plant a disclosure, for tests in other modules that must draw one.
#[cfg(test)]
pub(crate) fn plant_edit_disclosure_for_test(disclosure: EditDisclosure) {
    record_edit_disclosure(Some(disclosure));
}

/// **What an image export IS** — the format, the pages, the resolution and
/// whether transparency survives — decided as a value before anything is
/// written, and with the one combination pdfcer refuses named as an enum rather
/// than as a `bool`. `OPERATOR_REQUESTS.md` O120.
pub use pdfcer_gui_base::imageexport;

/// **What a TEXT export is** — which pages, what goes between them, and how
/// the bytes are encoded — plus the pure parts of making one.
pub use pdfcer_gui_base::exporttext;
/// **What a TABLE export is** — the plan, the cell grid, number recognition
/// and CSV encoding.
pub use pdfcer_gui_base::tableexport;
pub use pdfcer_gui_base::wordexport;
/// File ▸ Export ▸ Tables…: detect, filter to the plan, write.
mod export_tables;
/// File ▸ Export ▸ Word document…: lay out, find tables, write `.docx`.
mod export_word;
/// **A text file becomes pages** — `Action::ImportText`'s body, on
/// `EditSession::place_text`. Mostly a disclosure: its header lists the
/// judgements `PlaceTextReport` carries about the operator's own file, and why
/// the engine's ready-made sentences are not the ones printed.
pub mod importtext;
mod purge_passwords;
/// **The two actions that change what is SELECTED and nothing else.** Its
/// header records why that is a real boundary rather than a size cut — every
/// other `Action` variant asks the document to
/// change, and these two touch only shell state.
pub mod selecting;
/// A tagged PDF's own structure for the Word and table exports, with its disclosure.
use pdfcer_gui_base::taggedexport as tagged;

#[cfg(test)]
mod tests;
