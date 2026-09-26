//! # `handles` — eight grips plus move, and the cursor over each
//!
//! `GUI_ROADMAP.md` Phase 1.3: *"Eight handles plus move, per the convention
//! every drawing tool shares. Cursor changes over a handle, over a movable
//! object, over the canvas."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/handles.md`.
//!
//! ## conventions: handles
//!
//! Corpus: `ui-conventions/handles.md`.
//!
//! - H1 appear-on-selection: the eight grips are drawn when something is
//!   selected at the Object rung, before any drag.
//! - H2 standard-set: **complete as of 2026-08-20** — eight resize grips, the
//!   body, and a rotate handle offset above the top edge on a stem, which is
//!   the arrangement PowerPoint, Illustrator, Figma, Inkscape, Visio and Konva
//!   all present. This row read *"GAP: no rotate handle, because no engine verb
//!   rotates anything"*, and it ended *"when that lands, the handle above the
//!   top edge is the shape to build, not a menu item."* `Pass 113.0` landed it
//!   and that is the shape that was built.
//! - H3 screen-sized: `GRIP_SIZE_PX` is in points and does not scale with zoom,
//!   so a corner on a plan at 20 % is as grabbable as one at 400 %.
//! - H4 target-not-smaller: `GRIP_GRAB_SLACK_PX` expands the live area beyond
//!   the drawn square. Never the reverse.
//! - H5 grips-outrank-body: checked first, because corner grips sit ON the
//!   box's edge and half of each square overlaps the interior — if the body won,
//!   each would be a half-size target on its outer half only.
//! - H6 cursor-names-it: `Grip::cursor` gives each grip its diagonal or axis
//!   arrow and the body a move cursor.
//! - H7 painted-equals-grabbable: the same predicate decides both. **This row
//!   exists because it failed on 2026-08-20**: a dimension's vertex handles were
//!   painted from the selection and hit-tested behind a capability the mode did
//!   not have, so they were visible and untouchable in the very mode that
//!   authors dimensions.
//! - H8 published: `SELECTION_OUTLINE_REGION` publishes the box every grip is
//!   derived from, and `dimdrag::VERTEX_REGION` publishes each vertex handle
//!   indexed — so a driven check aims at what the application says rather than
//!   at a guess.
//! - H9 vertex-editing: a perimeter ce dimension's corners are handles and drag
//!   to reshape. **GAP: no right-click to add or remove a point**, though both
//!   engine verbs and the preflight that greys the menu item already exist.

use egui::{CursorIcon, Pos2, Rect, Vec2};

/// The side length of a grip square, in screen points.
pub const GRIP_SIZE_PX: f32 = 8.0;

/// Extra slack, in screen points, around a grip's drawn square when
/// hit-testing it.
pub const GRIP_GRAB_SLACK_PX: f32 = 2.0;

/// The smallest box, in screen points, that gets mid-edge grips on an axis.
pub const MIN_MID_GRIP_EXTENT_PX: f32 = GRIP_SIZE_PX * 3.0;

/// The smallest box, **across** an axis, that still gets that axis's
/// mid-edge grips — the rule that stops a grip swallowing the body.
pub const MIN_BODY_STRIP_PX: f32 = GRIP_SIZE_PX + 2.0 * (GRIP_SIZE_PX / 2.0 + GRIP_GRAB_SLACK_PX);

/// One grip on the selection's bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Grip {
    /// Top-left corner.
    NorthWest,
    /// Top edge, centred.
    North,
    /// Top-right corner.
    NorthEast,
    /// Right edge, centred.
    East,
    /// Bottom-right corner.
    SouthEast,
    /// Bottom edge, centred.
    South,
    /// Bottom-left corner.
    SouthWest,
    /// Left edge, centred.
    West,
    /// The body of the selection.
    ///
    /// Not drawn as a square: the whole interior *is* the target, which is
    /// what every drawing tool does and what an operator will try first.
    Move,
    /// **The rotate handle**, offset above the top edge on a stem.
    ///
    /// # Why above, and why on a stem
    ///
    /// Because that is where PowerPoint, Illustrator, Figma, Inkscape, Visio
    /// and Konva's `Transformer` all put it, and the standing tie-breaker for
    /// anything an operator compares against the tools they already use is to
    /// behave the way those tools behave.
    ///
    /// The **offset** is what makes it reachable on a selection whose top edge
    /// is already crowded by the north grip; the **stem** is what says the two
    /// belong together, without which the handle reads as an unrelated dot
    /// floating over the page.
    ///
    /// # It is drawn as a CIRCLE
    ///
    /// Every square on this canvas resizes. A shape that resized in one place
    /// and rotated in another would be a private convention the operator has to
    /// learn, which is `handles.md` H2's stated failure mode — *"the operator
    /// has to learn a control they already knew."*
    ///
    /// # And it is not a resize
    ///
    Rotate,
}

