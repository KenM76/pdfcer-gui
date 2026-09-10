//! # `dialogs::redact::disclosures` — the parts of the engine's report that
//! reached nobody
//!
//! ★★★ **Added 2026-09-09.** `pdfcer-core` at `369d4de` shipped
//! `CarrierAction::CheckedClean`, three whole-file-sweep counters and — since
//! long before that — a `RedactionReport::notes` field, and the engine-API
//! drift gate showed that this shell read **none of the four**. Not "read them
//! and decided not to show them": never read them. `report.carriers` was
//! touched at exactly two sites and both were the same `==` filter against
//! `DisclosedNotScrubbed`; `report.notes` was touched nowhere in the crate.
//!
//! ## Why a file of its own, and where the seam is
//!
//! `dialogs/redact.rs` stood at 1,453 lines, forty-seven under rule R2's
//! ceiling, and the three blocks below plus their arguments are about sixty. So
//! a split was compulsory; what follows is why *this* split rather than a
//! mechanical one.
//!
//! The seam is **"a disclosure derived from the engine's report, drawn into the
//! report body, that gates nothing."** Every block here reads
//! [`RedactionReport`] and paints; none of them contributes to
//! [`super::residual_lines`], none is acknowledgeable, none can block the
//! confirm control. That is a genuinely different job from the rest of the
//! dialog, which is a transaction: a destination choice, two acknowledgements
//! and an irreversible verb. Keeping the two apart means an edit to the wording
//! of a disclosure cannot reach the transaction by accident.
//!
//! ★ **The derivations live here too**, and that is deliberate but conditional.
//! `crate::redact` is where a derivation belongs when two surfaces must agree
//! on it — [`crate::redact::residual_count`]'s doc comment argues that at length
//! and it is right. Nothing else consumes these three; the moment something
//! does, they move down into `crate::redact` and this file keeps only the
//! painting. Putting them there today would have been a layer for one caller.
//!
//! ## ★★ Rule 1 and the reassuring sentence
//!
//! [`crate::text::redact`]'s rule 1 — *never say "removed" without
//! qualification when anything was left* — is the reason the census in
//! [`checked_clean`] is safe to draw at all. It is not an "all clear". It is
//! drawn from `CheckedClean`, the residual section is drawn from
//! `DisclosedNotScrubbed`, and the two are different verdicts on different
//! carriers: there is no report in which showing one suppresses the other.
//! A future edit that made this block conditional on the residual list being
//! empty would break that property, and would be the defect this note exists to
//! prevent.
//!
//! ## ★★★ What guards the wiring, and it is not a test
//!
//! Everything below paints into a `&mut Ui`, which is not an oracle, so no unit
//! test in this crate can observe that any of it is **drawn**. The derivations
//! are tested — [`checked_clean_names`] is separated out precisely so one of
//! them can be — but a build where all three call sites in
//! [`super::RedactDialog::report`] had been deleted would pass every test in the
//! workspace, which is the exact shape of the defect this module closes.
//!
//! The guard is **`cargo clippy --workspace --all-targets -- -D warnings`**, and
//! it was measured on 2026-09-09 rather than assumed: commenting out the three
//! call sites produces five `never used` warnings — [`sweep`],
//! [`checked_clean`], [`checked_clean_names`], [`engine_notes`] and
//! [`REGION_ENGINE_NOTES`] — and `-D warnings` turns each into an error. That
//! works only because every item here is `pub(super)` or private. **If one of
//! these is ever raised to `pub`, the tripwire goes silent for it**, and the
//! function can be orphaned in a green build. Do not raise them without
//! replacing the guard with a driven `ui-verify` assertion.
//!
//! ⚠ A driven assertion is still owed regardless: dead-code analysis proves the
//! call exists, not that the label reaches a pixel the operator can read. That
//! is the same gap the project's own note about panels shipping unreachable was
//! written for, and it is open.

use egui_shell::theme::Theme;
use pdfcer_core::redact::{CarrierAction, RedactionReport};

use crate::text::redact as t;

/// The collapsed notes section's widget id.
///
/// A stable salt rather than the label, because the label carries the note
/// **count** and egui derives a `CollapsingHeader`'s open/closed state from its
/// id: without this, a report with three notes and a report with four would be
/// two different widgets, and the section would spring back open every time the
/// count changed.
const REGION_ENGINE_NOTES: &str = "redact-apply-engine-notes"; // ui-text-exempt: widget id salt, never displayed

// ---------------------------------------------------------------------------
// 1. What the whole-file sweep will remove
// ---------------------------------------------------------------------------

