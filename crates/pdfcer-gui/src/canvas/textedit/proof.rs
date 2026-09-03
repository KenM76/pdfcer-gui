//! # `canvas::textedit::proof` — **the tail did not move, proved in the bytes**
//!
//! `crate::redact::proof`'s shape applied to `DEFECTS.md` **D4b**: a claim about
//! what an edit does to a *file*, asserted against the file, with the falsifying
//! run beside it.
//!
//! ## What is being proved, and why nothing else in the suite proves it
//!
//! D4b says two things are wrong on commit, and both are about **text the
//! operator did not touch**:
//!
//! 1. a right-aligned / centred / justified tail is pushed off the edge it is
//!    flush against;
//! 2. a rotated line's tail is displaced along user-space x, which is not the
//!    direction its baseline runs.
//!
//! Neither is visible in any assertion about the edit itself. The edited run
//! comes out correct in both builds — the right glyphs, the right place, the
//! right font. What differs is a `Tm` belonging to something else, twenty bytes
//! further down the content stream, and the only oracle that can see it is the
//! stream.
//!
//! ## ★ The falsifying run is the point of the module
//!
//! Every assertion here is made **twice**: once through
//! [`super::disposition::options`], which is the shipped decision, and once
//! through `EditOptions::default()`, which is **verbatim what the old shell
//! passed at its only call site**. The second run is not decoration — it is what
//! makes the first one evidence:
//!
//! * a build with the fix reverted produces exactly `EditOptions::default()`, so
//!   the falsifying run *is* the broken build, executed;
//! * a fixture that did not actually exercise the defect would make both runs
//!   agree, and the `assert_ne!` between them would fail — so this cannot pass
//!   by flattering the thing it measures, which is `HANDOFF.md` §10's
//!   ink-simplification lesson in a different suit.
//!
//! ## Why the fixture's content stream is uncompressed
//!
//! Because the oracle is a byte scan for a `Tm` operand triple, and a
//! Flate-compressed stream would answer "absent" for a correct build and a
//! broken one alike — a false pass in the only direction that matters. See
//! `tools/gen-textedit-fixtures.py`, which says the same thing from the
//! producing side.
//!
//! ## Where the *second process* proof lives
//!
//! Not here. This module proves the arithmetic against the written bytes in one
//! process; `tools/ui-verify`'s `text_edit_pins_an_aligned_tail` drives the real
//! binary, saves a copy through the real command, and re-opens it in a second
//! process — which is the only honest proof that an edit reached the bytes *by
//! the route an operator takes*. The two answer different questions and neither
//! substitutes for the other.

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
use std::path::PathBuf;

use pdfcer_core::document::Document;
use pdfcer_core::edit::EditSession;
use pdfcer_core::text_edit::{
    BlockRecognitionOptions, EditOptions, EditRequest, EditableTextModel, FollowerDisposition,
    GlyphRef, ReflowEngine, TextPosition, reflow_recognition_options,
};

use super::disposition::{self, Reason};

/// The fixture this module is written against.
///
/// In **this** repository's `fixtures/`, not the engine's, and it is the one
/// fixture in either tree that carries right-aligned and rotated text — see
/// `DEFECTS.md` D4's closing section, which is an inventory of the conditions
/// every existing fixture omits by construction.
fn fixture() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tail-alignment.pdf");
    assert!(
        p.exists(),
        "the fixture is missing at {}. Regenerate it: python tools/gen-textedit-fixtures.py",
        p.display()
    );
    p
}

/// Open the fixture as a fresh session.
fn session() -> (EditSession, Vec<u8>) {
    let path = fixture();
    let base = std::fs::read(&path).expect("the fixture reads");
    let doc = Document::load(&path).expect("the fixture loads");
    (EditSession::new(doc), base)
}

/// The index of the run whose text is exactly `needle`, and the matrices in
/// force at its first glyph.
///
/// Panics rather than returning an `Option`: a fixture that stopped containing
/// its own text would otherwise make every test below pass vacuously, which is
/// the failure mode `run-all.sh`'s three-state model exists to refuse.
fn extract(session: &EditSession) -> pdfcer_core::text_extract::PageText {
    let view = session.view();
    let pages = pdfcer_core::page_tree::pages_in(&view).expect("a page tree");
    pdfcer_core::text_extract::extract_page_view(
        &view,
        &pages[0],
        0,
        &pdfcer_core::text_extract::ExtractOptions::default().with_provenance(true),
    )
    .expect("the page's text extracts")
}

