//! # `app::actions::bookmarks` — the three verbs whose subject is one entry in
//! the document's outline
//!
//! Split out of [`super::action`] under **R2** on 2026-08-28, the day the
//! family grew from one verb to three. `super`'s own declaration of `action`
//! wrote the rule down in advance — *"the next family of variants to **grow**
//! is the one that will have to become a sub-enum beside `PageAction` and
//! `DimensionAction`"* — and named markup as the measured candidate on the
//! grounds that markup was the largest. Markup did not grow that week;
//! bookmarks did, from `AddBookmark` alone to add, rename and delete, when
//! `pdfcer-core` `Pass 156.0` shipped `set_outline_title` and
//! `delete_outline_item`. The measurement stands and is still the answer the
//! day markup grows; the rule was about growth, and this is what grew.
//!
//! ## What makes these a family rather than a size-driven cut
//!
//! Every variant here **addresses its operand by `ObjId`** and by nothing
//! else, and that is a property no other family in the enum has for the same
//! reason. The reason is the one the engine reported from its own CLI:
//!
//! > *"the indices shift after every add … I got this wrong myself while
//! > driving the command and nested something two levels deeper than intended,
//! > and the output looked entirely plausible."*
//!
//! An outline is a tree that every edit to it renumbers. A position in the
//! walk — "the fourth row", "the second child of the first" — names a
//! different bookmark after any add, any delete, and any undo of either.
//! `OutlineItem::id` exists precisely so a GUI does not have to hold one, and
//! its own doc comment says so: *"identity is what a GUI needs and the tree
//! cannot otherwise supply."*
//!
//! ⇒ So the shared property is not *"they are all about bookmarks"*, which
//! would be a subject label. It is that **all three are resolvable after the
//! frame that raised them**, which is the one thing the action funnel requires
//! of an operand and the one thing a tree position cannot promise.
//!
//! ## ★★ `/Count` is two different quantities, and the sign carries open/closed
//!
//! §12.3.3 is where implementations of this feature go wrong, and the engine
//! sent the table to this shell unprompted because it expected us to build a
//! panel:
//!
//! | | root `/Outlines` (Table 152) | an item (Table 153) |
//! |---|---|---|
//! | counts | all visible items **including** the top level | visible **descendants**, excluding itself |
//! | sign | **cannot** be negative | **positive = open, negative = closed** |
//!
//! A **closed** item contributes exactly **1** to its ancestors' counts,
//! however large its subtree is. Three consequences, and this module is built
//! around all three:
//!
//! 1. **Nothing here diffs a count to describe an edit.** Adding a bookmark
//!    under a collapsed ancestor leaves the document's total unchanged, so a
//!    surface reporting *"added N"* from a root-count diff reports **zero for
//!    a correct save**. [`BookmarkAction::Add`] adds one bookmark and the panel
//!    says one bookmark; there is no number to get wrong.
//! 2. **A delete's count comes from the engine, not from the tree we drew.**
//!    See [`BookmarkAction::Delete`].
//! 3. **`open` is the only reason a disclosure about visibility can be
//!    written at all.** `pdfcer_core::outline::OutlineItem::open` is the shell's
//!    read of that sign, and §12.3.3 defines no `/Open` key, so the sign is the
//!    only carrier there is.
//!
//! ## ★ Reorder and re-parent are here now, and this paragraph used to say
//! they were not
//!
//! It read: *"**Reorder and re-parent.** The engine's note of 2026-08-28 lists
//! them as not shipped … so there is no variant for either and no drag handle
//! in the panel. R9: a capability that does not exist renders nothing."* That
//! was correct for one day. `pdfcer-core` `Pass 161.0` shipped
//! `move_outline_item` and `set_outline_open`, and [`BookmarkAction::Move`] and
//! [`BookmarkAction::SetOpen`] are the surface for them.
//!
//! The sentence is kept rather than deleted because it is the record of the
//! rule being applied correctly: nothing was greyed, nothing was drawn as a
//! promise, and the day the engine could honour the gesture the panel grew it.
//! That is R9 working, not R9 being overtaken.
//!
//! **What is still deliberately absent:** a verb that deletes the whole
//! outline. `EditError::OutlineRootIsNotAnItem` refuses the root by name,
//! because deleting it is *"a different act that gets its own verb when it is
//! wanted"*, and this shell does not want it yet.
//!
//! ## ★★ The family's shared property survived the growth, and that is the
//! test of whether the cut was right
//!
//! Both new variants address their operand by `ObjId` — and
//! [`BookmarkAction::Move`] addresses its **destination** that way too, because
//! `OutlinePlacement` is built from anchors rather than positions for the
//! reason its own doc comment gives about this exact surface: *"A shell that
//! reads a panel, lets the operator drag a row, and then calls with the index
//! it read has a race with its own undo stack."*

