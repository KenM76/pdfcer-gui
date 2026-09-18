//! `dock::compass` — the five places a panel can be released over one
//! compartment, and which one the pointer is in.
//!
//! Over any stack the dock draws, five releases are possible: join the group,
//! or split against one of its four edges. [`DropZone`] names them,
//! [`Compass`] divides a rectangle into them, and
//! [`DockLayout::resolve_drop`] turns a pointer position into the
//! [`DropTarget`] the drop grammar acts on.
//!
//! ## ★ A left or right edge splits the SIDE, not the stack
//!
//! The layout has four fixed levels — side, column, stack, tab — and the only
//! horizontal split it can express is a new column. Releasing against a
//! stack's left edge therefore inserts a **full-height column** at that
//! stack's own column boundary, rather than a half-width neighbour inside the
//! column. That is the grammar showing through, and it is why the preview is a
//! replay: the operator is shown the column that would actually appear, not an
//! outline of the half of the stack they aimed at.
//!
//! ## ★ The tab strip is not one of the five
//!
//! A pointer over a stack's tab bar is asking *between which two tabs*, which
//! [`super::geometry::DockGeometry::gap_in`] already answers and
//! [`super::drag`] already draws a caret for. The strip is therefore resolved
//! before the compass and reported as a landing with no zone, so an overlay
//! knows to draw the caret and not the zones. The compass divides what is left
//! of the compartment — its body.
//!
//! ## ★★ The zones are drawn as they are hit, because they are one definition
//!
//! Four edge bands over a rectangle overlap at its corners, and a compass that
//! *draws* full-length bands but *resolves* a corner by some other rule shows
//! the operator one outcome and commits another — failure mode #2 with extra
//! steps. Here a corner belongs to the **nearest** edge, measured as a
//! fraction of that edge's own band rather than in points, which makes the
//! boundary between two zones the straight line from the outer corner to the
//! inner one: the zones are a mitred picture frame around a centre rectangle,
//! and [`Compass::outline`] returns exactly the quadrilateral each one is hit
//! as. An overlay that fills those five quads has drawn the hit test.

use egui::{Pos2, Rect};

use super::drop::DropTarget;
use super::geometry::{ColumnAddr, DockGeometry, StackAddr};
use super::model::{DockLayout, Stack};

/// **One of the five releases possible over a compartment.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DropZone {
    /// Join the hovered stack as a tab.
    Centre,
    /// Start a column before the hovered stack's column.
    Left,
    /// Start a column after the hovered stack's column.
    Right,
    /// Split the hovered stack's column above it.
    Top,
    /// Split the hovered stack's column below it.
    Bottom,
}

impl DropZone {
    /// A stable short name, for a published region and for a diagnostic line.
    ///
    /// Lower case and never translated: it is an identifier a harness matches
    /// on, not a label an operator reads.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Centre => "centre", // ui-text-exempt: a region key, never displayed
            Self::Left => "left",     // ui-text-exempt: a region key, never displayed
            Self::Right => "right",   // ui-text-exempt: a region key, never displayed
            Self::Top => "top",       // ui-text-exempt: a region key, never displayed
            Self::Bottom => "bottom", // ui-text-exempt: a region key, never displayed
        }
    }

    /// Every zone, in the order an overlay should draw them: the centre first,
    /// so an edge's ink wins where they meet at a rounded corner.
    pub const ALL: [Self; 5] = [
        Self::Centre,
        Self::Left,
        Self::Right,
        Self::Top,
        Self::Bottom,
    ];
}

/// **A rectangle divided into the five drop zones.**
///
/// Cheap enough to build per frame and per candidate; it holds two rects and
/// derives everything else. See the module header for the mitre rule that
/// makes [`Compass::zone_at`] and [`Compass::outline`] one definition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Compass {
    /// The whole compartment body the zones divide.
    pub area: Rect,
    /// The centre zone — `area` less one edge band on each side.
    pub centre: Rect,
}

impl Compass {
    /// Divide `area`.
    ///
    /// The edge bands are a fraction of each dimension so that a small
    /// compartment's edges stay reachable, capped so that a large one's centre
    /// stays dominant — the centre is the common release, and an edge split is
    /// the deliberate one.
    #[must_use]
    pub fn new(area: Rect) -> Self {
        let centre = area.shrink2(egui::vec2(band(area.width()), band(area.height())));
        Self { area, centre }
    }

