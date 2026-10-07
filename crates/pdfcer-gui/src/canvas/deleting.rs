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
    /// One **subpath** of one path object inside a form XObject —
    /// `delete_subpath_in_form`.
    SubpathInForm {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by leaf index.
        leaf: usize,
        /// The subpath, in decomposition order.
        subpath: usize,
    },
    /// One **visual line** of one text object inside a form XObject —
    /// `delete_text_run_in_form`, once per show operator of the line.
    TextLineInForm {
        /// The 0-based page.
        page: usize,
        /// The enclosing text object, by leaf index.
        leaf: usize,
        /// The line, in content order.
        line: usize,
    },
    /// One **anchor** of one path object inside a form XObject —
    /// `delete_node_in_form`.
    NodeInForm {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by leaf index.
        leaf: usize,
        /// The anchor, object-scoped.
        node: usize,
    },
}

/// Which index space the entered object is addressed in.
#[derive(Debug, Clone, Copy)]
enum Address {
    /// The page's own paint order.
    Page(usize),
    /// `PageObjects::leaves` — painted from inside a form XObject.
    Leaf(usize),
}

impl Address {
    fn of(entry: &Selection) -> Result<Self, Refusal> {
        if let Some(leaf) = entry.object.leaf_index() {
            return Ok(Self::Leaf(leaf));
        }
        entry
            .object
            .page_object_index()
            .map(Self::Page)
            .ok_or(Refusal::UnaddressableObject)
    }

    const fn subpath(self, page: usize, subpath: usize) -> DeleteSubject {
        match self {
            Self::Page(object) => DeleteSubject::Subpath {
                page,
                object,
                subpath,
            },
            Self::Leaf(leaf) => DeleteSubject::SubpathInForm {
                page,
                leaf,
                subpath,
            },
        }
    }

    const fn text_line(self, page: usize, line: usize) -> DeleteSubject {
        match self {
            Self::Page(object) => DeleteSubject::TextLine { page, object, line },
            Self::Leaf(leaf) => DeleteSubject::TextLineInForm { page, leaf, line },
        }
    }

    const fn node(self, page: usize, node: usize) -> DeleteSubject {
        match self {
            Self::Page(object) => DeleteSubject::Node { page, object, node },
            Self::Leaf(leaf) => DeleteSubject::NodeInForm { page, leaf, node },
        }
    }
}

pub use pdfcer_gui_base::refusals::delete::Refusal;

/// **Which delete verb this selection reaches, or why none does.**
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

/// The Part rung: one subpath, or one label, in whichever index space the
/// entered object lives.
fn part_rung(
    selection: &SelectionState,
    page: usize,
    provider: &ObjectModelProvider,
) -> Result<DeleteSubject, Refusal> {
    let entry = entered(selection, page)?;
    let part = entry.subpath.ok_or(Refusal::NoPartEntered)?;
    let at = Address::of(&entry)?;
    match provider.part_kind_of(entry.object) {
        Some(PartKind::Subpath) => Ok(at.subpath(page, part)),
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
            if provider.text_line_delete_would_move_next_of(entry.object, part) {
                return Err(Refusal::RunWouldMoveNext(part));
            }
            Ok(at.text_line(page, part))
        }
        None => Err(Refusal::NoPartsInObject),
    }
}

/// The Node rung: one anchor, in whichever index space the entered object
/// lives.
fn node_rung(
    selection: &SelectionState,
    page: usize,
    provider: &ObjectModelProvider,
) -> Result<DeleteSubject, Refusal> {
    let entry = entered(selection, page)?;
    let node = entry.node.ok_or(Refusal::NoNodeEntered)?;
    let at = Address::of(&entry)?;
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
            Ok(at.node(page, node))
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
        DeleteSubject::SubpathInForm {
            page,
            leaf,
            subpath,
        } => VectorAction::DeleteSubpathInForm {
            page,
            leaf,
            subpath,
        },
        DeleteSubject::TextLineInForm { page, leaf, line } => {
            VectorAction::DeleteTextLineInForm { page, leaf, line }
        }
        DeleteSubject::NodeInForm { page, leaf, node } => {
            VectorAction::DeleteNodeInForm { page, leaf, node }
        }
    }
}

/// **Say why nothing was deleted** — on the trace always, on screen when the
/// operator could not have known.
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
