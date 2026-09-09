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

/// ★★★ **Which of his BOM cells actually hit the ambiguity refusal, and is the
/// remedy that refusal names performable on them?**
///
/// `EditRefusal::AmbiguousOnThePage` fires on the intersection of two
/// conditions, not on either alone:
///
/// 1. the clicked run spans **more than one show operator** (otherwise the pin
///    is exact, `find` is cleared, and the whole-operator route works), **and**
/// 2. its text occurs **more than once** on the page.
///
/// Everything measured so far has counted condition 2 in isolation — 122 runs
/// with repeated text on that sheet — which is the number that made the report
/// legible but is **not** the number of cells he cannot edit. Most of those may
/// be single-operator and edit perfectly.
///
/// ⇒ So this counts the intersection. That is the population of the defect.
///
/// ## ⚠ And the second question is the one that could be a defect of ours
///
/// `text::editrefusal::ambiguous_on_the_page` tells him:
///
/// > *"Click in the line again and include more of it in your change — a longer
/// > stretch appears only once."*
///
/// That remedy is real for a **line of prose**: a repeated phrase inside a
/// longer sentence has a unique superstring. It is **not obviously real for a
/// BOM cell**, where the run IS the whole cell and there is nothing longer to
/// include. A refusal naming a remedy the operator cannot perform is worse than
/// one naming none, and this project's own rule is that a sentence naming a
/// remedy is a claim about the build.
///
/// So this also asks, for each affected run, whether the LINE it sits on
/// carries anything beyond the run itself — which is what "include more of it"
/// would have to mean.
#[test]
#[ignore = "reads a file outside the repository; run by hand"]
fn how_many_runs_actually_hit_the_ambiguity_refusal_and_is_the_remedy_real() {
    let Some(session) = session() else { return };
    use pdfcer_core::text_edit::{BlockRecognitionOptions, EditableTextModel, TextPosition};

    let view = session.view();
    let pages = pdfcer_core::page_tree::pages_in(&view).expect("a page tree");

    for (page_index, page) in pages.iter().enumerate().take(4) {
        let Ok(text) = pdfcer_core::text_extract::extract_page_view(
            &view,
            page,
            page_index,
            &pdfcer_core::text_extract::ExtractOptions::default().with_provenance(true),
        ) else {
            continue;
        };
        if text.runs.len() < 50 {
            continue;
        }
        let whole: String = text.runs.iter().map(|r| r.text.as_str()).collect();
        let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());

        let mut repeated = 0usize;
        let mut split = 0usize;
        let mut refused = 0usize;
        let mut refused_with_no_longer_stretch = 0usize;
        let mut samples: Vec<String> = Vec::new();

        for (i, run) in text.runs.iter().enumerate() {
            let t = run.text.trim();
            if t.is_empty() {
                continue;
            }
            let n = whole.matches(run.text.as_str()).count();
            let one_operator = pdfcer_gui::canvas::textedit::pin::spans_one_operator(&model, i);
            if n > 1 {
                repeated += 1;
            }
            if !one_operator {
                split += 1;
            }
            if n > 1 && !one_operator {
                refused += 1;
                // ★ Is there anything else on this run's LINE? If the line is
                // the run and nothing else, "include more of it" names an
                // action he cannot take.
                let alone = model
                    .line_range_at(TextPosition::new(i, 0))
                    .is_none_or(|(from, to)| from.run == to.run);
                if alone {
                    refused_with_no_longer_stretch += 1;
                    if samples.len() < 12 {
                        samples.push(format!("{:?} x{n}", run.text));
                    }
                }
            }
        }

        println!(
            "\n=== page {} ({} runs) ===",
            page_index + 1,
            text.runs.len()
        );
        println!("  text repeats on the page      : {repeated}");
        println!("  spans >1 show operator        : {split}");
        println!("  ★ BOTH — the refused population: {refused}");
        println!(
            "  ⚠ …of those, alone on their line (remedy IMPOSSIBLE): {refused_with_no_longer_stretch}"
        );
        for sample in &samples {
            println!("      {sample}");
        }
    }
}