    /// **Which zone `pos` is in**, or `None` if it is outside `area` entirely.
    #[must_use]
    pub fn zone_at(&self, pos: Pos2) -> Option<DropZone> {
        if !self.area.contains(pos) {
            return None;
        }
        let x = band(self.area.width());
        let y = band(self.area.height());
        // How far each edge is, as a fraction of its own band: under one is
        // inside that band. A zero-width band can never be entered, and saying
        // so with an infinity rather than a division keeps this total.
        let reach = |d: f32, band: f32| if band > 0.0 { d / band } else { f32::INFINITY };
        let candidates = [
            (DropZone::Left, reach(pos.x - self.area.left(), x)),
            (DropZone::Right, reach(self.area.right() - pos.x, x)),
            (DropZone::Top, reach(pos.y - self.area.top(), y)),
            (DropZone::Bottom, reach(self.area.bottom() - pos.y, y)),
        ];
        let (zone, nearest) =
            candidates
                .into_iter()
                .fold((DropZone::Left, f32::INFINITY), |best, c| {
                    if c.1 < best.1 { c } else { best }
                });
        if nearest < 1.0 {
            Some(zone)
        } else {
            Some(DropZone::Centre)
        }
    }

    /// **The quadrilateral `zone` is hit as**, clockwise, for an overlay to
    /// fill.
    ///
    /// Every zone is four points, the centre included, so a caller draws five
    /// polygons rather than a rect and four special cases.
    #[must_use]
    pub fn outline(&self, zone: DropZone) -> [Pos2; 4] {
        let (a, c) = (self.area, self.centre);
        match zone {
            DropZone::Centre => [
                c.left_top(),
                c.right_top(),
                c.right_bottom(),
                c.left_bottom(),
            ],
            DropZone::Left => [a.left_top(), c.left_top(), c.left_bottom(), a.left_bottom()],
            DropZone::Right => [
                c.right_top(),
                a.right_top(),
                a.right_bottom(),
                c.right_bottom(),
            ],
            DropZone::Top => [a.left_top(), a.right_top(), c.right_top(), c.left_top()],
            DropZone::Bottom => [
                c.left_bottom(),
                c.right_bottom(),
                a.right_bottom(),
                a.left_bottom(),
            ],
        }
    }
}

/// **Where a release at one pointer position would put the dragged panel.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropLanding {
    /// What the drop grammar would be asked to do.
    pub target: DropTarget,
    /// The stack the pointer is over — what an overlay highlights.
    pub over: StackAddr,
    /// Which zone of that stack's body, or `None` when the pointer is on its
    /// tab strip, where the caret is the affordance and the compass is not
    /// drawn at all.
    pub zone: Option<DropZone>,
}

impl DockLayout {
    /// **Resolve a pointer position over the dock into a landing.**
    ///
    /// `None` when the pointer is over no compartment this geometry recorded —
    /// the canvas, a splitter, a side that drew nothing — which is the same
    /// answer as *"a release here docks nothing"*.
    ///
    /// Reads both the geometry and this layout because the two halves of the
    /// question need different sources: *which compartment* is a rect
    /// question, and *which boundary within it* is a count question. The
    /// centre zone appends, so it needs the tab count and cannot be resolved
    /// from rects alone.
    #[must_use]
    pub fn resolve_drop(&self, geometry: &DockGeometry, pos: Pos2) -> Option<DropLanding> {
        let stack = geometry.stack_at(pos)?;
        let tabs = stack_of(self, stack)?.tabs.len();
        if geometry.strip_at(pos) == Some(stack) {
            let gap = geometry.gap_in(stack, pos).unwrap_or(tabs);
            return Some(DropLanding {
                target: DropTarget::Tab { stack, gap },
                over: stack,
                zone: None,
            });
        }
        let zone = Compass::new(body_of(geometry, stack)?).zone_at(pos)?;
        let column = ColumnAddr::new(stack.side, stack.column);
        let target = match zone {
            DropZone::Centre => DropTarget::Tab { stack, gap: tabs },
            DropZone::Left => DropTarget::Column {
                side: stack.side,
                gap: stack.column,
            },
            DropZone::Right => DropTarget::Column {
                side: stack.side,
                gap: stack.column + 1,
            },
            DropZone::Top => DropTarget::Stack {
                column,
                gap: stack.stack,
            },
            DropZone::Bottom => DropTarget::Stack {
                column,
                gap: stack.stack + 1,
            },
        };
        Some(DropLanding {
            target,
            over: stack,
            zone: Some(zone),
        })
    }
}