fn run_of(session: &EditSession, needle: &str) -> (usize, [f32; 6], [f32; 6]) {
    let text = extract(session);
    let idx = text
        .runs
        .iter()
        .position(|r| r.text.trim() == needle)
        .unwrap_or_else(|| {
            panic!(
                "the fixture no longer contains a run reading {needle:?}; it holds {:?}",
                text.runs.iter().map(|r| r.text.clone()).collect::<Vec<_>>()
            )
        });
    let p = text.runs[idx].glyphs[0]
        .provenance
        .as_ref()
        .expect("provenance was requested");
    (idx, p.text_matrix, p.ctm)
}

/// The index and matrices of the first run whose text matrix is **rotated**.
///
/// ★ **Why the rotated line cannot be found by its text, and this is a real
/// finding rather than a test detail.** `extract_page_view`'s line clustering
/// groups glyphs by horizontal proximity, so a line whose baseline runs *up* the
/// page is not clustered at all: `TITLE VERTICAL` comes back as the fifteen runs
/// `"T"`, `"IT"`, `"L"`, `"E"`, `" V"`, … one or two glyphs each.
///
/// Two consequences, both true of the shipped shell and neither a problem:
///
/// * a caret placed on rotated text lands in a **fragment**, so the edit replaces
///   that fragment. It is still the right operator, because the provenance pin
///   identifies the show operator rather than the run, and the fragment is by
///   construction a substring of that operator's decoded text.
/// * this helper has to ask the *geometry* rather than the text, which is what
///   `disposition::choose` reads anyway.
fn rotated_run(session: &EditSession) -> (usize, [f32; 6], [f32; 6]) {
    let text = extract(session);
    for (i, r) in text.runs.iter().enumerate() {
        if let Some(p) = r.glyphs.first().and_then(|g| g.provenance.as_ref())
            && p.text_matrix[1].abs() > 1e-6
        {
            return (i, p.text_matrix, p.ctm);
        }
    }
    panic!("the fixture no longer carries a rotated run");
}

/// The [`Reason`] the shipped rule reaches for the run reading `needle`.
///
/// This is [`super::plan`]'s body with the document plumbing removed — the same
/// three derivations in the same order, against the same engine calls — because
/// `plan` needs an `OpenDoc` and the point here is the arithmetic.
fn reason_for(session: &EditSession, needle: &str) -> Reason {
    let text = extract(session);
    let (run, tm, ctm) = run_of(session, needle);
    // ★ The multi-run test, derived exactly as `plan` derives it: the DEFAULT
    // recognition, because the question is how the thing the operator clicked
    // was segmented, and the relaxed model below answers a different question
    // about the same page.
    let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
    let shares = model
        .line_range_at(TextPosition::new(run, 0))
        .is_some_and(|(from, to)| from.run != to.run);
    let relaxed = EditableTextModel::recognize(&text, &reflow_recognition_options());
    let finding = relaxed
        .block_at(TextPosition::new(run, 0))
        .and_then(|b| ReflowEngine::new(&relaxed).detect_alignment(b).ok())
        .map(disposition::from_detection);
    disposition::choose(tm, ctm, shares, finding)
}

/// Edit `find` to `replace` under `opts` and return the **appended** bytes — the
/// incremental update's own revision, and only it.
///
/// The base revision is excluded deliberately, and it is the whole reason this
/// helper exists rather than a `std::fs::read`. §7.5.6 forbids an incremental
/// update from rewriting what came before it, so the original `Tm` is *always*
/// still present in the first `base.len()` bytes — of a correct build and a
/// broken one alike. A scan over the whole file would therefore answer "the tail
/// is unmoved" for every build ever written. What is being asked is what the
/// **new** content object says, and that is what was appended.
fn appended_after_edit(find: &str, replace: &str, opts: &EditOptions) -> Vec<u8> {
    let (mut session, base) = session();
    let text = extract(&session);
    // The pin is set when the find string is itself a run; when it is a whole
    // operator whose runs are fragments (the rotated line), the find alone
    // locates it, because every string on this page is unique.
    let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
    let mut req = EditRequest::find_replace(0, find, replace);
    if let Some(run) = text.runs.iter().position(|r| r.text.trim() == find) {
        req.pinned_span = model
            .provenance(GlyphRef::new(run, 0))
            .map(|p| p.operator_span);
    }
    drop(text);

    session
        .edit_text(&req, opts)
        .expect("the fixture's runs are editable Helvetica");
    let (bytes, _report) = session
        .to_incremental_bytes(&pdfcer_core::writer::SaveOptions::identity())
        .expect("the session saves incrementally");
    assert!(
        bytes.len() > base.len() && bytes[..base.len()] == base[..],
        "an incremental update leaves the base revision byte-verbatim (§7.5.6)"
    );
    bytes[base.len()..].to_vec()
}

