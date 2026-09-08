//! **A probe, not a test** — measuring the operator's 2026-09-08 report against
//! his own file, headlessly.
//!
//! > *"In `SW41177.pdf` I can't edit some of the text, for example `#2 USE
//! > SPACERS 8 9 10 11 IF REQUIRED.` The BOM I can, but it only sometimes works
//! > — in fact I thought it had also stopped working, but after trying multiple
//! > times in multiple places it started working."*
//!
//! # Why a `#[ignore]`d probe rather than a test
//!
//! It reads a file that is **not in this repository** — a copy of his own
//! drawing under `target/scratch/` — so it cannot run in CI and must never
//! fail a build. It exists to be run by hand, print measurements, and be
//! deleted or turned into a real test with a committed fixture once the cause
//! is known.
//!
//! ★ Written as a file rather than as a shell one-liner because the question is
//! *what does the engine say about each run*, and that needs the same planner
//! the GUI uses — not a byte grep, which is what produced two wrong diagnoses of
//! his last text-editing report.
//!
//! Run with:
//!
//! ```text
//! cargo test --test ken_sw41177_probe -- --ignored --nocapture
//! ```

use pdfcer_core::document::Document;
use pdfcer_core::edit::EditSession;

const FIXTURE: &str = "target/scratch/docs/SW41177-ken.pdf";

/// The exact string he named.
const HIS_TEXT: &str = "USE SPACERS";

/// The page's text through the session's own view, the way
/// `canvas::textedit::facewall` reads it — overlay included, not the file on
/// disk.
fn page_text(session: &EditSession, index: usize) -> Option<pdfcer_core::text_extract::PageText> {
    let view = session.view();
    let pages = pdfcer_core::page_tree::pages_in(&view).ok()?;
    pdfcer_core::text_extract::extract_page_view(
        &view,
        pages.get(index)?,
        index,
        &pdfcer_core::text_extract::ExtractOptions::default(),
    )
    .ok()
}

fn session() -> Option<EditSession> {
    let path = format!("{}/../../{FIXTURE}", env!("CARGO_MANIFEST_DIR"));
    let p = std::path::Path::new(&path);
    if !p.exists() {
        println!("SKIP: {path} is not present — copy his file there first");
        return None;
    }
    Some(EditSession::new(Document::load(p).expect("his file loads")))
}

/// Find every page carrying his phrase, and report what the extraction sees.
#[test]
#[ignore = "reads a file outside the repository; run by hand"]
fn where_is_his_text_and_what_does_the_engine_say() {
    let Some(session) = session() else { return };
    let pages = session.pages().expect("a page tree");
    println!("pages: {}", pages.len());

    for (i, _) in pages.iter().enumerate() {
        let Some(text) = page_text(&session, i) else {
            println!("page {i}: EXTRACTION FAILED");
            continue;
        };
        let whole: String = text.runs.iter().map(|r| r.text.as_str()).collect();
        if !whole.contains(HIS_TEXT) {
            continue;
        }
        println!("\n=== page {i} carries {HIS_TEXT:?} ===");
        println!("runs on this page: {}", text.runs.len());

        // ★ Which RUNS carry it, and how the phrase is split between them. This
        // is the measurement that separates "one run" from "several", and the
        // second is what `EditRefusal::SplitAcrossPieces` is about.
        let mut carriers = Vec::new();
        for (r, run) in text.runs.iter().enumerate() {
            if run.text.contains("SPACERS") || run.text.contains("USE ") {
                carriers.push((r, run.text.clone()));
            }
        }
        println!("runs whose own text mentions it: {}", carriers.len());
        for (r, t) in carriers.iter().take(12) {
            println!("  run {r}: {t:?}");
        }

        // ★★ And how many times the phrase occurs on the page. An edit located
        // by `find` alone is refused when the page carries the text more than
        // once — `EditRefusal::AmbiguousOnThePage` — and *"sometimes works"* on
        // a BOM is exactly what a repeated value looks like from his chair.
        let occurrences = whole.matches(HIS_TEXT).count();
        println!("occurrences of {HIS_TEXT:?} on page {i}: {occurrences}");
    }
}

/// What a BOM cell looks like: how often short values repeat on one page.
///
/// ★★★ This is the *"only sometimes works"* half of his report and the reason
/// it is measured separately. A bill of materials is a grid of short strings —
/// quantities, item numbers, part codes — and many of them repeat. An edit
/// located by text alone cannot tell two identical cells apart, so pdfcer
/// refuses by name rather than guessing which one he meant.
///
/// ⇒ If that is the cause, *"it worked after trying in multiple places"* is not
/// flakiness at all: it is him landing on a cell whose text happens to be
/// unique on that page.
#[test]
#[ignore = "reads a file outside the repository; run by hand"]
fn how_many_short_runs_repeat_on_the_busiest_page() {
    let Some(session) = session() else { return };
    let pages = session.pages().expect("a page tree");

    let mut worst = (0usize, 0usize);
    for (i, _) in pages.iter().enumerate() {
        let Some(text) = page_text(&session, i) else {
            continue;
        };
        if text.runs.len() > worst.1 {
            worst = (i, text.runs.len());
        }
    }
    let (page, runs) = worst;
    println!("busiest page is {page} with {runs} runs");

    let text = page_text(&session, page).expect("re-extract");
    let whole: String = text.runs.iter().map(|r| r.text.as_str()).collect();

    let mut repeated = 0usize;
    let mut samples = Vec::new();
    for run in &text.runs {
        let t = run.text.trim();
        if t.is_empty() {
            continue;
        }
        let n = whole.matches(t).count();
        if n > 1 {
            repeated += 1;
            if samples.len() < 10 {
                samples.push((t.to_owned(), n));
            }
        }
    }
    println!("runs whose text appears MORE THAN ONCE on that page: {repeated} of {runs}");
    for (t, n) in samples {
        println!("  {t:?} x{n}");
    }
}

