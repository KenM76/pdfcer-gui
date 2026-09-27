//! Design and rationale: `docs/modules/pdfcer-gui-base/printverdicts_tests.md`.
#![cfg(test)]
//! # `printverdicts_tests` — the corrected clip count, proved headlessly
//!
//! ## What is proved here, and why none of it needs a window
//!
//!
//! **Every failure mode here is silent.** A cache key that never matches
//! produces the *old* count, which is the correct answer to a different
//! question and looks exactly like the feature working on a job where nothing
//! is blank. A cache key that matches too readily produces a smaller count,
//! which looks exactly like the feature working *properly*. Neither is visible
//! in a screenshot of the button, and the difference between them is a warning
//! removed on evidence about a different page — on the one control in this
//! application with no undo behind it. So the assertions below are on the
//! *state* the claim reports as well as on its number: `Geometric(3)` and
//! `AtMost(3)` are the same photograph and a different truth.
//!
//! `#![cfg(test)]` is the FIRST line, so nothing here reaches a release build.

use super::*;
use crate::printspooler::{DeviceGeometry, JobResolution};

/// A placement that either overhangs the printable rectangle or does not.
fn placed(clipped: bool) -> Placement {
    Placement {
        scale: 1.0,
        offset_x_pt: 0.0,
        offset_y_pt: 0.0,
        clipped,
    }
}

/// A job whose sheets are the given `(document page, clipped)` pairs, in send
/// order.
fn job_of(sheets: &[(usize, bool)]) -> Job {
    Job {
        device: DeviceGeometry {
            dpi: (600, 600),
            printable_pt: (600.0, 780.0),
            physical_pt: (612.0, 792.0),
            offset_pt: (6.0, 6.0),
        },
        resolution: JobResolution {
            dpi: 300,
            device_dpi: 600,
            capped: false,
            uncapped_page_mb: 34,
        },
        plans: sheets
            .iter()
            .map(|&(index, clipped)| PagePlan {
                index,
                placement: placed(clipped),
                render_scale: 4.0,
                tile: None,
            })
            .collect(),
    }
}

/// Page sizes in document order, one US Letter per page — indexed by
/// `PagePlan::index`, never by a plan position.
fn page_sizes(pages: usize) -> Vec<(f64, f64)> {
    vec![(612.0, 792.0); pages]
}

/// The context a plain frame would build.
fn context() -> Context {
    Context::new(
        pdfcer_render::AnnotationScope::Document,
        &pdfcer_core::settings::Settings::default(),
        (600.0, 780.0),
        None,
    )
}

/// Record `overhang` for the plan at `position` in the job's send order.
fn examine(
    verdicts: &mut Verdicts,
    context: &Context,
    job: &Job,
    position: usize,
    overhang: Overhang,
) {
    verdicts.remember(
        context,
        &job.plans[position],
        &page_sizes(job.plans.len().max(8)),
        overhang,
    );
}

// ===========================================================================
// The arithmetic
// ===========================================================================

/// **Printing without ever opening the preview reports the geometric
/// count, in the geometric words.** The ruling this feature degrades to.
#[test]
fn a_job_nobody_has_previewed_reports_the_plain_geometric_count() {
    let job = job_of(&[(4, false), (0, true), (2, true)]);
    let verdicts = Verdicts::default();
    assert_eq!(
        verdicts.claim(&context(), &job, &page_sizes(8)),
        ClipClaim::Geometric(2),
        "with nothing examined the count must be `Job::clipped()` exactly, said in the words \
         it has always been said in"
    );
    assert_eq!(
        verdicts.claim(&context(), &job, &page_sizes(8)).count(),
        job.clipped(),
        "the never-previewed count IS the geometric count — if these ever differ, the \
         degradation is no longer honest"
    );
}

