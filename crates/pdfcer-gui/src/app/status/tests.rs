//! # `app::status` tests — the bar's rules, and the two defects it surfaced
//!
//! Split out of `status.rs` on 2026-08-26 under R2, when the narrow-window shed
//! rule and the theme-derived panel height took that file to 1,561 lines.
//! Nothing moved but the tests, and they moved **whole**: the gate's own header
//! is explicit that the right response to it firing is to split the module, not
//! to shrink the prose.
//!
//! ## ★ What these are really guarding
//!
//! **R128.** A status bar whose height depends on what it has to say forms a
//! measured feedback loop with a per-frame fit-to-viewport zoom — 230 % → 224 %
//! → 215 % on a real document. Most of the assertions here are one property in
//! different clothes: *the bar is exactly as tall whatever is in it.*
//!
//! ★★ And one of them, [`tests::the_panel_is_tall_enough_for_the_controls_the_theme_actually_draws`],
//! guards the case its neighbour could not see: the neighbour measures in an
//! `egui::Context::default()`, which carries egui's own spacing and **not this
//! application's theme**, and in that world the bar's controls really are under
//! 24 points. In the shipped theme they are 30, and two points of two controls
//! were clipped off the bottom of the window at every UI scale. That is R1's
//! founding shape verbatim — a test whose harness cannot contain the condition
//! that breaks the real program.

#![cfg(test)]
// ★ Scoped to the tests, because the non-test users of this alias moved
// into `status::disclosure` when the three rule-4 lines were split out.
// At the top of the file it is an unused import that only
// `clippy --all-targets` sees — `cargo build` skips the test module, so the
// build stays green while the gate goes red.
use super::test_support::{opened, settled_bar_frame};
use super::*;
use crate::find::FindState;
use crate::text::status as t;
use egui::{Context, RawInput};

// =======================================================================
// R128 — the height that must not move
// =======================================================================

/// Measure the height [`show`] consumes for one frame.
fn bar_height(ctx: &Context, status: &Status) -> f32 {
    let mut height = f32::NAN;
    let mut find = FindState::default();
    let mut filter = PickFilter::default();
    let mut max_zoom = crate::app::prefs::DEFAULT_MAX_ZOOM_PERCENT;
    let _ = ctx.run_ui(RawInput::default(), |ui| {
        let mut actions = Vec::new();
        height = ui
            .scope(|ui| {
                show(
                    ui,
                    status,
                    &mut find,
                    &mut filter,
                    &mut max_zoom,
                    &mut crate::app::prefs::WheelPaging::default(),
                    &mut actions,
                )
            })
            .response
            .rect
            .height();
    });
    height
}

