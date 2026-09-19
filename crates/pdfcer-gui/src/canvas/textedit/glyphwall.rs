//! # `canvas::textedit::glyphwall` — his typo, the pin that was stopping it, and
//! the occurrence count that makes dropping the pin safe
//!
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
//! So the pin does not come off at all. [`EditRequest::spanning_from`] starts the
//! span search **at the pinned operator** rather than at the first operator on
//! the page, with every other guard unchanged: `find` says *what*, the pin says
//! *which one*, and the run reached is the one the caret is in.
//!
//! ⚠ Counting the occurrences and dropping the pin when the text is unique is
//! **not** a weaker version of this and must not be reintroduced as a fallback.
//! `find_anchor` tries a **single-operator** match across the whole page before
//! the spanning search runs at all, so a single-operator twin anywhere on the
//! sheet beats a spanning occurrence above it — dropping the pin can make the
//! clicked run *unreachable*, not merely ambiguous. [`super::Plan::occurrences`]
//! carries the engine's own ruling.
//!
//! ## ★★★ The two fixtures, and why the second one is the important one
//!
//! Both are authored by `tools/gen-per-glyph-fixtures.py` with **uncompressed**
//! content streams, so `grep` answers *"how many show operators hold this
//! line?"* without running the program under test.
//!
//! | fixture | shape | what it holds down |
//! |---|---|---|
//! | `per-glyph-operators.pdf` | one per-glyph run `ABC`, unique on the page | the pin and the flag go out together and the edit **lands** |
//! | `per-glyph-twice.pdf` | the **same** per-glyph run `ABC`, twice | the **clicked** occurrence changes and the other does not |
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
//! `per-glyph-twice.pdf` **succeeds**, on whichever occurrence it reaches first.
//! So the pin below is choosing between two edits the engine would both have
//! made — without this a reader could believe it was decoration over something
//! the engine disambiguated anyway.

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

/// The per-glyph run both fixtures draw, and the correction made to it.
///
/// Three characters rather than his thirty-six, because the property under test
/// is *"more than one show operator"* and three is the smallest number that also
/// exercises a middle operator — an edit that spans only the first and last
/// would pass on a matcher that never looked between them.
const RUN: &str = "ABC";
const FIXED: &str = "ABCD";

/// A correction to the **same** run that changes its first and last characters
/// and leaves the middle one alone.
///
/// ★★★ The common-prefix/common-suffix walk in [`super::narrow`] therefore
/// reports the whole run as changed, three operators are touched, and
/// [`super::narrow::narrow`] answers `None` — which is the only way to reach the
/// spanning form deliberately now that the single-operator change narrows. See
/// [`a_change_that_straddles_operators_keeps_the_spanning_form`].
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
///
/// ★ Separate from [`page_text`] and not a convenience: with two identical
/// strings on one page, *which* one changed is expressible only as a position
/// in this list. A concatenated string can say the fix is present; it cannot
/// say the clicked run is the one that has it, and that is the whole question
/// `span_from_pin` exists to answer.
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

