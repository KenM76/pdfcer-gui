//! # `app::actions::preflightfix` — a Preflight fix through the edit funnel
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/preflightfix.md`.

use pdfcer_core::edit::{EditError, EditSession};
use pdfcer_core::preflight::{FixupReport, FixupScope, FixupTools, Profile};
use pdfcer_emf::RecordingFlattener;
use pdfcer_gui_base::preflightfix::{FixNote, PreflightFix};
use pdfcer_render::convert_colours::IcccerConverter;

use crate::app::state::OpenDoc;

/// Run one Fix or Fix all click as one undo step, then park the engine's
/// report where the panel reads it.
///
/// The engine's fixup verbs never fail, so the closure is always `Ok`; a
/// session another thread still holds is the funnel's refusal, and leaves no
/// note because nothing ran.
pub(super) fn apply(doc: &mut OpenDoc, fix: &PreflightFix) {
    let page = doc.view.page_index;
    let operands = fix.report.findings.len();
    let mut done: Option<FixupReport> = None;
    super::funnel::vector_edit(doc, "preflight-fix", page, operands, |session| {
        let report = run(session, fix);
        let disclosures = report
            .steps
            .iter()
            .flat_map(|s| s.disclosures.iter().cloned())
            .collect();
        done = Some(report);
        Ok::<_, EditError>(disclosures)
    });
    let Some(report) = done else {
        return;
    };
    let scope = fix.rule.as_deref().unwrap_or("all"); // ui-text-exempt: a trace token
    crate::diag::trace(|| {
        let outcomes: Vec<String> = report
            .steps
            .iter()
            .map(|s| format!("{}:{}", s.fixup, s.outcome.as_str()))
            .collect();
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "preflight-fixed scope={scope} steps={} changed={} undo={} outcomes={}",
            report.steps.len(),
            u8::from(report.changed()),
            report.undo_entries,
            outcomes.join(",")
        )
    });
    doc.preflight_fixed = Some(FixNote {
        epoch: doc.edit_epoch,
        text: report.to_text(),
    });
}

/// The engine call: one rule's fixups, or every rule's.
///
/// The colour converter and the transparency flattener are supplied, so the
/// convert-colours and flatten fixups can apply; core cannot reach the
/// renderer and takes both as trait objects.
fn run(session: &mut EditSession, fix: &PreflightFix) -> FixupReport {
    let converter = IcccerConverter::new();
    let flattener = RecordingFlattener::new();
    let tools = FixupTools::new()
        .with_converter(&converter)
        .with_flattener(&flattener);
    let Some(rule_id) = fix.rule.as_deref() else {
        return session.apply_profile_fixups(&fix.profile, &fix.report, &tools, true);
    };
    let Some(rule) = fix.profile.rules.iter().find(|r| r.id == rule_id) else {
        return FixupReport::default();
    };
    let fixups = pdfcer_gui_base::preflightfix::rule_fixups(rule);
    if let [only] = fixups.as_slice() {
        let scope = FixupScope::Rule {
            rule_id,
            findings: &fix.report.findings,
        };
        return session.apply_fixup(only, &scope, &tools);
    }
    // Several fixups on one rule: run them through the profile verb over a
    // one-rule profile, which keeps them one undo step.
    let one = Profile {
        rules: vec![rule.clone()],
        ..fix.profile.clone()
    };
    session.apply_profile_fixups(&one, &fix.report, &tools, true)
}
