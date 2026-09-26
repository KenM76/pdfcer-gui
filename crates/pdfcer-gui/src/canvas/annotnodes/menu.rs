//! # `canvas::annotnodes::menu` — **the right-click route to a shape's nodes**
//!
//! ## The operator's report, and the half of it that was still open
//!
//! > *"I also can't edit or delete nodes of a markup shape once it is drawn."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/annotnodes/menu.md`.

use pdfcer_core::edit::EditError;
use pdfcer_core::object::ObjId;
use pdfcer_core::vector::Point;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::dimdrag::VertexIntent;
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::SelectionState;

/// `egui::Memory` key for the node or segment the last right-click landed on.
const PICK_MEMORY_KEY: &str = "pdfcer-markup-node-pick"; // ui-text-exempt: internal memory id, never displayed

/// `markup-node-menu id=… pick=… insert=… remove=…` — what a right-click on a
/// markup shape resolved to, and what the two rows will look like.
pub const TRACE_MENU: &str = "markup-node-menu"; // ui-text-exempt: diagnostic trace name

/// `markup-node-command id=… cmd=… pick=…` — a menu row was pressed and this is
/// the operand it was carrying.
pub const TRACE_COMMAND: &str = "markup-node-command"; // ui-text-exempt: diagnostic trace name

/// How much slack a right-click gets around a **segment**, in points.
const SEGMENT_SLACK_PT: f32 = 6.0;

/// **What the right-click landed on.**
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum NodePick {
    /// On or near an existing node, by index.
    Node(usize),
    /// On or near a segment. `after` is the **flat** index of the segment's
    /// first node, which is exactly the engine's `VertexEdit::Insert { after }`
    /// spelling — `after == len - 1` is the closing segment of a closed shape
    /// and appends, which is what the engine's own doc comment says it means.
    /// For an `/Ink` it is converted to `(stroke, after)` by
    /// [`super::ink::StrokeTable`] when the plan is built, and because the
    /// segment list never spans two strokes the new point always lands in the
    /// stroke the operator pointed at.
    Segment {
        /// The segment's first node, flat index.
        after: usize,
        /// Where on it the pointer was, in **page** space (PDF user space,
        /// y-up), projected onto the segment.
        ///
        /// Interpolated between the two nodes in page space from a parameter
        /// measured in screen space, rather than converted back from the
        /// pointer: the pointer is up to [`SEGMENT_SLACK_PT`] off the line, and
        /// an inserted node that is not ON the segment it was inserted into
        /// visibly kinks the shape on the frame it appears.
        at: Point,
    },
    /// Nowhere near the shape.
    #[default]
    Elsewhere,
}

impl NodePick {
    /// A short word for the trace. Never displayed.
    fn word(self) -> String {
        match self {
            // ui-text-exempt: diagnostic trace fragments, never displayed in the UI.
            Self::Node(index) => format!("node:{index}"),
            Self::Segment { after, .. } => format!("segment:{after}"),
            Self::Elsewhere => "elsewhere".to_owned(),
        }
    }
}

/// **How one of the two node rows should be drawn**, decided by the engine.
///
/// See the module header's table. The three states are R9's three answers, and
/// the enum exists so a caller cannot express a fourth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowState {
    /// Drawn and pressable.
    Live,
    /// Drawn and **greyed**, with the command's own tooltip explaining why.
    /// Reached only by `ReshapeWouldBreachVertexFloor` — the one refusal that
    /// stops being true when the operator draws another corner.
    Greyed,
    /// **Not drawn at all.** The shape's kind will never accept this edit.
    Absent,
}

impl RowState {
    /// Whether the row is drawn — the `visible_when` half.
    #[must_use]
    pub fn shown(self) -> bool {
        !matches!(self, Self::Absent)
    }

    /// Whether the row is pressable — the `enabled_when` half.
    #[must_use]
    pub fn enabled(self) -> bool {
        matches!(self, Self::Live)
    }

