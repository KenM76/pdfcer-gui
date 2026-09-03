//! # `canvas::markup` tests — the pure rules, enumerated
//!
//! Split out of [`super`] under **R2** on 2026-08-28, when the markup router
//! landed and took that file past 1,500 lines.
//!
//! ## ★★ The seam is the one that module's own §5 already draws
//!
//! [`super`]'s header states it: *"[`super::spec`] and [`super::action`] are
//! pure functions of plain data, so every rule above is testable with no window
//! and no document."* This is the enumeration of those rules, and the parent is
//! the rules — the same split `gesture::meaning` and `canvas::keys` took in the
//! same week.

// ★★ The INNER attribute, not just the `mod tests;` declaration in the parent.
// `check-ui-strings.sh`'s exclusion 2b recognises a whole test file from the
// FILE rather than from its name, and without it every assertion message here
// is reported as operator-facing copy.
#![cfg(test)]

use super::*;

/// A band geometry from two corners.
fn band(start: (f64, f64), end: (f64, f64)) -> Geometry {
    Geometry::Band { start, end }
}

// -----------------------------------------------------------------
// ★ The families partition the kinds
// -----------------------------------------------------------------

/// ★ **Every kind belongs to exactly one gesture family.**
///
/// The property `canvas::interact`'s routing and
/// `gesture::press_kind`'s early return both rest on, asserted as a
/// partition rather than as three membership lists. A kind in **two**
/// families would be reached by two gestures — a press that both started a
/// band and placed a vertex — and a kind in **none** would arm a tool whose
/// press means nothing, which is the *"visible control, silently inert"*
/// failure with a crosshair on it.
#[test]
fn the_three_families_partition_every_kind() {
    for &kind in MarkupKind::ALL {
        let members = usize::from(kind.is_band())
            + usize::from(kind.is_vertex())
            + usize::from(kind.is_freehand());
        assert_eq!(members, 1, "{kind:?} belongs to {members} families");
    }
    // …and each family is non-empty, so the partition is a real one rather
    // than "everything is a band".
    assert!(MarkupKind::ALL.iter().any(|k| k.is_band()));
    assert!(MarkupKind::ALL.iter().any(|k| k.is_vertex()));
    assert!(MarkupKind::ALL.iter().any(|k| k.is_freehand()));
    // The rect predicate is a refinement of the band family, not a fourth
    // one: an Arrow is a band and is not a rect.
    for &kind in MarkupKind::ALL {
        assert!(!kind.is_rect() || kind.is_band(), "{kind:?}");
    }
    assert!(!MarkupKind::Arrow.is_rect());
}

// -----------------------------------------------------------------
// ★ The arrow keeps its direction
// -----------------------------------------------------------------

/// ★ **An arrow dragged up-and-left keeps its head at the end the operator
/// dragged to.**
///
/// The `:5624-5627` decision, asserted in the direction that a normalising
/// implementation fails. A normalised rect would report
/// `start = (min, min)`, which for this drag is the **head**, so the
/// arrowhead would be at the tail — and, with a single head, nothing in the
/// document would say so.
#[test]
fn an_arrow_dragged_backwards_keeps_its_head_at_the_end() {
    let tail = (400.0, 500.0);
    let head = (120.0, 700.0); // up and to the left: both axes reversed
    let Some(MarkupSpec::Line {
        start,
        end,
        endings,
        ..
    }) = spec_default_pen(MarkupKind::Arrow, &band(tail, head))
    else {
        panic!("Arrow must author a /Line");
    };
    assert_eq!(start, tail, "the tail must stay the tail");
    assert_eq!(end, head, "the head must stay the head");
    assert_eq!(
        endings,
        (LineEnding::None, LineEnding::OpenArrow),
        "one head, at the end of the drag — `text/commands.rs` promises \
         \"drag from the tail to the head\""
    );
}

