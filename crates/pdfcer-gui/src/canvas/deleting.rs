//! # `canvas::deleting` — **which delete verb the rung the operator is on reaches**
//!
//! The delete twin of [`crate::canvas::moving::eligible`], and it exists for
//! the same reason that one does: a selection ladder with three rungs addresses
//! three different things, `pdfcer-core` has a different verb for each, and the
//! decision about *which* must be made in one pure function that both the
//! keyboard and the ribbon ask — or the key and the command act on different
//! things, which `app::keyboard`'s header calls the defect the single
//! dispatcher exists to make impossible.
//!
//! ## ★★★ What this closes, and why it is one defect three times
//!
//! Until 2026-09-05 Delete acted at the **Object rung only**. Every deeper rung
//! declined:
//!
//! ```text
//! canvas-delete-declined level=Part reason=no-verb-for-rung
//! ```
//!
//! …and that trace line was the whole of the response. Nothing on screen, no
//! sentence, no sound. The operator's own words about the same shape of failure
//! elsewhere: *"nothing happens"*.
//!
//! The refusal was honest when it was written — `SelectionState::deletable_
//! objects_on` carries the argument, and it is a good one: at the Part rung the
//! selection names one subpath or one label *inside* an object, while
//! `delete_objects` removes **whole objects**, and on the measured CAD export
//! one path object holds **1,194 subpaths** and one text object holds **all 237
//! pdf dimension labels**. Borrowing the Object rung's verb would delete a whole
//! drawing view because the operator asked to remove one line of it.
//!
//! What was wrong is that the engine had shipped the three verbs that do the
//! right thing and nothing here called them:
//!
//! | rung | what is selected | verb |
//! |---|---|---|
//! | Part, on a **path** | one subpath | `EditSession::delete_subpath` (Pass 25.2) |
//! | Part, on **text** | one show operator — one label | `EditSession::delete_text_run` (`Pass 32.0`) |
//! | Node | one anchor | `EditSession::delete_node` (Pass 36.1) |
//!
//! Each of the three had its **move** twin wired — `move_subpath` through
//! `VectorAction::MoveSubpath`, `move_node` / `move_nodes` through
//! `VectorAction::MoveNode` / `MoveNodes` — so for a fortnight a line could be
//! entered, selected and **dragged**, and could not be removed.
//!
//! ## ★★ The vocabulary, because getting it wrong here is a documented cost
//!
//! **R8b Rule 15.** The 237 labels on the operator's SolidWorks export are
//! **pdf dimensions** — page content pdfcer reads and must not silently alter.
//! A **ce dimension** is the thing pdfcer itself authors, and this module never
//! touches one: `Action::Dimension` is a different family entirely. Nothing
//! here is ever a bare "dimension".
//!
//! ## Why this is pure, and what it deliberately cannot do
//!
//! No egui, no pointer, no document, no `&mut` anything. It takes the selection
//! and the page's object model and returns either the one verb to raise or the
//! one reason none applies. That is what lets every rule below be a unit test
//! rather than something to be hoped for in a running window — and it is what
//! lets `canvas::keys` and `app::dispatch::format` ask the identical question,
//! which is the property the ribbon's Delete had already been found to have
//! lost once.
//!
//! ## ★ R83: a refusal the model can see BEFORE the press is a refusal that
//! names its remedy
//!
//! One refusal here is asked ahead of the verb rather than reported after it:
//! [`Refusal::RunWouldMoveNext`]. `ObjectModelProvider::text_run_delete_would_
//! move_next` answers from the same `positioned_by` flag
//! `EditSession::delete_text_run` refuses on (§9.4.2 — a following run with no
//! positioning operator of its own starts wherever this one ends, so removing
//! this one would slide it). Both answers are therefore the same answer, and
//! asking first buys the one thing the engine's own refusal cannot deliver to
//! an operator: **the remedy**. `EditError` is `Display` output and
//! `check-ui-strings.sh` exclusion 3 forbids routing it to a surface, so a
//! refusal that arrives from the engine reaches the operator as
//! `text::status::edit_declined_by_engine`, which by design names no cause.
//! Asked here, the sentence can say *delete the later label first*, which is
//! true, always works, and is what they need.
//!
//! Every other refusal below is left to the engine, deliberately: it is the
//! engine's judgement, it is made against the bytes rather than against a
//! model, and `apply::vector_edit`'s `Err` arm already routes it to the decline
//! channel with a worded sentence (`OPERATOR_REQUESTS.md` O116). Duplicating
//! those predicates here would be a second statement of a destructive rule,
//! which is the drift `deletable_objects_on`'s own header refuses.