use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;

/// The verbs whose subject is one entry in the document's outline.
///
/// See the module header for what makes them a family: every one of them names
/// its operand by `ObjId`, because an outline is a tree that every edit to it
/// renumbers.
/// ★ `Eq` was dropped on 2026-08-29 when [`BookmarkAction::Paste`] arrived.
///
/// `pdfcer_core::outline::OutlineClip` derives `PartialEq` and not `Eq`, because
/// a bookmark's colour is three `f64`s and floats have no total equality. That
/// is the engine's correct choice and it propagates: an enum holding one cannot
/// be `Eq` either.
///
/// Nothing depended on it — `Eq` over `PartialEq` buys a `HashMap` key and no
/// action is ever one — but it is recorded rather than silently removed,
/// because a dropped trait bound is exactly the kind of change a diff makes
/// look deliberate and a reader cannot date.
#[derive(Debug, Clone, PartialEq)]
pub enum BookmarkAction {
    /// ★ **Add a bookmark to the document's outline.**
    ///
    /// Raised by `crate::panels::bookmarks::add` and by nothing else.
    ///
    /// # ★ Why nothing here counts anything
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
    /// ★ **Rename a bookmark** — write a new `/Title` onto one outline item.
    ///
    /// Raised by `crate::panels::bookmarks::edit` and by nothing else.
    /// `pdfcer-core` `Pass 156.0`; the engine's covering note calls it *"the
    /// commonest bookmark edit there is"*, which is why it is the verb the
    /// panel puts first once a row is selected.
    ///
    /// # ★ The verb with no structural risk, and saying so is load-bearing
    ///
    /// `set_outline_title`'s own doc comment is unusually reassuring, and the
    /// reassurance is a fact a reader of *this* file needs:
    ///
    /// > *"a title is a text string (§7.9.2) on one dictionary, and nothing in
    /// > the `/First`/`/Last`/`/Next`/`/Prev`/`/Count` machinery depends on
    /// > it."*
    ///
    /// ⇒ **A rename cannot move, orphan, hide or renumber anything.** That is
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
    /// path every other text string uses"*, because `Pass 150.0` shipped a
    /// defect from two paths disagreeing about PDFDocEncoding. So an em dash or
    /// an accented name in this `String` needs nothing from this crate.
    Rename {
        /// The outline item whose `/Title` is being replaced.
        item: ObjId,
        /// The new title. Trimmed and non-empty by the time it gets here — a
        /// bookmark with a blank title is legal and is an invisible row, which
        /// is the same defect as no row.
        title: String,
    },
    /// ★★★ **Delete a bookmark AND everything under it.**
    ///
    /// Raised by `crate::panels::bookmarks::edit` and by nothing else.
    /// `pdfcer-core` `Pass 156.0`.
    ///
    /// # ★★ The subtree goes too, and that is a decision with a reason
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
    /// ⇒ This is therefore a verb whose blast radius is **larger than the thing
    /// the operator clicked**, and the whole of the UI obligation follows from
    /// that one sentence. It is stated before the press by
    /// `crate::panels::bookmarks::edit`, from the tree the panel already drew,
    /// and it is stated again after the press from the engine's own count. See
    /// [`delete`] for why the answer is given twice and why the two numbers are
    /// allowed to differ.
    ///
    /// # Why there is no confirmation dialog, and it IS a choice
    ///
    /// `HANDOFF.md`'s rule is *confirmed or clearly undoable*, and this is the
    /// second. One press produces **one** `EditSession` command, so one
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
    /// ★★★ **Move a bookmark — reorder it among its siblings, or re-parent it
    /// under a different one — carrying its whole subtree.**
    ///
    /// Raised by `crate::panels::bookmarks::reorder` and by nothing else.
    /// `pdfcer-core` `Pass 161.0`, the half of bookmark editing this shell
    /// shipped without: `Pass 156.0` gave it rename and delete, and the
    /// engine's covering note is blunt about what was still missing —
    ///
    /// > *"an outline in the wrong **order** could only be fixed by deleting a
    /// > branch and re-authoring it, which loses every destination, colour and
    /// > style on it and is not an edit any operator would call a
    /// > reorganisation."*
    ///
    /// # ★★ The subtree travels, and the destination does not move
    ///
    /// `move_outline_item`'s own words: *"A chapter dragged under a different
    /// part takes its sections with it."* That matches
    /// [`Self::Delete`]'s subtree semantics and Acrobat's model — its
    /// `PDBookmark` unlink/add-child pair operates on the node, which owns its
    /// children wherever `/Parent` points, and there is no API path that leaves
    /// them behind.
    ///
    /// ⇒ So this verb, like the delete, has a **blast radius larger than the
    /// row the operator clicked** — and unlike the delete, the size of it is
    /// reported by the engine rather than counted by the panel. See [`move_to`]
    /// for the two numbers and why both are needed.
    ///
    /// # ★★★ Why the placement is an anchor and NEVER an index
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
    /// # ★ The expansion of the destination is NOT folded in here
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
    /// ★★★ **Put a copied bookmark subtree into this document's outline.**
    ///
    /// `OPERATOR_REQUESTS.md` **O59** item 3. Raised by
    /// `panels::bookmarks::clip::paste_row` and by nothing else.
    ///
    /// ★★ **Acrobat cannot do this between two files at all**, by Adobe's own
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
    /// ★ The panel warns about this **before** the press as well, from
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
    /// ★★ **Expand or collapse a bookmark** — flip the sign on its `/Count`.
    ///
    /// Raised by `crate::panels::bookmarks::reorder`'s disclosure triangle and
    /// by nothing else. `pdfcer-core` `Pass 161.0`.
    ///
    /// # ★★★ This is a document edit, and every other program makes it a view
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
    /// # ★★ The magnitude is the engine's problem, and getting it wrong is
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

/// Apply one bookmark verb.
///
/// The dispatch half of this module, reached from `PdfcerApp::apply`'s single
/// [`super::action::Action::Bookmark`] arm. It is a free function taking
/// `&mut OpenDoc` rather than a method, exactly like [`super::dimensions::apply`]
/// and [`super::pages::apply`], because the caller is the one place that owns
/// the borrow and the arm should be one line.
///
/// **Every arm goes through [`super::apply::vector_edit`]** — the
/// cancel–mutate–bump–invalidate protocol — and none of them may hand-roll it.
/// Its doc comment carries the argument: four hand-written copies of a
/// four-step protocol are four chances to omit a step, and the two steps most
/// easily omitted (the epoch bump and the structural resync) fail *silently*,
/// leaving an edit that happened in the document and did not happen on screen.
///
/// ★ The `page` argument passed to `vector_edit` is **`0` for all three**, and
/// that is honest rather than lazy: an outline is document-level, no page is
/// being edited, and the parameter exists only so the diagnostic trace can say
/// which sheet a geometry edit touched. [`super::dimensions::apply`] passes `0`
/// for its group verbs for the identical reason. The one exception is
/// [`BookmarkAction::Add`], which passes the destination page — not because a
/// page is being changed, but because the page is the operand that decides what
/// the bookmark points at, and a trace that could not say which one would be
/// unable to check the commonest thing to get wrong.
pub(super) fn apply(doc: &mut OpenDoc, action: BookmarkAction) {
    match action {
        // ★ One bookmark, one undo entry, and NO count reported.
        //
        // See the variant: `/Count` is two quantities and its sign is the
        // open/closed flag, so a bookmark added under a collapsed ancestor
        // leaves the document's total unchanged. A disclosure built by diffing
        // it would say "0" for a correct save.
        //
        // The destination is an explicit page at `Fit`, which is the only form
        // `add_outline_item` authors without refusing — named and remote
        // destinations are refused by name, and `DestView::Unknown` is refused
        // because the reader keeps an extension's fit NAME and discards its
        // parameters, so re-emitting it would write a view that is not the one
        // the source had.
        BookmarkAction::Add {
            parent,
            title,
            page,
        } => {
            super::apply::vector_edit(doc, "add-bookmark", page, 1, |session| {
                session
                    .add_outline_item(
                        parent,
                        &title,
                        Some(pdfcer_core::outline::Destination::Page {
                            page_index: page,
                            view: pdfcer_core::outline::DestView::Fit,
                        }),
                    )
                    .map(|_| Vec::new())
            });
        }
        BookmarkAction::Rename { item, title } => rename(doc, item, &title),
        BookmarkAction::Delete { item } => delete(doc, item),
        BookmarkAction::Move { item, to } => move_to(doc, item, to),
        BookmarkAction::Paste { clip, to } => paste(doc, &clip, to),
        BookmarkAction::SetOpen { item, open } => set_open(doc, item, open),
    }
}

/// **Put a copied bookmark subtree into the outline**, as one undoable command.
///
/// `OPERATOR_REQUESTS.md` **O59** item 3.
///
/// # ★★★ The disclosure, and why a zero drops the clause entirely
///
/// `OutlinePasteOutcome::destinations_dropped` counts bookmarks that arrived
/// **without** their destination, because it named a page this document does
/// not have. The engine drops rather than clamps, and that is the right choice:
/// clamping would send the operator to *some* page, confidently and wrongly,
/// where a bookmark that plainly does nothing at least shows what happened.
///
/// A zero says nothing at all. `app::status` has one slot for consequences and
/// a sentence reading *"0 destinations were dropped"* would evict a real one to
/// report an absence — which is `rename`'s argument below, applied to the arm
/// that does have something to say when there is something.
///
/// # ★ It reports what happened; the panel predicted it
///
/// `panels::bookmarks::clip::paste_row` warns before the press, from
/// `OutlineClip::deepest_page()` against the page count. The two are not
/// duplicates and neither is sufficient alone: a prediction is a guess nobody
/// confirmed, and a report arrives too late to choose differently. The operator
/// gets the choice *and* the outcome.
fn paste(
    doc: &mut OpenDoc,
    clip: &pdfcer_core::outline::OutlineClip,
    to: pdfcer_core::edit::OutlinePlacement,
) {
    // ★ Page 0: an outline is a document-level structure reached from the
    // catalogue's `/Outlines` and never from a page, so there is no page this
    // edit is "on". `vector_edit` wants one for its trace and its invalidation;
    // zero is the honest answer and is what `super::bookmarks`' other arms pass.
    super::apply::vector_edit(doc, "paste-bookmark", 0, clip.len(), |session| {
        session.paste_outline_item(clip, to).map(|outcome| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "bookmark-paste-applied items={} dropped={}",
                    outcome.items_pasted, outcome.destinations_dropped
                )
            });
            if outcome.destinations_dropped == 0 {
                Vec::new()
            } else {
                vec![crate::text::panels::bookmark_paste_dropped(
                    outcome.destinations_dropped,
                )]
            }
        })
    });
}

