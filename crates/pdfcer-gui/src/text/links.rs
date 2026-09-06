//! # `text::links` — what the program says about a link it cannot follow
//!
//! Consumed by [`crate::canvas::links`]. Five sentences, and **four of them are
//! about failure**, which is the shape of the problem rather than pessimism.
//!
//! ## ★★★ Why a link that works says nothing at all
//!
//! Because it navigates. `pdfcer_core::outline::Destination::Page` is the only
//! variant this program can perform, and performing it *is* the feedback — the
//! page turns, the zoom changes, the operator arrives. A status line reading
//! *"followed a link"* over a view that has visibly moved is noise, and R9's
//! reading of the whole surface is that a capability which works is not
//! something to announce.
//!
//! The **cursor** is the whole of the pre-click affordance: a pointing hand
//! over a link that can be followed and nothing over one that cannot. That is
//! rule 4's pre-commit clause — a cursor is an affordance, not a mark on the
//! content — and it is also every reader ever written.
//!
//! ## ★★ Why the four failures are FOUR sentences and not one
//!
//! `Destination` has five variants and only one navigates. The engine's own
//! note on shipping the reader is the argument, quoted because it is exact:
//!
//! > *"A viewer that maps the last four to 'no link here' reports a document
//! > full of working links as empty. One that maps them to a page jump lies
//! > about where it goes."*
//!
//! And they fail for **different reasons with different remedies**, which is
//! why one generic *"this link doesn't work"* would be worse than useless:
//!
//! | variant | cause | what the operator can do |
//! |---|---|---|
//! | `UnmappedPage` | the target page is not in this document | usually **a page delete** — theirs or an earlier tool's |
//! | `Named` | a name neither namespace defines | usually a **page-range extraction that dropped `/Names`** |
//! | `Remote` | `/GoToR` — a page of another file | open that file |
//! | `NonNavigation` | `/URI`, `/Launch`, `/JavaScript`, `/SubmitForm` | nothing here; pdfcer does not perform these |
//!
//! Telling somebody their link is broken when the truth is *"this link opens a
//! web page and this program does not open web pages"* sends them looking for
//! a defect in their document that does not exist.
//!
//! ## ★ Where these appear, and where they must never appear
//!
//! **Off-canvas, in the status line, on a click.** Never as a mark on the page,
//! never as a tint over the link's rectangle, never as a badge. Rule 4's
//! disclosure clause is explicit that an inference is reported *beside* the
//! content and not drawn *into* it, and this project's own record of the old
//! GUI is that *"the nagging and red flagging … made for a lot of extra bugs in
//! the visibility when editing"*.
//!
//! ★★ They are also raised **only on a click**, never on hover. A sentence that
//! appeared merely because the pointer crossed a rectangle would fire dozens of
//! times crossing a table of contents, and a status line that changes without
//! the operator having done anything is a status line they stop reading.

/// A `/GoTo` whose target page is not in this document's page tree.
///
/// ★ The commonest cause by a distance is a **page delete** — this document was
/// made by extracting or removing pages from a larger one, and the link's
/// target went with them. Saying so is the difference between a sentence the
/// operator can act on and one they can only be annoyed by.
#[must_use]
pub fn unmapped_page() -> &'static str {
    "This link points at a page that is not in this document. That usually means the page was deleted, or this file was made from a range of a larger one."
}

/// A named destination that neither §12.3.2.3 namespace defines.
///
/// ★ Deliberately does **not** say "broken link". The name is very often
/// perfectly good and the *name table* is what went missing — a page-range
/// extraction that dropped `/Names` leaves every by-name link in the file
/// unresolvable at once, which is a different repair from fixing one link.
#[must_use]
pub fn unresolved_name(name: &str) -> String {
    format!(
        "This link points at a destination named \"{name}\", which this document does not define. That usually means the name table was lost when the file was made."
    )
}

/// A `/GoToR` — a destination in a different file.
///
/// ★ pdfcer does not open it, and the sentence says which file rather than
/// merely refusing: the operator can open it themselves, and a refusal that
/// withholds the filename makes them go looking through the document's
/// internals for something the program already knew.
#[must_use]
pub fn remote(file: &str) -> String {
    format!("This link points into another file — {file}. Open that file to follow it.")
}

/// A `/URI`, `/Launch`, `/JavaScript`, `/SubmitForm` or other non-navigation
/// action.
///
/// ★★ **Recognised and disclosed, never executed**, and the sentence is worded
/// so that it cannot be read as a failure. pdfcer's standing rule is that it
/// *never executes anything it fetched*; a link that runs JavaScript is a link
/// this program will describe and will not perform, which is a decision rather
/// than a gap. Whether `/URI` should one day open a browser is the operator's
/// call and is tracked as its own request, not something a viewer decides on
/// their behalf.
#[must_use]
pub fn non_navigation(action: &str) -> String {
    format!(
        "This link is a {action} action, not a page jump. pdfcer shows what it is and does not run it."
    )
}

/// As [`non_navigation`], **naming the file the action opens**.
///
/// ## ★ Why the file is worth its own sentence
///
/// The engine began resolving a `/Launch` action's file specification on
/// 2026-09-06, having previously resolved one for `/GoToR` and discarded it
/// here — the same key, the same question, answered in one case only. The
/// operator case that drove it is one this audience has: a table-of-contents
/// PDF whose entries open the other drawings in a folder.
///
/// Before this, such a link said *"this is a Launch action"* and stopped,
/// which tells an operator that something exists and nothing about it. The
/// file name is the whole content of the link.
///
/// ## ★★ It still says pdfcer does not run it, and that clause is not padding
///
/// R13 — recognised and disclosed, **never executed**. Naming a file is the
/// point at which a reader might reasonably expect a click to open it, so the
/// refusal has to travel in the same sentence as the name. A disclosure that
/// grew more informative and quietly dropped its refusal would be the worse
/// half of this change.
pub fn non_navigation_file(action: &str, file: &str) -> String {
    format!(
        "This link is a {action} action that opens {file}. pdfcer shows what it is and does not run it."
    )
}

/// **A remote file and a page number**, for [`remote`]'s hole.
///
/// ★★ `page` arrives **1-based**. `RemoteTarget::PageNumber` is 0-based, as
/// every page index in `pdfcer-core` is, and every page number this program
/// shows an operator is 1-based — the engine's own reply on shipping the reader
/// flagged that conversion as the thing it nearly got wrong in its CLI. It is
/// done at the call site rather than here so that this function has one job and
/// the arithmetic sits next to the type it is converting from.
#[must_use]
pub fn remote_page(file: &str, page: u64) -> String {
    format!("{file}, page {page}")
}

/// **A remote file and a destination NAME**, for [`remote`]'s hole.
///
/// ★ The name is shown rather than swallowed. It belongs to the *target file's*
/// namespace (§12.6.4.3) and is meaningless here, but it is what the operator
/// would search for in that file — and a sentence naming only the file leaves
/// them to open a hundred-page document and hunt.
#[must_use]
pub fn remote_named(file: &str, name: &str) -> String {
    format!("{file}, {name:?}")
}

/// A `/Link` carrying neither `/Dest` nor `/A`.
///
/// Table 173 gives a link no other way to act, so this one is clickable and can
/// never do anything. It is genuinely malformed — usually the residue of an
/// action stripped by a sanitiser — and unlike the four above there is nothing
/// to point the operator at except the fact.
#[must_use]
pub fn no_destination() -> &'static str {
    "This link has no destination at all. It is a clickable box the document never finished."
}