/// ★★★ **Ask the engine about EVERY cell on his BOM sheet, one fresh session
/// per cell, and print the ones it refuses.**
///
/// # Why this replaces two wrong diagnoses rather than adding a third
///
/// The BOM half of his report has now been explained twice and both
/// explanations were wrong:
///
/// 1. *"the planner throws the pin away"* — it does not; it keeps it
///    deliberately.
/// 2. *"a repeated cell hits `AmbiguousOnThePage`"* — measured, and the
///    population of that refusal on his file is **zero**. It needs a run that
///    both repeats AND spans several show operators; his sheets carry 57–133
///    repeating runs and 4–11 multi-operator runs, and **they do not
///    intersect**.
///
/// ⇒ Both were derived from counting one condition and reasoning about the
/// other. This one counts the outcome instead: it performs the edit and reads
/// the answer. That is slower — one `Document::load` per cell — and it is the
/// only measurement that cannot be wrong about what the engine does.
///
/// ⚠ A fresh session per cell is not an optimisation to remove. `edit_text`
/// mutates, so a shared session would measure the Nth edit **on a document
/// already edited N-1 times**, and `PageEditedThisSession` is a real refusal
/// this file would then attribute to the cell.
#[test]
#[ignore = "reads a file outside the repository, one document load per cell; run by hand"]
fn which_cells_on_his_bom_sheet_does_the_engine_actually_refuse() {
    use pdfcer_core::text_edit::{
        BlockRecognitionOptions, EditOptions, EditRequest, EditableTextModel, RefusalClass as _,
    };

    let Some(first) = session() else { return };
    // The BOM is the busiest sheet.
    let (page, count) = {
        let session = &first;
        let pages = session.pages().expect("a page tree");
        let mut best = (0usize, 0usize);
        for (i, _) in pages.iter().enumerate() {
            let n = page_text(session, i).map_or(0, |t| t.runs.len());
            if n > best.1 {
                best = (i, n);
            }
        }
        best
    };
    println!("busiest sheet is page {} with {count} runs", page + 1);
    drop(first);

    // Every run's text and pin, measured once from a read-only session.
    let plans: Vec<(usize, String, bool)> = {
        let session = session().expect("a session");
        let view = session.view();
        let pages = pdfcer_core::page_tree::pages_in(&view).expect("a page tree");
        let text = pdfcer_core::text_extract::extract_page_view(
            &view,
            &pages[page],
            page,
            &pdfcer_core::text_extract::ExtractOptions::default().with_provenance(true),
        )
        .expect("the sheet extracts");
        let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
        text.runs
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.text.trim().is_empty())
            .map(|(i, r)| {
                (
                    i,
                    r.text.clone(),
                    pdfcer_gui::canvas::textedit::pin::spans_one_operator(&model, i),
                )
            })
            .collect()
    };

    // ★ Sampled, and the sample is STATED. Every 7th run keeps the run under a
    // minute while covering the whole sheet rather than its first screenful —
    // a prefix would measure the title block and call it a bill of materials.
    let sample: Vec<_> = plans.iter().step_by(7).collect();
    println!(
        "asking the engine about {} of {} cells",
        sample.len(),
        plans.len()
    );

    let mut refused: Vec<(usize, String, String)> = Vec::new();
    let mut accepted = 0usize;
    let mut no_pin = 0usize;

    for (i, run_text, one_operator) in sample {
        let Some(mut session) = session() else { return };
        let (span, target) = {
            let view = session.view();
            let pages = pdfcer_core::page_tree::pages_in(&view).expect("a page tree");
            let text = pdfcer_core::text_extract::extract_page_view(
                &view,
                &pages[page],
                page,
                &pdfcer_core::text_extract::ExtractOptions::default().with_provenance(true),
            )
            .expect("the sheet extracts");
            let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
            match pdfcer_gui::canvas::textedit::pin::of_run(&model, *i) {
                Some(p) => (p.span, p.target),
                None => {
                    no_pin += 1;
                    continue;
                }
            }
        };

        // ★★★ THE REPLACEMENT REUSES THE RUN'S OWN CHARACTERS, and the first
        // draft of this probe did not.
        //
        // It appended a `Z`, and **30 of 31 cells came back
        // `UnsupportedFont`** — a result that looked like a spectacular
        // finding and was an artefact of the harness. His drawing's fonts are
        // SUBSET-EMBEDDED: they carry the glyphs the drawing uses and no
        // others, so asking for a `Z` on a sheet with no `Z` refuses
        // correctly, on every cell, for a reason that has nothing to do with
        // the cell.
        //
        // ⇒ Doubling the run's LAST character asks for a glyph the run itself
        // proves is present, so a refusal is about the location rather than
        // about the alphabet — which is the question.
        let replacement = match run_text.chars().last() {
            Some(c) => format!("{run_text}{c}"),
            None => continue,
        };
        let mut request = if *one_operator {
            EditRequest::whole_operator(page, span, &replacement)
        } else {
            let mut r = EditRequest::find_replace(page, run_text, &replacement);
            r.pinned_span = Some(span);
            r
        };
        request.target = target;

        match session.edit_text(&request, &EditOptions::default()) {
            Ok(_) => accepted += 1,
            Err(e) => refused.push((*i, run_text.clone(), format!("{:?}", e.refusal_kind()))),
        }
    }

    println!("\naccepted : {accepted}");
    println!("no pin   : {no_pin}");
    println!("REFUSED  : {}", refused.len());
    for (i, t, kind) in refused.iter().take(30) {
        println!("  run {i:4}  {kind:<16} {t:?}");
    }
}

