//! Tests for [`super`] — restyling existing text.
//!
//! ## What these are for, and what they cannot be
//!
//! R1: *"the tests pass" is not a report of working software.* These prove the
//! **engine chain** — that a selection's run ordinals reach `format_text` and
//! that the document afterwards says what the operator asked for. They cannot
//! prove the panel, the combo box or the disclosure line, and the driven check
//! in `tools/ui-verify` is what does that.
//!
//! ★ Every assertion below is on the **document after the edit**, read back
//! through a fresh extraction — never on the return value of the thing under
//! test. A test that asserts "the function returned Ok" is a test of the
//! function's own opinion of itself.

#![cfg(test)]
// ★ The INNER attribute, load-bearing rather than redundant beside the
// `#[cfg(test)] mod tests;` that declares this file.
//
// `tools/gates/check-ui-strings.sh` and `check-theme-colors.sh` both recognise
// this exact line as "nothing in this file reaches the shipped binary". The
// property that earns the exemption is *not in the release build*, and a
// filename is a restatement of that which goes stale; the attribute is the
// fact itself. Without it, every assertion message below is reported as
// operator-facing copy — which is how a 28-hit report once trained people to
// ignore this gate.

use super::{StyleChange, apply};
use crate::app::state::{OpenDoc, ROTATED_TEXT, open_local_fixture};

/// The `Tf` size and `/BaseFont` in force on `run` of page 0, read fresh.
///
/// Goes through `pin::inspect` — the same road the panel reads by — rather than
/// through a private path, so a break in that road fails these too.
fn style_of(doc: &OpenDoc, run: usize) -> (f32, Option<String>) {
    let read = crate::canvas::textedit::pin::inspect(doc, 0, run)
        .expect("the fixture's first run carries provenance");
    (read.style.size, read.style.font_resource)
}

/// How many runs page 0 has, from the shared extraction.
fn run_count(doc: &OpenDoc) -> usize {
    doc.page_text().map_or(0, |t| t.runs.len())
}

/// ★★★ **The headline: a size change reaches the file.**
///
/// The operator's ask, reduced to its smallest true statement — press a number,
/// and the text on the page is that size afterwards.
///
/// Falsified by removing `StyleChange::stamp`'s `Size` arm: with nothing
/// stamped the request is empty, the engine returns `NoOp`, and the size read
/// back is unchanged, which fails here by name.
#[test]
fn a_size_change_reaches_the_document() {
    let mut doc = open_local_fixture(ROTATED_TEXT);
    let (before, _) = style_of(&doc, 0);
    assert!(before > 0.0, "the fixture's first run has a size to change");

    let target = f64::from(before) * 2.0;
    apply(&mut doc, 0, &[0], &StyleChange::Size(target));

    let (after, _) = style_of(&doc, 0);
    assert!(
        (f64::from(after) - target).abs() < 0.01,
        "the run's size should now be {target}, and it is {after}"
    );
}

/// **An edit through this module is undoable**, because it went through the
/// funnel rather than around it.
///
/// The property that is easiest to lose and hardest to notice: a verb called
/// directly on the session still edits the document and still saves, and only
/// `Ctrl+Z` tells you it was wrong.
#[test]
fn a_restyle_is_one_undoable_command() {
    let mut doc = open_local_fixture(ROTATED_TEXT);
    let (before, _) = style_of(&doc, 0);
    assert!(
        !doc.session.can_undo(),
        "a freshly opened document has no history"
    );

    apply(
        &mut doc,
        0,
        &[0],
        &StyleChange::Size(f64::from(before) * 2.0),
    );
    assert!(doc.session.can_undo(), "the restyle is in the undo log");

    let session = std::sync::Arc::get_mut(&mut doc.session).expect("sole owner in a test");
    session.undo().expect("undo the restyle");
    doc.edit_epoch = doc.edit_epoch.wrapping_add(1);

    let (after, _) = style_of(&doc, 0);
    assert!(
        (after - before).abs() < 0.01,
        "undo should put the size back to {before}, and it is {after}"
    );
}

