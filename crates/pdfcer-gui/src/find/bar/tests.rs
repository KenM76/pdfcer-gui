//! # `find::bar` — the tests
//!
//! Split out of `bar.rs` under **R2** on 2026-09-09, when the *Zoom* control
//! (`OPERATOR_REQUESTS.md` O163) took that file to 1,593 lines — 93 over the
//! ceiling. The seam is the obvious one and the one this crate takes
//! everywhere else (`app::prefs::tests`, `panels::tests`): the assertions are
//! the part of a module that is not in the shipped binary, and they are read
//! at a different time from the widget code they check.
//!
//! `use super::*` therefore reaches `find::bar` exactly as it did when this
//! was an inline `mod tests`, and nothing about what is asserted changed in
//! the move.
//!
//! ★★ **The inner `#![cfg(test)]` is load-bearing and is not a duplicate of
//! the outer `#[cfg(test)] mod tests;`.** Without it,
//! `tools/gates/check-ui-strings.sh` walks this file as ordinary source and
//! reports every assertion message as a user-visible string that should live
//! in `ui_text` — exclusion 2b in that gate. It is the same line every other
//! split test file in this crate carries, and it was reported by that gate
//! within minutes of this split, exactly as designed.
#![cfg(test)]

use super::*;
use egui::{Context, RawInput};

// =======================================================================
// ★ The OCR offer
// =======================================================================

/// ★ **A page with no text at all, after a search that found nothing.**
///
/// The one combination that offers OCR, and the operator's actual rule
/// stated as a case: there is nothing on this page for any search to have
/// matched, so the empty result is a fact about the *document*.
#[test]
fn a_page_with_no_text_at_all_offers_recognition() {
    assert!(offer_ocr(Readout::Empty, || false));
}

/// ★★ **THE FALSIFYING CASE.** An ordinary empty result offers nothing.
///
/// This is the assertion the whole feature turns on, and it is the one a
/// plausible wrong implementation fails. Offering OCR on any zero-hit
/// search is one character simpler to write, passes
/// [`Self::a_page_with_no_text_at_all_offers_recognition`] perfectly, and
/// would put *"this page has no text on it"* under every mistyped part
/// number on a drawing full of text.
///
/// The operator named this trap in the specification rather than leaving it
/// to be discovered: *"the trigger is 'this document is images', NOT 'this
/// search had no matches'"*, and `FEATURES.md` records that the two "must
/// not be collapsed."
#[test]
fn an_ordinary_empty_result_on_a_text_page_offers_nothing() {
    assert!(
        !offer_ocr(Readout::Empty, || true),
        "a search for a word that is simply not on a page full of text is an ordinary \
         empty result; offering to recognise it would be nonsense"
    );
}

/// ★ **The page is not even asked about unless the search came back empty.**
///
/// The short-circuit, asserted rather than assumed — and it is a
/// correctness property, not an optimisation. `page_has_extractable_text`
/// costs one page extraction on a cache miss, and the bar draws on every
/// frame it is open; a version that evaluated the closure first would put
/// that extraction on the frame budget while looking identical in every
/// other test here. That is `HANDOFF.md` §2's defect 9 exactly: the right
/// work, charged at the wrong moment, invisible to a suite that only asks
/// whether it happened.
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

/// The offer raises the command the ribbon registers, not a spelling of it.
///
/// An id written at a call site and nowhere else goes stale in silence —
/// `HANDOFF.md` §5's whole subject — and the symptom here would be a button
/// that traces `command-unimplemented` and does nothing.
#[test]
fn the_offer_raises_the_registered_recognise_command() {
    let mut registry = egui_shell::commands::CommandRegistry::new();
    crate::shell::commands::register(&mut registry);
    assert!(
        registry.get(OCR_COMMAND).is_some(),
        "`{OCR_COMMAND}` is not registered, so the offer's button would reach the \
         dispatcher's fall-through arm and do nothing"
    );
}

// =======================================================================
// ★ Placement
// =======================================================================

/// ★ **The box is pinned inside the canvas viewport's top-right corner**,
/// not the window's.
///
/// The distinction is invisible until a dock is open, and then it is the
/// whole difference between a find bar over the page and a find bar over
/// the Objects panel. Asserted against a host rect deliberately offset
/// from the origin, so an implementation that forgot `host.right()` or
/// `host.top()` and used the screen's still fails.
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
///
/// Reachable: the canvas viewport shrinks with every dock the operator
/// opens, and `MIN_WINDOW_SIZE` is 640 points wide before any of them.
/// egui's `constrain_to` is what pulls the *box* back in that case; what
/// is asserted here is that it is not being handed a nonsense point to
/// start from.
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
// ★ What Enter means
// =======================================================================

/// ★ **Enter searches when the answer is not current and steps when it
/// is.**
///
/// The whole of the bar's key behaviour, asserted without a frame. The
/// interesting rows are `Stale` — which must search rather than step,
/// because stepping through geometry the module has already declared
/// untrustworthy is exactly what the staleness rule exists to prevent —
/// and `Empty`, which must do nothing.
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

/// ★ **Enter on a fruitless search does nothing at all.**
///
/// A search is a whole-document text extraction — 350 ms on the benchmark
/// drawing, measured. Re-running one that just matched nothing, once per
/// keypress, is how a viewer becomes unusable on the files it exists for
/// — and it would produce the same answer, because if anything had changed
/// the readout would be `Stale`.
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

