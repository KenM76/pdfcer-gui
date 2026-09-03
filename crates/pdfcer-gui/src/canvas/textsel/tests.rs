//! # `canvas::textsel` tests — driven against real extractions of real files
//!
//! Split out of `textsel.rs` on 2026-08-26 under R2, when §8's rotated-text
//! work took that file over the 1,500-line limit. Nothing moved but the tests,
//! and they moved **whole** rather than being thinned: the gate's own header is
//! explicit that the right response to it firing is to split the module, not to
//! shrink the prose.
//!
//! ## ★ Every assertion here drives the ENGINE
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

#![cfg(test)]
// ★ The INNER attribute, and it is load-bearing rather than redundant beside
// the `#[cfg(test)] mod tests;` that declares this file.
//
// `tools/gates/check-ui-strings.sh` and `check-theme-colors.sh` both recognise
// this exact line as "nothing in this file reaches the shipped binary", and
// both say why it is the marker rather than the filename: the property that
// earns the exemption is *not in the release build*, and a filename is a
// restatement of that which goes stale the moment a third such module exists.
//
// Without it this file's assertion messages are reported as operator-facing
// copy — 78 of them on the split that created it, which is exactly the noise
// exclusion 2 of that gate was written to remove. `canvas/selection/tests.rs`
// learned the same thing on 2026-08-18.

use super::*;
use crate::app::state::{FOUR_PAGES, OpenDoc, ROTATED_TEXT, open_fixture, open_local_fixture};