    /// A short word for the trace. Never displayed.
    fn word(self) -> &'static str {
        match self {
            // ui-text-exempt: diagnostic trace fragments, never displayed in the UI.
            Self::Live => "live",
            Self::Greyed => "greyed",
            Self::Absent => "absent",
        }
    }
}

/// **What the two node rows of `canvas.markup` look like this frame.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rows {
    /// *Add a point here.*
    pub insert: RowState,
    /// *Remove this point.*
    pub remove: RowState,
}

impl Default for Rows {
    fn default() -> Self {
        Self {
            insert: RowState::Absent,
            remove: RowState::Absent,
        }
    }
}

/// **Which node or segment a right-click at `screen` landed on.**
#[must_use]
pub fn pick_at(
    doc: &OpenDoc,
    map: &PageMapping,
    selection: &SelectionState,
    screen: egui::Pos2,
) -> NodePick {
    // A node first — see the precedence note. `node_at` owns the node tolerance
    // and the coincident-node tie-break, so the two gestures that can grab a
    // node (the drag and this menu) cannot disagree about which one they got.
    if let Some(index) = super::node_at(doc, map, selection, screen) {
        return NodePick::Node(index);
    }
    let Some(shape) = super::geometry(doc, selection) else {
        return NodePick::Elsewhere;
    };
    let canvas = super::nodes(doc, selection);
    if canvas.len() != shape.points.len() {
        // The painter's list and the geometry have gone out of step, which can
        // only happen if a node failed to convert to canvas space. Refusing is
        // right: an index into one list used against the other is the *"the
        // right verb on the wrong corner"* defect this module's trace exists to
        // catch, and it is cheaper to offer no row than to offer a wrong one.
        return NodePick::Elsewhere;
    }
    // The segments come from `Geometry::segment_pairs` — the SAME list the
    // preview is drawn from — rather than from a loop of this module's own.
    // That is what makes two facts true by construction rather than by
    // agreement: a polygon's closing edge is offered exactly when the preview
    // draws it, and a freehand mark's two strokes have NO segment between them
    // to right-click on, so *"Add a point here"* can never bridge two strokes.
    let mut best: Option<(f32, (usize, usize), f32)> = None;
    for (first, second) in shape.segment_pairs() {
        let (Some(a), Some(b)) = (canvas.get(first), canvas.get(second)) else {
            continue;
        };
        let (distance, t) = distance_to_segment(screen, map.to_screen(*a), map.to_screen(*b));
        if distance <= SEGMENT_SLACK_PT && best.is_none_or(|(best_d, _, _)| distance < best_d) {
            best = Some((distance, (first, second), t));
        }
    }
    let Some((_, (after, next), t)) = best else {
        return NodePick::Elsewhere;
    };
    let (Some(a), Some(b)) = (shape.points.get(after), shape.points.get(next)) else {
        return NodePick::Elsewhere;
    };
    NodePick::Segment {
        after,
        at: Point::new(
            f64::from(t).mul_add(b.x - a.x, a.x),
            f64::from(t).mul_add(b.y - a.y, a.y),
        ),
    }
}

/// Distance from `p` to the segment `a`–`b`, and **where along it** the closest
/// point is, as a parameter in `0.0..=1.0`.
fn distance_to_segment(p: egui::Pos2, a: egui::Pos2, b: egui::Pos2) -> (f32, f32) {
    let ab = b - a;
    let length_squared = ab.length_sq();
    if length_squared <= f32::EPSILON {
        return (p.distance(a), 0.0);
    }
    let t = (((p - a).dot(ab)) / length_squared).clamp(0.0, 1.0);
    (p.distance(a + t * ab), t)
}