/// ★★★ **Ask the engine to make his edit, and print exactly what it says.**
///
/// The two measurements above ruled out the two obvious causes for the line he
/// named: it is **one run**, and it occurs **once** on its page — so it is
/// neither split across pieces nor ambiguous. Whatever refuses it is something
/// else, and guessing is what produced two wrong diagnoses of his last report.
///
/// ⇒ So this performs the edit the GUI would perform and prints the engine's
/// own answer verbatim, plus the run's font, which is the next most likely
/// cause and the one `EditRefusal::UnsupportedFont` exists for.
#[test]
#[ignore = "reads a file outside the repository; run by hand"]
fn what_does_the_engine_say_when_his_edit_is_attempted() {
    let Some(mut session) = session() else { return };

    let text = page_text(&session, 0).expect("page 0 extracts");
    let Some((index, run)) = text
        .runs
        .iter()
        .enumerate()
        .find(|(_, r)| r.text.contains("USE SPACERS"))
    else {
        println!("his line is not on page 0 any more");
        return;
    };
    println!("run {index}: {:?}", run.text);
    println!("  glyphs        : {}", run.glyphs.len());
    println!("  mcid          : {:?}", run.mcid);
    println!("  artifact      : {:?}", run.artifact);

    // The edit he would make: correct one character.
    let req = pdfcer_core::text_edit::EditRequest::find_replace(0, "SPACERS", "SPACER");
    match session.edit_text(&req, &pdfcer_core::text_edit::EditOptions::default()) {
        Ok(report) => {
            println!("ACCEPTED — the engine made the edit");
            println!("  operators_spanned  : {}", report.operators_spanned);
            println!("  followers          : {}", report.followers_repositioned);
            for d in &report.disclosures {
                println!("  disclosure: {d}");
            }
        }
        Err(e) => {
            println!("REFUSED — {e}");
            use pdfcer_core::text_edit::RefusalClass as _;
            println!("  kind: {:?}", e.refusal_kind());
        }
    }
}

/// ★★★ **The shell's OWN request shape**, which is not the one above.
///
/// The plain `find_replace` probe is accepted, so the engine can make his edit.
/// But the GUI does not send that: `canvas::textedit::plan` builds a **pinned
/// whole-operator** request — `find` cleared, `pinned_span` set to the run the
/// operator clicked — because that is what makes a click mean *this* run rather
/// than the first occurrence of some text.
///
/// ⇒ So the question that matters is whether the engine accepts **that** shape
/// for his line. If it does, the refusal is further up in the shell; if it does
/// not, this is the defect and it is one the plain probe would never have found.
///
/// ★ The pin is measured from a **provenance-carrying** extraction, which is
/// the only kind that can produce one. An extraction without provenance yields
/// no pin and the shell falls back — a difference invisible in the text.
#[test]
#[ignore = "reads a file outside the repository; run by hand"]
fn what_happens_with_the_pinned_request_the_shell_actually_sends() {
    let Some(mut session) = session() else { return };
    use pdfcer_core::text_edit::{
        BlockRecognitionOptions, EditOptions, EditRequest, EditableTextModel,
    };

    let (index, span, target, run_text) = {
        let view = session.view();
        let pages = pdfcer_core::page_tree::pages_in(&view).expect("a page tree");
        let text = pdfcer_core::text_extract::extract_page_view(
            &view,
            &pages[0],
            0,
            &pdfcer_core::text_extract::ExtractOptions::default().with_provenance(true),
        )
        .expect("page 0 extracts with provenance");

        let Some((i, run)) = text
            .runs
            .iter()
            .enumerate()
            .find(|(_, r)| r.text.contains("USE SPACERS"))
        else {
            println!("his line is not on page 0");
            return;
        };
        let run_text = run.text.clone();

        let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
        match pdfcer_gui::canvas::textedit::pin::of_run(&model, i) {
            Some(p) => (i, p.span, p.target, run_text),
            None => {
                println!("★★★ NO PIN for run {i} ({run_text:?})");
                println!("    `pin::of_run` returned None — the run carries no provenance, so");
                println!("    the shell cannot pin it and the whole pinned route is unavailable.");
                return;
            }
        }
    };
    println!("run {index}: {run_text:?}");
    println!("  pin span   : {span:?}");
    println!("  pin target : {target:?}");

    let mut req = EditRequest::whole_operator(0, span, "#2 USE SPACER 8 9 10 11 IF REQUIRED.");
    req.target = target;
    match session.edit_text(&req, &EditOptions::default()) {
        Ok(report) => {
            println!("ACCEPTED with the pinned shape");
            println!("  operators_spanned: {}", report.operators_spanned);
        }
        Err(e) => {
            println!("★★★ REFUSED with the pinned shape — {e}");
            use pdfcer_core::text_edit::RefusalClass as _;
            println!("    kind: {:?}", e.refusal_kind());
        }
    }
}

/// The exact count `plan::page_occurrences` computes for his run.
#[test]
#[ignore = "reads a file outside the repository; run by hand"]
fn what_count_decides_whether_the_pin_comes_off() {
    let Some(session) = session() else { return };
    let text = page_text(&session, 0).expect("page 0 extracts");
    let whole: String = text.runs.iter().map(|r| r.text.as_str()).collect();
    let Some(run) = text.runs.iter().find(|r| r.text.contains("USE SPACERS")) else {
        return;
    };
    let needle = run.text.as_str();
    println!("needle : {needle:?}");
    println!("count  : {}", whole.matches(needle).count());
    println!("⇒ the pin is dropped ONLY when this is exactly 1");
}