/// ★ **An edit disclosure does not change the bar's height** — R128 for
/// the sentence a move or a delete puts there.
///
/// # Why this needs its own test beside the fill one
///
/// Same rule, different arrival. A fill disclosure follows the operator
/// typing into a field; this one follows a **drag on the canvas**, which
/// is the gesture during which a re-fit is most damaging — R128's measured
/// symptom is *"the page jumped when I clicked an object"*, and a drag is
/// a click that has not finished. If this line grew the bar, the page
/// would re-fit on the frame the operator released the mouse, the object
/// would land somewhere other than where they dropped it, and the
/// investigation would start in the move code, where nothing is wrong.
///
/// # The three assertions, and why none of them is the obvious one
///
/// 1. **A measurement happened at all** (`Some(_)`, never `None`) — see
///    [`bar_frame`] for why a bare `f32` would let a vacuous run pass.
/// 2. **The sentence reached the painter** — more shapes with the
///    disclosure live than without it. Without this, assertion 3 is
///    satisfied just as well by a `disclosure_line` that returned early
///    and drew nothing, which is true and proves nothing.
/// 3. **The height did not move.** Asserted as `Some(true)` rather than
///    with a bare `assert!`, so a run in which either frame failed to
///    measure reads as `None` and fails, rather than reading as agreement.
///
/// The planted notes are the **worst case that can actually occur**: two
/// of `pdfcer-core`'s real sentences at once, which is what a node drag
/// that both expands a rectangle and materialises an implicit start
/// returns. They are long, and long is the point — the defence against
/// them is eliding inside a bounded sub-region with the whole text on
/// hover, not wrapping, because wrapping is how a one-row bar becomes a
/// two-row bar.
#[test]
fn an_edit_disclosure_does_not_change_the_bar_height() {
    let ctx = Context::default();
    let status = opened();
    let Status::Open(doc) = &status else {
        unreachable!("`opened()` returns an open document");
    };

    let absent = settled_bar_frame(&ctx, &status);

    crate::app::actions::plant_edit_disclosure_for_test(crate::app::actions::EditDisclosure {
        epoch: doc.edit_epoch,
        notes: vec![
            "This shape was stored as a rectangle, which can only describe a box with \
                 square corners. Moving a corner independently makes it a four-sided shape \
                 that is no longer a box, so it has been rewritten as four lines. It draws \
                 identically; dragging the corner back will not restore the original \
                 rectangle form."
                .to_owned(),
            "This point had no coordinates of its own — the file re-used the start of \
                 the shape before it. A move instruction naming the point has been added so \
                 it can be placed independently."
                .to_owned(),
        ],
    });
    // The precondition, asserted rather than assumed — the same shape the
    // fill test above uses, and for the same reason: without it the height
    // comparison below measures that an absent line did not change the
    // height.
    assert!(
        crate::app::actions::last_edit_disclosure(doc.edit_epoch).is_some(),
        "the planted disclosure is not live for this document's epoch, so the bar drew \
         no line and everything below proves nothing"
    );

    let present = settled_bar_frame(&ctx, &status);

    let drew = match (absent, present) {
        (Some((_, before)), Some((_, after))) => Some(after > before),
        _ => None,
    };
    assert_eq!(
        drew,
        Some(true),
        "the bar painted no more shapes with a live edit disclosure ({absent:?}) than \
         without one ({present:?}); the sentence never reached the painter, so the height \
         comparison would be vacuous. `None` here means a frame did not measure at all, \
         which is the other failure and is not a pass"
    );

    let same_height = match (absent, present) {
        (Some((before, _)), Some((after, _))) => Some((after - before).abs() < 0.01),
        _ => None,
    };
    assert_eq!(
        same_height,
        Some(true),
        "an edit disclosure changed the bar's height ({absent:?} → {present:?}); that \
         re-fits the page on the frame an operator finishes a drag, and the symptom is \
         read as a move bug"
    );
}

/// ★★★ **The panel is tall enough for the controls THIS APPLICATION draws
/// into it** — the test the one below could not be.
///
/// `the_bar_is_exactly_as_tall_open_as_closed` asserts the same property
/// and passes against a build where it is false, because it measures in an
/// `egui::Context::default()` — egui's own spacing, not this application's
/// theme. In that world the controls are under 24 points and the assertion
/// is true. In the real window `Metrics::control_height` is 28, a button
/// adds 2 points of padding on each side, and the bar's content is 30
/// against a panel whose content box was 26.
///
/// Measured on a real window at both scales before this was written:
/// `status-bar 972.0 .. 1002.0` in a 1000-point client at 1.00, and
/// `416.4 .. 446.4` in a 444.4-point client at 1.80. Two points of two
/// controls clipped off the bottom of the window, at every scale.
///
/// ★ So this one **applies the theme** and asserts against every preset the
/// application ships, not just the current one — because the defect was
/// introduced by a preset raising its control height, and the next one will
/// be too.
#[test]
fn the_panel_is_tall_enough_for_the_controls_the_theme_actually_draws() {
    for preset in egui_shell::theme::Preset::ALL {
        let preset = *preset;
        let theme = egui_shell::theme::Theme::new(preset);
        let ctx = Context::default();
        theme.apply(&ctx);
        // Two frames: egui settles over a pass, and a first-frame galley is
        // not a steady-state measurement. Same reason `settled_bar_frame`
        // exists.
        let _ = bar_height(&ctx, &opened());
        let content = bar_height(&ctx, &opened());
        let panel = height_for(&theme);
        // ★ `- FRAME_MARGIN_PTS`, and the first version of this assertion
        // omitted it and was therefore too loose to fail. egui insets a
        // panel's content by 2 points top and bottom, so a 30-point panel
        // has 26 points to lay out in — which is exactly the arithmetic
        // that let 30 points of controls hang 2 points out of a 30-point
        // panel and off the bottom of the window. Comparing against the
        // panel's OUTER height measures the wrong box, and it passed
        // against a deliberately broken `height_for`.
        let usable = panel - FRAME_MARGIN_PTS;
        assert!(
            usable >= content,
            "the `{preset:?}` theme draws {content} pt of status bar into a panel pinned at \
             {panel} pt, whose content box is {usable} pt — so the bottom {:.1} pt of its \
             controls are clipped off the window. `status::height_for` must account for \
             `Metrics::control_height` ({}) plus the button padding either side.",
            content - usable,
            theme.metrics.control_height
        );
    }
}