/// **The epoch moves**, which is what makes every cached read — the panel's own
/// stamp among them — notice.
///
/// ★ Its own test rather than an assertion inside the one above, because it is
/// a different failure: an edit that lands in the file and does not bump the
/// epoch shows the operator the *old* size in the panel for ever after, and the
/// page they are looking at is right while the numbers beside it are wrong.
#[test]
fn a_restyle_bumps_the_edit_epoch() {
    let mut doc = open_local_fixture(ROTATED_TEXT);
    let before = doc.edit_epoch;
    let (size, _) = style_of(&doc, 0);
    apply(&mut doc, 0, &[0], &StyleChange::Size(f64::from(size) + 3.0));
    assert_ne!(doc.edit_epoch, before, "the edit epoch must move");
}

/// ★★ **Every run of a multi-run selection is restyled, not just the first.**
///
/// The case the descending-order argument exists for. It is asserted on the
/// *last* run as well as the first, because an implementation that restyled
/// only the head of the list would pass a test that looked at the head.
#[test]
fn a_multi_run_selection_restyles_every_run() {
    let mut doc = open_local_fixture(ROTATED_TEXT);
    let count = run_count(&doc);
    assert!(
        count >= 2,
        "the fixture must have at least two runs; it has {count}"
    );

    // Only the runs that actually carry provenance — a derived-whitespace run
    // has no show operator to pin and is correctly skipped by the engine.
    let pinnable: Vec<usize> = (0..count)
        .filter(|r| crate::canvas::textedit::pin::inspect(&doc, 0, *r).is_some())
        .take(3)
        .collect();
    assert!(
        pinnable.len() >= 2,
        "at least two runs must be pinnable; {} were",
        pinnable.len()
    );

    let before: Vec<f32> = pinnable.iter().map(|r| style_of(&doc, *r).0).collect();
    apply(&mut doc, 0, &pinnable, &StyleChange::Size(31.0));

    for (run, was) in pinnable.iter().zip(before) {
        let (now, _) = style_of(&doc, *run);
        assert!(
            (now - 31.0).abs() < 0.01,
            "run {run} was {was} and should now be 31, but it is {now}"
        );
    }
}

/// **A selection covering no runs changes nothing and says so.**
///
/// The guard that stops an empty gesture reaching the engine at all. Asserted
/// on the epoch rather than on the decline, because "nothing happened to the
/// document" is the claim that matters and the sentence is `text::status`'s
/// business.
#[test]
fn an_empty_selection_edits_nothing() {
    let mut doc = open_local_fixture(ROTATED_TEXT);
    let before = doc.edit_epoch;
    apply(&mut doc, 0, &[], &StyleChange::Size(31.0));
    assert_eq!(doc.edit_epoch, before, "an empty run list must not edit");
}

/// ★★★ **A run ordinal that does not exist is declined, not guessed at.**
///
/// The failure this guards is the expensive one: an out-of-range ordinal that
/// fell back to "the first run whose text matches" would restyle a piece of
/// text the operator never selected, in a file they then send to somebody.
#[test]
fn an_out_of_range_run_edits_nothing() {
    let mut doc = open_local_fixture(ROTATED_TEXT);
    let before = doc.edit_epoch;
    apply(&mut doc, 0, &[9_999], &StyleChange::Size(31.0));
    assert_eq!(
        doc.edit_epoch, before,
        "an unpinnable run must decline rather than edit something else"
    );
}

/// **Bold reaches the file on a page with no bold face**, which is the case the
/// engine's two-verb complement exists for.
///
/// ★ Asserted by the *absence of a refusal* and the presence of an edit rather
/// than by reading a "synthetic" flag out of the file: `R90`'s synthesis is
/// deliberately not recorded in the PDF — it is re-detectable from the bytes,
/// which is a different question from the one this test asks.
#[test]
fn bold_applies_on_a_page_with_no_bold_face() {
    let mut doc = open_local_fixture(ROTATED_TEXT);
    let before = doc.edit_epoch;
    apply(
        &mut doc,
        0,
        &[0],
        &StyleChange::Weight {
            bold: true,
            italic: false,
        },
    );
    assert_ne!(
        doc.edit_epoch, before,
        "bold must apply on every page; if this fails the two-verb complement is broken or the page grew a real Bold"
    );
}