use crate::canvas::selection::{Selection, SelectionLevel, SelectionState};
use crate::panels::objects::provider::{ObjectModelProvider, PartKind};

/// **The one verb a Delete on this selection reaches.**
///
/// One variant per `EditSession` delete verb this shell can address, and no
/// variant without one — R9, applied to a routing enum: a case that renders
/// nothing must not be representable as though it did something.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteSubject {
    /// Whole objects out of the page's own paint order — `delete_objects`.
    ///
    /// The Object rung, and the only rung that was ever wired. Ascending and
    /// de-duplicated, because the engine resolves **every** index before
    /// planning anything and one stale entry refuses the whole call.
    Objects {
        /// The 0-based page the indices are positions on.
        page: usize,
        /// Paint-order indices, ascending and unique.
        objects: Vec<usize>,
    },
    /// Whole objects painted from **inside a form XObject** —
    /// `delete_objects_in_form`. A different index space, which is what the
    /// variant says.
    LeavesInForm {
        /// The 0-based page.
        page: usize,
        /// Leaf indices, ascending and unique.
        leaves: Vec<usize>,
    },
    /// One **subpath** of one path object — `delete_subpath`.
    Subpath {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The subpath, in decomposition order.
        subpath: usize,
    },
    /// One **show operator** of one text object — `delete_text_run`.
    ///
    /// One label off a sheet whose 237 labels share a single `BT`…`ET`.
    TextRun {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The run, in content order — the numbering the hit test returns.
        run: usize,
    },
    /// One **anchor** of one path object — `delete_node`.
    Node {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The anchor, object-scoped.
        node: usize,
    },
}

/// Why a Delete removes nothing.
///
/// ★ Every variant is reported **by name** on the diagnostic channel, and the
/// three that an operator can meet without having made a mistake carry a
/// sentence in [`crate::text::deleting`]. That split is the whole design: a bar
/// that narrates the obvious stops being read, and a program that says nothing
/// when a key does nothing is the founding defect of this project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The page has no readable object model, so nothing can be verified and
    /// nothing may be promised. Reachable when the page failed to decompose.
    ///
    /// ★ Only the deeper rungs need the model at all; the Object rung is
    /// answered from the selection alone, so a page that will not decompose
    /// can still have its objects deleted. That asymmetry is deliberate — see
    /// [`subject`].
    NoObjectModel,
    /// Nothing is selected on this page.
    NothingSelected,
    /// A rung above Object with no entry to be inside of. `normalise` makes
    /// this unrepresentable; it is carried so the recovery is named rather
    /// than silent.
    NoPartEntered,
    /// The Node rung with no anchor picked — *"inside this part, nothing
    /// picked yet"*, which is a real state the ladder can be in.
    NoNodeEntered,
    /// The entered target is painted **inside a form XObject**, and
    /// `pdfcer-core` has no `delete_subpath_in_form`, `delete_node_in_form` or
    /// `delete_text_run_in_form` — measured against the locked engine, not
    /// assumed. Its six form-interior verbs are five moves and one whole-object
    /// delete.
    ///
    /// A real engine gap rather than a shell one, and the only rung pair left
    /// that cannot delete. It is named so the operator is told a limit rather
    /// than shown a key that does nothing.
    InsideForm,
    /// The entered object is not addressable by a page paint-order index and
    /// is not a leaf either — unreachable through `TargetId`'s two variants,
    /// carried so a third variant is a compile error rather than a silence.
    UnaddressableObject,
    /// The entered object has no parts at all — an image, or a form treated as
    /// one object. There is nothing below it to delete.
    NoPartsInObject,
    /// The **Node** rung on a text object. A run's glyphs are not anchors and
    /// `pdfcer-core` has no verb that removes one character from a show
    /// operator; editing the string is `format_text`'s job and a different
    /// gesture entirely.
    NoNodeVerbForText,
    /// **Several anchors are selected and `delete_node` is singular.**
    ///
    /// ★ Refused rather than looped, and this is the one judgement in this
    /// module that is worth arguing with. `move_nodes` exists and takes a
    /// slice, so a multi-anchor drag is one command; there is no `delete_nodes`,
    /// so a multi-anchor delete would be N commands and N undo entries for one
    /// press — and worse, each `delete_node` excises a byte span and therefore
    /// **renumbers**, so the second index would be planned against offsets the
    /// first invalidated. Acting on only the entered one would be the
    /// `selected_nodes_on` defect exactly: four anchors highlighted, one
    /// removed, nothing said. Carries the count, because a refusal that cannot
    /// say how many were selected is one the operator cannot act on.
    ManyNodes(usize),
    /// **§9.4.2 — removing this label would slide the next one.** R83, asked
    /// before the press. Carries the run index the operator picked; the remedy
    /// is to delete the later one first. See the module header for why this one
    /// refusal is pre-empted and the rest are left to the engine.
    RunWouldMoveNext(usize),
}

