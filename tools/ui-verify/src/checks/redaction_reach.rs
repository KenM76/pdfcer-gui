//! # `checks::redaction_reach` — the reach setting decides what survives the
//! save, measured on the bytes, twice
//!
//! ## The defect this exists to catch
//!
//! The preference exists, the settings window offers it, the enum maps onto
//! the engine's `ResidualScope` — and the value never arrives. Every unit test
//! under `crates/pdfcer-gui/src/redact/` calls the apply verb with a reach
//! handed to it directly; none can see the chain in front of that verb:
//! `preferences.txt` → the loaded preference → the action → the dialog →
//! `EditSession::set_residual_scope`. A build where any link dropped the value
//! passes all of them, ships a control that does nothing, and removes text the
//! operator explicitly asked it to leave.
//!
//! That is the sharp direction of the failure, and it is why this is driven:
//! the operator's complaint was *"content I did not select was removed"*, and a
//! setting that silently does not apply is that complaint again with a control
//! painted on top of it.
//!
//! ## The fixture puts one string in two places, and that is the whole design
//!
//! * page 1 **draws** [`SECRET`] — the copy that gets marked, which every reach
//!   must remove;
//! * the trailer's `/Info` dictionary holds a **second copy** in `/Title` — the
//!   copy no viewer shows, whose fate the reach setting decides;
//! * page 2 draws [`SURVIVOR`] and is never marked.
//!
//! `/Info` is not an exotic carrier. A CAD exporter writes the drawing title
//! into it, and on this operator's files that title is very often the words he
//! is redacting.
//!
//! ## The three byte assertions, and which one is the verdict
//!
//! | needle | narrow run | wide run | what a wrong answer means |
//! |---|---|---|---|
//! | `/Title (SECRET)` | present | absent | **the verdict.** Present in both: the setting is inert in the widening direction. Absent in both: inert in the narrowing direction — which is the original complaint |
//! | `(SECRET) Tj` | absent | absent | the marked copy on the page survived a redaction that reported success. No reach makes that acceptable |
//! | `SURVIVOR` | present | present | the control. It is drawn on a page nobody marked, so its absence means either the removal took a page it was never given, or the output's streams are compressed — in which case the rows above are absences from a scan that could not have seen anything |
//!
//! ## The second instrument: the dialog either asks for a tick or it does not
//!
//! Under the narrow reach the saved file really does still contain the text, so
//! the dialog raises its residual acknowledgement — a region declared **only
//! while it is being asked for**. Under the default reach there is nothing left
//! to acknowledge and it must not appear.
//!
//! Two independent instruments on one question: what is in the file, and what
//! the program said about it before writing. A build that got one right and the
//! other wrong is disclosing something it did not do.
//!
//! ## What this covers that [`super::redaction`] deliberately does not
//!
//! **The search-and-mark route.** Its neighbour marks whole pages, by choice,
//! so that it needs no keyboard, and its header names the typed route as its
//! own gap. This check has to type, because a reach setting is about *a string
//! found elsewhere* and a whole-page mark carries no string to look for.

use std::path::{Path, PathBuf};

use super::driving::{self, declared};
use super::redaction::{
    ACK_REGION, APPLY_REGION, CONFIRM_REGION, DESTINATION_NEW_FILE_REGION, EDIT_TAB, MODE,
    PANEL_EVENT, PREPARED_EVENT, REDACT, REFUSED_EVENT, REGION_PREFIX, SECRET, SURVIVOR,
    WRITE_FAILED_EVENT, WRITTEN_EVENT, census, click_command, click_region, click_tab, contains,
    launch, region,
};
use super::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::Session;
use crate::report::CheckReport;

/// The marking panel itself — the pane a wheel notch has to land in.
const PANEL_REGION: &str = "redact-panel";

/// The marking panel's search field.
const QUERY_REGION: &str = "redact-query";

/// The marking panel's *find and mark* control, drawn only once the query field
/// holds something — so its absence is evidence the typing did not land.
const SEARCH_REGION: &str = "redact-search";

