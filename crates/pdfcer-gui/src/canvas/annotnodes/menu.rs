//! # `canvas::annotnodes::menu` — **the right-click route to a shape's nodes**
//!
//! ## The operator's report, and the half of it that was still open
//!
//! > *"I also can't edit or delete nodes of a markup shape once it is drawn."*
//!
//! [`super`] closed the *moving* half on 2026-09-05 and half-closed the other
//! two: inserting and removing a node work, and they are reached by **arming
//! the Points tool (`A`) and holding `Ctrl` or `Ctrl+Shift`**. Nothing on
//! screen says so. The note filed with that work said what was missing, and
//! this module is that sentence built:
//!
//! > The natural way to add or remove a corner is a **right-click on the
//! > shape** — *"add a point here"*, *"remove this point"* — which is how the
//! > engine itself describes these two operations. The chords above are a
//! > stopgap. The right-click menu lives in a file another job was working in
//! > today, so it is written down here rather than half-built.
//!
//! ## ★★★ Why the menu route needs no armed tool, and the chord route does
//!
//! It looks like an inconsistency and it is a rule applied at its own edge.
//!
//! The **chord** route requires the Points tool armed because `Ctrl` already
//! means *take this out of the selection* everywhere else on this canvas. A
//! bare `Ctrl`-press on a node is therefore ambiguous by construction, and
//! arming the tool whose whole subject is nodes is what disambiguates it: the
//! operator has said, in advance, *"I am working on points now."*
//!
//! A **menu row is unambiguous by construction.** The operator read the words
//! *"Add a point here"* and chose them. There is no second reading to rule out,
//! so requiring an armed tool first would be carrying a rule past the reason
//! that produced it — and it would put a mode between the operator and a verb
//! they had already named, which is the shape of every complaint in
//! `OPERATOR_REQUESTS.md` about this canvas.
//!
//! ⇒ Recorded here rather than at the call site because it is the kind of
//! asymmetry a later reader "fixes".
//!
//! ## ★★ The engine is ASKED, never restated
//!
//! [`super`]'s header carries the matrix — which subtype accepts a move, an
//! insert, a remove, and what its vertex floor is. **This module does not read
//! that table.** It builds the exact [`VertexEdit`] the row would commit and
//! hands it to `EditSession::reshape_annotation_preview`, which shares one body
//! (`reshape_plan`) with the mutating verb and therefore cannot disagree with
//! what pressing the row would do.
//!
//! What the answer is used for is the R9 decision, and the **error variant is
//! what decides it**:
//!
//! | preview says | row | why |
//! |---|---|---|
//! | `Ok` | drawn, live | it would work |
//! | `Err(ReshapeWouldBreachVertexFloor)` | drawn, **greyed**, tooltip explains | *temporarily* unavailable — draw another corner and it comes back |
//! | any other `Err` | **absent** | a property of the shape's kind, which will not change while the operator looks at it |
//!
//! That is R9 exactly — *"an unavailable capability renders nothing; greying is
//! reserved for temporarily unavailable and is always explained on hover"* —
//! and it is derived rather than declared. A `/Line` gets no *Add a point here*
//! because the engine says `GeometryNotReshapable`, not because this file holds
//! a list of subtypes; the day the engine teaches `/Line` to grow a third
//! point, the row appears with nothing here edited.
//!
//! ## ★★★ The operand problem, and where it is parked
//!
//! A menu row carries **a command id and nothing else**
//! (`egui_shell::manifest::Item::Command`). *"Add a point here"* needs to know
//! **which segment**, and *"Remove this point"* needs to know **which node** —
//! facts that exist only at the instant of the secondary click, on a surface
//! the dispatcher never sees.
//!
//! So the pick is **parked for the life of the popup**, in `egui::Memory`,
//! beside the one thing already parked there for the same reason: which of the
//! canvas menus is open (`canvas::menus`' `MENU_MEMORY_KEY`). The two have the
//! identical lifetime and the identical argument —
//!
//! > `egui` opens a popup ON the secondary click and draws it on every
//! > subsequent frame until it is dismissed. The pointer moves during those
//! > frames — onto the menu itself, which is not over the shape any more — so
//! > recomputing from the live pointer would swap the row's meaning out from
//! > under the operator's hand while they were reading it.
//!
//! ⇒ [`park`] is called once, on the click. [`parked`] is read on every frame
//! the menu is drawn (to decide the two rows' states) **and** again when the
//! command is dispatched (to build the action). One value, three readers, no
//! second derivation of *"which corner did they mean"*.
//!
//! `Memory` rather than `PdfcerApp` state for `MENU_MEMORY_KEY`'s stated
//! reason: this is frame-local interaction state with no meaning across a
//! document, `Memory` is per-`egui::Context`, and a document change therefore
//! starts the next frame with no pick and no popup in flight.
//!
//! ## Rule 15
//!
//! Everything here is about a **markup shape**. A **ce dimension** is also a
//! `/Line`, its nodes are `canvas::dimdrag`'s, and its verb is
//! `move_dimension_vertex`, which **re-measures** — a different operation with
//! a different undo entry and a different disclosure. [`super::geometry`] is
//! the one gate, and it refuses anything that is not
//! [`AnnotKind::Markup`](crate::canvas::selection::AnnotKind::Markup) before
//! this module sees it. The engine refuses again by name
//! (`EditError::AnnotationIsCeDimension`) as the backstop; nothing here routes
//! a dimension anywhere.
//!
//! ## Rule 4 / R8b
//!
//! Nothing in this module paints. It answers three questions — *which node or
//! segment is under the pointer*, *what would the engine allow*, *what action
//! does the row raise* — and every one of them is about the cursor rather than
//! about the document.