/// **The whole-file sweep's own counts**, drawn with the other removal counts.
///
/// Placed immediately after `info_scrubbed` in the body, because the two answer
/// the same question about two different halves of the file: that one says what
/// the trailer's document-information dictionary carried, this one says what
/// everything else carried. The engine counts them separately for exactly that
/// reason, and its own words on the field are *"A single total would hide the
/// fact that the second number is the one nobody expected to be non-zero."*
///
/// ★ Drawn only when the sweep actually edited something. `objects` is the
/// engine's total — dictionaries scrubbed, plus metadata packets blanked, plus
/// content streams blanked — so it is zero exactly when the sweep changed
/// nothing, and a "0 objects will be scrubbed" line on an ordinary redaction
/// would be noise in a report whose warnings have to keep their force.
pub(super) fn sweep(ui: &mut egui::Ui, report: &RedactionReport) {
    if report.residual_sweep_objects_scrubbed == 0 {
        return;
    }
    ui.add_space(4.0);
    ui.label(t::sweep_scrubbed_line(
        report.residual_sweep_entries_scrubbed,
        report.residual_sweep_objects_scrubbed,
        report.residual_content_streams_blanked,
    ));
}

// ---------------------------------------------------------------------------
// 2. What pdfcer checked and found clean
// ---------------------------------------------------------------------------

/// **The diligence census** — every carrier the engine looked inside and found
/// nothing in.
///
/// Drawn with the proof, under [`crate::text::redact::verified_line`], because
/// it is the same kind of statement: evidence that a check happened. It is
/// muted rather than plain, one step below the verification line, because the
/// verification line is this shell's own sweep of the finished bytes and this
/// is the engine's sweep of the carriers — related, and not equally strong.
///
/// ★★★ The whole argument for its existence is the engine's, on the variant:
///
/// > *"a shell that tells an operator 'nothing to do' when the truth is
/// > 'checked, clean' has taken away the one thing that distinguishes a
/// > diligence sweep from a no-op."*
///
/// The derivation is [`checked_clean_names`], separated from the painting so a
/// test can observe it: a `&mut Ui` is not an oracle, and a check that had to
/// read pixels to prove a filter reads the right variant would be measuring the
/// wrong thing.
pub(super) fn checked_clean(ui: &mut egui::Ui, theme: &Theme, report: &RedactionReport) {
    let names = checked_clean_names(report);
    if names.is_empty() {
        return;
    }
    ui.add_space(4.0);
    ui.label(egui::RichText::new(t::checked_clean_line(&names)).color(theme.palette.text_muted));
}

/// **The carriers the engine checked and found nothing in**, already in the
/// operator's words.
///
/// ★★★ The filter is `== CarrierAction::CheckedClean` and it is the *third*
/// distinct verdict this shell now distinguishes, which is the whole point of
/// the variant: `Absent` means the carrier is not in the document, `Scrubbed`
/// means pdfcer took something out of it, and this means pdfcer looked and
/// there was nothing to take. Before 2026-09-09 the third was folded into
/// silence.
///
/// ⚠ `CarrierAction` is `#[non_exhaustive]`, so this cannot be written as an
/// exhaustive `match` and a sixth variant would land here as neither
/// `CheckedClean` nor `DisclosedNotScrubbed` — invisible, exactly as
/// `CheckedClean` was. **The tripwire is `tools/gates/check-engine-api-drift.sh`,
/// not this file:** that gate is what surfaced `CheckedClean`, and it fails on
/// any engine item this crate does not consume or exempt. There is no way to
/// write the guard locally, so the guard is the gate, and this note is here so
/// the next person does not go looking for one.
#[must_use]
pub(super) fn checked_clean_names(report: &RedactionReport) -> Vec<&'static str> {
    report
        .carriers
        .iter()
        .filter(|c| c.action == CarrierAction::CheckedClean)
        .map(|c| t::carrier_name(c.carrier))
        .collect()
}

// ---------------------------------------------------------------------------
// 3. The engine's own notes
// ---------------------------------------------------------------------------