/// ★★★ **HIS TYPO. The pin stays on, the span search starts at it, and the
/// correction lands.**
///
/// Driven through the real [`super::plan`] rather than by hand-building an
/// `EditRequest`, because the claim is about **what the shell decides**. A test
/// that assembled the request itself would pass on a build where `plan` had gone
/// back to sending the pin, which is precisely the build this exists to catch.
#[test]
fn a_typo_in_a_run_written_one_glyph_at_a_time_can_be_corrected() {
    let doc = crate::app::state::open_local_fixture(UNIQUE);
    let planned = super::plan(&doc, 0, 0, RUN, FIXED);

    // ★★★ **THE PIN STAYS ON**, and the obvious simplification here is to
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
    // ★★★ **AND IT IS NARROWED**, which is the shape that keeps the line where
    // the producer put it — [`super::narrow`], engine request `G028`.
    //
    // The `D` is appended inside the run's last show operator, so that operator
    // alone is rewritten and the request goes out in the whole-operator form:
    // pin set, `find` empty, `span_from_pin` off. Nothing spans, so nothing is
    // emptied and there is nothing for the engine to compensate.
    //
    // ⚠ This asserts the exact shape rather than *"narrowed or spanning"*,
    // because a disjunction over two mechanisms asserts neither of them. The
    // spanning form is still reachable and is still held down — by
    // [`a_change_that_straddles_operators_keeps_the_spanning_form`], which names
    // its own shape just as exactly.
    assert!(
        planned.narrowed,
        "★★★ THE REQUEST MUST BE NARROWED TO ONE OPERATOR. `{RUN}`→`{FIXED}` appends inside \
         the run's last show operator, and a request that spans instead is redrawn from that \
         operator's own origin — measured at +240.16 pt on his own sheet. Engine request \
         `G028`"
    );
    assert!(
        !planned.request.span_from_pin,
        "★★ …and a narrowed request must NOT also ask to span. The two are alternatives: \
         `span_from_pin` lets the match run on past the pinned operator, which is the very \
         collapse the narrowing exists to avoid"
    );
    assert!(
        planned.request.find.is_empty(),
        "★★ …and its `find` must be empty, which is what makes the pin mean *this whole show \
         operator*. A `find` sent beside a pin is confined to that one operator and, on a \
         per-glyph run, cannot match. Got `{}`",
        planned.request.find
    );
    assert_eq!(
        planned.request.replace, "CD",
        "★★★ …and the replacement must be that operator's WHOLE new text, not the changed \
         character. The whole-operator form replaces everything the pinned operator shows, \
         so sending only `D` deletes the `C`"
    );

    let mut session = session(UNIQUE);
    // ★ Sampled BEFORE the edit, from the same session the edit runs in, so the
    // comparison below is against this document rather than against a number
    // written down when the fixture was authored.
    let before_left = left_edge(&session, 0);
    let report = session
        .edit_text(&planned.request, &planned.options)
        .expect("the correction must reach the document — this is O142");
    assert_eq!(
        report.operators_spanned, 1,
        "★★★ and the engine must agree it touched ONE operator. `operators_spanned` above 1 \
         means the narrowing was computed and then not obeyed — the request that went out is \
         not the request that was planned — and the placement defect `G028` describes is back"
    );
    assert!(
        page_text(&session).contains(FIXED),
        "and the corrected text must be IN the page, not merely un-refused: a verb that \
         returns Ok and leaves the run alone is what this assertion exists to catch"
    );

    // ★★★ **AND THE LINE MUST NOT MOVE** — the operator's own report, O213:
    //
    // > *"the entire line shifts to the right after instead of staying in
    // > place."*
    //
    // A spanning edit puts the replacement into the operator holding the match's
    // END and empties the ones before it, each kept as `() Tj` so the producer's
    // positioning chain survives. Where the engine's compensation for that
    // collapse runs, the line holds; where it does not, the line is redrawn from
    // its final fragment's origin. Both were measured, with the same request
    // shape:
    //
    // | | left edge, after | `followers_repositioned` |
    // |---|---|---|
    // | this fixture | held to the last decimal | 2 |
    // | his page-1 title-block line | moved **+240.16 pt** | 0 |
    //
    // ⇒ **The fixture is the case where it works**, which is exactly why a bound
    // held only here would have gone on passing through the defect he reported.
    // Engine request `G028` carries the reproduction; [`super::narrow`] is the
    // shell's answer, and the narrowing asserted above is what this measures.
    //
    // ★ The tolerance is 0.5 pt: loose enough that a legitimate re-spacing of
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

/// ★★★ **THE SPANNING FORM, KEPT UNDER TEST** — a change that touches the run's
/// first and last operators and cannot be narrowed to either.
///
/// [`super::narrow`] answers `None` whenever the changed region touches two show
/// operators, and the request then goes out as it always did: pin on, whole-run
/// `find`, `span_from_pin` set. That path still ships, still reaches runs no
/// other shape can, and would otherwise have stopped being exercised the day the
/// single-operator case started narrowing — a mechanism with no test, in a file
/// whose other tests are green.
///
/// ⚠ The placement caveat is real and is not asserted away here: this is the
/// shape `G028` describes, and on his own sheet it moves the line. What is
/// asserted is that the shell still *reaches* the run and still addresses
/// **this** occurrence. Placement on a straddling change is the engine's to fix.
#[test]
fn a_change_that_straddles_operators_keeps_the_spanning_form() {
    let doc = crate::app::state::open_local_fixture(UNIQUE);
    let planned = super::plan(&doc, 0, 0, RUN, STRADDLED);

    assert!(
        !planned.narrowed,
        "★★★ `{RUN}`→`{STRADDLED}` changes the first and last characters, so the changed \
         region covers all three show operators and no single one of them can carry it. A \
         build that narrowed here would be picking an operator to put the text in"
    );
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
///
/// ★★★ It names a run, and that is the whole point. The minimum `llx` over
/// *every* run on the page answers a different question — *"is anything on this
/// page still at the far left?"* — and on a sheet with a title block something
/// always is, so an edit could fling the corrected line across the page while
/// that number did not move at all. The fixtures here hold one and two runs, so
/// both readings agree on them and the defect would have shipped invisibly.
///
/// Panics when the run has no `bbox`, which is the honest outcome: a run whose
/// box could not be derived cannot answer the question, and `f64::INFINITY`
/// folded in silently would make the comparison pass.
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

/// ★★★ **THE GUARD, INVERTED 2026-09-08: two identical runs on one page, and
/// the shell now edits THE ONE THAT WAS CLICKED.**
///
/// # What this test asserted until today, and why it was right then
///
/// It asserted a **refusal**. Three things, each with its own way of going
/// missing: that the count saw both occurrences, that the pin was kept — which
/// made the request unmatchable *on purpose* — and that the refusal classified
/// as `AmbiguousOnThePage` rather than as [`EditRefusal::SplitAcrossPieces`],
/// both being true of this page and only the first being what stopped it.
///
/// That was the best available answer while `EditRequest` carried no way to
/// say *which* occurrence. The shell had exactly two options — address this
/// run **or** span across operators — and it chose to refuse rather than to
/// guess on a signed drawing.
///
/// # What changed
///
/// `Pass 272.0` gave it a third:
/// [`EditRequest::spanning_from`](pdfcer_core::text_edit::EditRequest::spanning_from)
/// starts the span search **at the pinned operator**. `find` says what, the
/// pin says which one, and every guard the span search already had is
/// unchanged. Shipped the same day this shell filed for it.
///
/// ⇒ So the page that could not be edited is now edited **correctly**, and
/// this test proves it by the only means that discriminates: it checks that
/// the *first* occurrence changed and the *second* did not. A build that
/// scanned from operator 0 would also produce a page containing the fix and
/// would pass any assertion phrased as "the corrected text is present".
///
/// ★ That trap is not hypothetical — the engine's own reply records its third
/// sabotage passing twice, once because the fixture lacked a third
/// occurrence and once because the assertion asked *"does this operator appear
/// somewhere"* rather than naming the line.
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
        planned.narrowed,
        "★★★ …and the request must be narrowed to the one show operator the change touched. \
         `{RUN}`→`{FIXED}` appends inside the run's last operator; a request that spans \
         instead is redrawn from that operator's own origin. Engine request `G028`, and \
         `a_change_that_straddles_operators_keeps_the_spanning_form` is where the spanning \
         shape is held instead"
    );
    assert!(
        planned.request.find.is_empty(),
        "★★ …with an empty `find`, which is what makes the pin mean *this whole show \
         operator* rather than *this string inside it*"
    );
    assert!(
        !planned.request.span_from_pin,
        "★★ …and without the spanning flag, which would let the match run on past the pinned \
         operator and reinstate the collapse the narrowing exists to avoid"
    );

    let mut session = session(TWICE);
    let before = page_runs(&session);
    assert!(
        before.iter().filter(|r| r.trim() == RUN).count() >= 2,
        "★ the control: this fixture must really hold TWO identical runs, or this test is \
         measuring the unique case and asserts nothing about disambiguation. Regenerate \
         with `python tools/gen-per-glyph-fixtures.py`. Got: {before:?}"
    );

    // ★★★ The control that the fixture is really per-glyph is
    // [`the_fixtures_runs_are_written_one_glyph_per_operator`], which reads
    // `Plan::one_operator` over both fixtures. It cannot be `operators_spanned`
    // any more: a narrowed request touches exactly one operator **by design**,
    // so that number no longer distinguishes a split run from a whole one.
    assert!(
        !planned.one_operator,
        "★ the control: the clicked run must really be written across several show \
         operators, or this test is measuring the shape that never had the defect"
    );
    session
        .edit_text(&planned.request, &planned.options)
        .expect("★★★ the edit that was refused until Pass 272.0 must now land");

    // ★★★ THE ASSERTION THAT DISCRIMINATES. Not "the page contains ABCD" —
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

    // ★ And it must be the FIRST — the run `plan` was given (index 0). Order
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
