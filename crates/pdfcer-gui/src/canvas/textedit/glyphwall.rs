//! # `canvas::textedit::glyphwall` — his typo, the pin that was stopping it, and
//! the occurrence count that makes dropping the pin safe
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/glyphwall.md`.

#![cfg(test)]
// ---------------------------------------------------------------------------
// A deliberate duplicate of the `#![cfg(test)]` above, for
// `tools/gates/check-ui-strings.sh` rather than for rustc — the same device
// `proof.rs` and `facewall.rs` use and for the same reason: the gate reads
// modules line by line and cannot see an inner attribute at the top of a file
// it is scanning for bare string literals.
// ---------------------------------------------------------------------------
#![cfg(test)]

use pdfcer_core::document::Document;
use pdfcer_core::edit::EditSession;
use pdfcer_core::text_edit::{EditOptions, EditRequest};

/// The per-glyph run both fixtures draw, and the correction made to it.
const RUN: &str = "ABC";
const FIXED: &str = "ABCD";

/// A correction to the **same** run that changes its first and last characters
/// and leaves the middle one alone.
const STRADDLED: &str = "XBY";

/// The fixture whose page holds the run **once**.
const UNIQUE: &str = "per-glyph-operators.pdf";
/// The fixture whose page holds the identical run **twice**.
const TWICE: &str = "per-glyph-twice.pdf";

fn session(fixture: &str) -> EditSession {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(fixture);
    assert!(
        path.exists(),
        "the fixture is missing at {}. Regenerate both with \
         `python tools/gen-per-glyph-fixtures.py`",
        path.display()
    );
    EditSession::new(Document::load(&path).expect("the fixture loads"))
}

/// The page's runs **in order**, as the session now sees them.
fn page_runs(session: &EditSession) -> Vec<String> {
    let view = session.view();
    let pages = pdfcer_core::page_tree::pages_in(&view).expect("a page tree");
    pdfcer_core::text_extract::extract_page_view(
        &view,
        &pages[0],
        0,
        &pdfcer_core::text_extract::ExtractOptions::default(),
    )
    .expect("the page's text extracts")
    .runs
    .iter()
    .map(|r| r.text.clone())
    .collect()
}

/// The page's text as the session now sees it — the overlay, not the file.
fn page_text(session: &EditSession) -> String {
    let view = session.view();
    let pages = pdfcer_core::page_tree::pages_in(&view).expect("a page tree");
    let text = pdfcer_core::text_extract::extract_page_view(
        &view,
        &pages[0],
        0,
        &pdfcer_core::text_extract::ExtractOptions::default(),
    )
    .expect("the page's text extracts");
    text.runs.iter().map(|r| r.text.as_str()).collect()
}

/// **The control that makes every assertion below evidence: the fixture's
/// run really is split across operators.**
#[test]
fn the_fixtures_runs_are_written_one_glyph_per_operator() {
    for fixture in [UNIQUE, TWICE] {
        let s = session(fixture);
        let planned = super::plan(
            &crate::app::state::open_local_fixture(fixture),
            0,
            0,
            RUN,
            FIXED,
        );
        assert!(
            !planned.one_operator,
            "{fixture}'s run 0 must span more than one show operator, or this module is \
             measuring the exact-pin branch. Regenerate with \
             `python tools/gen-per-glyph-fixtures.py`"
        );
        drop(s);
    }
}

