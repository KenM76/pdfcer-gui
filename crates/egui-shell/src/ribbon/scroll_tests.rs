//! # `ribbon::scroll_tests` — the band's horizontal scroll, driven
//!
//! Separate from [`super::width_tests`] to keep that file under the
//! 1,500-line ceiling (R2), and the seam is real: every test in `width_tests`
//! renders a **fresh, unscrolled** band at a series of widths, and every test
//! here has to **drive a click first**, because the left arrow does not exist
//! until something has scrolled.
//!
//! ★★★ That difference is not incidental — it is why a width sweep can test
//! the right arrow exhaustively and say nothing at all about the left one.
//! `no_visible_group_overlaps_the_overflow_affordance` walks every width and
//! cannot catch a left-arrow overlap, because there is no width at which an
//! unscrolled band draws a left arrow. **A guard can be correct, thorough, and
//! blind by construction.**

use egui::{Pos2, Rect, Vec2};

use super::tests::{registry, shell};
use super::width_tests::{SLACK, context};
use super::{Ribbon, RibbonState, report};

/// ★★★ **No visible group runs under the LEFT scroll arrow either** — the twin
/// of the right-hand affordance's overlap test, and the only thing that holds
/// the left arrow's reservation in place.
///
/// The right-hand affordance's reservation is taken from the band's right edge
/// before any group is laid out. The left arrow needs the mirror of that: its
/// rect is `full.min .. full.left() + reserve`, it is drawn **after** the
/// groups, and unless `groups_rect` starts past it the two overlap exactly.
///
/// What that costs an operator is worse than a cosmetic overlap. The arrow
/// wins the hit test, so on a scrolled band **the leading control is
/// unreachable and clicking it scrolls the ribbon instead** — a control that
/// looks normal, is drawn normally, and does something entirely different from
/// what it says.
///
/// ★ The band must be SCROLLED for the left arrow to exist at all, which is why
/// this test drives a click rather than merely rendering — and why no
/// width-sweep test can stand in for it: those render a fresh, unscrolled band,
/// and there is no width at which an unscrolled band draws a left arrow.
#[test]
fn no_visible_group_overlaps_the_left_scroll_arrow() {
    let ctx = context();
    let shell = shell();
    let registry = registry();

    // ★★ SWEPT, not fixed at one width. At 180 pt a scrolled band draws no
    // group at all — only the two arrows — so a single-width version of this
    // test can run its loop body zero times and report green over the very
    // defect it exists to catch. The `examined` counter below is what makes
    // that failure loud instead of silent: two samples either side of a
    // transition look exactly like no transition.
    let mut examined = 0_usize;

    for width in (200..900).step_by(37).map(|w| w as f32) {
        let mut state = RibbonState::new();
        state.set_active_tab("view");

        let draw = |input: egui::RawInput, state: &mut RibbonState| -> Vec<(String, Rect)> {
            let mut rects = Vec::new();
            let mut sink = |name: &str, rect: Rect| rects.push((name.to_owned(), rect));
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, 400.0))),
                ..input
            };
            let _ = ctx.run_ui(input, |ui| {
                let _ = Ribbon::new()
                    .reporting_rects_to(&mut sink)
                    .render(ui, &shell, &registry, state);
            });
            rects
        };

        let before = draw(egui::RawInput::default(), &mut state);
        let Some(right) = before
            .iter()
            .find(|(n, _)| n == report::overflow())
            .map(|(_, r)| *r)
        else {
            continue; // Nothing overflows at this width; nothing to scroll.
        };

        let at = right.center();
        let mut input = egui::RawInput::default();
        input.events.extend([
            egui::Event::PointerMoved(at),
            egui::Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            },
        ]);
        let _ = draw(input, &mut state);
        let after = draw(egui::RawInput::default(), &mut state);

        let Some(arrow) = after
            .iter()
            .find(|(n, _)| n == "ribbon.scroll.left")
            .map(|(_, r)| *r)
        else {
            continue; // The click did not scroll (already at the end).
        };

        for (name, group) in after
            .iter()
            .filter(|(n, _)| n.starts_with("ribbon.group.view.") && !n.ends_with(".caption"))
        {
            examined += 1;
            assert!(
                group.left() + SLACK >= arrow.right(),
                "at {width} pt, {name} at {group:?} runs under the left scroll arrow at {arrow:?}. The arrow is drawn last, so it wins the hit test: that group's leading control is unreachable and clicking it scrolls the ribbon instead"
            );
        }
    }

    assert!(
        examined > 0,
        "no scrolled band anywhere in the sweep drew a group beside a left arrow, so this test asserted nothing at all — a check that cannot fail is not evidence"
    );
}