use pdfcer_core::edit::{EditError, VertexEdit};
use pdfcer_core::object::ObjId;
use pdfcer_core::vector::Point;

use crate::app::actions::Action;
use crate::app::actions::annot::AnnotAction;
use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::SelectionState;

/// `egui::Memory` key for the node or segment the last right-click landed on.
///
/// One `Id`, per `egui::Context`, for the module header's reason: the pick is
/// taken at the click and read on every frame the popup is drawn, so it has to
/// outlive the click and must not outlive the session.
const PICK_MEMORY_KEY: &str = "pdfcer-markup-node-pick"; // ui-text-exempt: internal memory id, never displayed

/// `markup-node-menu id=… pick=… insert=… remove=…` — what a right-click on a
/// markup shape resolved to, and what the two rows will look like.
///
/// ★ Distinct from [`super::TRACE_MOVE`] and its three siblings, which report a
/// gesture that **changed the document**. This reports a *question being
/// opened*, and a check that read only one of the two could not tell a menu
/// that offered the wrong row from a command that acted on the wrong node.
pub const TRACE_MENU: &str = "markup-node-menu"; // ui-text-exempt: diagnostic trace name

/// `markup-node-command id=… cmd=… pick=…` — a menu row was pressed and this is
/// the operand it was carrying.
///
/// ★ It carries the **pick**, not just the command id, because the whole class
/// of defect this design can produce is *the right verb on the wrong corner*.
/// A trace line has to carry the number a wrong build would get wrong.
pub const TRACE_COMMAND: &str = "markup-node-command"; // ui-text-exempt: diagnostic trace name

/// How much slack a right-click gets around a **segment**, in points.
///
/// Wider than [`super::NODE_GRAB_SLACK_PT`]'s companion tolerance, and
/// deliberately: a node is a drawn 7 pt square the operator can aim at, and a
/// segment is a hairline they cannot. The standing convention at
/// `handles::grip_at` — *a grip's live area may exceed its drawn one and never
/// the reverse* — is about drawn affordances; a segment has no drawn affordance
/// at all, so the number is chosen from what a hand can hold steady rather than
/// from a picture.
///
/// ★ It is not so wide that it swallows the nodes: [`pick_at`] asks for a node
/// **first**, so a click near a corner is a corner even though it is also near
/// two segments. See that function's precedence note.
const SEGMENT_SLACK_PT: f32 = 6.0;

