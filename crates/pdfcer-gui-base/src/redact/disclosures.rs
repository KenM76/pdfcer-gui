//! # `redact::disclosures` — the parts of the engine's report that
//! reached nobody
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/redact/disclosures.md`.

use egui_shell::theme::Theme;
use pdfcer_core::redact::{CarrierAction, RedactionReport};

use crate::text::redact as t;

/// The collapsed notes section's widget id.
const REGION_ENGINE_NOTES: &str = "redact-apply-engine-notes"; // ui-text-exempt: widget id salt, never displayed

// ---------------------------------------------------------------------------
// 1. What the whole-file sweep will remove
// ---------------------------------------------------------------------------

/// **The whole-file sweep's own counts**, drawn with the other removal counts.
pub fn sweep(ui: &mut egui::Ui, report: &RedactionReport) {
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
pub fn checked_clean(ui: &mut egui::Ui, theme: &Theme, report: &RedactionReport) {
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
pub fn checked_clean_names(report: &RedactionReport) -> Vec<&'static str> {
    report
        .carriers
        .iter()
        .filter(|c| c.action == CarrierAction::CheckedClean)
        .map(|c| t::carrier_name(c.carrier))
        .collect()
}

// ---------------------------------------------------------------------------
// 3. The copies pdfcer found and was told to leave
// ---------------------------------------------------------------------------

/// **The matches a narrower redaction reach declines to act on.**
pub fn left_by_choice(ui: &mut egui::Ui, theme: &Theme, report: &RedactionReport) {
    let names = left_by_choice_names(report);
    if names.is_empty() {
        return;
    }
    ui.add_space(10.0);
    ui.separator();
    ui.label(egui::RichText::new(t::left_by_choice_line(&names)).color(theme.palette.notice));
}

/// **The carriers holding a copy that the reach setting leaves alone**, already
/// in the operator's words.
#[must_use]
pub fn left_by_choice_names(report: &RedactionReport) -> Vec<&'static str> {
    report
        .carriers
        .iter()
        .filter(|c| c.action == CarrierAction::FoundNotScrubbed)
        .map(|c| t::carrier_name(c.carrier))
        .collect()
}

// ---------------------------------------------------------------------------
// 4. The engine's own notes
// ---------------------------------------------------------------------------

/// **`RedactionReport::notes`, at the foot of the report, collapsed.**
pub fn engine_notes(ui: &mut egui::Ui, theme: &Theme, report: &RedactionReport) {
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
                // `note` verbatim. A note pdfcer wrote about its own
                // uncertainty is the one thing this report must not paraphrase
                // — and the sentences that needed translating are already
                // translated, above, by the derivations that own them.
                ui.label(note);
                ui.add_space(4.0);
            }
        });
}

// ---------------------------------------------------------------------------
// 5. The text the marks will destroy
// ---------------------------------------------------------------------------

/// The trace region and key for the block below.
const REGION_REMOVED_TEXT: &str = "redact-apply-removed-text"; // ui-text-exempt: trace region name, never displayed

/// What [`removed_text_lines`] decided, and the lines it decided on.
pub struct RemovedText {
    /// `listed`, `no-text` or `unreported` — the trace's word for the branch.
    pub state: &'static str,
    /// How many characters the engine reported across every region, before any
    /// cap and counted in `char`s.
    ///
    /// The trace's only measure of the content, and it exists because the
    /// content itself must not reach a log file. A driven check that knows what
    /// its fixture says can compare this against that length and catch a build
    /// that listed the wrong strings; a check that only read `entries` could
    /// not tell the right text from any text.
    pub chars: usize,
    /// Every line to draw, in order. Never empty.
    pub lines: Vec<String>,
}

/// **What will be removed, in words rather than in counts.**
///
/// `OPERATOR_REQUESTS.md` **O217**, fourth bullet. Drawn after every count and
/// before the residual sections: the counts answer *how much*, this answers
/// *what*, and only the second can be checked against an intention.
///
/// **A safety control, not a convenience.** Every other edit in this program
/// costs an undo; this one costs the content. The shell's unit of text
/// selection is the visual line, and `G032` records that the engine groups a
/// line with no horizontal-gap criterion, so a bill-of-materials row welds its
/// item number, part number, description and quantity into one selectable
/// thing — an operator who marks the quantity has marked the row. R8b forbids
/// saying so on the canvas, and is right to: the mark must render as it will
/// render. This list is the only place that over-take can be seen while it is
/// still reversible, and the only form that makes it obvious is the
/// neighbours' own words.
///
/// **The trace carries the shape and not the text.** `entries` and `state`,
/// never a character of what was removed: `PDFCER_DIAG` writes to stderr and
/// gets redirected into log files, and a redaction surface that copies the
/// operator's confidential strings into a second file has undone its own job.
/// A driven check can therefore prove this block was drawn, was reachable, and
/// found text — not that the strings it printed were the right ones. That is
/// the harness's standing limit here, and it is stated in the check.
///
/// [`ui_rect_visible`] rather than [`ui_rect`]: the block sits at the bottom of
/// a scrolling report, which is precisely where a region gets published while
/// being unreadable.
///
/// [`ui_rect_visible`]: crate::diag::ui_rect_visible
/// [`ui_rect`]: crate::diag::ui_rect
pub fn removed_text(ui: &mut egui::Ui, theme: &Theme, report: &RedactionReport) {
    let drawn = removed_text_lines(report);

    let block = ui.vertical(|ui| {
        ui.add_space(6.0);
        ui.label(t::removed_text_heading());
        ui.add_space(2.0);
        ui.label(egui::RichText::new(t::removed_text_lead()).color(theme.palette.text_muted));
        ui.add_space(4.0);
        for line in &drawn.lines {
            ui.label(line);
        }
    });

    crate::diag::ui_rect_visible(REGION_REMOVED_TEXT, block.response.rect, ui.clip_rect());
    crate::diag::trace_on_change(REGION_REMOVED_TEXT, || {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "state={} entries={} chars={} lines={}",
            drawn.state,
            report.redacted_text.len(),
            drawn.chars,
            drawn.lines.len()
        )
    });
}

/// Every line [`removed_text`] will draw, in order, and which branch drew them.
pub fn removed_text_lines(report: &RedactionReport) -> RemovedText {
    // Counted over the whole vector rather than over `lines`, so that a report
    // long enough to hit either cap still measures what is GOING rather than
    // what is shown. The two numbers diverging is the caps working.
    let chars = report
        .redacted_text
        .iter()
        .map(|text| text.chars().count())
        .sum();

    if report.redacted_text.is_empty() {
        return if report.glyphs_removed == 0 {
            RemovedText {
                // ui-text-exempt: trace token, never displayed.
                state: "no-text",
                chars,
                lines: vec![t::removed_text_none().to_owned()],
            }
        } else {
            RemovedText {
                // ui-text-exempt: trace token, never displayed.
                state: "unreported",
                chars,
                lines: vec![t::removed_text_undecodable(report.glyphs_removed)],
            }
        };
    }

    let mut lines: Vec<String> = report
        .redacted_text
        .iter()
        .take(t::MAX_ENTRIES)
        .map(|text| t::removed_text_entry(text))
        .collect();

    if report.redacted_text.len() > t::MAX_ENTRIES {
        lines.push(t::removed_text_more(
            report.redacted_text.len() - t::MAX_ENTRIES,
        ));
    }

    RemovedText {
        // ui-text-exempt: trace token, never displayed.
        state: "listed",
        chars,
        lines,
    }
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

    /// **`CheckedClean` reaches the census and the other four verdicts do
    /// not.**
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

    /// **The census is empty when nothing was checked clean**, so the block
    /// is drawn on the strength of a measurement and never as decoration.
    #[test]
    fn a_report_with_nothing_clean_says_nothing() {
        let report = report_with(&[
            ("xfa", CarrierAction::DisclosedNotScrubbed),
            ("xmp", CarrierAction::Scrubbed),
        ]);
        assert!(checked_clean_names(&report).is_empty());
    }

    /// **The two "not scrubbed" verdicts never reach each other's section.**
    #[test]
    fn a_declined_match_and_a_real_failure_are_never_the_same_list() {
        let report = report_with(&[
            ("info", CarrierAction::FoundNotScrubbed),
            ("xmp", CarrierAction::FoundNotScrubbed),
            ("xfa", CarrierAction::DisclosedNotScrubbed),
            ("struct_tree", CarrierAction::CheckedClean),
            ("attachments", CarrierAction::Scrubbed),
        ]);
        let declined = left_by_choice_names(&report);
        assert_eq!(
            declined.len(),
            2,
            "two carriers were found and left by choice: {declined:?}"
        );
        assert!(
            declined.iter().all(|n| !n.contains("XFA")),
            "a carrier pdfcer COULD NOT scrub was excused as a setting: {declined:?}"
        );
        assert!(
            checked_clean_names(&report)
                .iter()
                .all(|n| !declined.contains(n)),
            "a carrier holding a surviving copy was also called clean"
        );
    }

    /// **The declined list is empty on every report from a default reach**, so
    /// the block is drawn on a measurement and never as decoration.
    #[test]
    fn a_report_with_nothing_declined_says_nothing() {
        let report = report_with(&[
            ("info", CarrierAction::Scrubbed),
            ("xfa", CarrierAction::DisclosedNotScrubbed),
            ("xmp", CarrierAction::CheckedClean),
        ]);
        assert!(left_by_choice_names(&report).is_empty());
    }

    /// **The census never prints an engine key**, which is the second half of
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

    /// A report that removed exactly `texts`, having counted `glyphs` codes.
    fn text_report(texts: &[&str], glyphs: u64) -> RedactionReport {
        let mut report = RedactionReport::default();
        report.redacted_text = texts.iter().map(|s| (*s).to_owned()).collect();
        report.glyphs_removed = glyphs;
        report
    }

    /// **A line is drawn in all three states.**
    #[test]
    fn a_line_is_drawn_whatever_the_report_says() {
        for report in [
            text_report(&[], 0),
            text_report(&[], 12),
            text_report(&["BRACKET"], 7),
        ] {
            assert!(!removed_text_lines(&report).lines.is_empty());
        }
    }

    /// Marks that hold no text say so, rather than saying nothing.
    #[test]
    fn marks_over_line_work_report_that_they_hold_no_text() {
        let drawn = removed_text_lines(&text_report(&[], 0));
        assert_eq!(drawn.state, "no-text");
        assert_eq!(drawn.lines, vec![t::removed_text_none()]);
    }

    /// **Codes counted with no text reported is a disclosure, not a gap.**
    #[test]
    fn codes_removed_without_any_text_reported_are_disclosed() {
        let drawn = removed_text_lines(&text_report(&[], 37));
        assert_eq!(drawn.state, "unreported");
        assert_eq!(drawn.lines.len(), 1);
        assert!(
            drawn.lines[0].contains("37"),
            "the count is not in: {}",
            drawn.lines[0]
        );
        assert_ne!(drawn.lines[0], t::removed_text_none());
    }

    /// Every distinct string the engine reported reaches a line of its own,
    /// quoted so a cell's padding is visible as part of what went.
    #[test]
    fn each_reported_string_reaches_its_own_quoted_line() {
        let drawn = removed_text_lines(&text_report(&["1BRACKET", " 4 "], 11));
        assert_eq!(drawn.state, "listed");
        assert_eq!(drawn.lines.len(), 2);
        assert!(drawn.lines[0].contains("1BRACKET"));
        assert_eq!(drawn.lines[1], "\u{201c} 4 \u{201d}");
    }

    /// **The entry cap names its own remainder.** A list that stopped at two
    /// hundred without saying so would be a silent truncation on the one
    /// surface in this program where silence costs content.
    #[test]
    fn the_entry_cap_says_how_many_it_did_not_list() {
        let texts: Vec<String> = (0..t::MAX_ENTRIES + 3)
            .map(|n| format!("run {n}"))
            .collect();
        let borrowed: Vec<&str> = texts.iter().map(String::as_str).collect();

        let drawn = removed_text_lines(&text_report(&borrowed, 999));
        assert_eq!(drawn.lines.len(), t::MAX_ENTRIES + 1);
        let last = drawn.lines.last().expect("the cap line");
        assert!(last.contains('3'), "the remainder is not in: {last}");
    }

    /// **A long entry is cut on a `char` boundary.**
    #[test]
    fn a_long_entry_is_cut_on_a_char_boundary_not_a_byte_one() {
        // One ASCII character in front of the two-byte run, so that the cap's
        // byte offset lands INSIDE a sequence rather than between two. Without
        // it every boundary is even, a byte cut is legal, and the wrong
        // implementation truncates quietly instead of panicking.
        let long = format!("a{}", "\u{00e9}".repeat(t::MAX_CHARS + 5));
        let drawn = removed_text_lines(&text_report(&[&long], 0));

        assert_eq!(drawn.lines.len(), 1);
        assert_eq!(
            drawn.lines[0].matches('\u{00e9}').count(),
            t::MAX_CHARS - 1,
            "the cut landed somewhere other than the character cap"
        );
        assert!(
            drawn.lines[0].contains('6'),
            "the six characters it dropped are not named in: {}",
            drawn.lines[0]
        );
    }

    /// **The three trace tokens are distinct.**
    #[test]
    fn each_branch_reports_a_different_state_to_the_trace() {
        let states = [
            removed_text_lines(&text_report(&[], 0)).state,
            removed_text_lines(&text_report(&[], 9)).state,
            removed_text_lines(&text_report(&["x"], 1)).state,
        ];
        for (index, state) in states.iter().enumerate() {
            assert!(
                !states[index + 1..].contains(state),
                "two branches both report state={state}"
            );
        }
    }

    /// **`chars` counts characters and survives both caps.**
    #[test]
    fn the_traced_character_count_measures_what_goes_not_what_is_shown() {
        let plain = removed_text_lines(&text_report(&["abc", "de"], 5));
        assert_eq!(plain.chars, 5);

        let accented = removed_text_lines(&text_report(&["\u{00e9}\u{00e9}"], 2));
        assert_eq!(accented.chars, 2, "the count is in bytes, not characters");

        let long = "x".repeat(t::MAX_CHARS * 2);
        let capped = removed_text_lines(&text_report(&[&long], 0));
        assert_eq!(
            capped.chars,
            t::MAX_CHARS * 2,
            "the character cap reached the count of what is going"
        );
    }
}