/// **What the engine would allow for this pick** — the two rows' states.
#[must_use]
pub fn rows(doc: &OpenDoc, selection: &SelectionState, pick: NodePick) -> Rows {
    let Some(shape) = super::geometry(doc, selection) else {
        return Rows::default();
    };
    let session = &doc.session;
    // `from` is only read by a MOVE plan's delta and neither row moves, so
    // the node's own position is passed — a zero displacement, never consulted.
    let state = |intent: VertexIntent, index: usize, at: Point| {
        let from = shape.points.get(index).copied().unwrap_or(at);
        let Some(plan) = super::planned(&shape, intent, index, from, at) else {
            // An `/Ink` anchor index the stroke table cannot place. The engine
            // cannot be asked; the row is absent, as for any other refusal
            // that is not the floor.
            return RowState::Absent;
        };
        match plan.preview(session, shape.id) {
            Ok(()) => RowState::Live,
            // The temporary refusals, one per family: a closed shape at three
            // corners, an open one at two, a freehand stroke at two. Draw or
            // add another point and the row comes back, which is what makes
            // greying-with-a-reason correct here and wrong everywhere else in
            // this module.
            Err(
                EditError::ReshapeWouldBreachVertexFloor { .. }
                | EditError::InkStrokeWouldBreachPointFloor { .. },
            ) => RowState::Greyed,
            // `GeometryNotReshapable` for a `/Line`, a `/Square`, a `/Circle`
            // or a text markup; `AnnotationLocked` for a shape the FILE forbids
            // changing; `AnnotationIsCeDimension` for the shape
            // `canvas::dimdrag` owns; the ink index refusals for anchors that
            // have gone out of step with the file. None of them stops being
            // true while the operator looks at the menu, so none of them is
            // greyed.
            Err(_) => RowState::Absent,
        }
    };
    match pick {
        NodePick::Node(index) => Rows {
            insert: RowState::Absent,
            remove: state(VertexIntent::Remove, index, Point::new(0.0, 0.0)),
        },
        NodePick::Segment { after, at } => Rows {
            insert: state(VertexIntent::Insert, after, at),
            remove: RowState::Absent,
        },
        // Both absent, and this is the common case rather than an edge one: a
        // right-click on a markup shape's INTERIOR is a right-click on the
        // shape, opens the markup menu, and is nowhere near an edge. The menu
        // still offers properties, the clipboard and delete — a menu with
        // something to say, which is what stops the whole context from
        // collapsing to "nothing happened".
        NodePick::Elsewhere => Rows::default(),
    }
}

/// **Park the pick for the life of the popup.** Called once, on the click.
pub fn park(ctx: &egui::Context, pick: NodePick) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(PICK_MEMORY_KEY), pick));
}

/// Read the parked pick. [`NodePick::Elsewhere`] before any right-click.
#[must_use]
pub fn parked(ctx: &egui::Context) -> NodePick {
    ctx.data_mut(|d| {
        d.get_temp::<NodePick>(egui::Id::new(PICK_MEMORY_KEY))
            .unwrap_or_default()
    })
}

/// Record what the menu resolved to, once per click.
pub fn trace(id: ObjId, pick: NodePick, rows: Rows) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        // Placed directly above the literal — see `canvas::trace_layout`.
        format!(
            "{TRACE_MENU} id={} pick={} insert={} remove={}",
            id.num,
            pick.word(),
            rows.insert.word(),
            rows.remove.word(),
        )
    });
}