/// **What the right-click landed on.**
///
/// Three answers and no `Option`, because "the pointer was nowhere near the
/// shape" is a real state a menu has to render (both node rows absent, the rest
/// of the markup menu intact) rather than an error, and an `Option<Pick>` would
/// let a caller `unwrap_or_default` its way into treating it as node 0.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum NodePick {
    /// On or near an existing node, by index.
    Node(usize),
    /// On or near a segment. `after` is the index of the segment's **first**
    /// node, which is exactly the engine's `VertexEdit::Insert { after }`
    /// spelling — `after == len - 1` is the closing segment of a closed shape
    /// and appends, which is what the engine's own doc comment says it means.
    Segment {
        /// The segment's first node.
        after: usize,
        /// Where on it the pointer was, in **page** space (PDF user space,
        /// y-up), projected onto the segment.
        ///
        /// ★ Interpolated between the two nodes in page space from a parameter
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
///
/// Both default to [`RowState::Absent`], which is the honest answer for every
/// selection that is not a reshapable markup shape and for a pointer that was
/// nowhere near one: the rows are not drawn, and the rest of the markup menu —
/// properties, the clipboard, delete — opens exactly as it would have.
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
///
/// # ★ Precedence: a node beats a segment, always
///
/// Every node lies on two segments, so within [`SEGMENT_SLACK_PT`] of a corner
/// both answers are true and exactly one can be offered. The corner wins,
/// because *"remove this point"* names a thing the operator can see and *"add a
/// point here"* would insert a duplicate one grid-unit from a node they were
/// plainly aiming at. It is also the safer error: an unwanted removal is one
/// `Ctrl+Z`, and an unwanted insertion leaves a shape that looks unchanged
/// with an extra vertex nobody can find.
///
/// # ★ The comparison is in SCREEN space
///
/// [`super::node_at`]'s argument, unchanged and for the same reason: a
/// tolerance in canvas or page space would shrink as the operator zooms out —
/// exactly when a shape's segments are closest together — and balloon as they
/// zoom in, so that at 800 % a click anywhere near a shape would claim one of
/// its edges.
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
    let Some((_, points, closed)) = super::geometry(doc, selection) else {
        return NodePick::Elsewhere;
    };
    let canvas = super::nodes(doc, selection);
    if canvas.len() != points.len() {
        // The painter's list and the geometry have gone out of step, which can
        // only happen if a node failed to convert to canvas space. Refusing is
        // right: an index into one list used against the other is the *"the
        // right verb on the wrong corner"* defect this module's trace exists to
        // catch, and it is cheaper to offer no row than to offer a wrong one.
        return NodePick::Elsewhere;
    }
    // The segments, in the same order and with the same closing rule the
    // preview uses — `preview_of` adds the closing segment for a closed shape
    // and this must agree with it, or the row offered on a polygon's last edge
    // would be about a segment nothing draws.
    let mut best: Option<(f32, usize, f32)> = None;
    let last = canvas.len().saturating_sub(1);
    for after in 0..canvas.len() {
        let next = if after == last {
            if !closed || canvas.len() < 3 {
                break;
            }
            0
        } else {
            after + 1
        };
        let (a, b) = (map.to_screen(canvas[after]), map.to_screen(canvas[next]));
        let (distance, t) = distance_to_segment(screen, a, b);
        if distance <= SEGMENT_SLACK_PT && best.is_none_or(|(best_d, _, _)| distance < best_d) {
            best = Some((distance, after, t));
        }
    }
    let Some((_, after, t)) = best else {
        return NodePick::Elsewhere;
    };
    let next = if after == last { 0 } else { after + 1 };
    let (a, b) = (points[after], points[next]);
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
///
/// ★ Clamped to the ends, which is the C5 convention `canvas::selection::annot`
/// states: without the clamp a short segment would claim a stripe across the
/// sheet, and the insertion parameter could land off the end of the edge the
/// operator pointed at.
///
/// A degenerate segment (two coincident nodes, which `/Vertices` permits and
/// the engine does not de-duplicate) answers `t = 0.0` and the distance to that
/// point, so it behaves as the single point it is drawn as.
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
///
/// Asked of `reshape_annotation_preview`, per frame, with the exact
/// [`VertexEdit`] the row would commit. See the module header for why the
/// *error variant* is what separates a greyed row from an absent one.
///
/// ★ The cost is one annotation walk per row per frame, and only while the
/// popup is open — [`crate::canvas::menus`] calls this from inside its own
/// "is a markup menu the one being drawn" branch. [`super`]'s header records
/// the standing engine advice this obeys: *"ask the preview verb every frame
/// rather than catching the error afterwards — a verb with no preflight makes
/// the UI find out by pressing."*
#[must_use]
pub fn rows(doc: &OpenDoc, selection: &SelectionState, pick: NodePick) -> Rows {
    let Some((id, _, _)) = super::geometry(doc, selection) else {
        return Rows::default();
    };
    let session = &doc.session;
    let state = |edit: VertexEdit| match session.reshape_annotation_preview(id, edit) {
        Ok(_) => RowState::Live,
        // The ONE temporary refusal: a closed shape at three corners, an open
        // one at two. Draw another corner and the row comes back, which is what
        // makes greying-with-a-reason correct here and wrong everywhere else in
        // this module.
        Err(EditError::ReshapeWouldBreachVertexFloor { .. }) => RowState::Greyed,
        // `GeometryNotReshapable` for a `/Line`, an `/Ink`, a `/Square`, a
        // `/Circle` or a text markup; `AnnotationLocked` for a shape the FILE
        // forbids changing; `AnnotationIsCeDimension` for the shape
        // `canvas::dimdrag` owns. None of them stops being true while the
        // operator looks at the menu, so none of them is greyed.
        Err(_) => RowState::Absent,
    };
    match pick {
        NodePick::Node(index) => Rows {
            insert: RowState::Absent,
            remove: state(VertexEdit::Remove { index }),
        },
        NodePick::Segment { after, at } => Rows {
            insert: state(VertexEdit::Insert { after, at }),
            remove: RowState::Absent,
        },
        // ★ Both absent, and this is the common case rather than an edge one: a
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
///
/// Called by [`crate::canvas::menus`] on the frame of the secondary click, so a
/// driven check can read *which corner the menu thinks it is about* without
/// pressing anything — which is the only way to tell "the row was greyed
/// correctly" from "the row was greyed because the pick was wrong".
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
///
/// # ★★ Why the guard is re-asked here and not trusted from the menu
///
/// The row was drawn from [`rows`] on some earlier frame, and everything
/// between then and now is a frame in which the document could have changed —
/// an undo, a background reflow, another surface's edit. Re-asking
/// [`rows`] with the parked pick costs one annotation walk on a press and
/// removes the whole class of *"the menu was right when it was drawn"*, which
/// is the class `MenuHost::with_condition`'s own header is about in the
/// opposite direction.
///
/// ⇒ So a row that has stopped being live raises **nothing**, and the trace
/// says which. It does not raise a refusal sentence: the operator pressed a row
/// that the engine now declines, which is the state R9 grey already describes,
/// and a status line arriving after a menu closed would be a second explanation
/// for a first-order rarity.
#[must_use]
pub fn action_for(
    ctx: &egui::Context,
    doc: &OpenDoc,
    selection: &SelectionState,
    insert: bool,
) -> Option<Action> {
    let (id, _, _) = super::geometry(doc, selection)?;
    let pick = parked(ctx);
    let states = rows(doc, selection, pick);
    let action = match (insert, pick) {
        (true, NodePick::Segment { after, at }) if states.insert.enabled() => {
            AnnotAction::InsertNode { id, after, at }
        }
        (false, NodePick::Node(index)) if states.remove.enabled() => {
            AnnotAction::RemoveNode { id, index }
        }
        _ => {
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
    ///
    /// ★ Falsified: returning `t = 0.0` instead of the projection makes the
    /// midpoint assertion fail on both coordinates, and returning the pointer
    /// itself makes the y assertion fail by the 4 pt offset below.
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
    /// ★ Falsified: dropping `.clamp(0.0, 1.0)` gives `t = 2.0` and a distance
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
    /// ★ Falsified: removing the `length_squared` guard makes `t` NaN, and
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
    ///
    /// ★ Falsified: defining `shown` as `matches!(self, Self::Live)` makes the
    /// greyed assertion fail, which is the exact regression — an unavailable
    /// row disappearing instead of explaining itself.
    #[test]
    fn a_greyed_row_is_drawn_and_an_absent_one_is_not() {
        assert!(RowState::Live.shown() && RowState::Live.enabled());
        assert!(RowState::Greyed.shown() && !RowState::Greyed.enabled());
        assert!(!RowState::Absent.shown() && !RowState::Absent.enabled());
    }

    /// With no shape picked, both rows are absent — which is what lets the rest
    /// of the markup menu open over a shape's interior.
    ///
    /// ★ Falsified: defaulting `Rows` to `Live` makes both assertions fail.
    #[test]
    fn no_pick_draws_neither_node_row() {
        let rows = Rows::default();
        assert!(!rows.insert.shown());
        assert!(!rows.remove.shown());
    }

    /// The pick's default is *nowhere near the shape*, so a frame before any
    /// right-click cannot be read as "node 0".
    ///
    /// ★ Falsified: making `Node(0)` the default makes this fail, and would
    /// have offered *Remove this point* on the first vertex of every shape
    /// before the operator had pointed at anything.
    #[test]
    fn the_default_pick_names_no_node() {
        assert_eq!(NodePick::default(), NodePick::Elsewhere);
    }
}
