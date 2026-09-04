//! `export_dxf_writes_the_pages_geometry` — the DXF reaches disk, and the file
//! agrees with what the shell said it wrote.
//!
//! # The gap this closes
//!
//! `file.export_dxf` was the first entry in `shell::commands::reach`'s
//! `SCAFFOLDED` list, with the recorded reason *"No recorded reason anywhere.
//! Scaffolded by omission, not by decision."* `pdfcer-core`'s `export::dxf` had
//! shipped the whole time and the **old shell has the feature**, so this was a
//! regression against `FEATURES.md`'s `gui` column rather than a gap.
//!
//! # Why this needs driving
//!
//! Because the interesting half is not the writer — that is `pdfcer-core`'s, and
//! it is tested there. It is the six links between a ribbon press and a file:
//!
//! 1. the arm builds a window and computes a scale suggestion from the page's
//!    **own** dimension groups;
//! 2. the window raises an `Action` carrying a `DxfOptions`;
//! 3. the apply arm fetches the page's decomposition **from the shared cache**,
//!    which may not be filled yet;
//! 4. it calls the writer;
//! 5. it opens a save dialog — a modal OS window, which is why this is an
//!    action at all;
//! 6. it writes the bytes and discloses what was left behind.
//!
//! Link 3 is the one that cannot be unit-tested: the decomposition is produced
//! by the canvas's own provider, keyed on `(page, epoch)`, and whether it is
//! populated at the moment an export runs is a question about a **running
//! frame**.
//!
//! # ★ The assertion that makes this more than a smoke test
//!
//! **The file on disk is counted, and the count is compared against the
//! trace.** The trace says what the shell believed it wrote; the file says what
//! was written. A build whose apply arm reported an outcome from one page and
//! wrote bytes from another — or wrote nothing and reported a success — passes
//! every other assertion here and fails this one.
//!
//! It is the same shape as `pages_drag`'s caret-versus-gap cross-check, and for
//! the same reason: two values that are *supposed* to describe one thing are
//! exactly the pair a refactor separates.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, declared_or_in_overflow, frame_of, list,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode this runs in.
///
/// ★ **Read**, deliberately, and it is an assertion rather than a convenience.
/// An export reads the document and writes elsewhere, so there is no mode in
/// which it should be refused — and a reading stance exporting a drawing is
/// exactly what a reading stance is for. If a capability gate ever creeps onto
/// this command, this is where it shows up.
const MODE: &str = "read";
/// The window's own region.
const WINDOW: &str = "dialog:export-dxf";
/// The Export button.
const EXPORT: &str = "export-dxf.export";
/// The trace the window emits when it opens, carrying the scale suggestion.
const OPENED: &str = "export-dxf-open";
/// The trace the apply arm emits on a successful write.
const WROTE: &str = "export-dxf";
/// The environment seam that answers the save dialog.
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name

/// See the module documentation.
pub struct ExportDxfWritesThePagesGeometry;

