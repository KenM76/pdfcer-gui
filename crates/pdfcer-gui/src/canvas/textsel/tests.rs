//! # `canvas::textsel` tests — driven against real extractions of real files
//!
//!
//! ## Every assertion here drives the ENGINE
//!
//! `PageText`, `TextRun` and `ExtractedGlyph` are all `#[non_exhaustive]`, so
//! this crate cannot construct one. That is a constraint worth naming rather
//! than working around, because it means there is no way to write a test here
//! that passes against a fixture the engine would extract differently — every
//! number below came out of a real file.
//!
//! Two fixtures, from two trees, and the difference matters:
//!
//! | fixture | tree | what it is for |
//! |---|---|---|
//! | `pageops/four-pages.pdf` | the engine's, READ-ONLY | ordinary horizontal text — the rules of §1–§7 |
//! | `rotated-text.pdf` | **this** repository's `fixtures/` | strings at 0°, 90°, 180°, 270° and 30° — §8. The engine's corpus contains no such page, which is why this one had to be authored; see [`super::fixture`] |
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textsel/tests.md`.

#![cfg(test)]
// The INNER attribute, and it is load-bearing rather than redundant beside
// the `#[cfg(test)] mod tests;` that declares this file.
//
// `tools/gates/check-ui-strings.sh` and `check-theme-colors.sh` both recognise
// this exact line as "nothing in this file reaches the shipped binary", and
// both say why it is the marker rather than the filename: the property that
// earns the exemption is *not in the release build*, and a filename is a
// restatement of that which goes stale the moment a third such module exists.
//

use super::*;
use crate::app::state::{FOUR_PAGES, OpenDoc, ROTATED_TEXT, open_fixture, open_local_fixture};