/// **The operator's own case: one sheet, blank overhang, no warning at
/// all.** Operator request O113.
#[test]
fn a_single_sheet_examined_and_found_blank_removes_the_warning_entirely() {
    let job = job_of(&[(0, true)]);
    let context = context();
    let mut verdicts = Verdicts::default();
    assert_eq!(job.clipped(), 1, "the geometric fact is unchanged");

    examine(&mut verdicts, &context, &job, 0, Overhang::BlankBand);

    let claim = verdicts.claim(&context, &job, &page_sizes(8));
    assert_eq!(claim, ClipClaim::None);
    assert!(
        claim.commit_label().is_none(),
        "the button must read plain Print: the only clipped sheet was examined and loses \
         nothing, so there is no disclosure left to make"
    );
    assert!(
        claim.summary(1).is_none(),
        "and the caption above it must go too, or the count and the picture disagree again"
    );
}

/// **Blank, inked and unexamined together — the mixed case the whole
/// design is for.**
#[test]
fn blank_inked_and_unexamined_sheets_produce_a_ceiling() {
    let job = job_of(&[(0, true), (1, true), (2, true), (3, true), (4, true)]);
    let context = context();
    let mut verdicts = Verdicts::default();

    examine(&mut verdicts, &context, &job, 0, Overhang::BlankBand);
    examine(&mut verdicts, &context, &job, 1, Overhang::Losing);

    let claim = verdicts.claim(&context, &job, &page_sizes(8));
    assert_eq!(
        claim,
        ClipClaim::AtMost(4),
        "geometric 5 minus 1 known blank = 4, with the three unexamined sheets still counted \
         because a claim about them would be invented"
    );
    let label = claim.commit_label().expect("a ceiling still discloses");
    assert!(
        label.contains("may"),
        "a number nobody measured must not be stated as one that was: {label}"
    );
}

/// **Examine every clipped sheet and the count becomes exact.**
#[test]
fn examining_every_clipped_sheet_makes_the_count_a_measurement() {
    let job = job_of(&[(0, true), (1, true), (2, true), (3, false)]);
    let context = context();
    let mut verdicts = Verdicts::default();

    examine(&mut verdicts, &context, &job, 0, Overhang::BlankBand);
    examine(&mut verdicts, &context, &job, 1, Overhang::BlankBand);
    examine(&mut verdicts, &context, &job, 2, Overhang::Losing);

    let claim = verdicts.claim(&context, &job, &page_sizes(8));
    assert_eq!(claim, ClipClaim::Measured(1));
    let label = claim
        .commit_label()
        .expect("one sheet really does lose ink");
    assert!(
        !label.contains("may"),
        "a measured count must NOT hedge — softening a true statement to match a better one \
         is how the next defect gets built: {label}"
    );
    assert!(label.contains("lose content"), "{label}");
}

/// **"We could not look" is not "we looked and it was fine".**
#[test]
fn a_sheet_that_would_not_render_stays_counted() {
    let job = job_of(&[(0, true), (1, true)]);
    let context = context();
    let mut verdicts = Verdicts::default();

    examine(&mut verdicts, &context, &job, 0, Overhang::Unknown);
    examine(&mut verdicts, &context, &job, 1, Overhang::BlankBand);

    assert_eq!(
        verdicts.claim(&context, &job, &page_sizes(8)),
        ClipClaim::AtMost(1),
        "the unrenderable sheet must still be counted, and because it is unresolved the \
         remaining number is a ceiling rather than a measurement"
    );
}

/// **A second copy of the same sheet inherits the verdict, and that is
/// derivation rather than invention.**
#[test]
fn both_copies_of_one_examined_sheet_are_subtracted() {
    let job = job_of(&[(3, true), (3, true)]);
    let context = context();
    let mut verdicts = Verdicts::default();

    examine(&mut verdicts, &context, &job, 0, Overhang::BlankBand);

    assert_eq!(
        verdicts.claim(&context, &job, &page_sizes(8)),
        ClipClaim::None,
        "the second copy is the same page under the same placement rendered from the same \
         raster; counting it as unexamined would be pretending not to know something"
    );
}

// ===========================================================================
// The key — every one of these is a verdict that must NOT survive
// ===========================================================================