/// **Rename one bookmark**, as one undoable command, disclosing nothing.
///
/// # Why the disclosure list is empty, deliberately
///
/// `vector_edit` surfaces whatever this returns to `app::status`, and the rule
/// that module's header states is that a disclosure is *"the part they cannot
/// see"*. A rename has no such part. `set_outline_title` writes `/Title` on one
/// dictionary and touches nothing else — its own doc comment says *"nothing in
/// the `/First`/`/Last`/`/Next`/`/Prev`/`/Count` machinery depends on it"* — so
/// the entire effect of this call is a row in the panel beside the operator's
/// pointer reading the words they just typed.
///
/// Emitting *"Bookmark renamed."* would put a sentence in the one slot
/// `app::status` has for consequences, describing something with no
/// consequences, and it would evict the previous edit's real disclosure to do
/// it. `super::forms::rename` reports one because a form field's rename
/// **does** have an invisible part: renaming a parent renames its descendants,
/// and the count of them is not on screen. Nothing analogous exists here.
///
/// # What happens on a refusal
///
/// `vector_edit` traces it and leaves the document alone, which is the whole
/// response this shell gives to any engine refusal today. Three are reachable:
/// `DocumentEncrypted`, the certification gate, and `NotADictionary` if the id
/// no longer resolves to an outline item — which is what an id from a stale
/// draft becomes after an undo. The panel does not pre-empt any of them; see
/// `crate::panels::bookmarks::edit`'s header for why the encryption case is
/// not gated at the widget.
fn rename(doc: &mut OpenDoc, item: ObjId, title: &str) {
    super::apply::vector_edit(doc, "rename-bookmark", 0, 1, |session| {
        session.set_outline_title(item, title).map(|()| Vec::new())
    });
}

