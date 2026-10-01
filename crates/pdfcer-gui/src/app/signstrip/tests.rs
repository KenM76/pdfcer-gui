use egui::{Rect, pos2};

use super::{Progress, next, progress};
use crate::canvas::forms::boxes::FieldTarget;

fn target(page: usize, field: &str, widget: usize) -> FieldTarget {
    FieldTarget {
        page,
        field: field.to_owned(),
        widget,
        rect: Rect::from_min_max(pos2(0.0, 0.0), pos2(10.0, 10.0)),
    }
}

fn three() -> Vec<FieldTarget> {
    vec![target(0, "a", 0), target(0, "b", 0), target(1, "c", 0)]
}

#[test]
fn a_field_with_two_widgets_counts_once() {
    let list = vec![target(0, "a", 0), target(1, "a", 1), target(1, "b", 0)];
    assert_eq!(
        progress(&list, |f| f == "a"),
        Progress {
            signed: 1,
            total: 2
        }
    );
}

#[test]
fn next_starts_at_the_first_box_and_walks_in_order() {
    let list = three();
    let none = |_: &str| false;
    assert_eq!(next(&list, none, None), Some(0));
    assert_eq!(next(&list, none, Some(0)), Some(1));
    assert_eq!(next(&list, none, Some(1)), Some(2));
}

#[test]
fn next_skips_signed_boxes_and_wraps() {
    let list = three();
    assert_eq!(next(&list, |f| f == "b", Some(0)), Some(2));
    assert_eq!(next(&list, |f| f == "b", Some(2)), Some(0));
    assert_eq!(next(&list, |f| f != "a", Some(0)), Some(0));
}

#[test]
fn next_is_none_when_everything_is_signed() {
    assert_eq!(next(&three(), |_| true, Some(1)), None);
    assert_eq!(next(&[], |_| false, None), None);
}

#[test]
fn a_stale_index_past_the_end_wraps_rather_than_panics() {
    assert_eq!(next(&three(), |_| false, Some(9)), Some(0));
}