/// ★★★ **The two-verb retry, and the engine defect it found — now FIXED, and
/// this test predicted its own failure.**
///
/// `textedit/format_family.pdf` is a `/Times-Roman` run `hello world` on a page
/// that also carries `/F2` (`Calibri-Bold`, fully covering) and `/F3`
/// (`Times-Bold`, whose `/Differences` remaps `o` to `/bullet`, so it does NOT
/// cover the run).
///
/// # What this test used to assert, and why it was right to
///
/// Asking for synthetic bold was refused by `gate_synthesis` — *"a REAL bold
/// face is available"* — **naming `Times-Bold`**, because it matched the run's
/// family. This module took that offer and the offer was refused for coverage.
/// So **bold was unreachable on that page through either verb**, while
/// `format-text --set-font F2` succeeded from the command line throughout.
///
/// It was written as a **characterisation** test, with the engine revision
/// named, and it closed with a prediction:
///
/// > When the engine picks a covering face, this test starts failing on
/// > assertion 1 — and **that failure is the good news, not a regression**.
/// > `pdfcer-core` at `914389c`, 2026-08-27.
///
/// ## The prediction came true the same night
///
/// `Pass 144.0` (`cfa2c44`): *"the face a synthesis refusal names is now one
/// `set_font` accepts."* The engine reproduced all three commands on the
/// release binary before accepting the report, found the cause —
/// `gate_synthesis` decided *"a real bold face is available"* from **two string
/// tests on `/BaseFont`** and never asked whether the face could show this
/// run's characters — and moved the four acceptance conditions into one shared
/// predicate so a gate and a commit cannot disagree.
///
/// ★ Their own note on it is worth keeping: it is R221's third instance — *a
/// predicate deciding whether a capability applies, written by hand at a
/// different call site as a parallel description of when the real function
/// succeeds* — and it **inverts** the usual risk analysis, because a false
/// positive here removes the capability entirely rather than costing a slow
/// path or a wrong pixel.
///
/// # What is asserted NOW
///
/// The three things that were the point all along, with the first two flipped
/// from "nothing happened" to "the right thing happened":
///
/// 1. **Bold reaches the run**, and the face it lands on is one that can
///    actually show `hello world` — which on this page is `/F2`, not the `/F3`
///    the old gate named;
/// 2. **the epoch moves**, because the page really changed;
/// 3. **nothing is declined**, because nothing refused.
///
/// ★ The **face is asserted by name**, not merely "something changed". A build
/// that fell back to synthesis would also change the run and bump the epoch,
/// and would be a *worse* answer on a page that carries a covering real face —
/// so a test that only checked for movement would pass on the second-best
/// outcome.
#[test]
fn bold_takes_the_covering_real_face_on_a_page_that_has_one() {
    use crate::app::state::open_fixture;
    let mut doc = open_fixture("textedit/format_family.pdf");
    let (size_before, face_before) = style_of(&doc, 0);
    let epoch_before = doc.edit_epoch;

    apply(
        &mut doc,
        0,
        &[0],
        &StyleChange::Weight {
            bold: true,
            italic: false,
        },
    );

    let (size_after, face_after) = style_of(&doc, 0);
    assert_ne!(
        face_before, face_after,
        "bold must reach the run: `Pass 144.0` makes `gate_synthesis` name a face `set_font` \
         accepts, and on this page that face is `/F2` (Calibri-Bold). If this fails, either the \
         engine regressed or this shell stopped taking the offer the refusal names"
    );
    assert_eq!(
        face_after.as_deref(),
        Some("F2"),
        "and it must be the COVERING face. `/F3` is Times-Bold, which remaps `o` to a bullet \
         and cannot show `hello world`; falling back to synthesis would also move the run and \
         would be the second-best answer on a page carrying a real bold that works"
    );
    assert_eq!(
        size_before, size_after,
        "a weight change must not move the size"
    );
    assert_ne!(
        doc.edit_epoch, epoch_before,
        "the page changed, so the epoch must move — it is what re-reads every panel"
    );
    assert_eq!(
        crate::app::status::decline::recorded_for_test(),
        None,
        "nothing refused, so the operator is owed no sentence. A decline here would mean the \
         retry path recorded a refusal it then recovered from, which reads as a failure that \
         worked"
    );
}
