//! `vertexintent` — **what a corner drag on a ce dimension or ink mark asks for.**
//!
//! The decision from tool and modifiers stays with the canvas; this is only the answer's type.

/// **What a corner drag is asking for** — move that corner, add one after it,
/// or take it away.
///
/// Derived from the armed tool and the modifiers held on the frame being drawn:
/// `canvas::dimdrag::intent` in the application crate makes the decision, and
/// its module header says why it is read live rather than sampled at the press.
///
/// Three variants rather than a `bool` pair, for the same reason as the
/// canvas's `DimensionPress`: the three reach **three different engine verbs**,
/// and the one thing that must never happen on this canvas is a gesture aimed
/// at the wrong verb. A pair of booleans has a fourth state that means nothing
/// and would have to be resolved somewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VertexIntent {
    /// Reshape: the corner follows the pointer. `move_dimension_vertex`.
    #[default]
    Move,
    /// Add a corner immediately **after** the grabbed one, at the drop point.
    /// `insert_dimension_vertex`.
    Insert,
    /// Take the grabbed corner away. The drop point is ignored — a removal has
    /// no destination. `remove_dimension_vertex`.
    Remove,
}