/// **Which delete verb this selection reaches, or why none does.**
///
/// Asked from exactly two places — `canvas::keys`' Delete/Backspace and
/// `app::dispatch::format`'s `format.delete` — because a destructive rule
/// stated twice is a rule that drifts, and the drift here removes a drawing
/// view instead of a line.
///
/// # ★ Why `provider` is an `Option` and the Object rung does not need it
///
/// The Object rung's operand list comes from the selection alone: an entry
/// already holds a resolved `TargetId`, and `object_indices_on` is a filter
/// over four integers. The deeper rungs need the object model to answer *what
/// kind of part is this* — a subpath and a show operator wear the same
/// `subpath: Some(n)` field on [`Selection`] and reach different verbs — so
/// they and only they decline [`Refusal::NoObjectModel`] when it is absent.
///
/// Making the whole function require a model would have made a page that
/// cannot decompose un-deletable at the rung where deletion needs no
/// decomposition at all, which is a limit invented by a signature rather than
/// by a fact.
pub fn subject(
    selection: &SelectionState,
    page: usize,
    provider: Option<&ObjectModelProvider>,
) -> Result<DeleteSubject, Refusal> {
    match selection.level() {
        SelectionLevel::Object => object_rung(selection, page),
        SelectionLevel::Part => {
            let provider = provider.ok_or(Refusal::NoObjectModel)?;
            part_rung(selection, page, provider)
        }
        SelectionLevel::Node => {
            let provider = provider.ok_or(Refusal::NoObjectModel)?;
            node_rung(selection, page, provider)
        }
    }
}

/// The Object rung: whole objects, in whichever of the two index spaces the
/// selection is made of.
///
/// ★ Unchanged behaviour, lifted verbatim out of `canvas::keys`. The page's own
/// paint order wins when both are present, because `delete_objects` is the verb
/// with the erase preview and the leaf list is the fallback for a selection
/// made **entirely** of form-interior targets — which is the state an ordinary
/// click has been able to produce since the deep hit test landed.
fn object_rung(selection: &SelectionState, page: usize) -> Result<DeleteSubject, Refusal> {
    let objects = selection.object_indices_on(page);
    if !objects.is_empty() {
        return Ok(DeleteSubject::Objects { page, objects });
    }
    let leaves = selection.leaf_indices_on(page);
    if leaves.is_empty() {
        Err(Refusal::NothingSelected)
    } else {
        Ok(DeleteSubject::LeavesInForm { page, leaves })
    }
}

