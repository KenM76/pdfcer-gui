//! # `canvas::textedit::wordwall` — editing lines a word processor wrote as
//! many text objects
//!
//! Each line of `fixtures/word-fragmented-lines.pdf` is several `BT … ET`
//! objects. The whole-line request matches nothing there; these tests pin that
//! the narrowed fallback lands an edit touching one object and that an edit
//! reaching across objects is refused as a split, not as a moved line.

#![cfg(test)]
#![cfg(test)]

use pdfcer_core::document::Document;
use pdfcer_core::edit::EditSession;

use super::tier::Tier;

const FIXTURE: &str = "word-fragmented-lines.pdf";

fn session() -> EditSession {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(FIXTURE);
    assert!(
        path.exists(),
        "regenerate with `python fixtures/word-fragmented-lines.PROVENANCE.py`"
    );
    EditSession::new(Document::load(&path).expect("the fixture loads"))
}

fn page_text(session: &EditSession) -> String {
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
    .map(|r| r.text.as_str())
    .collect::<Vec<_>>()
    .join("|")
}

/// Plans `original` → `draft` on run `run`, commits it to a fresh session,
/// and answers whether it landed, on which tier, whether that tier reached one
/// operator, and the page text after.
fn commit(run: usize, original: &str, draft: &str) -> (bool, Tier, bool, String) {
    let doc = crate::app::state::open_local_fixture(FIXTURE);
    let planned = super::plan(&doc, 0, run, original, draft);
    let mut session = session();
    let (result, tier) = planned.attempt("test", |r| session.edit_text(r, &planned.options));
    let one = planned.reached_one_operator(tier);
    (result.is_ok(), tier, one, page_text(&session))
}

const DATE: &str = "Date Premises Required____ ";
const LAW: &str = "(   ) Common-Law ";
const NAME: &str = "Applicant\u{2019}s Name____ ";

/// The control: every line really is several operators, and the request for
/// the whole line really is refused — or the tests below prove nothing.
#[test]
fn every_line_is_several_operators_and_the_line_request_is_refused() {
    let doc = crate::app::state::open_local_fixture(FIXTURE);
    let text = doc.provenance_page_text(0).expect("the page's text");
    for (run, line, parts) in [(0, DATE, 2), (2, LAW, 4), (4, NAME, 4)] {
        assert_eq!(text.runs[run].text, line, "regenerate the fixture");
        let ops = super::tier::operator_ranges(&text.runs[run]).expect("provenance");
        assert_eq!(ops.len(), parts, "run {run} must be {parts} operators");
        let planned = super::plan(&doc, 0, run, line, &line.replacen(' ', "  ", 1));
        let refused = session().edit_text(&planned.request, &planned.options);
        assert!(
            refused.is_err(),
            "the whole-line request for run {run} must fail"
        );
    }
}

#[test]
fn typing_inside_a_fragment_lands_through_the_narrowed_request() {
    let (landed, tier, one, page) = commit(0, DATE, "Date Premises Required_____ ");
    assert!(landed, "the edit must land");
    assert_eq!(tier, Tier::Narrowed);
    assert!(one, "it reaches one operator");
    assert!(
        page.contains("Required_____"),
        "and the page holds it: {page}"
    );
}

#[test]
fn a_word_split_across_objects_edits_the_piece_that_changed() {
    let (landed, tier, _, page) = commit(2, LAW, "(   ) Common-Laws ");
    assert!(
        landed && tier == Tier::Narrowed,
        "landed={landed} tier={tier:?}"
    );
    assert!(page.contains("Laws"), "{page}");
}

#[test]
fn the_text_after_a_second_font_run_edits() {
    let (landed, tier, _, page) = commit(4, NAME, "Applicant\u{2019}s Names____ ");
    assert!(
        landed && tier == Tier::Narrowed,
        "landed={landed} tier={tier:?}"
    );
    assert!(page.contains("Names____"), "{page}");
}

/// An edit reaching across two text objects is refused, and refused as a
/// split — the classifier's input says more than one operator was reached.
#[test]
fn a_change_across_two_objects_is_refused_as_a_split() {
    let (landed, tier, one, page) = commit(2, LAW, "(   ) CommonXaw ");
    assert!(
        !landed,
        "the engine cannot match across text objects yet: {page}"
    );
    assert_eq!(tier, Tier::Narrowed);
    assert!(
        !one,
        "the refusal must be classified as reaching several pieces"
    );
}
