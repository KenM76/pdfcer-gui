//! Unit tests for `canvas::forms::choosing` — the pure halves only.
//!
//! What is here is what a function of its arguments can be judged on: which
//! exports a pick produces, which row a list opens on, and where the highlight
//! goes. What is **not** here is the popup itself — its side, its constraint
//! rectangle and whether a press on it reaches the page are facts about a laid
//! out `egui` frame and a real pointer, and R1 says those are asserted by
//! driving the binary. `tools/ui-verify/` is where that lives.

use super::{first_selected, step, wanted};

/// Three options with distinct export and display halves, which is the shape
/// that makes a display/export confusion visible at all: a fixture whose two
/// halves are equal passes under either reading.
fn options() -> Vec<(String, String)> {
    vec![
        ("CA".to_owned(), "Canada".to_owned()),
        ("MX".to_owned(), "Mexico".to_owned()),
        ("AR".to_owned(), "Argentina".to_owned()),
    ]
}

#[test]
fn a_single_select_pick_is_the_one_export_and_nothing_else() {
    let opts = options();
    let selected = vec!["CA".to_owned()];
    assert_eq!(wanted(&opts, &selected, "MX", false), vec!["MX".to_owned()]);
}

#[test]
fn a_single_select_pick_sends_the_export_rather_than_the_display() {
    let opts = options();
    // The row the operator reads says "Mexico"; `/V` must hold `MX`.
    let values = wanted(&opts, &[], "MX", false);
    assert_eq!(values, vec!["MX".to_owned()]);
    assert!(!values.iter().any(|v| v == "Mexico"));
}

#[test]
fn a_multi_select_pick_adds_to_what_is_already_ticked() {
    let opts = options();
    let selected = vec!["CA".to_owned()];
    assert_eq!(
        wanted(&opts, &selected, "AR", true),
        vec!["CA".to_owned(), "AR".to_owned()]
    );
}

#[test]
fn a_multi_select_pick_on_a_ticked_row_removes_it() {
    let opts = options();
    let selected = vec!["CA".to_owned(), "AR".to_owned()];
    assert_eq!(wanted(&opts, &selected, "CA", true), vec!["AR".to_owned()]);
}

#[test]
fn unticking_the_last_multi_select_row_clears_the_field() {
    let opts = options();
    let selected = vec!["MX".to_owned()];
    // Empty is a real command here — it is how a multi-select field is
    // answered "none of these" — and is what `set_choice_value` takes to
    // remove `/V` and `/I`.
    assert!(wanted(&opts, &selected, "MX", true).is_empty());
}

#[test]
fn a_selection_stored_as_a_display_value_is_still_recognised() {
    let opts = options();
    // `/V` legally holds either half, and files in the wild hold the display
    // one. A strict export match would read this field as unanswered and the
    // tick below as an addition rather than a removal.
    let selected = vec!["Mexico".to_owned()];
    assert!(wanted(&opts, &selected, "MX", true).is_empty());
    assert_eq!(first_selected(&opts, &selected), Some(1));
}

#[test]
fn a_stored_value_matching_no_option_is_dropped_rather_than_carried() {
    let opts = options();
    // Another program wrote `BR`; the option list does not have it. Carrying
    // it into the command would hand `set_choice_value` a value it must refuse
    // by name, so the operator's tick on Canada would fail citing Brazil.
    let selected = vec!["BR".to_owned()];
    assert_eq!(wanted(&opts, &selected, "CA", true), vec!["CA".to_owned()]);
    assert_eq!(first_selected(&opts, &selected), None);
}

#[test]
fn a_list_opens_on_the_answered_row_and_on_the_first_when_unanswered() {
    let opts = options();
    assert_eq!(first_selected(&opts, &["AR".to_owned()]), Some(2));
    assert_eq!(first_selected(&opts, &[]), None);
}

#[test]
fn the_highlight_wraps_at_both_ends() {
    assert_eq!(step(0, 3, false), 1);
    assert_eq!(step(2, 3, false), 0);
    assert_eq!(step(0, 3, true), 2);
    assert_eq!(step(2, 3, true), 1);
}

#[test]
fn the_highlight_stays_put_on_an_empty_option_list() {
    // Reachable: a `/Ch` field with an empty `/Opt` is not offered on the
    // canvas at all, but the arithmetic must not divide by zero if it ever is.
    assert_eq!(step(0, 0, false), 0);
    assert_eq!(step(0, 0, true), 0);
}
