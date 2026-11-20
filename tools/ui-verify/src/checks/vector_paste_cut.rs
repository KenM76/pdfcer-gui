//! `an_svg_or_metafile_pastes_as_a_drawing_and_cuts_as_vector` — another
//! program's SVG, then its metafile, paste onto the page as drawings, and Cut
//! as vector removes the pasted drawing after placing it with the PDF and
//! pdfcer's own object clip.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/vector_paste_cut.md`.

use crate::checks::cross_window_paste::{CAPTURE_ENV, click, to_edit_tab};
use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::os_image_paste::{at, count, rect, show};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const DOC: &str = "D:/Dev/pdfcer/fixtures/synthetic/pageops/four-pages.pdf";
const SVG_FILE: &str = "1-image_svg_xml.bin"; // ui-text-exempt: a capture file name
const EMF_FILE: &str = "2-CF_ENHMETAFILE.bin"; // ui-text-exempt: a capture file name
const PASTED: &str = "clip-pasted"; // ui-text-exempt: a trace event name
const CUT: &str = "ribbon.item.edit.cut_as_vector"; // ui-text-exempt: a trace region name
const CUT_OUT: &str = "clipboard-cut-out"; // ui-text-exempt: a trace event name
const PLACED: &str = "clipboard-copy-out"; // ui-text-exempt: a trace event name
const DELETED: &str = "delete-objects"; // ui-text-exempt: a trace event name
const PDF_SUFFIX: &str = "-application_pdf.bin"; // ui-text-exempt: a capture file name
const FIRST: (f64, f64) = (200.0, 500.0);
const SECOND: (f64, f64) = (400.0, 250.0);
/// How far a pasted drawing's centre may sit from the pointer, in points.
const TOLERANCE_PT: f64 = 2.0;

/// See the module documentation.
pub struct AVectorPastesAndCuts;

impl Check for AVectorPastesAndCuts {
    fn name(&self) -> &'static str {
        "an_svg_or_metafile_pastes_as_a_drawing_and_cuts_as_vector"
    }

    fn defect(&self) -> &'static str {
        "an SVG or metafile another program copied pastes as a picture or not at all, or Cut as \
         vector leaves the drawing on the page or places no PDF and object clip"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let dir = ctx.out("vector-paste-capture");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)
        .map_err(|e| Error::new(format!("cannot make {}: {e}", dir.display())))?;
    for (fixture, file) in [("vector-art.svg", SVG_FILE), ("vector-art.emf", EMF_FILE)] {
        let source = driving::repo_fixture(fixture, "It is checked in.")?;
        std::fs::copy(&source, dir.join(file))
            .map_err(|e| Error::new(format!("copying {fixture}: {e}")))?;
    }
    let (session, pointer) = launch(ctx, report, &dir)?;
    let judged = drive(ctx, report, &session, &pointer, &dir);
    let parked = pointer.gone(&session);
    match judged {
        Ok(None) => parked.map(|_| None),
        other => other,
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    dir: &std::path::Path,
) -> Result<Option<String>> {
    if let Some(failure) = paste_drawing(ctx, report, session, pointer, FIRST, "svg")? {
        return Ok(Some(failure));
    }
    // With the SVG gone the metafile is the best vector format left.
    std::fs::remove_file(dir.join(SVG_FILE))
        .map_err(|e| Error::new(format!("cannot remove the SVG capture: {e}")))?;
    if let Some(failure) = paste_drawing(ctx, report, session, pointer, SECOND, "emf")? {
        return Ok(Some(failure));
    }
    cut(ctx, report, session, pointer, dir)
}

/// Hover `p`, paste, and require a content drawing of `format` added there.
fn paste_drawing(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    p: (f64, f64),
    format: &str,
) -> Result<Option<String>> {
    let added = format!("add-{format}");
    let (before, added_before) = (count(session, PASTED)?, count(session, &added)?);
    pointer.hover(session, at(ctx, session, p)?)?;
    pointer.paste(session, None, "x")?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(line) = trace.events(PASTED).nth(before) else {
        return Ok(Some(format!(
            "★ pasting the {format} capture traced no `{PASTED}` line."
        )));
    };
    report.note(format!("`{}`", line.raw));
    if line.get("kind") != Some("vector")
        || line.get("format") != Some(format)
        || line.get("as") != Some("content")
    {
        return Ok(Some(format!(
            "★★ the {format} capture should paste as `kind=vector format={format} as=content`; \
             it traced `{}`.",
            line.raw
        )));
    }
    if let Some(r) = rect(line) {
        let (cx, cy) = ((r[0] + r[2]) / 2.0, (r[1] + r[3]) / 2.0);
        if (cx - p.0).abs() > TOLERANCE_PT || (cy - p.1).abs() > TOLERANCE_PT {
            return Ok(Some(format!(
                "the {format} drawing landed at {}, centred away from the pointer {p:?}.",
                show(r)
            )));
        }
    }
    Ok((count(session, &added)? == added_before).then(|| {
        format!("the {format} paste traced and the engine added nothing (no `{added}` line).")
    }))
}

/// Cut as vector on the pasted metafile, which the paste left selected.
fn cut(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    dir: &std::path::Path,
) -> Result<Option<String>> {
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");
    let deleted = count(session, DELETED)?;
    to_edit_tab(session, pointer, ui_rect)?;
    click(session, pointer, ui_rect, CUT)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(cut) = trace.last(CUT_OUT) else {
        return Ok(Some(format!(
            "★ `{CUT}` was clicked and no `{CUT_OUT}` line followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("`{}`", cut.raw));
    let placed = trace.last(PLACED).map(|l| l.raw.clone()).unwrap_or_default();
    report.note(format!("`{placed}`"));
    let formats = trace
        .last(PLACED)
        .and_then(|l| l.get("formats"))
        .unwrap_or("");
    let has = |name: &str| formats.split(',').any(|f| f == name);
    if !placed.contains("selection=true") || !has("application/pdf") || !has("pdfcer-gui ObjectClip")
    {
        return Ok(Some(format!(
            "★★ the cut should place the selection with the PDF and pdfcer's object clip; it \
             traced `{placed}`."
        )));
    }
    if count(session, DELETED)? == deleted {
        return Ok(Some(format!(
            "★★★ the cut placed the clip and left the drawing on the page: no `{DELETED}` line."
        )));
    }
    Ok(pdf_file(dir).is_none_or(|b| !b.starts_with(b"%PDF-")).then(|| {
        format!("the capture folder holds no `*{PDF_SUFFIX}` starting `%PDF-`.")
    }))
}

/// The PDF entry the last capture wrote.
fn pdf_file(dir: &std::path::Path) -> Option<Vec<u8>> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(std::result::Result::ok)
        .find(|e| e.file_name().to_string_lossy().ends_with(PDF_SUFFIX))
        .and_then(|e| std::fs::read(e.path()).ok())
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    dir: &std::path::Path,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe, or build the default."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let doc = ctx.out("vector-paste-source.pdf");
    std::fs::copy(DOC, &doc).map_err(|e| Error::new(format!("copying {DOC}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("vector-paste.trace.txt"));
    spec.pdf = Some(doc);
    let capture = dir.display().to_string();
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", "mode.edit"),
        (CAPTURE_ENV, capture.as_str()),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("vector-paste.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    Ok((session, pointer))
}
