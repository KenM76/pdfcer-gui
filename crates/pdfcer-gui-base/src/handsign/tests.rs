use super::*;

fn line(points: &[(f32, f32)]) -> Vec<Pos2> {
    points.iter().map(|&(x, y)| pos2(x, y)).collect()
}

fn wide_mark() -> Mark {
    // 200 wide, 50 tall: a typical signature's proportions.
    Mark {
        strokes: vec![
            line(&[(10.0, 10.0), (60.0, 60.0), (110.0, 20.0)]),
            line(&[(120.0, 30.0), (210.0, 40.0)]),
        ],
    }
}

#[test]
fn a_short_mark_is_centred_and_stays_inside_the_box() {
    let target = Rect::from_min_max(pos2(100.0, 500.0), pos2(400.0, 540.0));
    let fitted = fit(&wide_mark(), target).expect("fits");
    let all: Vec<Pos2> = fitted.strokes.iter().flatten().copied().collect();
    let max_x = all.iter().map(|p| p.x).fold(f32::MIN, f32::max);
    let min_x = all.iter().map(|p| p.x).fold(f32::MAX, f32::min);
    let min_y = all.iter().map(|p| p.y).fold(f32::MAX, f32::min);
    let max_y = all.iter().map(|p| p.y).fold(f32::MIN, f32::max);
    assert!((min_x - (100.0 + 0.03 * 300.0)).abs() < 1e-3, "left inset");
    assert!(max_x <= 400.0, "inside on the right: {max_x}");
    // Width binds at 0.95 * 300 / 200 = 1.425; height 50 * 1.425 = 71.25 > 36,
    // so the mark stands on the lower edge and rises above the box.
    assert!(
        (max_y - (540.0 - 0.05 * 40.0)).abs() < 1e-3,
        "stands on the edge"
    );
    assert!(min_y >= 540.0 - 2.0 * 40.0, "rises at most two box heights");
}

#[test]
fn a_tall_box_centres_the_mark_vertically() {
    let target = Rect::from_min_max(pos2(0.0, 0.0), pos2(300.0, 200.0));
    let fitted = fit(&wide_mark(), target).expect("fits");
    let ys: Vec<f32> = fitted.strokes.iter().flatten().map(|p| p.y).collect();
    let (lo, hi) = (
        ys.iter().copied().fold(f32::MAX, f32::min),
        ys.iter().copied().fold(f32::MIN, f32::max),
    );
    assert!(
        ((lo + hi) / 2.0 - 100.0).abs() < 1e-3,
        "centred: {lo}..{hi}"
    );
}

#[test]
fn the_rise_limit_binds_for_a_tall_mark_in_a_short_box() {
    let tall = Mark {
        strokes: vec![line(&[(0.0, 0.0), (10.0, 100.0)])],
    };
    let target = Rect::from_min_max(pos2(0.0, 100.0), pos2(300.0, 120.0));
    let fitted = fit(&tall, target).expect("fits");
    let ys: Vec<f32> = fitted.strokes.iter().flatten().map(|p| p.y).collect();
    let height =
        ys.iter().copied().fold(f32::MIN, f32::max) - ys.iter().copied().fold(f32::MAX, f32::min);
    assert!((height - 0.95 * 2.0 * 20.0).abs() < 1e-3, "height {height}");
}

#[test]
fn a_tap_alone_is_not_a_signature() {
    let tap = Mark {
        strokes: vec![line(&[(5.0, 5.0)])],
    };
    assert!(!tap.has_extent());
    let box_ = Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 20.0));
    assert!(fit(&tap, box_).is_none());
    assert!(fit(&Mark::default(), box_).is_none());
}

#[test]
fn a_dot_inside_a_signature_survives_as_two_points() {
    let mut mark = wide_mark();
    mark.strokes.push(line(&[(150.0, 5.0)]));
    let box_ = Rect::from_min_max(pos2(0.0, 0.0), pos2(300.0, 40.0));
    let fitted = fit(&mark, box_).expect("fits");
    assert_eq!(fitted.strokes.last().map(Vec::len), Some(2));
}

#[test]
fn pen_width_follows_the_mark_and_is_clamped() {
    let small = Rect::from_min_max(pos2(0.0, 0.0), pos2(30.0, 5.0));
    let large = Rect::from_min_max(pos2(0.0, 0.0), pos2(3000.0, 2000.0));
    assert_eq!(fit(&wide_mark(), small).map(|f| f.width), Some(PEN_MIN));
    assert_eq!(fit(&wide_mark(), large).map(|f| f.width), Some(PEN_MAX));
}

#[test]
fn simplification_drops_collinear_points_and_keeps_corners() {
    let mark = Mark {
        strokes: vec![line(&[
            (0.0, 0.0),
            (1.0, 0.0),
            (2.0, 0.0),
            (3.0, 0.0),
            (3.0, 5.0),
        ])],
    };
    let simple = mark.simplified(SIMPLIFY_TOLERANCE);
    assert_eq!(
        simple.strokes[0],
        line(&[(0.0, 0.0), (3.0, 0.0), (3.0, 5.0)])
    );
}

#[test]
fn normalising_keeps_proportions_and_the_text_form_round_trips() {
    let n = wide_mark().normalised();
    let b = n.bounds().expect("bounds");
    assert!((b.width() - 1.0).abs() < 1e-6);
    assert!((b.height() - 0.25).abs() < 1e-6);
    let back = Mark::from_text(&n.to_text()).expect("parses");
    assert_eq!(back.strokes.len(), 2);
    assert!((back.bounds().expect("b").height() - 0.25).abs() < 1e-3);
}

#[test]
fn a_damaged_saved_file_is_ignored_whole() {
    assert!(Mark::from_text("0,0 1,1\n0,0 nonsense\n").is_none());
    assert!(Mark::from_text("0,0 NaN,1\n").is_none());
    assert!(Mark::from_text("\n\n").is_none());
}

#[test]
fn the_signed_set_is_what_the_last_measurement_found() {
    let mut h = HandSigned::default();
    assert!(!h.is_current(0));
    let mark = |field: &str, page| SignedMark {
        field: field.to_owned(),
        page,
        bounds: pdfcer_core::page_tree::Rect {
            llx: 0.0,
            lly: 0.0,
            urx: 1.0,
            ury: 1.0,
        },
    };
    h.measured(0, vec![mark("Sig1", 0), mark("Sig1", 1), mark("Sig2", 1)]);
    assert_eq!(h.marks().len(), 3);
    assert!(h.is_current(0) && !h.is_current(1));
    assert!(h.is_signed("Sig1") && !h.is_signed("Sig3"));
    assert_eq!(h.signed_count(), 2);
    h.measured(1, Vec::new());
    assert!(!h.is_signed("Sig1"));
}
