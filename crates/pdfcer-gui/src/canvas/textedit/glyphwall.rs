//! # `canvas::textedit::glyphwall` — his typo, the pin that was stopping it, and
//! the occurrence count that makes dropping the pin safe
//!
//! `OPERATOR_REQUESTS.md` **O142**. The operator, 2026-09-05:
//!
//! > *"on page 2 there is a spelling mistake — clien instead of client. if I try
//! > to edit the edit is not accepted."*
//!
//! This module is [`super::facewall`]'s sibling and is built on the same
//! principle: **the claim that matters is executable, or it rots.** `facewall`
//! held a limitation and went red the day the engine lifted it. This one holds a
//! *capability* and a *guard*, and the guard is the half that would otherwise be
//! untestable in the only direction that matters.
//!
//! ## ★★★ What was actually wrong, which is not what it looked like
//!
//! His producer writes **one glyph per show operator**: a thirty-six character
//! line is thirty-six `Tj`s, stepped along one row by x-only `Td`s. `Pass 256.0`
//! taught `edit_text` to match a `find` across exactly that shape, and the
//! engine measured the correction on his own file:
//!
//! ```text
//! "clien"->"client"  operators_spanned=5  followers_repositioned=4
//! ```
//!
//! **That capability was in this shell's pin, and his typo still failed.** The
//! standing diagnosis was that the shell sends the *whole run* as `find` and
//! that `text_extract` synthesises the spaces inside it, so the string named
//! characters no operator ever wrote and no matcher could ever reach.
//!
//! ⇒ **Measured on his own file, that diagnosis is false.** One `EditSession`
//! per shape, page 2, the run he reported:
//!
//! | request | result |
//! |---|---|
//! | whole-run `find` **+ pin** — what this shell sent | `NotFound` |
//! | whole-run `find`, **no pin** | **OK**, `operators_spanned=36` |
//! | `"clien"` **+ pin** | `NotFound` |
//! | `"clien"`, **no pin** | **OK**, `operators_spanned=5` |
//!
//! The whole-run `find` — thirty-six characters, spaces and all — matches
//! perfectly once the pin is off. Thirty-six characters, thirty-six operators:
//! **the spaces are in the operators.** The synthesised-space case is real and
//! documented (a trace of one of his CAD title-block cells showed twenty-one),
//! but it belongs to a different producer and was not what stopped this.
//!
//! **The pin was.** `Pass 256.0`'s contract carries one clause that decides
//! everything here — *"a pinned request never spans"* — so a request carrying
//! both a `find` and a `pinned_span` is confined to the single operator the pin
//! names, and on his line that operator holds one character. A thirty-six
//! character `find` cannot match inside it. The engine was answering the
//! question it had been asked, correctly, every time.
//!
//! ## ★★★ Why the fix is not simply "drop the pin"
//!
//! Because the pin is the **only** thing `EditRequest` carries that can choose
//! between two identical strings on one page. There is no occurrence index on
//! the request; `pinned_span` is the whole of its disambiguation. Dropping it on
//! a page where the text occurs twice hands the choice to the engine's
//! left-to-right scan, which takes the first — and the document this was
//! reported against is a **signed quotation**. Editing the wrong occurrence of a
//! word on one of those is not a defect the operator reports; it is one he finds
//! later, in a document he has already sent.
//!
//! So the pin comes off only when [`super::page_occurrences`] says the text
//! appears exactly once, and that function's header carries the argument for why
//! a count over extracted text is a **conservative** stand-in for a count over
//! operator text: every way the two can differ pushes the count up, so `n == 1`
//! means the engine has at most one candidate. The proxy can refuse a safe edit;
//! it cannot license an unsafe one.
//!
//! ## ★★★ The two fixtures, and why the second one is the important one
//!
//! Both are authored by `tools/gen-per-glyph-fixtures.py` with **uncompressed**
//! content streams, so `grep` answers *"how many show operators hold this
//! line?"* without running the program under test.
//!
//! | fixture | shape | what it holds down |
//! |---|---|---|
//! | `per-glyph-operators.pdf` | one per-glyph run `ABC`, unique on the page | the pin comes off and the edit **lands** |
//! | `per-glyph-twice.pdf` | the **same** per-glyph run `ABC`, twice | the pin stays on and the edit is **refused** |
//!
//! ⚠ **Without the second, the guard is untestable in the only direction that
//! matters.** A build that dropped the pin unconditionally would satisfy every
//! assertion made against the first fixture, for ever, while silently editing
//! the wrong occurrence. And that build is not hypothetical — it is the obvious
//! simplification of this code, and the one a future session will reach for on
//! seeing a count it thinks is redundant.
//!
//! ★★ [`the_engine_would_have_edited_the_wrong_one`] is what makes that concrete:
//! it asserts, against the engine directly, that an unpinned request on
//! `per-glyph-twice.pdf` **succeeds** — so the refusal below is a decision this
//! shell took, not a limitation it inherited. Without it a reader could believe
//! the guard was decoration over something the engine would have refused anyway.