/// **Delete one bookmark and its whole subtree**, as one undoable command,
/// disclosing how many items went.
///
/// # ★★ The count is the disclosure, and it comes from the engine
///
/// `delete_outline_item` returns `usize` — the number of items actually
/// removed, the clicked one included. That number is the answer to the question
/// this verb raises and cannot answer any other way: **the subtree went too**,
/// and on a collapsed parent the operator could not see how large it was.
///
/// This is `HANDOFF.md`'s *"disclose off-canvas, never on the page"* in its
/// plainest form. The panel already stated the expected size before the press,
/// from the tree it had drawn; this states what the engine actually removed.
///
/// ★ **The two numbers are allowed to differ, and that is the reason both are
/// said.** `read_outline` gives up part-way on a cycle, on excessive depth, or
/// on exhausting its item budget — the panel draws a truncation notice when it
/// does — so the shell's pre-press count is a count of *what pdfcer could
/// read*, and the engine's is a count of *what it removed*. On any ordinary
/// document they agree. On a damaged one the after-the-fact number is the true
/// one, and an operator who saw "3" promised and "47" reported has been told
/// something real about their file rather than being quietly lied to by the
/// only number they were shown.
///
/// # Why one is not spelled as none
///
/// Deleting a leaf removes exactly one item, and *"Bookmark deleted, including
/// its 0 bookmarks beneath it"* is the shape of sentence that makes a program
/// look like it is reading from a template. The catalog branches on the count;
/// see `crate::text::panels::bookmark_deleted`.
///
/// # What it cannot be asked to do
///
/// The **outline root** is refused by name — `EditError::OutlineRootIsNotAnItem`
/// — because the root is not an item, carries no `/Title`, and deleting it means
/// deleting the whole outline, *"a different act that gets its own verb when it
/// is wanted."* The panel cannot raise that refusal: `read_outline` reports the
/// root's *children* as its top-level items, so no `ObjId` the panel can offer
/// is the root's. The refusal is therefore unreachable from this surface rather
/// than routed around, which is the better of the two outcomes and is recorded
/// here so nobody adds a guard for a case that cannot occur.
fn delete(doc: &mut OpenDoc, item: ObjId) {
    super::apply::vector_edit(doc, "delete-bookmark", 0, 1, |session| {
        session
            .delete_outline_item(item)
            .map(|removed| vec![crate::text::panels::bookmark_deleted(removed)])
    });
}