/// ★★★ **What can he actually TYPE into this drawing?**
///
/// # Where this question came from — a harness artefact worth more than the
/// measurement it broke
///
/// The probe above appended a `Z` in its first draft and **30 of 31 cells came
/// back `UnsupportedFont`.** That looked like a finding and was an artefact:
/// his drawing's fonts are subset-embedded, so a `Z` on a sheet with no `Z` is
/// refused correctly, on every cell, for a reason that has nothing to do with
/// the cell.
///
/// ⇒ But *the artefact is a report about his working day*. Editing a bill of
/// materials means typing part numbers and quantities, and the alphabet he is
/// allowed is not the keyboard's — it is **whatever the drawing already
/// contains**. A capital `Z` is not a strange thing to want; `SPACER` →
/// `SPAZER` is silly, but `12` → `13` on a sheet whose quantities happen never
/// to include a `3` is exactly the same refusal.
///
/// ★ *"It only sometimes works"* is what a per-character alphabet feels like
/// from the operator's chair, and it is a better fit for his words than either
/// of the two explanations that preceded it — both of which turned out to be
/// about populations that are empty on this file.
///
/// # What this measures
///
/// For one cell on the BOM sheet, every printable ASCII character in turn:
/// append it, ask the engine, record accept or refuse. That is the alphabet.
#[test]
#[ignore = "reads a file outside the repository, one document load per character; run by hand"]
fn which_characters_can_he_type_into_his_bom_sheet() {
    use pdfcer_core::text_edit::{
        BlockRecognitionOptions, EditOptions, EditRequest, EditableTextModel, RefusalClass as _,
    };

    // A word cell rather than a number cell: it exercises letters, and it is
    // the kind of thing he would retype.
    const AIM: &str = "BRACE";
    let page = 2; // 0-based; the busiest sheet, measured by the probe above.

    let mut ok = String::new();
    let mut refused = String::new();
    let mut other: Vec<(char, String)> = Vec::new();

    for c in (0x20u8..0x7f).map(char::from) {
        let Some(mut session) = session() else { return };
        let (index, span, target, run_text) = {
            let view = session.view();
            let pages = pdfcer_core::page_tree::pages_in(&view).expect("a page tree");
            let text = pdfcer_core::text_extract::extract_page_view(
                &view,
                &pages[page],
                page,
                &pdfcer_core::text_extract::ExtractOptions::default().with_provenance(true),
            )
            .expect("the sheet extracts");
            let Some((i, run)) = text
                .runs
                .iter()
                .enumerate()
                .find(|(_, r)| r.text.trim() == AIM)
            else {
                println!("{AIM:?} is not on page {}", page + 1);
                return;
            };
            let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
            let Some(pin) = pdfcer_gui::canvas::textedit::pin::of_run(&model, i) else {
                println!("no pin for {AIM:?}");
                return;
            };
            (i, pin.span, pin.target, run.text.clone())
        };
        let _ = index;

        let mut request =
            EditRequest::whole_operator(page, span, &format!("{}{c}", run_text.trim()));
        request.target = target;
        match session.edit_text(&request, &EditOptions::default()) {
            Ok(_) => ok.push(c),
            Err(e) => {
                let kind = format!("{:?}", e.refusal_kind());
                if kind.contains("UnsupportedFont") {
                    refused.push(c);
                } else {
                    other.push((c, kind));
                }
            }
        }
    }

    println!("\n=== the alphabet of {AIM:?} on page {} ===", page + 1);
    println!("ACCEPTED ({:3}): {ok}", ok.chars().count());
    println!("REFUSED  ({:3}): {refused}", refused.chars().count());
    for (c, kind) in &other {
        println!("  {c:?} -> {kind}");
    }
    println!(
        "\n=> he may type {} of the 95 printable ASCII characters into this cell",
        ok.chars().count()
    );
}

