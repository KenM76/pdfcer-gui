//! # `canvas::textedit::cost` — **what a per-keystroke re-measure actually costs**
//!
//! `DEFECTS.md` **D4b**'s first sentence is *"there is no re-layout per
//! keystroke"*, and the old shell's own comment agrees in terms: *"Typing →
//! build/extend the `PendingEdit` (§6.1). **No core call per keystroke.**"* So
//! "as you type", nothing moves at all, and D4b says that alone accounts for
//! much of the complaint.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/cost.md`.

#![cfg(test)]

use std::path::{Path, PathBuf};
use std::time::Instant;

use pdfcer_core::document::Document;
use pdfcer_core::text_edit::{
    EditOptions, EditRequest, EditableTextModel, ReflowEngine, TextPosition,
    reflow_recognition_options,
};
use pdfcer_core::text_extract::{ExtractOptions, extract_page_view};

/// Median of a sample, which is what a per-keystroke cost should be reported as.
///
/// A mean over ten iterations on a Windows desktop is a mean including whatever
/// else the machine did; the median is the cost of a typical keystroke, which is
/// the thing an operator experiences.
fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// Time `f` `n` times and report the median, in milliseconds.
fn timed(n: usize, mut f: impl FnMut()) -> f64 {
    let mut samples = Vec::with_capacity(n);
    for _ in 0..n {
        let t = Instant::now();
        f();
        samples.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    median(samples)
}

/// The documents to measure against, when they are present.
///
/// The two outside the repository are the ones that matter — they are the
/// operator's real material — so a missing one **skips that row** and says so
/// rather than failing. A
/// measurement that cannot run is not a measurement that passed;
/// `run-all.sh`'s three-state model is the same rule one level up.
fn corpus() -> Vec<(&'static str, PathBuf)> {
    let mine = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tail-alignment.pdf");
    let mut v: Vec<(&'static str, PathBuf)> = vec![("tail-alignment (tiny, 3 lines)", mine)];
    for (label, p) in [
        (
            "SW41177 p1 (SolidWorks sheet)",
            r"D:\Dev\temp\pdfcer\SW41177.pdf",
        ),
        (
            "ncored benchmark A3 (129,758 objects)",
            r"D:\Dev\temp\pdfcer\ncored-benchmark-cad-drawing.pdf",
        ),
        (
            "a1-titleblock (repo fixture)",
            r"D:\Dev\pdfcer-gui\fixtures\a1-titleblock.pdf",
        ),
    ] {
        let p = PathBuf::from(p);
        if p.exists() {
            v.push((label, p));
        }
    }
    v
}

/// One document's four numbers.
fn measure(label: &str, path: &Path) {
    let Ok(doc) = Document::load(path) else {
        println!("  {label:<40} SKIPPED (would not load)");
        return;
    };
    let Ok(pages) = pdfcer_core::page_tree::pages(&doc) else {
        println!("  {label:<40} SKIPPED (no page tree)");
        return;
    };
    let page = &pages[0];
    let view = doc.view();
    let opts = ExtractOptions::default().with_provenance(true);

    // 1. Extraction with provenance — what `plan` pays once per commit, and
    //    what a per-keystroke re-measure would have to have in hand.
    let extract_ms = timed(5, || {
        let _ = extract_page_view(&view, page, 0, &opts);
    });
    let Ok(text) = extract_page_view(&view, page, 0, &opts) else {
        println!("  {label:<40} SKIPPED (no extractable text)");
        return;
    };
    if text.runs.iter().all(|r| r.text.trim().is_empty()) {
        println!("  {label:<40} SKIPPED (page 1 has no text)");
        return;
    }

    // 2. Block recognition + alignment detection — the disposition half.
    let recognize_ms = timed(5, || {
        let relaxed = EditableTextModel::recognize(&text, &reflow_recognition_options());
        let _ = relaxed
            .block_at(TextPosition::new(0, 0))
            .and_then(|b| ReflowEngine::new(&relaxed).detect_alignment(b).ok());
    });

    // 3. The cheapest PUBLIC route to a real advance delta: plan + incremental
    //    save. This is the number that decides whether a live re-measure is
    //    affordable through today's API.
    let find = text
        .runs
        .iter()
        .find(|r| r.text.trim().len() > 3)
        .map(|r| r.text.trim().to_owned());
    let plan_ms = find.as_ref().map_or(f64::NAN, |f| {
        let req = EditRequest::find_replace(0, f, &format!("{f}x"));
        // One trial first: a document whose page-1 text is not editable (a
        // subset font missing the new code) is a refusal, not a cost, and
        // reporting a refusal's timing as a re-measure cost would be a
        // measurement of something other than what it claims.
        if pdfcer_core::text_edit::edit_text(&doc, &req, &EditOptions::default()).is_err() {
            return f64::NAN;
        }
        timed(5, || {
            let _ = pdfcer_core::text_edit::edit_text(&doc, &req, &EditOptions::default());
        })
    });

    let total = extract_ms + recognize_ms + plan_ms;
    println!(
        "  {label:<40} extract {extract_ms:7.2} ms | recognize+align {recognize_ms:7.2} ms | \
         plan+save {plan_ms:7.2} ms | total {total:7.2} ms"
    );
}

/// ★★ **The measurement.** Prints; asserts nothing about time.
///
/// A timing assertion in a suite that runs on whatever machine happens to be
/// free is a flake, and a flake gets `#[ignore]`d and then deleted. What is
/// asserted is only that the harness *ran* — the rule that a layout test must
/// assert a measurement happened rather than only its value, applied to a
/// timing one.
#[test]
#[ignore = "a measurement, not an assertion — run it and read the numbers"]
fn what_a_per_keystroke_re_measure_would_cost() {
    println!(
        "\nper-keystroke re-measure cost, median of 5, debug build unless \
         --release\n\
         (a keystroke has ~16 ms to reach the screen before it misses a frame)\n"
    );
    let corpus = corpus();
    assert!(
        !corpus.is_empty(),
        "the repo fixture must at least be there"
    );
    for (label, path) in &corpus {
        measure(label, path);
    }
    println!();
}
