//! Design and rationale: `docs/modules/pdfcer-gui/render/hairline.md`.
#![cfg(test)]
//! # `render::hairline` — **the proof that "line weights off" actually thins the
//! drawing, and thins it in the RIGHT DIRECTION**
//!
//!
//! > *"awhile ago you told me you removed the button to show all lines without
//! > their thickness — thin lines or something like cad has. The button never
//! > worked but I do want that display option!"*
//!
//! A whole file, `#![cfg(test)]`, on the precedent of `canvas::textedit::proof`
//! and `canvas::textedit::cost`: it compiles to nothing in a release build, and
//! `render::worker` sits at 1,474 of R2's 1,500 lines with no room for a
//! measurement this size.
//!
//! ## Why this exists when four other tests already cover the feature
//!
//! Because all four of them are about **wiring**, and wiring can be perfect
//! over a renderer that ignores the field.
//!
//! | test | proves |
//! |---|---|
//! | `settings::only_the_canvas_worker_sets_stroke_display` | the option cannot leak into an export |
//! | `settings::every_export_path_renders_real_widths_with_line_weights_off` | the request carries it and the funnel does not |
//! | `worker::the_render_key_moves_when_line_weights_are_turned_off` | the cache cannot serve a stale picture |
//! | `app::actions::chrome`'s pair | the toggle reaches the view state |
//!
//! Every one of those passes on a build where `pdfcer-render` silently drops
//! `stroke_display` — where the enum is threaded end to end, three rasters are
//! spawned, the cache is correctly invalidated, and **the picture is identical**.
//! That is exactly the operator's complaint about the control this replaces:
//! *"the button never worked."* So this file measures the PIXELS.
//!
//! ## It also falsifies the OPPOSITE convention, which is the real risk
//!
//! The two display modes routinely confused with each other are opposites:
//!
//! | | | precedent |
//! |---|---|---|
//! | **line weights OFF ← what he asked for** | every stroke capped at one device pixel | AutoCAD `LWDISPLAY` off |
//! | enhance thin lines | sub-pixel strokes bumped **up** to one pixel | Acrobat's preference of that name |
//!
//! **One makes thick things thin; the other makes thin things thick.** A test
//! that asserted only *"the picture changed"* would pass on a build that
//! shipped the wrong one — and shipping the wrong one is worse than shipping
//! nothing, because it looks like the feature working while doing the reverse.
//!
//! ⇒ So the assertion is **signed**: strictly LESS ink with the mode on. Ink is
//! counted as dark pixels, which is what a stroke deposits and what "adjacent
//! geometry merges into one bar" means when he complains about it.
//!
//! ## The fixture, and why it is pinned
//!
//! `fixtures/a1-titleblock.pdf`, always, ignoring any `--pdf` idea of a
//! subject, for the reason `three_clicks_round_a_hole_measure_the_hole` pins
//! its own: **on a page whose strokes are already sub-pixel the defect cannot
//! occur**, so an arbitrary fixture would make the check unable to fail.
//!
//! Measured before it was chosen: its three content streams carry 156 `B`,
//! 77 `s` and 30 `S` operators and set **no** `w` at all, so every stroke is at
//! the PDF default of 1.0 user-space unit (§8.4.3.2's initial value). That is
//! the honest CAD case — a drawing whose weights are the producer's defaults —
//! and it is *harmless at page fit and fat when you zoom in*, which is the
//! whole of what he reported.
//!
//! ## THE SCALE IS THE PRECONDITION, and getting it wrong makes the test
//! vacuous rather than red
//!
//! At scale 1.0 a 1.0-unit stroke is one device pixel, the engine's §8.4.3.2
//! floor already holds it there, and `Hairline`'s ceiling has **nothing to
//! cap** — the two renders come out identical and the test would fail against a
//! perfectly good build. So it renders at [`SCALE`] = 4.0, which is 400 %:
//! the zoom he named as where he actually reads a title block, and where a
//! 1.0-unit stroke is 4 device pixels of solid black.
//!
//! [`the_two_modes_are_identical_where_there_is_nothing_to_cap`] pins the other
//! end of that, so the number is a measured boundary rather than a lucky
//! constant.

use pdfcer_core::document::Document;
use pdfcer_render::font::StrokeDisplay;

/// Device pixels per PDF user-space unit — 400 %.
///
/// See the module header: at 1.0 the fixture's default-width strokes are
/// already one pixel and there is nothing for the ceiling to do.
const SCALE: f32 = 4.0;

/// A pixel this dark or darker counts as ink.
const INK: u8 = 128;

/// Render the pinned fixture's first page under one stroke-display convention
/// and return `(ink pixels, total pixels)`.
///
/// Built through `RenderOptions::default()` **plus one assignment**, which is
/// deliberately the same two lines `render::worker::render_on_worker` executes
/// — a test that constructed its options some other way would be measuring a
/// third code path. (This file is `#![cfg(test)]`, so
/// `app::settings::tests::no_call_site_builds_its_own_options` exempts it by
/// the rule written on its own scan: what earns the exemption is "not in the
/// shipped binary".)
fn ink_at(display: StrokeDisplay) -> (u64, u64) {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/a1-titleblock.pdf");
    assert!(
        path.exists(),
        "the pinned fixture is missing at {} — this measurement cannot be taken on another \
         document, because a page whose strokes are already hairline cannot exhibit the effect",
        path.display()
    );
    let doc = Document::load(&path).expect("the fixture loads");
    let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
    let session = pdfcer_core::edit::EditSession::new(doc);

    let mut options = pdfcer_render::RenderOptions::default();
    options.stroke_display = display;

    let view = session.view();
    let rendered = pdfcer_render::render_page_with_view(&view, &pages[0], SCALE, &options)
        .expect("the page rasterizes");

    let pixmap = rendered.pixmap;
    let total = u64::from(pixmap.width()) * u64::from(pixmap.height());
    let ink = pixmap
        .pixels()
        .iter()
        .filter(|p| {
            // Demultiplied is unnecessary here: the page is rendered on an
            // opaque white backdrop, so alpha is 255 everywhere and the
            // premultiplied channels are the channels.
            p.red() <= INK && p.green() <= INK && p.blue() <= INK
        })
        .count() as u64;
    (ink, total)
}

