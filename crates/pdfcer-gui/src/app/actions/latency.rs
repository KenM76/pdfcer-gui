//! # `app::actions::latency` — **which half of an edit is the one you can feel?**
//!
//! An instrument, not a feature. It exists to answer one question, and the
//! whole design of `OPERATOR_REQUESTS.md` **O63** turns on the answer.
//!
//! ## The question
//!
//! **Ken, 2026-08-30:** *"we need to make it so we have a live preview as we
//! drag and move and resize and rotate … the live preview should remain while
//! the update to the pdf structure runs in the background"*, clarified minutes
//! later to *"live preview request is for everything we do."*
//!
//! He is describing a delay he can feel between acting and seeing. An edit that
//! reaches the screen is two things in series:
//!
//! | half | what it is | crate |
//! |---|---|---|
//! | **(a) the commit** | `EditSession`'s verb — locate, plan, rewrite the content stream, stage the objects | `pdfcer-core` |
//! | **(b) the raster** | redrawing the page at the current zoom because the edit epoch moved | `pdfcer-render` |
//!
//! **They need completely different fixes.** If (a) dominates, his description
//! is right as written and the shell needs an optimistic edit model with a queue
//! and backpressure — the screen showing a document state the engine has not
//! reached yet, with everything that implies about a refusal arriving for an
//! edit already shown. If (b) dominates, the fix is far smaller and carries no
//! such risk: keep displaying the last good frame while the next renders behind
//! it. The document is never ahead of the screen, because the edit really did
//! happen before the frame was asked for.
//!
//! ## ★★★ Why this is a `#[test]` and not a reasoned paragraph
//!
//! `BENCHMARK.md` exists because an earlier session asserted a performance
//! weakness **from architecture** and was wrong — it declared pdfcer's whole-page
//! raster a weakness needing a tile cache, and the operator's contrary report
//! ("it feels faster and more pleasant than the tiled competitor") turned out to
//! be correct on measurement.
//!
//! The same trap is open here, and the prior is strong enough to be dangerous: a
//! 129,758-object CAD sheet takes roughly a second to rasterise at scale 1, so
//! *"it is obviously the raster"* is the comfortable answer. Comfortable answers
//! reasoned from architecture are exactly what this file exists to refuse.
//!
//! ## Running it
//!
//! ```text
//! cargo test -p pdfcer-gui --release edit_latency -- --ignored --nocapture
//! ```
//!
//! `#[ignore]` because it wants a multi-megabyte drawing that is not in either
//! fixture corpus, and `--release` because a debug-build measurement of a
//! release-build question is not a measurement of anything. It **skips with a
//! printed reason** rather than failing when the drawing is absent: a machine
//! without it has not found a defect.
//!
//! ## What it deliberately does NOT measure
//!
//! The raster. `pdfcer-render`'s cost is already measured, already traced by the
//! running program (`render-inline ms=`, `render-async-done ms=`) and already
//! written up in `BENCHMARK.md`. Measuring it again here would produce a second
//! number for one fact, and two numbers for one fact drift. This file measures
//! **only the commit**, because the commit is the half nobody has a number for.

#![cfg(test)]

use std::path::PathBuf;
use std::time::Instant;

/// The dense CAD drawing `BENCHMARK.md` is written about.
///
/// ★ Not in `fixtures/` and not in the engine's corpus — it is 5.6 MB of the
/// operator's real work, and it lives outside both repositories. Named by
/// absolute path here rather than copied in, because a benchmark corpus that
/// grows by copying is a repository that grows without bound.
///
/// ★★ The path in this project's own role documentation was `D:\Dev\temp\pdfcer\`
/// and is **wrong** as of 2026-08-30 — the file is at `D:\Dev\pdfTests\`. Found
/// by looking, which is the reason this constant exists instead of a sentence in
/// a document: a path in prose goes stale silently, and a path in a test that
/// skips with its own reason tells the next reader where it looked.
const DRAWING: &str = r"D:\Dev\pdfTests\ncored-benchmark-cad-drawing.pdf";

/// A smaller, ordinary page, for the contrast that makes the big number mean
/// something.
///
/// A single measurement on a hard page cannot distinguish *"this verb is slow"*
/// from *"this page is hard"*, and the two have different fixes.
fn ordinary() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/a1-titleblock.pdf")
}

