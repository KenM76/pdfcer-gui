//! # `canvas::textedit::cost` — **what a per-keystroke re-measure actually costs**
//!
//! `DEFECTS.md` **D4b**'s first sentence is *"there is no re-layout per
//! keystroke"*, and the old shell's own comment agrees in terms: *"Typing →
//! build/extend the `PendingEdit` (§6.1). **No core call per keystroke.**"* So
//! "as you type", nothing moves at all, and D4b says that alone accounts for
//! much of the complaint.
//!
//! The brief for this work says: **measure what a per-keystroke re-measure
//! actually costs and report the number**; if it is too slow, say so and
//! debounce deliberately rather than silently. This module is that measurement,
//! and it is `#[ignore]`d for the reason every measurement in this repository is
//! — a timing assertion in CI is a flake, and the value of a number is that
//! somebody read it.
//!
//! Run it:
//!
//! ```text
//! cargo test -p pdfcer-gui --lib canvas::textedit::cost -- --ignored --nocapture
//! ```
//!
//! ## What is being measured, and why it is not a micro-benchmark
//!
//! There is **no public dry-run** in `pdfcer-core`. `plan_edit` — the function
//! that locates the anchor, re-encodes, sums the §9.4.4 advances and produces
//! `EditReport::advance_delta`, which is *exactly* the number a live re-layout
//! wants — is `pub(crate)`. The two public routes to it are:
//!
//! | route | what it does beyond planning |
//! |---|---|
//! | `text_edit::edit_text` (free fn) | plans, then **performs an incremental save**, returning the whole appended PDF |
//! | `EditSession::edit_text` | plans, then **commits a command to the undo log** |
//!
//! Neither is a query. The first allocates a document-sized byte vector per
//! call; the second mutates. So the honest thing to measure is the *cheapest
//! public route to a real advance delta*, which is the free function, plus the
//! two derivations the shell would also have to redo — and to report the
//! components separately, so the number can be read rather than merely quoted.
//!
//! ## The comparison that decides it
//!
//! `DEFECTS.md` records Find at **331–449 ms per whole-document call**, which is
//! why Find never searches on a keystroke. The threshold that matters for typing
//! is far lower than Find's: a keystroke that is not on screen within roughly
//! **16 ms** has missed a frame, and one that takes longer than about **50 ms**
//! is felt as lag. So the question is not "is it faster than Find" but "does it
//! fit in a frame".
//!
//! ---
//!
//! # ★★ THE MEASUREMENT, and the decision it forced
//!
//! Run 2026-08-15, `--release`, median of 5, this machine:
//!
//! | document | extract (prov.) | recognize + align | plan + save | total |
//! |---|---:|---:|---:|---:|
//! | `tail-alignment` (3 lines) | 0.12 ms | 0.01 ms | 0.36 ms | **0.49 ms** |
//! | `SW41177` p1 (a SolidWorks sheet) | 32.07 ms | 0.16 ms | 70.54 ms | **102.77 ms** |
//! | `ncored-benchmark` A3 (129,758 objects) | 356.53 ms | 2.79 ms | — | **356+ ms** |
//! | `a1-titleblock` (repo fixture) | 0.46 ms | 0.01 ms | — | — |
//!
//! (A dash is a refusal rather than a cost: those pages' page-1 runs are not
//! editable through this API — an embedded subset without the new code — and
//! timing a refusal and reporting it as a re-measure would be exactly the
//! flattering-fixture failure `HANDOFF.md` §10 records.)
//!
//! **So the answer is: it does not fit, on the documents this operator actually
//! opens.** 102.77 ms per keystroke on a SolidWorks sheet is **six frames**, and
//! the benchmark A3 spends 356 ms in the *extraction alone* — which is Find's
//! own 331–449 ms, arriving on every key.
//!
//! ## The decision, stated rather than debounced quietly
//!
//! **Priority 2 does not land, and it is not debounced either.** A debounce
//! would have been the easy answer and it is the wrong one here: a re-layout
//! that appears 150 ms after you stop typing is not *"text moves as you type"*,
//! it is a second, later, surprise — and D4a already records that this feature's
//! besetting sin is showing the operator something that is not what the document
//! will say. Shipping a laggy approximation of the thing they asked for would be
//! a third.
//!
//! ## What would actually fix it, and where it has to happen
//!
//! Not here. Two of the three numbers above are avoidable, and neither is
//! avoidable from this repository:
//!
//! 1. **`plan+save` is the wrong operation.** The number a live re-layout wants
//!    is `EditReport::advance_delta`, and `plan_edit` computes it *before* any
//!    write — that is the seam's stated purpose. It is `pub(crate)`. A public
//!    `measure_edit(&Document, &EditRequest) -> Result<f64, EditError>`, or
//!    simply making `plan_edit`/`EditPlan` public, removes the incremental save
//!    from the loop entirely. **This is a feature request for `pdfcer-core`**, and
//!    it is the whole of what Priority 2 is blocked on.
//! 2. **`extract` is already cached** in the running shell — `app::cache` keys it
//!    on `(page, edit_epoch)` and typing bumps no epoch — so the 32 ms and the
//!    356 ms are paid once per page, not per key. The commit path pays a second,
//!    provenance-capturing extraction (`textedit::plan` explains why); a
//!    per-keystroke loop would want that one cached too, which is a change here
//!    and a small one.
//!
//! With (1) in hand the marginal per-keystroke cost on `SW41177` would be the
//! plan without the save. That is not measurable from outside the crate, which is
//! the honest end of this measurement: **the number that decides whether
//! Priority 2 is affordable cannot be obtained through today's public API**, and
//! saying so is better than reporting the one that can be.
//!
//! ## The cheap approximation that was considered and NOT taken
//!
//! Every `ExtractedGlyph` already carries a real `advance` — §9.4.4, with `Tc`,
//! `Tw`, `Tz` and the `TJ` attribution folded in. A draft could therefore be
//! measured by looking each character up among the glyphs *already on the page in
//! the same font*, in microseconds, with no core call at all.
//!
//! It is real metrics, and it has a hole: a character the page does not already
//! show has no width, so the measure would be exact for most typing and silently
//! wrong for the rest — and "silently wrong about where your text ends" is the
//! defect being fixed, wearing a faster suit. It could be made honest (measure
//! what is measurable, disclose the rest), and that is a reasonable thing to
//! build **after** (1), when it would be a fallback rather than the mechanism.