/// How many strokes the renderer THINNED at `display`, on the pinned fixture.
fn hairlined_at(display: StrokeDisplay) -> usize {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/a1-titleblock.pdf");
    let doc = Document::load(&path).expect("the fixture loads");
    let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
    let session = pdfcer_core::edit::EditSession::new(doc);

    let mut options = pdfcer_render::RenderOptions::default();
    options.stroke_display = display;

    pdfcer_render::render_page_with_view(&session.view(), &pages[0], SCALE, &options)
        .expect("the page rasterizes")
        .diagnostics
        .strokes_hairlined
}

/// **Turning line weights off puts strictly LESS ink on the page** — the
/// one assertion in the whole feature that a build with perfect plumbing and an
/// indifferent renderer cannot satisfy.
#[test]
fn line_weights_off_puts_less_ink_on_a_real_drawing() {
    let (actual, total) = ink_at(StrokeDisplay::Actual);
    let (hairline, _) = ink_at(StrokeDisplay::Hairline);

    assert!(
        actual > 0,
        "the fixture rendered no ink at all at scale {SCALE}, so this measurement is about \
         nothing. Check that a1-titleblock.pdf still draws."
    );
    assert!(
        hairline < actual,
        "LINE WEIGHTS OFF DID NOT THIN THE DRAWING. At scale {SCALE} the page has {actual} ink \
         pixels of {total} with real widths and {hairline} with `StrokeDisplay::Hairline`. \
         Equal means the engine ignored the field and the control is inert — the operator's \
         exact complaint about the button this replaces. MORE means the OPPOSITE convention \
         shipped (Acrobat's \"enhance thin lines\", which makes thin things thicker); he asked \
         for AutoCAD's `LWDISPLAY` off, which makes thick things thinner."
    );

    let removed = (actual - hairline) as f64 / actual as f64;
    assert!(
        removed >= 0.05,
        "line weights off removed only {:.1} % of the ink ({actual} -> {hairline} pixels). \
         Measured on 2026-09-05 at scale {SCALE} it removes 17.7 % (484,078 -> 398,578), and \
         the rest of the page's ink is text and fills, which this mode correctly does not \
         touch. A figure this small suggests the ceiling is reaching only some strokes.",
        removed * 100.0
    );
}

/// **Where there is nothing to cap, the two modes are the same picture** —
/// the boundary that keeps the test above from being satisfiable by any change
/// at all.
#[test]
fn the_two_modes_are_identical_where_there_is_nothing_to_cap() {
    // Scale 1.0 renders are taken by calling the helper's body at a different
    // scale, so the constant is not silently shared with the test above.
    let at = |display: StrokeDisplay| -> u64 {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/a1-titleblock.pdf");
        let doc = Document::load(&path).expect("the fixture loads");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
        let session = pdfcer_core::edit::EditSession::new(doc);
        let mut options = pdfcer_render::RenderOptions::default();
        options.stroke_display = display;
        let view = session.view();
        let rendered = pdfcer_render::render_page_with_view(&view, &pages[0], 1.0, &options)
            .expect("the page rasterizes");
        rendered
            .pixmap
            .pixels()
            .iter()
            .filter(|p| p.red() <= INK && p.green() <= INK && p.blue() <= INK)
            .count() as u64
    };

    assert_eq!(
        at(StrokeDisplay::Actual),
        at(StrokeDisplay::Hairline),
        "at scale 1.0 every stroke on this fixture is already one device pixel, so a CEILING at \
         one device pixel must change nothing. A difference here means the mode is SETTING the \
         width rather than capping it — which on a drawing that already uses hairlines would \
         make them thicker, i.e. the opposite convention"
    );
}

/// **BOTH SENTENCES ARE REACHABLE, and the zero one is not hypothetical.**
#[test]
fn the_hairline_counter_counts_what_was_thinned_and_not_what_was_drawn() {
    let thinned = hairlined_at(StrokeDisplay::Hairline);
    let untouched = hairlined_at(StrokeDisplay::Actual);

    assert!(
        thinned > 0,
        "line weights OFF thinned {thinned} strokes on a real CAD sheet. Zero means the \
         disclosure would say `line_weights_no_effect` on a drawing full of linework — which is \
         the sentence's exact opposite — so either `stroke_display` is not reaching the \
         renderer, or `strokes_hairlined` is not being incremented."
    );
    assert_eq!(
        untouched, 0,
        "line weights ON thinned {untouched} strokes, and it must thin none: with \
         `StrokeDisplay::Actual` every width is faithful. A non-zero here means the counter is \
         reporting strokes DRAWN rather than strokes THINNED — in which case \
         `line_weights_no_effect` could never appear, and the operator loses the one sentence \
         that tells `nothing here was thick enough` from `the setting is broken`."
    );
}