impl Grip {
    /// The eight resize grips, clockwise from the top-left.
    ///
    /// Clockwise so the order is the one a reader traces with a finger, which
    /// makes an off-by-one in a table obvious rather than plausible.
    pub const RESIZE: [Self; 8] = [
        Self::NorthWest,
        Self::North,
        Self::NorthEast,
        Self::East,
        Self::SouthEast,
        Self::South,
        Self::SouthWest,
        Self::West,
    ];

    /// The cursor shown while the pointer is over this grip.
    #[must_use]
    pub fn cursor(self) -> CursorIcon {
        match self {
            Self::NorthWest | Self::SouthEast => CursorIcon::ResizeNwSe,
            Self::NorthEast | Self::SouthWest => CursorIcon::ResizeNeSw,
            Self::North | Self::South => CursorIcon::ResizeVertical,
            Self::East | Self::West => CursorIcon::ResizeHorizontal,
            Self::Move => CursorIcon::Move,
            // egui 0.35 has no rotate cursor, so this is the nearest honest
            // thing rather than the right thing: `Grab` says *"this is a handle
            // you take hold of"*, which is true, where `Default` would say
            // nothing and `Crosshair` would suggest precision placement.
            // Recorded as a compromise rather than a choice — `handles.md` H6
            // asks the cursor to NAME the gesture, and this one only hints at
            // it. A custom cursor is a texture and an atlas entry, which is a
            // real piece of work for one glyph.
            Self::Rotate => CursorIcon::Grab,
        }
    }

    /// Whether this grip resizes rather than moves or rotates.
    #[must_use]
    pub fn is_resize(self) -> bool {
        matches!(
            self,
            Self::NorthWest
                | Self::North
                | Self::NorthEast
                | Self::East
                | Self::SouthEast
                | Self::South
                | Self::SouthWest
                | Self::West
        )
    }

    /// Where this grip's centre sits on a screen-space bounding box.
    #[must_use]
    pub fn anchor(self, bounds: Rect) -> Pos2 {
        let mid = bounds.center();
        match self {
            Self::NorthWest => bounds.left_top(),
            Self::North => Pos2::new(mid.x, bounds.top()),
            Self::NorthEast => bounds.right_top(),
            Self::East => Pos2::new(bounds.right(), mid.y),
            Self::SouthEast => bounds.right_bottom(),
            Self::South => Pos2::new(mid.x, bounds.bottom()),
            Self::SouthWest => bounds.left_bottom(),
            Self::West => Pos2::new(bounds.left(), mid.y),
            Self::Move => mid,
            // Above the top edge, centred, by the stem's length. The one grip
            // whose centre is OUTSIDE the box, which is what the offset is for.
            Self::Rotate => Pos2::new(mid.x, bounds.top() - ROTATE_STEM_PX),
        }
    }

    /// **The corner a drag on this grip must leave EXACTLY WHERE IT IS.**
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::NorthWest => Self::SouthEast,
            Self::North => Self::South,
            Self::NorthEast => Self::SouthWest,
            Self::East => Self::West,
            Self::SouthEast => Self::NorthWest,
            Self::South => Self::North,
            Self::SouthWest => Self::NorthEast,
            Self::West => Self::East,
            Self::Move => Self::Move,
            Self::Rotate => Self::Rotate,
        }
    }

    #[must_use]
    pub fn pivot(self, bounds: Rect) -> Pos2 {
        let mid = bounds.center();
        match self {
            Self::NorthWest => bounds.right_bottom(),
            Self::North => Pos2::new(mid.x, bounds.bottom()),
            Self::NorthEast => bounds.left_bottom(),
            Self::East => Pos2::new(bounds.left(), mid.y),
            Self::SouthEast => bounds.left_top(),
            Self::South => Pos2::new(mid.x, bounds.top()),
            Self::SouthWest => bounds.right_top(),
            Self::West => Pos2::new(bounds.right(), mid.y),
            Self::Move => mid,
            // The CENTRE, and for this grip it is the real answer rather than
            // a harmless one. A rotation turns the selection about its middle —
            // which is what every drawing program does, and the only choice that
            // leaves the object where the operator can still see it. The eight
            // resize grips pivot about an opposite corner because a resize has
            // an edge that must not move; a rotation has no such edge.
            Self::Rotate => mid,
        }
    }
}