/// Whether `needle` appears in `hay`.
fn holds(hay: &[u8], needle: &str) -> bool {
    hay.windows(needle.len()).any(|w| w == needle.as_bytes())
}

// ===========================================================================
// The three findings — the rule reaching the right answer on real geometry
// ===========================================================================

/// ★★ **A right-aligned block is detected as right-aligned**, against the real
/// engine on a real page.
///
/// The unit tests in [`super::disposition`] assert what the rule does with a
/// finding; this asserts that the finding the engine actually produces for
/// right-aligned text is the one those tests assume. Without it the whole fix
/// could be correct and unreachable — which, per that module's header, is
/// exactly what happens if the *default* block recogniser is used instead of the
/// relaxed one.
#[test]
fn a_right_aligned_block_reaches_the_pin_rule() {
    let (session, _) = session();
    let reason = reason_for(&session, "REVISION B");
    assert_eq!(
        reason,
        Reason::Flush(pdfcer_core::text_edit::BlockAlignment::Right),
        "the engine must see this block as right-aligned; if it reports \
         AlignmentUndetectable the relaxed recogniser is not being used"
    );
    assert_eq!(
        disposition::options(reason).disposition,
        FollowerDisposition::Pin
    );
}

/// ★★ **Rotated text reaches the rotation guard.**
///
/// `[0 1 -1 0 e f]` — the shape a SolidWorks title block's side text has, and
/// the case `DEFECTS.md` D4b says *"bites rotated CAD title-block text
/// specifically, which is exactly this operator's documents."*
#[test]
fn a_rotated_line_reaches_the_rotation_guard() {
    let (session, _) = session();
    let (run, tm, ctm) = rotated_run(&session);
    let text = extract(&session);
    let relaxed = EditableTextModel::recognize(&text, &reflow_recognition_options());
    let finding = relaxed
        .block_at(TextPosition::new(run, 0))
        .and_then(|b| ReflowEngine::new(&relaxed).detect_alignment(b).ok())
        .map(disposition::from_detection);
    // ★ Passing the engine's real finding in, rather than `None`, is what makes
    // this an assertion about the RUNG ORDER as well as about the guard: this
    // block's alignment is whatever the recogniser makes of fifteen one-glyph
    // runs, and the answer must be `Rotated` regardless of it.
    // ★ `false` for the multi-run rung, deliberately, and it is the same kind of
    // statement the real `finding` beside it makes: this asserts that ROTATION
    // wins, so every rung below it must be given the value that would otherwise
    // answer, and `true` here would let `SharesTheLine` claim the result and the
    // test would pass while measuring the wrong rung.
    assert_eq!(
        disposition::choose(tm, ctm, false, finding),
        Reason::Rotated,
        "a [0 1 -1 0 e f] text matrix must reach the rotation guard, and it must win over whatever the alignment detector said (it said {finding:?})"
    );
}

/// ★ **Upright left-aligned text still reflows** — the selectivity control.
///
/// A build that answered `Pin` unconditionally would satisfy both tests above
/// and would not be the fix: it would freeze every line on every ordinary
/// document, so an edit that lengthened a word would overlap the next one
/// instead of pushing it along. This is the assertion that tells the fix from a
/// blanket pin, and it is the reason the fixture carries a third text object
/// that neither of the other tests looks at.
#[test]
fn upright_left_aligned_text_still_reflows() {
    let (session, _) = session();
    let reason = reason_for(&session, "PLAIN LEFT ONE");
    assert_eq!(
        disposition::options(reason).disposition,
        FollowerDisposition::Reflow,
        "left-aligned upright text must keep the engine's default; got {reason:?}"
    );
}

// ===========================================================================
// ★★ The bytes — and the falsifying run beside each one
// ===========================================================================