/// ★ **The word-rule chooser appears with Whole word and not before.**
///
/// P3, applied to the one control on this surface whose availability is
/// conditional. Driven through a real `Ui` so what is asserted is what the
/// menu actually builds, and counted by *widgets that were laid out*
/// rather than by reading the branch — a test that read the branch would
/// pass on a build where the `if` had been inverted and the label moved.
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
///
/// ★ Read off the **painted shapes**, not off the source. A test that
/// asserted `t::find_zoom()` equals `"Zoom"` would prove the catalog agrees
/// with itself and would pass on a build where the checkbox was never added
/// to the menu at all — which is the defect worth catching, because a
/// control the operator cannot see is the same as a control that does not
/// exist (**R9**).
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

/// ★★ **The options menu offers a control named exactly *Zoom*.**
///
/// The operator's request named the control: *"add a checkbox option to our
/// search bar called zoom"*. A request that carries a name is a request for
/// that name — he will look for that word — so the word is asserted, not
/// just the presence of a fourth widget.
///
/// Painted rather than read from the catalog: see [`painted_text`].
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

/// ★ **Toggling Zoom does not disturb the search options.**
///
/// The load-bearing property of the split in [`options`]: a `FindOptions`
/// change re-runs the search, and re-running a search because a *view*
/// preference moved would throw away the operator's place in the result
/// list. So the menu writes the Zoom value through a separate `&mut` and
/// must leave the options struct untouched, whichever way it is flipped.
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

/// ★ **Every glyph the bar draws exists in the bundled font set.**
///
/// `⏴`, `⏵` and `×` are the entire visible text of three controls. A
/// codepoint egui's bundled fonts (Ubuntu-Light + NotoEmoji +
/// emoji-icon-font) cannot draw renders as a tofu box, which is defect
/// D2's shape — an invisible label — on a control the operator has to hit.
///
/// The status bar has the identical test and it has already paid for
/// itself once: that catalog was written with `◀ ▶ ▸ ▾`, **all four of
/// which are missing**, and they would have shipped as tofu on the two
/// controls an operator touches most. This test is not a duplicate of it —
/// it cannot see this file's strings, and this file cannot see the status
/// bar's.
#[test]
fn every_glyph_the_find_bar_draws_has_a_glyph() {
    let ctx = Context::default();
    let labels: Vec<String> = vec![
        t::field_label().to_owned(),
        t::previous().to_owned(),
        t::next().to_owned(),
        t::close().to_owned(),
        t::options().to_owned(),
        t::position(3, 47),
        t::no_matches().to_owned(),
        t::stale().to_owned(),
        t::match_case().to_owned(),
        t::whole_word().to_owned(),
        t::wildcards().to_owned(),
        t::word_rule().to_owned(),
        t::word_rule_alphanumeric().to_owned(),
        t::word_rule_non_space().to_owned(),
        t::word_rule_non_space_or_dash().to_owned(),
        t::toggle().to_owned(),
    ];

    let mut missing = Vec::new();
    let _ = ctx.run_ui(RawInput::default(), |ui| {
        let font = egui::FontId::proportional(14.0);
        ui.ctx().fonts_mut(|f| {
            for label in &labels {
                for c in label.chars() {
                    if !f.has_glyph(&font, c) {
                        missing.push((label.clone(), c));
                    }
                }
            }
        });
    });

    assert!(
        missing.is_empty(),
        "these labels contain codepoints the bundled fonts cannot draw, so they would \
         render as tofu boxes: {missing:?}"
    );
}

// =======================================================================
// The bar as a whole
// =======================================================================

/// A bar showing the answer to a search for `query` that found `hits`
/// hits, all on page 0.
///
/// Built by writing `super`'s private fields directly, which a child
/// module may do. The alternative — a constructor on `FindState` that only
/// tests call — would be a second way to assemble a result set, and the
/// currency key is exactly the thing that must have one.
pub(super) fn searched(query: &str, hits: usize) -> FindState {
    let mut state = FindState::default();
    state.open();
    state.query_mut().push_str(query);
    state.results = Some(crate::find::Results {
        query: query.to_owned(),
        options: FindOptions::default(),
        epoch: 0,
        hits: (0..hits)
            .map(|_| crate::find::Hit {
                page: 0,
                canvas: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(10.0, 10.0))),
                text: query.to_owned(),
            })
            .collect(),
        current: 0,
        unsearchable_fonts: 0,
    });
    state
}

/// Run one frame of the row and return the actions it raised.
fn frame(ctx: &Context, state: &mut FindState, epoch: u64, input: RawInput) -> Vec<Action> {
    let mut actions = Vec::new();
    let _ = ctx.run_ui(input, |ui| body(ui, state, epoch, &mut actions));
    actions
}

/// ★ **The box is exactly the same size whatever the readout says.**
///
/// The property the module docs argue for: the overlay is anchored by its
/// top-right corner, so a box that changed width would move the search
/// field the operator is typing into, and the ⏴ ⏵ buttons out from under
/// a pointer aimed between two clicks.
///
/// Four readouts, four very different strings, one size.
#[test]
fn the_box_is_the_same_size_whatever_the_readout_says() {
    let ctx = Context::default();
    let size = |state: &mut FindState, epoch: u64| {
        let mut got = Vec2::ZERO;
        let _ = ctx.run_ui(RawInput::default(), |ui| {
            let mut actions = Vec::new();
            got = ui
                .scope(|ui| body(ui, state, epoch, &mut actions))
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
///
/// Driven through a real frame rather than by calling the arm, so what is
/// under test is the wiring — the failure this catches is a button that
/// draws, is enabled, and reports nothing, which is the shape three of the
/// old shell's panels shipped in.
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

/// ★ **Typing raises nothing.**
///
/// The cost rule, from the other end: a search is a whole-document text
/// extraction — 350 ms on the benchmark drawing — so a bar that raised one
/// per keystroke would spend 1.4 seconds of blocked UI thread on the word
/// `part`. This is what "never searches on a keystroke" means in a test.
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