/// How far above the selection box the rotate handle's centre sits, in points.
pub const ROTATE_STEM_PX: f32 = 20.0;

/// **The rotate handle's square**, above the top edge on its stem.
#[must_use]
pub fn rotate_rect(bounds: Rect) -> Rect {
    rotate_rect_in(GripFrame::Upright(bounds))
}

/// [`rotate_rect`] in an arbitrary [`GripFrame`].
#[must_use]
pub fn rotate_rect_in(frame: GripFrame) -> Rect {
    let bounds = match frame {
        GripFrame::Turned(_) => {
            return Rect::from_center_size(
                frame.pushed().anchor(Grip::Rotate),
                Vec2::splat(GRIP_SIZE_PX),
            );
        }
        GripFrame::Upright(bounds) => bounds,
    };
    // Anchored to the pushed box for the same reason the eight scale grips are:
    // on a tiny selection the rotate handle would otherwise sit *inside* the
    // body it is supposed to hover above. See [`grip_bounds`].
    Rect::from_center_size(
        Grip::Rotate.anchor(grip_bounds(bounds)),
        Vec2::splat(GRIP_SIZE_PX),
    )
}

/// The box the **grips** are anchored to, which is the selection's own box
/// grown outward when the selection is too small to hold them.
///
/// # The defect this closes, in the operator's own words
///
/// He asked, on 2026-09-04: *"zoom in on the atoms of the banana pdf file and
/// see what happens when you try to draw a box around a molecule and move it,
/// or select the ion and move it."* The answer, measured by driving the binary
/// on that fixture: **an object smaller than 12 pt on screen could not be moved
/// at all.** Every press landed in a grip, the drag was routed to the resize
/// machinery, and the engine refused it by name — `resize-declined
/// reason=Degenerate`. The banana's cells are 0.85 pt across, and the fixture's
/// own text says reading their labels takes about 12,000 %.
///
/// Two constants, each correct in isolation, made it:
///
/// - every corner grip reaches `GRIP_SIZE_PX / 2 + GRIP_GRAB_SLACK_PX` = **6 pt**
///   into the box it is drawn on, and there are two of them per axis;
/// - `pdfcer_gui::canvas::overlay::MIN_OUTLINE_EXTENT_PX` **floors the drawn box at
///   6 pt**, so an object with the least body to spare is floored to a size at
///   which it has none.
///
/// ⇒ The objects that most needed a body to grab were guaranteed not to have
/// one. Above 12 pt the body was a *hole* rather than a region: at 13.4 × 12.5 pt
/// the four corners leave a 1.4 × 0.5 pt gap that a harness hits by computing
/// the exact centre and a hand does not.
///
/// # Why outward, and why this is not an invention
///
/// The conventional answer across the whole product class is the same one:
/// **when the box is too small to hold its handles, the handles go outside the
/// box.** Inkscape draws its scale arrows outside the bounding box
/// unconditionally; Figma moves a small frame's handles out; Illustrator's
/// move gesture aims at the path rather than the bounding-box interior. The
/// convergence of the product class *is* the specification here — an invented
/// interaction would be a defect even if it worked, because the operator
/// already knows this one from every other drawing program on his machine.
///
/// # The rule
///
/// Per axis, grow by exactly enough to reach [`MIN_BODY_STRIP_PX`] and no more:
///
/// ```text
/// push = max(0, (MIN_BODY_STRIP_PX - extent) / 2)     on each side
/// ```
///
/// **Above the threshold the push is exactly zero and every grip lands byte
/// for byte where it did before.** That property is what makes this safe to
/// apply unconditionally: there is no second layout to keep in step, no mode to
/// be in, and no zoom at which behaviour changes discontinuously — the push
/// grows smoothly from 0 as the box shrinks through 20 pt.
///
/// # What this costs, stated rather than discovered later
///
/// A pushed grip can overlap a **neighbouring** object. On a dense drawing at
/// low zoom that means the grips of one tiny object may sit over another one.
/// That is the trade every program in the class makes, and it is the right way
/// round: the alternative is an object that cannot be moved at all, which is
/// the defect being fixed. It also cannot mislead — grips are drawn as the
/// cursor's own furniture, never as content, so Rule 4 is untouched.
///
/// ⚠ This deliberately does **not** grow the body. [`grip_at`] still tests
/// `bounds.contains(pointer)` for [`Grip::Move`], so the region that means
/// "drag this object" is exactly the object's own drawn outline. Growing that
/// too would make a 0.85 pt cell claim 20 pt of the canvas and steal presses
/// aimed at its neighbours.
#[must_use]
pub fn grip_bounds(bounds: Rect) -> Rect {
    let push = Vec2::new(
        ((MIN_BODY_STRIP_PX - bounds.width()) / 2.0).max(0.0),
        ((MIN_BODY_STRIP_PX - bounds.height()) / 2.0).max(0.0),
    );
    bounds.expand2(push)
}