/// The compartment less its tab strip, which the compass does not divide.
///
/// The strip spans the compartment's full width along its top, so subtracting
/// it leaves no band that belongs to neither. A stack with no strip recorded —
/// too narrow to draw one — is divided whole.
///
/// Visible to [`super::overlay`] because the rectangle the overlay fills must
/// be the rectangle the resolution divided. Two spellings of "the compartment
/// less its strip" would put the zones the operator sees a strip's height away
/// from the zones the release is read against.
pub(super) fn body_of(geometry: &DockGeometry, addr: StackAddr) -> Option<Rect> {
    let stack = geometry.stack_rect(addr)?;
    let Some(strip) = geometry.strip_rect(addr) else {
        return Some(stack);
    };
    Some(Rect::from_min_max(
        egui::pos2(stack.left(), strip.bottom().min(stack.bottom())),
        stack.max,
    ))
}

/// The stack at `addr`, or `None` if the address has gone stale.
fn stack_of(layout: &DockLayout, addr: StackAddr) -> Option<&Stack> {
    layout
        .side(addr.side)
        .columns
        .get(addr.column)?
        .stacks
        .get(addr.stack)
}

/// How much of one dimension an edge band takes.
fn band(extent: f32) -> f32 {
    (extent * EDGE_FRACTION).min(EDGE_MAX_PTS)
}

/// The share of a compartment's width or height one edge zone takes, before
/// the cap. Two opposite bands therefore never claim more than half of it, so
/// the centre zone cannot vanish however small the compartment.
const EDGE_FRACTION: f32 = 0.25;

