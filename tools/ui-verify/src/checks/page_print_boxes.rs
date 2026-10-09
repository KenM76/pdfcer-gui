//! `resizing_a_sheet_says_which_print_boxes_reach_past_it` — shrinking a sheet
//! whose bleed and trim boxes no longer fit on it says so, box by box, and
//! says nothing of an art box that still fits.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/page_print_boxes.md`.

use std::path::PathBuf;

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::page_size::{APPLY, OFFSCREEN, click_dialog_region, pick_a6_portrait};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// `page-size-applied …` — the box-only resize ran, with its counts.
const APPLIED: &str = "page-size-applied"; // ui-text-exempt: a trace event name, never displayed
/// `page-size-changed … disclosures=…` — the sentences the resize raised.
const CHANGED: &str = "page-size-changed"; // ui-text-exempt: a trace event name, never displayed
/// The counts owed for the fixture: bleed and trim past A6, art inside it.
const OWED: [(&str, &str); 3] = [
    ("bleed_outside", "1"),
    ("trim_outside", "1"),
    ("art_outside", "0"),
];

/// See the module documentation.
pub struct ResizingASheetSaysWhichPrintBoxesReachPastIt;

impl Check for ResizingASheetSaysWhichPrintBoxesReachPastIt {
    fn name(&self) -> &'static str {
        "resizing_a_sheet_says_which_print_boxes_reach_past_it"
    }

    fn defect(&self) -> &'static str {
        "a sheet shrunk under its bleed or trim box is printed cut to the new paper with nothing \
         said, or an art box that still fits is reported as cut"
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

/// One 600 × 800 sheet: bleed box the whole sheet, trim box inset 10 pt, art
/// box a 100 pt square near the origin. A6 (297.6 × 419.5) holds the art box
/// and neither of the others.
fn fixture(ctx: &CheckContext) -> Result<PathBuf> {
    let content = b"0 G 1 w 20 20 100 100 re S".to_vec();
    let mut stream = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
    stream.extend_from_slice(&content);
    stream.extend_from_slice(b"\nendstream");
    let bytes = crate::fixture::pdf_of(&[
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 600 800] /BleedBox [0 0 600 800] \
          /TrimBox [10 10 590 790] /ArtBox [20 20 120 120] /Resources << >> /Contents 4 0 R >>"
            .to_vec(),
        stream,
    ]);
    let path = ctx.out("page-print-boxes.pdf");
    std::fs::write(&path, bytes)
        .map_err(|e| Error::new(format!("cannot write {}: {e}", path.display())))?;
    Ok(path)
}

fn launch(ctx: &CheckContext, pdf: PathBuf) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("page-print-boxes.trace.txt"));
    spec.pdf = Some(pdf);
    for (key, value) in [
        ctx.profile.diag_env,
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", "mode.edit"),
    ] {
        spec.env.push((key.to_owned(), value.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("page-print-boxes.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");
    let (session, pointer) = launch(ctx, fixture(ctx)?)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(30);
    let failure = steps(&session, &pointer, ui_rect, report);
    let parked = pointer.gone(&session);
    match failure? {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// Resize to A6 portrait; read the counts and the sentences.
fn steps(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    if session.trace()?.events(APPLIED).next().is_some() {
        return Ok(Some(format!(
            "`{APPLIED}` is in the trace before anything was pressed."
        )));
    }
    let mut scratch = CheckReport::new("", "");
    crate::checks::page_size::open_and_census(
        session,
        pointer,
        ui_rect,
        &mut scratch,
        "the fixture",
    )?;
    pick_a6_portrait(session, pointer, ui_rect)?;
    click_dialog_region(session, pointer, ui_rect, APPLY)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(applied) = trace.events(APPLIED).last() else {
        return Ok(Some(format!(
            "A6 was applied and no `{APPLIED}` followed: the resize did not run."
        )));
    };
    report.note(format!("applied: `{}`", applied.raw));
    for (key, want) in OWED {
        if applied.get(key) != Some(want) {
            return Ok(Some(format!(
                "`{APPLIED}` says {key}={:?}; the fixture owes {want}.",
                applied.get(key)
            )));
        }
    }
    let said = trace
        .events(CHANGED)
        .last()
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    report.note(format!("said: `{said}`"));
    for (phrase, owed) in [("bleed box", true), ("trim box", true), ("art box", false)] {
        if said.contains(phrase) != owed {
            return Ok(Some(format!(
                "the resize's sentences {} \"{phrase}\": `{said}`.",
                if owed { "never mention" } else { "mention" }
            )));
        }
    }
    Ok(None)
}
