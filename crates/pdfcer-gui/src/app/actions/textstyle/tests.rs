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
//! Every assertion below is on the **document after the edit**, read back
//! through a fresh extraction — never on the return value of the thing under
//! test. A test that asserts "the function returned Ok" is a test of the
//! function's own opinion of itself.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/textstyle/tests.md`.

#![cfg(test)]
// The INNER attribute, load-bearing rather than redundant beside the
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

/// **The headline: a size change reaches the file.**
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
#[test]
fn a_restyle_bumps_the_edit_epoch() {
    let mut doc = open_local_fixture(ROTATED_TEXT);
    let before = doc.edit_epoch;
    let (size, _) = style_of(&doc, 0);
    apply(&mut doc, 0, &[0], &StyleChange::Size(f64::from(size) + 3.0));
    assert_ne!(doc.edit_epoch, before, "the edit epoch must move");
}

/// **Every run of a multi-run selection is restyled, not just the first.**
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
#[test]
fn an_empty_selection_edits_nothing() {
    let mut doc = open_local_fixture(ROTATED_TEXT);
    let before = doc.edit_epoch;
    apply(&mut doc, 0, &[], &StyleChange::Size(31.0));
    assert_eq!(doc.edit_epoch, before, "an empty run list must not edit");
}

/// **A run ordinal that does not exist is declined, not guessed at.**
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

/// **Bold binds a REAL `Helvetica-Bold` on a page carrying no bold face**
/// — the standard-14 rung, and the reason `set_style` was wired.
#[test]
fn bold_binds_a_real_standard_face_on_a_page_with_no_bold_face() {
    let mut doc = open_local_fixture(ROTATED_TEXT);
    let before = doc.edit_epoch;
    let (size_before, face_before) = style_of(&doc, 0);
    assert!(
        face_before.is_some(),
        "the fixture's first run names a font resource to compare against"
    );

    apply(
        &mut doc,
        0,
        &[0],
        &StyleChange::Weight {
            bold: Some(true),
            italic: None,
        },
    );

    assert_ne!(
        doc.edit_epoch, before,
        "bold must apply on every page; if this fails the ladder is broken or the page grew a real Bold"
    );

    let (size_after, face_after) = style_of(&doc, 0);
    assert_ne!(
        face_after, face_before,
        "rung 2 must bind the standard-14 sibling as a NEW /Font resource; an unchanged \
         resource key means the letters were stroke-thickened instead"
    );
    // The size is checked because binding a font resource re-emits `Tf`,
    // which carries the size in the same operator. A rung that bound the right
    // face and lost the size would satisfy the assertion above and wreck the
    // drawing.
    assert!(
        (size_after - size_before).abs() < 0.01,
        "binding a bold face must not change the size: {size_before} -> {size_after}"
    );
}

/// **The ladder SAYS which rung it took**, and rung 2's sentence is the
/// one that did not exist until the verb was wired.
///
/// # Why the sentence needs its own test and the edit does not cover it
///
/// `super::ladder_note` maps [`StyleRung`] onto one operator sentence, and
/// **every one of its arms can legitimately return `None`** — for a change that
/// is not a style change, for a rung this build has no sentence for, and for a
/// mixed per-axis outcome no gesture here can produce. A mapping whose wrong
/// answer looks exactly like one of its right answers cannot be checked by
/// reading it.
///
/// The operator's symptom would be silence: the text changes, correctly,
/// and pdfcer says nothing about how. That is the quietest defect this module
/// can ship and it would survive every other test in this file, all of which
/// assert on the document rather than on what was disclosed.
///
/// Read through `last_edit_disclosure` — the **status bar's own door**, keyed
/// on the epoch — rather than by calling `ladder_note` directly, so a break
/// anywhere between the mapping and the operator's eye fails here. It is a
/// thread-local, so this is safe beside the parallel tests around it.
///
/// [`StyleRung`]: pdfcer_core::text_edit::StyleRung
#[test]
fn the_standard_face_rung_tells_the_operator_it_used_a_real_face() {
    let mut doc = open_local_fixture(ROTATED_TEXT);
    apply(
        &mut doc,
        0,
        &[0],
        &StyleChange::Weight {
            bold: Some(true),
            italic: None,
        },
    );

    let shown = crate::app::actions::disclosure::last_edit_disclosure(doc.edit_epoch)
        .expect("binding a standard-14 sibling owes the operator a sentence");
    let joined = shown.notes.join(" ");
    // Asserted on the SUBSTANCE, not on the whole sentence. A catalog string
    // is reworded for clarity often and that is not a regression; what must not
    // change is that the operator is told a real typeface was used and which
    // one. Matching the full sentence would make this test an obstacle to
    // editing prose, which is how a test gets weakened instead of fixed.
    assert!(
        joined.contains("Helvetica-Bold"),
        "the sentence must name the face that was bound; it said: {joined}"
    );
    assert!(
        joined.contains("every PDF reader is required to carry"),
        "rung 2's sentence must say the face needs no embedding; it said: {joined}"
    );
    assert!(
        !joined.contains("thickened or slanted the letters artificially instead"),
        "rung 2 bound a real face, so the synthetic sentence must NOT appear; it said: {joined}"
    );
}

