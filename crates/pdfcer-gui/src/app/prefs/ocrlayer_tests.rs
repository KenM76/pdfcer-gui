//! Tests for `crate::app::prefs::ocrlayer`, kept in the gui because they reach gui modules.

use crate::app::prefs::ocrlayer::*;

/// What the writer produces, the parser reads — over the default and over
/// a value with no round number in it.
///
/// The second case is the one that matters. A round-trip tested only on
/// the default would pass on a writer that emitted a constant.
#[test]
fn what_is_written_is_what_comes_back() {
    for rgb in [
        crate::canvas::ocrlayer::DEFAULT_COLOUR,
        [1, 130, 255],
        [0, 0, 0],
    ] {
        assert_eq!(parse(&format(rgb)), Some(rgb), "{rgb:?} did not survive");
    }
}

/// The forms a hand-editor will actually type.
#[test]
fn the_notations_an_operator_writes_are_all_read() {
    assert_eq!(parse("#CC0099"), Some([204, 0, 153]));
    assert_eq!(parse("cc0099"), Some([204, 0, 153]));
    assert_eq!(parse("  #cC0099  "), Some([204, 0, 153]));
    // Doubled rather than shifted — see `parse`.
    assert_eq!(parse("#F80"), Some([255, 136, 0]));
}

/// A value this cannot read is refused, so the caller can report it.
#[test]
fn a_value_it_cannot_read_is_refused_rather_than_guessed() {
    for bad in [
        "",
        "#",
        "#CC009",
        "#CC00999",
        "magenta",
        "204,0,153",
        "#GG0099",
    ] {
        assert_eq!(parse(bad), None, "`{bad}` should not have parsed");
    }
}