/// Run `body` against page 0 of a fixture, with the real extraction.
///
/// Everything below drives a **real** `PageText`: `PageText`, `TextRun` and
/// `ExtractedGlyph` are all `#[non_exhaustive]`, so this crate cannot build
/// one — which is a constraint worth naming rather than working around,
/// because it means every assertion here is about the engine's actual output
/// on an actual file.
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
///
/// [`on_page`]'s twin, and separate from it rather than parameterised,
/// because the two fixtures come from different trees for a reason
/// `app::state::open_local_fixture` sets out: the engine's corpus is
/// read-only and contains no page of rotated strings, so this shell had to
/// author one.
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
///
/// Derived from the string's own glyphs rather than from the generator's
/// matrix, so the helper keeps working if the fixture moves — and derived
/// at all, rather than guessed, for the reason [`first_glyph_centre`]
/// gives: a coordinate that misses is symptom-identical to a hit test that
/// is broken.
fn on_string(ctx: &PageContext<'_>, word: &str, fraction: f32) -> Pos2 {
    // ★ The ENGINE's lines since 2026-08-27. This used to read a shell-side
    // census that recovered the writing direction from glyph origins; `Pass
    // 139.2` publishes `Line::direction` from the text rendering matrix, and
    // the census is deleted. A test that kept its own copy of a rule the engine
    // now owns would keep passing while the product broke.
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
// ★ §8 — text that does not run along the page's x axis
// =======================================================================

/// ★★★ **The operator's report, as an assertion.**
///
/// > *"when I copy and paste into notepad, I get the text on one line as
/// > expected […] as it is now […] it pastes each letter onto its own
/// > line."*
///
/// Sweep the whole 90° string and the clipboard must hold `UPWARD` — six
/// characters, no newline. Before §8 it held `U\nP\nW\nA\nR\nD`.
///
/// The `\n` assertion is separate from the equality on purpose: an equality
/// that failed would report the whole string and leave a reader to spot the
/// escapes, and this is the exact character the defect is about.
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

/// ★★ **And it shades as one block**, which is the other half of the same
/// report.
///
/// > *"when I select the text it shades each letter as part of the same
/// > block."*
///
/// One quad, not six. And the quad must be **taller than it is wide**,
/// because the string runs up the page — a box of the right area in the
/// wrong orientation is exactly what the pre-§8 code produced and would
/// satisfy a count-only assertion.
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

/// ★★ **The 180° string too**, and it reaches the fix by a different route.
///
/// 90° and 270° break the extraction's *baseline* clause; 180° breaks its
/// *backward-jump* clause, because `advance` is published as a positive
/// magnitude and text advancing in −x therefore looks like a jump of twice
/// the advance at every glyph. Same symptom, different line of `classify`,
/// and a fix that handled only the vertical case would have looked complete
/// on the operator's own file.
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

/// ★★★ **The horizontal string is untouched**, asserted against the same
/// page that contains four rotated ones.
///
/// This is the regression guard with the hardest possible input: a page
/// whose census is *not* empty, so the rotated path is live, containing
/// ordinary text that must come out exactly as it always did — one line,
/// one box, wider than tall.
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

/// ★★★ **The operator's OWN file**, which is the only evidence that any of the
/// above matters.
///
/// `#[ignore]`d, and the reason is a rule rather than a convenience:
/// `SW41177.pdf` is a customer drawing exported from SOLIDWORKS, and the
/// standing instruction is that SolidWorks-derived work product does not enter
/// a repository that could be published. It cannot be committed as a fixture,
/// so it cannot be a test that runs on a clean checkout — it is a test that
/// runs **here**, on the operator's machine, against the file the report named.
///
/// Run it deliberately:
/// `cargo test -p pdfcer-gui --lib the_operators_own_vertical_stamp -- --ignored --nocapture`
///
/// ## What it checks that the synthetic fixture cannot
///
/// `fixtures/rotated-text.pdf` reproduces the *mechanism* — see
/// [`super::fixture`] — but it is a page this project wrote, and a page this
/// project wrote is a page this project already understood. The real stamp is
/// 82 glyphs of a Windows path in 8 pt Arial, laid down by SOLIDWORKS' own PDF
/// exporter, on page 36 of a 36-page drawing set, in a title block full of
/// other text. Three things about it were surprises:
///
/// * only **10 of its 72 runs** hold more than one glyph, which is what killed
///   the first design (see [`super::writing`] §2.1);
/// * its worst inter-glyph gap is **0.010 pt** against a 1.600 pt threshold, so
///   the strict `Break::None` rule costs it nothing;
/// * it sits at `x = 1205.8` on a landscape sheet — the far right in the
///   *file*, the bottom left **on screen**, because the page carries a
///   `/Rotate`. That is the difference `tilt_at` exists to handle and is
///   invisible in any fixture authored without one.
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
        // ★ Only the rotated ones, which is what this probe is about. Before
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

/// ★★★ **The cursor's question, answered in CANVAS space.**
///
/// > *"In Adobe when I hover over it the I cursor re-orients itself to match
/// > the text orientation […] as it is now the I cursor doesn't reorient."*
///
/// [`tilt_at`] is what turns the I-beam, and the assertion that matters is the
/// **sign**. `writing` measures directions in PDF user space, which is Y-up;
/// the cursor lives in canvas space, which is Y-down. A string running **up**
/// the page is `+90°` in the file and must come back as `−90°` on screen, and
/// an implementation that forgot the flip would pass every test that only
/// checked the magnitude — and would then draw a beam that leaned the wrong way
/// at every angle that is not a multiple of 90°, where nobody would notice
/// until they met a skewed stamp.
///
/// The negative half is asserted too: horizontal text and blank paper must both
/// answer `None`, so the beam stays upright rather than being told to turn by
/// zero degrees. Those are the same picture and different costs — `None` skips
/// a bitmap lookup — but more importantly they are different *claims*, and this
/// module's job is to make the claim rather than the picture.
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

/// ★ **A rotated band's `/QuadPoints` are the true parallelogram**, not its
/// bounding box.
///
/// The canvas wash is a `Rect` and therefore over-covers a 30° band at the
/// corners — that is stated in §8 and accepted. What must NOT happen is the
/// same approximation reaching [`TextSelection::page_quads`], because those
/// are written into the file as a text markup's `/QuadPoints` and an
/// over-covering highlight in a saved document is permanent.
///
/// Asserted by the one property that separates the two: a bounding box has
/// its corners on the page's axes, so `ul.1 == ur.1`. A real 30° band does
/// not.
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
///
/// Derived from the extraction rather than guessed, for the reason
/// `ui-verify`'s `coords` module gives about guessed points: a coordinate
/// that misses is symptom-identical to a hit test that is broken, and this
/// project has already filed one retracted defect on exactly that.
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
// ★ One derivation — module header §5
// =======================================================================

/// ★ **What is highlighted is what is copied.**
///
/// The brief's own requirement, asserted the only way it can be asserted
/// from outside: select every character on the page, and check that the
/// value carries *both* halves and that they describe the same thing —
/// non-empty text, at least one box, and a box count that cannot exceed the
/// number of lines the engine derived.
///
/// The last clause is what makes this more than "something was produced": a
/// build whose grouping key was wrong would emit one box per **glyph**, and
/// a page of text has far more glyphs than lines.
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

/// ★ **The boxes exist in both spaces, index for index** — module header
/// §5.1.
///
/// The property the text-markup kinds rest on: the wash the operator sees
/// and the `/QuadPoints` written into the file are the same boxes, so a
/// build where one vector was filtered and the other was not would mark
/// glyphs it never highlighted. Asserted as an equal length **and** as a
/// per-entry correspondence of *width* — a length check alone would pass on
/// a build that pushed the right number of wrong quads.
///
/// The width comparison is deliberately loose about units: canvas space is
/// scaled by nothing here (it is page points, Y-down) so on an upright page
/// the two widths are equal, and the assertion is written as "both
/// non-degenerate and within a point" rather than as equality, because a
/// rotated fixture would legitimately swap the axes.
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

/// ★ **An edit makes a selection stale, and a stale selection paints
/// nothing** — module header §7.
///
/// Both halves, because the second is the one rule 4 turns on: a stored quad
/// after an edit may be over different glyphs, and drawing it anyway is the
/// thing `crate::find`'s staleness section calls out as forbidden outright.
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

/// ★ **A double-click selects a word, and a triple-click selects at least as
/// much.**
///
/// The two emphatic gestures, asserted *against each other* rather than
/// separately: a build where triple-click fell through to the word case
/// would pass two independent "selects something" tests and fail this one.
/// A word is also asserted to contain no whitespace, which is what
/// distinguishes it from a line on any page whose lines have more than one
/// word — and the test says so rather than assuming it.
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

/// ★ **A plain click clears** — Acrobat, Inkscape and SolidWorks alike.
///
/// Expressed as `None` rather than as an empty selection, which is the
/// invariant `TextSelection`'s own docs rest on: the field on the document
/// is a two-state question.
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

/// ★ **A drag selects the range between its ends, and it is
/// direction-blind.**
///
/// Dragging right-to-left must select exactly what dragging left-to-right
/// selected — the case a naive implementation gets wrong by assuming the
/// press is the earlier position, and the same class of error
/// `GestureOutcome::Markup`'s docs record for a normalised rect.
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

        // A quarter of the way across the line, not one canvas unit: a
        // one-unit sweep can begin and end inside the same glyph, which
        // resolves both ends onto the *same* caret boundary and therefore
        // covers nothing. That is correct behaviour and a useless fixture —
        // and it is what the first draft of this test did.
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
///
/// ★ Compared against **`plain_text()`**, not `sourced_text()`, and the
/// difference is a lesson worth keeping: the first draft of this test split
/// `sourced_text()` on whitespace and looked for the words in the copy, and
/// it failed with `select-all dropped "OneChapter"`. `sourced_text()`
/// deliberately omits every derived space and line break — it is the honest
/// lower bound on *what the file provides* — so on this fixture it runs
/// `Page One` and `Chapter 1` together into a token that exists in no
/// selection anyone could make.
///
/// That is exactly the distinction a copy has to get right in the other
/// direction: [`resolve`] walks the **runs**, derived-whitespace runs
/// included, which is what makes a copied paragraph paste as a paragraph.
/// Asserting against `plain_text()` is asserting against the same
/// segmentation the operator can see on the page.
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
///
/// Asserted at a point far outside the page box, because
/// `EditableTextModel::hit_test` deliberately falls back to the *nearest*
/// line rather than answering `None` — so the clearing has to come from the
/// range covering no glyphs, and a build that had "nearest line" leak into a
/// selection would fail here rather than in front of an operator.
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