/// The dialog's **residual** acknowledgement, declared only while the report
/// carries something the write will leave behind.
const RESIDUAL_ACK_REGION: &str = "redact-apply-residual-ack";

/// The preference key, spelled as the shell's preference file parses it.
///
/// A copy of a name owned by another crate, and it carries that arrangement's
/// tripwire: a rename there makes every launch here run at the shipped default,
/// both runs then agree, and this check reports the setting as inert when it was
/// merely asked for under the wrong name. The truth in that event is the
/// preference writer's own emission of this key.
const REACH_KEY: &str = "redaction_reach";

/// The narrow reach — *change only what I marked*.
const NARROW: &str = "marked-only";

/// The shipped default, which additionally scrubs carriers no viewer shows.
const WIDE: &str = "hidden-carriers";

/// See the module documentation.
pub struct TheRedactionReachSettingDecidesWhatSurvives;

impl Check for TheRedactionReachSettingDecidesWhatSurvives {
    fn name(&self) -> &'static str {
        "the_redaction_reach_setting_decides_what_survives"
    }

    fn defect(&self) -> &'static str {
        "The setting that decides how far a redaction reaches beyond the marked regions never \
         arrives at the engine, so it removes copies the operator told it to leave — or leaves \
         copies he told it to remove — while every unit test passes, because they hand the reach \
         straight to the verb and cannot see the chain in front of it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

// ---------------------------------------------------------------------------
// The fixture
// ---------------------------------------------------------------------------

/// The needle naming the copy in the document properties.
fn title_needle() -> String {
    format!("/Title ({SECRET})")
}

/// The needle naming the copy drawn on the page.
fn page_needle() -> String {
    format!("({SECRET}) Tj")
}

/// Two pages and a document-information dictionary, uncompressed.
///
/// Uncompressed for its neighbour's reason: the verdict is a byte scan, and a
/// `/FlateDecode` stream would hide the page copy from it — a false pass in the
/// direction that matters most.
fn fixture_bytes() -> Vec<u8> {
    let stream = |text: &str| {
        let c = format!("BT /F1 18 Tf 40 120 Td ({text}) Tj ET");
        format!("<< /Length {} >>\nstream\n{c}\nendstream", c.len())
    };
    let page = |contents: u32| {
        format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 200] \
             /Resources << /Font << /F1 7 0 R >> >> /Contents {contents} 0 R >>"
        )
    };
    let bodies: Vec<String> = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 >>".to_owned(),
        page(5),
        page(6),
        stream(SECRET),
        stream(SURVIVOR),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
        format!("<< {} /Author (KEEPTHIS) >>", title_needle()),
    ];

    let mut buf = b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in bodies.iter().enumerate() {
        offsets.push(buf.len());
        buf.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
    }
    let xref_at = buf.len();
    let n = bodies.len() + 1;
    buf.extend_from_slice(format!("xref\n0 {n}\n0000000000 65535 f \n").as_bytes());
    for off in &offsets {
        buf.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size {n} /Root 1 0 R /Info 8 0 R >>\nstartxref\n{xref_at}\n%%EOF\n")
            .as_bytes(),
    );
    buf
}

// ---------------------------------------------------------------------------
// One run
// ---------------------------------------------------------------------------

