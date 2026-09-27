//! # `subactions` — the per-domain verbs an Action carries, as data
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/subactions.md`.

use pdfcer_core::object::ObjId;
use pdfcer_core::vector::{Handle, Matrix, Point};

/// The verbs whose subject is one entry in the document's outline.
#[derive(Debug, Clone, PartialEq)]
pub enum BookmarkAction {
    /// **Add a bookmark to the document's outline.**
    ///
    /// Raised by `crate::panels::bookmarks::add` and by nothing else.
    ///
    /// # Why nothing here counts anything
    ///
    /// `EditSession::add_outline_item` maintains `/Count`, and `/Count` is two
    /// different quantities — see the module header's table. The consequence
    /// the engine flagged as *"the entire difficulty of the feature"*: **adding
    /// a bookmark under a collapsed ancestor does not change the document's
    /// total**, because the new item is not visible. A surface reporting
    /// *"added N"* by diffing the root count therefore reports **zero for a
    /// correct save**.
    ///
    /// So this variant carries one bookmark, the apply arm adds one bookmark,
    /// and the panel says one bookmark. There is no number to get wrong.
    ///
    /// # Why the parent is an `ObjId` and not a position
    ///
    /// Because a position is invalidated by the very edit this performs. The
    /// engine hit that in its own CLI — *"the indices shift after every add …
    /// I got this wrong myself while driving the command and nested something
    /// two levels deeper than intended, and the output looked entirely
    /// plausible."* `OutlineItem::id` exists for this.
    ///
    /// `None` is the top level, which is `add_outline_item`'s own spelling.
    Add {
        /// The item it goes under, or `None` for the top level.
        parent: Option<ObjId>,
        /// The title. Trimmed and non-empty by the time it gets here.
        title: String,
        /// The 0-based page it points at — the one the operator is looking at.
        page: usize,
    },
    /// **Rename a bookmark** — write a new `/Title` onto one outline item.
    ///
    /// Raised by `crate::panels::bookmarks::edit` and by nothing else. The
    /// commonest bookmark edit there is, which is why it is the verb the panel
    /// puts first once a row is selected.
    ///
    /// # The verb with no structural risk, and saying so is load-bearing
    ///
    /// `set_outline_title`'s own doc comment is unusually reassuring, and the
    /// reassurance is a fact a reader of *this* file needs:
    ///
    /// > *"a title is a text string (§7.9.2) on one dictionary, and nothing in
    /// > the `/First`/`/Last`/`/Next`/`/Prev`/`/Count` machinery depends on
    /// > it."*
    ///
    /// **A rename cannot move, orphan, hide or renumber anything.** That is
    /// why this arm reports no disclosure at all: there is no consequence the
    /// operator cannot see. The new title appears in the row they are looking
    /// at, on the next frame, and that is the whole of what happened. Every
    /// other verb in this enum owes a sentence to `app::status`; this one owes
    /// none, and inventing one — *"Bookmark renamed."* under a row that now
    /// visibly reads the new name — would be noise standing where a real
    /// disclosure belongs.
    ///
    /// # Why the title travels by value
    ///
    /// The panel holds a **draft** that the operator is still typing into, and
    /// the queue drains after the frame. Borrowing it would tie the action's
    /// lifetime to the panel state, which `PdfcerApp::apply` cannot reach — it
    /// has no `egui::Context` and deliberately does not — so the operand comes
    /// with it, which is what an action *is*: a complete statement of intent,
    /// resolvable after the frame that raised it.
    ///
    /// Encoding is the engine's problem and is documented as deliberately not
    /// ours: `set_outline_title` routes through *"the same `crate::textstring`
    /// path every other text string uses"*, one path so that two cannot
    /// disagree about PDFDocEncoding. So an em dash or an accented name in this
    /// `String` needs nothing from this crate.
    Rename {
        /// The outline item whose `/Title` is being replaced.
        item: ObjId,
        /// The new title. Trimmed and non-empty by the time it gets here — a
        /// bookmark with a blank title is legal and is an invisible row, which
        /// is the same defect as no row.
        title: String,
    },
    /// **Delete a bookmark AND everything under it.**
    ///
    /// Raised by `crate::panels::bookmarks::edit` and by nothing else.
    ///
    /// # The subtree goes too, and that is a decision with a reason
    ///
    /// The engine takes Acrobat's behaviour and states the alternative it
    /// rejected, which is the part worth carrying here because it is the part
    /// an operator would otherwise discover:
    ///
    /// > *"promoting orphaned children to the deleted item's parent silently
    /// > **reorganises** a document's navigation, and an operator who deleted
    /// > one chapter heading would find its ten sections spliced into the top
    /// > level. Deleting what was asked for is the predictable act."*
    ///
    /// This is therefore a verb whose blast radius is **larger than the thing
    /// the operator clicked**, and the whole of the UI obligation follows from
    /// that one sentence. It is stated before the press by
    /// `crate::panels::bookmarks::edit`, from the tree the panel already drew,
    /// and it is stated again after the press from the engine's own count. See
    /// `pdfcer_gui::app::actions::bookmarks::delete` for why the answer is given twice and why the two numbers are
    /// allowed to differ.
    ///
    /// # Why there is no confirmation dialog, and it IS a choice
    ///
    /// A destructive act must be **confirmed or clearly undoable**, and this
    /// is the second. One press produces **one** `EditSession` command, so one
    /// `Ctrl+Z` puts the entire subtree back — the engine plans every relink
    /// (`/Prev`, `/Next`, the parent's `/First`/`/Last`, every open ancestor's
    /// `/Count`) inside that one command, so there is no half-undone state to
    /// reach. A modal would buy nothing that the undo does not already buy, and
    /// it would cost the thing modals always cost: an operator who has answered
    /// *"are you sure?"* four times stops reading it, and the fifth one is the
    /// one that mattered.
    ///
    /// The consequence the operator actually needs is **not** *"are you
    /// sure?"* — it is *"this takes the eleven bookmarks underneath as well"*,
    /// which a confirmation dialog is a bad place to put because it arrives
    /// after the decision. It is on the panel, beside the button, before the
    /// press.
    ///
    /// # No page index
    ///
    /// An outline is a document-level structure (§12.3.3) reached from the
    /// catalogue's `/Outlines`, not from any page. The item's own destination
    /// may name a page, and it is irrelevant here: this deletes the bookmark,
    /// never the page it points at, and nothing on any page changes.
    Delete {
        /// The outline item to remove, together with its whole subtree.
        item: ObjId,
    },
    /// **Move a bookmark — reorder it among its siblings, or re-parent it
    /// under a different one — carrying its whole subtree.**
    ///
    /// Raised by `crate::panels::bookmarks::reorder` and by nothing else.
    ///
    /// Without it an outline in the wrong **order** could only be fixed by
    /// deleting a branch and re-authoring it, which loses every destination,
    /// colour and style on it — not an edit any operator would call a
    /// reorganisation.
    ///
    /// # The subtree travels, and the destination does not move
    ///
    /// `move_outline_item`'s own words: *"A chapter dragged under a different
    /// part takes its sections with it."* That matches
    /// [`Self::Delete`]'s subtree semantics and Acrobat's model — its
    /// `PDBookmark` unlink/add-child pair operates on the node, which owns its
    /// children wherever `/Parent` points, and there is no API path that leaves
    /// them behind.
    ///
    /// So this verb, like the delete, has a **blast radius larger than the
    /// row the operator clicked** — and unlike the delete, the size of it is
    /// reported by the engine rather than counted by the panel. See `pdfcer_gui::app::actions::bookmarks::move_to`
    /// for the two numbers and why both are needed.
    ///
    /// # Why the placement is an anchor and NEVER an index
    ///
    /// `OutlinePlacement`'s own doc comment states the rule and names the
    /// failure this shell would otherwise walk into:
    ///
    /// > *"An outline's siblings are a **doubly-linked list** (§12.3.3 Table
    /// > 153: `/Prev`, `/Next`), not an array — there is no stored index, so an
    /// > index parameter would have to be *counted* by walking the chain, and
    /// > every caller holding one would be holding a number that silently goes
    /// > stale the moment any sibling is added or removed. **A shell that reads
    /// > a panel, lets the operator drag a row, and then calls with the index
    /// > it read has a race with its own undo stack.**"*
    ///
    /// That is this panel, described from the other side of the API. It is the
    /// same rule the whole of this module is built on — every variant here
    /// addresses its operand by `ObjId` — applied to the *destination* as well
    /// as to the subject.
    ///
    /// # Why there is no separate promote or demote verb
    ///
    /// Because they are this variant with a different anchor, and the engine
    /// refuses to spell one operation twice: *"a second spelling of one
    /// operation is exactly how two implementations of one rule come to
    /// disagree (`R171`)."* Re-parenting to the top level is
    /// `FirstChild { parent: None }` or `After` a top-level sibling; nesting is
    /// `LastChild { parent: Some(..) }`. The panel's three drop bands produce
    /// all of them.
    ///
    /// # The expansion of the destination is NOT folded in here
    ///
    /// The engine shipped [`Self::SetOpen`] alongside this verb and said why in
    /// a sentence that binds this shell:
    ///
    /// > *"Expand/collapse ships alongside, as a separate verb, because whether
    /// > a move should reveal a collapsed destination has two defensible
    /// > answers and both now exist."*
    ///
    /// pdfcer takes *"leave it as the operator set it"*, which is
    /// `move_outline_item`'s own default — a destination parent that already
    /// has children keeps its `/Count` sign — and discloses the consequence
    /// instead. A `reveal: bool` on this variant would bury a second state
    /// change inside an unrelated command and would produce **one** undo entry
    /// **Put a copied bookmark subtree into this document's outline.**
    ///
    /// `OPERATOR_REQUESTS.md` **O59** item 3. Raised by
    /// `panels::bookmarks::clip::paste_row` and by nothing else.
    ///
    /// **Acrobat cannot do this between two files at all**, by Adobe's own
    /// documentation. There is therefore no established behaviour to match and
    /// no borrowed wording — which is why the disclosure below is written from
    /// what the operation does rather than from what a reference implementation
    /// says about it.
    ///
    /// # The disclosure this arm owes
    ///
    /// `OutlinePasteOutcome::destinations_dropped`. A destination naming a page
    /// this document does not have is **dropped, not clamped** — so the
    /// bookmark arrives, shows, keeps its title, and does nothing when clicked.
    /// Nothing on screen distinguishes it from one that works.
    ///
    /// The panel warns about this **before** the press as well, from
    /// `OutlineClip::deepest_page()` against the page count. The two are not
    /// duplicates: the panel's is a prediction the operator can act on, and
    /// this one is what actually happened. A prediction alone would be a guess
    /// nobody confirmed; a report alone would arrive too late to choose
    /// differently.
    Paste {
        /// The copied roots and their children.
        clip: Box<pdfcer_core::outline::OutlineClip>,
        /// Where they go, as an anchor. Never a position — `Move`'s rule, and
        /// its documentation carries why.
        to: pdfcer_core::edit::OutlinePlacement,
    },
    /// for two acts.
    Move {
        /// The bookmark being moved, together with everything filed under it.
        item: ObjId,
        /// Where it is going, as an anchor. Never a position.
        to: pdfcer_core::edit::OutlinePlacement,
    },
    /// **Expand or collapse a bookmark** — flip the sign on its `/Count`.
    ///
    /// Raised by `crate::panels::bookmarks::reorder`'s disclosure triangle and
    /// by nothing else.
    ///
    /// # This is a document edit, and every other program makes it a view
    /// setting
    ///
    /// The single most surprising thing about this verb, and the reason the
    /// triangle's hover text says it out loud. §12.3.3 Table 153 carries
    /// open-or-closed as the **sign** on `/Count` and defines no `/Open` key,
    /// so there is nowhere in the file to record a per-viewer answer. Expanding
    /// a bookmark therefore:
    ///
    /// * writes objects, and marks the document modified;
    /// * lands on the undo stack as one entry;
    /// * is **seen by everybody who opens the file afterwards**.
    ///
    /// An operator who collapses three chapters to find their place, saves, and
    /// sends the drawing out has changed what the recipient sees. That is not a
    /// defect — it is what the format is — and it is why the disclosure is on
    /// the control rather than in a release note.
    ///
    /// # The magnitude is the engine's problem, and getting it wrong is
    /// silent
    ///
    /// `set_outline_open` propagates the flip up the ancestor chain by the
    /// `/Count` **magnitude**, not by one, and its doc comment is emphatic:
    ///
    /// > *"a closed node contributes 1 (itself); an open one contributes
    /// > `1 + magnitude`. So expanding a node with magnitude 7 adds **7** to
    /// > every ancestor up to the first closed one — not 1, and not 8."*
    ///
    /// Nothing in this shell computes that, and nothing in this shell may. A
    /// wrong `/Count` is invisible: the file opens, the outline draws, and the
    /// only symptom is another reader's panel disagreeing about what is there.
    ///
    /// # A leaf is never asked
    ///
    /// An item with no descendants carries no `/Count` at all — Table 153 makes
    /// it *"required if the item has any descendants"* — so there is nothing to
    /// flip. `set_outline_open` answers `Ok(false)` rather than refusing,
    /// because *"asking a leaf to expand is what a 'collapse all' sweep does to
    /// every row it walks"*, and the panel simply draws no triangle on one.
    /// R83: never offer a control for something that cannot work.
    SetOpen {
        /// The bookmark whose `/Count` sign is being flipped.
        item: ObjId,
        /// `true` to expand it, `false` to collapse it.
        open: bool,
    },
}
/// **What a File-tab command asks the document to do**, when the answer is an
/// edit rather than a write.
#[derive(Debug, Clone, PartialEq)]
pub enum FileAction {
    /// **Turn a plain text file into new pages** — `file.import_text`.
    ///
    /// Raised by `pdfcer_gui::dialogs::import_text`'s Import button and by nothing
    /// else. This module's header carries why the file is read here rather than
    /// in the window.
    ImportText {
        /// The text file to read. UTF-8 is assumed and a failure to decode is
        /// reported by `pdfcer_gui::app::actions::importtext::import`, not guessed at in the dialog.
        path: std::path::PathBuf,
        /// The sheet, margins, face and size the operator chose.
        ///
        /// **Boxed**, and clippy is not the reason: `PageTemplate` carries a
        /// rect, four margins, a face, a size, an optional leading, an
        /// alignment, a colour and a policy — the largest thing any `Action`
        /// variant holds — and every action in the queue is as large as the
        /// largest. Boxing keeps a queue of a hundred selection changes from
        /// carrying a page template each.
        template: Box<pdfcer_core::text_edit::PageTemplate>,
        /// Where the new pages land, in the engine's own vocabulary — the same
        /// field `pdfcer_gui::app::actions::pages::PageAction::InsertPagesFromFile`
        /// carries, verbatim and for the same reason.
        position: pdfcer_core::pageops::InsertPosition,
    },
}
/// Everything the operator can ask of the document's **set of pages**.
#[derive(Debug, Clone, PartialEq)]
pub enum PageAction {
    /// **Insert another document's pages into this one, after the current page.**
    ///
    /// Raised by `pages.insert_from_file` once the picker has answered.
    ///
    /// # Why this is an editing verb and not an open
    ///
    /// `pdfcer_core::pageops::insert` also inserts pages, and returns the bytes
    /// of a **new document**. Wiring that would have meant replacing
    /// `OpenDoc::session` wholesale, which discards the undo stack — invisible
    /// in any test that checks page counts, and visible the first time an
    /// operator presses Ctrl+Z twice.
    ///
    /// So it was filed rather than shipped, and `pdfcer-core` answered the same
    /// day with `EditSession::insert_pages`: the missing member of the
    /// `delete_pages` / `reorder_pages` / `rotate_pages` family. It records
    /// **one** undoable command however many pages arrive, exactly as a reorder
    /// does however many pages move.
    ///
    /// # What it does not carry, and why the operator is told
    ///
    /// The session verb copies each page and everything reachable from it —
    /// content, resources, fonts, XObjects — at fresh object numbers. It does
    /// **not** merge the source's document-level structures: outlines, the
    /// AcroForm field tree, named destinations, page labels. That is the honest
    /// cost of staying incremental, because a document-level merge rewrites
    /// objects an incremental save exists in order not to touch.
    ///
    /// `crate::text::pages::inserted` says so, because an operator whose
    /// bookmarks did not come across is entitled to know that before they go
    /// looking for a bug.
    /// **Merge a whole document into this one**, with its form, bookmarks and
    /// named destinations.
    ///
    /// It carries a path and nothing else, where [`Self::InsertPagesFromFile`]
    /// carries a page list and a position. That asymmetry IS the difference
    /// between the two verbs: a merge takes the whole document and appends it,
    /// so there is nothing to choose. See `pdfcer_gui::app::actions::pages::merge_into` for why
    /// `InsertPosition::End` is a decision rather than a default.
    MergeIntoDocument {
        /// The document to merge in.
        path: std::path::PathBuf,
    },
    InsertPagesFromFile {
        /// The document to take pages from.
        path: std::path::PathBuf,
        /// Which of ITS pages, 0-based, **in the order the operator asked
        /// for**.
        ///
        /// Order is carried rather than sorted, and duplicates are kept,
        /// because the range grammar treats the text as a sequence: `3,1-2`
        /// inserts source page 3 first, and `1,1` inserts a page twice. Both
        /// are things an operator can only ask for in one gesture if this
        /// field preserves them.
        pages: Vec<usize>,
        /// Where they land, in the engine's own vocabulary.
        ///
        /// `pdfcer_core::pageops::InsertPosition` directly rather than a
        /// local enum mapped at the boundary. Four choices — `Start`, `End`,
        /// `Before(n)`, `After(n)` — and a second spelling of them would be a
        /// second place for "before" and "after" to drift, where the drift is
        /// silent because both compile and both insert *somewhere*.
        position: pdfcer_core::pageops::InsertPosition,
    },
    /// **Turn the operand pages by `delta` degrees**, as one undoable command.
    ///
    /// Raised by `pages.rotate_left` (−90) and `pages.rotate_right` (+90).
    ///
    /// # Why a delta rather than an absolute angle
    ///
    /// Because that is what the button means and what `EditSession::rotate_pages`
    /// implements: a selection of pages at 0°, 90° and 180° turned right lands
    /// at 90°, 180° and 270°, **not** all at 90°. The engine's own doc comment
    /// confirms Acrobat persists the absolute result of exactly that
    /// arithmetic. An absolute variant would be a different verb (*set the
    /// rotation of these pages to N*), which no control in this build offers.
    ///
    /// # It changes no page's identity
    ///
    /// A rotation rewrites one `/Rotate` entry per page. Nothing is added,
    /// removed or renumbered, so both selections survive it untouched — which
    /// is why the apply arm's resync is about *pictures* (every cached raster
    /// of a turned page is now wrong) and not about *indices*.
    RotatePages {
        /// 0-based page indices, ascending and unique.
        pages: Vec<usize>,
        /// A relative turn in degrees, a multiple of 90.
        delta: i32,
    },
    /// **Put the operand pages on a different size of paper**, as one
    /// undoable command however many sheets it touched.
    ///
    /// Raised by `crate::dialogs::page_size` and by nothing else — the
    /// window is the only place the two questions this verb needs answering
    /// (*which sheet, and do you understand it will crop*) can both be put.
    ///
    /// # It changes no page's identity, and it does not move the drawing
    ///
    /// A `/MediaBox` change rewrites one page-dictionary entry. Nothing is
    /// added, removed or renumbered, so both selections survive it untouched
    /// and the resync is about **pictures** — every cached raster of a
    /// resized sheet is now the wrong shape — exactly as it is for
    /// [`Self::RotatePages`].
    ///
    /// ⚠ And nothing drawn on the page moves either. That is the whole
    /// subject of `pdfcer_gui::app::actions::pagesize`, which carries the
    /// measurement: an A1 drawing put on A4 paper is **cropped, not shrunk**.
    ///
    /// # Why a rectangle rather than a width and a height
    ///
    /// `pdfcer_gui::app::actions::Action::NewSized` carries a *size* because a
    /// new page's lower-left corner is the origin by construction. An
    /// existing page's is not — §7.7.3.3 does not require one, and imposition
    /// output and cropped scans carry offset boxes — so the corner is a real
    /// choice, it is made once in
    /// `pdfcer_gui::app::actions::pagesize::SheetSurvey::target_rect` against the
    /// sheets actually picked, and it travels with the request rather than
    /// being re-derived from a survey that may no longer exist.
    SetPageSize {
        /// 0-based page indices, ascending and unique.
        pages: Vec<usize>,
        /// The `/MediaBox` to write on every one of them, §7.9.5-normalized
        /// by the engine before anything is touched.
        rect: pdfcer_core::page_tree::Rect,
    },
    /// **Remove the operand pages from the document**, as one undoable
    /// command.
    ///
    /// Raised by `pages.delete` from the ribbon's Pages tab and from the page
    /// tile's context menu.
    ///
    /// # This is the one action in the enum that renumbers pages
    ///
    /// A selection is an identity — page, object, subpath, node — not a
    /// position, and this is that rule's page-level instance. After the
    /// removal, every index above the lowest
    /// deleted page names a **different sheet**. Both selections in the
    /// application are therefore invalid, in different ways, and the apply arm
    /// deals with both:
    ///
    /// * the **page** selection named exactly the sheets that no longer exist,
    ///   so it is cleared;
    /// * the **canvas** selection names objects on a page *index*, and that
    ///   index now resolves to another sheet's content, so it is cleared too.
    ///
    /// # It is destructive and, until undo lands, irreversible
    ///
    /// No confirmation dialog, deliberately, and the reasoning is at the apply
    /// arm: `crate::app::save`'s `save_pending` is the one predicate this
    /// application consults before a destructive path, the engine records the
    /// removal as an undoable command already, and **nothing is written to
    /// disk** — the operator's file on disk is untouched until they save a
    /// **Put copied pages into this document**, after `after`.
    ///
    /// `OPERATOR_REQUESTS.md` **O59**, item 2. Raised by
    /// `app::dispatch::pageclip::paste` and by nothing else.
    ///
    /// # It carries BYTES, and those bytes are a whole PDF
    ///
    /// `PageClip::bytes` is a complete document. The engine chose that
    /// deliberately — `pageops::assemble` already does object copying,
    /// reference remapping and page-tree construction on every split and merge,
    /// so a private page format would have been a second implementation of the
    /// most-exercised code in that crate.
    ///
    /// The clip travels as bytes rather than as a `PageClip` for `Action`'s own
    /// requirements (`Clone`, `PartialEq`) and because that is what the
    /// clipboard is holding. It is re-parsed at apply time, which is one
    /// document parse per paste and is the same cost `paste_pages` pays
    /// internally anyway.
    ///
    /// # What the arm must surface, and why it is not optional
    ///
    /// `InsertOutcome::orphaned_widgets`. A page's `/Annots` reaches its
    /// form-field boxes, so they travel; the `/AcroForm` that owns them is a
    /// catalog entry and does not. The boxes arrive drawn, positioned and
    /// looking exactly like working fields, belonging to nothing — so nothing
    /// can fill them and no viewer complains. The engine measured **two** on
    /// its own smoke test, which makes this the ordinary case for pasting a
    /// page out of a form rather than an exotic one.
    PastePages {
        /// The clip — a complete PDF document.
        bytes: Vec<u8>,
        /// The 0-based page to insert **after**.
        after: usize,
    },
    /// copy, which is a separate deliberate act with its own dialog.
    DeletePages {
        /// 0-based page indices, ascending and unique.
        pages: Vec<usize>,
    },
    /// **Put the document's pages in a new order**, as one undoable command.
    ///
    /// Raised by `pages.move_up` and `pages.move_down`, which differ only in
    /// the permutation `crate::panels::pages::ops::move_order` computes.
    ///
    /// `order[i]` is the **current** 0-based index of the page that should end
    /// up at position `i` — `EditSession::reorder_pages`' contract verbatim,
    /// carried through unaltered so there is no second spelling of it to drift.
    /// The engine refuses anything that is not a permutation of
    /// `0..page_count`, and `move_order` builds one by construction.
    ///
    /// # A reorder renumbers positions without destroying anything
    ///
    /// Which makes it the *middle* case between a move (nothing changes
    /// identity) and a delete (identities cease to exist), and the two
    /// selections get two different answers:
    ///
    /// * the **page** selection follows its sheets, through
    ///   `pdfcer_gui::panels::pages::select::PageSelection::remap` — the
    ///   permutation states exactly where each one went, so clearing would
    ///   throw away information the edit had in hand and make the reorder
    ///   arrows unusable twice in a row;
    /// * the **canvas** selection is cleared, because its entries carry a page
    ///   *index* and this crate cannot rewrite them —
    ///   `crate::canvas::selection::SelectionState` exposes no mutator for the
    ///   page of an entry, and inventing one would put a second page-remapping
    ///   rule in the module that owns object identity. Clearing is the honest
    ///   answer and it is stated rather than silent.
    ReorderPages {
        /// The new order, as `order[new_position] = current_index`.
        order: Vec<usize>,
    },
    /// **Write the operand pages out as a new standalone document.**
    ///
    /// Raised by `pages.extract`. The one page verb that changes **no**
    /// document: `pdfcer_core::pageops::extract` returns the complete bytes of a
    /// freestanding PDF and the open session is not touched, which is exactly
    /// what the Review mode's stance requires — `crate::panels::pages`' header
    /// quotes the operator: *"an extraction writes a different file."*
    ///
    /// # Why it is an action at all, when it mutates nothing
    ///
    /// For [`Self::SaveCopy`]'s reason and only that one: it opens a **native
    /// save dialog**, and `crate::app::files::pick_save_path` carries a
    /// frame-timing requirement dispatch cannot honour — `PdfcerApp::central`
    /// dispatches the canvas's context-menu tokens from inside
    /// `egui::CentralPanel::show`, and a modal opened mid-layout blocks the
    /// frame it is being drawn in. The apply phase is always outside every
    /// closure. The page tile's context menu is dispatched from a panel body
    /// rather than the canvas, but the rule is the surface's, not the caller's.
    ExtractPages {
        /// 0-based page indices, ascending and unique. **Order is honoured** by
        /// the engine, so this is simultaneously "extract these pages" and
        /// "extract them in this order"; the panel produces them ascending.
        pages: Vec<usize>,
    },
}
/// **What is selected**, as an action a panel can raise.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectionAction {
    /// **Everything on the page, wherever it now sits** — `edit.select_all`.
    ///
    /// The operator, 2026-09-01: *"we should be able to select things off the
    /// side of the page, especially since I sometimes drop objects there, and
    /// when I do I can't get them back."* The whole argument is on the arm in
    /// `pdfcer_gui::app::actions::selecting::apply_action`, where the marquee's `Enclosed` mode and the
    /// deliberately unbounded rectangle are.
    SelectAllOnPage,
    /// **Select exactly this object** — raised by the Objects panel when a
    /// row is clicked.
    ///
    /// # Why a panel raises an action instead of writing the selection
    ///
    /// Because a panel body is handed `&OpenDoc`, not `&mut`, and that is
    /// deliberate: a surface that could mutate the document while it is being
    /// drawn is a surface that can change what a later widget in the same frame
    /// is describing. Every other panel that changes something raises an action
    /// for the same reason, and this is not the place to make an exception.
    ///
    /// # One selection, written from both ends
    ///
    /// `doc.selection` is the only notion of *"the thing I am working on"*, and
    /// a panel must not grow a private second one. A panel-local focus field
    /// that the canvas neither writes nor reads is how the operator gets
    /// *"when I have an object selected like text the Tool tab doesn't switch
    /// to giving me the editable stuff for that object"* — the panel and the
    /// canvas each believing something different is selected, with no bridge
    /// between them. Raising this action is the bridge.
    SelectObject {
        /// The page the object is on, in the session's page space.
        page: usize,
        /// Which object, as a paint-order target — or `None` to select
        /// nothing.
        ///
        /// `None` rather than a second variant, because a row click is one
        /// act with one outcome: *this row is now the selection*. Clicking the
        /// already-selected row makes that selection empty, which is what
        /// clicking a selected item does in every list in every application,
        /// and splitting it into Select and Clear would make the caller decide
        /// which act it was performing when it only ever performs one.
        object: Option<crate::canvastarget::TargetId>,
    },
}
/// One change to the marks on a page.
#[derive(Debug, Clone, PartialEq)]
pub enum VectorAction {
    /// Remove the canvas selection's objects from `page`, as **one**
    /// undoable command.
    ///
    /// Raised by the canvas when Delete or Backspace is pressed with a
    /// non-empty selection and no text field focused — the defect `DEFECTS.md`
    /// D1 is about, from the other end. D1's fix (`ctx.text_edit_focused()`
    /// rather than `ctx.egui_wants_keyboard_input()`) made the key *reachable*
    /// after a canvas click; this is the verb it reaches.
    ///
    /// # The operand list is already clean, and must be
    ///
    /// `objects` arrives ascending and de-duplicated from
    /// `pdfcer_gui::canvas::selection::SelectionState::object_indices_on`,
    /// because `EditSession::delete_objects` resolves **every** index before
    /// planning anything: one stale or duplicated entry refuses the whole
    /// call. That refusal is the correct engine behaviour — the alternative
    /// is deleting the prefix that happened to resolve — so the shell's job
    /// is to hand it a list that can succeed.
    ///
    /// # Why the page travels with the list
    ///
    /// A paint-order index is a position on **one page**. Re-deriving the
    /// page here from `doc.view.page_index` would be a second source of truth
    /// that is right until the moment it matters: an action is applied after
    /// the frame that raised it, and a page step raised in the same frame is
    /// applied first if it was pushed first. Carrying the page makes the
    /// statement complete.
    DeleteSelection {
        /// The 0-based page the indices are positions on.
        page: usize,
        /// Paint-order indices, ascending and unique.
        objects: Vec<usize>,
    },
    /// Displace the canvas selection's objects on `page` by a **page-space**
    /// delta, as **one** undoable command.
    ///
    /// Raised by `pdfcer_gui::canvas::moving::drag` when a move drag that began
    /// inside the selection is released. The Object-rung member of the move
    /// family; its siblings are [`Self::MoveSubpath`] and [`Self::MoveNode`].
    ///
    /// # Why the whole list travels, exactly as it does for Delete
    ///
    /// `EditSession::move_objects` takes a **slice**, and resolves *and
    /// type-checks* every index before planning anything, so one non-path or
    /// one stale entry refuses the whole call rather than moving the prefix
    /// that happened to qualify. Emitting one `move_object` per selected
    /// object would be wrong twice over: N undo entries for one drag, and — the
    /// correctness half — each call re-splices the content stream, so the
    /// second index would be planned against byte offsets the first already
    /// invalidated. `docs/core-api/02` states it in a box: *"Never loop the
    /// singular verbs over a selection."*
    ///
    /// # Why this does NOT invalidate the selection, and Delete does
    ///
    /// Because `move_*` **does not renumber**, and that is measured rather
    /// than assumed — `crates/pdfcer-core/tests/object_identity_across_edits.rs`
    /// decomposes, edits, and decomposes again. A move rewrites operands
    /// *inside* existing operators, so no operator is added or removed and the
    /// second decomposition yields the same objects at the same indices. The
    /// `delete_*` family excises byte **spans** and therefore does renumber,
    /// which is why `pdfcer_core::vector::remap_index_after_delete` exists and
    /// why nothing like it is needed here. See
    /// `pdfcer_gui::canvas::moving`'s header for the full table.
    ///
    /// # Units
    ///
    /// `dx`/`dy` are **PDF user-space** points, Y-**up** — produced by
    /// `pdfcer_gui::canvas::moving::page_delta`, which is the one place a
    /// canvas-space drag crosses into page space. A screen-pixel delta here
    /// would compile, run, and scale the move with the magnification.
    MoveSelection {
        /// The 0-based page the indices are positions on.
        page: usize,
        /// Paint-order indices, ascending and unique.
        objects: Vec<usize>,
        /// Horizontal displacement, PDF user-space points.
        dx: f64,
        /// Vertical displacement, PDF user-space points (Y is up).
        dy: f64,
    },
    /// Delete every selected object **inside a form XObject** —
    /// `EditSession::delete_objects_in_form`, `OPERATOR_REQUESTS.md` O70.
    ///
    /// Beside [`Self::MoveLeavesInForm`] and for its reason: the indices are a
    /// different address space from `DeleteSelection`'s, and the variant is
    /// what says which. One command however many leaves, exactly as its
    /// page-level twin.
    DeleteLeavesInForm {
        /// The 0-based page.
        page: usize,
        /// Leaf indices, ascending and unique.
        leaves: Vec<usize>,
    },
    /// **Remove ONE subpath of one path object** — `EditSession::delete_
    /// subpath`, Pass 25.2, and the Part rung's delete verb.
    ///
    /// # What it closes
    ///
    ///
    /// The engine's own reason for the verb is the operator's file: *"one
    /// stroked path with 1194 subpaths covering a whole isometric view"*, on
    /// which `delete_object` can only remove the entire view. *"Delete this
    /// line"* is what he means, and this is that operation.
    ///
    /// # Deleting the only subpath deletes the object, and that is the verb's
    /// rule rather than this shell's
    ///
    /// A painting operator with no path left is not a smaller object; it is
    /// meaningless. So a one-subpath path object vanishes, which is both
    /// correct and exactly what the operator asked for — they entered the
    /// object, found it had one line in it, and deleted that line.
    ///
    /// # Disclosures
    ///
    /// **Always empty**, measured against the locked engine rather than assumed:
    /// both arms of `plan_delete_subpath` return `disclosures: Vec::new()`.
    /// The arm below still routes them, because the funnel does that for every
    /// verb and a hand-written exception here would be the thing that stops
    /// being true when the planner grows a re-spelling path.
    DeleteSubpath {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The subpath, in decomposition order — the order
        /// `hit_test_subpaths` returns, so a picked line goes straight here.
        subpath: usize,
    },
    /// **Remove ONE label off a sheet that holds all of them in one text
    /// object** — `EditSession::delete_text_run`, `Pass 32.0`.
    ///
    /// # The defect this closes, in the engine's own measurement
    ///
    /// > *"on the operator's drawing **one text object holds all 237 dimension
    /// > labels**, so deleting 'a label' deleted every one of them."*
    ///
    ///
    /// Those 237 are **pdf dimensions** (R8b Rule 15): page content pdfcer
    /// reads and must not silently alter. A **ce dimension** is one pdfcer
    /// authors, lives in `super::dimensions`, and has nothing to do with this
    /// variant.
    ///
    /// # The refusal that is asked BEFORE the press, and where
    ///
    /// §9.4.2: a following run with no positioning operator of its own starts
    /// wherever this one ends, so excising this one **slides it**. The engine
    /// refuses with `DeleteWouldMoveNextRun`, and
    /// `crate::canvas::deleting` asks the identical question ahead of the
    /// press through `ObjectModelProvider::text_line_delete_would_move_next` —
    /// R83 — so the operator gets the remedy (*delete the later label first*)
    /// instead of a cause-less decline. This variant is therefore never raised
    /// for a run the guard would refuse; if one arrives anyway the engine
    /// still refuses it and the funnel still declines in words.
    ///
    /// # Disclosures
    ///
    /// **Always empty** — `plan_delete_text_run` returns `Vec::new()` on both
    /// of its arms. Routed anyway, for [`Self::DeleteSubpath`]'s reason.
    DeleteTextLine {
        /// The 0-based page.
        page: usize,
        /// The enclosing text object, by paint-order index.
        object: usize,
        /// The visual line, numbered as
        /// `ObjectModelProvider::text_line_count` counts — **not** a
        /// show-operator index. The apply arm translates it into the line's
        /// run range and calls `delete_text_run` once per run, **descending**,
        /// because `plan_delete_text_run` excises `TextRun::bytes` and every
        /// later run in the same object shifts when an earlier one goes.
        line: usize,
    },
    /// **Remove ONE anchor of one path object** — `EditSession::delete_
    /// node`, Pass 36.1, and the Node rung's delete verb.
    ///
    /// Its twins [`Self::MoveNode`] and [`Self::MoveNodes`] have been wired
    /// since Pass 28.0, so on a CAD export the operator could nudge one point
    /// of a polyline and could not remove it.
    ///
    /// ⚠ **This is not the markup-annotation vertex verb.** That family edits
    /// an annotation's `/Vertices` through `reshape_annotation` and its
    /// helpers in `super::annots` are confusingly called `move_node` and
    /// `remove_node`. They share nothing with this but a name — different
    /// address space, different engine verb, different undo command — and the
    /// collision deferred this gap by an evening once already.
    ///
    /// # THE DISCLOSURE THIS ONE OWES, and it is the whole reason the arm
    /// is not one line
    ///
    /// `delete_node` returns a disclosure list that is **non-empty when
    /// deleting the point discarded a curve**:
    ///
    /// > *"The curve that ran into this point was removed along with it, so the
    /// > shape now goes straight from the point before to the point after."*
    ///
    /// That is a shape change the operator **cannot reverse by re-adding a
    /// point** — the two control points are gone — and the engine's own doc
    /// says rule 4 forbids letting them find it out from a diff: *"the caller
    /// must surface these."*
    ///
    /// The surfacing is `super::funnel::vector_edit_on_page`'s, not this
    /// arm's, and that is the point: the funnel records **every** verb's
    /// disclosure list to the status bar's row, stamped with the epoch the edit
    /// produced. So returning the list from the closure *is* surfacing it, and
    /// a hand-written `record_note` beside it would be a second mechanism for
    /// the same sentence — the one that later forgets to retire itself.
    ///
    /// The one refusal worth knowing about, because it is the commonest:
    /// `NodeDeleteWouldEmptySubpath`, when the line has only two points left.
    /// Left to the engine, where it is judged against the bytes.
    DeleteNode {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The anchor, **object-scoped** — the numbering
        /// [`Self::MoveNode`] takes, `vector::anchor_count` reports and
        /// `pdfcer node-move --node N` addresses. A second numbering would make
        /// the number pdfcer shows disagree with the number the operator can
        /// act on.
        node: usize,
    },
    /// Move one **Bézier control point of an object inside a form
    /// XObject** — `EditSession::move_handle_in_form`. O70.
    ///
    /// Absolute, as [`Self::MoveHandle`] is and for its reason: the operand is
    /// a coordinate pair.
    MoveHandleInForm {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by **leaf** index.
        leaf: usize,
        /// The anchor the handle serves, object-scoped.
        node: usize,
        /// Which of the two controls.
        handle: pdfcer_core::vector::Handle,
        /// Where it lands, in PDF user space.
        to: pdfcer_core::vector::Point,
    },
    /// Displace one **subpath of an object inside a form XObject** —
    /// `EditSession::move_subpath_in_form`. `OPERATOR_REQUESTS.md` O70.
    MoveSubpathInForm {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by **leaf** index.
        leaf: usize,
        /// The subpath, in decomposition order.
        subpath: usize,
        /// Horizontal displacement, PDF user-space points.
        dx: f64,
        /// Vertical displacement, PDF user-space points (Y is up).
        dy: f64,
    },
    /// Move one **anchor of an object inside a form XObject** to an
    /// absolute page-space point — `EditSession::move_node_in_form`.
    ///
    /// Absolute rather than a delta, exactly as [`Self::MoveNode`]: the operand
    /// being rewritten is a coordinate pair, and expressing the drag as *"where
    /// the point ends up"* is what makes a refusal leave the document
    /// untouched rather than half-moved.
    MoveNodeInForm {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by **leaf** index.
        leaf: usize,
        /// The anchor, object-scoped.
        node: usize,
        /// Where it lands, in PDF user space.
        to: pdfcer_core::vector::Point,
    },
    /// Move **several anchors** of an object inside a form XObject, as one
    /// command — `EditSession::move_nodes_in_form`.
    MoveNodesInForm {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by **leaf** index.
        leaf: usize,
        /// Each anchor and where it lands, object-scoped indices.
        moves: Vec<(usize, pdfcer_core::vector::Point)>,
    },
    /// Displace every selected object **inside a form XObject** by a
    /// page-space delta — `EditSession::move_objects_in_form`.
    ///
    ///
    /// The coordinates are **page space**, exactly as the page-level verbs
    /// take, and that is the engine's contract rather than this shell's choice:
    /// `FormLeaf` reports geometry already mapped out of the form's own space,
    /// so a caller never has to know the placement matrix. The one thing that
    /// differs is which list the index is a position in.
    MoveLeavesInForm {
        /// The 0-based page.
        page: usize,
        /// Leaf indices, ascending and unique.
        leaves: Vec<usize>,
        /// Horizontal displacement, PDF user-space points.
        dx: f64,
        /// Vertical displacement, PDF user-space points (Y is up).
        dy: f64,
    },
    /// Displace **one subpath** of one path object by a page-space delta, as
    /// one undoable command — the Part rung's move verb.
    ///
    /// Raised only when the entered object decomposes into *subpaths*. A text
    /// object's Part rung is a show-operator run, which `move_subpath` has
    /// nothing to translate, so the canvas declines and traces rather than
    /// borrowing the Object rung's verb — the same rule, and the same reason,
    /// as `SelectionState::deletable_objects_on`'s rung guard.
    MoveSubpath {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The subpath, in decomposition order.
        subpath: usize,
        /// Horizontal displacement, PDF user-space points.
        dx: f64,
        /// Vertical displacement, PDF user-space points (Y is up).
        dy: f64,
    },
    /// **Displace ONE LINE of a text object** —
    /// `EditSession::move_text_run`, the Part rung's move verb for text, and
    /// `OPERATOR_REQUESTS.md` O188's move half.
    ///
    /// # What the operator asked for, and why it took a month
    ///
    ///
    /// Those labels are **pdf dimensions** (R8b Rule 15) — page content
    /// a CAD exporter wrote. This moves them; it does not re-measure them, and
    /// they have nothing to do with the **ce dimensions** pdfcer authors.
    ///
    /// # The refusal that is asked BEFORE the press, and where
    ///
    /// 9.4.2 again, and the mirror image of [`Self::DeleteTextLine`]'s: a run
    /// with no positioning operator of its own starts wherever the previous one
    /// ended, so there is no operand to rewrite; and a run whose SUCCESSOR is
    /// in that state cannot move without dragging the successor along.
    /// `crate::canvas::moving::eligible` asks
    /// `ObjectModelProvider::text_line_move_refusal_of` before a ghost is drawn,
    /// and **that is the engine's own guard rather than a copy of it** —
    /// `pdfcer_core::vector::edit::text_run_move_refusal`, the function
    /// `plan_move_text_run` runs first. Compare the note on
    /// [`Self::DeleteTextLine`], whose pre-check IS a hand-rolled copy because
    /// the delete side has no exported twin yet.
    ///
    /// # Disclosures — and this one is NOT always empty
    ///
    /// The planner rewrites `Tm` or `Td` operands where it can. Where the run
    /// was placed by `TD`, `T*`, `'`, `"` or by nothing at all it **inserts a
    /// `Td`**, and discloses that it did: the page looks identical, the move is
    /// exact, and dragging back by the same amount will not restore the
    /// original bytes. Rule 4 in its purest form — an inference the operator
    /// cannot see, so it is reported off-canvas and nothing is drawn
    /// differently. The funnel records them; this variant needs no code for it.
    MoveTextLine {
        /// The 0-based page.
        page: usize,
        /// The enclosing text object, by paint-order index.
        object: usize,
        /// The visual line, in the numbering
        /// `ObjectModelProvider::text_line_count` produces and
        /// [`Self::DeleteTextLine`] already uses. **Nothing renumbers**: the
        /// move family rewrites operands in place, so the selection survives
        /// the drag naming the same line. The apply arm hands the line's runs
        /// to `EditSession::move_text_runs` as one set.
        line: usize,
        /// Horizontal displacement, PDF user-space points.
        dx: f64,
        /// Vertical displacement, PDF user-space points (Y is up).
        dy: f64,
    },
    /// **Displace one line of a text object INSIDE a form XObject** —
    /// `EditSession::move_text_runs_in_form`.
    ///
    /// **This is the variant O188 is actually about.** On the operator's
    /// SolidWorks sets the title block *is* a form XObject, drawn once per
    /// sheet; the labels he wants to nudge live inside it. A page-scoped verb
    /// alone would have answered his request everywhere except where he asked
    /// it, which the engine said in as many words when it shipped the pair
    /// together.
    ///
    /// ⚠ **One call changes every sheet the form is drawn on**, because the
    /// form's stream is shared. `FormSurgeryOutcome::invocations` and `::pages`
    /// are the measured pair that says how many, and the engine folds the reach
    /// sentence into `disclosures` when the count is above one — so routing
    /// the disclosures through the funnel, as every `*_in_form` arm here does,
    /// IS how the operator is told. There is no second mechanism and there must
    /// not be one.
    MoveTextLineInForm {
        /// The 0-based page.
        page: usize,
        /// The enclosing text object, by **leaf** index — a different address
        /// space from [`Self::MoveTextLine`]'s `object`, which is why it is a
        /// separate variant rather than a flag.
        leaf: usize,
        /// The visual line, numbered as
        /// `ObjectModelProvider::text_line_count_of` counts for that leaf.
        line: usize,
        /// Horizontal displacement, PDF user-space points.
        dx: f64,
        /// Vertical displacement, PDF user-space points (Y is up).
        dy: f64,
    },
    /// **Displace SEVERAL chunks of one text object by one drag** —
    /// `EditSession::move_text_runs` over every run of every named line: one
    /// command, one press of Undo.
    ///
    /// A set is planned whole, so a run positioned relative to its
    /// predecessor moves with it when both are in the set.
    ///
    /// # The refusal is asked of the whole set, before the ghost
    ///
    /// `crate::canvas::moving::run_move` asks the engine's set guard,
    /// `text_run_move_refusal_of_set`, and refuses the drag whole if it
    /// declines, so this arm cannot be reached holding a set the planner will
    /// decline.
    /// Moving the movable ones and leaving the rest would read as a rendering
    /// fault rather than as a refusal.
    MoveTextLines {
        /// The 0-based page.
        page: usize,
        /// The enclosing text object, by paint-order index.
        object: usize,
        /// The visual lines, ascending and unique, never fewer than two.
        lines: Vec<usize>,
        /// Horizontal displacement, PDF user-space points.
        dx: f64,
        /// Vertical displacement, PDF user-space points (Y is up).
        dy: f64,
    },
    /// [`Self::MoveTextLines`] for a text object **inside a form XObject**.
    ///
    /// ⚠ One call changes every sheet the form is drawn on, for
    /// [`Self::MoveTextLineInForm`]'s reason: the form's stream is shared, and
    /// the engine's own reach sentence is what tells the operator so.
    MoveTextLinesInForm {
        /// The 0-based page.
        page: usize,
        /// The enclosing text object, by **leaf** index.
        leaf: usize,
        /// The visual lines, ascending and unique, never fewer than two.
        lines: Vec<usize>,
        /// Horizontal displacement, PDF user-space points.
        dx: f64,
        /// Vertical displacement, PDF user-space points (Y is up).
        dy: f64,
    },
    /// Drag **one anchor** of one path object to an absolute page-space point
    /// — the Node rung's move verb.
    ///
    /// # Why a destination and not a displacement
    ///
    /// Because that is `EditSession::move_node`'s signature, and the signature
    /// is right: the operand being rewritten *is* a coordinate pair, and the
    /// planner maps the destination through the object's CTM affine inverse in
    /// one step. Expressing it as a delta would make the planner reconstruct
    /// the point it was given, in a space the caller would then have had to
    /// name. The canvas computes it as *"where the anchor is now, plus the
    /// drag"*, and refuses the move outright if the decomposition can no
    /// longer say where the anchor is — see
    /// `pdfcer_gui::canvas::moving::Refusal::NodeNotFound`.
    ///
    /// `node` is **object-scoped**: the space `vector::anchor_count` reports
    /// and `pdfcer node-move --node N` addresses. A second numbering would
    /// make the number pdfcer shows disagree with the number the operator can
    /// act on.
    MoveNode {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The anchor, object-scoped.
        node: usize,
        /// Where the anchor ends up, in PDF user space.
        to: Point,
    },
    /// **Move many of one object's nodes at once** — what a RESIZE is, in
    /// the absence of a scale verb.
    ///
    ///
    /// So a resize is expressed as what it *is* — every node of the path moved
    /// to `anchor + (p - anchor) * (sx, sy)` — and `EditSession::move_nodes`
    /// takes a slice, which makes the whole gesture **one command and one undo
    /// entry**. A per-node loop would be neither: N undo entries for one drag,
    /// and each move planned against byte offsets the previous one invalidated.
    ///
    /// The geometry is computed by `crate::canvas::resizing`, which is pure and
    /// tested; this variant carries the result and nothing else. That is the
    /// funnel's rule and it matters more here than usual — an action that
    /// carried a grip and two factors would put the arithmetic in `apply`,
    /// where it could not be tested without a document.
    /// **Drag one Bézier handle** — move a control point of `node`, leaving the
    /// on-curve anchor itself exactly where it is.
    ///
    /// # Why this is a separate verb and not "move a node that happens to be
    /// a control point"
    ///
    /// Because the two change different things about the path, and the engine
    /// draws the distinction in the type. `move_node` moves a point the curve
    /// passes **through**; this moves a point that governs the curve's
    /// **shape** and that the curve never touches. A single "move a point" verb
    /// would have to infer which the operator meant from what they grabbed,
    /// which is exactly the inference `pdfcer-core`'s own `Handle` type exists
    /// to remove.
    ///
    /// # The disclosure it owes, and it is not the obvious one
    ///
    /// `EditSession::move_handle` returns a list of sentences that is **empty
    /// unless a `v`/`y` segment had to be re-spelled as `c`**. Table 59 gives a
    /// cubic three spellings and two of them omit a control point by making it
    /// equal to a point the segment already has; a handle that must hold its
    /// own value cannot be expressed in those, so the operator's drag rewrites
    /// the operator.
    ///
    /// The curve draws **identically**. Nothing on the page changes. What
    /// changes is that the original bytes are gone and dragging back does not
    /// restore them — which is precisely the class of thing rule 4's surviving
    /// half is about: *an inference the operator cannot see still owes an
    /// off-canvas report*. The apply arm forwards those sentences to the
    /// disclosure channel for that reason, and for no other.
    MoveHandle {
        /// The 0-based page.
        page: usize,
        /// The object whose handle moves, by paint-order index.
        object: usize,
        /// The anchor the handle belongs to, object-scoped.
        node: usize,
        /// Which side of the anchor — arriving or leaving.
        ///
        /// The engine's own enum rather than a `bool`, because "incoming" and
        /// "outgoing" have no natural true/false and a caller that got the
        /// polarity backwards would drag the neighbouring curve instead, which
        /// looks like a coordinate bug rather than an inverted flag.
        handle: Handle,
        /// Where the control point ends up, in PDF user space.
        to: Point,
    },
    MoveNodes {
        /// The 0-based page.
        page: usize,
        /// The object whose nodes move, by paint-order index.
        object: usize,
        /// Every node's new position, object-scoped, in PDF user space.
        ///
        /// Absolute destinations rather than displacements, matching
        /// [`Self::MoveNode`] and for the same reason its docs give: the
        /// operand the planner rewrites is a coordinate pair, so "where the
        /// point ends up" is what lets it map one point through the object's
        /// CTM inverse instead of decomposing a translation.
        moves: Vec<(usize, Point)>,
    },
    /// **Move, resize or rotate any objects at all** — `Pass 113.0`,
    /// 2026-08-20, and the verb this shell had been waiting for since the eight
    /// resize grips were drawn at S4.
    ///
    /// # What it closes
    ///
    /// The operator, three times, escalating:
    ///
    /// > *"there was no way to reposition, resize, or rotate it on the screen.
    /// > Can I please please please have that too?"*
    /// > *"can I please please please have the capability to move the text
    /// > after?"*
    ///
    /// [`Self::MoveNodes`] and `move_objects` cannot answer either. They rewrite
    /// numeric **operands**, and a text run and an image carry no coordinate
    /// operands at all — which is why `move_objects` is path-only by name.
    ///
    /// # The one thing that must not be got wrong: the matrix is PAGE space
    ///
    /// `cm` composes into the CTM in force at that point in the stream — the
    /// object's **user** space, not the page's. The engine emits
    /// `X = CTM × M × CTM⁻¹` per object, from *that object's own* captured CTM,
    /// so a selection spanning two local spaces gets two different `cm`
    /// operands for one gesture and both land where the operator pointed.
    ///
    /// **This shell passes page space and nothing else.** There is no
    /// local-space variant and no flag. Had the engine emitted a caller's matrix
    /// directly it would have been right only where an object's CTM happens to
    /// be the identity and **silently wrong at every scale or slant the producer
    /// left in force** — the object landing twice as far as the pointer went,
    /// with nothing erroring.
    ///
    /// # Why it takes a SLICE, and why that retired a refusal
    ///
    /// One gesture is one command and one undo entry — this project's standing
    /// rule. `canvas::resizing` used to decline a multi-object resize by name
    /// (*"pdfcer resizes one shape at a time"*), because `move_nodes` is
    /// per-object and N objects would have been N commands. That refusal is
    /// **gone**: the transform takes every index at once and scales them all
    /// about one pivot, which is what every drawing application does.
    ///
    /// # What the engine collapses, and why the count is not ours
    ///
    ///
    /// > *"can you get cut copy and paste working for objects I select on the
    /// > canvas?"* — asked in the first week and repeatedly since.
    ///
    /// # Why the clip travels as BYTES
    ///
    /// Because that is what the shell is holding: `canvas::clipboard` parks an
    /// `ObjectClip::to_bytes` payload in `egui::Memory` so that the same
    /// representation serves the in-process clipboard and the OS one. See
    /// `Clipped::Selection` for the three reasons, and for why the third decides
    /// it.
    ///
    /// The deserialisation therefore happens here, in the apply arm, and its
    /// refusals are the engine's own — `ClipError::NotAClip` is checked **before
    /// any length prefix is read**, so an unrelated payload the OS clipboard
    /// hands back is refused with a sentence rather than with whatever a length
    /// prefix read out of the wrong bytes.
    ///
    /// # `at` is a PAGE-SPACE matrix, exactly as [`Self::TransformObjects`]
    ///
    /// `Matrix::IDENTITY` is paste-in-place, `translate` is paste-with-offset,
    /// and `Matrix::about` gives paste-scaled and paste-rotated from the same
    /// verb. That is why the request asked for a matrix rather than a
    /// displacement: a future *paste special* is already built.
    PasteObjects {
        /// The 0-based page to paste onto.
        page: usize,
        /// `ObjectClip::to_bytes` — magic-prefixed, versioned, bit-exact.
        clip: Vec<u8>,
        /// Where it lands, **in PAGE space**.
        at: Matrix,
    },
    TransformObjects {
        /// The 0-based page.
        page: usize,
        /// Which objects, by paint-order index. Every kind is accepted — path,
        /// text, image, form XObject, inline image, in any mixture.
        objects: Vec<usize>,
        /// The transform, **in PAGE space**. See the variant's docs.
        matrix: Matrix,
    },
    /// Join consecutive text runs of one page text object into one run
    /// (`EditSession::merge_text_runs`, default `MergeOptions`). One undo entry,
    /// `CommandKind::MergeTextRuns`; later runs renumber down by `runs.len() - 1`.
    MergeTextRuns {
        /// The 0-based page.
        page: usize,
        /// The page object index of the text object.
        object: usize,
        /// Ascending, consecutive run indices; at least two.
        runs: Vec<usize>,
    },
}
/// The verbs whose subject is a form XObject — a drawing invoked by a page,
/// possibly by many pages, possibly several times by one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XObjectAction {
    /// **Give this page its own private copy of a shared drawing.**
    ///
    /// `EditSession::unshare_form`. Raised by `crate::app::dispatch::format`'s
    /// `format.unshare_form` arm — from the Format contextual tab and from the
    /// canvas context menu — and by nothing else.
    ///
    /// # Why the operator needs this, concretely
    ///
    /// ISO 32000-1 §8.10.1 names a CAD system's standard component as the
    /// *purpose* of form XObjects, and this operator's drawing sets are exactly
    /// that: one title block, one stream object, invoked from thirty-six
    /// sheets. Since `pdfcer-core` `Pass 119.0` this shell can **edit text
    /// inside a form**, which means an operator fixing a typo on sheet 12
    /// changes all thirty-six — and pdfcer cannot prevent that structurally,
    /// because there is exactly one stream object to write.
    ///
    /// `pdfcer-core`'s decision 076 ruled that edit-in-place-and-disclose is the
    /// **default**, and `R206` requires that two defensible behaviours ship as
    /// two options. This variant is the second option. Until it existed the
    /// operator had the default and no choice at all, which is the state `R206`
    /// exists to prevent.
    ///
    /// # Both fields are load-bearing and neither is redundant
    ///
    /// `page` is **not** merely for the trace, unlike
    /// `super::annot::AnnotAction::Delete`'s. The verb's signature is
    /// `(page_index, form)` and the page is half the operand: unsharing is
    /// defined as *"re-point **this page's** references"*, and the same form on
    /// a different page is a different, equally valid call that this one must
    /// not perform.
    ///
    /// `form` is the **outermost** enclosing form's `ObjId`, resolved before the
    /// action was raised. See the module header for why that resolution is not
    /// done here and why the innermost form would be refused.
    ///
    /// # `Copy`, which its neighbours are not
    ///
    /// Both fields are `Copy` — a `usize` and an `ObjId` — so the whole enum is,
    /// and `Action` is not made heavier by carrying it. `super::annot` and
    /// `super::bookmarks` are not `Copy` because they carry `String`s and
    /// `Vec`s; nothing here needs one, and nothing here should grow one: a
    /// second copy of a name the document already holds is how a stale operand
    /// gets written back.
    Unshare {
        /// The 0-based page whose references move. Half the operand, not a
        /// trace field.
        page: usize,
        /// The **outermost** enclosing form's stream object.
        form: ObjId,
    },
}
/// **Which attachment**, addressed the only two ways a PDF makes possible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachmentRef {
    /// An entry in the catalogue's `/Names /EmbeddedFiles` name tree, by its
    /// raw key bytes.
    DocumentLevel {
        /// The key, verbatim. §7.9.6 requires keys to be *"compared for
        /// equality on a simple byte-by-byte basis"*, which is what makes
        /// carrying the bytes both necessary and sufficient.
        key: Vec<u8>,
    },
    /// A `/FileAttachment` annotation (§12.5.6.15), by the annotation's own
    /// object id.
    ///
    /// Only constructible when the listing reported one — `Attachment`'s
    /// `annot_id` is an `Option`, `None` when the `/Annots` entry was a direct
    /// dictionary rather than a reference. The panel offers no control for a
    /// row it cannot address, which is R9 rather than caution: a Save button
    /// that could not name its operand would be an affordance for something
    /// that cannot work.
    PageAnnotation {
        /// The annotation object.
        annot: ObjId,
    },
}