/// **The frame the grips are laid out in** — an upright box, or a turned one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GripFrame {
    /// An axis-aligned screen box: every content selection, every form field,
    /// and every annotation whose appearance is not turned.
    Upright(Rect),
    /// A turned annotation's four **placed corners** in screen space, in the
    /// artwork's own frame: `[lower-left, lower-right, upper-right, upper-left]`
    /// as `pdfcer_gui::canvas::annotquad::OrientedBox::corners` orders them.
    ///
    /// "Lower" and "upper" name the *artwork's* edges, not the page's, which
    /// is the whole content of this variant: after a 100° turn the artwork's
    /// lower-left corner is at the top of the screen, and a grip the operator
    /// grabs there must be the one that belongs to that corner of the mark.
    Turned([Pos2; 4]),
}

impl GripFrame {
    /// The upright box that bounds this frame — what a caller needs when it
    /// genuinely wants an axis-aligned extent (a published diagnostic region, a
    /// containment pre-filter, a degenerate-size test).
    #[must_use]
    pub fn bounds(self) -> Rect {
        match self {
            Self::Upright(r) => r,
            Self::Turned(q) => q
                .iter()
                .skip(1)
                .fold(Rect::from_two_pos(q[0], q[0]), |acc, p| {
                    acc.union(Rect::from_two_pos(*p, *p))
                }),
        }
    }

    /// This frame's corners rounded to whole points, for a diagnostic line.
    #[must_use]
    pub fn corners_for_trace(self) -> [(i32, i32); 4] {
        self.corners()
            .map(|p| (p.x.round() as i32, p.y.round() as i32))
    }

    /// This frame's corners, always four, in the same order either way:
    /// `[SW, SE, NE, NW]` **of the frame**.
    #[must_use]
    fn corners(self) -> [Pos2; 4] {
        match self {
            Self::Upright(r) => [
                r.left_bottom(),
                r.right_bottom(),
                r.right_top(),
                r.left_top(),
            ],
            Self::Turned(q) => q,
        }
    }

    /// This frame grown outward until it can hold its own grips — the turned
    /// counterpart of [`grip_bounds`], and it must exist for the same reason.
    #[must_use]
    fn pushed(self) -> Self {
        let Self::Turned(q) = self else {
            return self;
        };
        let (u, v) = ((q[1] - q[0]), (q[3] - q[0]));
        let (w, h) = (u.length(), v.length());
        if !w.is_finite() || !h.is_finite() || w <= f32::EPSILON || h <= f32::EPSILON {
            return self;
        }
        let (u, v) = (u / w, v / h);
        let half_w = w / 2.0 + ((MIN_BODY_STRIP_PX - w) / 2.0).max(0.0);
        let half_v = h / 2.0 + ((MIN_BODY_STRIP_PX - h) / 2.0).max(0.0);
        let centre = q[0] + (q[2] - q[0]) / 2.0;
        Self::Turned([
            centre - u * half_w - v * half_v,
            centre + u * half_w - v * half_v,
            centre + u * half_w + v * half_v,
            centre - u * half_w + v * half_v,
        ])
    }

