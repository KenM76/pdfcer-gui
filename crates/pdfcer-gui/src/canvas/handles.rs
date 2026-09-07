//! # `canvas::handles` — eight grips plus move, and the cursor over each
//!
//! `GUI_ROADMAP.md` Phase 1.3: *"Eight handles plus move, per the convention
//! every drawing tool shares. Cursor changes over a handle, over a movable
//! object, over the canvas."*
//!
//! ## Rule 4 says these are welcome, and says exactly why
//!
//! `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`, fourth clause of
//! the disclosure rule:
//!
//! > **A pre-commit affordance is not content marking.** A snap indicator, a
//! > hover highlight, a rubber-band, a selection handle — these are the
//! > *cursor*; they describe what is about to happen and they are welcome.
//! > What is forbidden is styling content that has **already been applied**
//! > as though it were pending.
//!
//! So the grips are drawn, and nothing else is. No badge, no tint, no dashed
//! "provisional" layer over content, nothing that would make a screenshot of
//! the editing canvas differ from a screenshot of the same document saved and
//! reopened. The grips vanish with the selection because they are the
//! cursor's statement about the selection, not a property of the page.
//!
//! ## ★ These are SCREEN-space rects, deliberately, and it is the one place
//!
//! Everything else in `canvas/` past [`crate::canvas::mapping`] is page
//! space. A grip is the exception and must be: it is a **fixed number of
//! screen pixels**, because it is something the operator has to hit with a
//! mouse, and a grip sized in page units would be a 3-pixel speck at fit-page
//! and a slab the size of the object at 800%. It sits on the *output* side of
//! the boundary — the selection's bounds are converted to screen once, by
//! [`crate::canvas::mapping::PageMapping::rect_to_screen`], and the grips are
//! laid out on the result.
//!
//! ## What a grip drag does today, stated so it is not mistaken for an oversight
//!
//! [`Grip::Move`] is live: a drag on the selection's body moves it, through
//! `EditSession::move_objects`.
//!
//! The **eight resize grips change the cursor and consume the drag, and
//! perform no edit yet.** That is not a placeholder left in by accident, and
//! the reason is worth writing down rather than rediscovering: `pdfcer-core`
//! has `move_object`, `move_objects`, `move_subpath`, `move_node`,
//! `move_nodes` and `move_handle` — and **no scale or resize verb for a
//! vector object at all**. `GUI_ROADMAP.md` 1.2 (*"move and resize anything
//! carrying a `/Rect`"*, `FEATURES.md:208`) is the row that gives them one,
//! and it covers annotations, form widgets, redaction marks, links and ce
//! dimensions — objects whose size is a rectangle in the file rather than a
//! consequence of their path data.
//!
//! Consuming the drag is the deliberate part. Without it, a drag that started
//! on a grip would fall through and become a **marquee**, so aiming at a
//! resize handle would silently replace the selection the operator was trying
//! to resize. Swallowing the gesture is the honest behaviour until the verb
//! exists.
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
///
/// Large enough to hit with a mouse without a steady hand, small enough that
/// eight of them around a modest selection do not obscure it. It is also the
/// *drawn* size — grip and target are the same square, which is what makes
/// "aim at the thing you can see" true rather than approximately true.
pub const GRIP_SIZE_PX: f32 = 8.0;

/// Extra slack, in screen points, around a grip's drawn square when
/// hit-testing it.
///
/// Small and asymmetric with the selection catch radius on purpose: a grip is
/// a visible target the operator is aiming at, so it needs far less
/// forgiveness than an invisible hairline does, and every point of slack here
/// is a point stolen from the body-drag region just inside it.
pub const GRIP_GRAB_SLACK_PX: f32 = 2.0;

/// The smallest box, in screen points, that gets mid-edge grips on an axis.
///
/// Below three grip-widths the mid-edge grip would sit on top of its two
/// corner neighbours, producing an unaimable pile that looks like a rendering
/// fault. Corner grips are always offered — they are the ones that survive a
/// small box — so nothing is unreachable, there is simply less on screen.
pub const MIN_MID_GRIP_EXTENT_PX: f32 = GRIP_SIZE_PX * 3.0;