/// ★★★ **The alphabet PER FONT — because the probe above measured one cell and
/// its number went into three documents as though it described the drawing.**
///
/// # The correction, and where it came from
///
/// `which_characters_can_he_type_into_his_bom_sheet` aims at the run whose text
/// is `"BRACE"`, measures 46 of 95 printable ASCII accepted, and that is a true
/// statement **about that cell's font**. It was written up as *"his drawing's
/// fonts accept 46 of 95, with every lowercase letter absent"* — a claim about
/// the document, from a sample of one.
///
/// `pdfcer-core` measured the same file per font and got a different shape:
///
/// > four of the six fonts 72/95, missing exactly `h j l q z Z`; two of them
/// > 38/95 and 24/95, missing all lowercase.
///
/// ⇒ **And their framing is better than either number**: a subset can draw
/// exactly what the document already contains, so the edits that fail are the
/// ones introducing a **novel** character. That describes *"it only sometimes
/// works"* more precisely than any coverage fraction does — it is not that
/// lowercase is banned, it is that `h`, `j`, `l`, `q`, `z` and `Z` never appear
/// on those sheets.
///
/// # Why this exists rather than adopting their number
///
/// Because the number in this repository's documents is the one this
/// repository has to be able to re-run. A measurement borrowed from another
/// project's commit message is a citation, not a measurement, and this session
/// has already corrected three claims that were exactly that.
///
/// ★ One cell per distinct `font_resource`, so the sample covers the page's
/// fonts rather than its geometry.
#[test]
#[ignore = "reads a file outside the repository, ~95 document loads per font; run by hand"]
fn what_can_he_type_into_each_of_the_sheets_fonts() {
    // ★★★ EVERY SHEET IN THE SAMPLE, not one, and that is the correction
    // this probe exists for. `pdfcer-core` measured the same file per font
    // and got four fonts at 72/95 where this repository's documents said
    // 46/95 — and both are right, about different SHEETS. A title/BOM sheet
    // is set entirely in capitals, so its fonts carry no lowercase at all; a
    // sheet with prose notes carries nearly all of it. Sampling one sheet and
    // writing the number up as a fact about the drawing is the same shape of
    // error as sampling one cell.
    for page in 0..4 {
        measure_page(page);
    }
}