#![cfg(test)]
// ---------------------------------------------------------------------------
// ★ A deliberate duplicate of the `#![cfg(test)]` above, for
// `tools/gates/check-ui-strings.sh` rather than for rustc — the same device
// `proof.rs` and `facewall.rs` use and for the same reason: the gate reads
// modules line by line and cannot see an inner attribute at the top of a file
// it is scanning for bare string literals.
// ---------------------------------------------------------------------------
#![cfg(test)]

use pdfcer_core::document::Document;
use pdfcer_core::edit::EditSession;
use pdfcer_core::text_edit::{EditOptions, EditRequest};

use crate::text::textedit::EditRefusal;

/// The per-glyph run both fixtures draw, and the correction made to it.
///
/// Three characters rather than his thirty-six, because the property under test
/// is *"more than one show operator"* and three is the smallest number that also
/// exercises a middle operator — an edit that spans only the first and last
/// would pass on a matcher that never looked between them.
const RUN: &str = "ABC";
const FIXED: &str = "ABCD";

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

/// ★★★ **The control that makes every assertion below evidence: the fixture's
/// run really is split across operators.**
///
/// If it were one operator the exact-pin path would be taken, the `find` would
/// be dropped, and every test here would be measuring the branch that already
/// worked — passing, and about nothing. This is the same hazard `facewall`'s
/// first control covers, and it is worth the four lines for the same reason.
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

/// ★★★ **HIS TYPO. The pin comes off on a unique run, and the correction
/// lands.**
///
/// Driven through the real [`super::plan`] rather than by hand-building an
/// `EditRequest`, because the claim is about **what the shell decides**. A test
/// that assembled the request itself would pass on a build where `plan` had gone
/// back to sending the pin, which is precisely the build this exists to catch.
#[test]
fn a_typo_in_a_run_written_one_glyph_at_a_time_can_be_corrected() {
    let doc = crate::app::state::open_local_fixture(UNIQUE);
    let planned = super::plan(&doc, 0, 0, RUN, FIXED);

    assert_eq!(
        planned.occurrences,
        Some(1),
        "the run occurs once on this page, and that count is what licenses dropping the pin"
    );
    assert!(
        planned.request.pinned_span.is_none(),
        "★★★ THE WHOLE FIX. `Pass 256.0`: a pinned request never spans, so a `find` sent \
         beside a pin is confined to one operator — which on a per-glyph run holds one \
         character. The pin must come OFF for the cross-operator matcher to run"
    );
    assert_eq!(
        planned.request.find, RUN,
        "and the `find` must survive: with the pin gone it is the only thing locating the run"
    );

    let mut session = session(UNIQUE);
    // ★ Sampled BEFORE the edit, from the same session the edit runs in, so the
    // comparison below is against this document rather than against a number
    // written down when the fixture was authored.
    let before_left = left_edge(&session);
    let report = session
        .edit_text(&planned.request, &planned.options)
        .expect("the correction must reach the document — this is O142");
    assert!(
        report.operators_spanned > 1,
        "★★ it must LAND BY SPANNING, not by finding one operator that happened to hold the \
         whole run. `operators_spanned` was {}; a build where the fixture stopped being \
         per-glyph would satisfy every other assertion here while testing nothing",
        report.operators_spanned
    );
    assert!(
        page_text(&session).contains(FIXED),
        "and the corrected text must be IN the page, not merely un-refused: a verb that \
         returns Ok and leaves the run alone is what this assertion exists to catch"
    );

    // ★★★ **AND THE LINE MUST NOT MOVE.** This is the assertion that nearly did
    // not get written, and the reason it exists is worth the paragraph.
    //
    // Spanning operators means the engine puts the replacement into the operator
    // holding the match's END and empties the ones before it (each kept as
    // `() Tj` so the producer's positioning chain survives). On his own file
    // that edit's disclosure reports the tail re-spaced by a net advance of
    // **-437.080 pt** — which reads exactly like a line being flung across the
    // page, and would be a far worse defect than the typo it fixed.
    //
    // ⇒ It was measured rather than reasoned about, and it is bookkeeping over
    // the collapsed operators rather than a displacement: on his file the line's
    // left edge moved by **0.042 pt**, about fifteen microns. But *"we measured
    // it once and it was fine"* is precisely the claim this project has learnt
    // rots quietly, so the bound is held here instead — the day a change to the
    // spanning path starts moving lines, it is a red test and not a report
    // from him.
    //
    // ★ The tolerance is 0.5 pt: loose enough that a legitimate re-spacing of
    // sub-point size never trips it, tight enough that no displacement a reader
    // could see can pass. A whole-operator shift on this fixture is 7 pt.
    let moved = left_edge(&session) - before_left;
    assert!(
        moved.abs() < 0.5,
        "★★★ THE CORRECTION MOVED THE LINE. Its left edge shifted by {moved:.3} pt, and an \
         edit that relocates the text it corrects is a worse defect than the typo. The \
         engine puts the replacement in the operator holding the match's END and empties \
         the earlier ones; if that stops compensating, this is where it shows"
    );
}