/// **The two-verb retry, and the engine defect it found — now FIXED, and
/// this test predicted its own failure.**
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
            bold: Some(true),
            italic: None,
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

// ===========================================================================
// `reflow_refusal` — the mapping that had no test, and shipped ten wrong
// sentences because of it
// ===========================================================================

/// **THE DEFECT THIS FILE HAD NO TEST FOR, until 2026-09-07.**
#[test]
fn an_engine_decline_with_no_discriminant_names_no_cause_and_promises_no_remedy() {
    use crate::text::textedit::ReflowRefusal;
    use pdfcer_core::text_edit::ReflowApplyError as E;

    // The causes a reflow can raise as `Unsupported`. The shell must give the
    // same honest answer to each until it maps causes on purpose; a test that
    // tried one cause would pass against a build that special-cased it.
    use pdfcer_core::text_edit::UnsupportedCause as C;
    let every_unsupported = [
        C::NoContents,
        C::NoFont,
        C::FontUnresolvable,
        C::NoShowOperators,
        C::ShowOperatorsNotFound,
        C::ShowWithoutFont,
        C::DegenerateCtm,
        C::RotatedOrSkewed { matrix: "CTM" },
        C::MixedScale {
            matrix: "text matrix",
        },
        C::NoSpaceGlyph {
            font: "Helvetica".to_owned(),
        },
    ];

    for cause in every_unsupported {
        let sentence = cause.to_string();
        let got = super::reflow_refusal(&E::Unsupported(cause));
        assert_eq!(
            got,
            ReflowRefusal::EngineDeclined,
            "every `Unsupported` cause must reach the same shell refusal until the shell maps \
             causes deliberately. This one did not: {sentence:?}.\n\
             Splitting the mapping belongs here, by matching on `UnsupportedCause` — never on \
             its sentence."
        );
    }

    // And the sentence must not name a cause. This is the actual assertion:
    // the old wording was not wrong because it was `PageSetChanged`, it was
    // wrong because it CLAIMED SOMETHING about the operator's document that the
    // shell had no way to know.
    let line = ReflowRefusal::EngineDeclined.line();
    for invented in [
        "added, removed or reordered",
        "Save this file and open it again",
        "save and reopen",
        "encrypted",
        "rotated",
    ] {
        assert!(
            !line.contains(invented),
            "`EngineDeclined` names a cause it cannot know: {invented:?} appears in {line:?}. \
             One in ten `Unsupported` sentences has that cause; naming it means being wrong nine \
             times out of ten, confidently, which is what this variant exists to stop."
        );
    }
    // It must still say the thing that matters most after any refusal.
    assert!(
        line.contains("has not been changed"),
        "every refusal sentence must tell the operator nothing was written: {line:?}"
    );
}