/// **Move one bookmark and its whole subtree**, as one undoable command,
/// disclosing what the operator could not watch.
///
/// # ★★★ Three disclosures, from three different sources, and each is needed
///
/// | Sentence | Source | Answers |
/// |---|---|---|
/// | [`crate::text::panels::bookmarks::bookmark_moved`] | `OutlineMove::visible_items` and `::reparented`, from the **engine** | *how many rows moved, and did it change owner?* |
/// | [`crate::text::panels::bookmarks::bookmark_move_took_hidden`] | the **tree the panel drew**, read before the call | *how many bookmarks were in the branch that nobody could see?* |
/// | [`crate::text::panels::bookmarks::bookmark_move_into_collapsed`] | the tree read **after** the call | *why has the bookmark not appeared where I put it?* |
///
/// The first two are two different quantities and it is §12.3.3 that makes
/// them so. `OutlineMove::visible_items` is *"the item plus its **visible**
/// descendants"*, which for a **collapsed** chapter of forty sections is
/// **one** — Table 153's sign convention gives a closed node a contribution of
/// exactly 1 to its ancestors however large it is. The engine's own doc is
/// explicit that a shell must report that number and must not recompute it:
///
/// > *"A shell can say 'moved 1 bookmark (7 nested)' only if the core tells
/// > it; recomputing it shell-side would be a second implementation of the sign
/// > convention."*
///
/// ⇒ So the engine's number is reported verbatim, and the branch size is a
/// **separate sentence from a separate source**, offered only when the item was
/// collapsed — which is exactly when the two disagree. This is the same posture
/// [`delete`] takes about its own before-and-after counts, with the difference
/// that there the two numbers answer one question and here they answer two.
///
/// # ★★ Why the collapsed-destination check is made AFTER the call
///
/// Because it is a fact about the document the move produced, and only the move
/// knows where the bookmark went. `OutlineMove::to_parent` names it — and it is
/// carried on the report precisely so a shell does not have to work it out —
/// but whether that parent is *open* is a `/Count` sign the panel must read
/// back, because the move itself may have changed it: `move_outline_item`
/// leaves a parent that already had children exactly as the operator set it,
/// and **opens one that was a leaf**.
///
/// ★ The immediate parent is enough, and a walk to the root would be a walk
/// nothing could reach. A drop lands on a row, a row is drawn only when every
/// ancestor above it is open, so a collapsed grandparent implies a destination
/// that was never on screen to be dropped on.
///
/// ★ `to_parent` may be the **outline root**, for a move to the top level.
/// `read_outline` reports the root's *children* as its top-level items and
/// never the root itself, so the lookup answers `None` and no sentence is
/// drawn — which is correct: the top level is always visible.
///
/// # What happens on a refusal
///
/// A sentence, through `app::status::decline`, recorded from **inside** the
/// closure — [`crate::app::status::decline::record_resize_not_rebuildable`]'s
/// placement and its stated reason: whether the engine will refuse is not
/// knowable before the call. `vector_edit`'s `Err` arm traces and, by its own
/// recorded decision, says nothing, and **a refusal must be a sentence, never a
/// silence.**
///
/// The panel forecasts and refuses the one case an operator can act on — a drop
/// into the bookmark's own subtree — before raising this action at all, so what
/// reaches here is the residue: `/Encrypt`, the certification gate, and an id
/// that stopped resolving between the frame and the apply. See
/// [`crate::text::panels::bookmarks::bookmark_move_declined_engine`] for the
/// table and for why a catch-all is honest when none of the causes has a remedy.
fn move_to(doc: &mut OpenDoc, item: ObjId, to: pdfcer_core::edit::OutlinePlacement) {
    super::apply::vector_edit(doc, "move-bookmark", 0, 1, |session| {
        // Read BEFORE the move: after it, the item is somewhere else and its
        // own `/Count` sign has been carried along with it, which is fine — but
        // the tree walk that finds it would be walking the arrangement the
        // operator did not press the button on.
        let before = pdfcer_core::outline::read_outline(&session.view());
        // `None` unless the bookmark was **closed and not empty**, which is the
        // only case where the engine's count and the branch size differ. A leaf
        // and an open branch both need no second sentence: for the leaf there
        // is nothing hidden, and for the open one `visible_items` already
        // counted it.
        let hidden = crate::panels::bookmarks::tree::find(&before.items, item)
            .filter(|found| !found.open)
            .map(crate::panels::bookmarks::tree::descendants)
            .filter(|count| *count > 0);

        let report = match session.move_outline_item(item, to) {
            Ok(report) => report,
            Err(error) => {
                // ★★★ Which sentence, decided here and nowhere else. The panel
                // raises a drop it has already forecast as impossible —
                // deliberately, see `panels::bookmarks::reorder::settle` — so
                // this arm is the one place that tells the operator's own
                // mistake apart from the document's refusal.
                crate::app::status::decline::record_bookmark_move_refused(matches!(
                    error,
                    pdfcer_core::edit::EditError::OutlineMoveIntoOwnSubtree { .. }
                ));
                return Err(error);
            }
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // ★ Every field of the engine's report, plus the shell's own branch
            // size. A line saying only "a move applied" would be identical for
            // a build that reordered where it should have re-parented, or that
            // reported the branch size where it should have reported what moved
            // on screen — and those are the two things a wrong build gets wrong
            // here, because they are two numbers that agree on every ordinary
            // document and part company on exactly the collapsed one.
            format!(
                "bookmark-move-report item={} moved={} reparented={} visible={} hidden={}",
                item.num,
                u8::from(report.moved),
                u8::from(report.reparented),
                report.visible_items,
                hidden.unwrap_or(0),
            )
        });
        if !report.moved {
            // The engine wrote nothing and created no undo entry. The panel
            // dims its caret over a landing it can see is a no-op, so reaching
            // this means the shell's forecast and the engine's answer
            // disagreed — which is worth one line rather than a shrug.
            return Ok(vec![
                crate::text::panels::bookmarks::bookmark_move_no_change().to_owned(),
            ]);
        }
        let mut notes = vec![crate::text::panels::bookmarks::bookmark_moved(
            report.visible_items,
            report.reparented,
        )];
        if let Some(count) = hidden {
            notes.push(crate::text::panels::bookmarks::bookmark_move_took_hidden(
                count,
            ));
        }
        let after = pdfcer_core::outline::read_outline(&session.view());
        if crate::panels::bookmarks::tree::find(&after.items, report.to_parent)
            .is_some_and(|parent| !parent.open)
        {
            notes.push(crate::text::panels::bookmarks::bookmark_move_into_collapsed().to_owned());
        }
        Ok(notes)
    });
}