/// …and every rectangle kind IS normalised, in all four drag directions,
/// because a `Rect` with `llx > urx` is not a rectangle any reader draws.
///
/// Asserted over all four kinds and all four directions rather than one
/// case, because the failure is per-kind: it is exactly the shape of
/// mistake that gets fixed for Rectangle and left in Ellipse.
#[test]
fn every_rectangle_kind_is_normalised_in_all_four_drag_directions() {
    let corners = [(100.0_f64, 200.0_f64), (300.0, 500.0)];
    for kind in [
        MarkupKind::Rectangle,
        MarkupKind::Ellipse,
        MarkupKind::Highlight,
    ] {
        assert!(kind.is_rect(), "{kind:?}");
        for (a, b) in [
            (corners[0], corners[1]),
            (corners[1], corners[0]),
            ((corners[0].0, corners[1].1), (corners[1].0, corners[0].1)),
            ((corners[1].0, corners[0].1), (corners[0].0, corners[1].1)),
        ] {
            let rect = match spec_default_pen(kind, &band(a, b)) {
                Some(MarkupSpec::Square { rect, .. } | MarkupSpec::Circle { rect, .. }) => rect,
                Some(MarkupSpec::TextMarkup { quads, .. }) => {
                    assert_eq!(quads.len(), 1, "a highlight authors exactly one quad");
                    PageRect::from_corners(100.0, 200.0, 300.0, 500.0)
                }
                other => panic!("{kind:?} authored {other:?}"),
            };
            assert!(
                rect.llx < rect.urx && rect.lly < rect.ury,
                "{kind:?} {rect:?}"
            );
            assert!((rect.llx - 100.0).abs() < 1e-9 && (rect.ury - 500.0).abs() < 1e-9);
        }
    }
}

/// Each kind authors its own subtype, and nothing borrows another's.
#[test]
fn each_kind_authors_its_own_subtype() {
    let (a, b) = ((10.0, 20.0), (30.0, 40.0));
    let run = Geometry::Vertices(vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)]);
    let ink = Geometry::Strokes(vec![vec![(0.0, 0.0), (1.0, 1.0), (2.0, 0.5)]]);
    assert!(matches!(
        spec_default_pen(MarkupKind::Rectangle, &band(a, b)),
        Some(MarkupSpec::Square { .. })
    ));
    assert!(matches!(
        spec_default_pen(MarkupKind::Ellipse, &band(a, b)),
        Some(MarkupSpec::Circle { .. })
    ));
    assert!(matches!(
        spec_default_pen(MarkupKind::Arrow, &band(a, b)),
        Some(MarkupSpec::Line { .. })
    ));
    assert!(matches!(
        spec_default_pen(MarkupKind::Highlight, &band(a, b)),
        Some(MarkupSpec::TextMarkup {
            kind: TextMarkupKind::Highlight,
            ..
        })
    ));
    assert!(matches!(
        spec_default_pen(MarkupKind::PolyLine, &run),
        Some(MarkupSpec::PolyLine { .. })
    ));
    assert!(matches!(
        spec_default_pen(MarkupKind::Polygon, &run),
        Some(MarkupSpec::Polygon { .. })
    ));
    // ★ The cloud row is the one that could plausibly have been left out,
    // and leaving it out is exactly the defect this test is for: a cloud IS
    // a `/Polygon` in the file, so an arm that fell through to
    // `MarkupSpec::Polygon` would author a legal annotation, render, save,
    // reopen — and simply have no cloudy border. There is no error, no
    // refusal and no trace line to notice; the only symptom is that the
    // revision cloud tool draws polygons.
    assert!(matches!(
        spec_default_pen(MarkupKind::Cloud, &run),
        Some(MarkupSpec::Cloud { .. })
    ));
    assert!(matches!(
        spec_default_pen(MarkupKind::Ink, &ink),
        Some(MarkupSpec::Ink { .. })
    ));
    assert_eq!(
        MarkupKind::ALL.len(),
        8,
        "a ninth kind must be given a subtype here, not left to inherit one"
    );
}

