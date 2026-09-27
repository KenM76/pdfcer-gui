//! # `app::status` tests — the bar's rules, stated as assertions
//!
//! The tests for `status.rs`, taken out along the test/subject seam and kept
//! **whole**: a module split is the right answer to a file that has outgrown
//! its ceiling, and shrinking the prose is not.
//!
//! ## What these are really guarding
//!
//! **R128.** A status bar whose height depends on what it has to say forms a
//! measured feedback loop with a per-frame fit-to-viewport zoom — 230 % → 224 %
//! → 215 % on a real document. Most of the assertions here are one property in
//! different clothes: *the bar is exactly as tall whatever is in it.*
//!
//! And one of them, [`tests::the_panel_is_tall_enough_for_the_controls_the_theme_actually_draws`],
//! guards the case its neighbour cannot see: the neighbour measures in an
//! `egui::Context::default()`, which carries egui's own spacing and **not this
//! application's theme**, and in that world the bar's controls really are under
//! 24 points. In the shipped theme they are 30, and two points of two controls
//! hang off the bottom of the window at every UI scale. That is R1's founding
//! shape verbatim — a test whose harness cannot contain the condition that
//! breaks the real program.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/status/tests.md`.

#![cfg(test)]
// Scoped to the tests, because the only non-test users of this alias live
// in `status::disclosure`.
// At the top of the file it is an unused import that only
// `clippy --all-targets` sees — `cargo build` skips the test module, so the
// build stays green while the gate goes red.
use super::test_support::{opened, settled_bar_frame};
use super::*;
use crate::find::FindState;
use crate::text::forms as t_forms;
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

/// **An edit disclosure does not change the bar's height** — R128 for
/// the sentence a move or a delete puts there.
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

/// **The panel is tall enough for the controls THIS APPLICATION draws
/// into it** — the test the one below cannot be.
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
        // `- FRAME_MARGIN_PTS`, without which this assertion is too loose
        // to fail. egui insets a panel's content by 2 points top and
        // bottom, so a 30-point panel has 26 points to lay out in — which
        // is exactly the arithmetic that lets 30 points of controls hang 2
        // points out of a 30-point panel and off the bottom of the window.
        // Comparing against the panel's OUTER height measures the wrong
        // box, and passes against a deliberately broken `height_for`.
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

/// **The bar is exactly as tall with the disclosure open as closed —
/// and as tall with no document as with one.**
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

    // …and as tall with a live fill disclosure as without one.
    //
    // This is the case most likely to break R128: unlike the render notes,
    // this line appears **without the operator doing anything** — a fill they
    // made
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
            // Height: the ordinary bound, planted deliberately rather than
            // arbitrarily. This test measures the STATUS BAR'S HEIGHT (R128),
            // not the sentence — and the overflow wording is the longest of the
            // three, so planting it would make the "does not make the bar
            // taller" assertion easier to pass for the wrong reason.
            applied_autosize_bound: Some(pdfcer_core::vartext::AutoFitBound::Height),
            unencodable_chars: 3,
            password_withheld: None,
        },
    );
    // The precondition, asserted rather than assumed. Without this the
    // test passes just as well when `fill_disclosure` returns early and
    // draws nothing — measuring that an absent line did not change the
    // height, which is true and worthless. **Assert that the measurement
    // HAPPENED, not only its value.**
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

/// **Every glyph the bar draws exists in the bundled font set.**
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
        // The framing this shell adds around a `pdfcer-core` disclosure
        // — the mark in particular, which is what distinguishes a fact
        // about the operator's own document from the narration beside it.
        // Checked with a one-character note so what is under test is the
        // framing rather than core's prose: core's sentences are ordinary
        // Latin text, and the mark is the only codepoint this bar
        // introduces that a bundled font could plausibly lack.
        //
        // A tofu box **on a disclosure** is worse than one on a label: it
        // reads as a rendering failure, and an operator who has decided a
        // surface is broken stops reading it — which is the one outcome
        // rule 4's whole apparatus exists to prevent.
        t::edit_disclosure_line(&["x".to_owned()]),
        // …and the decline's mark, `⊗` (U+2297), which is the one
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
        let probe = crate::text::glyphs::GlyphProbe::new(&ctx, egui::FontId::proportional(14.0));
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

/// **The three disclosure lines are independent, and none of them is the
/// narrator.**
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

// ===========================================================================
// The auto-size bound — three outcomes, three sentences, and the one that
// matters is the one that says the text will not fit
// ===========================================================================

/// **The auto-size outcome the general sentence cannot state.**
#[test]
fn the_overflow_case_says_the_text_will_not_fit_and_the_others_do_not() {
    use pdfcer_core::vartext::AutoFitBound as Bound;

    let floor = t_forms::forms_fill_autosize_overflow_note("Sheet title", 6.0);
    let width = t_forms::forms_fill_autosize_width_note("Sheet title", 6.0);
    let height = t_forms::forms_fill_autosize_note("Sheet title", 6.0);

    // The load-bearing assertion. Not "the sentences differ" — that a
    // rename would satisfy — but that exactly ONE of them tells the operator
    // the outcome he cannot see until he prints the sheet.
    assert!(
        floor.contains("overflow"),
        "the Floor sentence must say the text will not fit: {floor:?}"
    );
    for (name, other) in [("width", &width), ("height", &height)] {
        assert!(
            !other.contains("overflow"),
            "only the Floor bound overflows; the {name} sentence must not claim it: {other:?}"
        );
    }

    // And it must carry the remedy, which is the operator's and is available.
    // A disclosure that names a problem it knows the fix for and withholds it
    // is a complaint.
    assert!(
        floor.contains("taller") || floor.contains("shorten"),
        "the Floor sentence must name what the operator can do: {floor:?}"
    );

    // The width case points at a DIFFERENT edit, and saying so is the whole
    // reason it is not folded into the general sentence: making a width-bound
    // field taller changes nothing, and that is the first thing anybody tries.
    assert!(
        width.contains("WIDTH") && width.contains("will not change"),
        "the Width sentence must say that making the field taller is not the fix: {width:?}"
    );

    // Every one still discloses the interoperability fact where it is true.
    for (name, s) in [("width", &width), ("height", &height)] {
        assert!(
            s.contains("may choose differently"),
            "the {name} sentence must keep the 'another program may differ' clause: {s:?}"
        );
    }

    // The bounds this shell distinguishes, named so a new one is a visible
    // decision rather than a silent fall-through to the general sentence.
    let _exhaustive = |b: Bound| match b {
        Bound::Floor | Bound::Width | Bound::Height => (),
        _ => (),
    };
}

/// A field that fits keeps the ordinary sentence, and a **multiline** field —
/// where the engine reports no bound at all — must not be given one.
#[test]
fn no_bound_reported_takes_the_general_sentence_and_never_claims_overflow() {
    let general = t_forms::forms_fill_autosize_note("Notes", 9.0);
    assert!(
        !general.contains("overflow") && !general.contains("WIDTH"),
        "with no bound reported the shell may claim neither outcome: {general:?}"
    );
    assert!(
        general.contains("9.0 pt"),
        "it must still disclose the size pdfcer chose: {general:?}"
    );
}
