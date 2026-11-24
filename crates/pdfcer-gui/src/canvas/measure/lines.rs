//! The two tools whose picks are LINES on the page: Two-line (a linear or
//! angular ce dimension between them) and Centre line (the line midway between
//! them, or the bisector of the angle they make).
//!
//! Both share [`super::state::MeasureState::two_lines`]: a kind switch clears
//! it, so one tool never inherits the other's half-picked pair.

use pdfcer_core::dimension::{CentreLineRefusal, TwoLinePlacement};
use pdfcer_core::vector::Point;
use pdfcer_core::vector::linepick::{TwoLineRelation, pick_line_in_page};
use pdfcer_gui_base::measure::place::Placing;

use super::MeasureKind;
use super::state::MeasureState;
use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::target::CanvasTargetProvider;

/// Take one click of a line-picking tool at page point `p`.
///
/// The pick reads the page's decomposition, which is why `needs_targets` asks
/// for one whenever a measure tool is armed: picking a line is reading the
/// page, not selecting it. A click that hits no line does nothing.
pub(super) fn take(
    st: &mut MeasureState,
    doc: &OpenDoc,
    page_index: usize,
    p: Point,
    targets: Option<&dyn CanvasTargetProvider>,
    map: &PageMapping,
    actions: &mut Vec<Action>,
) {
    let Some(model) = targets.and_then(|t| t.page_objects_model(page_index)) else {
        return;
    };
    let Some(line) = pick_line_in_page(model, p, map.tolerance()) else {
        return;
    };
    // The operator's threshold from `doc.settings`, never
    // `ParallelPolicy::default()`: a literal here would make the persisted
    // Settings value do nothing for the gestures it exists for.
    let epsilon = doc.settings.parallel_epsilon_degrees;
    st.two_lines.offer_line(line, epsilon);
    if st.kind == MeasureKind::CentreLine {
        centre_line(st, doc, epsilon, actions);
    } else {
        two_line(st, doc, epsilon);
    }
}

/// Two lines picked: hand the ce dimension they call for to the placing click,
/// or say by name why they cannot be dimensioned together.
fn two_line(st: &mut MeasureState, doc: &OpenDoc, epsilon: f64) {
    match st.two_lines.authoring(epsilon, TwoLinePlacement::default()) {
        Some(Ok(authoring)) => {
            // The disclosure travels with the action: a note recorded now
            // would be overwritten by the commit's own apply phase.
            let disclosures = crate::text::measure::two_line_reading(
                authoring.forced_parallel,
                authoring.measured_angle_degrees,
                authoring.apex_is_real() == Some(false),
            )
            .into_iter()
            .collect();
            st.placing = Some(Placing {
                kind: authoring.kind,
                disclosures,
            });
            st.two_lines.clear();
        }
        // Nothing is committed, so `record_note` is the only place the
        // sentence can go. The pair is cleared so the next click is a first
        // pick, as the operator will expect.
        Some(Err(refusal)) => {
            crate::app::actions::record_note(
                doc.edit_epoch,
                crate::text::measure::two_line_refused(refusal).to_owned(),
            );
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("two-line-refused reason={refusal:?}")
            });
            st.two_lines.clear();
        }
        None => {}
    }
}

/// Two lines picked: commit the centre line between them at once.
///
/// There is no placing click, because a centre line carries no label. The
/// pair is cleared whatever the outcome.
fn centre_line(st: &mut MeasureState, doc: &OpenDoc, epsilon: f64, actions: &mut Vec<Action>) {
    let Some(result) = st.two_lines.centre_line(epsilon) else {
        return;
    };
    st.two_lines.clear();
    match result {
        Ok(authoring) => {
            let angled = !matches!(authoring.relation, TwoLineRelation::Parallel { .. });
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "centre-line relation={} forced_parallel={} overlap={:.2}",
                    if angled { "angled" } else { "parallel" },
                    u8::from(authoring.forced_parallel),
                    authoring.overlap
                )
            });
            let disclosures = crate::text::measure::centre_line_reading(
                authoring.forced_parallel,
                authoring.measured_angle_degrees,
                angled,
            )
            .into_iter()
            .collect();
            actions.push(Action::Dimension(DimensionAction::Commit {
                page: st.page_index,
                group: st.group,
                kind: authoring.kind,
                disclosures,
            }));
        }
        Err(refusal) => refused(doc, refusal),
    }
}

/// Say by name why no centre line was drawn.
fn refused(doc: &OpenDoc, refusal: CentreLineRefusal) {
    crate::app::actions::record_note(
        doc.edit_epoch,
        crate::text::measure::centre_line_refused(refusal).to_owned(),
    );
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!("centre-line-refused reason={refusal:?}")
    });
}
