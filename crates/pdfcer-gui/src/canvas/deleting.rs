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
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/deleting.md`.

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
    /// One **visual line** of one text object — `delete_text_run`, applied to
    /// every show operator the line is written in.
    ///
    /// One label off a sheet whose 237 show operators share a single
    /// `BT`…`ET` and form 144 lines.
    TextLine {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The line, in content order — the numbering the hit test returns.
        line: usize,
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
/// Every variant is reported **by name** on the diagnostic channel, and the
/// three that an operator can meet without having made a mistake carry a
/// sentence in [`crate::text::deleting`]. That split is the whole design: a bar
/// that narrates the obvious stops being read, and a program that says nothing
/// when a key does nothing is the founding defect of this project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The page has no readable object model, so nothing can be verified and
    /// nothing may be promised. Reachable when the page failed to decompose.
    ///
    /// Only the deeper rungs need the model at all; the Object rung is
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
    /// The entered target is painted **inside a form XObject**, and this shell
    /// declines rather than deleting it.
    ///
    /// THE SENTENCE IS THIS SHELL'S, NOT THE ENGINE'S. The pinned
    /// `pdfcer-core` declares `delete_text_run_in_form`,
    /// `delete_subpath_in_form` and `delete_node_in_form` alongside the six
    /// form-interior move verbs. Nothing upstream forbids this. What is missing
    /// is entirely local: a leaf carries no page paint-order index, so
    /// `part_hits_of` matches nothing for it and the Part rung cannot be
    /// entered inside a form in the first place. Opening that seam is the work,
    /// and it is the same seam the in-form move verbs already went through.
    ///
    /// Until it is opened the refusal is still the right behaviour, because a
    /// key that silently does nothing is worse than one that says why — but it
    /// must not be worded, here or on screen, as a limit of the engine. D57
    /// carries the repair.
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
    /// Refused rather than looped, and this is the one judgement in this
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
    /// **Several chunks are selected and `delete_text_run` is singular.**
    ///
    /// The Part-rung twin of [`Self::ManyNodes`], and it refuses for the
    /// stronger of that variant's two reasons. A plural MOVE is safe and is
    /// built — `move_text_run` rewrites an operand in place, adds no operator a
    /// run index counts, so a loop over N lines renumbers nothing. Deleting
    /// **excises** a show operator, so every later run index shifts down by one
    /// and the second call in a loop would address a line the first moved.
    ///
    /// Carries the count, for the same reason `ManyNodes` does: a refusal that
    /// cannot say how many were selected is one the operator cannot act on.
    ManyLines(usize),
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
/// # Why `provider` is an `Option` and the Object rung does not need it
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
        Some(PartKind::TextLine) => {
            // The whole selected set, not the entered entry — `moving::eligible`
            // reads it the same way one line below and builds a plural move out
            // of it. There is no plural DELETE, and looping this one is not the
            // same judgement as looping the move: excising a show operator
            // renumbers every later run. See `Refusal::ManyLines`.
            let lines = selection.selected_parts_on(page, entry.object);
            if lines.len() > 1 {
                return Err(Refusal::ManyLines(lines.len()));
            }
            // R83, and the whole reason this function takes a provider rather
            // than a `PartKind`. See the module header.
            if provider.text_line_delete_would_move_next(object, part) {
                return Err(Refusal::RunWouldMoveNext(part));
            }
            Ok(DeleteSubject::TextLine {
                page,
                object,
                line: part,
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
            // The whole selected set, not the entered entry — the same read
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
        Some(PartKind::TextLine) => Err(Refusal::NoNodeVerbForText),
        None => Err(Refusal::NoPartsInObject),
    }
}

/// The entered entry of a deeper rung, refusing one that belongs to another
/// page rather than addressing page A's index space with page B's number.
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
        DeleteSubject::TextLine { page, object, line } => {
            VectorAction::DeleteTextLine { page, object, line }
        }
        DeleteSubject::Node { page, object, node } => {
            VectorAction::DeleteNode { page, object, node }
        }
    }
}

/// **Say why nothing was deleted** — on the trace always, on screen when the
/// operator could not have known.
///
/// # Which refusals get a sentence, and the rule behind the split
///
/// Five do, and they are the five an operator meets **without having made a
/// mistake**:
///
/// * [`Refusal::RunWouldMoveNext`] — they picked a label, pressed Delete, and
///   the file's own structure forbids it. There is a remedy and it always
///   works.
/// * [`Refusal::InsideForm`] — they have an outline round the thing they want
///   gone and the key does nothing. From where they sit, Delete is broken.
/// * [`Refusal::ManyNodes`] — they Shift-clicked four anchors and watched four
///   highlight. Removing one silently would be worse than refusing.
/// * [`Refusal::ManyLines`] — the same press one rung up, and the operator has
///   more reason to be surprised: the identical set can be dragged as one.
/// * [`Refusal::NoObjectModel`] — the page will not decompose, so nothing
///   inside an object can be named. The Object rung still works and the
///   sentence says so.
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
/// # `model_attempted`, and why a refusal carries how it was reached
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