impl Check for ExportDxfWritesThePagesGeometry {
    fn name(&self) -> &'static str {
        "export_dxf_writes_the_pages_geometry"
    }

    fn defect(&self) -> &'static str {
        "File > Export to DXF is drawn and does nothing — or it opens a window whose Export \
         button writes no file, or writes one whose contents do not match what the shell \
         reported, so an operator hands a CAD file to somebody with counts that describe a \
         different page"
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

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no fixture document. Pass --pdf."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, a ribbon tab, \
             a ribbon control and a button. Reported as SKIPPED rather than passed: a check \
             that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    // ★ Removed before the run, not just named. A file left by an earlier run
    // would let a build that writes NOTHING pass every assertion below — which
    // is `a_driven_check_that_does_not_establish_its_preconditions_measures_the_previous_run`
    // in the Rust RAG, and it is the failure this line exists to prevent.
    let target = ctx.out("export_dxf.dxf");
    let _ = std::fs::remove_file(&target);
    if target.exists() {
        return Err(Error::new(format!(
            "cannot clear {} before the run, so a file written by an earlier run could be \
             mistaken for this one's.",
            target.display()
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("export_dxf.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((SAVE_PATH_ENV.to_owned(), target.display().to_string()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;

    // --- 1: the File tab ---------------------------------------------------
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.file").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.file` region in {MODE}. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);

    // --- 2: open the window ------------------------------------------------
    let Some(item) =
        declared_or_in_overflow(&session, &driver, ui_rect, "ribbon.item.file.export_dxf")?
    else {
        return Ok(Some(format!(
            "the File tab declares no `ribbon.item.file.export_dxf`, on the band or in the \
             overflow. Items declared: {}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "ribbon.item.file."
            ))
        )));
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(20);

    let trace = session.trace()?;
    if declared(&trace, ui_rect, WINDOW).is_none() {
        let unimplemented = trace
            .events("command-unimplemented")
            .any(|l| l.get("id") == Some("file.export_dxf"));
        return Ok(Some(if unimplemented {
            "`file.export_dxf` was clicked and traced `command-unimplemented` — it is still \
             scaffolded. Drawn, on the ribbon, and with no dispatch arm behind it."
                .to_owned()
        } else {
            format!(
                "`file.export_dxf` was clicked and no `{WINDOW}` region appeared. The arm ran \
                 and built no window — or declined it. In {MODE} it should not decline: an \
                 export reads the document and writes elsewhere."
            )
        }));
    }

    // --- 3: ★ the scale was INFERRED, not assumed --------------------------
    //
    // The window traces its suggestion on open, and the value is the whole
    // point of the feature: `Uncalibrated` on a drawing with no calibrated
    // groups is CORRECT and is what this fixture will produce, while a build
    // that quietly defaulted without querying would trace nothing at all.
    let Some(opened) = trace.last(OPENED) else {
        return Ok(Some(format!(
            "the window drew and traced no `{OPENED}` line, so it never asked the document \
             what scale the page is at. That query is the feature — without it this is a \
             paper-scale converter with extra steps."
        )));
    };
    report.note(format!("scale inference: `{}`", opened.raw));

    // --- 4: export ---------------------------------------------------------
    let Some(button) = declared(&trace, ui_rect, EXPORT) else {
        return Ok(Some(format!(
            "the window declares no `{EXPORT}` region, so there is nothing to press."
        )));
    };
    driver.click_at(frame_of(&session, &trace, ui_rect, EXPORT)?.declared_center(button))?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(wrote) = trace.last(WROTE) else {
        let declined = trace.last("export-dxf-declined");
        let cancelled = trace.last("export-dxf-cancelled");
        let failed = trace.last("export-dxf-failed");
        return Ok(Some(format!(
            "Export was pressed and no `{WROTE}` line followed. declined={} cancelled={} \
             failed={} — a DECLINE means the page's decomposition was not in the cache when \
             the export ran, which is the one link in this chain no unit test can reach.",
            declined.map_or("none".to_owned(), |l| l.raw.clone()),
            cancelled.map_or("none".to_owned(), |l| l.raw.clone()),
            failed.map_or("none".to_owned(), |l| l.raw.clone()),
        )));
    };
    report.note(format!("wrote: `{}`", wrote.raw));

    // --- 5: ★ the file is on disk, and it is a DXF -------------------------
    if !target.exists() {
        return Ok(Some(format!(
            "the shell traced a successful export and {} does not exist. The disclosure and \
             the disk disagree, which is the worst of the three possible failures here: the \
             operator has been told a file was written.",
            target.display()
        )));
    }
    let written = std::fs::read_to_string(&target).map_err(|e| {
        Error::new(format!(
            "the export wrote {} and it cannot be read back: {e}",
            target.display()
        ))
    })?;
    if !written.contains("SECTION") || !written.contains("ENTITIES") {
        return Ok(Some(format!(
            "{} was written and is not a DXF — no SECTION or ENTITIES marker in {} bytes. \
             Something reached disk; it is not what the operator asked for.",
            target.display(),
            written.len()
        )));
    }
    report.note(format!(
        "{} bytes on disk, with a DXF header",
        written.len()
    ));

    // --- 6: ★★ the file agrees with what the shell REPORTED ----------------
    //
    // The cross-check, and the reason this check is worth more than a smoke
    // test. `polylines` counts LINE + LWPOLYLINE entities, so those two markers
    // together must appear at least that many times in the file. A build that
    // reported an outcome from one page and wrote another's bytes passes every
    // assertion above and fails here.
    //
    // `>=` rather than `==`: the strings also appear in the DXF's TABLES
    // section as layer and linetype records, which are not entities. The
    // asymmetry is deliberate — an undercount is the failure worth catching,
    // and a bound that is loose in the safe direction beats one that fails on a
    // correct file.
    let Some(reported) = wrote.get("polylines").and_then(|v| v.parse::<usize>().ok()) else {
        return Err(Error::new(format!(
            "the `{WROTE}` line carries no readable `polylines=` count, so the file cannot be \
             checked against it: `{}`",
            wrote.raw
        )));
    };
    let in_file = written.matches("LWPOLYLINE").count() + written.matches("\nLINE\n").count();
    if reported > 0 && in_file < reported {
        return Ok(Some(format!(
            "★ the shell reported {reported} line entities and the file holds {in_file}. The \
             disclosure describes a page the bytes do not — which an operator has no way to \
             notice, because both numbers look reasonable and only one of them is in the file \
             they hand to somebody else."
        )));
    }
    if reported == 0 {
        report.note(
            "the page produced no line entities, which this fixture may legitimately do — the \
             count cross-check is vacuous on this run and says so rather than passing quietly",
        );
    } else {
        report.note(format!(
            "{reported} line entities reported, {in_file} found in the file"
        ));
    }
    Ok(None)
}
