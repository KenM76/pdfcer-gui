//! [`Plan`]: the one edit a node-drag frame asks the engine for, previewed
//! and turned into its action.

use super::*;

/// **The edit one frame asks the engine for**, in whichever of the two verb
/// families the shape belongs to.
#[derive(Debug, Clone, PartialEq)]
pub enum Plan {
    /// A `/Polygon`, `/PolyLine` or `/Line` — one index.
    Vertex(VertexEdit),
    /// An `/Ink` — `(stroke, point)`, converted from the flat anchor index by
    /// [`ink::StrokeTable`].
    Ink(InkEdit),
}

impl Plan {
    /// **Ask the engine whether this edit is allowed**, without doing it.
    ///
    /// Each family's preview shares one body with its mutating verb —
    /// `reshape_plan` for vertices, `ink_plan` for ink — so neither can
    /// disagree with what the release would do. The forecast itself is not
    /// returned: the drag needs a yes or a worded no, and the disclosure the
    /// ink forecast carries (`appearance_was_pdfces`) is read at apply time by (old-name-exempt: the engine's own field name, quoted verbatim)
    /// `app::actions::annots`, where the sentence can reach the status line.
    pub fn preview(&self, session: &EditSession, id: ObjId) -> Result<(), EditError> {
        match self {
            Self::Vertex(edit) => session.reshape_annotation_preview(id, *edit).map(|_| ()),
            Self::Ink(edit) => session.reshape_ink_preview(id, edit).map(|_| ()),
        }
    }

    /// **The action a release raises** for this plan — one of the six node
    /// variants of [`AnnotAction`], each of which reaches exactly one engine
    /// verb.
    #[must_use]
    pub fn action(self, id: ObjId) -> AnnotAction {
        match self {
            Self::Vertex(VertexEdit::Move { index, dx, dy }) => {
                AnnotAction::MoveNode { id, index, dx, dy }
            }
            Self::Vertex(VertexEdit::Insert { after, at }) => {
                AnnotAction::InsertNode { id, after, at }
            }
            Self::Vertex(VertexEdit::Remove { index }) => AnnotAction::RemoveNode { id, index },
            Self::Ink(InkEdit::MovePoint {
                stroke,
                point,
                dx,
                dy,
            }) => AnnotAction::MoveInkPoint {
                id,
                stroke,
                point,
                dx,
                dy,
            },
            Self::Ink(InkEdit::InsertPoint { stroke, after, at }) => AnnotAction::InsertInkPoint {
                id,
                stroke,
                after,
                at,
            },
            Self::Ink(InkEdit::RemovePoint { stroke, point }) => {
                AnnotAction::RemoveInkPoint { id, stroke, point }
            }
            Self::Ink(InkEdit::MoveStroke { stroke, dx, dy }) => {
                AnnotAction::MoveInkStroke { id, stroke, dx, dy }
            }
            Self::Ink(InkEdit::RemoveStroke { stroke }) => {
                AnnotAction::RemoveInkStroke { id, stroke }
            }
            // `ReplaceStroke`, and whatever `#[non_exhaustive]` adds: nothing
            // here builds one. The general decline rather than a panic: a wrong
            // sentence on release is recoverable, a crash mid-gesture is not.
            Self::Ink(_) => AnnotAction::DeclineNodeEdit {
                why: crate::text::markup::NodeEditRefusal::Refused,
            },
        }
    }

    /// A short word for the trace. Never displayed.
    pub(super) fn word(&self) -> &'static str {
        match self {
            // ui-text-exempt: diagnostic trace fragments, never displayed in the UI.
            Self::Vertex(_) => "vertex",
            Self::Ink(_) => "ink",
        }
    }
}