/// The x of the leftmost extracted run on page 0 — the line's left edge, and the
/// one number that says whether a spanning edit left the text where the producer
/// put it.
fn left_edge(session: &EditSession) -> f64 {
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
        .iter()
        .filter_map(|r| r.bbox.map(|b| b.llx))
        .fold(f64::INFINITY, f64::min)
}

/// ★★★ **THE GUARD. Two identical runs on one page, and the shell refuses
/// rather than choosing.**
///
/// This is the test that stops the fix above from becoming a defect. It asserts
/// three separate things, and each has its own way of going missing:
///
/// 1. the count **saw** both occurrences;
/// 2. the pin was **kept**, which is what makes the request unmatchable on
///    purpose;
/// 3. the resulting refusal classifies as [`EditRefusal::AmbiguousOnThePage`]
///    and not as [`EditRefusal::SplitAcrossPieces`] — both are true of this page
///    and only the first is what stopped it.
#[test]
fn a_typo_that_appears_twice_on_the_page_is_refused_rather_than_guessed() {
    let doc = crate::app::state::open_local_fixture(TWICE);
    let planned = super::plan(&doc, 0, 0, RUN, FIXED);

    assert_eq!(
        planned.occurrences,
        Some(2),
        "★★★ the count is the guard. If this reads Some(1) the shell is about to drop the \
         pin on a page holding two candidates and let the engine pick one"
    );
    assert!(
        planned.request.pinned_span.is_some(),
        "★★★ THE PIN MUST STAY ON. It is the only disambiguator `EditRequest` carries — \
         there is no occurrence index — so keeping it is how this shell declines to guess. \
         The request is unmatchable ON PURPOSE"
    );

    let mut session = session(TWICE);
    let error = session
        .edit_text(&planned.request, &planned.options)
        .expect_err("a pinned request cannot span, so this must be refused");

    // ★★ The classification, which is what the operator actually reads. Both
    // facts are true of this page — the run IS split and the text DOES appear
    // twice — and reporting the split would be this shell explaining its own
    // refusal with somebody else's reason.
    // ★ `RefusalClass` is the extension trait that puts `refusal_kind` on
    // `EditError`; imported here rather than at the module head because this is
    // the only test that classifies, and the import is part of what the
    // assertion below is about.
    use pdfcer_core::text_edit::RefusalClass;
    let why = EditRefusal::of(
        error.refusal_kind(),
        planned.one_operator,
        None,
        planned.occurrences,
    );
    assert_eq!(
        why.name(),
        "AmbiguousOnThePage",
        "★★★ the sentence must name the thing that ACTUALLY stopped it. \
         `SplitAcrossPieces` is also true here and would tell him pdfcer cannot edit this \
         kind of text — which is now false, and would send him away from a document pdfcer \
         can in fact correct once he selects more of the line"
    );
    assert!(
        why.line().contains('2'),
        "and the count reaches him: \"appears twice\" is a page he can look at, and the \
         sentence interpolates the number for exactly that reason. Got: {}",
        why.line()
    );
}

/// ★★★ **The guard is a DECISION, not an inherited limitation** — asserted
/// against the engine directly, with no `plan` in the way.
///
/// Without this, a reader could believe `per-glyph-twice.pdf` refuses because
/// the engine refuses it, which would make the test above vacuous and the count
/// decoration. It does not: **unpinned, the engine applies the edit happily**,
/// to whichever occurrence it reaches first.
///
/// ⚠ That is the build this module exists to prevent shipping, and this test is
/// the closest thing to it that can be safely written down: it demonstrates the
/// wrong behaviour on a throwaway session, so that nobody has to wonder what
/// would happen if the pin were dropped unconditionally.
#[test]
fn the_engine_would_have_edited_the_wrong_one() {
    let mut session = session(TWICE);
    let report = session
        .edit_text(
            &EditRequest::find_replace(0, RUN, FIXED),
            &EditOptions::default(),
        )
        .expect(
            "★★★ UNPINNED, THE ENGINE ACCEPTS THIS. If it ever refuses, the engine has \
             gained an ambiguity check of its own and this shell's count may be able to \
             retire — read its refusal before deleting anything",
        );
    assert!(report.operators_spanned > 1, "and it spans, as on his file");

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
