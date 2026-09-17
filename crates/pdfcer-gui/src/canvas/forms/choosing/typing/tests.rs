//! Unit tests for the editable combo box — the pure halves only.
//!
//! The drop button's arithmetic, what a click on one side of it means, and the
//! guard that stops a field committing a value nobody typed. The text box
//! itself, the popup and the focus dance are facts about a laid out `egui`
//! frame and a real pointer, and R1 puts those in `tools/ui-verify/`.

use egui::{Rect, pos2};

use super::{arrival, arrow_strip, display_matches, hit_arrow};

/// A widget wider than it is tall, which is what a combo box on a real form
/// is: the button is then a square taken from the height.
fn wide() -> Rect {
    Rect::from_min_size(pos2(100.0, 200.0), egui::vec2(180.0, 20.0))
}

#[test]
fn the_button_is_square_on_a_field_wide_enough_to_hold_one() {
    assert!((arrow_strip(wide()) - 20.0).abs() < f32::EPSILON);
}

#[test]
fn the_button_never_takes_more_than_half_a_narrow_field() {
    // 14 pt wide and 40 tall — a square button would leave no text area at
    // all, and the operator could not place a caret in the field.
    let tall = Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(14.0, 40.0));
    assert!((arrow_strip(tall) - 7.0).abs() < f32::EPSILON);
}

#[test]
fn the_button_is_never_zero_wide() {
    // A degenerate rectangle would otherwise produce an inverted arrow rect,
    // which `egui` draws as nothing and hit-tests as everything.
    let flat = Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(0.0, 0.0));
    assert!(arrow_strip(flat) >= 1.0);
}

#[test]
fn a_click_on_the_right_hand_strip_is_a_click_on_the_button() {
    let r = wide();
    assert!(hit_arrow(r, pos2(r.max.x - 1.0, r.center().y)));
}

#[test]
fn a_click_in_the_text_area_is_not_a_click_on_the_button() {
    let r = wide();
    assert!(!hit_arrow(r, pos2(r.min.x + 1.0, r.center().y)));
    // The boundary itself belongs to the button, so the two halves partition
    // the field with no point that is neither.
    assert!(!hit_arrow(
        r,
        pos2(r.max.x - arrow_strip(r) - 0.1, r.center().y)
    ));
    assert!(hit_arrow(r, pos2(r.max.x - arrow_strip(r), r.center().y)));
}

#[test]
fn arriving_on_the_button_opens_the_list_and_arriving_in_the_text_does_not() {
    let r = wide();
    assert!(arrival(r, pos2(r.max.x - 2.0, r.center().y), 1).open);
    assert!(!arrival(r, pos2(r.min.x + 2.0, r.center().y), 1).open);
}

#[test]
fn an_arrival_carries_the_highlight_it_was_given_either_way() {
    let r = wide();
    assert_eq!(arrival(r, pos2(r.max.x - 2.0, r.center().y), 2).hl, 2);
    assert_eq!(arrival(r, pos2(r.min.x + 2.0, r.center().y), 2).hl, 2);
}

fn options() -> Vec<(String, String)> {
    vec![
        ("S".to_owned(), "Small".to_owned()),
        ("M".to_owned(), "Medium".to_owned()),
    ]
}

/// ★★★ The guard that stops a field the operator only **looked at** from
/// being written.
///
/// `/V` legally holds either half of an option, and the box shows the display
/// half. A field whose `/V` is the export `S` therefore draws "Small", and a
/// naive "the draft differs from the stored value" test would commit `Small`
/// on the way out of a field nobody typed in — a document edit, and an undo
/// entry, for a glance.
#[test]
fn the_display_of_a_stored_export_is_not_a_change() {
    assert!(display_matches(&options(), &["S".to_owned()], "Small"));
    assert!(display_matches(&options(), &["S".to_owned()], "S"));
}

#[test]
fn a_genuinely_typed_value_is_a_change() {
    // ui-text-exempt: a value the operator types into a form, not a string
    // this program displays.
    let typed = "Extra large";
    assert!(!display_matches(&options(), &["S".to_owned()], typed));
    // Another option's display is a change too — picking by typing is exactly
    // what bit 19 is for, and `set_choice_value` resolves it against `/Opt`.
    assert!(!display_matches(&options(), &["S".to_owned()], "Medium"));
}

#[test]
fn nothing_stored_means_anything_typed_is_a_change() {
    assert!(!display_matches(&options(), &[], "Small"));
}