    /// Where `grip`'s centre sits in this frame.
    #[must_use]
    pub fn anchor(self, grip: Grip) -> Pos2 {
        if let Self::Upright(r) = self {
            return grip.anchor(r);
        }
        let [sw, se, ne, nw] = self.corners();
        let mid = |a: Pos2, b: Pos2| a + (b - a) / 2.0;
        let centre = mid(mid(sw, ne), mid(se, nw));
        match grip {
            Grip::SouthWest => sw,
            Grip::South => mid(sw, se),
            Grip::SouthEast => se,
            Grip::East => mid(se, ne),
            Grip::NorthEast => ne,
            Grip::North => mid(ne, nw),
            Grip::NorthWest => nw,
            Grip::West => mid(nw, sw),
            Grip::Move => centre,
            Grip::Rotate => {
                let top = mid(ne, nw);
                // Outward from the centre through the top edge's midpoint. On a
                // degenerate frame (every corner coincident) the direction is
                // zero-length and `normalized` answers zero, so the handle lands
                // on the box rather than at infinity — the same "degenerate is
                // survivable, NaN is not" posture `screen_vec_to_page` takes.
                top + (top - centre).normalized() * ROTATE_STEM_PX
            }
        }
    }

    /// The point a drag on `grip` must leave exactly where it is — the mirror of
    /// [`Self::anchor`], and the same fact as [`Grip::pivot`] in a turned frame.
    #[must_use]
    pub fn pivot(self, grip: Grip) -> Pos2 {
        if let Self::Upright(r) = self {
            return grip.pivot(r);
        }
        match grip {
            Grip::Move | Grip::Rotate => self.anchor(Grip::Move),
            other => self.anchor(other.opposite()),
        }
    }
}

/// The grips to draw for a screen-space selection box, with their squares.
#[must_use]
pub fn grip_rects(bounds: Rect) -> Vec<(Grip, Rect)> {
    grip_rects_in(GripFrame::Upright(bounds))
}

/// [`grip_rects`] in an arbitrary [`GripFrame`] — the general form, and what
/// every painter and hit test now calls.
#[must_use]
pub fn grip_rects_in(frame: GripFrame) -> Vec<(Grip, Rect)> {
    let bounds = match frame.pushed() {
        GripFrame::Turned(quad) => {
            let frame = GripFrame::Turned(quad);
            let edge = |a: Pos2, b: Pos2| (b - a).length();
            let wide = edge(quad[0], quad[1]) >= MIN_MID_GRIP_EXTENT_PX;
            let tall = edge(quad[1], quad[2]) >= MIN_MID_GRIP_EXTENT_PX;
            return Grip::RESIZE
                .into_iter()
                .filter(|g| match g {
                    Grip::North | Grip::South => wide,
                    Grip::East | Grip::West => tall,
                    _ => true,
                })
                .map(|g| {
                    (
                        g,
                        Rect::from_center_size(frame.anchor(g), Vec2::splat(GRIP_SIZE_PX)),
                    )
                })
                .collect();
        }
        // `pushed` is the identity on an upright frame, so this is the same
        // `bounds` the caller handed in — matched rather than re-destructured
        // with an `unreachable!`, because a panic macro in a shipped painter is
        // a crash where a compiler-checked match is nothing.
        GripFrame::Upright(bounds) => bounds,
    };
    // Everything below anchors to the PUSHED box, never to `bounds`.
    //
    // [`grip_bounds`] grows the anchor box outward when the selection is too
    // small to hold its own grips, which is what makes the body of a 0.85 pt
    // cell reachable at all. Above [`MIN_BODY_STRIP_PX`] the push is exactly
    // zero and this is the same computation it always was.
    let anchors = grip_bounds(bounds);
    debug_assert!(
        anchors.width() + f32::EPSILON >= MIN_BODY_STRIP_PX
            && anchors.height() + f32::EPSILON >= MIN_BODY_STRIP_PX,
        // ui-text-exempt: a debug_assert message. It names an internal invariant
        // for a developer and cannot reach a release build, let alone the operator.
        "grip_bounds must guarantee a body strip on both axes; got {anchors:?}"
    );

    // Only ONE condition per mid-edge grip now, and it is about piling.
    //
    // There used to be two. The second — *"does the perpendicular axis have a
    // body left after this grip eats 6 pt of it?"* — was the 2026-09-04 fix for
    // a 160 × 20 pt form field whose centre sat inside its own North grip. It is
    // gone because [`grip_bounds`] now makes it **unfalsifiable**: the pushed box
    // always has a body strip, so the condition could never be false and a
    // condition that cannot fail is not a guard, it is decoration that reads
    // like one. The `debug_assert` above is what took over its job, and it names
    // the invariant instead of silently depending on it.
    //
    // The piling condition stays, and stays measured against the PUSHED box:
    // whether a mid-edge grip lands on top of its corner neighbours is a
    // question about the spacing it is actually drawn at.
    let wide = anchors.width() >= MIN_MID_GRIP_EXTENT_PX;
    let tall = anchors.height() >= MIN_MID_GRIP_EXTENT_PX;
    Grip::RESIZE
        .into_iter()
        .filter(|g| match g {
            Grip::North | Grip::South => wide,
            Grip::East | Grip::West => tall,
            _ => true,
        })
        .map(|g| {
            (
                g,
                Rect::from_center_size(g.anchor(anchors), Vec2::splat(GRIP_SIZE_PX)),
            )
        })
        .collect()
}