/// The Part rung: one subpath, or one label.
///
/// ★★ The kind decides the verb, and the **address space decides whether a verb
/// exists at all** — asked in that order, exactly as `moving::eligible` asks
/// it, because a form-interior part has no delete verb of any kind and saying
/// so first is what stops the kind match promising one.
fn part_rung(
    selection: &SelectionState,
    page: usize,
    provider: &ObjectModelProvider,
) -> Result<DeleteSubject, Refusal> {
    let entry = entered(selection, page)?;
    let part = entry.subpath.ok_or(Refusal::NoPartEntered)?;
    if entry.object.is_leaf() {
        return Err(Refusal::InsideForm);
    }
    let object = entry
        .object
        .page_object_index()
        .ok_or(Refusal::UnaddressableObject)?;
    match provider.part_kind_of(entry.object) {
        Some(PartKind::Subpath) => Ok(DeleteSubject::Subpath {
            page,
            object,
            subpath: part,
        }),
        Some(PartKind::Run) => {
            // R83, and the whole reason this function takes a provider rather
            // than a `PartKind`. See the module header.
            if provider.text_run_delete_would_move_next(object, part) {
                return Err(Refusal::RunWouldMoveNext(part));
            }
            Ok(DeleteSubject::TextRun {
                page,
                object,
                run: part,
            })
        }
        None => Err(Refusal::NoPartsInObject),
    }
}

/// The Node rung: one anchor.
fn node_rung(
    selection: &SelectionState,
    page: usize,
    provider: &ObjectModelProvider,
) -> Result<DeleteSubject, Refusal> {
    let entry = entered(selection, page)?;
    let node = entry.node.ok_or(Refusal::NoNodeEntered)?;
    if entry.object.is_leaf() {
        return Err(Refusal::InsideForm);
    }
    let object = entry
        .object
        .page_object_index()
        .ok_or(Refusal::UnaddressableObject)?;
    match provider.part_kind_of(entry.object) {
        Some(PartKind::Subpath) => {
            // ★ The whole selected set, not the entered entry — the same read
            // `moving::eligible` makes, and for the same reason its comment
            // gives: the model has held a multi-anchor selection since the Node
            // rung landed, and a consumer that asks `entered_object()` sees the
            // first entry only. There it produces `move_nodes`; here there is
            // no plural verb, so it produces a refusal that says how many.
            let nodes = selection.selected_nodes_on(page, entry.object);
            if nodes.len() > 1 {
                return Err(Refusal::ManyNodes(nodes.len()));
            }
            Ok(DeleteSubject::Node { page, object, node })
        }
        Some(PartKind::Run) => Err(Refusal::NoNodeVerbForText),
        None => Err(Refusal::NoPartsInObject),
    }
}

/// The entered entry of a deeper rung, refusing one that belongs to another
/// page rather than addressing page A's index space with page B's number.
///
/// [`crate::canvas::moving`]'s `entered_entry` in every respect; kept separate
/// rather than made public there because the two modules' refusal enums are
/// different types and a shared helper would have to be generic over the error
/// to save four lines.
fn entered(selection: &SelectionState, page: usize) -> Result<Selection, Refusal> {
    selection
        .entered_object()
        .filter(|e| e.page == page)
        .ok_or(Refusal::NothingSelected)
}