/// **`RedactionReport::notes`, at the foot of the report, collapsed.**
///
/// ★★★ These were being discarded, and one of them is load-bearing: when the
/// residual sweep cannot scrub a stream object, the **object numbers** exist
/// only in a note. [`crate::text::redact::residual_sweep_line`] tells the
/// operator to look here for them, which is a promise this function keeps.
///
/// ★★ **Collapsed by default**, and the reason is `OPERATOR_REQUESTS.md` O160 —
/// his report that this dialog's warnings had become something to click past.
/// These notes are the engine's prose: they cite ISO 32000-1 by table number
/// and there can be a dozen on one sheet. Open by default they would bury the
/// residual section under spec citations, which is the same failure in a new
/// place. Closed, they cost one line and lose nothing.
///
/// ★ **Not styled as a warning**, even though some of them are about residuals.
/// The residuals that matter are already lifted out into the danger-coloured
/// section above by their own derivations; painting this section red as well
/// would double-count them and dilute the colour that means "read this".
pub(super) fn engine_notes(ui: &mut egui::Ui, theme: &Theme, report: &RedactionReport) {
    if report.notes.is_empty() {
        return;
    }
    ui.add_space(10.0);
    egui::CollapsingHeader::new(t::engine_notes_heading(report.notes.len()))
        .id_salt(REGION_ENGINE_NOTES)
        .default_open(false)
        .show(ui, |ui| {
            ui.label(egui::RichText::new(t::engine_notes_lead()).color(theme.palette.text_muted));
            ui.add_space(4.0);
            for note in &report.notes {
                // ★ `note` verbatim. A note pdfcer wrote about its own
                // uncertainty is the one thing this report must not paraphrase
                // — and the sentences that needed translating are already
                // translated, above, by the derivations that own them.
                ui.label(note);
                ui.add_space(4.0);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::redact::CarrierStatus;

    /// A report carrying exactly the carriers described.
    fn report_with(carriers: &[(&'static str, CarrierAction)]) -> RedactionReport {
        let mut report = RedactionReport::default();
        report.carriers = carriers
            .iter()
            .map(|(carrier, action)| CarrierStatus {
                carrier,
                present: *action != CarrierAction::Absent,
                action: *action,
            })
            .collect();
        report
    }

    /// ★★★ **`CheckedClean` reaches the census and the other four verdicts do
    /// not.**
    ///
    /// The defect this whole module closes was not a wrong filter — it was **no
    /// filter**, so a test that only asserted `CheckedClean` is picked up would
    /// pass on a build that picked up everything. Both halves are asserted, and
    /// the second half is the one with teeth: `DisclosedNotScrubbed` appearing
    /// here would put a residual into a sentence that says pdfcer found no
    /// trace of it, which is rule 1 broken in the worst available direction.
    #[test]
    fn only_the_checked_clean_carriers_are_called_clean() {
        let report = report_with(&[
            ("info", CarrierAction::CheckedClean),
            ("residual_sweep", CarrierAction::CheckedClean),
            ("xmp", CarrierAction::Scrubbed),
            ("xfa", CarrierAction::DisclosedNotScrubbed),
            ("attachments", CarrierAction::Absent),
            ("object_streams", CarrierAction::DroppedByRewrite),
        ]);
        let names = checked_clean_names(&report);
        assert_eq!(names.len(), 2, "two carriers were checked clean: {names:?}");
        assert!(
            names.iter().all(|n| !n.contains("XFA")),
            "★ a DisclosedNotScrubbed carrier reached the CLEAN census: {names:?}"
        );
        assert!(
            names.iter().any(|n| n.contains("document properties")),
            "the /Info carrier is the commonest CheckedClean of all: {names:?}"
        );
    }

    /// ★★ **The census is empty when nothing was checked clean**, so the block
    /// is drawn on the strength of a measurement and never as decoration.
    ///
    /// Without this, a `checked_clean_names` that returned every carrier would
    /// still satisfy the test above's first assertion if the counts happened to
    /// line up, and a reassuring sentence would appear on a report that had
    /// earned nothing.
    #[test]
    fn a_report_with_nothing_clean_says_nothing() {
        let report = report_with(&[
            ("xfa", CarrierAction::DisclosedNotScrubbed),
            ("xmp", CarrierAction::Scrubbed),
        ]);
        assert!(checked_clean_names(&report).is_empty());
    }

    /// ★ **The census never prints an engine key**, which is the second half of
    /// what this module was written for and is asserted at the derivation
    /// rather than only at the string, because the mapping happens here.
    #[test]
    fn the_census_speaks_english_not_engine_keys() {
        let report = report_with(&[
            ("struct_tree", CarrierAction::CheckedClean),
            ("residual_sweep", CarrierAction::CheckedClean),
            ("ocg", CarrierAction::CheckedClean),
        ]);
        for name in checked_clean_names(&report) {
            for key in ["struct_tree", "residual_sweep", "ocg"] {
                assert!(
                    !name.contains(key),
                    "the raw engine key `{key}` reached the operator in: {name}"
                );
            }
        }
    }
}