/// The two variants that still carry a named cause must only be reachable from
/// an engine variant that actually names it.
#[test]
fn a_named_cause_comes_from_a_named_engine_variant() {
    use crate::text::textedit::ReflowRefusal;
    use pdfcer_core::text_edit::ReflowApplyError as E;

    assert_eq!(
        super::reflow_refusal(&E::Encrypted),
        ReflowRefusal::Encrypted,
        "`E::Encrypted` is a distinct engine variant, so the shell may and must name that cause"
    );
    assert_eq!(
        super::reflow_refusal(&E::NoProvenance),
        ReflowRefusal::CannotTrace,
        "`NoProvenance` names its own cause and keeps its own sentence"
    );

    // ⚠ `PageSetChanged` is currently constructed NOWHERE. It is kept because
    // the guard it describes is real PDF behaviour a future engine may
    // reinstate by name. If this assertion ever fails, that has happened — and
    // the right response is to delete this line, not the variant.
    assert!(
        !matches!(
            super::reflow_refusal(&E::Unsupported(
                pdfcer_core::text_edit::UnsupportedCause::NoContents
            )),
            ReflowRefusal::PageSetChanged
        ),
        "`PageSetChanged` is being reached from an undiscriminated `Unsupported` again, which is \
         the exact defect corrected on 2026-09-07"
    );
}

/// **THE TRAP THE ENGINE WARNED ABOUT, MADE INTO AN ASSERTION.**
#[test]
fn the_one_recoverable_refusal_keeps_its_remedy() {
    use crate::text::textedit::ReflowRefusal;
    use pdfcer_core::text_edit::ReflowApplyError as E;

    let got = super::reflow_refusal(&E::PageEditedThisSession);
    assert_eq!(
        got,
        ReflowRefusal::PageAlreadyEdited,
        "`PageEditedThisSession` must keep its remedy. It is unreachable at engine \
         `025d703d` (`G015` deleted its only producer) and the engine kept the variant, so \
         this arm is the thing that has to be correct when a future engine reinstates the \
         guard. If this reads `Other` or `EngineDeclined`, a wildcard has come back — \
         which is the defect corrected twice on 2026-09-07."
    );

    // And the remedy must actually be IN the sentence. Reaching the right
    // variant while its wording lost the instruction would satisfy the
    // assertion above and help nobody.
    let line = got.line();
    assert!(
        line.contains("Save this file and open it again"),
        "the recoverable refusal must tell the operator the one thing that clears it: {line:?}"
    );
    assert!(
        line.contains("added text to this page"),
        "it must name the cause, which pdfcer now genuinely knows — the engine's own variant \
         says so. A vague sentence here is the pre-2026-09-07 behaviour: {line:?}"
    );
}