/// Run `body` against page 0 of a fixture, with the real extraction.
fn on_page<R>(body: impl FnOnce(&PageContext<'_>) -> R) -> R {
    let doc: OpenDoc = open_fixture(FOUR_PAGES);
    let text = doc.page_text().expect("the fixture's first page extracts");
    let page = doc.pages.first().expect("the fixture has pages");
    body(&PageContext {
        text: &text,
        page,
        index: 0,
        epoch: doc.edit_epoch,
    })
}

/// Run `body` against page 0 of **this** repository's rotated-text fixture.
fn on_rotated_page<R>(body: impl FnOnce(&PageContext<'_>) -> R) -> R {
    let doc: OpenDoc = open_local_fixture(ROTATED_TEXT);
    let text = doc.page_text().expect("the fixture's page extracts");
    let page = doc.pages.first().expect("the fixture has a page");
    body(&PageContext {
        text: &text,
        page,
        index: 0,
        epoch: doc.edit_epoch,
    })
}

/// The canvas point on a rotated string, `fraction` of the way along it.
fn on_string(ctx: &PageContext<'_>, word: &str, fraction: f32) -> Pos2 {
    let model = pdfcer_core::text_edit::EditableTextModel::recognize(
        ctx.text,
        &pdfcer_core::text_edit::BlockRecognitionOptions::default(),
    );
    let line = model
        .lines()
        .iter()
        .find(|l| {
            l.glyphs
                .iter()
                .filter_map(|g| {
                    let run = ctx.text.runs.get(g.run)?;
                    let glyph = run.glyphs.get(g.glyph)?;
                    let lo = glyph.text_start as usize;
                    run.text.get(lo..lo + glyph.text_len as usize)
                })
                .collect::<String>()
                == word
        })
        .unwrap_or_else(|| panic!("the fixture no longer carries a rotated {word}"));
    let glyph = |g: &pdfcer_core::text_edit::GlyphRef| {
        ctx.text
            .runs
            .get(g.run)
            .and_then(|r| r.glyphs.get(g.glyph))
            .expect("the reference is live")
    };
    let first = glyph(line.glyphs.first().expect("a line has glyphs"));
    let last = glyph(line.glyphs.last().expect("a line has glyphs"));
    // The band's length, from the first origin to the last plus its
    // advance, measured along the line's own direction — so `fraction` is a
    // position in the STRING rather than a guess in points. A guessed
    // coordinate that lands past the end is symptom-identical to a hit test
    // that is broken, and that confusion has already cost this project one
    // retracted defect.
    let dir = line.direction;
    let span = (last.x - first.x, last.y - first.y);
    let length = span.0.mul_add(dir.0, span.1 * dir.1) + last.advance;
    let along = fraction * length;
    // A quarter of the size off the baseline, towards the ascender, which
    // is where the ink is.
    let up = (-dir.1, dir.0);
    let pdf = egui::pos2(
        along.mul_add(dir.0, (first.size * 0.25).mul_add(up.0, first.x)),
        along.mul_add(dir.1, (first.size * 0.25).mul_add(up.1, first.y)),
    );
    crate::viewer::pdf_space_to_canvas(pdf, ctx.page).expect("a real page projects")
}

// =======================================================================
// §8 — text that does not run along the page's x axis
// =======================================================================

/// **The operator's report, as an assertion.**
#[test]
fn sweeping_a_vertical_string_copies_it_on_one_line() {
    on_rotated_page(|ctx| {
        let from = on_string(ctx, "UPWARD", 0.02);
        let to = on_string(ctx, "UPWARD", 0.98);
        let selection = drag(ctx, from, to).expect("the sweep covers the string");
        assert!(
            !selection.text.contains('\n'),
            "the copy still breaks a rotated line at every letter: {:?}",
            selection.text
        );
        assert_eq!(selection.text, "UPWARD");
    });
}

/// **And it shades as one block**, which is the other half of the same
/// report.
#[test]
fn a_vertical_selection_is_one_tall_band() {
    on_rotated_page(|ctx| {
        let from = on_string(ctx, "UPWARD", 0.02);
        let to = on_string(ctx, "UPWARD", 0.98);
        let selection = drag(ctx, from, to).expect("the sweep covers the string");
        assert_eq!(
            selection.quads.len(),
            1,
            "a rotated string should band as one box, not one per letter"
        );
        let band = selection.quads[0];
        assert!(
            band.height() > band.width() * 2.0,
            "the band is {:.1} wide by {:.1} tall, which is not a vertical string",
            band.width(),
            band.height()
        );
    });
}

/// **The 180° string too**, and it reaches the fix by a different route.
#[test]
fn an_upside_down_string_copies_on_one_line_too() {
    on_rotated_page(|ctx| {
        let from = on_string(ctx, "INVERTED", 0.02);
        let to = on_string(ctx, "INVERTED", 0.98);
        let selection = drag(ctx, from, to).expect("the sweep covers the string");
        assert_eq!(selection.text, "INVERTED");
        assert_eq!(selection.quads.len(), 1);
    });
}

/// **The horizontal string is untouched**, asserted against the same
/// page that contains four rotated ones.
#[test]
fn horizontal_text_on_a_rotated_page_is_unchanged() {
    on_rotated_page(|ctx| {
        let selection = select_all(ctx).expect("the page has text");
        assert!(
            selection.text.starts_with("HORIZONTAL"),
            "the horizontal string is no longer first in content order: {:?}",
            selection.text
        );
        let horizontal = selection
            .quads
            .first()
            .copied()
            .expect("select-all produces boxes");
        assert!(
            horizontal.width() > horizontal.height() * 2.0,
            "the horizontal string banded as {:.1} x {:.1}",
            horizontal.width(),
            horizontal.height()
        );
    });
}

/// **The operator's OWN file**, which is the only evidence that any of the
/// above matters.
#[test]
#[ignore = "reads a customer drawing outside the repository; run deliberately"]
fn the_operators_own_vertical_stamp_comes_back_whole() {
    let path = std::path::Path::new("D:/Dev/temp/pdfcer/SW41177.pdf");
    assert!(
        path.exists(),
        "this check reads the operator's own drawing at {}. It is not committed \\
         and never will be; if it has moved, point this line at it or skip the check.",
        path.display()
    );
    let doc = pdfcer_core::document::Document::load(path).expect("the drawing loads");
    let session = pdfcer_core::edit::EditSession::new(doc);
    let view = session.view();
    let pages = pdfcer_core::page_tree::pages_in(&view).expect("a page tree");
    let last = pages.len() - 1;
    let opts = pdfcer_core::text_extract::ExtractOptions::default();
    let text = pdfcer_core::text_extract::extract_page_view(&view, &pages[last], last, &opts)
        .expect("the last page's text extracts");

    let model = pdfcer_core::text_edit::EditableTextModel::recognize(
        &text,
        &pdfcer_core::text_edit::BlockRecognitionOptions::default(),
    );
    let words: Vec<String> = model
        .lines()
        .iter()
        // Only the rotated ones, which is what this probe is about. Before
        // `Pass 139.2` the shell had to recover that fact; the engine publishes
        // it now, and every glyph on a line shares it by construction.
        .filter(|line| line.direction.1.abs() > f32::EPSILON || line.direction.0 < 0.0)
        .map(|line| {
            line.glyphs
                .iter()
                .filter_map(|g| {
                    let run = text.runs.get(g.run)?;
                    let glyph = run.glyphs.get(g.glyph)?;
                    let lo = glyph.text_start as usize;
                    run.text.get(lo..lo + glyph.text_len as usize)
                })
                .collect()
        })
        .collect();
    eprintln!("rotated lines on page {}: {words:#?}", last + 1);

    let stamp = words
        .iter()
        .find(|w| w.contains("SW41177"))
        .unwrap_or_else(|| {
            panic!("no rotated line on the last page carries the drawing number: {words:#?}")
        });
    assert!(!stamp.contains('\n'), "the stamp still breaks: {stamp:?}");
    assert!(
        stamp.len() > 40,
        "the stamp came back as a fragment rather than a path: {stamp:?}"
    );
}

/// **The cursor's question, answered in CANVAS space.**
#[test]
fn the_cursor_is_told_which_way_the_text_under_it_runs() {
    on_rotated_page(|ctx| {
        let on_upward = on_string(ctx, "UPWARD", 0.5);
        let tilt = tilt_at(ctx, on_upward).expect("the pointer is over the 90° string");
        assert!(
            (tilt + 90.0).abs() < 1.0,
            "a string running UP the page is -90° on a Y-down canvas, not {tilt}°"
        );

        // 30°, where the sign convention is the whole answer rather than a
        // quarter turn that happens to be symmetric.
        let on_skewed = on_string(ctx, "SKEWED", 0.5);
        let skew = tilt_at(ctx, on_skewed).expect("the pointer is over the 30° string");
        assert!(
            (skew + 30.0).abs() < 2.0,
            "a string rising at 30° in the FILE falls at -30° on screen, not {skew}°"
        );

        // Ordinary text needs no answer, and blank paper must not be given one.
        let on_horizontal = crate::viewer::pdf_space_to_canvas(egui::pos2(80.0, 703.0), ctx.page)
            .expect("a real page projects");
        assert!(
            tilt_at(ctx, on_horizontal).is_none(),
            "horizontal text asked the cursor to turn"
        );
        let blank = crate::viewer::pdf_space_to_canvas(egui::pos2(400.0, 310.0), ctx.page)
            .expect("a real page projects");
        assert!(
            tilt_at(ctx, blank).is_none(),
            "blank paper asked the cursor to turn"
        );
    });
}

/// **A rotated band's `/QuadPoints` are the true parallelogram**, not its
/// bounding box.
#[test]
fn a_skewed_band_is_marked_as_a_parallelogram() {
    on_rotated_page(|ctx| {
        let from = on_string(ctx, "SKEWED", 0.02);
        let to = on_string(ctx, "SKEWED", 0.98);
        let selection = drag(ctx, from, to).expect("the sweep covers the string");
        let quad = selection.page_quads.first().copied().expect("one band");
        assert!(
            (quad.ul.1 - quad.ur.1).abs() > 1.0,
            "the marked quad is axis-aligned, so it is a bounding box rather than the band: {quad:?}"
        );
    });
}

/// The canvas point at the centre of the first glyph the page draws.
fn first_glyph_centre(ctx: &PageContext<'_>) -> Pos2 {
    let run = ctx
        .text
        .runs
        .iter()
        .find(|r| !r.glyphs.is_empty())
        .expect("the fixture's page draws glyphs");
    let g = run.glyphs.first().expect("checked non-empty");
    let pdf = egui::pos2(g.x + g.advance / 2.0, g.y + g.size * 0.25);
    crate::viewer::pdf_space_to_canvas(pdf, ctx.page).expect("a real page projects")
}

// =======================================================================
// One derivation — module header §5
// =======================================================================

/// **What is highlighted is what is copied.**
#[test]
fn a_selection_carries_its_text_and_its_boxes_from_one_pass() {
    on_page(|ctx| {
        let all = select_all(ctx).expect("the fixture's page has text");
        assert!(!all.text.is_empty(), "select-all copied nothing");
        assert!(!all.quads.is_empty(), "select-all highlighted nothing");
        let lines = model(ctx).lines().len();
        assert!(
            all.quads.len() <= lines.max(1),
            "{} boxes for {lines} derived lines — the per-line grouping is not grouping",
            all.quads.len()
        );
        assert_eq!(all.page, 0);
        assert!(all.live(ctx.epoch));
    });
}

/// **The boxes exist in both spaces, index for index** — module header
/// §5.1.
#[test]
fn every_painted_box_has_the_page_space_quad_a_markup_would_use() {
    on_page(|ctx| {
        let all = select_all(ctx).expect("the fixture's page has text");
        assert!(!all.page_quads.is_empty(), "no quads to author from");
        assert_eq!(
            all.quads.len(),
            all.page_quads.len(),
            "the wash and the mark must describe the same boxes"
        );
        for (canvas, quad) in all.quads.iter().zip(&all.page_quads) {
            let quad_width = (quad.ur.0 - quad.ul.0).abs();
            let quad_height = (quad.ul.1 - quad.ll.1).abs();
            assert!(
                quad_width > 0.0 && quad_height > 0.0,
                "a degenerate quad marks nothing: {quad:?}"
            );
            assert!(
                (f64::from(canvas.width()) - quad_width).abs() < 1.0
                    || (f64::from(canvas.height()) - quad_width).abs() < 1.0,
                "the painted box {canvas:?} and the authored quad {quad:?} are not the \
                 same box"
            );
        }
        // …and `marks` is the accessor that enforces the revision, exactly
        // as `highlights` does for the painted half.
        assert_eq!(all.marks(ctx.epoch).len(), all.page_quads.len());
        assert!(
            all.marks(ctx.epoch + 1).is_empty(),
            "a stale selection must not author an annotation over glyphs that may have moved"
        );
    });
}

/// **An edit makes a selection stale, and a stale selection paints
/// nothing** — module header §7.
#[test]
fn an_edit_makes_a_selection_stale_and_stops_the_highlight() {
    on_page(|ctx| {
        let all = select_all(ctx).expect("the fixture's page has text");
        assert!(!all.highlights(0, ctx.epoch).is_empty());
        assert!(!all.live(ctx.epoch + 1), "one edit later");
        assert!(
            all.highlights(0, ctx.epoch + 1).is_empty(),
            "a quad recorded before an edit may cover different glyphs after it"
        );
        assert!(
            all.highlights(1, ctx.epoch).is_empty(),
            "…and a selection describes one page, so another page's overlay gets nothing"
        );
    });
}

// =======================================================================
// The gestures
// =======================================================================

/// **A double-click selects a word, and a triple-click selects at least as
/// much.**
#[test]
fn a_double_click_takes_a_word_and_a_triple_click_takes_at_least_the_line() {
    on_page(|ctx| {
        let at = first_glyph_centre(ctx);
        let word = click(ctx, None, at, false, true, false)
            .expect("a double-click on a glyph selects its word");
        let line = click(ctx, None, at, false, false, true)
            .expect("a triple-click on a glyph selects its line");
        assert!(!word.text.is_empty());
        assert!(
            !word.text.trim().contains(char::is_whitespace),
            "a word must not span a space: {:?}",
            word.text
        );
        assert!(
            line.text.len() >= word.text.len(),
            "a line ({:?}) cannot be shorter than a word inside it ({:?})",
            line.text,
            word.text
        );
    });
}

/// **A plain click clears** — Acrobat, Inkscape and SolidWorks alike.
#[test]
fn a_plain_click_clears_the_selection() {
    on_page(|ctx| {
        let at = first_glyph_centre(ctx);
        assert!(
            click(ctx, None, at, false, false, false).is_none(),
            "a click collapses the range, and an empty range is no selection"
        );
    });
}

/// **A drag selects the range between its ends, and it is
/// direction-blind.**
#[test]
fn a_drag_selects_the_same_range_in_both_directions() {
    on_page(|ctx| {
        let all = select_all(ctx).expect("the fixture's page has text");
        // Two points well inside the selection's own first box, so the drag
        // is known to be over glyphs rather than guessed to be.
        let box_ = all.quads[0];
        let left = egui::pos2(box_.min.x + 1.0, box_.center().y);
        let right = egui::pos2(box_.max.x - 1.0, box_.center().y);

        let forward = drag(ctx, left, right).expect("a drag across a line selects it");
        let backward = drag(ctx, right, left).expect("…and so does the same drag reversed");
        assert_eq!(
            forward.text, backward.text,
            "a gesture must mean the same thing in both directions"
        );
        assert_eq!(forward.quads, backward.quads);
        assert!(!forward.text.is_empty());
    });
}

/// Shift+click extends from the anchor rather than starting again — and with
/// nothing selected it behaves as a plain click, because there is no anchor
/// to extend from.
#[test]
fn shift_click_extends_from_the_anchor_and_needs_one() {
    on_page(|ctx| {
        let all = select_all(ctx).expect("the fixture's page has text");
        let box_ = all.quads[0];
        let start = egui::pos2(box_.min.x + 1.0, box_.center().y);
        let end = egui::pos2(box_.max.x - 1.0, box_.center().y);

        let quarter = egui::pos2(box_.min.x + box_.width() / 4.0, box_.center().y);
        let seed = drag(ctx, start, quarter).expect("a quarter-line sweep selects glyphs");
        let extended = click(ctx, Some(&seed), end, true, false, false)
            .expect("shift+click extends to the pointer");
        assert!(
            extended.text.len() > seed.text.len(),
            "extending must grow the range: {:?} then {:?}",
            seed.text,
            extended.text
        );

        assert!(
            click(ctx, None, end, true, false, false).is_none(),
            "shift+click with nothing selected has no anchor, so it clears like a plain click"
        );
    });
}

/// Ctrl+A takes the whole page and nothing beyond it — the range is clamped
/// by `resolve_range`, so the last run's end is a real boundary rather than
/// a byte past one.
#[test]
fn select_all_takes_every_character_on_the_page() {
    on_page(|ctx| {
        let all = select_all(ctx).expect("the fixture's page has text");
        let plain = ctx.text.plain_text();
        assert!(
            plain.split_whitespace().count() >= 4,
            "vacuous unless the fixture really has several words: {plain:?}"
        );
        for word in plain.split_whitespace() {
            assert!(
                all.text.contains(word),
                "select-all dropped {word:?} from {:?}",
                all.text
            );
        }
        // …and the separators came too, or the copy would paste as one word.
        assert!(
            all.text.contains(char::is_whitespace),
            "a copy that drops the derived spaces pastes as one word: {:?}",
            all.text
        );
    });
}

/// A drag that touches no glyph selects nothing.
#[test]
fn a_degenerate_drag_selects_nothing() {
    on_page(|ctx| {
        let far = egui::pos2(-10_000.0, -10_000.0);
        assert!(
            drag(ctx, far, far).is_none(),
            "a zero-length drag covers no glyphs, wherever it is"
        );
    });
}

// =======================================================================
// Overshooting the text — the clamp in `drag`
//
// `EditableTextModel::hit_test` answers `None` beyond one line-height of
// every line, which this shell asked for. The consequence it did not ask
// for is that a focus past the end of a run resolved to nothing and took
// the whole selection with it.
// =======================================================================

/// **A sweep that runs off the end of a line keeps what it swept.**
#[test]
fn overshooting_the_end_of_a_line_keeps_the_selection() {
    on_rotated_page(|ctx| {
        let from = on_string(ctx, "HORIZONTAL", 0.02);
        for x in [170.0_f32, 200.0] {
            let to = crate::viewer::pdf_space_to_canvas(egui::pos2(x, 703.0), ctx.page)
                .expect("a real page projects");
            assert!(
                hit(&model(ctx), ctx, to).is_none(),
                "x={x} was supposed to be OUTSIDE every line's reach — if the engine has \
                 widened it, this test is no longer about the clamp"
            );
            let swept = drag(ctx, from, to)
                .unwrap_or_else(|| panic!("the sweep to x={x} cancelled itself"));
            assert_eq!(swept.text, "HORIZONTAL");
        }
    });
}

/// **And it does not stop at the first gap** — the clamp scans backwards
/// from the pointer, so a sweep that crosses blank paper and keeps going
/// selects through to the furthest text it passed.
#[test]
fn the_clamp_finds_the_furthest_text_the_drag_passed_not_the_nearest() {
    on_rotated_page(|ctx| {
        let from = on_string(ctx, "HORIZONTAL", 0.02);
        let to = crate::viewer::pdf_space_to_canvas(egui::pos2(580.0, 703.0), ctx.page)
            .expect("a real page projects");
        assert!(hit(&model(ctx), ctx, to).is_none());
        let swept = drag(ctx, from, to).expect("the sweep cancelled itself");
        assert!(
            swept.text.contains("DOWNWARD") || swept.text.contains("INVERTED"),
            "the clamp stopped at the near edge of the gap: {:?}",
            swept.text
        );
    });
}

/// A sweep begun on blank paper still selects nothing. The clamp widens what
/// a resolvable **anchor** can reach; it does not manufacture one.
#[test]
fn a_sweep_begun_off_the_text_still_selects_nothing() {
    on_rotated_page(|ctx| {
        let blank = |x: f32, y: f32| {
            crate::viewer::pdf_space_to_canvas(egui::pos2(x, y), ctx.page)
                .expect("a real page projects")
        };
        assert!(drag(ctx, blank(560.0, 60.0), blank(580.0, 60.0)).is_none());
    });
}
// =======================================================================
// Ordering
//
// The two keyboard verbs and the cost gate in front of them moved to
// `clipboard.rs` with their tests — see this module's §8.
// =======================================================================

/// The ordering helper puts the earlier position first, on both axes of the
/// key — the run before the offset, which is the order content is in.
#[test]
fn positions_order_by_run_then_offset() {
    let a = TextPosition::new(1, 5);
    let b = TextPosition::new(1, 9);
    let c = TextPosition::new(2, 0);
    assert_eq!(ordered(a, b), (a, b));
    assert_eq!(
        ordered(b, a),
        (a, b),
        "the same pair reversed must order the same way"
    );
    // Across runs, the run index decides regardless of the offsets — a
    // position at byte 0 of run 2 is after byte 5 of run 1.
    assert_eq!(ordered(c, a), (a, c));
    assert_eq!(ordered(a, c), (a, c));
}
