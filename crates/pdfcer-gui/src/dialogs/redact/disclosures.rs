//! # `dialogs::redact::disclosures` — the parts of the engine's report that
//! reached nobody
//!
//!
//! ## Why a file of its own, and where the seam is
//!
//! The blocks below plus their arguments would put `dialogs/redact.rs` over
//! rule R2's ceiling, so a split is compulsory; what follows is why *this*
//! split rather than a mechanical one.
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
//! and it is right. Nothing else consumes these; the moment something does,
//! they move down into `crate::redact` and this file keeps only the painting.
//! Putting them there today would have been a layer for one caller.
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
//! are tested — [`checked_clean_names`] and [`left_by_choice_names`] are
//! separated out precisely so they can be — but a build where every call site in
//! [`super::RedactDialog::report`] had been deleted would pass every test in the
//! workspace, which is the exact shape of the defect this module closes.
//!
//!
//! ⚠ A driven assertion is still owed regardless: dead-code analysis proves the
//! call exists, not that the label reaches a pixel the operator can read. That
//! is the same gap the project's own note about panels shipping unreachable was
//! written for.
//!
//! Section 5 is the only block that closes it. It publishes its rect through
//! [`crate::diag::ui_rect_visible`], so a driven check can tell *drawn and
//! readable* from *drawn behind the fold* from *not drawn at all*; the other
//! four publish nothing and the gap is open for them. The asymmetry is
//! deliberate rather than unfinished: section 5 is the one block whose absence
//! costs content rather than context.

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
// 3. The copies pdfcer found and was told to leave
// ---------------------------------------------------------------------------