/// **Scroll the marking panel until `name` is on screen, and return where.**
///
/// The panel is taller than the slot a side dock gives it, so at the harness's
/// window size its search field and its *Find & mark* control start below the
/// fold. They are declared with the shell's visible-only helper — a rect appears
/// only while the control is actually drawable — so *absent* here means *not on
/// screen*, and the answer to that is a wheel notch rather than a failure.
///
/// Rewinds to the top first, then walks down one notch at a time, because the
/// controls this check needs are not in one direction from each other: the apply
/// control sits above the search row, so reaching the second scrolls the first
/// away.
///
/// An exhausted walk is an `Err`, i.e. a SKIP: the check could not reach the
/// control, which is a different claim from the control not working.
fn reveal(session: &Session, driver: &Driver, ui_rect: &str, name: &str) -> Result<LRect> {
    /// Enough to rewind any panel this shell draws to its top.
    const REWIND: i32 = 12;
    /// One notch at a time, so the control is found at the first position that
    /// shows it rather than scrolled past.
    const STEPS: usize = 16;

    let panel = region(&session.trace()?, ui_rect, PANEL_REGION, REGION_PREFIX)?;
    let at = session.frame()?.declared_center(panel);
    driver.scroll_at(at, REWIND)?;
    session.settle(6);
    for _ in 0..STEPS {
        if let Some(rect) = declared(&session.trace()?, ui_rect, name)
            && rect.is_substantial()
        {
            return Ok(rect);
        }
        driver.scroll_at(at, -1)?;
        session.settle(6);
    }
    Err(Error::new(format!(
        "`{name}` never came into view after rewinding the marking panel and scrolling {STEPS} \
         notches down it. Either the panel does not scroll — in which case everything below its \
         fold is unreachable and Mark whole page is the only way to make a mark — or the control \
         is not drawn in this state at all."
    )))
}

/// What one launch produced.
struct Outcome {
    /// The bytes the redaction wrote.
    bytes: Vec<u8>,
    /// Whether the dialog asked the operator to acknowledge something the write
    /// would leave behind.
    residual_ack_offered: bool,
}

/// Write the reach preference into the profile the binary under test resolves.
///
/// Through `sandbox::write_prefs` rather than `fs::write`, which carries the
/// startup-offer suppression as a header — see its own contract for what a
/// direct write costs. Only this one key is written: every other preference is
/// then absent, which the loader reads as *use the default*, isolating the
/// variable under test from whatever a previous check left behind.
fn write_reach(exe: &Path, reach: &str) -> Result<()> {
    let dir = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory to write userdata into"))?
        .join("userdata");
    std::fs::create_dir_all(&dir)
        .map_err(|e| Error::new(format!("could not create {}: {e}", dir.display())))?;
    crate::sandbox::write_prefs(&dir, &format!("{REACH_KEY} = {reach}\n")).map_err(|e| {
        Error::new(format!(
            "could not write the reach preference in {}: {e}",
            dir.display()
        ))
    })
}