/// Which grip a screen-space `pointer` is over, or `None` if it is over
/// neither a grip nor the selection's body.
///
/// # Resize grips win over the body, and that is not arbitrary
///
/// The corner grips sit *on* the box's edge, so half of each square overlaps
/// the interior. If the body won, the corner grips would be half-size targets
/// on their outer halves only — the operator would aim at a square and get a
/// move. Checking the grips first makes the drawn square and the live target
/// the same shape, which is the same argument that puts Bézier handles ahead
/// of the nodes they belong to.
#[must_use]
/// Which grips a selection offers, because it has a verb behind each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GripSet {
    /// The eight scale grips.
    pub resize: bool,
    /// The rotate handle above the top edge.
    ///
    /// Deliberately not collapsed into [`Self::resize`]: *"can this be
    /// scaled"* and *"can this be turned"* are two questions about the engine's
    /// verb list, and a shell that inferred one from the other would offer
    /// rotation to the next kind that gains a resize verb without anybody
    /// deciding.
    ///
    /// That caution paid on the day it was written. Until 2026-08-28 this
    /// field's doc said *"never true without `resize` today"* — and
    /// [`GripSet::rotate_only`] now exists, because a ce dimension turns and
    /// does not scale. A struct that had collapsed the two would have had to be
    /// un-collapsed to ship that, and the intervening builds would have offered
    /// eight scale grips around a dimension whose extent is its measurement.
    pub rotate: bool,
}

mod name;

impl GripSet {
    /// Everything — page **content** at the Object rung, and a **markup
    /// annotation**, which gained the second half on 2026-08-28.
    pub const fn all() -> Self {
        Self {
            resize: true,
            rotate: true,
        }
    }

    /// The eight scale grips and no rotate handle — a **form field's box**.
    pub const fn scale_only() -> Self {
        Self {
            resize: true,
            rotate: false,
        }
    }

    /// **Neither — a mark that can be MOVED and nothing else.**
    pub const fn move_only() -> Self {
        Self {
            resize: false,
            rotate: false,
        }
    }

    /// **The rotate handle alone** — a selected **ce dimension**.
    pub const fn rotate_only() -> Self {
        Self {
            resize: false,
            rotate: true,
        }
    }
}

pub fn grip_at(bounds: Rect, pointer: Pos2, offer: GripSet) -> Option<Grip> {
    grip_at_in(GripFrame::Upright(bounds), pointer, offer)
}