/// Time one closure, repeated, and report the median in milliseconds.
///
/// ★ The **median**, not the mean and not the best. The mean is dragged by a
/// single scheduler hiccup on a machine that is also running an editor and a
/// browser; the best-of is a number the operator will never experience. The
/// median is what a press feels like.
fn median_ms(runs: usize, mut body: impl FnMut()) -> f64 {
    let mut samples: Vec<f64> = Vec::with_capacity(runs);
    for _ in 0..runs {
        let started = Instant::now();
        body();
        samples.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    samples.sort_by(f64::total_cmp);
    samples[samples.len() / 2]
}

/// Load a document, or print why it could not and return `None`.
fn load(path: &std::path::Path) -> Option<pdfcer_core::document::Document> {
    if !path.is_file() {
        println!(
            "SKIPPED: {} is not on this machine, so nothing was measured. That is not a \
             failure — this test needs a drawing that lives outside both repositories.",
            path.display()
        );
        return None;
    }
    match pdfcer_core::document::Document::load(path) {
        Ok(doc) => Some(doc),
        Err(e) => {
            println!("SKIPPED: {} would not load: {e}", path.display());
            None
        }
    }
}

/// **How long does the ENGINE take to accept one edit?**
///
/// Prints a table. Asserts nothing about the numbers, and that is deliberate:
/// a threshold asserted here would be a number nobody chose, on hardware nobody
/// specified, and the first slow machine would turn a measurement into a red
/// suite. What it asserts is that the measurement **happened** — a run that
/// silently measured nothing is the failure this whole harness exists to remove.
#[test]
#[ignore = "needs the 5.6 MB CAD drawing; run with --ignored --nocapture --release"]
fn edit_latency_the_commit_half() {
    let mut measured = 0_usize;

    for (label, path) in [
        ("ordinary A1 title block", ordinary()),
        ("dense CAD site plan", PathBuf::from(DRAWING)),
    ] {
        let Some(doc) = load(&path) else { continue };

        // ★ The load itself, for scale. It is not part of an edit's latency —
        // it happens once when the document opens — but without it the reader
        // cannot tell a slow verb from a slow file.
        let load_ms = median_ms(3, || {
            let _ = pdfcer_core::document::Document::load(&path);
        });

        // The cheapest possible session: no edit, just the construction the
        // shell does on open. If THIS is slow, every number below inherits it.
        let session_ms = median_ms(5, || {
            let _ = pdfcer_core::edit::EditSession::new(
                pdfcer_core::document::Document::load(&path).expect("it loaded a moment ago"),
            );
        });

        // ★★★ The number the whole question turns on: a read-only query of the
        // page the operator is looking at.
        //
        // `EditSession::view()` is what every panel, every properties field and
        // every selection hit-test goes through, and it is the closest thing to
        // "the shell asking the engine what is on the page" that can be timed
        // without choosing one particular verb and generalising from it.
        let doc_for_view =
            pdfcer_core::document::Document::load(&path).expect("it loaded a moment ago");
        let session = pdfcer_core::edit::EditSession::new(doc_for_view);
        let view_ms = median_ms(5, || {
            let _ = session.view();
        });

        // ★★ Decomposition — the read every canvas gesture starts with.
        //
        // `page_objects` is the shell's cache over `decompose_page`, keyed on
        // `(page, edit_epoch)`, and **an edit invalidates it**. So this cost is
        // paid again after every commit, before the operator can grab anything.
        // It is part of the felt latency even though nothing draws it.
        let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
        let decompose_ms = median_ms(3, || {
            let view = session.view();
            let _ = pdfcer_core::vector::decompose_page(
                &view,
                &pages[0],
                pdfcer_core::vector::Matrix::IDENTITY,
            );
        });

        // ★★★ THE NUMBER THE WHOLE QUESTION TURNS ON: one real edit.
        //
        // `move_objects` is the verb behind a drag-move, and it is the cheapest
        // realistic commit — it rewrites operands in place rather than
        // restructuring anything. If THIS is slow, every edit is slow and his
        // description of the fix is right as written. If it is fast, the delay
        // he can feel is downstream of the engine and lives in this shell.
        let model = {
            let view = session.view();
            pdfcer_core::vector::decompose_page(
                &view,
                &pages[0],
                pdfcer_core::vector::Matrix::IDENTITY,
            )
        };
        let subject: Vec<usize> = match &model {
            Ok(m) => (0..m.objects.len().min(1)).collect(),
            Err(_) => Vec::new(),
        };
        let move_ms = if subject.is_empty() {
            f64::NAN
        } else {
            median_ms(3, || {
                // A fresh session per sample: the second move on one session
                // measures a session that already holds staged content, which
                // is a different and easier question than the first move does.
                let doc = pdfcer_core::document::Document::load(&path).expect("it loaded");
                let mut session = pdfcer_core::edit::EditSession::new(doc);
                let _ = session.move_objects(0, &subject, 1.0, 1.0);
            })
        };

        println!("\n=== {label} ===");
        println!("  {:<28} {:>9.1} ms", "Document::load", load_ms);
        println!(
            "  {:<28} {:>9.1} ms",
            "EditSession::new (+load)", session_ms
        );
        println!("  {:<28} {:>9.3} ms", "EditSession::view", view_ms);
        println!("  {:<28} {:>9.1} ms", "decompose_page", decompose_ms);
        println!(
            "  {:<28} {:>9.1} ms",
            "move_objects (+load+session)", move_ms
        );
        println!("  (pages: {})", doc_page_count(&doc));
        measured += 1;
    }

    assert!(
        measured > 0,
        "nothing was measured at all — both documents were missing, so this run told you \
         NOTHING about the program. A test that can only report one outcome cannot detect the \
         thing it was added to detect."
    );
}

/// How many pages, for the header line.
fn doc_page_count(doc: &pdfcer_core::document::Document) -> usize {
    pdfcer_core::page_tree::pages(doc).map_or(0, |p| p.len())
}