/// ★ **The bar is exactly as tall with the disclosure open as closed —
/// and as tall with no document as with one.**
///
/// Rule R128, asserted rather than argued. A status panel whose height
/// varies feeds the fit-to-viewport recompute, and the measured result
/// on pdfcer was a page that shrank 230 % → 224 % → 215 % across three
/// frames with no zoom input, plus click coordinates that went stale
/// between the frame they were captured on and the next render. The
/// symptom reads as a selection bug and gets investigated in the
/// selection code, where nothing is wrong.
///
/// This is the property that forbids an [`egui::CollapsingHeader`] here:
/// changing its own height is the entire behaviour of that widget. It is
/// also what `ui.set_min_height` in [`show`] is for — without it the row
/// would shrink to whatever the content happened to need.
#[test]
fn the_bar_is_exactly_as_tall_open_as_closed() {
    let ctx = Context::default();
    let status = opened();
    let empty = Status::Empty;

    let closed_no_doc = bar_height(&ctx, &empty);

    ctx.data_mut(|d| d.insert_temp(egui::Id::new(notes::NOTES_OPEN_ID), false));
    let closed = bar_height(&ctx, &status);

    ctx.data_mut(|d| d.insert_temp(egui::Id::new(notes::NOTES_OPEN_ID), true));
    let open = bar_height(&ctx, &status);

    assert!(
        (open - closed).abs() < 0.01,
        "opening the render notes changed the bar's height ({closed} → \
         {open}); that is R128's feedback loop, and it is measured in \
         page zoom, not in pixels"
    );
    assert!(
        (closed_no_doc - closed).abs() < 0.01,
        "opening a document changed the bar's height ({closed_no_doc} → \
         {closed}), which re-fits the page on the frame it opens"
    );
    assert!(
        closed <= ROW_HEIGHT_PTS + 0.01,
        "the bar's content ({closed} pt) overflowed its allocated row \
         ({ROW_HEIGHT_PTS} pt); either the row is too short or something \
         here is laying out vertically"
    );

    // ★ …and as tall with a live fill disclosure as without one.
    //
    // Added 2026-08-14 with `fill_disclosure`, and it is the case most
    // likely to break R128 in future: unlike the render notes, this line
    // appears **without the operator doing anything** — a fill they made
    // on the canvas puts a sentence in the bar on the next frame. If it
    // grew the bar, the page would silently re-fit at the moment the
    // operator finished typing into a field, and the symptom would be
    // "the page jumped when I filled in the form", investigated in the
    // form code, where nothing would be wrong.
    //
    // Two sentences at once is the worst case: both are joined onto one
    // line precisely so this stays a single row.
    let Status::Open(doc) = &status else {
        unreachable!("`opened()` returns an open document");
    };
    crate::panels::forms::edit::plant_fill_disclosure_for_test(
        crate::panels::forms::edit::FillDisclosure {
            field: "A field with a long enough name to need eliding".to_owned(),
            epoch: doc.edit_epoch,
            applied_autosize: Some(12.0),
            unencodable_chars: 3,
        },
    );
    // The precondition, asserted rather than assumed. Without this the
    // test passes just as well when `fill_disclosure` returned early and
    // drew nothing — measuring that an absent line did not change the
    // height, which is true and worthless. `HANDOFF.md` §10's rule: assert
    // the measurement HAPPENED, not only its value.
    assert!(
        crate::panels::forms::edit::last_fill_disclosure(doc.edit_epoch).is_some(),
        "the planted disclosure is not live for this document's epoch, so \
         the bar drew no line and the height comparison below proves nothing"
    );
    let disclosing = bar_height(&ctx, &status);
    assert!(
        (disclosing - closed).abs() < 0.01,
        "a fill disclosure changed the bar's height ({closed} → \
         {disclosing}); that re-fits the page on the frame an operator \
         finishes typing into a field"
    );
}