/// **Expand or collapse one bookmark**, as one undoable command, disclosing
/// nothing.
///
/// # ★★ Why the disclosure list is empty, and it is the same ruling as
/// [`rename`]'s
///
/// `vector_edit` surfaces whatever this returns to `app::status`, and that
/// module's rule is that a disclosure is *"the part they cannot see"*. The
/// whole effect of this verb is rows appearing or disappearing in the panel the
/// operator's pointer is in. There is no invisible part, and emitting
/// *"Bookmark expanded."* would put a sentence in the one slot `app::status`
/// has for consequences — evicting the previous edit's real disclosure — to
/// describe something already on screen.
///
/// ★ The fact that **is** surprising is disclosed, and it is disclosed
/// **before** the press, on the triangle's hover text: this writes into the
/// document. See
/// [`crate::text::panels::bookmarks::bookmark_expand_tooltip`], which carries
/// the argument. A consequence an operator can still decide against belongs in
/// front of the control, not behind it.
///
/// # ★ One label for both directions
///
/// `vector_edit`'s label is a string literal by construction —
/// `tools/gates/check-trace-names.py` reads it out of the call site — so the
/// two directions cannot take two labels without splitting this into two
/// functions that differ by a boolean. They are one verb with one operand and
/// one refusal set; which way it went is a **key** on the panel's own trace
/// line (`bookmark-disclosure open=`), which is where a driven check reads it.
///
/// # What it cannot be asked to do
///
/// A **leaf** — Table 153 makes `/Count` *"required if the item has any
/// descendants"*, so an item without them has no open-or-closed state.
/// `set_outline_open` answers `Ok(false)` rather than refusing, and the panel
/// draws no triangle on one, so this is unreachable from that surface rather
/// than routed around.
///
/// The **outline root** is refused by name (`OutlineRootIsNotAnItem`) because
/// its `/Count` is the *other* quantity — it counts visible items at every
/// level and *"cannot be negative"* — so it has no state to set. As with
/// [`delete`], no `ObjId` the panel can offer is the root's, since
/// `read_outline` reports the root's children as its top-level items.
fn set_open(doc: &mut OpenDoc, item: ObjId, open: bool) {
    super::apply::vector_edit(doc, "set-bookmark-open", 0, 1, |session| {
        session.set_outline_open(item, open).map(|_| Vec::new())
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The three verbs are three distinct values**, so a match on them
    /// cannot silently collapse.
    ///
    /// `PartialEq` is derived and `super::super::tests` compares whole
    /// `Action`s, so a variant that failed to distinguish itself would make
    /// those comparisons pass for the wrong reason — the exact failure mode
    /// the engine reported on this same Pass, where *"all three of our
    /// sabotage checks survived the first test suite"* because the fixtures
    /// could not tell the answers apart.
    #[test]
    fn the_three_verbs_are_distinguishable() {
        let id = ObjId::new(7, 0);
        let add = BookmarkAction::Add {
            parent: Some(id),
            title: "Chapter 3".to_owned(),
            page: 4,
        };
        let rename = BookmarkAction::Rename {
            item: id,
            title: "Chapter 3".to_owned(),
        };
        let delete = BookmarkAction::Delete { item: id };
        assert_ne!(add, rename);
        assert_ne!(rename, delete);
        assert_ne!(add, delete);
    }

    /// ★ **A rename of the same item to two different titles is two different
    /// actions**, and a rename of two different items to the same title is
    /// too.
    ///
    /// Both halves matter and only one of them is obvious. The queue may hold
    /// more than one action from a single frame, and `PdfcerApp::apply` applies
    /// them in order; a variant that compared equal on only one of its two
    /// fields would let a de-duplicating caller — or a test asserting "the
    /// queue holds what I expected" — accept the wrong one.
    ///
    /// The fixture deliberately makes the two answers different in each
    /// direction, which is the discipline the engine's note asks for: *"when
    /// you assert that A and B differ, check your fixture can tell them
    /// apart."*
    #[test]
    fn a_rename_is_identified_by_both_its_item_and_its_title() {
        let a = ObjId::new(7, 0);
        let b = ObjId::new(8, 0);
        let same_item_new_title = (
            BookmarkAction::Rename {
                item: a,
                title: "one".to_owned(),
            },
            BookmarkAction::Rename {
                item: a,
                title: "two".to_owned(),
            },
        );
        assert_ne!(same_item_new_title.0, same_item_new_title.1);

        let same_title_new_item = (
            BookmarkAction::Rename {
                item: a,
                title: "one".to_owned(),
            },
            BookmarkAction::Rename {
                item: b,
                title: "one".to_owned(),
            },
        );
        assert_ne!(same_title_new_item.0, same_title_new_item.1);
    }

    /// ★ **The generation number is part of the identity.**
    ///
    /// `ObjId` is `(num, generation)`, and a delete addressed to `7 0 R` must
    /// not compare equal to one addressed to `7 1 R`. This is cheap to assert
    /// and it pins the thing that would make the whole "address by id, never
    /// by position" argument in the module header hollow: an id that only
    /// half-identifies is a position with extra steps.
    #[test]
    fn an_objid_generation_distinguishes_two_deletes() {
        assert_ne!(
            BookmarkAction::Delete {
                item: ObjId::new(7, 0)
            },
            BookmarkAction::Delete {
                item: ObjId::new(7, 1)
            },
        );
    }
}