fn measure_page(page: usize) {
    use pdfcer_core::text_edit::{
        BlockRecognitionOptions, EditOptions, EditRequest, EditableTextModel, RefusalClass as _,
    };

    // One representative run per font resource, chosen as the LONGEST run in
    // that font: a long run proves more of the subset is reachable and gives
    // the replacement a wider choice of its own characters to reuse.
    let mut per_font: Vec<(String, usize, String)> = Vec::new();
    {
        let Some(session) = session() else { return };
        let view = session.view();
        let pages = pdfcer_core::page_tree::pages_in(&view).expect("a page tree");
        let text = pdfcer_core::text_extract::extract_page_view(
            &view,
            &pages[page],
            page,
            &pdfcer_core::text_extract::ExtractOptions::default().with_provenance(true),
        )
        .expect("the sheet extracts");
        for (i, run) in text.runs.iter().enumerate() {
            if run.text.trim().is_empty() {
                continue;
            }
            // ★ The font lives on the GLYPH's provenance, not on the run:
            // a run is closed on geometry, so it can in principle carry
            // glyphs from more than one resource. The first glyph's is the
            // one the pin will address.
            let font = run
                .glyphs
                .first()
                .and_then(|g| g.provenance.as_ref())
                .and_then(|p| p.font_resource.as_ref())
                .map_or_else(
                    || "none".to_owned(),
                    |f| String::from_utf8_lossy(f).into_owned(),
                );
            match per_font.iter_mut().find(|(f, _, _)| *f == font) {
                Some(slot) if slot.2.len() < run.text.len() => {
                    *slot = (font, i, run.text.clone());
                }
                Some(_) => {}
                None => per_font.push((font, i, run.text.clone())),
            }
        }
    }
    println!(
        "page {} carries {} distinct fonts",
        page + 1,
        per_font.len()
    );

    for (font, _, sample) in &per_font {
        let mut ok = String::new();
        let mut refused = String::new();
        for c in (0x20u8..0x7f).map(char::from) {
            let Some(mut session) = session() else { return };
            let (span, target, run_text) = {
                let view = session.view();
                let pages = pdfcer_core::page_tree::pages_in(&view).expect("a page tree");
                let text = pdfcer_core::text_extract::extract_page_view(
                    &view,
                    &pages[page],
                    page,
                    &pdfcer_core::text_extract::ExtractOptions::default().with_provenance(true),
                )
                .expect("the sheet extracts");
                let Some((i, run)) = text
                    .runs
                    .iter()
                    .enumerate()
                    .find(|(_, r)| r.text == *sample)
                else {
                    break;
                };
                let model =
                    EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
                let Some(pin) = pdfcer_gui::canvas::textedit::pin::of_run(&model, i) else {
                    break;
                };
                (pin.span, pin.target, run.text.clone())
            };

            let mut request =
                EditRequest::whole_operator(page, span, &format!("{}{c}", run_text.trim()));
            request.target = target;
            match session.edit_text(&request, &EditOptions::default()) {
                Ok(_) => ok.push(c),
                Err(e) if format!("{:?}", e.refusal_kind()).contains("UnsupportedFont") => {
                    refused.push(c);
                }
                Err(_) => {}
            }
        }
        println!("\n--- font {font} (sample {:?}) ---", sample.trim());
        println!("  accepted {:>2}/95", ok.chars().count());
        println!("  refused        : {refused}");
    }
}

// ---------------------------------------------------------------------------
// 2026-09-09 — two more probes, for two of the operator's four reports that
// morning: *"we're back to the apply redactions box that just tells me we
// can't do it"* and *"some text in sw41177 still isn't editable"*. Both are
// measured against HIS files (three of them, staged under
// `target/scratch/ken/`), headlessly, with the engine at the revision
// `Cargo.lock` pins — which is the only way to know whether a fix the engine
// shipped hours ago reaches his drawing or not.
// ---------------------------------------------------------------------------

const HIS_FILES: [&str; 3] = [
    "target/scratch/ken/SW41177.pdf",
    "target/scratch/ken/SW41177 INSTALLATION.pdf",
    "target/scratch/ken/SW41177 MATERIAL REQUIREMENTS.pdf",
];

fn his(path: &str) -> Option<EditSession> {
    let full = format!("{}/../../{path}", env!("CARGO_MANIFEST_DIR"));
    let p = std::path::Path::new(&full);
    if !p.exists() {
        println!("SKIP: {full} is not present");
        return None;
    }
    Some(EditSession::new(Document::load(p).expect("his file loads")))
}