// =======================================================================
// Legibility — the labels that are glyphs
// =======================================================================

/// ★ **Every glyph the bar draws exists in the bundled font set.**
///
/// `⏴`, `⏵`, `⏷`, `−` and `·` are not decoration: three of them are the
/// entire visible text of a control. A codepoint the font set cannot
/// draw renders as a tofu box, which is defect D2's shape — an invisible
/// label — with the operator's page position behind it.
///
/// **This test has already paid for itself.** The catalog was written
/// with `◀` `▶` for the page steps and `▸` `▾` for the disclosure, and
/// all four are missing from egui's bundled fonts (Ubuntu-Light +
/// NotoEmoji + emoji-icon-font). They would have shipped as four tofu
/// boxes on the two controls an operator touches most.
///
/// Checked against `FontFamily::Proportional`, which is what every label
/// and button on this bar resolves to, and run inside a real pass because
/// egui has no fonts before one.
///
/// ## ★★ Corrected 2026-08-14: this test used to ask the wrong question
///
/// It called [`epaint::Fonts::has_glyph`], **which returns false
/// negatives**, and the one it returned here was expensive: it reported
/// `⚠` (U+26A0) as undrawable, `DEFECTS.md` D12 was filed on that
/// reading, and thirteen shipped sentences were recorded as rendering
/// tofu when they render correctly. `has_glyph` returns
/// `resolve_face(c) != replacement_face_key` — so it says "no" to every
/// codepoint whose first supporting face happens to be the face that also
/// supplies `epaint`'s substitution mark `◻`, which for the proportional
/// family is `NotoEmoji-Regular`, which is `⚠`'s supplier.
///
/// It now asks [`crate::icons::glyphs::GlyphProbe`], which lays the
/// character out and looks at what was drawn. The full mechanism, the
/// measurements and the three-sentinel fingerprint are in that module's
/// header.
///
/// **The mark on the edit-disclosure line was chosen under the wrong
/// reading and is deliberately left alone.** `⚑` draws, it is in the
/// bar today, and re-opening a settled copy decision on the strength of
/// a correction to the diagnosis is churn, not a fix. What changed is
/// what this test *knows*, not what it protects.
///
/// This gate is now the narrow, hand-listed one; the broad one is
/// [`crate::icons::glyphs::tests::every_glyph_the_catalog_draws_has_a_glyph`],
/// which reads the whole catalog from source and needs no list.
#[test]
fn every_glyph_the_status_bar_draws_has_a_glyph() {
    let ctx = Context::default();
    let labels: Vec<String> = vec![
        t::diagnostics_toggle(false).to_owned(),
        t::diagnostics_toggle(true).to_owned(),
        t::diagnostics_join(&["a".to_owned(), "b".to_owned()]),
        t::zoom_out().to_owned(),
        t::zoom_in().to_owned(),
        t::zoom_percent(100.0),
        t::fit_actual_size().to_owned(),
        t::fit_width().to_owned(),
        t::fit_height().to_owned(),
        t::fit_page().to_owned(),
        t::prev_page().to_owned(),
        t::next_page().to_owned(),
        t::page_of_total(42),
        t::page_number(37),
        t::page_clamped_note(99, 42, 42),
        t::page_rejected_note().to_owned(),
        // ★ The framing this shell adds around a `pdfcer-core` disclosure
        // — the mark in particular, which is what distinguishes a fact
        // about the operator's own document from the narration beside it.
        // Checked with a one-character note so what is under test is the
        // framing rather than core's prose: core's sentences are ordinary
        // Latin text, and the mark is the only codepoint this bar
        // introduces that a bundled font could plausibly lack.
        //
        // ★★ **The line was drafted with `⚠` and this test rejected it —
        // wrongly.** That rejection became `DEFECTS.md` D12, and the
        // diagnosis in it was backwards: `⚠` draws. The mark here stayed
        // `⚑`, and stays `⚑`; see the doc comment above for why a
        // corrected diagnosis is not a reason to re-litigate the copy.
        //
        // A tofu box **on a disclosure** is worse than one on a label: it
        // reads as a rendering failure, and an operator who has decided a
        // surface is broken stops reading it — which is the one outcome
        // rule 4's whole apparatus exists to prevent. That reasoning was
        // always right; only the measurement behind it was wrong.
        t::edit_disclosure_line(&["x".to_owned()]),
        // ★ …and the decline's mark, `⊗` (U+2297), which is the one
        // codepoint the worded decline introduces.
        //
        // Listed here as well as being swept by the catalog-wide gate,
        // because a tofu box **on a decline** is the worst place for one:
        // the sentence's whole job is to say that a command the operator
        // invoked did not run, and a line that opens with a broken box
        // reads as a rendering failure rather than as an answer. An
        // operator who has decided a surface is broken stops reading it.
        t::zoom_declined_no_selection().to_owned(),
        t::zoom_declined_not_drawn().to_owned(),
        // …and the save's decline, which wears the same `⊗`. Listed
        // separately rather than trusted to the two above, because the
        // list is what a reader consults to know which sentences were
        // measured, and a decline that reaches the bar without appearing
        // here is one nobody checked.
        t::save_copy_failed().to_owned(),
    ];

    let mut missing = Vec::new();
    let _ = ctx.run_ui(RawInput::default(), |ui| {
        let ctx = ui.ctx().clone();
        let probe = crate::icons::glyphs::GlyphProbe::new(&ctx, egui::FontId::proportional(14.0));
        for label in &labels {
            for c in label.chars() {
                if !probe.can_draw(&ctx, c) {
                    missing.push((label.clone(), c));
                }
            }
        }
    });

    assert!(
        missing.is_empty(),
        "these labels contain codepoints the bundled fonts cannot draw, \
         so they would render as tofu boxes: {missing:?}"
    );
}