/// **Every format refusal an operator can cause gets its OWN sentence**,
/// and the posture refusal is the one the wildcard would have eaten.
///
/// # Why a wildcard needs a test and a review will not do
///
/// `FormatError` is `#[non_exhaustive]` and [`super::refusal_of`] ends in
/// `_ => Other`. A variant that arrives from the engine therefore lands in the
/// generic sentence **silently** — no compiler error, no warning, nothing red.
/// `crate::text::textedit`'s header states the rule: *"any `match` of yours
/// ending in `_` just gained a variant it will not distinguish, and the one it
/// will not distinguish is the one you care about."*
///
/// [`FormatError::SynthesisRefusedByPosture`] became reachable on
/// 2026-09-11, when `StyleChange::stamp` moved from `set_synthetic` to
/// `set_style`. It fires **only** when the operator explicitly chose
/// `StylePolicy::Refuse` and pdfcer then walked every real rung and found
/// nothing — so collapsing it into `Other` would answer *"never fake it"* with
/// *"pdfcer could not change that text"*: their own instruction, obeyed
/// exactly, reported back as a malfunction.
///
/// The **distinctness** sweep is the assertion that does the work. Checking
/// that each variant maps to something passes trivially; checking that no two
/// map to the same thing is the property a wildcard destroys, one variant at a
/// time, and it is what fails the moment somebody deletes a named arm.
///
/// [`FormatError::SynthesisRefusedByPosture`]: pdfcer_core::text_edit::FormatError::SynthesisRefusedByPosture
#[test]
fn every_actionable_format_refusal_keeps_its_own_sentence() {
    use crate::text::status::TextStyleRefusal as R;
    use pdfcer_core::text_edit::FormatError as E;

    // The two faces the fixture claims WOULD show the run. Named once so
    // the construction and the sentence assertion cannot drift apart while
    // both keep passing.
    const REMEDY_A: &str = "Times-Roman";
    const REMEDY_B: &str = "Helvetica";

    // One real error per case, CONSTRUCTED rather than described — so a field
    // the engine renames fails to compile here instead of drifting quietly.
    let cases = [
        (
            E::TargetFontMissing("Gill Sans".to_owned()),
            R::FaceNotOnPage,
        ),
        (
            E::ShearUnsupported("the follower would move".to_owned()),
            R::ItalicWouldMove,
        ),
        (
            E::NoFaceWithoutStyle {
                run_font: "Helvetica-Bold".to_owned(),
                style: "bold",
            },
            R::NoPlainFace("bold"),
        ),
        //
        // It wraps `text_edit::Refusal`, which is `#[non_exhaustive]` with all
        // fields public and, until `Pass 295.0`, **no public constructor**
        // (`Refusal::char_refusal` is private). A struct expression is `E0639`
        // and there was nothing else to call, so no consumer of `pdfcer-core`
        // could produce one — the public variant was untestable by
        // construction and for ever.
        //
        // It was filed as
        // `request_format_error_coverage_failure_cannot_be_constructed_by_a_consumer.md`
        // and `Refusal::new` is now public. The engine kept `#[non_exhaustive]`
        // — which is right; it reserves the right to add a fifth field — and
        // removed the side effect nobody chose.
        //
        // The arm it covers is `FaceLacksCharacters`, the refusal that fires
        // on `Times-Bold`-with-a-remapped-`o`: the exact page that produced this
        // project's `Pass 144.0` request, and the one of the named arms most
        // likely to be broken by a careless edit. `RInvTrigger::TargetAbsent`
        // and a real character are used rather than defaults, so the case is a
        // refusal that could actually arrive rather than a shape that merely
        // type-checks.
        (
            E::CoverageFailure(pdfcer_core::text_edit::Refusal::new(
                pdfcer_core::text_edit::RInvTrigger::TargetAbsent,
                Some('o'),
                "Times-Bold",
                "R-INV-1: character U+006F 'o' has no code in font 'Times-Bold'",
                // A REAL remedy list, not `Vec::new()`. `Pass 296.1`
                // added this fifth argument and `rustc`'s own suggestion was
                // `/* Vec<String> */`. An empty vec compiles, keeps every
                // assertion below green, and silently declines the feature the
                // engine had just shipped — the whole point of
                // `remedy_faces` is that the sentence can NAME the faces that
                // would show the run, so the fixture has to carry names or the
                // interpolating branch is executed by no test at all.
                vec![REMEDY_A.to_owned(), REMEDY_B.to_owned()],
            )),
            R::FaceLacksCharacters(vec![REMEDY_A.to_owned(), REMEDY_B.to_owned()]),
        ),
        // `rung_one` carries a WHOLE CLAUSE since `Pass 295.0`, not the face
        // list it used to (`passed`). The old field was interpolated straight
        // after the words `page faces`, which ran them together — `page
        // facesHelvetica-Bold` — and, worse, read as *"X was used"* where it
        // meant *"X was tried and rejected"*. `thiserror`'s format string
        // cannot branch, so the branch moved to the construction site.
        (
            E::SynthesisRefusedByPosture {
                style: "bold",
                run_font: "Helvetica".to_owned(),
                rung_one: "rung 1: page faces Helvetica-Bold could not show the run".to_owned(),
                flag: "style_policy",
            },
            R::FakingDeclined,
        ),
        (E::InvalidRenderMode { mode: 9 }, R::RenderModeInvalid),
        (E::ConflictingRenderMode, R::RenderModeWithFakeBold),
        (E::BadTargetWidth(-1.0), R::WidthNotPositive),
        (
            E::NoAdvanceWidth {
                base_font: "Helvetica".to_owned(),
            },
            R::WidthNoMetrics,
        ),
        (E::WidthFitKerned, R::WidthKerned),
        (
            E::TextRun(pdfcer_core::vector::VectorEditError::TextRunHasNoWidth { index: 0 }),
            R::WidthNoBaseline,
        ),
    ];

    let mut seen = Vec::new();
    for (error, want) in cases {
        let got = super::refusal_of(&error);
        assert_eq!(
            got, want,
            "{error} must reach {want:?} and reached {got:?}; a wildcard hit means a named arm went"
        );
        seen.push(got);
    }

    for (i, a) in seen.iter().enumerate() {
        for b in seen.iter().skip(i + 1) {
            assert_ne!(
                a, b,
                "two distinct engine refusals produced the same operator sentence: {seen:?}"
            );
        }
    }

    // The variant is half the delivery. `refusal_of` reaching
    // `FaceLacksCharacters` was already true when the payload did not exist,
    // so an assertion on the variant alone passes just as happily against
    // `Vec::new()` — which is exactly the shape of a consumed-in-name-only
    // wiring. What has to be asserted is that the engine's list reaches the
    // OPERATOR'S SENTENCE, by name, because the names are the only part of
    // this refusal anybody can act on.
    let named = super::refusal_of(&E::CoverageFailure(pdfcer_core::text_edit::Refusal::new(
        pdfcer_core::text_edit::RInvTrigger::TargetAbsent,
        Some('o'),
        "Times-Bold",
        "R-INV-1: character U+006F 'o' has no code in font 'Times-Bold'",
        vec![REMEDY_A.to_owned(), REMEDY_B.to_owned()],
    )))
    .line();
    assert!(
        named.contains(REMEDY_A) && named.contains(REMEDY_B),
        "the coverage sentence must NAME the faces the engine says would show this run, and it said: {named}"
    );

    // Empty is a REAL value, not a missing one. An engine that knows of no
    // covering face sends an empty list, and the shell must answer with a
    // whole sentence of its own rather than a dangling "can show this text"
    // with nothing in front of it.
    let bare = super::refusal_of(&E::CoverageFailure(pdfcer_core::text_edit::Refusal::new(
        pdfcer_core::text_edit::RInvTrigger::TargetAbsent,
        Some('o'),
        "Times-Bold",
        "R-INV-1: character U+006F 'o' has no code in font 'Times-Bold'",
        Vec::new(),
    )))
    .line();
    assert!(
        !bare.is_empty() && !bare.contains(REMEDY_A),
        "with no remedy the sentence must still stand on its own, and it said: {bare}"
    );
    assert_ne!(
        named, bare,
        "the remedy list changed nothing about the sentence, which means it was not consumed"
    );

    // The control. `NoOp` is the variant this shell raises internally to
    // abort a `vector_edit`, and it is CORRECT for it to land in `Other` — the
    // operator never sees it. Asserted so that "everything maps to something
    // distinct" cannot be satisfied by deleting the wildcard.
    assert_eq!(
        super::refusal_of(&E::NoOp),
        R::Other,
        "a refusal with nothing for the operator to act on belongs in the generic sentence"
    );
}