/// **A rendering setting changes ⇒ every verdict is void.**
#[test]
fn a_verdict_does_not_survive_a_change_of_rendering_settings() {
    let job = job_of(&[(0, true)]);
    let before = context();
    let mut verdicts = Verdicts::default();
    examine(&mut verdicts, &before, &job, 0, Overhang::BlankBand);
    assert_eq!(
        verdicts.claim(&before, &job, &page_sizes(8)),
        ClipClaim::None
    );

    let mut moved = pdfcer_core::settings::Settings::default();
    moved.cmyk_intent = pdfcer_core::settings::CmykIntent::NeutralBlack;
    // The perturbation has to LAND. One that happened to equal the default
    // would make this test pass by never testing anything, which reads
    // exactly like a cache key that works.
    assert_ne!(
        moved.cmyk_intent,
        pdfcer_core::settings::Settings::default().cmyk_intent
    );
    let after = Context::new(
        pdfcer_render::AnnotationScope::Document,
        &moved,
        (600.0, 780.0),
        None,
    );

    assert_eq!(
        verdicts.claim(&after, &job, &page_sizes(8)),
        ClipClaim::Geometric(1),
        "a page rendered under different settings is a different page; the remembered \
         'blank' describes pixels that no longer exist"
    );
}

/// **The annotation scope changes ⇒ every verdict is void.**
#[test]
fn a_verdict_does_not_survive_a_change_of_annotation_scope() {
    let job = job_of(&[(0, true)]);
    let before = context();
    let mut verdicts = Verdicts::default();
    examine(&mut verdicts, &before, &job, 0, Overhang::BlankBand);

    let after = Context::new(
        pdfcer_render::AnnotationScope::DocumentAndMarkups,
        &pdfcer_core::settings::Settings::default(),
        (600.0, 780.0),
        None,
    );
    assert_eq!(
        verdicts.claim(&after, &job, &page_sizes(8)),
        ClipClaim::Geometric(1),
        "markup can be printed out in the border; a verdict taken without it says nothing \
         about the page that is now being printed"
    );
}

/// **The printable rectangle changes ⇒ every verdict is void.**
#[test]
fn a_verdict_does_not_survive_a_change_of_printable_area() {
    let job = job_of(&[(0, true)]);
    let before = context();
    let mut verdicts = Verdicts::default();
    examine(&mut verdicts, &before, &job, 0, Overhang::BlankBand);

    let after = Context::new(
        pdfcer_render::AnnotationScope::Document,
        &pdfcer_core::settings::Settings::default(),
        (560.0, 740.0),
        None,
    );
    assert_eq!(
        verdicts.claim(&after, &job, &page_sizes(8)),
        ClipClaim::Geometric(1),
        "a narrower printable area crops further into the page: the band the verdict was \
         taken over is not the band that will be cropped"
    );
}

/// **The placement changes ⇒ that sheet's verdict is void**, and this is
/// the one the texture's own key would have missed.
#[test]
fn a_verdict_does_not_survive_a_change_of_placement() {
    let context = context();
    let before = job_of(&[(0, true)]);
    let mut verdicts = Verdicts::default();
    examine(&mut verdicts, &context, &before, 0, Overhang::BlankBand);
    assert_eq!(
        verdicts.claim(&context, &before, &page_sizes(8)),
        ClipClaim::None
    );

    // The same page, the same raster, scaled up onto the sheet: a different
    // part of it now falls outside the printable rectangle.
    let mut after = before.clone();
    after.plans[0].placement.scale = 1.4;

    assert_eq!(
        verdicts.claim(&context, &after, &page_sizes(8)),
        ClipClaim::Geometric(1),
        "the pixels are unchanged and the BAND is not; a verdict keyed only on what the \
         texture is keyed on would have survived this and answered for the wrong strip"
    );
}

/// **The page's own size changes ⇒ that sheet's verdict is void.**
#[test]
fn a_verdict_does_not_survive_the_page_being_resized() {
    let context = context();
    let job = job_of(&[(0, true)]);
    let mut verdicts = Verdicts::default();
    verdicts.remember(
        &context,
        &job.plans[0],
        &[(612.0, 792.0)],
        Overhang::BlankBand,
    );
    assert_eq!(
        verdicts.claim(&context, &job, &[(612.0, 792.0)]),
        ClipClaim::None
    );

    assert_eq!(
        verdicts.claim(&context, &job, &[(1224.0, 1584.0)]),
        ClipClaim::Geometric(1),
        "an ANSI D sheet is not the US Letter the verdict was taken on"
    );
}

