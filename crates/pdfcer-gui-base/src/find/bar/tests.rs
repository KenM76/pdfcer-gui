//! # `find::bar` — the tests
//!
//! The assertions for [`super`], in a file of their own — the seam this crate
//! takes everywhere else (`app::prefs::tests`, `panels::tests`): assertions are
//! the part of a module that is not in the shipped binary, and they are read at
//! a different time from the widget code they check. `use super::*` reaches
//! `find::bar` exactly as an inline `mod tests` would.
//!
//! **The inner `#![cfg(test)]` is load-bearing and is not a duplicate of
//! the outer `#[cfg(test)] mod tests;`.** Without it,
//! `tools/gates/check-ui-strings.sh` walks this file as ordinary source and
//! reports every assertion message as a user-visible string that should live
//! in `ui_text` — exclusion 2b in that gate. Every split test file in this
//! crate carries the same line, for the same reason.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/find/bar/tests.md`.
#![cfg(test)]

use super::*;
use egui::{Context, RawInput};

// =======================================================================
// The OCR offer
// =======================================================================

/// **A page with no text at all, after a search that found nothing.**
#[test]
fn a_page_with_no_text_at_all_offers_recognition() {
    assert!(offer_ocr(Readout::Empty, || false));
}

/// **THE FALSIFYING CASE.** An ordinary empty result offers nothing.
#[test]
fn an_ordinary_empty_result_on_a_text_page_offers_nothing() {
    assert!(
        !offer_ocr(Readout::Empty, || true),
        "a search for a word that is simply not on a page full of text is an ordinary \
         empty result; offering to recognise it would be nonsense"
    );
}

/// **The page is not even asked about unless the search came back empty.**
#[test]
fn the_page_is_not_extracted_unless_the_search_found_nothing() {
    for readout in [
        Readout::Idle,
        Readout::Stale,
        Readout::At {
            current: 1,
            total: 4,
        },
    ] {
        let mut asked = false;
        let offer = offer_ocr(readout, || {
            asked = true;
            false
        });
        assert!(!offer, "{readout:?} must not offer recognition");
        assert!(
            !asked,
            "{readout:?} asked the page for its text; that is a page extraction charged to \
             a frame the operator did not ask anything on"
        );
    }
}

// =======================================================================
// Placement
// =======================================================================

/// **The box is pinned inside the canvas viewport's top-right corner**,
/// not the window's.
#[test]
fn the_overlay_is_pinned_inside_the_hosts_top_right_corner() {
    let host = Rect::from_min_max(Pos2::new(200.0, 90.0), Pos2::new(1000.0, 700.0));
    let at = anchor_right_top(host);

    assert!(host.contains(at), "the pivot must be on the canvas: {at:?}");
    assert!(at.y > host.top(), "it must not sit on the canvas's edge");
    assert!(at.y < host.top() + 40.0, "…nor halfway down the page");
    assert!(at.x < host.right(), "…nor flush against the right edge");
    assert!(
        at.x - BAR_WIDTH_PTS > host.left(),
        "an ordinary canvas must be wide enough for the whole box, or the \
         constraint rather than this function is deciding the layout"
    );
}

/// A host far narrower than the box still yields a pivot on the canvas.
#[test]
fn a_narrow_host_still_yields_a_pivot_on_the_canvas() {
    for narrow in [
        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(200.0, 400.0)),
        // Degenerate: a window dragged to nothing, which egui reports
        // before it clamps.
        Rect::from_min_max(Pos2::new(50.0, 50.0), Pos2::new(54.0, 54.0)),
    ] {
        let at = anchor_right_top(narrow);
        assert!(
            narrow.contains(at),
            "{at:?} is outside {narrow:?}; the pivot must never leave the canvas"
        );
    }
}

// =======================================================================
// What Enter means
// =======================================================================

/// **Enter searches when the answer is not current and steps when it
/// is.**
#[test]
fn enter_searches_when_there_is_no_current_answer_and_steps_when_there_is() {
    assert_eq!(
        enter_intent(Readout::Idle, false),
        Some(FindRequest::Search)
    );
    assert_eq!(enter_intent(Readout::Idle, true), Some(FindRequest::Search));
    assert_eq!(
        enter_intent(Readout::Stale, false),
        Some(FindRequest::Search),
        "a stale bar must search again, not step through hits it has disowned"
    );
    assert_eq!(
        enter_intent(
            Readout::At {
                current: 1,
                total: 9
            },
            false
        ),
        Some(FindRequest::Step(Step::Next))
    );
    assert_eq!(
        enter_intent(
            Readout::At {
                current: 1,
                total: 9
            },
            true
        ),
        Some(FindRequest::Step(Step::Previous)),
        "Shift+Enter goes backwards"
    );
}