/// ★ **A vertex run and an ink stroke reach the file in the order they were
/// drawn**, point for point.
///
/// The one-derivation promise for the two list-driven families. `/Vertices`
/// and `/InkList` are sequences whose consecutive entries are joined by a
/// segment, so a build that sorted, de-duplicated or re-ordered them would
/// author a *different figure* from the one the preview drew — and the
/// difference would only be visible after saving.
///
/// The polygon row carries the extra claim: the closing vertex is **not**
/// appended, because `/Polygon` closes by specification and a duplicate
/// first point would author a zero-length segment.
#[test]
fn a_vertex_run_and_an_ink_stroke_are_authored_in_drawing_order() {
    let points = vec![(10.0, 10.0), (40.0, 12.0), (25.0, 60.0), (12.0, 30.0)];
    let Some(MarkupSpec::PolyLine { vertices, .. }) =
        spec_default_pen(MarkupKind::PolyLine, &Geometry::Vertices(points.clone()))
    else {
        panic!("PolyLine must author a /PolyLine");
    };
    assert_eq!(vertices, points, "the run must arrive as it left");

    let Some(MarkupSpec::Polygon { vertices, .. }) =
        spec_default_pen(MarkupKind::Polygon, &Geometry::Vertices(points.clone()))
    else {
        panic!("Polygon must author a /Polygon");
    };
    assert_eq!(
        vertices, points,
        "a /Polygon closes by specification; appending the first point again \
         would author a duplicate vertex and a zero-length closing segment"
    );

    let strokes = vec![vec![(0.0, 0.0), (5.5, 9.25), (12.0, 3.0)]];
    let Some(MarkupSpec::Ink {
        strokes: authored, ..
    }) = spec_default_pen(MarkupKind::Ink, &Geometry::Strokes(strokes.clone()))
    else {
        panic!("Ink must author an /Ink");
    };
    assert_eq!(authored, strokes);
}

/// ★ **No geometric markup is authored with a filled interior**, so a
/// comment never hides the drawing it is a comment about.
///
/// The Polygon row is the new one and is the one that could plausibly have
/// gone the other way: `/Polygon` is the only new kind with an `/IC` slot.
#[test]
fn a_shape_markup_is_never_filled() {
    let cases: [(MarkupKind, Geometry); 3] = [
        (MarkupKind::Rectangle, band((0.0, 0.0), (10.0, 10.0))),
        (MarkupKind::Ellipse, band((0.0, 0.0), (10.0, 10.0))),
        (
            MarkupKind::Polygon,
            Geometry::Vertices(vec![(0.0, 0.0), (10.0, 0.0), (5.0, 8.0)]),
        ),
    ];
    for (kind, geometry) in cases {
        match spec_default_pen(kind, &geometry) {
            Some(
                MarkupSpec::Square {
                    interior, border, ..
                }
                | MarkupSpec::Circle {
                    interior, border, ..
                }
                | MarkupSpec::Polygon {
                    interior, border, ..
                },
            ) => {
                assert_eq!(interior, None, "{kind:?} must not fill");
                assert!(border.is_some(), "{kind:?} must have a visible border");
            }
            other => panic!("{kind:?} authored {other:?}"),
        }
    }
}

/// A kind holding another family's geometry authors nothing rather than
/// guessing — [`Refusal::Mismatched`], from both ends.
#[test]
fn a_mismatched_kind_and_geometry_authors_nothing() {
    assert_eq!(
        spec_default_pen(MarkupKind::Ink, &band((0.0, 0.0), (1.0, 1.0))),
        None,
        "an /Ink cannot be built from two points"
    );
    assert_eq!(
        spec_default_pen(
            MarkupKind::Rectangle,
            &Geometry::Vertices(vec![(0.0, 0.0), (1.0, 1.0)])
        ),
        None
    );
    assert_eq!(
        action(
            MarkupKind::Polygon,
            0,
            Geometry::Strokes(vec![vec![(0.0, 0.0), (1.0, 1.0)]]),
            pen::Pen::default(),
        ),
        Err(Refusal::Mismatched)
    );
}