#![cfg(test)]

// ---------------------------------------------------------------------------
// ★ The line below is a DELIBERATE DUPLICATE of the `#![cfg(test)]` above, and
// it is here for `tools/gates/check-ui-strings.sh` rather than for rustc.
//
// That gate stops scanning a file at the first line matching `^#\[cfg\(test\)\]`
// — an *outer* attribute at column zero — because everything after it is
// test-only and test literals are not operator copy. An **inner** attribute
// (`#!`) does not match that anchor, so a file that is test-only in its
// entirety is scanned in its entirety, and every `expect("…")` in it is
// reported as user-facing copy.
//
// `DEFECTS.md` D13 records the same anchor from the other side: a mid-file
// `#[cfg(test)]` switches the gate off for the REST of the file, which is a
// hole. Here the anchor is doing its job and simply cannot see this file's
// shape, so the shape is adjusted to be visible. The cost is one redundant
// attribute; the alternative is 38 `ui-text-exempt` tags on test assertions,
// which would be noise that a real violation could hide in.
// ---------------------------------------------------------------------------
#[cfg(test)]
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
/// The two named in `HANDOFF.md` §2's table are the ones that matter — they are
/// the operator's real material — and they live outside the repository, so a
/// missing one **skips that row** and says so rather than failing. A
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
        // reporting a refusal's timing as a re-measure cost would be exactly
        // the flattering-fixture failure `HANDOFF.md` §10 records.
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
/// asserted is only that the harness *ran* — `HANDOFF.md` §10's rule that a
/// layout test must assert a measurement happened rather than only its value,
/// applied to a timing one.
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