/// Every engine decline reaches a shell refusal that suits it, and none of them
/// collapses into the general one.
#[test]
fn each_engine_decline_reaches_a_refusal_that_suits_it() {
    use crate::text::textedit::ReflowRefusal;
    use pdfcer_core::text_edit::ReflowApplyError as E;
    use pdfcer_core::text_edit::ReflowDecline as D;

    // One real error per decline, constructed rather than described.
    let cases = [
        (E::PageEditedThisSession, D::RetryAfterSaveAndReopen),
        (E::PageIndex(99), D::NotFound),
        (
            E::Unsupported(pdfcer_core::text_edit::UnsupportedCause::RotatedOrSkewed {
                matrix: "CTM",
            }),
            D::NotReflowable,
        ),
    ];

    let mut seen = Vec::new();
    for (error, want_decline) in cases {
        assert_eq!(
            error.decline(),
            want_decline,
            "the engine's own discriminant moved; re-read `ReflowDecline` before touching \
             `reflow_refusal`"
        );
        seen.push(super::reflow_refusal(&error));
    }

    for (i, a) in seen.iter().enumerate() {
        for b in seen.iter().skip(i + 1) {
            assert_ne!(
                a, b,
                "two distinct engine declines produced the same shell refusal. That is the \
                 collapse `ReflowDecline` was requested to end: {seen:?}"
            );
        }
    }
    assert!(
        seen.contains(&ReflowRefusal::PageAlreadyEdited),
        "the save-and-reopen case must survive the walk. ⚠ It is unreachable from an \
         operator gesture at engine `025d703d` and is asserted anyway: this walk is over \
         the DECLINE enum, which is exhaustive, so a decline losing its distinct outcome \
         is the defect regardless of whether anything currently produces it: {seen:?}"
    );
}