/// **Enter on a fruitless search does nothing at all.**
#[test]
fn enter_does_not_re_run_a_search_that_found_nothing() {
    assert_eq!(enter_intent(Readout::Empty, false), None);
    assert_eq!(enter_intent(Readout::Empty, true), None);
}

// =======================================================================
// The readout's four sentences
// =======================================================================

/// The four states produce four different readouts, and only one of them
/// is blank.
#[test]
fn the_four_readouts_are_four_different_sentences() {
    let idle = readout_text(Readout::Idle).0;
    let empty = readout_text(Readout::Empty).0;
    let stale = readout_text(Readout::Stale).0;
    let at = readout_text(Readout::At {
        current: 3,
        total: 47,
    })
    .0;

    assert!(idle.is_empty(), "nothing asked, nothing answered");
    assert!(!empty.is_empty() && !stale.is_empty() && !at.is_empty());
    assert_ne!(empty, stale, "`no matches` and `stale` are different facts");
    assert_ne!(empty, at);
    assert_eq!(at, "3 of 47");
}

/// Every readout that says something also explains itself on hover.
///
/// The blank one does not, and must not: a hover target with no text is
/// worse than none.
#[test]
fn every_readout_that_says_something_explains_itself() {
    for readout in [
        Readout::Empty,
        Readout::Stale,
        Readout::At {
            current: 1,
            total: 1,
        },
    ] {
        let (text, hover) = readout_text(readout);
        assert!(!text.is_empty());
        assert!(
            !hover.is_empty(),
            "{readout:?} says something and explains nothing"
        );
    }
    assert!(readout_text(Readout::Idle).1.is_empty());
}

// =======================================================================
// The options menu
// =======================================================================

/// **The word-rule chooser appears with Whole word and not before.**
#[test]
fn the_word_rule_chooser_appears_only_with_whole_word() {
    let ctx = Context::default();
    let widgets = |whole_word: bool| {
        let mut options = FindOptions {
            whole_word,
            ..FindOptions::default()
        };
        let mut height = 0.0_f32;
        // The Zoom control is not what this test is about, but the menu
        // draws it either way, so it is held at its shipped value.
        let mut zoom_on_jump = true;
        let _ = ctx.run_ui(RawInput::default(), |ui| {
            height = ui
                .scope(|ui| options_menu(ui, &mut options, &mut zoom_on_jump))
                .response
                .rect
                .height();
        });
        height
    };
    let plain = widgets(false);
    let with_rule = widgets(true);
    assert!(
        with_rule > plain + 20.0,
        "switching Whole word on must add the rule chooser to the menu \
         ({plain} pt -> {with_rule} pt)"
    );
}

// =======================================================================
// The Zoom control — O163
// =======================================================================

/// Every string the given closure painted, flattened.
fn painted_text(ctx: &Context, build: impl FnMut(&mut egui::Ui)) -> Vec<String> {
    fn walk(shape: &egui::Shape, out: &mut Vec<String>) {
        match shape {
            egui::Shape::Text(text) => out.push(text.galley.text().to_owned()),
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    walk(shape, out);
                }
            }
            _ => {}
        }
    }
    let output = ctx.run_ui(RawInput::default(), build);
    let mut out = Vec::new();
    for clipped in &output.shapes {
        walk(&clipped.shape, &mut out);
    }
    out
}

/// **The options menu offers a control named exactly *Zoom*.**
#[test]
fn the_options_menu_offers_a_control_named_zoom() {
    let ctx = Context::default();
    let mut options = FindOptions::default();
    let mut zoom_on_jump = true;
    let painted = painted_text(&ctx, |ui| options_menu(ui, &mut options, &mut zoom_on_jump));
    assert!(
        painted.iter().any(|s| s == "Zoom"),
        "the operator asked for a checkbox called Zoom and the menu painted {painted:?}"
    );
}