/// **HIS TYPO. The pin stays on, the span search starts at it, and the
/// correction lands.**
#[test]
fn a_typo_in_a_run_written_one_glyph_at_a_time_can_be_corrected() {
    let doc = crate::app::state::open_local_fixture(UNIQUE);
    let planned = super::plan(&doc, 0, 0, RUN, FIXED);

    // **THE PIN STAYS ON**, and the obvious simplification here is to
    // drop it because this fixture holds the run only once.
    //
    // `EditRequest::spanning_from` starts the span search at the pinned
    // operator rather than at the first operator on the page, so the pin does
    // not have to be traded away to reach a split run. And trading it away was
    // never as safe as it reads: `find_anchor` tries a **single-operator**
    // match across the whole page before the spanning search runs, so a
    // single-operator twin anywhere on the sheet beats a spanning occurrence
    // above it — dropping the pin can make the clicked run *unreachable*, not
    // merely ambiguous.
    assert!(
        planned.request.pinned_span.is_some(),
        "★★★ THE PIN MUST STAY ON. It is what makes the edit address THIS run, and \
         `span_from_pin` is what lets it span anyway. A build that dropped it again would \
         pass every other assertion here on this fixture — which holds one occurrence — \
         and reach the wrong text on a page with two"
    );
    // **AND IT SPANS FROM THE PIN.** The whole-run `find` says what, the
    // pin says which one, and `span_from_pin` lets the match run on past the
    // one-glyph operator the pin names. The engine narrows the rewrite to the
    // part that differs by itself.
    assert!(
        planned.request.span_from_pin,
        "★★★ THE REQUEST MUST SPAN FROM THE PIN. A `find` beside a plain pin is confined to \
         the one operator the pin names, which on a per-glyph run holds a single character, \
         and the engine refuses it. That is the defect O142 reported"
    );
    assert_eq!(
        planned.request.find, RUN,
        "★★ …and the `find` must be the whole run: it says WHAT, while the pin says WHICH ONE"
    );
    assert_eq!(
        planned.request.replace, FIXED,
        "★★ …and the replacement must be the whole corrected run, matching the whole-run `find`"
    );

    let mut session = session(UNIQUE);
    // Sampled BEFORE the edit, from the same session the edit runs in, so the
    // comparison below is against this document rather than against a number
    // written down when the fixture was authored.
    let before_left = left_edge(&session, 0);
    session
        .edit_text(&planned.request, &planned.options)
        .expect("the correction must reach the document — this is O142");
    assert!(
        page_text(&session).contains(FIXED),
        "and the corrected text must be IN the page, not merely un-refused: a verb that \
         returns Ok and leaves the run alone is what this assertion exists to catch"
    );

    // **AND THE LINE MUST NOT MOVE** — the operator's own report, O213:
    //
    // > *"the entire line shifts to the right after instead of staying in
    // > place."*
    //
    // A spanning edit puts the replacement into the operator holding the match's
    // END and empties the ones before it; the engine keeps the line where the
    // match began.
    //
    // The tolerance is 0.5 pt: loose enough that a legitimate re-spacing of
    // sub-point size never trips it, tight enough that no displacement a reader
    // could see can pass. A whole-operator shift on this fixture is 7 pt.
    let moved = left_edge(&session, 0) - before_left;
    assert!(
        moved.abs() < 0.5,
        "★★★ THE CORRECTION MOVED THE LINE. Its left edge shifted by {moved:.3} pt, and an \
         edit that relocates the text it corrects is a worse defect than the typo. This is \
         O213 on a fixture"
    );
}

/// **A change that touches the run's first and last operators** and leaves
/// the middle one alone — the engine cannot narrow it to one operator, so the
/// span genuinely crosses all three.
///
/// Asserts that the shell still *reaches* the run and still addresses **this**
/// occurrence.
#[test]
fn a_change_that_straddles_operators_keeps_the_spanning_form() {
    let doc = crate::app::state::open_local_fixture(UNIQUE);
    let planned = super::plan(&doc, 0, 0, RUN, STRADDLED);

    assert!(
        planned.request.pinned_span.is_some(),
        "★★★ and the pin must still be on — it is the only thing `EditRequest` carries that \
         chooses between two identical strings on one page"
    );
    assert!(
        planned.request.span_from_pin,
        "★★★ …with the flag that makes the pin survivable. A `find` beside a plain pin is \
         confined to the one operator the pin names, which on a per-glyph run holds a single \
         character, and the engine refuses it. That is the defect O142 reported"
    );
    assert_eq!(
        planned.request.find, RUN,
        "and the `find` must be the whole run: it says WHAT, while the pin says WHICH ONE"
    );

    let mut session = session(UNIQUE);
    let report = session
        .edit_text(&planned.request, &planned.options)
        .expect("★★★ the straddling correction must still reach the document");
    assert!(
        report.operators_spanned > 1,
        "★★ it must land BY SPANNING. `operators_spanned` was {}; a fixture that stopped \
         being per-glyph would satisfy everything else here while testing nothing",
        report.operators_spanned
    );
    assert!(
        page_text(&session).contains(STRADDLED),
        "and the corrected text must be IN the page, not merely un-refused"
    );
}

/// **The left edge of run `index` on page 0** — the one number that says whether
/// an edit left the text where the producer put it.
fn left_edge(session: &EditSession, index: usize) -> f64 {
    let view = session.view();
    let pages = pdfcer_core::page_tree::pages_in(&view).expect("a page tree");
    let text = pdfcer_core::text_extract::extract_page_view(
        &view,
        &pages[0],
        0,
        &pdfcer_core::text_extract::ExtractOptions::default(),
    )
    .expect("the page's text extracts");
    text.runs
        .get(index)
        .unwrap_or_else(|| panic!("the page must still hold run {index}"))
        .bbox
        .unwrap_or_else(|| panic!("run {index} must carry a bounding box to measure"))
        .llx
}