/// **A composite-font reflow refusal names the FONT, and does so from the
/// engine's own discriminant rather than from its prose.**
#[test]
fn a_composite_font_refusal_says_it_is_the_font() {
    use crate::text::textedit::ReflowRefusal;
    use pdfcer_core::text_edit::ReflowApplyError as E;
    use pdfcer_core::text_edit::{RInvTrigger, Refusal};

    // The refusal as `reflow_apply::refuse_if_composite` actually builds it:
    // trigger `Composite`, no character (the whole run is the problem, not one
    // code), the real `/BaseFont` off his drawing, the engine's own message,
    // and an EMPTY remedy list — there is no standard-14 face that makes a
    // composite run re-wrappable, and the engine says so by sending none.
    let composite = E::Refused(Refusal::new(
        RInvTrigger::Composite,
        None,
        "AQHZBV+CenturyGothic",
        "R-INV-4: font 'AQHZBV+CenturyGothic' is a composite (Type 0 / CIDFont) run; \
         within-block reflow of composite/CJK fonts is deferred (FF-E)",
        Vec::new(),
    ));
    assert_eq!(
        super::reflow_refusal(&composite),
        ReflowRefusal::FontIsComposite,
        "a composite refusal must reach the sentence that names the font as the cause; if this \
         reads `EngineDeclined` the mapping has collapsed back to the vague answer O198 was \
         filed about"
    );

    // The falsification. Same engine VARIANT, different TRIGGER.
    let other_trigger = E::Refused(Refusal::new(
        RInvTrigger::TargetAbsent,
        Some('o'),
        "Times-Bold",
        "R-INV-1: character U+006F 'o' has no code in font 'Times-Bold'",
        Vec::new(),
    ));
    assert_ne!(
        super::reflow_refusal(&other_trigger),
        ReflowRefusal::FontIsComposite,
        "a non-composite `Refused` reached the composite sentence, so the arm is matching on \
         the variant rather than on `RInvTrigger`. The operator would be told his font stores \
         more than one byte per character when the engine said nothing of the kind."
    );

    // And the sentence has to carry the two things the variant promises: the
    // font named as the cause, and the standing guarantee that nothing was
    // written. It must NOT offer a remedy — there is none, and R9 forbids
    // inventing one.
    let line = ReflowRefusal::FontIsComposite.line();
    assert!(
        line.contains("font"),
        "the composite sentence must name the font as the cause: {line:?}"
    );
    assert!(
        line.contains("has not been changed"),
        "every refusal sentence must tell the operator nothing was written: {line:?}"
    );
    for invented in ["choose", "try", "instead", "Save this file"] {
        assert!(
            !line.contains(invented),
            "the composite sentence offers a remedy that does not exist ({invented:?}): {line:?}"
        );
    }
}

/// **Bold off returns a bolded run to its own regular face**, the page's,
/// and says so: the first press bound `Helvetica-Bold` (rung 2), the second
/// asks the axis off (`StyleTarget`) and rung 1 binds the run's original
/// `Helvetica` resource again.
#[test]
fn bold_off_returns_to_the_regular_face_the_page_carries() {
    let mut doc = open_local_fixture(ROTATED_TEXT);
    let (size_before, face_before) = style_of(&doc, 0);
    let weight = |bold| StyleChange::Weight {
        bold: Some(bold),
        italic: None,
    };
    apply(&mut doc, 0, &[0], &weight(true));
    assert_ne!(style_of(&doc, 0).1, face_before, "the first press bolds");
    let bolded = doc.edit_epoch;

    apply(&mut doc, 0, &[0], &weight(false));
    assert_ne!(doc.edit_epoch, bolded, "bold off must apply");
    let (size_after, face_after) = style_of(&doc, 0);
    assert_eq!(
        face_after, face_before,
        "bold off must bind the run's own regular face, the resource it started in"
    );
    assert!((size_after - size_before).abs() < 0.01);
    let shown = crate::app::actions::disclosure::last_edit_disclosure(doc.edit_epoch)
        .expect("taking bold off owes the operator a sentence");
    let joined = shown.notes.join(" ");
    assert!(
        joined.contains("without bold") && joined.contains("already carried"),
        "the sentence must say the page's own face without bold was used; it said: {joined}"
    );
}
