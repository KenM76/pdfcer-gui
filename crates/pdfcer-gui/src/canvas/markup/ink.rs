//! # `canvas::markup::ink` — freehand, and the hundreds of points nobody asked
//! for
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/markup/ink.md`.

use egui::Pos2;
use pdfcer_core::page_tree::Page;

use super::{Geometry, MarkupKind};
use crate::app::actions::Action;
use crate::canvas::gesture::{DragKind, Phase};
use crate::canvas::mapping::PageMapping;

/// Where the trail lives between the frames of one drag.
// ui-text-exempt: an `egui::Id` source string, never displayed.
const INK_MEMORY_KEY: &str = "pdfcer-markup-ink-trail";

/// **The simplification tolerance of the SHIPPED pen**, in PDF points — the
/// largest distance a removed point may have lain from the line that replaced
/// it.
pub const SIMPLIFY_TOLERANCE_PTS: f32 = (super::PEN_WIDTH_PTS as f32) / 4.0;

/// The pointer trail of one freehand drag, in **canvas space**.
#[derive(Debug, Clone, PartialEq)]
struct Trail {
    /// Every distinct position the pointer has been at, in order, including the
    /// press origin as the first entry.
    points: Vec<Pos2>,
}

/// Read the trail without creating one.
fn read(ctx: &egui::Context) -> Option<Trail> {
    ctx.data_mut(|d| d.get_temp::<Trail>(egui::Id::new(INK_MEMORY_KEY)))
}

/// Write the trail back.
fn store(ctx: &egui::Context, trail: Trail) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(INK_MEMORY_KEY), trail));
}

/// Forget the trail.
fn clear(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<Trail>(egui::Id::new(INK_MEMORY_KEY)));
}

/// **Keep the trail alive exactly while a freehand drag is** — §2's derived
/// lifetime, in one line.
pub(in crate::canvas) fn sync(ctx: &egui::Context, active: Option<DragKind>) {
    let freehand = matches!(active, Some(DragKind::Markup(kind)) if kind.is_freehand());
    if !freehand && read(ctx).is_some() {
        clear(ctx);
    }
}

/// **Everything one frame of a freehand drag is resolved against.**
pub(in crate::canvas) struct Stroke<'a> {
    /// Where the trail is stored between the frames of this drag.
    pub ctx: &'a egui::Context,
    /// Which markup kind is armed. Checked rather than assumed — see [`drag`]'s
    /// family guard.
    pub kind: MarkupKind,
    /// Where the button went down, in canvas space. Seeded as the trail's first
    /// point; see [`drag`]'s section on why it is appended rather than used as
    /// an anchor.
    pub from: Pos2,
    /// Where the pointer is now, in canvas space.
    pub to: Pos2,
    /// Draw the trail, or commit the stroke.
    pub phase: Phase,
    /// The page the stroke is authored onto.
    pub page_index: usize,
    /// That page, for the canvas→PDF transform. `None` when the frame has none,
    /// which is refused rather than authored.
    pub page: Option<&'a Page>,
}

/// Apply one frame of a freehand drag: extend the trail, or commit the stroke.
pub(in crate::canvas) fn drag(
    pen: super::pen::Pen,
    stroke: Stroke<'_>,
    actions: &mut Vec<Action>,
) -> Option<Vec<Pos2>> {
    let Stroke {
        ctx,
        kind,
        from,
        to,
        phase,
        page_index,
        page,
    } = stroke;
    if !kind.is_freehand() {
        return None;
    }
    let mut trail = read(ctx).unwrap_or_else(|| Trail { points: vec![from] });
    // Distinct positions only — §3.1. A stationary pointer under a held button
    // reports `dragged` on every frame, and each of those would otherwise be two
    // more `Real`s in `/InkList` and one more `l` in the appearance stream.
    if trail.points.last() != Some(&to) {
        trail.points.push(to);
    }
    let raw = trail.points.len();
    // The PEN's tolerance, not the shipped constant — §3.2's rule.
    //
    // `SIMPLIFY_TOLERANCE_PTS` is a `const` derived from the pen's *default*
    // 2 pt width. At a 0.25 pt pen — the width that exists to match a CAD
    // sheet's own linework — a fixed 0.5 pt tolerance is four times the
    // stroke's half-width, so the simplified centreline can leave the body of
    // the stroke entirely and the operator gets a visibly different curve from
    // the one they drew. See `Pen::simplify_tolerance_pts` for the table.
    //
    // Read here rather than inside `simplify`, so that function stays a pure
    // `(points, tolerance)` and its measurement tests can sweep the tolerance
    // without constructing a pen.
    let kept = simplify(&trail.points, pen.simplify_tolerance_pts());
    store(ctx, trail);

    if phase == Phase::InFlight {
        return Some(kept);
    }

    // From here on the gesture is over: the trail must not survive into the next
    // stroke whatever happens below, including every refusal.
    clear(ctx);
    let Some(page) = page else {
        super::decline(kind, page_index, super::Refusal::NoPage);
        return None;
    };
    let mut points: Vec<(f64, f64)> = Vec::with_capacity(kept.len());
    for at in &kept {
        let Some(point) = super::vertex::page_point(*at, page) else {
            super::decline(kind, page_index, super::Refusal::DegeneratePage);
            return None;
        };
        points.push(point);
    }
    match super::action(kind, page_index, Geometry::Strokes(vec![points]), pen) {
        Ok(raised) => {
            super::trace_commit(
                kind,
                page_index,
                // ui-text-exempt: diagnostic trace, never displayed in the UI.
                //
                // `raw` BESIDE `kept` — §3.3. A build whose simplification did
                // nothing at all would emit an otherwise identical line, and the
                // only external evidence that this feature works is the two
                // numbers differing. It is also how the *real* retention figure
                // in §3.3 is obtained, as opposed to the synthetic one.
                &format!("raw={raw} kept={}", kept.len()),
            );
            actions.push(raised);
        }
        Err(reason) => super::decline(kind, page_index, reason),
    }
    None
}

/// Paint the freehand trail.
pub(in crate::canvas) fn draw_preview(
    painter: &egui::Painter,
    map: &PageMapping,
    trail: &[Pos2],
    pen: super::pen::Pen,
) {
    if trail.len() < 2 {
        return;
    }
    // DOCUMENT COLOUR: the pen, read from the one place `spec` reads it.
    let stroke = egui::Stroke::new(
        super::pen_px(map, pen),
        super::pen_color(MarkupKind::Ink, pen),
    );
    let screen: Vec<Pos2> = trail.iter().map(|p| map.to_screen(*p)).collect();
    painter.add(egui::Shape::line(screen, stroke));
}

/// **Ramer–Douglas–Peucker**, iteratively.
#[must_use]
fn simplify(points: &[Pos2], tolerance: f32) -> Vec<Pos2> {
    // `is_nan()` beside `<= 0.0` rather than `!(tolerance > 0.0)`, which says the
    // same thing and which clippy refuses on a partially ordered type. A NaN
    // tolerance would make every comparison below false and silently keep every
    // point, which is a simplification that did nothing wearing a green test.
    if points.len() < 3 || tolerance <= 0.0 || tolerance.is_nan() {
        return points.to_vec();
    }
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    let mut stack = vec![(0_usize, points.len() - 1)];
    while let Some((first, last)) = stack.pop() {
        if last <= first + 1 {
            continue;
        }
        let (mut worst, mut worst_at) = (0.0_f32, first);
        for (i, p) in points.iter().enumerate().take(last).skip(first + 1) {
            let d = distance_to_segment(*p, points[first], points[last]);
            if d > worst {
                worst = d;
                worst_at = i;
            }
        }
        if worst > tolerance {
            keep[worst_at] = true;
            stack.push((first, worst_at));
            stack.push((worst_at, last));
        }
    }
    points
        .iter()
        .zip(keep)
        .filter_map(|(p, k)| k.then_some(*p))
        .collect()
}

/// Perpendicular distance from `p` to the segment `a`–`b`.
fn distance_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let len2 = ab.length_sq();
    if len2 <= 0.0 || len2.is_nan() {
        return (p - a).length();
    }
    let t = (((p - a).x * ab.x + (p - a).y * ab.y) / len2).clamp(0.0, 1.0);
    (p - (a + ab * t)).length()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A trail shaped like a hand-drawn stroke: a sweeping arc with two
    /// lateral disturbances on it, sampled at 60 Hz for four seconds.
    fn hand_drawn_trail(n: usize) -> Vec<Pos2> {
        (0..n)
            .map(|i| {
                #[allow(clippy::cast_precision_loss)]
                let t = i as f32 / (n as f32 - 1.0);
                let angle = t * std::f32::consts::PI * 0.75;
                let (x, y) = (200.0 + 160.0 * angle.cos(), 200.0 + 160.0 * angle.sin());
                // The slow wander, across the direction of travel.
                let wander = (t * 47.0).sin() * 1.2;
                // …and the fast jitter: a cheap deterministic hash of the sample
                // index, mapped to ±0.3 pt. Deterministic so the numbers §3.3
                // quotes can be re-measured.
                #[allow(clippy::cast_precision_loss)]
                let hashed = ((i as u32).wrapping_mul(2_654_435_761) >> 8) % 601;
                #[allow(clippy::cast_precision_loss)]
                let jitter = (hashed as f32 / 1000.0) - 0.3;
                // Applied along the RADIAL direction, which for a circular arc
                // is the normal — i.e. across the direction of travel. The first
                // version of this fixture used `(sin, -cos)`, which is the
                // *tangent*, so both disturbances merely re-spaced the samples
                // along a path whose shape they never changed; RDP removed them
                // almost for free and the fixture flattered the tolerance. The
                // measurement is only worth quoting because that was caught by
                // computing the retention independently and finding it too good.
                let off = wander + jitter;
                Pos2::new(x + off * angle.cos(), y + off * angle.sin())
            })
            .collect()
    }

    /// **The measured retention at the shipped tolerance, and the bound RDP
    /// promises.**
    #[test]
    fn the_measured_retention_at_the_shipped_tolerance() {
        let trail = hand_drawn_trail(240);
        assert_eq!(trail.len(), 240, "the fixture is 4 s at 60 Hz");

        let mut previous = usize::MAX;
        for tolerance in [0.125_f32, 0.25, 0.5, 1.0, 2.0] {
            let kept = simplify(&trail, tolerance);
            // Property 1: every dropped point lay within `tolerance` of the
            // polyline that replaced it. Measured against the SIMPLIFIED
            // polyline, which is the thing that is actually drawn.
            let worst = trail
                .iter()
                .map(|p| {
                    kept.windows(2)
                        .map(|w| distance_to_segment(*p, w[0], w[1]))
                        .fold(f32::INFINITY, f32::min)
                })
                .fold(0.0_f32, f32::max);
            assert!(
                worst <= tolerance + 1e-3,
                "tolerance {tolerance}: a point lay {worst} from the simplified polyline, \
                 which breaks the bound the shipped tolerance is derived from"
            );
            // Property 2: monotonic.
            assert!(
                kept.len() <= previous,
                "tolerance {tolerance} kept {} where {previous} were kept at a tighter one",
                kept.len()
            );
            previous = kept.len();
            // Printed, not merely asserted, so the figures quoted in the module
            // header's section 3.3 can be RE-MEASURED rather than trusted:
            // `cargo test -p pdfcer-gui --lib the_measured_retention -- --nocapture`.
            // A number in prose that nobody can reproduce is the shape of claim
            // this project has had to correct four times.
            eprintln!(
                // ui-text-exempt: a test measurement, printed to the test
                // harness's stderr and never to an operator.
                "ink-simplify tolerance={tolerance} kept={} of {} worst={worst}",
                kept.len(),
                trail.len()
            );
            if (tolerance - SIMPLIFY_TOLERANCE_PTS).abs() < 1e-6 {
                // The shipped value. Asserted as a band rather than an exact
                // count: the figure quoted in section 3.3 is what this fixture
                // produces, and a fixture-sensitive equality would fail on a
                // compiler that rounds `sin` one ulp differently.
                assert!(
                    kept.len() < trail.len() / 4,
                    "at the shipped tolerance {tolerance} pt the trail retained {} of {} \
                     points ({}%), which is not the reduction section 3.3 records",
                    kept.len(),
                    trail.len(),
                    kept.len() * 100 / trail.len()
                );
                assert!(
                    worst > tolerance / 2.0,
                    "at the shipped tolerance the worst deviation was only {worst} pt, so this \
                     fixture is not exercising the bound and the table in section 3.3 is \
                     measuring an easier curve than it claims — which is exactly how the first \
                     version of this fixture was wrong"
                );
                assert!(
                    kept.len() >= 8,
                    "at the shipped tolerance the trail retained only {} points, which is a \
                     simplification that has eaten the stroke rather than tidied it",
                    kept.len()
                );
            }
        }
        // The ends always survive: a stroke that lost its first or last point
        // would start or stop somewhere the operator did not.
        let kept = simplify(&trail, SIMPLIFY_TOLERANCE_PTS);
        assert_eq!(kept.first(), trail.first());
        assert_eq!(kept.last(), trail.last());
    }

    /// A straight run collapses to its two ends, and a zero tolerance keeps
    /// everything — the two extremes that say the algorithm is doing anything at
    /// all.
    #[test]
    fn a_straight_run_collapses_and_a_zero_tolerance_keeps_everything() {
        let straight: Vec<Pos2> = (0..50).map(|i| Pos2::new(i as f32 * 3.0, 100.0)).collect();
        assert_eq!(simplify(&straight, SIMPLIFY_TOLERANCE_PTS).len(), 2);
        assert_eq!(simplify(&straight, 0.0).len(), 50);
        assert_eq!(simplify(&straight, -1.0).len(), 50);
        // Fewer than three points has nothing to remove.
        let two = vec![Pos2::ZERO, Pos2::new(10.0, 10.0)];
        assert_eq!(simplify(&two, 100.0), two);
    }

    /// **A hairpin keeps its point**, which is what distinguishes the segment
    /// distance from the line distance.
    #[test]
    fn a_hairpin_keeps_its_apex_where_an_infinite_line_would_lose_it() {
        let hairpin = vec![
            Pos2::new(0.0, 0.0),
            Pos2::new(50.0, 0.0),
            Pos2::new(100.0, 0.0),
            Pos2::new(50.0, 0.0),
            Pos2::new(0.0, 0.0),
        ];
        let kept = simplify(&hairpin, SIMPLIFY_TOLERANCE_PTS);
        assert!(
            kept.contains(&Pos2::new(100.0, 0.0)),
            "the fold's apex was deleted: {kept:?}"
        );
        // …and the distance function is why.
        let apex = Pos2::new(100.0, 0.0);
        let (a, b) = (Pos2::ZERO, Pos2::ZERO);
        assert!(distance_to_segment(apex, a, b) > 99.0);
    }

    /// **A stationary pointer contributes one point, not one per frame.**
    #[test]
    fn a_stationary_pointer_contributes_one_point() {
        let ctx = egui::Context::default();
        let mut actions = Vec::new();
        let at = Pos2::new(40.0, 40.0);
        for _ in 0..50 {
            let _ = drag(
                crate::canvas::markup::pen::Pen::default(),
                Stroke {
                    ctx: &ctx,
                    kind: MarkupKind::Ink,
                    from: at,
                    to: at,
                    phase: Phase::InFlight,
                    page_index: 0,
                    page: None,
                },
                &mut actions,
            );
        }
        let trail = read(&ctx).expect("a trail exists");
        assert_eq!(
            trail.points.len(),
            1,
            "fifty identical frames produced {} points",
            trail.points.len()
        );
        assert!(actions.is_empty());
    }

    /// **The trail is derived, so every way a drag can end discards it.**
    #[test]
    fn every_way_a_drag_can_end_discards_the_trail() {
        let ctx = egui::Context::default();
        let mut actions = Vec::new();
        for step in 0..5 {
            let _ = drag(
                crate::canvas::markup::pen::Pen::default(),
                Stroke {
                    ctx: &ctx,
                    kind: MarkupKind::Ink,
                    from: Pos2::ZERO,
                    to: Pos2::new(step as f32 * 10.0, 5.0),
                    phase: Phase::InFlight,
                    page_index: 0,
                    page: None,
                },
                &mut actions,
            );
        }
        assert!(read(&ctx).is_some(), "a trail is in flight");

        // Still in flight: nothing is discarded.
        sync(&ctx, Some(DragKind::Markup(MarkupKind::Ink)));
        assert!(read(&ctx).is_some());

        // Escaped, interrupted, or simply over — all three are `None`.
        sync(&ctx, None);
        assert!(read(&ctx).is_none(), "the trail must not outlive its drag");

        // …and another kind of drag is not this one, so a band started after an
        // interrupted stroke does not inherit its points.
        let _ = drag(
            crate::canvas::markup::pen::Pen::default(),
            Stroke {
                ctx: &ctx,
                kind: MarkupKind::Ink,
                from: Pos2::ZERO,
                to: Pos2::new(9.0, 9.0),
                phase: Phase::InFlight,
                page_index: 0,
                page: None,
            },
            &mut actions,
        );
        sync(&ctx, Some(DragKind::Markup(MarkupKind::Rectangle)));
        assert!(read(&ctx).is_none());
    }

    /// A non-freehand kind is refused here, exactly as a non-band kind is refused
    /// by [`super::band::drag`] — the two guards are the two halves of one
    /// routing rule, and a build that lost the routing would otherwise author a
    /// one-segment `/Ink` for every rectangle drawn.
    #[test]
    fn a_non_freehand_kind_is_refused_by_the_freehand_gesture() {
        let ctx = egui::Context::default();
        let mut actions = Vec::new();
        for kind in [
            MarkupKind::Rectangle,
            MarkupKind::Highlight,
            MarkupKind::Polygon,
        ] {
            assert_eq!(
                drag(
                    crate::canvas::markup::pen::Pen::default(),
                    Stroke {
                        ctx: &ctx,
                        kind,
                        from: Pos2::ZERO,
                        to: Pos2::new(10.0, 10.0),
                        phase: Phase::InFlight,
                        page_index: 0,
                        page: None,
                    },
                    &mut actions,
                ),
                None,
                "{kind:?}"
            );
        }
        assert!(actions.is_empty());
        assert!(read(&ctx).is_none());
    }

    /// **The preview draws the points that will be committed**, not the raw
    /// trail.
    #[test]
    fn the_preview_is_the_simplified_trail_that_will_be_authored() {
        let ctx = egui::Context::default();
        let mut actions = Vec::new();
        let trail = hand_drawn_trail(120);
        let mut last = None;
        for (i, p) in trail.iter().enumerate() {
            last = drag(
                crate::canvas::markup::pen::Pen::default(),
                Stroke {
                    ctx: &ctx,
                    kind: MarkupKind::Ink,
                    from: trail[0],
                    to: *p,
                    phase: Phase::InFlight,
                    page_index: 0,
                    page: None,
                },
                &mut actions,
            );
            assert!(last.is_some(), "frame {i} drew nothing");
        }
        let previewed = last.expect("a preview");
        let raw = read(&ctx).expect("a trail").points;
        assert!(
            previewed.len() < raw.len(),
            "the preview drew every raw point ({} of {})",
            previewed.len(),
            raw.len()
        );
        assert_eq!(
            previewed,
            simplify(&raw, SIMPLIFY_TOLERANCE_PTS),
            "the preview must be the exact polyline the release authors"
        );
    }

    /// The tolerance is derived from the pen and stays so — a test rather than a
    /// comment, because §3.2's whole claim is that the two move together.
    #[test]
    fn the_tolerance_is_half_the_pens_half_width() {
        #[allow(clippy::cast_possible_truncation)]
        let half_width = (super::super::PEN_WIDTH_PTS as f32) / 2.0;
        assert!(
            (SIMPLIFY_TOLERANCE_PTS - half_width / 2.0).abs() < 1e-6,
            "the simplified centreline must stay strictly inside the drawn stroke"
        );
    }

    /// **The guarantee holds at EVERY width the operator can set**, not just
    /// at the shipped one.
    #[test]
    fn the_guarantee_holds_at_every_width_the_operator_can_set() {
        use super::super::pen::{MAX_WIDTH_PTS, MIN_WIDTH_PTS, Pen};

        // The two ends and the shipped middle. The ends are what matter: a
        // derivation that had been pinned to the default would pass at the
        // middle and fail at both ends, which is exactly the shape of the
        // defect.
        for width in [MIN_WIDTH_PTS, super::super::PEN_WIDTH_PTS, MAX_WIDTH_PTS] {
            let pen = Pen {
                width_pts: width,
                ..Pen::default()
            };
            #[allow(clippy::cast_possible_truncation)]
            let half_width = (width as f32) / 2.0;
            let tolerance = pen.simplify_tolerance_pts();
            assert!(
                (tolerance - half_width / 2.0).abs() < 1e-6,
                "at a {width} pt pen the tolerance is {tolerance} pt but the \
                 half-width is {half_width} pt — the simplified centreline can \
                 leave the stroke it is meant to stay inside"
            );
        }
    }

    /// The shipped constant and the shipped pen agree.
    #[test]
    fn the_shipped_constant_matches_the_shipped_pen() {
        assert!(
            (SIMPLIFY_TOLERANCE_PTS - super::super::pen::Pen::default().simplify_tolerance_pts())
                .abs()
                < 1e-6,
            "§3.3's measurements are quoted against a tolerance the shipped \
             build no longer uses"
        );
    }
}