/// ★★★ The smallest box, **across** an axis, that still gets that axis's
/// mid-edge grips — the rule that stops a grip swallowing the body.
///
/// # The defect this closes, measured
///
/// [`MIN_MID_GRIP_EXTENT_PX`] gates a mid-edge grip on its **own** axis, so it
/// cannot pile onto its corner neighbours. Nothing gated it on the
/// **perpendicular** one, and that is the axis a mid-edge grip eats into.
///
/// A form field of 160 × 20 pt at the operator's fitted 29.55 % is **47.3 × 5.9
/// px**. The box is wide enough for North and South, so both are offered — and
/// each reaches `GRIP_SIZE_PX / 2 + GRIP_GRAB_SLACK_PX` = **6 px** into a box
/// that is 5.9 px tall. **Dead centre of the field is inside its own North
/// grip.** An operator dragging a short field from the middle to move it gets a
/// degenerate resize instead, which the engine then refuses by name:
/// `resize-widget-commit … grip=North sy=-42.5314` → `edit-widget-refused …
/// rectangle has no area`. Found by a driven check whose own press landed there.
///
/// # Why this number
///
/// Two opposing mid-edge grips consume `2 × 6 = 12 px` of the crossing axis, and
/// a body worth aiming at needs at least a grip's width of its own. Below
/// **20 px** across, the mid-edge pair is withheld and the corners — which are
/// the grips that survive a small box, and the ones a resize actually wants —
/// are all that is offered.
///
/// ⇒ ★★ **This was a partial answer until 2026-09-05, and the rest is now
/// built rather than recorded.** Withholding the mid-edge pair fixed the short
/// *strip*; it did nothing for a box that is small in **both** axes, where the
/// four corner grips cover the body between them and there is no mid-edge pair
/// left to withhold. That case is not exotic — it is the operator's own
/// molecular-structure fixture, whose cells are **0.85 pt across**. See
/// [`grip_bounds`], which pushes the grips OUTWARD instead of dropping them.
///
/// This constant keeps its job: it is the width of body a box must have before
/// the grips can sit on its own edges, and [`grip_bounds`] is the function that
/// guarantees it by construction rather than by a filter.
pub const MIN_BODY_STRIP_PX: f32 = GRIP_SIZE_PX + 2.0 * (GRIP_SIZE_PX / 2.0 + GRIP_GRAB_SLACK_PX);