/// **The redaction gate, on his files.** `Apply redactions` opens by calling
/// `prepare_redaction_apply`, whose FIRST step is `to_full_bytes` — a full
/// rewrite of the session. If that refuses, the window shows
/// `FullRewriteUnavailable`: *"this document cannot be rewritten in full"* —
/// which is the sentence an operator reads as *"we can't do it"*. This probe
/// asks that exact question of each of his three drawings, with no marks
/// placed, so the answer is about the FILE and nothing else.
#[test]
#[ignore = "reads files outside the repository; run by hand"]
fn can_his_drawings_be_rewritten_in_full_for_a_redaction() {
    for path in HIS_FILES {
        let Some(session) = his(path) else { continue };
        match session.to_full_bytes(&pdfcer_core::writer::SaveOptions::identity()) {
            Ok((bytes, report)) => println!(
                "{path}: FULL REWRITE OK — {} bytes, report {report:?}",
                bytes.len()
            ),
            Err(e) => println!("{path}: FULL REWRITE REFUSED — {e}"),
        }
    }
}

/// **The font-coverage remedy, end to end, on his file.** The engine's
/// refusal now names the standard-14 faces that would accept the character
/// (`Pass 274.0`), and as of `Pass 279.0` (`5b8ec61`) it names only faces
/// `set_font` would actually author fresh rather than resolve back onto the
/// refusing subset. This probe: attempts the edit he described, prints the
/// refusal verbatim, takes the FIRST standard-14 face named in it, sets the
/// run to that face through the same `FormatRequest` the Format tab sends,
/// and retries the edit. PASS is the second attempt being accepted.
#[test]
#[ignore = "reads files outside the repository; run by hand"]
fn does_the_face_the_refusal_names_actually_unblock_his_edit() {
    let Some(mut session) = his(HIS_FILES[0]) else {
        return;
    };
    let text = page_text(&session, 0).expect("page 0 extracts");
    let Some((index, run)) = text
        .runs
        .iter()
        .enumerate()
        .find(|(_, r)| r.text.contains("USE SPACERS"))
    else {
        println!("his line is not on page 0");
        return;
    };
    println!("run {index}: {:?}", run.text);
    // A lowercase word: his subsets are capitals-only on this sheet, so this
    // is the edit that refuses.
    let req = pdfcer_core::text_edit::EditRequest::find_replace(0, "SPACERS", "spacers");
    let opts = pdfcer_core::text_edit::EditOptions::default();
    let first = session.edit_text(&req, &opts);
    let message = match &first {
        Ok(_) => {
            println!("ACCEPTED on the first attempt — nothing to remedy");
            return;
        }
        Err(e) => {
            println!("REFUSED — {e}");
            e.to_string()
        }
    };
    const STD14: [&str; 12] = [
        "Helvetica-BoldOblique",
        "Helvetica-Oblique",
        "Helvetica-Bold",
        "Helvetica",
        "Times-BoldItalic",
        "Times-Italic",
        "Times-Bold",
        "Times-Roman",
        "Courier-BoldOblique",
        "Courier-Oblique",
        "Courier-Bold",
        "Courier",
    ];
    let mut named: Vec<(usize, &str)> = STD14
        .iter()
        .filter_map(|f| message.find(f).map(|at| (at, *f)))
        .collect();
    named.sort();
    // Longest-name-first above, so "Helvetica-Bold" is not reported as
    // "Helvetica" at the same offset; the sort then puts them in message order.
    named.dedup_by_key(|(at, _)| *at);
    println!(
        "faces named by the refusal, in order: {:?}",
        named.iter().map(|(_, f)| *f).collect::<Vec<_>>()
    );
    let Some((_, face)) = named.first() else {
        println!("★★★ the refusal named NO face — the remedy is absent from the sentence");
        return;
    };
    let fmt = pdfcer_core::text_edit::FormatRequest::new(0, "SPACERS")
        .font(pdfcer_core::text_edit::FontSelector::new(face));
    match session.format_text(&fmt, &pdfcer_core::text_edit::FormatOptions::default()) {
        Ok(report) => println!("set_font({face}) ACCEPTED — {report:?}"),
        Err(e) => {
            println!("set_font({face}) REFUSED — {e}");
            return;
        }
    }
    match session.edit_text(&req, &opts) {
        Ok(report) => println!(
            "★ SECOND ATTEMPT ACCEPTED — the named face unblocked the edit (operators_spanned={})",
            report.operators_spanned
        ),
        Err(e) => println!("★★★ SECOND ATTEMPT STILL REFUSED — {e}"),
    }
}
