//! # `editmodel::proof` — **the tail did not move, proved in the bytes**
//!
//! `crate::redact::proof`'s shape applied to `DEFECTS.md` **D4b**: a claim about
//! what an edit does to a *file*, asserted against the file, with the falsifying
//! run beside it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/editmodel/proof.md`.

#![cfg(test)]

use std::path::PathBuf;

use pdfcer_core::document::Document;
use pdfcer_core::edit::EditSession;
use pdfcer_core::text_edit::{
    BlockRecognitionOptions, EditOptions, EditRequest, EditableTextModel, FollowerDisposition,
    GlyphRef, ReflowEngine, TextPosition, reflow_recognition_options,
};

use super::disposition::{self, Reason};

/// The fixture this module is written against.
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
fn reason_for(session: &EditSession, needle: &str) -> Reason {
    let text = extract(session);
    let (run, tm, ctm) = run_of(session, needle);
    // The multi-run test, derived exactly as `plan` derives it: the DEFAULT
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

/// **A right-aligned block is detected as right-aligned**, against the real
/// engine on a real page.
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

/// **Rotated text reaches the rotation guard.**
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
    // Passing the engine's real finding in, rather than `None`, is what makes
    // this an assertion about the RUNG ORDER as well as about the guard: this
    // block's alignment is whatever the recogniser makes of fifteen one-glyph
    // runs, and the answer must be `Rotated` regardless of it.
    // `false` for the multi-run rung, deliberately, and it is the same kind of
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

/// **Upright left-aligned text still reflows** — the selectivity control.
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
// The bytes — and the falsifying run beside each one
// ===========================================================================

/// **The right-aligned tail does not move, and under the old shell's
/// options it does.**
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

/// **The rotated line's tail does not move, and under the old shell's
/// options it slides along the wrong axis.**
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
    // Inverted for the same reason as its right-aligned sibling, and here
    // the engine's rule bites harder: a rotated follower differs from the
    // edited run in `a`, `b`, `c` AND `d`, so the "differs in `e` alone" test
    // ends the line at the first character of it.
    //
    // Note what is NOT weakened by this. The shell still answers
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

/// **The case pinning still uniquely prevents: two runs on ONE baseline.**
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

/// **The edit itself reaches the bytes**, under both dispositions.
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