/// One grip on the selection's bounding box.
///
/// Named by compass point rather than by index, because an index would have
/// to be read against a table to know which corner it meant, and the cursor
/// mapping below is exactly such a table — written once, here.
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
    /// ★★ **The rotate handle**, offset above the top edge on a stem.
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
    /// # ★ It is drawn as a CIRCLE
    ///
    /// Every square on this canvas resizes. A shape that resized in one place
    /// and rotated in another would be a private convention the operator has to
    /// learn, which is `handles.md` H2's stated failure mode — *"the operator
    /// has to learn a control they already knew."*
    ///
    /// # And it is not a resize
    ///
    /// [`Self::is_resize`] answers `false`, so `gesture::meaning` routes a press
    /// on it to its own drag kind rather than to `DragKind::Resize`. That
    /// predicate used to be `self != Self::Move`, which would have quietly made
    /// this the ninth resize grip — a rotate handle that scaled the object, and
    /// a defect nobody would have thought to test for.
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
    ///
    /// The diagonal cursors are *shared between opposite corners* — NW and SE
    /// both read as `ResizeNwSe` — because that is what the cursor is
    /// describing: the **axis of the resize**, not which corner is under the
    /// hand. Every platform's own resize cursors work this way, and giving
    /// each corner its own arrow would be a private convention the operator
    /// has to learn.
    #[must_use]
    pub fn cursor(self) -> CursorIcon {
        match self {
            Self::NorthWest | Self::SouthEast => CursorIcon::ResizeNwSe,
            Self::NorthEast | Self::SouthWest => CursorIcon::ResizeNeSw,
            Self::North | Self::South => CursorIcon::ResizeVertical,
            Self::East | Self::West => CursorIcon::ResizeHorizontal,
            Self::Move => CursorIcon::Move,
            // ★ egui 0.35 has no rotate cursor, so this is the nearest honest
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
    ///
    /// ★★ This was `self != Self::Move`, and leaving it that way when
    /// [`Self::Rotate`] arrived would have made the rotate handle **the ninth
    /// resize grip**: `gesture::meaning` asks exactly this question to decide
    /// between `DragKind::Resize` and everything else, so a press on the handle
    /// would have scaled the object about a corner. It would have looked like a
    /// deliberate feature and nothing in the suite asked about it.
    ///
    /// The enumeration is deliberate rather than a negation for that reason: a
    /// tenth affordance added later has to be classified rather than defaulting
    /// into the resize family.
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
    ///
    /// [`Self::Move`] answers with the box's centre. It has no drawn square,
    /// so the value is only meaningful as "the middle of the thing" — used by
    /// nothing that paints, and defined rather than left as an `Option` so
    /// every arm of the enum has an answer and a future caller cannot be
    /// surprised by a `None`.
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

    /// ★★ **The corner a drag on this grip must leave EXACTLY WHERE IT IS.**
    ///
    /// [`Self::anchor`] answers where the grip *is*; this answers what it pivots
    /// about, and the two are opposite corners. Dragging the south-east grip moves
    /// the south-east corner and leaves the north-west one still — which is what
    /// every drawing application does, and what the standing *"behave the way the
    /// tools they already use behave"* tie-breaker asks for.
    ///
    /// # Why it is a method here and not arithmetic in `canvas::resizing`
    ///
    /// Because it is the same fact as `anchor`, mirrored, and the two must agree:
    /// the ghost is drawn about this point and the commit is computed about it, so
    /// a second spelling would be a preview and an edit that disagreed about which
    /// corner stayed still — an object that jumps on release by exactly the box's
    /// size.
    ///
    /// ★ A mid-edge grip pivots about the **opposite edge**, keeping the axis it
    /// does not scale centred. `East` returns the west edge at the same y, so the
    /// unscaled axis's factor of 1.0 leaves every point on it unmoved whatever y
    /// this returns — but returning the mid-point rather than a corner keeps the
    /// value meaningful if a future edit ever scales both.
    ///
    /// [`Self::Move`] pivots about itself: it does not resize, and a caller that
    /// reached here for it has already gone wrong. Returning the centre is the
    /// harmless answer — a scale about the centre with factors of 1.0 is the
    /// identity — rather than a panic in a frame that is trying to draw.
    /// The grip **diagonally opposite** this one — the one whose anchor is this
    /// grip's pivot.
    ///
    /// ★ Factored out of [`Self::pivot`] rather than duplicated in
    /// [`GripFrame::pivot`], because "which grip stays still" is a fact about
    /// the *enum*, not about the frame it is laid out in. Spelling it twice is
    /// how a turned frame and an upright one end up disagreeing about which
    /// corner a drag anchors to — an object that jumps on release by exactly
    /// the box's size, which is the failure `pivot`'s own doc comment names.
    ///
    /// [`Self::Move`] and [`Self::Rotate`] are their own opposite: neither has a
    /// corner that must not move, and both already answer with the centre.
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
            // ★ The CENTRE, and for this grip it is the real answer rather than
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
///
/// ★ Far enough that its grab area (the handle plus [`GRIP_GRAB_SLACK_PX`])
/// cannot overlap the north grip's, or the two would fight for the same press
/// and which one won would depend on the order they are checked in — the
/// failure `handles.md` H5's corollary is about. With a 7 pt handle and 2 pt of
/// slack on each, 20 pt clears both by a comfortable margin.
///
/// Screen-space, like every other number here (H3), so the handle sits the same
/// distance from the box at 20 % as at 400 %.
pub const ROTATE_STEM_PX: f32 = 20.0;

/// **The rotate handle's square**, above the top edge on its stem.
///
/// Separate from [`grip_rects`] rather than an entry in it, because every
/// consumer of that list treats its members as resize grips: the painter draws
/// them as squares and the hit test routes them to `DragKind::Resize`. Adding a
/// ninth entry would have made the rotate handle a square that resizes — the
/// same collision `Grip::is_resize`'s own note describes, arriving through the
/// list instead of through the predicate.
///
/// Drawn as a circle at this rect's centre; see [`Grip::Rotate`].
#[must_use]
pub fn rotate_rect(bounds: Rect) -> Rect {
    rotate_rect_in(GripFrame::Upright(bounds))
}

/// [`rotate_rect`] in an arbitrary [`GripFrame`].
///
/// In a turned frame the stem points out along the frame's own outward normal,
/// so the handle stays above the mark's own top edge at every angle rather than
/// above the page's. A handle that stayed at the page's top would end up
/// *inside* the mark for any turn past 90°.
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
/// # ★★★ The defect this closes, in the operator's own words
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
/// - [`crate::canvas::overlay::MIN_OUTLINE_EXTENT_PX`] **floors the drawn box at
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
/// ★ **Above the threshold the push is exactly zero and every grip lands byte
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
///
/// # ★★★ Why a type rather than an `Option<[Pos2; 4]>` parameter everywhere
///
/// Because six functions need the same question answered — the painter, the hit
/// test, the rotate handle, the ghost, the cursor and the drag — and this
/// project's standing rule (H7) is *one value, one decision, every consumer*.
/// The 2026-08-20 incident it comes from is a dimension's vertex handles being
/// painted from the selection and hit-tested from a capability check: each half
/// self-consistent, the pair invisible to any test of either half, and the
/// symptom a handle that was visible and untouchable in the mode that authors
/// dimensions.
///
/// # The upright case is bit-for-bit what it always was
///
/// [`GripFrame::Upright`] carries the same `Rect` the old signature took and
/// every arm below reduces to the old arithmetic for it. That is deliberate and
/// it is what makes this change unable to regress the overwhelming majority of
/// selections — nothing that is not *turned* takes a new code path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GripFrame {
    /// An axis-aligned screen box: every content selection, every form field,
    /// and every annotation whose appearance is not turned.
    Upright(Rect),
    /// A turned annotation's four **placed corners** in screen space, in the
    /// artwork's own frame: `[lower-left, lower-right, upper-right, upper-left]`
    /// as [`crate::canvas::annotquad::OrientedBox::corners`] orders them.
    ///
    /// ★ "Lower" and "upper" name the *artwork's* edges, not the page's, which
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
    ///
    /// ★ Rounded, and that is the point rather than tidiness: a trace is
    /// compared by a driven check, and a float printed with full precision
    /// differs between a debug and a release build on the last digit. Whole
    /// screen points are the resolution the assertion is about anyway.
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
    ///
    /// # ★★ The defect it inherits, and why it could not simply be skipped
    ///
    /// [`grip_bounds`]'s own header records it: a 0.85 pt cell's grips land on
    /// top of each other and on its body, so there is nothing to aim at. That
    /// is not a property of *upright* selections — a 0.85 pt mark turned 30° is
    /// exactly as unreachable — so a turned frame that skipped the push would
    /// have re-opened the defect for precisely the annotations this change is
    /// about.
    ///
    /// ★ The push is along the frame's **own** axes, not the page's, which is
    /// what keeps a pushed turned frame a rectangle. Expanding an axis-aligned
    /// bound instead would move the corners off the mark's diagonal and the
    /// grips would no longer sit on the outline they are drawn against.
    ///
    /// A degenerate frame (coincident corners) has no axes to push along;
    /// `normalized` answers zero there and the frame is returned unchanged,
    /// rather than becoming a NaN box that paints nothing anywhere.
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
    ///
    /// The turned arm is the upright arm's arithmetic written in terms of the
    /// frame's own corners rather than the page's axes — a mid-edge grip is the
    /// midpoint of the two corners it lies between, and the rotate handle is
    /// [`ROTATE_STEM_PX`] out along the frame's **own** outward normal from the
    /// top edge, so the stem still points away from the mark at every angle.
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
    ///
    /// ⚠ **[`Grip::Move`] and [`Grip::Rotate`] are handled before
    /// [`Grip::opposite`] is consulted, and getting that wrong is a real trap.**
    /// Both are their own opposite, so routing them through `anchor` would
    /// answer with the *rotate handle's position on its stem* — a point outside
    /// the box — where the correct answer for a rotation is the centre. The
    /// upright arm has never had this hazard because [`Grip::pivot`] answers
    /// both arms directly.
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
///
/// Mid-edge grips are omitted on an axis shorter than
/// [`MIN_MID_GRIP_EXTENT_PX`] — see that constant for why. The corners are
/// always present, so a selection is never left with nothing to grab.
///
/// `bounds` must already be the **visible** box, i.e. after
/// [`crate::canvas::overlay::visible_outline_rect`] has grown a degenerate
/// one. A zero-height rule would otherwise get eight grips stacked along a
/// line, which is both unaimable and a fair description of nothing.
#[must_use]
pub fn grip_rects(bounds: Rect) -> Vec<(Grip, Rect)> {
    grip_rects_in(GripFrame::Upright(bounds))
}

/// [`grip_rects`] in an arbitrary [`GripFrame`] — the general form, and what
/// every painter and hit test now calls.
///
/// # ★ The mid-edge omission test is taken on the frame's OWN edges
///
/// A turned box's screen width says nothing about whether its grips pile up:
/// a 200 × 10 pt bar turned 45° has a screen bound of ~148 × 148, which would
/// pass a width test while its two mid-edge grips sat 7 px apart on the short
/// edge. Measuring the frame's own edge lengths is the same question the
/// upright arm asks, asked correctly.
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
        // ★ `pushed` is the identity on an upright frame, so this is the same
        // `bounds` the caller handed in — matched rather than re-destructured
        // with an `unreachable!`, because a panic macro in a shipped painter is
        // a crash where a compiler-checked match is nothing.
        GripFrame::Upright(bounds) => bounds,
    };
    // ★★★ Everything below anchors to the PUSHED box, never to `bounds`.
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
    // ★ The piling condition stays, and stays measured against the PUSHED box:
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
///
/// ★★★ Two flags rather than one, added 2026-08-28 when annotations and form
/// fields gained a resize verb (`resize_annotation`, `edit_widget … with_rect`)
/// and neither gained a rotate one.
///
/// The single `offer_resize` bool this replaces was correct while exactly one
/// kind of thing could be resized. It cannot express *"eight grips, no rotate
/// handle"*, and the alternative — painting a rotate handle that does nothing —
/// is the **visible control, silently inert** failure this project spends its
/// time removing.
///
/// ★★★ **AND IT EARNED THE SECOND FLAG THE SAME DAY, FROM THE OTHER SIDE.**
///
/// `Pass 155.0` gave the engine `rotate_annotation` and `Pass 159.0` gave it
/// `rotate_dimension`, so within hours of this struct being written the two
/// flags stopped moving together in *both* directions:
///
/// | selection | resize | rotate | why |
/// |---|---|---|---|
/// | page content, Object rung | ✓ | ✓ | `transform_objects` does both |
/// | a **markup** annotation | ✓ | ✓ | `resize_annotation` **and** `rotate_annotation` |
/// | a **ce dimension** | ✗ | ✓ | its extent IS its measurement, so there is no scale verb and there will not be one — but a rotation is an isometry, so it can turn |
/// | a **form field's** box | ✓ | ✗ | a widget's rotation is `/MK /R`, a quantised 0/90/180/270 declaration, and it is not built |
///
/// ⇒ The ce-dimension row is the one a single bool could never have expressed,
/// and it is the row that proves the split was not premature: **rotate without
/// resize** is a real, shipping combination, not a hypothetical. The engine
/// declined a dimension scale by name and will keep declining it — *"either the
/// displayed value stays fixed while the geometry grows, so the dimension lies
/// about the drawing; or both change, so nothing was measured"* — so this is a
/// permanent asymmetry rather than a gap waiting to close.
///
/// ★★ It is one value passed to BOTH the painter and the hit test, which is
/// rule H7 and is why it is a struct rather than two arguments threaded
/// separately. That row exists because it failed on 2026-08-20: a dimension's
/// vertex handles were painted from the selection and hit-tested behind a
/// capability the mode did not have, so they were visible and untouchable in
/// the very mode that authors dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GripSet {
    /// The eight scale grips.
    pub resize: bool,
    /// The rotate handle above the top edge.
    ///
    /// ★ Deliberately not collapsed into [`Self::resize`]: *"can this be
    /// scaled"* and *"can this be turned"* are two questions about the engine's
    /// verb list, and a shell that inferred one from the other would offer
    /// rotation to the next kind that gains a resize verb without anybody
    /// deciding.
    ///
    /// ★★ That caution paid on the day it was written. Until 2026-08-28 this
    /// field's doc said *"never true without `resize` today"* — and
    /// [`GripSet::rotate_only`] now exists, because a ce dimension turns and
    /// does not scale. A struct that had collapsed the two would have had to be
    /// un-collapsed to ship that, and the intervening builds would have offered
    /// eight scale grips around a dimension whose extent is its measurement.
    pub rotate: bool,
}