/// **A verdict is remembered for the page it names, not for a position in
/// the plan list.**
#[test]
fn a_verdict_is_filed_under_the_document_page_and_not_the_send_position() {
    let context = context();
    let job = job_of(&[(7, true), (2, true)]);
    let mut verdicts = Verdicts::default();
    examine(&mut verdicts, &context, &job, 0, Overhang::BlankBand);

    assert_eq!(
        verdicts.claim(&context, &job, &page_sizes(8)),
        ClipClaim::AtMost(1),
        "page 7's verdict must apply to page 7 wherever it sits in the send order"
    );
}

/// A plan naming a page the document no longer has records nothing and is
/// counted as unexamined — a state one frame of a page-range edit can pass
/// through.
#[test]
fn a_plan_naming_a_missing_page_records_nothing_rather_than_panicking() {
    let context = context();
    let job = job_of(&[(9, true)]);
    let mut verdicts = Verdicts::default();
    verdicts.remember(&context, &job.plans[0], &page_sizes(2), Overhang::BlankBand);

    assert_eq!(
        verdicts.claim(&context, &job, &page_sizes(2)),
        ClipClaim::Geometric(1),
        "no page size, no sheet identity, no claim"
    );
}

// ===========================================================================
// The decision table itself
// ===========================================================================

/// **Every arm of the claim decision, stated as a table.**
#[test]
fn the_claim_decision_table_holds_in_every_arm() {
    let cases = [
        // (geometric, known_blank, unresolved) -> claim
        ((0, 0, 0), ClipClaim::None),
        ((2, 2, 0), ClipClaim::None),
        ((3, 0, 3), ClipClaim::Geometric(3)),
        ((3, 0, 1), ClipClaim::Geometric(3)),
        ((3, 0, 0), ClipClaim::Measured(3)),
        ((5, 2, 0), ClipClaim::Measured(3)),
        ((5, 1, 3), ClipClaim::AtMost(4)),
    ];
    for ((geometric, blank, unresolved), expected) in cases {
        assert_eq!(
            ClipClaim::from_counts(geometric, blank, unresolved),
            expected,
            "geometric={geometric} known_blank={blank} unresolved={unresolved}"
        );
    }
}

/// **The count is a CEILING on what will be lost, and never a claim that
/// something is safe.**
///
/// The property, over the whole decision table rather than over one instance:
///
/// ```text
/// known_inked  ≤  displayed  ≤  geometric
/// ```
///
/// The right inequality is what stops the correction ever inventing a clip;
/// the left is what stops it ever hiding one. A future edit that made the
/// count "smarter" by guessing about unexamined sheets would break the left
/// half, which is the half with the operator's paper behind it.
#[test]
fn the_count_is_never_below_what_is_known_lost_nor_above_the_geometric_count() {
    for geometric in 0..6usize {
        for blank in 0..=geometric {
            for inked in 0..=(geometric - blank) {
                let unresolved = geometric - blank - inked;
                let claim = ClipClaim::from_counts(geometric, blank, unresolved);
                assert!(
                    claim.count() >= inked,
                    "{claim:?} hides {inked} sheets known to lose ink"
                );
                assert!(
                    claim.count() <= geometric,
                    "{claim:?} invents a clip beyond the {geometric} the geometry reports"
                );
                assert_eq!(
                    claim.count(),
                    inked + unresolved,
                    "the displayed number is exactly known_inked + unexamined"
                );
            }
        }
    }
}

/// The four claim states put four different things on the button, and the
/// trace word separates them from outside the process.
#[test]
fn each_claim_state_is_distinguishable_in_the_trace() {
    let words = [
        ClipClaim::None.trace_word(),
        ClipClaim::Geometric(1).trace_word(),
        ClipClaim::Measured(1).trace_word(),
        ClipClaim::AtMost(1).trace_word(),
    ];
    let mut seen = words.to_vec();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(
        seen.len(),
        words.len(),
        "two states share a trace word: {words:?}"
    );
    assert_eq!(ClipClaim::None.count(), 0);
}