/// ★★ **The three disclosure lines are independent, and none of them is the
/// narrator.**
///
/// Asserted as a truth table because the obvious mistake, when a third line is
/// added beside two existing ones, is to make them alternatives — an `else if`
/// chain that shows whichever fires first. They answer different questions and
/// can all be true at once:
///
/// | line | answers |
/// |---|---|
/// | fill | what a form fill had to INFER |
/// | edit | what a move or delete had to change about an object's FORM |
/// | recovered | how this FILE was assembled before anything was drawn |
///
/// A document opened from a damaged index, edited, and with a form filled owes
/// the operator all three.
#[cfg(test)]
mod disclosure_independence {
    /// The region names are a cross-repo contract with `ui-verify`; a rename is
    /// an API change, not a tidy-up.
    #[test]
    fn each_disclosure_publishes_its_own_region() {
        let names = [
            super::REGION_FILL_DISCLOSURE,
            super::REGION_EDIT_DISCLOSURE,
            super::REGION_RECOVERED,
        ];
        for (i, a) in names.iter().enumerate() {
            assert!(
                a.starts_with("status-group:"),
                "{a} is not in the status bar's region namespace"
            );
            for b in names.iter().skip(i + 1) {
                assert_ne!(
                    a, b,
                    "two disclosure lines share a region name, so a driven check asserting one would silently be reading the other"
                );
            }
        }
    }
}