impl GripSet {
    /// Everything — page **content** at the Object rung, and a **markup
    /// annotation**, which gained the second half on 2026-08-28.
    ///
    /// ★ The annotation reached this set from `scale_only` when
    /// `rotate_annotation` shipped, and the two callers now name the identical
    /// value for two different reasons. That is fine and is not a merge waiting
    /// to happen: content rotates through `transform_objects`, an annotation
    /// through `rotate_annotation`, and the day either verb is withdrawn only
    /// one of the two callers changes.
    pub const fn all() -> Self {
        Self {
            resize: true,
            rotate: true,
        }
    }

    /// The eight scale grips and no rotate handle — a **form field's box**.
    ///
    /// ★★★ **A widget is the one thing on this canvas that scales and cannot
    /// turn**, and the asymmetry is the PDF standard's rather than a gap in
    /// pdfcer. `edit_widget(… with_rect)` rebuilds a field's appearance into a
    /// new box, so a resize is expressible; a widget's *rotation* is `/MK /R`
    /// (§12.5.6.19 Table 189), **a quantised 0/90/180/270 declaration the
    /// appearance generator reads** rather than a free-angle transform. There
    /// is no verb for it, and `rotate_annotation` refuses a widget by name and
    /// points at one that is not built.
    ///
    /// ⇒ So no rotate handle is painted over a form field and none is
    /// hit-tested there. **R9**: rendering nothing is the honest answer for a
    /// capability that does not exist. A ninth handle that declined on release
    /// would be the *"visible control, silently inert"* failure wearing the
    /// costume of a fix.
    ///
    /// ★ Until 2026-08-28 this said *"an annotation or a form field's box"*.
    /// The annotation moved to [`Self::all`] when `rotate_annotation` shipped;
    /// the widget stayed, and it is the only member left.
    pub const fn scale_only() -> Self {
        Self {
            resize: true,
            rotate: false,
        }
    }