/// ★★★ **The right-aligned tail does not move, and under the old shell's
/// options it does.**
///
/// The fixture's three right-aligned lines share one `BT`/`ET`, so the engine's
/// reflow walk reaches lines 2 and 3 from an edit to line 1. Line 3's `Tm` is
/// `1 0 0 1 412.64 668.00 Tm` and the replacement is longer than what it
/// replaces, so:
///
/// * under the shipped rule (`Pin`) that operator is re-emitted **verbatim**;
/// * under `EditOptions::default()` — the old shell's only call site — it is
///   rewritten with `e` increased by the advance delta, and the string is gone.
///
/// # ★★★ THIS TEST CHANGED MEANING ON 2026-08-20, AND THE REASON IS A FIX
///
/// It used to end with a **falsifier**: an assertion that `EditOptions::default()`
/// — plain `Reflow`, what the old shell always passed — *does* move line 3's
/// `Tm`, proving the fixture exercised the defect. That assertion now fails,
/// and the honest response is not to delete it.
///
/// The engine's `Pass 121.1` narrowed the reflow walk. It used to shift every
/// absolute `Tm` it passed until a `Td`/`TD`/`T*` boundary; it now continues a
/// line **only through a `Tm` that differs in `e` alone** — same orientation,
/// same scale, same baseline. Lines 2 and 3 of this block sit on different
/// baselines, so reflow no longer reaches them at all.
///
/// The number that earned that change, measured on the operator's real drawing:
/// a four-character edit reported `followers_repositioned=1676` and changed
/// **34,059 pixels across the whole sheet**, because a CAD stream positions
/// everything with `Tm` and never emits the `Td` the walk was looking for.
/// After the fix the same edit changed 42 pixels inside one label.
///
/// So the control is **inverted**, and it is worth more inverted than it was
/// before: it now asserts that *the engine's own default is safe for this
/// shape*, which is precisely the property `Pass 121.1` established and
/// precisely what would break if the walk were ever loosened again. The shell's
/// `Pin` rule is now belt-and-braces here rather than the only defence — and
/// the case where it is still the only defence has its own test and its own
/// block in the fixture: see [`a_same_baseline_follower_is_the_case_pinning_still_prevents`].
#[test]
fn the_right_aligned_tail_is_left_exactly_where_it_was() {
    const TAIL: &str = "412.64 668.00 Tm";
    let fixed = appended_after_edit(
        "REVISION B",
        "REVISION BBBB",
        &disposition::options(Reason::Flush(pdfcer_core::text_edit::BlockAlignment::Right)),
    );
    let reflowed = appended_after_edit("REVISION B", "REVISION BBBB", &EditOptions::default());

    assert!(
        holds(&fixed, TAIL),
        "the shipped rule must leave the untouched line's Tm verbatim; \
         `{TAIL}` is not in the appended revision"
    );
    assert!(
        holds(&reflowed, TAIL),
        "★ THE ENGINE'S REFLOW REACHED ANOTHER BASELINE. `Pass 121.1` narrowed the walk so a \
         following Tm continues the edited line only if it differs in `e` alone, and line 3 of \
         this block differs in `f` too. If this fires, either the walk has been loosened again \
         or this build links an engine older than `bab0a23` — the revision where one \
         four-character edit moved 1,676 labels."
    );
}

/// ★★★ **The rotated line's tail does not move, and under the old shell's
/// options it slides along the wrong axis.**
///
/// The follower is `0 1 -1 0 90.00 420.00 Tm`. Its baseline runs **up** the
/// page — the text-space x unit vector maps to `(0, 1)` in user space — so a
/// text-space advance of `Δ` should displace it by `(0, Δ)`. The engine's reflow
/// branch writes `e + Δ`, i.e. `(Δ, 0)`: the right magnitude on the wrong axis.
///
/// This is the assertion that would have caught D4b case 2 in the old shell, and
/// it could not have been written there, because no fixture in either repository
/// contained rotated text.
#[test]
fn the_rotated_tail_is_not_slid_along_the_wrong_axis() {
    const TAIL: &str = "0 1 -1 0 90.00 420.00 Tm";
    // The *operator's* decoded text, not a run's: the rotated line extracts as
    // fifteen fragments (see `rotated_run`), and `EditRequest::find` matches
    // within one show operator's decoded text, which is the whole string.
    let fixed = appended_after_edit(
        "TITLE VERTICAL",
        "TITLE VERTICALLY",
        &disposition::options(Reason::Rotated),
    );
    let broken = appended_after_edit(
        "TITLE VERTICAL",
        "TITLE VERTICALLY",
        &EditOptions::default(),
    );

    assert!(
        holds(&fixed, TAIL),
        "a rotated follower must be re-emitted verbatim; `{TAIL}` is not in the \
         appended revision"
    );
    // ★ Inverted on 2026-08-20 for the same reason as its right-aligned
    // sibling, and here the engine's rule bites harder: a rotated follower
    // differs from the edited run in `a`, `b`, `c` AND `d`, so `Pass 121.1`'s
    // "differs in `e` alone" test ends the line at the first character of it.
    //
    // ★★ Note what is NOT weakened by this. The shell still answers
    // `Reason::Rotated` and still pins, and it must: the engine's rule is about
    // where a line ENDS, and this shell's is about text whose baseline does not
    // run left-to-right, where adding a scalar to `e` is the right magnitude on
    // the wrong axis. Two different guards against two different errors that
    // happened to have one victim in this fixture.
    assert!(
        holds(&broken, TAIL),
        "★ THE ENGINE'S REFLOW CROSSED AN ORIENTATION CHANGE. `Pass 121.1` ends the edited \
         line at any following Tm that differs in more than `e`, and a quarter-turn matrix \
         differs in all four of a, b, c and d. If this fires, the walk has been loosened or \
         this build links an engine older than `bab0a23`."
    );
}