/// **The action a pressed node row raises**, or `None` if it cannot.
#[must_use]
pub fn action_for(
    ctx: &egui::Context,
    doc: &OpenDoc,
    selection: &SelectionState,
    insert: bool,
) -> Option<Action> {
    let shape = super::geometry(doc, selection)?;
    let id = shape.id;
    let pick = parked(ctx);
    let states = rows(doc, selection, pick);
    // Built through the SAME `planned` the row's state was asked with, so the
    // action a press raises is the edit the engine said yes to — for an `/Ink`
    // that includes the flat-index → `(stroke, point)` conversion, which a
    // second spelling here could get off by a stroke.
    let plan = match (insert, pick) {
        (true, NodePick::Segment { after, at }) if states.insert.enabled() => {
            super::planned(&shape, VertexIntent::Insert, after, at, at)
        }
        (false, NodePick::Node(index)) if states.remove.enabled() => {
            let at = shape
                .points
                .get(index)
                .copied()
                .unwrap_or(Point::new(0.0, 0.0));
            super::planned(&shape, VertexIntent::Remove, index, at, at)
        }
        _ => None,
    };
    let action = match plan {
        Some(plan) => plan.action(id),
        None => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI.
                format!(
                    "{TRACE_COMMAND} id={} cmd={} pick={} outcome=declined",
                    id.num,
                    if insert { "insert" } else { "remove" },
                    pick.word(),
                )
            });
            return None;
        }
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        format!(
            "{TRACE_COMMAND} id={} cmd={} pick={} outcome=raised",
            id.num,
            if insert { "insert" } else { "remove" },
            pick.word(),
        )
    });
    Some(Action::Annot(action))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A right-click on the middle of a segment picks that segment, and the
    /// insertion point is **on the line** rather than under the pointer.
    #[test]
    fn a_click_beside_a_segment_projects_onto_it() {
        let (distance, t) = distance_to_segment(
            egui::pos2(50.0, 4.0),
            egui::pos2(0.0, 0.0),
            egui::pos2(100.0, 0.0),
        );
        assert!((distance - 4.0).abs() < 1e-4, "{distance}");
        assert!((t - 0.5).abs() < 1e-4, "{t}");
    }

    /// **The clamp.** A click past the end of a segment is measured to the
    /// END, not to the infinite line — C5, the convention that stops a short
    /// edge claiming a stripe across the sheet.
    ///
    /// Falsified: dropping `.clamp(0.0, 1.0)` gives `t = 2.0` and a distance
    /// of 0, so both assertions fail.
    #[test]
    fn a_click_past_the_end_of_a_segment_is_measured_to_the_end() {
        let (distance, t) = distance_to_segment(
            egui::pos2(200.0, 0.0),
            egui::pos2(0.0, 0.0),
            egui::pos2(100.0, 0.0),
        );
        assert!((distance - 100.0).abs() < 1e-4, "{distance}");
        assert!((t - 1.0).abs() < 1e-4, "{t}");
    }

    /// A degenerate segment — two coincident nodes, which `/Vertices` permits
    /// — behaves as the point it is drawn as instead of dividing by zero.
    ///
    /// Falsified: removing the `length_squared` guard makes `t` NaN, and
    /// `assert!(t == 0.0)` fails (NaN compares false against everything).
    #[test]
    fn a_zero_length_segment_answers_its_own_point() {
        let (distance, t) = distance_to_segment(
            egui::pos2(3.0, 4.0),
            egui::pos2(0.0, 0.0),
            egui::pos2(0.0, 0.0),
        );
        assert!((distance - 5.0).abs() < 1e-4, "{distance}");
        assert!((t - 0.0).abs() < 1e-6, "{t}");
    }

    /// **R9, as a pair of booleans.** A greyed row is DRAWN and not pressable;
    /// an absent one is neither. The two halves are separate questions and a
    /// build that answered them from one field would either grey what should
    /// vanish or hide what should explain itself.
    #[test]
    fn a_greyed_row_is_drawn_and_an_absent_one_is_not() {
        assert!(RowState::Live.shown() && RowState::Live.enabled());
        assert!(RowState::Greyed.shown() && !RowState::Greyed.enabled());
        assert!(!RowState::Absent.shown() && !RowState::Absent.enabled());
    }

    /// With no shape picked, both rows are absent — which is what lets the rest
    /// of the markup menu open over a shape's interior.
    ///
    /// Falsified: defaulting `Rows` to `Live` makes both assertions fail.
    #[test]
    fn no_pick_draws_neither_node_row() {
        let rows = Rows::default();
        assert!(!rows.insert.shown());
        assert!(!rows.remove.shown());
    }

    /// The pick's default is *nowhere near the shape*, so a frame before any
    /// right-click cannot be read as "node 0".
    #[test]
    fn the_default_pick_names_no_node() {
        assert_eq!(NodePick::default(), NodePick::Elsewhere);
    }
}