    /// **The rotate handle alone** — a selected **ce dimension**.
    ///
    /// ★★★ The combination that could not be spelled before this struct had two
    /// fields, and the one that proves they had to be two.
    ///
    /// A ce dimension has **no scale verb and is never going to have one**.
    /// `pdfcer-core` declined it outright rather than leaving it unbuilt, and
    /// the argument is worth carrying here because it is the reason this
    /// constructor is not a temporary shape:
    ///
    /// > It has no honest reading. Either the displayed value stays fixed while
    /// > the geometry grows, so the dimension **lies about the drawing**; or
    /// > both change, so **nothing was measured** and the operator has *drawn*
    /// > a number rather than *taken* one.
    ///
    /// A **rotation** has no such problem, because a rotation is an isometry:
    /// every distance is preserved, so the measured value is identical either
    /// side of it *by construction*. That is what makes turning a dimension a
    /// legitimate drafting operation while scaling one is not.
    ///
    /// ★★ If an operator wants a dimension to read a different number for the
    /// same drawn line, the operation they want is `set_group_scale` — points
    /// per unit — which already ships and lives on the Measure surface. This
    /// canvas deliberately offers no handle for it: a scale that is a property
    /// of a *measurement group* has no grip on one member of that group.
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
///
/// # ★★★ The body test stays on the UPRIGHT bound, and that is not an oversight
///
/// The eight squares and the rotate handle move into the turned frame, because
/// they are affordances the operator aims at and they must be where they are
/// drawn. [`Grip::Move`] is different: it means *"the press landed on the
/// object"*, and narrowing it to the turned quad would make a turned mark
/// **harder to grab than an upright one**, on exactly the marks whose extent is
/// least obvious.
///
/// That is also the rule `selection::annot::Candidate` already follows for
/// selection itself, and it comes from the engine's own argument: `/Rect`
/// contains the pen half-width, so it is the geometry *plus* the tolerance a
/// hit test would otherwise have to add. Selection stays generous; only the
/// drawing gets more honest.
pub fn grip_at_in(frame: GripFrame, pointer: Pos2, offer: GripSet) -> Option<Grip> {
    let bounds = frame.bounds();
    if offer.rotate {
        // ★★ The rotate handle FIRST, and the reason is H7 rather than
        // geometry: it sits outside the box, so it collides with nothing and
        // the order could not matter for correctness. It is first because
        // **the same predicate decides painting and hit-testing**, and that
        // predicate is `GripSet` — so a handle painted here is grabbable
        // here, in one place, with nothing in between for a future edit to slip
        // a capability check into.
        //
        // That row exists because it failed on 2026-08-20: a dimension's vertex
        // handles were painted from the selection and hit-tested behind a
        // capability the mode did not have, so they were visible and untouchable
        // in the very mode that authors dimensions.
        if rotate_rect_in(frame)
            .expand(GRIP_GRAB_SLACK_PX)
            .contains(pointer)
        {
            return Some(Grip::Rotate);
        }
    }
    // ★ The eight scale grips are gated separately from the rotate handle
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
    ///
    /// ★ **The fixture was `10.0 × 200.0` until 2026-08-29 and is now
    /// `22.0 × 200.0`**, because 10 px wide fails the *other* rule — see
    /// [`MIN_BODY_STRIP_PX`]. At 10 px the East and West grips reach 6 px in
    /// from each side and cover the box entirely, so withholding them is
    /// correct and this test's `assert!(kinds.contains(&Grip::East))` was
    /// asserting the defect.
    ///
    /// 22 keeps the property this test is actually about: below
    /// `MIN_MID_GRIP_EXTENT_PX` (24) on the narrow axis, so North and South are
    /// still dropped for piling onto the corners; above `MIN_BODY_STRIP_PX`
    /// (20) across it, so a body survives and East and West are legitimately
    /// offered. **Two rules, two thresholds, and a fixture between them is what
    /// tests either one in isolation.**
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

        // …and symmetrically for a short one. ★ 22 rather than 10 for the same
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

    /// ★★ **`pivot` is the OPPOSITE of `anchor`, for every resize grip.**
    ///
    /// The property the whole resize rests on, asserted as a relation rather
    /// than against a table of corners — a table would pass for a build whose
    /// `pivot` returned `anchor` unchanged if somebody wrote the table from the
    /// same wrong function.
    ///
    /// The failure it forbids is specific and looks plausible: scaling about
    /// the grip being dragged makes the shape grow *away* from the operator's
    /// hand instead of towards it. It resizes; it is wrong; and it is the kind
    /// of wrong that survives a screenshot.
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

    /// ★★ **An inner rung offers `Move` and none of the eight.**
    ///
    /// The regression test for a defect found by driving: an anchor mark is
    /// centred on its point, so an anchor at a corner of the object's bounding
    /// box is half outside it — and the corner grip, with two points of grab
    /// slack, covers exactly that spot. A drag from a selected corner anchor
    /// raised no move at all, because the press had been claimed by the
    /// north-west grip.
    ///
    /// The operator's version is *"I can drag the middle nodes and not the end
    /// ones"*, which reads as a broken hit test rather than as two features
    /// competing for one pixel.
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

    /// ★★★ **A ce dimension's set: the ninth handle and NONE of the eight.**
    ///
    /// The combination [`GripSet`] grew a second field for, asserted rather
    /// than assumed, because it is the one a build can get wrong in two
    /// directions and look plausible in both:
    ///
    /// * a build that reused `GripSet::all()` would paint eight scale grips
    ///   around a dimension whose extent **is** its measurement — a resize that
    ///   the engine declines by name, offered on the canvas as though it did
    ///   not;
    /// * a build that left `GripSet::default()` alone would paint nothing, and
    ///   the rotation `pdfcer-core` shipped on `Pass 159.0` would be unreachable
    ///   with no affordance anywhere.
    ///
    /// The middle row — a press at a **corner** answering `Move` rather than
    /// `NorthWest` — is the load-bearing one. `grip_at` gates the eight
    /// separately from the ninth, and a build that gated them together would
    /// pass the first and third assertions and fail only this one.
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

#[cfg(test)]
mod body_strip_tests {
    use super::*;

    /// ★★★ **The centre of a short, wide selection is the BODY, not a grip.**
    ///
    /// The measured case: a 160 × 20 pt form field at the operator's fitted
    /// 29.55 % zoom is 47.3 × 5.9 px. Before this rule, dead centre answered
    /// `Grip::North` — so dragging the field to move it committed a degenerate
    /// resize the engine then refused, and the operator's field did not move
    /// and did not say why.
    ///
    /// ★ The numbers are the real ones from `widget-move.trace.txt` rather than
    /// round ones, because the defect is a threshold and a rounded fixture can
    /// sit on the comfortable side of it without anybody noticing.
    #[test]
    fn the_centre_of_a_short_field_is_the_body() {
        let field = Rect::from_min_size(Pos2::new(849.0, 957.3), Vec2::new(47.3, 5.9));
        assert_eq!(
            grip_at(field, field.center(), GripSet::all()),
            Some(Grip::Move),
            "the centre of a 5.9 px-tall box was inside its own North grip"
        );
    }

    /// ★★ …and on a short box **no grip's grab region reaches into the body at
    /// all**, which is the promise that replaced "the mid-edge pair is withheld".
    ///
    /// # Why this assertion changed on 2026-09-05, stated rather than quietly edited
    ///
    /// The original wording asserted that North and South were *dropped* from
    /// the offered list on a 5.9 px-tall field. That was the right assertion for
    /// the mechanism that existed at the time — a filter — and it is the wrong
    /// one for the mechanism that exists now. [`grip_bounds`] pushes the grips
    /// outward instead of dropping them, so on this field North and South are
    /// offered **and drawn 7.05 px clear of the box**, where they can be aimed
    /// at and where they eat nothing. Withholding them would now be a
    /// regression: they are the two grips that resize a short field's height,
    /// which is the one thing an operator is likely to want from it.
    ///
    /// So the promise is restated at the level it was always really about:
    /// **the body belongs to the body.** That is falsifiable against both
    /// mechanisms, which the old wording was not.
    ///
    /// Asserted separately from `the_centre_of_a_short_field_is_the_body`
    /// because the two are different promises: one is about where a press lands,
    /// the other about what the operator is shown. A grip painted where it
    /// cannot be aimed is the affordance R9 forbids, and the painter reads this
    /// same list.
    #[test]
    fn a_short_box_keeps_its_whole_body_and_its_grips_sit_outside_it() {
        let field = Rect::from_min_size(Pos2::new(849.0, 957.3), Vec2::new(47.3, 5.9));
        let offered = grip_rects(field);

        // Six, not eight, and the arithmetic is worth writing down because the
        // number is not obvious. The field is 47.3 wide and 5.9 tall, so only
        // the vertical axis is pushed: the anchor box is 47.3 x 20. North and
        // South survive because 47.3 clears MIN_MID_GRIP_EXTENT_PX (24). East
        // and West do not, because 20 does not — they are withheld for PILING
        // onto their corner neighbours, which is a different rule from the one
        // this test is about and one the push does not and should not touch.
        let names: Vec<Grip> = offered.iter().map(|(g, _)| *g).collect();
        assert!(
            names.contains(&Grip::North) && names.contains(&Grip::South),
            "the mid-edge pair that resizes a short field's HEIGHT was withheld: {names:?}"
        );
        assert!(
            !names.contains(&Grip::East) && !names.contains(&Grip::West),
            "East/West would pile onto the corners at 20 pt of pushed height: {names:?}"
        );

        // ★ The load-bearing assertion. Every grip's GRAB region — the drawn
        // square plus its slack, which is what `grip_at` tests — must miss the
        // horizontal strip through the middle of the field. Sampled across the
        // width rather than at the centre alone, because the old defect left a
        // 1.4 x 0.5 pt hole at dead centre and a centre-only test walks straight
        // through it.
        for i in 0..=20 {
            let x = field.left() + field.width() * (i as f32 / 20.0);
            let p = Pos2::new(x, field.center().y);
            assert_eq!(
                grip_at(field, p, GripSet::all()),
                Some(Grip::Move),
                "a grip claimed the body at x offset {i}/20 of a 5.9 px-tall field"
            );
        }
    }

    /// ★★★ **His banana. An object 0.85 pt across can be moved.**
    ///
    /// The report, verbatim: *"zoom in on the atoms of the banana pdf file and
    /// see what happens when you try to draw a box around a molecule and move
    /// it, or select the ion and move it."* Driving it produced
    /// `resize-declined reason=Degenerate` on every press, because the box was
    /// floored to [`crate::canvas::overlay::MIN_OUTLINE_EXTENT_PX`] = 6 pt and
    /// four corner grips reaching 6 pt each covered all of it.
    ///
    /// The fixture is the **floored** box, not the 0.85 pt one, because the
    /// floor is what the operator's pointer actually meets — testing the
    /// un-floored rect would test a rectangle nothing on screen corresponds to.
    #[test]
    fn the_smallest_object_the_shell_can_draw_is_still_grabbable() {
        let cell = Rect::from_min_size(
            Pos2::new(640.0, 480.0),
            Vec2::splat(crate::canvas::overlay::MIN_OUTLINE_EXTENT_PX),
        );
        // Every point of it, corners included — there is no part of a 6 pt box
        // an operator could be expected to aim at more carefully than another.
        for i in 0..=6 {
            for j in 0..=6 {
                let p = Pos2::new(
                    cell.left() + cell.width() * (i as f32 / 6.0),
                    cell.top() + cell.height() * (j as f32 / 6.0),
                );
                assert_eq!(
                    grip_at(cell, p, GripSet::all()),
                    Some(Grip::Move),
                    "a grip claimed ({i}/6, {j}/6) of a 6 pt cell: the banana defect"
                );
            }
        }
    }

    /// ★ …and the grips are still *there*, outside it, so the cell can be
    /// resized as well as moved.
    ///
    /// Asserted because the cheap way to pass the test above is to stop offering
    /// grips on a small box, which trades one lost capability for another.
    #[test]
    fn the_smallest_object_still_offers_grips_to_resize_it_by() {
        let cell = Rect::from_min_size(
            Pos2::new(640.0, 480.0),
            Vec2::splat(crate::canvas::overlay::MIN_OUTLINE_EXTENT_PX),
        );
        let offered = grip_rects(cell);
        assert!(
            offered.iter().any(|(g, _)| *g == Grip::NorthWest),
            "a small box must keep its corners: {:?}",
            offered.iter().map(|(g, _)| *g).collect::<Vec<_>>()
        );
        for (g, r) in &offered {
            assert!(
                !cell.contains(r.center()),
                "{g:?} is still anchored inside the 6 pt cell at {:?}",
                r.center()
            );
            assert!(
                grip_at(cell, r.center(), GripSet::all()) == Some(*g),
                "{g:?} is drawn where it cannot be aimed: the R9 failure"
            );
        }
    }

    /// ★★ **The push is exactly zero above the threshold**, which is what makes
    /// applying it unconditionally safe.
    ///
    /// If this ever fails, every comfortable selection in the product has moved
    /// its grips, and nothing else in the suite would say so in those words.
    #[test]
    fn a_box_with_a_body_is_not_pushed_at_all() {
        for (w, h) in [
            (MIN_BODY_STRIP_PX, MIN_BODY_STRIP_PX),
            (MIN_BODY_STRIP_PX, 400.0),
            (400.0, MIN_BODY_STRIP_PX),
            (300.0, 200.0),
        ] {
            let r = Rect::from_min_size(Pos2::new(100.0, 200.0), Vec2::new(w, h));
            assert_eq!(
                grip_bounds(r),
                r,
                "a {w} x {h} box was pushed, and it did not need to be"
            );
        }
    }

    /// ★ The push reaches the threshold and stops there, and never runs
    /// backwards as the box shrinks — so there is no zoom at which the
    /// affordance jumps.
    #[test]
    fn the_push_reaches_the_threshold_and_stops_there() {
        let mut previous = f32::INFINITY;
        for step in 0..=40 {
            let extent = MIN_BODY_STRIP_PX * (step as f32 / 40.0);
            let r = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::splat(extent));
            let pushed = grip_bounds(r);
            assert!(
                (pushed.width() - MIN_BODY_STRIP_PX).abs() < 1e-3,
                "a {extent} pt box was grown to {} rather than to the threshold",
                pushed.width()
            );
            let push = (pushed.width() - extent) / 2.0;
            assert!(
                push <= previous + 1e-3,
                "the push went UP as the box grew, at extent {extent}"
            );
            previous = push;
        }
    }

    /// ★ A comfortable box is unchanged, which is what says the rule is a floor
    /// and not a redesign.
    #[test]
    fn a_comfortable_box_still_gets_all_eight() {
        let roomy = Rect::from_min_size(Pos2::new(100.0, 200.0), Vec2::new(300.0, 200.0));
        assert_eq!(grip_rects(roomy).len(), 8);
        assert_eq!(
            grip_at(roomy, roomy.center(), GripSet::all()),
            Some(Grip::Move)
        );
    }
}