/// ★★★ **The case pinning still uniquely prevents: two runs on ONE baseline.**
///
/// Block D of the fixture, added 2026-08-20 with this test, and it exists
/// because `Pass 121.1` left the other three blocks unable to falsify anything.
///
/// # Why a fixture that cannot exhibit the hazard is a fixture that proves
/// # nothing
///
/// The engine's fix stopped reflow reaching any follower in blocks A, B or C —
/// every one of them sits on a different baseline or at a different
/// orientation. That is correct and it is what the fix was for. It also meant
/// that both falsifying assertions in this file went quiet, and **a quiet
/// falsifier is a test that has stopped measuring**: the two `Pin` assertions
/// beside them would have gone on passing against a build that pinned nothing,
/// because nothing was going to move either way.
///
/// So the fixture grew the one shape reflow still acts on: two show operators
/// at the same `f`, differing in `e` alone. That is a single visual line drawn
/// as two runs — a table cell beside another, a title-block field beside its
/// label — which is the overwhelmingly common shape on this operator's
/// documents and the one case where *"the rest of the line"* genuinely is the
/// rest of a line.
///
/// This is therefore the test that tells the shipped rule from a build that
/// pins nothing, and the `assert_ne!` is what stops it passing vacuously.
#[test]
fn a_same_baseline_follower_is_the_case_pinning_still_prevents() {
    // Computed by `tools/gen-textedit-fixtures.py` and printed by it, never
    // guessed: `72.00 + advance("CELL ONE") + 12.00`.
    const TAIL: &str = "144.67 140.00 Tm";
    let pinned = appended_after_edit(
        "CELL ONE",
        "CELL ONE LONGER",
        &disposition::options(Reason::SharesTheLine),
    );
    let reflowed = appended_after_edit("CELL ONE", "CELL ONE LONGER", &EditOptions::default());

    assert!(
        holds(&pinned, TAIL),
        "pinning must leave a same-baseline follower's Tm verbatim; `{TAIL}` is not in the \
         appended revision"
    );
    assert!(
        !holds(&reflowed, TAIL),
        "★ THE FALSIFIER DID NOT FIRE. A follower differing in `e` ALONE is the one shape \
         `Pass 121.1` still lets reflow move, so `EditOptions::default()` must rewrite this Tm. \
         If it does not, block D of the fixture is not the shape it is documented to be — \
         regenerate it with `python tools/gen-textedit-fixtures.py` — and every Pin assertion \
         in this file is passing for the wrong reason."
    );
    assert_ne!(
        pinned, reflowed,
        "the two dispositions must produce different bytes"
    );
}

/// ★ **The edit itself reaches the bytes**, under both dispositions.
///
/// The floor, and it is worth stating separately: every assertion above is about
/// text the operator did **not** touch, and all of them would pass against a
/// build whose edit did nothing at all. This is the one that says an edit
/// happened.
#[test]
fn the_replacement_text_is_in_the_appended_revision() {
    for (label, opts) in [
        ("pin", disposition::options(Reason::Rotated)),
        ("reflow", EditOptions::default()),
    ] {
        let out = appended_after_edit("SHEET 2", "SHEET 9", &opts);
        assert!(
            holds(&out, "SHEET 9"),
            "{label}: the replacement must be in the file"
        );
    }
}
