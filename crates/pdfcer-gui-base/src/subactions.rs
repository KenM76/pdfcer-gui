//! # `subactions` — the per-domain verbs an Action carries, as data
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/subactions.md`.

use pdfcer_core::object::ObjId;

mod vector;
pub use vector::VectorAction;

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
    /// Bates-number the picked sheets in document order, as one undo step.
    /// `stamp.pages` is the operand set; `first` is the first number.
    StampBates {
        /// The label format, position and pages.
        stamp: pdfcer_core::bates::BatesStamp,
        /// The number on the first stamped page.
        first: u64,
    },
    /// Remove the Bates labels pdfcer stamped, as one undo step. `None` is
    /// every page.
    RemoveBates {
        /// 0-based page indices, ascending and unique; `None` for all.
        pages: Option<Vec<usize>>,
    },
    /// Set or remove the picked sheets' visible area, as one undo step.
    SetCropBox {
        /// 0-based page indices, ascending and unique.
        pages: Vec<usize>,
        /// The rectangle in page space, or the whole sheet.
        edit: pdfcer_core::edit::CropBoxEdit,
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

/// One operator preference, carried from the surface that changed it to the
/// file that remembers it.
///
/// See the module header for the four properties every member shares and for
/// why the *live* half is already applied by the time one of these is raised.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefAction {
    /// **Persist the Find bar's *Zoom* control** — `OPERATOR_REQUESTS.md`
    /// **O163**.
    ///
    /// # Why this is not a `pdfcer_gui::find::FindRequest`
    ///
    /// `FindRequest`'s own doc states the rule its two variants share: *what
    /// has to go through the funnel is what needs the **document***, and both
    /// of them do. This needs the opposite — property 1 in the module header.
    /// `Action::Find`'s arm is inside a `Status::Open(doc)` match that would
    /// silently drop it, which is precisely the failure that property
    /// describes.
    FindZoom(bool),
    /// **Persist the Pages panel's previews tick and its time limit** —
    /// `OPERATOR_REQUESTS.md` **O187**: *"the draw page previews
    /// timeout needs to be remembered, and setting it to 0 should set it to
    /// infinity (never time out)"*.
    ///
    /// # Why one variant carries both, when they are two controls
    ///
    /// Because they are one **decision surface** — a checkbox and the box
    /// beside it — and `Prefs::save` is a whole-file write. Two variants would
    /// mean two file writes for the gesture *"turn previews off and set a
    /// limit for when I turn them back on"*, which is the sequence
    /// `crate::panels::pages::previews` explicitly designs for. Each raiser
    /// reads the value it did not change straight out of the cache in the same
    /// frame, so the module header's carry-never-re-read rule still holds for
    /// both halves.
    ///
    /// # It would survive inside the document guard today, and is above it
    /// anyway
    ///
    /// The Pages panel is only drawn with a document open, so unlike
    /// [`Self::FindZoom`] this one has no live counter-example. It is a
    /// preference regardless, because *which surface happens to raise a
    /// preference* is not a property of the preference — and a rule that held
    /// only while the panel needed a document is a rule waiting to be broken
    /// by a Settings entry for the same two values.
    PagePreviews {
        /// The tick, as it now stands.
        on: bool,
        /// The limit in milliseconds, as it now stands — **`0` meaning no
        /// limit**, the operator's own notation. Converted at exactly one
        /// place, `pdfcer_gui::panels::pages::thumbnails::budget_from_millis`.
        budget_ms: u64,
    },
}
/// **Where an Arrange command puts the selected mark.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrangeTo {
    /// Last in `/Annots`, so it is painted over everything else.
    ///
    /// **Last, not first.** §12.5.6 paints annotations in array order, so the
    /// *end* of the array is the top of the stack — the opposite of what "front"
    /// suggests to anyone thinking of a list. Getting this backwards is a defect
    /// that looks correct in every review and is obvious the first time a mark
    /// is arranged, which is why
    /// `pdfcer_gui::app::actions::reorder::tests::front_is_the_end_of_the_array` exists.
    Front,
    /// One place later — over the next thing it currently sits under.
    Forward,
    /// One place earlier — under the next thing it currently sits over.
    Backward,
    /// First in `/Annots`, so everything else is painted over it.
    Back,
}
impl ArrangeTo {
    /// Whether this end of the pair is the **front** — which is what
    /// [`crate::text::arrange::already_there`] needs in order to say which
    /// command the operator pressed.
    #[must_use]
    pub const fn toward_front(self) -> bool {
        matches!(self, Self::Front | Self::Forward)
    }
}
/// **Everything a refused canvas gesture is allowed to put on the status bar.**
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanvasDecline {
    /// A part or a node was entered inside a form XObject whose kind is not a
    /// path, so no geometry verb applies.
    ///
    /// `pdfcer_gui::app::status::decline::canvas::record_canvas` maps this to
    /// [`crate::text::status::InsideFormRefusal::NotAPath`] as a **constant**,
    /// because the canvas only ever meets that one of the two arms. The other,
    /// `NoContainingForm`, is written directly by `app::dispatch::format` from a
    /// place that has established a fact the canvas cannot.
    InsideFormNotAPath,
    /// **A drag on one line whose position this file does not state** —
    /// O188.
    ///
    /// `pdfcer_gui::app::status::decline::Declined::TextRunHasNoPositionOfItsOwn` carries the argument
    /// for why this refusal earns a sentence when most of its siblings in
    /// `canvas::moving::Refusal` do not.
    ///
    TextRunHasNoPositionOfItsOwn,
    /// **A drag on a line that the NEXT line's position is measured from**
    /// — O188.
    ///
    /// Twin of [`Self::TextRunHasNoPositionOfItsOwn`], and the one that reports
    /// a consequence rather than an absence: the move is possible and pdfcer is
    /// declining it, because it would carry a line the operator never selected.
    TextRunWouldDragTheNextLine,
}
impl CanvasDecline {
    /// The stable identifier this decline is **traced** under.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            // ui-text-exempt: stable diagnostic token, never displayed.
            Self::InsideFormNotAPath => "inside-form-not-a-path",
            // ui-text-exempt: stable diagnostic token, never displayed.
            Self::TextRunHasNoPositionOfItsOwn => "text-run-no-position-of-its-own",
            // ui-text-exempt: stable diagnostic token, never displayed.
            Self::TextRunWouldDragTheNextLine => "text-run-would-drag-next-line",
        }
    }
}