/// **Toggling Zoom does not disturb the search options.**
#[test]
fn toggling_zoom_leaves_the_search_options_alone() {
    let ctx = Context::default();
    for start in [true, false] {
        let before = FindOptions::default();
        let mut options = before;
        let mut zoom_on_jump = start;
        let _ = painted_text(&ctx, |ui| options_menu(ui, &mut options, &mut zoom_on_jump));
        assert_eq!(
            options, before,
            "drawing the menu with Zoom at {start} changed the search options"
        );
    }
}

// =======================================================================
// Legibility — the labels that are glyphs
// =======================================================================

// =======================================================================
// The bar as a whole
// =======================================================================

/// A bar showing the answer to a search for `query` that found `hits`
/// hits, all on page 0.
pub(super) fn searched(query: &str, hits: usize) -> FindState {
    FindState::searched(query, hits, 0)
}

/// Run one frame of the row and return the actions it raised.
fn frame(ctx: &Context, state: &mut FindState, epoch: u64, input: RawInput) -> Vec<Action> {
    let mut actions = Vec::new();
    let _ = ctx.run_ui(input, |ui| body(ui, state, (epoch, false), &mut actions));
    actions
}

/// **The box is exactly the same size whatever the readout says.**
#[test]
fn the_box_is_the_same_size_whatever_the_readout_says() {
    let ctx = Context::default();
    let size = |state: &mut FindState, epoch: u64| {
        let mut got = Vec2::ZERO;
        let _ = ctx.run_ui(RawInput::default(), |ui| {
            let mut actions = Vec::new();
            got = ui
                .scope(|ui| body(ui, state, (epoch, false), &mut actions))
                .response
                .rect
                .size();
        });
        got
    };

    let mut idle = FindState::default();
    idle.open();
    let baseline = size(&mut idle, 0);
    assert!(
        (baseline.x - BAR_WIDTH_PTS).abs() < 1.0,
        "the row must occupy exactly its reserved width, got {baseline:?}"
    );

    for (label, mut state) in [
        ("empty", searched("zzz", 0)),
        ("hits", searched("total", 47)),
    ] {
        let got = size(&mut state, 0);
        assert!(
            (got.x - baseline.x).abs() < 0.01 && (got.y - baseline.y).abs() < 0.01,
            "the `{label}` readout resized the box ({baseline:?} -> {got:?}); a \
             right-anchored box that resizes moves every control on it"
        );
    }

    // …and a stale one, which is the longest string of the three.
    let mut stale = searched("total", 47);
    let got = size(&mut stale, 1);
    assert!(
        (got.x - baseline.x).abs() < 0.01 && (got.y - baseline.y).abs() < 0.01,
        "the stale readout resized the box ({baseline:?} -> {got:?})"
    );
}

/// The step buttons raise nothing until there is something to step.
#[test]
fn the_step_buttons_are_inert_until_there_is_something_to_step() {
    let ctx = Context::default();
    let mut state = FindState::default();
    state.open();

    assert_eq!(state.readout(0), Readout::Idle);
    let actions = frame(&ctx, &mut state, 0, RawInput::default());
    assert!(
        actions.is_empty(),
        "an untouched bar must raise nothing at all"
    );
}

/// **Typing raises nothing.**
#[test]
fn typing_raises_no_search() {
    let ctx = Context::default();
    let mut state = FindState::default();
    state.open();
    state.query_mut().push_str("tot");

    let input = RawInput {
        events: vec![egui::Event::Text("a".to_owned())],
        ..Default::default()
    };
    let actions = frame(&ctx, &mut state, 0, input);
    assert!(
        actions.is_empty(),
        "a keystroke must not reach the engine; a search is a whole-document \
         text extraction"
    );
}

/// Closing the bar is a widget state change and raises no action.
///
/// Closing touches no document, so it does not go through the funnel — the
/// same rule that keeps `show_panel` out of the action list.
#[test]
fn closing_the_bar_is_not_a_document_action() {
    let mut state = FindState::default();
    state.open();
    state.close();
    assert!(!state.is_open());
}

/// Every rule the chooser offers is a real variant, and the list is the
/// whole of what a `#[non_exhaustive]` enum lets this crate name.
#[test]
fn every_word_rule_the_chooser_offers_has_a_label() {
    assert_eq!(crate::find::FindOptions::WORD_RULES.len(), 3);
    for rule in crate::find::FindOptions::WORD_RULES {
        assert!(!crate::find::bar::word_rule_label(*rule).is_empty());
    }
}