// -----------------------------------------------------------------
// Degenerate input
// -----------------------------------------------------------------

/// ★ **A drag that ends where it began commits nothing** — rather than a
/// 1-point mark nobody can see, holding a slot on the undo stack.
#[test]
fn a_drag_with_no_extent_commits_nothing() {
    for kind in [
        MarkupKind::Rectangle,
        MarkupKind::Ellipse,
        MarkupKind::Arrow,
        MarkupKind::Highlight,
    ] {
        assert_eq!(
            action(
                kind,
                0,
                band((100.0, 200.0), (100.0, 200.0)),
                pen::Pen::default()
            ),
            Err(Refusal::NoExtent),
            "{kind:?}"
        );
    }
}

/// …and the smallest real extent on **either** axis does commit. There is
/// no page-space threshold; egui's screen-space drag threshold is the only
/// one, which is what keeps a deliberate small mark at 16 % zoom from being
/// silently replaced by something else.
#[test]
fn the_smallest_real_extent_on_either_axis_still_commits() {
    for end in [(100.01, 200.0), (100.0, 200.01)] {
        let raised = action(
            MarkupKind::Rectangle,
            3,
            band((100.0, 200.0), end),
            pen::Pen::default(),
        )
        .expect("committed");
        assert_eq!(
            raised,
            Action::CommitMarkup {
                pen: pen::Pen::default(),
                page: 3,
                kind: MarkupKind::Rectangle,
                geometry: band((100.0, 200.0), end),
            }
        );
    }
}

/// ★ **A polygon needs three vertices and a polyline needs two** — the one
/// place this shell is deliberately stricter than `pdfcer-core`.
///
/// The engine's `validate_geometry` refuses `< 2` for both, so a two-vertex
/// `/Polygon` is legal PDF it would happily author: a closed shape from A to
/// B and back, which renders as a line. That is never what a gesture meant,
/// so the shell refuses it — and refuses it by **name**, so the trace
/// distinguishes "you double-clicked one click early" from "the run had no
/// extent".
#[test]
fn a_polygon_needs_three_vertices_where_a_polyline_needs_two() {
    let two = Geometry::Vertices(vec![(0.0, 0.0), (10.0, 5.0)]);
    let three = Geometry::Vertices(vec![(0.0, 0.0), (10.0, 5.0), (4.0, 9.0)]);
    assert!(action(MarkupKind::PolyLine, 0, two.clone(), pen::Pen::default()).is_ok());
    assert_eq!(
        action(MarkupKind::Polygon, 0, two, pen::Pen::default()),
        Err(Refusal::TooFewVertices),
        "a two-vertex polygon is a line drawn there and back"
    );
    assert!(action(MarkupKind::Polygon, 0, three, pen::Pen::default()).is_ok());
    assert_eq!(
        action(
            MarkupKind::PolyLine,
            0,
            Geometry::Vertices(vec![(1.0, 1.0)]),
            pen::Pen::default(),
        ),
        Err(Refusal::TooFewVertices)
    );
}

/// A run every point of which is the same point authors nothing — the
/// vertex and ink form of the zero-extent drag.
#[test]
fn a_run_with_no_extent_commits_nothing() {
    let same = vec![(50.0, 50.0), (50.0, 50.0), (50.0, 50.0)];
    for kind in [MarkupKind::PolyLine, MarkupKind::Polygon] {
        assert_eq!(
            action(
                kind,
                0,
                Geometry::Vertices(same.clone()),
                pen::Pen::default()
            ),
            Err(Refusal::NoExtent),
            "{kind:?}"
        );
    }
    assert_eq!(
        action(
            MarkupKind::Ink,
            0,
            Geometry::Strokes(vec![same.clone()]),
            pen::Pen::default()
        ),
        Err(Refusal::NoExtent)
    );
    // …and a single-point stroke, which the ENGINE would accept: its guard
    // is "every stroke is empty", and one point is not empty. It strokes
    // zero length and draws nothing.
    assert_eq!(
        action(
            MarkupKind::Ink,
            0,
            Geometry::Strokes(vec![vec![(1.0, 2.0)]]),
            pen::Pen::default(),
        ),
        Err(Refusal::NoExtent)
    );
    assert_eq!(
        action(
            MarkupKind::Ink,
            0,
            Geometry::Strokes(Vec::new()),
            pen::Pen::default()
        ),
        Err(Refusal::NoExtent)
    );
}