/// The widest an edge zone gets. Past this the compartment is large enough
/// that a proportional band would be a bigger target than it needs and would
/// eat the centre — the release the operator makes most often.
const EDGE_MAX_PTS: f32 = 64.0;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dock::model::{Column, DockSide, PanelId, SideLayout};
    use egui::{pos2, vec2};

    /// A layout with two columns on the left: `["pages","bookmarks"]` over
    /// `["layers"]` in the first, `["fields"]` alone in the second.
    fn layout() -> DockLayout {
        DockLayout::new(
            SideLayout::new([
                Column::new([Stack::tabbed(["pages", "bookmarks"]), Stack::new("layers")]),
                Column::new([Stack::new("fields")]),
            ]),
            SideLayout::none(),
        )
    }

    /// The height of the tab strip every stack in these fixtures gets.
    const STRIP: f32 = 20.0;

    /// Lay `layout`'s left side out over `rect`: columns left to right in even
    /// widths, stacks top to bottom in even heights, a strip across the top of
    /// each stack and its tabs in even widths along that strip.
    ///
    /// Deliberately not [`super::super::plan`]: this is a plausible geometry,
    /// not the real one, and the compass must not care which. What it must
    /// share with the real one is the relationship the resolution depends on —
    /// the strip along the top of the compartment — and that is asserted
    /// directly by `over_the_tab_strip_is_a_boundary_between_tabs`.
    #[allow(clippy::cast_precision_loss)]
    fn geometry_for(layout: &DockLayout, rect: Rect) -> DockGeometry {
        let mut g = DockGeometry::default();
        let side = DockSide::Left;
        let columns = &layout.side(side).columns;
        g.push_side(side, rect);
        let col_w = rect.width() / columns.len() as f32;
        for (ci, column) in columns.iter().enumerate() {
            let col_rect = Rect::from_min_size(
                pos2(rect.left() + ci as f32 * col_w, rect.top()),
                vec2(col_w, rect.height()),
            );
            g.push_column(ColumnAddr::new(side, ci), col_rect);
            let stack_h = col_rect.height() / column.stacks.len() as f32;
            for (si, stack) in column.stacks.iter().enumerate() {
                let addr = StackAddr::new(side, ci, si);
                let stack_rect = Rect::from_min_size(
                    pos2(col_rect.left(), col_rect.top() + si as f32 * stack_h),
                    vec2(col_w, stack_h),
                );
                g.push_stack(addr, stack_rect);
                let strip = Rect::from_min_size(stack_rect.min, vec2(col_w, STRIP));
                g.push_strip(addr, strip);
                let tab_w = col_w / stack.tabs.len() as f32;
                for ti in 0..stack.tabs.len() {
                    g.push_tab(
                        addr.tab(ti),
                        Rect::from_min_size(
                            pos2(strip.left() + ti as f32 * tab_w, strip.top()),
                            vec2(tab_w, STRIP),
                        ),
                    );
                }
            }
        }
        g
    }

    /// The whole left side in these fixtures: 400 wide, so each column is 200
    /// and each edge band is capped at 64; 400 tall, so the first column's two
    /// stacks are 200 each.
    const SIDE: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(400.0, 400.0));

    /// A point in the body of the stack at `(column, stack)`, offset from its
    /// top-left corner by `(dx, dy)` — measured below the strip, which is what
    /// the compass divides.
    fn in_body(column: usize, stack: usize, dx: f32, dy: f32) -> Pos2 {
        #[allow(clippy::cast_precision_loss)]
        let x = column as f32 * 200.0 + dx;
        #[allow(clippy::cast_precision_loss)]
        let y = stack as f32 * 200.0 + STRIP + dy;
        pos2(x, y)
    }

    fn resolve(pos: Pos2) -> Option<DropLanding> {
        let l = layout();
        l.resolve_drop(&geometry_for(&l, SIDE), pos)
    }

    #[test]
    fn the_centre_of_a_compartment_joins_the_group_as_its_last_tab() {
        let landing = resolve(in_body(0, 0, 100.0, 90.0)).expect("over the first stack");
        assert_eq!(landing.zone, Some(DropZone::Centre));
        assert_eq!(landing.over, StackAddr::new(DockSide::Left, 0, 0));
        assert_eq!(
            landing.target,
            DropTarget::Tab {
                stack: StackAddr::new(DockSide::Left, 0, 0),
                gap: 2,
            },
            "the two tabs it already has, so the boundary past the last"
        );
    }

    /// ★ The module header's rule made a test: an edge that looks like it
    /// splits the stack in half splits the whole side into another column.
    #[test]
    fn a_release_against_a_left_or_right_edge_starts_a_column_on_that_side_of_it() {
        let left = resolve(in_body(1, 0, 4.0, 100.0)).expect("over the second column");
        assert_eq!(left.zone, Some(DropZone::Left));
        assert_eq!(
            left.target,
            DropTarget::Column {
                side: DockSide::Left,
                gap: 1,
            },
            "before the column it was dropped on"
        );

        let right = resolve(in_body(1, 0, 196.0, 100.0)).expect("over the second column");
        assert_eq!(right.zone, Some(DropZone::Right));
        assert_eq!(
            right.target,
            DropTarget::Column {
                side: DockSide::Left,
                gap: 2,
            },
            "after it, which is a new last column"
        );
    }

    #[test]
    fn a_release_against_a_top_or_bottom_edge_splits_the_column_around_the_stack() {
        let top = resolve(in_body(0, 1, 100.0, 4.0)).expect("over the lower stack");
        assert_eq!(top.zone, Some(DropZone::Top));
        assert_eq!(
            top.target,
            DropTarget::Stack {
                column: ColumnAddr::new(DockSide::Left, 0),
                gap: 1,
            },
            "above the stack it was dropped on"
        );

        let bottom = resolve(in_body(0, 1, 100.0, 174.0)).expect("over the lower stack");
        assert_eq!(bottom.zone, Some(DropZone::Bottom));
        assert_eq!(
            bottom.target,
            DropTarget::Stack {
                column: ColumnAddr::new(DockSide::Left, 0),
                gap: 2,
            }
        );
    }

    /// ★ The strip is resolved before the compass, and reports no zone — which
    /// is how an overlay knows to draw the caret instead of the five quads.
    #[test]
    fn over_the_tab_strip_is_a_boundary_between_tabs_and_not_a_compass_zone() {
        // Two tabs across 200 points: centres at 50 and 150.
        let first = resolve(pos2(40.0, 10.0)).expect("over the strip");
        assert_eq!(first.zone, None);
        assert_eq!(
            first.target,
            DropTarget::Tab {
                stack: StackAddr::new(DockSide::Left, 0, 0),
                gap: 0,
            }
        );
        let between = resolve(pos2(60.0, 10.0)).expect("over the strip");
        assert_eq!(
            between.target,
            DropTarget::Tab {
                stack: StackAddr::new(DockSide::Left, 0, 0),
                gap: 1,
            }
        );
        let past = resolve(pos2(190.0, 10.0)).expect("over the strip");
        assert_eq!(
            past.target,
            DropTarget::Tab {
                stack: StackAddr::new(DockSide::Left, 0, 0),
                gap: 2,
            }
        );
    }

    /// ★★ A corner is in two bands at once, and the nearer edge wins **as a
    /// fraction of that edge's band** rather than in points. The fixture's
    /// bands differ — 50 across, 45 down — so a point 20 from the left edge and
    /// 19 from the top is nearer the top edge in points and belongs to the left
    /// zone anyway. Measuring in points would get this kind of case, and only
    /// this kind, backwards.
    #[test]
    fn a_corner_belongs_to_the_nearest_edge_measured_as_a_fraction_of_its_band() {
        let body = body_of(
            &geometry_for(&layout(), SIDE),
            StackAddr::new(DockSide::Left, 0, 0),
        )
        .expect("the first stack was laid out");
        let compass = Compass::new(body);
        assert_eq!(band(body.width()), 50.0, "200 wide, a quarter of it");
        assert_eq!(band(body.height()), 45.0, "180 tall, a quarter of it");

        let pos = pos2(body.left() + 20.0, body.top() + 19.0);
        assert_eq!(
            compass.zone_at(pos),
            Some(DropZone::Left),
            "20/50 is a smaller fraction of its band than 19/45 is of its"
        );
        let pos = pos2(body.left() + 20.0, body.top() + 17.0);
        assert_eq!(
            compass.zone_at(pos),
            Some(DropZone::Top),
            "17/45 is now the smaller fraction"
        );
    }

    /// ★★ The drawn shape and the hit test are one definition, measured
    /// against each other: every point on a fine grid lands in the zone whose
    /// [`Compass::outline`] contains it, by an inside-a-convex-polygon test
    /// written here rather than borrowed from the code under test.
    ///
    /// What this cannot catch is an error the two share — a wrong band width
    /// moves both together. [`band`] is asserted directly, above.
    #[test]
    fn every_point_lands_in_the_zone_whose_outline_contains_it() {
        let compass = Compass::new(Rect::from_min_max(pos2(10.0, 30.0), pos2(210.0, 130.0)));
        let mut counts = [0_usize; 5];
        for i in 0..=100 {
            for j in 0..=100 {
                #[allow(clippy::cast_precision_loss)]
                let pos = pos2(
                    compass.area.left() + compass.area.width() * i as f32 / 100.0,
                    compass.area.top() + compass.area.height() * j as f32 / 100.0,
                );
                let zone = compass.zone_at(pos).expect("inside the area");
                counts[DropZone::ALL.iter().position(|z| *z == zone).unwrap()] += 1;
                let inside = |z: DropZone| contains(&compass.outline(z), pos);
                assert!(
                    inside(zone),
                    "{pos:?} resolved to {zone:?}, which is not the quad drawn for it"
                );
                for other in DropZone::ALL.into_iter().filter(|z| *z != zone) {
                    assert!(
                        !inside(other) || on_a_seam(&compass, pos),
                        "{pos:?} resolved to {zone:?} but is also inside {other:?}"
                    );
                }
            }
        }
        for (zone, n) in DropZone::ALL.into_iter().zip(counts) {
            assert!(n > 200, "{zone:?} was only reached {n} times in the sweep");
        }
    }

    /// Whether `pos` is on or inside the clockwise convex quad `quad`.
    fn contains(quad: &[Pos2; 4], pos: Pos2) -> bool {
        (0..4).all(|i| {
            let (a, b) = (quad[i], quad[(i + 1) % 4]);
            let cross = (b.x - a.x) * (pos.y - a.y) - (b.y - a.y) * (pos.x - a.x);
            cross >= -SEAM_PTS
        })
    }

    /// Whether `pos` sits on a boundary between two zones, where belonging to
    /// both quads is the shapes touching rather than overlapping.
    fn on_a_seam(compass: &Compass, pos: Pos2) -> bool {
        DropZone::ALL.into_iter().any(|z| {
            let quad = compass.outline(z);
            (0..4).any(|i| {
                let (a, b) = (quad[i], quad[(i + 1) % 4]);
                let cross = (b.x - a.x) * (pos.y - a.y) - (b.y - a.y) * (pos.x - a.x);
                cross.abs() < SEAM_PTS
            })
        })
    }

    /// How close to a quad's edge counts as on it, in the cross-product units
    /// the test's own containment uses.
    const SEAM_PTS: f32 = 0.001;

    /// ★ The invariant that makes the compass safe to wire to a release: every
    /// point over the dock resolves to something the grammar accepts, and
    /// acting on it leaves a layout the dock can draw.
    #[test]
    fn every_point_over_the_dock_resolves_to_a_move_that_leaves_a_drawable_layout() {
        let base = layout();
        let geometry = geometry_for(&base, SIDE);
        let dragged = PanelId::new("fields");
        let home = StackAddr::from(base.find(&dragged).expect("the fixture docks it"));
        let (mut landed, mut missed, mut declined) = (0_usize, 0_usize, 0_usize);
        for i in 0..=80 {
            for j in 0..=80 {
                #[allow(clippy::cast_precision_loss)]
                let pos = pos2(
                    SIDE.left() + SIDE.width() * i as f32 / 80.0,
                    SIDE.top() + SIDE.height() * j as f32 / 80.0,
                );
                let Some(landing) = base.resolve_drop(&geometry, pos) else {
                    missed += 1;
                    continue;
                };
                landed += 1;
                assert!(
                    base.accepts_drop(landing.target),
                    "{pos:?} resolved to {:?}, which the layout it was resolved against declines",
                    landing.target
                );
                let mut after = base.clone();
                if !after.move_panel(&dragged, landing.target) {
                    declined += 1;
                    assert_eq!(
                        after, base,
                        "{pos:?}: a declined landing changed the layout"
                    );
                    assert_eq!(
                        landing.over, home,
                        "{pos:?} declined a landing that was not over the panel's own stack"
                    );
                }
                assert!(
                    after.is_normalized(),
                    "{pos:?} left a layout needing repair"
                );
                assert_eq!(
                    after.panels().count(),
                    base.panels().count(),
                    "{pos:?} lost or duplicated a panel"
                );
            }
        }
        assert!(
            landed > 6000,
            "only {landed} of the sweep's points were over a compartment"
        );
        assert_eq!(
            missed, 0,
            "the fixture tiles the side, so nothing should miss"
        );
        assert!(
            declined > 0,
            "nothing was declined, so the branch that must not mutate was never taken"
        );
    }

    #[test]
    fn a_point_over_no_compartment_resolves_to_nothing() {
        assert_eq!(resolve(pos2(600.0, 200.0)), None);
        assert_eq!(resolve(pos2(200.0, 600.0)), None);
        // And the division itself declines a point outside it, rather than
        // reporting the zone it would be in if the rectangle were bigger.
        assert_eq!(Compass::new(SIDE).zone_at(pos2(600.0, 200.0)), None);
        assert_eq!(
            Compass::new(SIDE).zone_at(pos2(399.0, 200.0)),
            Some(DropZone::Right)
        );
    }

    /// ★ The cap, stated as the thing it prevents: a tall compartment's bottom
    /// zone stays a band rather than becoming a third of the panel.
    #[test]
    fn a_large_compartment_gets_a_capped_edge_band_and_keeps_a_dominant_centre() {
        let tall = Compass::new(Rect::from_min_max(pos2(0.0, 0.0), pos2(200.0, 900.0)));
        assert_eq!(
            tall.centre.top(),
            EDGE_MAX_PTS,
            "a quarter of 900 is over the cap"
        );
        assert_eq!(tall.centre.bottom(), 900.0 - EDGE_MAX_PTS);
        assert_eq!(tall.centre.left(), 50.0, "a quarter of 200 is under it");
        assert!(
            tall.centre.height() > tall.area.height() * 0.8,
            "the cap is what keeps the centre dominant on a tall compartment"
        );
    }

    #[test]
    fn a_compartment_with_no_area_is_all_centre_rather_than_a_division_by_zero() {
        let flat = Compass::new(Rect::from_min_max(pos2(5.0, 5.0), pos2(5.0, 60.0)));
        assert_eq!(flat.zone_at(pos2(5.0, 30.0)), Some(DropZone::Centre));
        let empty = Compass::new(Rect::from_min_max(pos2(5.0, 5.0), pos2(5.0, 5.0)));
        assert_eq!(empty.zone_at(pos2(5.0, 5.0)), Some(DropZone::Centre));
    }
}