/// **THE GUARD, INVERTED 2026-09-08: two identical runs on one page, and
/// the shell now edits THE ONE THAT WAS CLICKED.**
#[test]
fn a_typo_that_appears_twice_on_the_page_edits_the_one_that_was_clicked() {
    let doc = crate::app::state::open_local_fixture(TWICE);
    let planned = super::plan(&doc, 0, 0, RUN, FIXED);

    assert!(
        planned.request.pinned_span.is_some(),
        "★★★ THE PIN. It is the only thing `EditRequest` carries that can choose between two \
         identical strings on one page — there is no occurrence index on the request — so a \
         build without it hands the choice to the engine's left-to-right scan"
    );
    assert!(
        planned.request.span_from_pin,
        "★★★ …and the span search must start AT the pin. Without it a whole-run `find` beside \
         a plain pin cannot match a per-glyph run"
    );
    assert_eq!(
        planned.request.find, RUN,
        "★★ …with the whole run as `find`: it says WHAT, while the pin says WHICH ONE"
    );

    let mut session = session(TWICE);
    let before = page_runs(&session);
    assert!(
        before.iter().filter(|r| r.trim() == RUN).count() >= 2,
        "★ the control: this fixture must really hold TWO identical runs, or this test is \
         measuring the unique case and asserts nothing about disambiguation. Regenerate \
         with `python tools/gen-per-glyph-fixtures.py`. Got: {before:?}"
    );

    // The control that the fixture is really per-glyph is
    // [`the_fixtures_runs_are_written_one_glyph_per_operator`], which reads
    // `Plan::one_operator` over both fixtures. It cannot be `operators_spanned`:
    // the engine narrows the rewrite to the operators that differ, so that
    // number does not distinguish a split run from a whole one.
    assert!(
        !planned.one_operator,
        "★ the control: the clicked run must really be written across several show \
         operators, or this test is measuring the shape that never had the defect"
    );
    session
        .edit_text(&planned.request, &planned.options)
        .expect("★★★ the edit that was refused until Pass 272.0 must now land");

    // THE ASSERTION THAT DISCRIMINATES. Not "the page contains ABCD" —
    // a build that scanned from operator 0 satisfies that too, on this exact
    // page, while having edited the wrong run.
    let after = page_runs(&session);
    assert_eq!(
        after.iter().filter(|r| r.trim() == FIXED).count(),
        1,
        "★★★ exactly ONE occurrence may have changed. Two means the verb rewrote both; \
         zero means it landed somewhere this test cannot see. Got: {after:?}"
    );
    assert_eq!(
        after.iter().filter(|r| r.trim() == RUN).count(),
        before.iter().filter(|r| r.trim() == RUN).count() - 1,
        "★★ …and exactly one untouched occurrence must remain, which is what says the OTHER \
         one is still there rather than having been consumed by a spanning match that ran \
         too far. Got: {after:?}"
    );

    // And it must be the FIRST — the run `plan` was given (index 0). Order
    // is the only thing that names which of two identical strings was edited;
    // asserting on their number cannot. This is the engine's own lesson about
    // its third sabotage, applied here.
    let first_fixed = after.iter().position(|r| r.trim() == FIXED);
    let first_run = after.iter().position(|r| r.trim() == RUN);
    assert!(
        matches!((first_fixed, first_run), (Some(f), Some(r)) if f < r),
        "★★★ the CLICKED run — run 0, the first on the page — must be the one that changed. \
         If the corrected text appears after the untouched one, the engine scanned from the \
         top of the page and the pin did nothing. Got: {after:?}"
    );
}

/// **The guard is a DECISION, not an inherited limitation** — asserted
/// against the engine directly, with no `plan` in the way.
#[test]
fn the_engine_would_have_edited_the_wrong_one() {
    let mut session = session(TWICE);
    let _report = session
        .edit_text(
            &EditRequest::find_replace(0, RUN, FIXED),
            &EditOptions::default(),
        )
        .expect(
            "★★★ UNPINNED, THE ENGINE ACCEPTS THIS. If it ever refuses, the engine has \
             gained an ambiguity check of its own and this shell's count may be able to \
             retire — read its refusal before deleting anything",
        );

    let after = page_text(&session);
    assert_eq!(
        after.matches(FIXED).count(),
        1,
        "★★ EXACTLY ONE of the two was changed, chosen by nothing but scan order. That is \
         the outcome the guard refuses on the operator's behalf: on a signed quotation it \
         is a silent wrong edit, and he would find it in a document he had already sent"
    );
    assert!(
        after.contains(RUN),
        "and the other one is still there, uncorrected — the two halves of the same defect"
    );
}