/// A non-finite coordinate is refused rather than authored into an
/// annotation's `/Rect`, in every family — because the check is written once
/// and a family it did not reach would be the one that shipped the NaN.
#[test]
fn a_non_finite_coordinate_is_refused_in_every_family() {
    assert_eq!(
        action(
            MarkupKind::Arrow,
            0,
            band((0.0, 0.0), (f64::NAN, 1.0)),
            pen::Pen::default()
        ),
        Err(Refusal::NotFinite)
    );
    assert_eq!(
        action(
            MarkupKind::PolyLine,
            0,
            Geometry::Vertices(vec![(0.0, 0.0), (1.0, f64::INFINITY)]),
            pen::Pen::default(),
        ),
        Err(Refusal::NotFinite)
    );
    assert_eq!(
        action(
            MarkupKind::Ink,
            0,
            Geometry::Strokes(vec![vec![(0.0, 0.0), (f64::NEG_INFINITY, 1.0)]]),
            pen::Pen::default(),
        ),
        Err(Refusal::NotFinite)
    );
}

/// The preview colour is the colour that will be committed, component for
/// component — one source, so the band cannot show one pen and the file
/// carry another. Asserted over every kind, because the pen is now read by
/// three gesture modules and a kind added to the wrong `rgb` arm would draw
/// a yellow polyline.
#[test]
fn the_preview_colour_is_the_committed_colour() {
    for &kind in MarkupKind::ALL {
        let (r, g, b) = pen::Pen::default().colour_for(kind);
        let c = pen_color(kind, pen::Pen::default());
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let expect = |v: f64| (v * 255.0).round() as u8;
        assert_eq!(
            (c.r(), c.g(), c.b()),
            (expect(r), expect(g), expect(b)),
            "{kind:?}"
        );
    }
    // …and only the highlighter is yellow. A yellow line on white paper
    // under black glyphs marks nothing visible, which is the one failure a
    // comment cannot afford — the same assertion `text::TextMarkKind` makes.
    for &kind in MarkupKind::ALL {
        let (r, g, b) = pen::Pen::default().colour_for(kind);
        let yellow = r > 0.5 && g > 0.5 && b < 0.5;
        assert_eq!(
            yellow,
            kind == MarkupKind::Highlight,
            "{kind:?} draws in ({r}, {g}, {b})"
        );
    }
}

/// The preview's stroke is the pen's real width at this magnification, and
/// never thinner than a visible hairline.
#[test]
fn the_preview_stroke_is_the_pen_width_at_this_zoom() {
    let extent = (612.0_f32, 792.0_f32);
    let widths: Vec<f32> = [0.1_f32, 1.0, 8.0]
        .iter()
        .map(|&zoom| {
            let rect =
                egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(extent.0 * zoom, extent.1 * zoom));
            pen_px(&PageMapping::new(rect, extent, zoom), pen::Pen::default())
        })
        .collect();
    #[allow(clippy::cast_possible_truncation)]
    let pen = PEN_WIDTH_PTS as f32;
    assert!((widths[1] - pen).abs() < 1e-3, "{widths:?}");
    assert!((widths[2] - pen * 8.0).abs() < 1e-2, "{widths:?}");
    assert!(widths[0] >= 1.0, "a hairline must stay visible: {widths:?}");
}