/// Drive one reach end to end: set the preference, mark by search, apply, write.
///
/// `Err` is a SKIP — the sequence could not be completed, so nothing was
/// measured. A finished run always yields an [`Outcome`]; the verdicts are taken
/// in [`drive`], where both runs can be compared.
fn run_once(
    ctx: &CheckContext,
    report: &mut CheckReport,
    ui_rect: &str,
    fixture: &Path,
    target: &Path,
    reach: &str,
) -> Result<Outcome> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    write_reach(&exe, reach)?;
    report.note(format!("`{REACH_KEY} = {reach}` written into the profile"));

    let _ = std::fs::remove_file(target);
    let session = launch(
        ctx,
        report,
        fixture,
        target,
        &format!("redaction-reach-{reach}.trace.txt"),
    )?;
    let driver = Driver::new(session.window());

    driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(16);
    click_tab(&session, &driver, ui_rect, EDIT_TAB)?;
    click_command(&session, &driver, ui_rect, REDACT, 20)?;

    // --- mark by search ---------------------------------------------------
    //
    // The typed route, which the neighbouring check names as its own gap. The
    // field takes the caret from a click; nothing in the trace confirms a
    // keystroke landed, so the confirmation is the census two clicks later.
    //
    // Through `reveal` rather than `region`: the marking controls sit below the
    // fold of a side dock at this window size, and a rect read without scrolling
    // to them is a rect off the bottom of the screen that a click cannot reach.
    reveal(&session, &driver, ui_rect, QUERY_REGION)?;
    click_region(&session, &driver, ui_rect, QUERY_REGION, 10)?;
    driver.type_ascii(SECRET)?;
    session.settle(10);

    reveal(&session, &driver, ui_rect, SEARCH_REGION).map_err(|e| {
        Error::new(format!(
            "{e}\n\
             A disabled control still declares its rect, so this is not the control being \
             greyed. Absence at every scroll position says the typed `{SECRET}` never reached \
             the query field."
        ))
    })?;
    click_region(&session, &driver, ui_rect, SEARCH_REGION, 20)?;
    let trace = session.trace()?;
    let Some((marks, pages)) = census(&trace) else {
        return Err(Error::new(format!(
            "the panel traced no `{PANEL_EVENT}` line after the search click, so this run cannot \
             tell a mark that was made from a click that missed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if marks == 0 {
        return Err(Error::new(format!(
            "the search for `{SECRET}` marked nothing (`{PANEL_EVENT} marks=0`). Either the typed \
             query never reached the field, or this build's search cannot find text it can \
             extract. Nothing after this point would mean anything."
        )));
    }
    report.note(format!(
        "{reach}: {marks} mark(s) on {pages} page(s), by search"
    ));

    // --- the report -------------------------------------------------------
    reveal(&session, &driver, ui_rect, APPLY_REGION)?;
    click_region(&session, &driver, ui_rect, APPLY_REGION, 20)?;
    let trace = session.trace()?;
    if trace.last(PREPARED_EVENT).is_none() {
        if let Some(refused) = trace.last(REFUSED_EVENT) {
            return Err(Error::new(format!(
                "the apply was refused under `{reach}`: `{}`. A refusal writes no file, so this \
                 run measured nothing.",
                refused.raw
            )));
        }
        return Err(Error::new(format!(
            "the apply control was clicked and traced neither `{PREPARED_EVENT}` nor \
             `{REFUSED_EVENT}`, so the dialog never opened. Trace: {}.",
            session.trace_path().display()
        )));
    }
    if let Some(prepared) = trace.last(PREPARED_EVENT) {
        report.note(format!("{reach}: `{}`", prepared.raw));
    }

    // --- the acknowledgements ---------------------------------------------
    //
    // Read before either is clicked, because clicking one re-lays the window
    // out and the other moves. Every rect below is re-read from a fresh trace
    // for the same reason.
    let residual_ack_offered = declared(&session.trace()?, ui_rect, RESIDUAL_ACK_REGION).is_some();
    if residual_ack_offered {
        click_region(&session, &driver, ui_rect, RESIDUAL_ACK_REGION, 12)?;
    }
    click_region(&session, &driver, ui_rect, ACK_REGION, 12)?;

    // --- write to a new file ----------------------------------------------
    //
    // The new-file destination for its neighbour's reason: the default
    // destination arms the next save and produces no file, and this check's
    // whole verdict is a file. Replacing the original would have the harness
    // overwrite its own fixture between the two runs.
    click_region(&session, &driver, ui_rect, DESTINATION_NEW_FILE_REGION, 16)?;
    region(&session.trace()?, ui_rect, CONFIRM_REGION, REGION_PREFIX).map_err(|e| {
        Error::new(format!(
            "{e}\n\
             Every acknowledgement this dialog offered was ticked and the confirm control is \
             still not offered, so the write cannot be reached under `{reach}`. The footer's \
             reserved height is a fixed constant and the narrow reach adds a checkbox to it, so \
             the first thing to measure is whether the confirm row has been pushed below the \
             fold rather than left disabled."
        ))
    })?;
    click_region(&session, &driver, ui_rect, CONFIRM_REGION, 24)?;

    let trace = session.trace()?;
    if trace.last(WRITTEN_EVENT).is_none() {
        if let Some(failed) = trace.last(WRITE_FAILED_EVENT) {
            return Err(Error::new(format!(
                "the write was refused after the confirm under `{reach}`: `{}`.",
                failed.raw
            )));
        }
        return Err(Error::new(format!(
            "nothing was written under `{reach}`: no `{WRITTEN_EVENT}` and no \
             `{WRITE_FAILED_EVENT}`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    drop(session);

    let bytes = std::fs::read(target)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", target.display())))?;
    Ok(Outcome {
        bytes,
        residual_ack_offered,
    })
}

// ---------------------------------------------------------------------------
// Both runs, and the verdict
// ---------------------------------------------------------------------------

/// Run the sequence twice. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a
/// pass.
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check types a query and clicks through two full \
             redactions. Reported as SKIPPED rather than passed: a check that did not run has \
             learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its controls are and this check has nothing to aim at.",
            ctx.profile.name
        ))
    })?;

    // Restore the shipped reach on every path out, including the SKIPs.
    //
    // A guard rather than a line at the end, because a preference left at the
    // narrow value would make every later redaction check run under a reach it
    // never chose — and unlike a scaled window, that one is invisible in the
    // failing check's own report. Failure to restore is reported and does not
    // change the verdict.
    struct RestoreReach(Option<PathBuf>);
    impl Drop for RestoreReach {
        fn drop(&mut self) {
            if let Some(exe) = &self.0
                && let Err(e) = write_reach(exe, WIDE)
            {
                eprintln!(
                    "ui-verify: WARNING — could not restore {REACH_KEY} to {WIDE} ({e}). Every later redaction check will run under the reach this one left behind. Fix by deleting userdata/preferences.txt beside the binary."
                );
            }
        }
    }
    let _restore = RestoreReach(ctx.resolve_exe());

    let fixture: PathBuf = ctx.out("redaction-reach-fixture.pdf");
    let source = fixture_bytes();
    std::fs::write(&fixture, &source)
        .map_err(|e| Error::new(format!("cannot write {}: {e}", fixture.display())))?;

    // The falsifying phase. All three needles are written into the fixture from
    // the constants this check scans for; if the scan cannot see them HERE,
    // every absence it reports below is worthless.
    for needle in [title_needle(), page_needle(), SURVIVOR.to_owned()] {
        if !contains(&source, needle.as_bytes()) {
            return Ok(Some(format!(
                "★ THE INSTRUMENT CANNOT SEE `{needle}` in the fixture it just wrote at {}, so \
                 this check's assertions could not fail and would pass against a build that did \
                 nothing. Harness defect, reported as a failure so it cannot be mistaken for a \
                 pass.",
                fixture.display()
            )));
        }
    }
    report.note(format!(
        "fixture {} — {} bytes, holding `{SECRET}` on page 1 AND in `/Info /Title`, and \
         `{SURVIVOR}` on page 2",
        fixture.display(),
        source.len()
    ));

    let narrow = run_once(
        ctx,
        report,
        ui_rect,
        &fixture,
        &ctx.out("redaction-reach-narrow.pdf"),
        NARROW,
    )?;
    let wide = run_once(
        ctx,
        report,
        ui_rect,
        &fixture,
        &ctx.out("redaction-reach-wide.pdf"),
        WIDE,
    )?;

    // --- the controls -----------------------------------------------------
    for (label, out) in [(NARROW, &narrow), (WIDE, &wide)] {
        if !contains(&out.bytes, SURVIVOR.as_bytes()) {
            return Ok(Some(format!(
                "★ the control string `{SURVIVOR}` is missing from the `{label}` output. It is \
                 drawn on a page nobody marked, so either the removal took a page it was never \
                 given, or the output's streams are compressed — and if they are, every absence \
                 asserted below is an absence from a scan that could not have seen anything."
            )));
        }
        if contains(&out.bytes, page_needle().as_bytes()) {
            return Ok(Some(format!(
                "★ THE MARKED COPY SURVIVED under `{label}`: `{}` is still drawn on page 1 of the \
                 saved file. No reach makes that acceptable — the reach decides what happens \
                 BEYOND the marks, and this is under one.",
                page_needle()
            )));
        }
    }

    // --- the verdict ------------------------------------------------------
    let narrow_kept = contains(&narrow.bytes, title_needle().as_bytes());
    let wide_kept = contains(&wide.bytes, title_needle().as_bytes());
    if !narrow_kept {
        return Ok(Some(format!(
            "★ THE NARROW REACH REMOVED THE COPY IT WAS TOLD TO LEAVE. With `{REACH_KEY} = \
             {NARROW}` in the preferences, `{}` is gone from the saved file. That is the \
             operator's original complaint with a control painted on top of it: the setting is \
             offered, remembered, and does not reach the engine.",
            title_needle()
        )));
    }
    if wide_kept {
        return Ok(Some(format!(
            "★ THE DEFAULT REACH LEFT A HIDDEN COPY. With `{REACH_KEY} = {WIDE}`, `{}` is still \
             in the saved file — so the shipped setting reaches no further than the narrowest \
             one, and every operator who never opens the settings window is saving redactions \
             that keep the text in the document properties.",
            title_needle()
        )));
    }
    report.note(format!(
        "the copy in `/Info /Title` survives `{NARROW}` and does not survive `{WIDE}`, measured \
         on the saved bytes of two files written by two processes"
    ));

    // --- the disclosure ---------------------------------------------------
    if !narrow.residual_ack_offered {
        return Ok(Some(format!(
            "★ THE NARROW REACH WROTE A FILE THAT STILL CONTAINS THE TEXT, WITH NOTHING TO TICK. \
             `{RESIDUAL_ACK_REGION}` was never declared, so the operator confirmed a redaction \
             and was not asked to acknowledge that a copy survives it. The bytes say one does."
        )));
    }
    if wide.residual_ack_offered {
        return Ok(Some(format!(
            "★ THE DEFAULT REACH ASKED THE OPERATOR TO ACKNOWLEDGE A SURVIVOR THAT IS NOT THERE. \
             `{RESIDUAL_ACK_REGION}` was declared under `{WIDE}`, and the saved bytes contain \
             neither copy. An acknowledgement demanded for nothing is how an operator learns to \
             tick without reading."
        )));
    }
    report.note(
        "the residual acknowledgement is raised under the narrow reach and not under the wide \
         one, so what the dialog asks matches what the file holds"
            .to_owned(),
    );

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The fixture contains everything this check scans for.**
    ///
    /// The same falsification the run performs, asserted at build time so a
    /// fixture that stopped carrying one of the three needles fails in the
    /// suite rather than on the operator's desk.
    #[test]
    fn the_generated_fixture_carries_all_three_needles() {
        let bytes = fixture_bytes();
        for needle in [title_needle(), page_needle(), SURVIVOR.to_owned()] {
            assert!(
                contains(&bytes, needle.as_bytes()),
                "the fixture does not contain `{needle}`, so every assertion about its absence \
                 would pass vacuously"
            );
        }
    }

    /// **The two needles are distinguishable.**
    ///
    /// The whole check rests on telling the page copy from the properties copy.
    /// If one were a substring of the other, a single removal would satisfy both
    /// assertions and the verdict would mean nothing.
    #[test]
    fn the_page_copy_and_the_properties_copy_are_different_strings() {
        assert!(!title_needle().contains(&page_needle()));
        assert!(!page_needle().contains(&title_needle()));
    }

    /// **The two reaches under test are different settings.**
    ///
    /// A copy-paste that made both runs write the same token would produce two
    /// identical outputs, and the verdict would then report the setting as inert
    /// when it had never been varied.
    #[test]
    fn the_two_reaches_are_not_the_same_token() {
        assert_ne!(NARROW, WIDE);
    }

    /// The region names this check aims at share the prefix it lists when it
    /// cannot find one.
    #[test]
    fn the_selectors_match_the_prefix_the_skip_reasons_quote() {
        for name in [QUERY_REGION, SEARCH_REGION, RESIDUAL_ACK_REGION] {
            assert!(
                name.starts_with(REGION_PREFIX),
                "`{name}` is not under the prefix this check quotes in its SKIP reasons, so a \
                 failure to find it would print an unhelpfully unrelated list"
            );
        }
    }
}