/// **The ONE action a Delete becomes**, once [`subject`] has said which verb.
///
/// A separate function from [`subject`] for [`crate::canvas::moving::action`]'s
/// reason: the decision is the part worth unit-testing exhaustively, and the
/// translation is a five-arm match that cannot fail. Keeping them apart is also
/// what lets the two call sites share the decision and differ in what they do
/// with it — the ribbon's arm holds the erase preview, the key's does not.
///
/// Infallible. Every variant of [`DeleteSubject`] names a verb this shell
/// calls; there is no arm that can decline here, which is R9 read as a type:
/// a routing enum must not be able to represent a case that renders nothing.
#[must_use]
pub fn action(subject: DeleteSubject) -> crate::app::actions::VectorAction {
    use crate::app::actions::VectorAction;
    match subject {
        DeleteSubject::Objects { page, objects } => VectorAction::DeleteSelection { page, objects },
        DeleteSubject::LeavesInForm { page, leaves } => {
            VectorAction::DeleteLeavesInForm { page, leaves }
        }
        DeleteSubject::Subpath {
            page,
            object,
            subpath,
        } => VectorAction::DeleteSubpath {
            page,
            object,
            subpath,
        },
        DeleteSubject::TextRun { page, object, run } => {
            VectorAction::DeleteTextRun { page, object, run }
        }
        DeleteSubject::Node { page, object, node } => {
            VectorAction::DeleteNode { page, object, node }
        }
    }
}

/// **Say why nothing was deleted** — on the trace always, on screen when the
/// operator could not have known.
///
/// # ★★ Which refusals get a sentence, and the rule behind the split
///
/// Four do, and they are the four an operator meets **without having made a
/// mistake**:
///
/// * [`Refusal::NoObjectModel`] — the page's content would not decompose, so
///   nothing inside an object can be addressed. **Added 2026-09-05**: it was
///   silent while it was only ever reachable through a frame that forgot to
///   ask, which made it a bug report rather than a message. Now that
///   `canvas::modelneed` asks on the frame the key arrives, reaching it means
///   the document really is unreadable at that depth — and an operator who
///   selected a line, pressed Delete and got nothing is owed that sentence.
///
/// * [`Refusal::RunWouldMoveNext`] — they picked a label, pressed Delete, and
///   the file's own structure forbids it. There is a remedy and it always
///   works.
/// * [`Refusal::InsideForm`] — they have an outline round the thing they want
///   gone and the key does nothing. From where they sit, Delete is broken.
/// * [`Refusal::ManyNodes`] — they Shift-clicked four anchors and watched four
///   highlight. Removing one silently would be worse than refusing.
///
/// The rest describe states the operator put themselves in and can see —
/// nothing selected, an image with no parts, the Node rung on a line of text —
/// and `moving::decline`'s argument applies unchanged: *a surface that narrates
/// the obvious stops being read*.
///
/// # Why the sentence travels as a note and not as a decline
///
/// `app::status::decline` is written by the one dispatcher and read by the one
/// bar; `record_notes` is the channel already used for *"a limit with nowhere
/// else to be said"* — `canvas::interact` raises one from the canvas when a
/// caret cannot be placed, which is the identical shape: no edit happened, no
/// epoch moved, and the operator is owed a sentence anyway. The epoch passed is
/// the **current** one, so the sentence stands until the next real edit moves
/// past it, which is what retires it without anything having to remember to.
/// # ★★★ `model_attempted`, and why a refusal carries how it was reached
///
/// [`Refusal::NoObjectModel`] is raised for two causes that look identical from
/// here: the page genuinely would not decompose, or **this frame never asked**
/// for the decomposition. The second is a defect that has shipped four times
/// (see `canvas::modelneed`), and for one commit it was reported in the first's
/// words — `reason=NoObjectModel`, with nothing to say which.
///
/// So the flag travels onto the trace as `asked=`. A `debug_assert` at
/// `canvas::keys`' call site turns the bad case into a panic under test; this
/// is the half that survives into a release build, where a driven check reads
/// it. **It is not shown to the operator** — from their chair both causes are
/// the same event and both are answered by the same sentence.
pub fn decline(selection: &SelectionState, reason: Refusal, epoch: u64, model_attempted: bool) {
    if let Some(sentence) = crate::text::deleting::refusal(reason) {
        crate::app::actions::record_note(epoch, sentence.to_owned());
    }
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-delete-declined level={:?} sel={} reason={reason:?} asked={model_attempted}",
            selection.level(),
            selection.len(),
        )
    });
}

#[cfg(test)]
mod tests;