/// **The matches a narrower redaction reach declines to act on.**
///
/// [`CarrierAction::FoundNotScrubbed`] — the carrier holds a copy of the marked
/// text, pdfcer can remove it, and the operator's reach setting says not to.
/// This block is what makes offering a narrow reach honest: the setting changes
/// what pdfcer may *change*, never what it *finds* or *reports*, so the operator
/// sees the full census either way and chooses with it in front of him.
///
/// **Notice weight, and the choice of colour is the substance of the block.**
/// Not muted like the clean census, because this is live text surviving into
/// the saved file rather than evidence that a check ran. Not
/// `palette.danger` like the residual section, because nothing failed — the
/// engine keeps this verdict apart from `DisclosedNotScrubbed` precisely so a
/// deliberate scope does not read as a fault, and painting both red would
/// collapse in the product the distinction the engine maintains in the report.
///
/// Drawn **above** the residual section so the danger colour stays last and
/// closest to the confirm control.
///
/// It gates nothing. It cannot enter [`crate::redact::residual_count`], cannot
/// be acknowledged, and cannot block the confirm control — an operator who set
/// a narrow reach and is then refused the save he asked for has been given a
/// setting that does not work.
pub(super) fn left_by_choice(ui: &mut egui::Ui, theme: &Theme, report: &RedactionReport) {
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
///
/// An `==` filter for the same reason [`checked_clean_names`] is one:
/// `CarrierAction` is `#[non_exhaustive]`, a `match` here could only be written
/// with a catch-all, and a catch-all is how a new verdict becomes invisible.
/// The tripwire for a new variant is `tools/gates/check-engine-api-drift.sh`,
/// not this file.
#[must_use]
pub(super) fn left_by_choice_names(report: &RedactionReport) -> Vec<&'static str> {
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

// ---------------------------------------------------------------------------
// 5. The text the marks will destroy
// ---------------------------------------------------------------------------

/// The trace region and key for the block below.
const REGION_REMOVED_TEXT: &str = "redact-apply-removed-text"; // ui-text-exempt: trace region name, never displayed

/// What [`removed_text_lines`] decided, and the lines it decided on.
///
/// The state is carried out of the derivation rather than re-derived by the
/// painter, so the trace reports what this function chose rather than what a
/// second reading of the same report would choose. Those are the same value
/// only while the two readings agree, and a trace that can disagree with the
/// screen is worse than none.
pub(super) struct RemovedText {
    /// `listed`, `no-text` or `unreported` — the trace's word for the branch.
    pub state: &'static str,
    /// How many characters the engine reported across every region, before any
    /// cap and counted in `char`s.
    ///
    /// ★ The trace's only measure of the content, and it exists because the
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
/// ★★ **A safety control, not a convenience.** Every other edit in this program
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
/// ★ **The trace carries the shape and not the text.** `entries` and `state`,
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
pub(super) fn removed_text(ui: &mut egui::Ui, theme: &Theme, report: &RedactionReport) {
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
///
/// ★ **It always returns at least one line.** An empty return would collapse
/// three different states — text found, no text present, and codes removed with
/// no text reported — into one blank area, which reads as *"pdfcer has nothing
/// to say about this"* in all three. The first two are findings and the third
/// is a disclosure; none of them is silence.
///
/// Neither cap cuts silently: [`t::MAX_ENTRIES`] is followed by
/// [`t::removed_text_more`] naming the remainder, and [`t::MAX_CHARS`] is
/// applied inside [`t::removed_text_entry`], which says how many characters of
/// that one region it did not print.
pub(super) fn removed_text_lines(report: &RedactionReport) -> RemovedText {
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

    /// **The two "not scrubbed" verdicts never reach each other's section.**
    ///
    /// This is the assertion with teeth in the whole module. The engine keeps
    /// `FoundNotScrubbed` — *told not to* — apart from `DisclosedNotScrubbed`
    /// — *tried and could not* — and warns that collapsing them makes a
    /// deliberate scope look like a failure and a real failure look like a
    /// preference. Both directions are wrong and both are one `==` away, so
    /// both are asserted: a declined match must not enter the blocking
    /// residual section, and a real failure must not be excused as a setting.
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

    /// A report that removed exactly `texts`, having counted `glyphs` codes.
    ///
    /// Built by assignment rather than a struct literal because
    /// [`RedactionReport`] is `#[non_exhaustive]`, the same reason
    /// [`report_with`] above is shaped this way.
    fn text_report(texts: &[&str], glyphs: u64) -> RedactionReport {
        let mut report = RedactionReport::default();
        report.redacted_text = texts.iter().map(|s| (*s).to_owned()).collect();
        report.glyphs_removed = glyphs;
        report
    }

    /// ★★★ **A line is drawn in all three states.**
    ///
    /// The property the whole block rests on. A state that produced no line
    /// would paint blank space under the heading, and blank space reads as
    /// *"nothing to report"* — true in one of these three cases and false in
    /// the other two.
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

    /// ★★ **Codes counted with no text reported is a disclosure, not a gap.**
    ///
    /// `glyphs_removed` is the only thing separating this from the case above,
    /// and getting that branch backwards would tell the operator a region full
    /// of text contained none — the worst sentence this dialog could print.
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

    /// ★ **The entry cap names its own remainder.** A list that stopped at two
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

    /// ★★ **A long entry is cut on a `char` boundary.**
    ///
    /// Written with a two-byte character on purpose: the strings arriving here
    /// are whatever the document's fonts decoded to, and a byte cut inside a
    /// multi-byte sequence panics the dialog rather than truncating it. The
    /// character count is the assertion; the panic is the failure this test
    /// exists to catch.
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

    /// ★★★ **The three trace tokens are distinct.**
    ///
    /// `state` is what a driven check reads to tell *"the block found text"*
    /// from *"the block found none"*, and two branches sharing a token would
    /// make that check pass on either — an assertion both outcomes satisfy.
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

    /// ★★ **`chars` counts characters and survives both caps.**
    ///
    /// It is the only field of the trace a driven check can compare against
    /// something it knows — the length of the string its own fixture puts on
    /// the page — so counting bytes would make every non-ASCII document
    /// disagree with the check by the length of its encoding, and counting the
    /// drawn lines would make a capped report understate what is going.
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