/// [`grip_at`] in an arbitrary [`GripFrame`] — the general form, and the one
/// the canvas calls.
pub fn grip_at_in(frame: GripFrame, pointer: Pos2, offer: GripSet) -> Option<Grip> {
    let bounds = frame.bounds();
    if offer.rotate {
        // The rotate handle FIRST, and the reason is H7 rather than
        // geometry: it sits outside the box, so it collides with nothing and
        // the order could not matter for correctness. It is first because
        // **the same predicate decides painting and hit-testing**, and that
        // predicate is `GripSet` — so a handle painted here is grabbable
        // here, in one place, with nothing in between for a future edit to slip
        // a capability check into.
        //
        if rotate_rect_in(frame)
            .expand(GRIP_GRAB_SLACK_PX)
            .contains(pointer)
        {
            return Some(Grip::Rotate);
        }
    }
    // The eight scale grips are gated separately from the rotate handle
    // above, which is the whole reason `GripSet` has two fields. An annotation
    // offers these and not that one.
    if offer.resize {
        for (grip, rect) in grip_rects_in(frame) {
            if rect.expand(GRIP_GRAB_SLACK_PX).contains(pointer) {
                return Some(grip);
            }
        }
    }
    bounds.contains(pointer).then_some(Grip::Move)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn box_of(w: f32, h: f32) -> Rect {
        Rect::from_min_size(Pos2::new(100.0, 200.0), Vec2::new(w, h))
    }

    /// A comfortable selection offers all eight grips, and each one sits
    /// where its name says.
    #[test]
    fn a_comfortable_box_offers_all_eight_grips_in_the_right_places() {
        let b = box_of(200.0, 100.0);
        let grips = grip_rects(b);
        assert_eq!(grips.len(), 8);

        let at = |g: Grip| {
            grips
                .iter()
                .find(|(k, _)| *k == g)
                .map(|(_, r)| r.center())
                .expect("grip present")
        };
        assert_eq!(at(Grip::NorthWest), b.left_top());
        assert_eq!(at(Grip::SouthEast), b.right_bottom());
        assert_eq!(at(Grip::North), Pos2::new(b.center().x, b.top()));
        assert_eq!(at(Grip::West), Pos2::new(b.left(), b.center().y));
    }

    /// A box too narrow for a mid-edge grip drops it rather than piling it
    /// on top of the corners — but keeps every corner, so nothing becomes
    /// unreachable.
    #[test]
    fn a_narrow_box_drops_its_mid_edge_grips_and_keeps_its_corners() {
        let narrow = box_of(22.0, 200.0);
        let kinds: Vec<Grip> = grip_rects(narrow).into_iter().map(|(g, _)| g).collect();
        assert!(!kinds.contains(&Grip::North));
        assert!(!kinds.contains(&Grip::South));
        assert!(kinds.contains(&Grip::East), "the tall axis keeps its grips");
        for corner in [
            Grip::NorthWest,
            Grip::NorthEast,
            Grip::SouthEast,
            Grip::SouthWest,
        ] {
            assert!(kinds.contains(&corner), "{corner:?} must always be offered");
        }

        // …and symmetrically for a short one. 22 rather than 10 for the same
        // reason the fixture above changed: a 10 px-tall box cannot hold its
        // North and South grips either, and this assertion is about the EAST
        // grip being dropped for piling, not about the body rule.
        let short = box_of(200.0, 22.0);
        let kinds: Vec<Grip> = grip_rects(short).into_iter().map(|(g, _)| g).collect();
        assert!(!kinds.contains(&Grip::East));
        assert!(kinds.contains(&Grip::North));
    }

    /// A grip wins over the body where they overlap, so the drawn square and
    /// the live target are the same shape.
    #[test]
    fn a_grip_wins_over_the_body_where_they_overlap() {
        let b = box_of(200.0, 100.0);
        // Just inside the top-left corner — inside the body, and inside the
        // NW grip's square.
        assert_eq!(
            grip_at(b, b.left_top() + Vec2::splat(2.0), GripSet::all()),
            Some(Grip::NorthWest)
        );
        // Well inside: the body.
        assert_eq!(grip_at(b, b.center(), GripSet::all()), Some(Grip::Move));
        // Well outside: nothing.
        assert_eq!(
            grip_at(b, b.left_top() - Vec2::splat(60.0), GripSet::all()),
            None
        );
    }

    /// Every grip has a cursor, opposite corners share an axis cursor, and
    /// the move grip is the only one that is not a resize.
    #[test]
    fn opposite_corners_share_a_resize_axis_and_move_stands_apart() {
        assert_eq!(Grip::NorthWest.cursor(), Grip::SouthEast.cursor());
        assert_eq!(Grip::NorthEast.cursor(), Grip::SouthWest.cursor());
        assert_eq!(Grip::North.cursor(), Grip::South.cursor());
        assert_eq!(Grip::East.cursor(), Grip::West.cursor());
        assert_ne!(Grip::NorthWest.cursor(), Grip::NorthEast.cursor());
        assert_eq!(Grip::Move.cursor(), CursorIcon::Move);
        assert!(!Grip::Move.is_resize());
        assert!(Grip::RESIZE.iter().all(|g| g.is_resize()));
        assert_eq!(Grip::RESIZE.len(), 8, "eight grips, plus move");
    }

    /// The grips are a fixed number of SCREEN points, so they do not change
    /// size with the zoom — the one place screen space is used inside the
    /// selection layer, and the property that makes it correct.
    #[test]
    fn grips_are_the_same_size_however_big_the_selection_is() {
        for (w, h) in [(40.0, 40.0), (2_000.0, 1_400.0), (60.0, 5_000.0)] {
            for (_, r) in grip_rects(box_of(w, h)) {
                assert!((r.width() - GRIP_SIZE_PX).abs() < f32::EPSILON);
                assert!((r.height() - GRIP_SIZE_PX).abs() < f32::EPSILON);
            }
        }
    }

    /// **`pivot` is the OPPOSITE of `anchor`, for every resize grip.**
    #[test]
    fn every_resize_grip_pivots_about_the_opposite_point() {
        let b = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 60.0));
        let mid = b.center();
        for g in Grip::RESIZE {
            let a = g.anchor(b);
            let p = g.pivot(b);
            // Reflecting the anchor through the box centre gives the pivot, on
            // every axis the grip actually scales. A mid-edge grip's other axis
            // is the centre in both, so the relation holds on both axes for all
            // eight without a special case.
            assert!(
                (a.x + p.x - 2.0 * mid.x).abs() < 1e-4,
                "{g:?}: anchor.x={} pivot.x={} do not straddle the centre",
                a.x,
                p.x
            );
            assert!(
                (a.y + p.y - 2.0 * mid.y).abs() < 1e-4,
                "{g:?}: anchor.y={} pivot.y={} do not straddle the centre",
                a.y,
                p.y
            );
            assert_ne!(
                a, p,
                "{g:?} pivots about itself, so a drag would scale about the hand"
            );
        }
    }

    /// **An inner rung offers `Move` and none of the eight.**
    #[test]
    fn an_inner_rung_offers_move_and_no_scale_handles() {
        let b = box_of(200.0, 100.0);
        let corner = b.min;
        assert_eq!(
            grip_at(b, corner, GripSet::all()),
            Some(Grip::NorthWest),
            "the Object rung still offers all eight"
        );
        assert_eq!(
            grip_at(b, corner, GripSet::default()),
            Some(Grip::Move),
            "an inner rung must hand the corner press to the MOVE gesture"
        );
        // And the interior is a move either way — that is how a move drag is
        // recognised at every rung, so withholding the eight must not withhold
        // it.
        assert_eq!(grip_at(b, b.center(), GripSet::default()), Some(Grip::Move));
        assert_eq!(grip_at(b, b.center(), GripSet::all()), Some(Grip::Move));
        // Outside is still nothing.
        assert_eq!(
            grip_at(
                b,
                Pos2::new(b.max.x + 50.0, b.max.y + 50.0),
                GripSet::default()
            ),
            None
        );
    }

    /// **A ce dimension's set: the ninth handle and NONE of the eight.**
    #[test]
    fn a_rotate_only_set_offers_the_handle_and_none_of_the_eight() {
        let b = box_of(200.0, 100.0);
        let handle = rotate_rect(b).center();
        assert_eq!(
            grip_at(b, handle, GripSet::rotate_only()),
            Some(Grip::Rotate),
            "the ninth handle is the whole point of this set"
        );
        assert_eq!(
            grip_at(b, b.min, GripSet::rotate_only()),
            Some(Grip::Move),
            "a corner press must NOT become a resize: a ce dimension has no scale verb, and \
             offering one would be a grip that the engine declines by name"
        );
        assert_eq!(
            grip_at(b, b.center(), GripSet::rotate_only()),
            Some(Grip::Move),
            "the body still moves — withholding the eight must not withhold the drag that \
             repositions the dimension"
        );
        // …and the same press on the handle finds nothing when the set does not
        // offer it, which is the widget's case. One predicate, both answers.
        assert_eq!(grip_at(b, handle, GripSet::scale_only()), None);
    }
}
