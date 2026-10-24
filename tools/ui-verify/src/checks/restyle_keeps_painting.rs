//! `a_large_restyle_keeps_painting` — a size typed for a clicked text object
//! of many runs restyles every run while the window keeps painting, and one
//! Ctrl+Z takes the whole change back. With `--pdf` and `--doc-point` it
//! drives the operator's drawing instead of the built-in page.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/restyle_keeps_painting.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared_in};
use crate::checks::scaled_text_size::wait;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::{Trace, TraceLine};

/// Startup commands: Edit mode with Properties mounted.
const INVOKE: &str = "mode.edit,file.properties";
/// Off-desktop viewport, so the check runs while the operator uses the machine.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// The size field.
const SIZE_REGION: &str = "properties.text.size";
/// The panel's read of the selection's first run; `size=` is its drawn size.
const SYNCED: &str = "text-style-synced";
/// The restyle's verdicts.
const VERDICTS: [&str; 2] = ["text-style-applied", "text-style-declined"]; // ui-text-exempt: trace event names, never displayed
/// The restyle's summary: `ops= applied= slices= engine_ms= wall_ms= hurried=`.
const BATCH: &str = "text-style-batch"; // ui-text-exempt: a trace event name, never displayed
/// A frame whose own work took 50 ms or more.
const LONG: &str = "frame-long"; // ui-text-exempt: a trace event name, never displayed
/// The longest frame the window may take while the restyle runs. One engine
/// call (about 105 ms on SW41177) plus the frame's own work fits under it
/// several times; a restyle done in one frame is tens of seconds.
const HOLD_MS: usize = 800;
/// The engine time past which the restyle must have been spread over frames.
const SPREAD_MS: usize = 400;
/// The fixture's line count: one text object, one show operator a line.
const LINES: usize = 120;
/// The fixture's drawn size, and the size typed over it.
const DRAWN_PT: f64 = 4.0;
const WANTED_PT: f64 = 6.0;

/// See the module documentation.
pub struct ALargeRestyleKeepsPainting;

impl Check for ALargeRestyleKeepsPainting {
    fn name(&self) -> &'static str {
        "a_large_restyle_keeps_painting"
    }

    fn defect(&self) -> &'static str {
        "a size typed for a clicked text object restyles its runs one by one inside one frame, \
         so on a dense drawing the window freezes for tens of seconds, and the change takes one \
         Ctrl+Z per run to take back"
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

/// A one-page Letter PDF whose one text object holds [`LINES`] lines.
fn fixture() -> Vec<u8> {
    let mut content = format!("BT /F1 {DRAWN_PT} Tf 72 740 Td");
    for line in 0..LINES {
        content.push_str(&format!(" (Line {line:03} of one text object) Tj 0 -5 Td"));
    }
    content.push_str(" ET");
    crate::fixture::pdf_of(&[
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] \
          /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>"
            .to_vec(),
        format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        )
        .into_bytes(),
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
            .to_vec(),
    ])
}

/// The file, a point on the object, and its page size.
struct Subject {
    pdf: std::path::PathBuf,
    at: DocPoint,
    page: PageGeometry,
}

/// The operator's drawing when `--pdf` is passed (copied first, since the
/// check edits it), else the built-in page.
fn subject(ctx: &CheckContext) -> Result<Subject> {
    let pdf = ctx.out("restyle-keeps-painting.pdf");
    let Some(source) = ctx.pdf.clone() else {
        std::fs::write(&pdf, fixture())
            .map_err(|e| Error::new(format!("cannot write {}: {e}", pdf.display())))?;
        return Ok(Subject {
            pdf,
            at: DocPoint::new(0, 80.0, 741.0),
            page: PageGeometry {
                width_pt: 612.0,
                height_pt: 792.0,
            },
        });
    };
    let at = ctx.target.ok_or_else(|| {
        Error::new("--pdf without --doc-point. Pass PAGE,X,Y in PDF user space on a text object.")
    })?;
    let page = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&source).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. Pass --page-size WxH.",
                source.display()
            ))
        })?,
    };
    std::fs::copy(&source, &pdf)
        .map_err(|e| Error::new(format!("copying {}: {e}", source.display())))?;
    Ok(Subject { pdf, at, page })
}

/// Launch the binary on the subject's file.
fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    subject: &Subject,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("restyle-keeps-painting.trace.txt"));
    spec.pdf = Some(subject.pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer =
        ScriptedPointer::attach(&mut spec, ctx.out("restyle-keeps-painting.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(40);
    Ok((session, pointer))
}

/// The last resolved panel read after `mark`, as `(size, run)`.
fn synced(session: &Session, mark: usize) -> Result<Option<(f64, usize)>> {
    Ok(session
        .trace()?
        .events(SYNCED)
        .filter(|l| l.lineno > mark && l.get("resolved") == Some("1"))
        .last()
        .and_then(|l| Some((l.get("size")?.parse().ok()?, l.get_usize("run")?))))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let subject = subject(ctx)?;
    let (session, pointer) = launch(ctx, report, &subject)?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    click_mode_segment(&session, &pointer, ui_rect, "edit")?;
    session.settle(20);
    let mapping = CanvasMapping::from_trace(
        &session.trace()?,
        &ctx.profile.vocab,
        subject.page,
        subject.at.page,
    )?;
    let mark = session.trace()?.mark();
    let at = mapping.doc_to_window(subject.at)?;
    pointer.click(&session, at)?;
    let Some((before, run)) = wait(20_000, || {
        session.settle(4);
        synced(&session, mark)
    })?
    else {
        return Err(Error::new(format!(
            "a click on the object produced no resolved `{SYNCED}` line, so Properties never \
             read it. SKIPPED: the aim or the selection failed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let wanted = if (before - WANTED_PT).abs() < 0.5 {
        DRAWN_PT
    } else {
        WANTED_PT
    };
    report.note(format!("the object's first run {run} reads {before:.2} pt"));
    let mark = session.trace()?.mark();
    type_size(ctx, &session, &pointer, wanted)?;
    let Some(verdict) = wait(600_000, || {
        session.settle(10);
        let trace = session.trace()?;
        Ok(VERDICTS
            .iter()
            .filter_map(|v| trace.last_after(v, mark))
            .max_by_key(|l| l.lineno)
            .cloned())
    })?
    else {
        return Ok(Some(format!(
            "{wanted:.0} was typed and Enter pressed, and no verdict followed in ten minutes. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let after = frame_after(&session, &pointer, at)?;
    if let Some(failure) = held(&session.trace()?, mark, &after, &verdict, report) {
        return Ok(Some(failure));
    }
    if let Some(failure) = batch(&session, report, mark, &verdict)? {
        return Ok(Some(failure));
    }
    undo_whole(&session, &pointer, report, before, wanted)
}

/// Type `wanted` into the size field and press Enter.
fn type_size(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    wanted: f64,
) -> Result<()> {
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or_default();
    let Some((field, viewport)) = declared_in(&session.trace()?, ui_rect, SIZE_REGION) else {
        return Err(Error::new(format!(
            "no `{SIZE_REGION}` region on screen, so there is nothing to type into. SKIPPED. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let vp = viewport.as_deref();
    pointer.click_in(session, vp, WindowPoint::centre_of(field))?;
    session.settle(12);
    pointer.key(session, vp, "A", Some("ctrl"))?;
    pointer.type_text(session, vp, &format!("{wanted:.0}"))?;
    session.settle(8);
    // Every step before this one was acknowledged, so an Enter that is not
    // means the frame that took it is still running: the freeze this check
    // exists for. The verdict wait then sees that frame end, and `held` fails
    // on its length.
    match pointer.key(session, vp, "Enter", None) {
        Err(e) if e.to_string().contains("was not acknowledged") => Ok(()),
        other => other.map(|_| ()),
    }
}

/// The acknowledgement of a hover sent after the verdict: a frame that began
/// after the verdict's frame ended, so that frame's own `frame-long` line,
/// written as it ends, is in the trace before it. Retried while the window is
/// still frozen, for up to ten minutes.
fn frame_after(session: &Session, pointer: &ScriptedPointer, at: WindowPoint) -> Result<TraceLine> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(600);
    loop {
        match pointer.hover(session, at) {
            Err(e)
                if e.to_string().contains("was not acknowledged")
                    && std::time::Instant::now() < deadline => {}
            other => return other,
        }
    }
}

/// A failure when a frame after `mark`, up to the frame that acknowledged
/// `after`, took over [`HOLD_MS`]. That span ends after the verdict on
/// purpose: the frame that ran the restyle traces its length when it ends,
/// after the verdict it emitted.
fn held(
    trace: &Trace,
    mark: usize,
    after: &TraceLine,
    verdict: &TraceLine,
    report: &mut CheckReport,
) -> Option<String> {
    let Some(worst) = trace
        .lines
        .iter()
        .filter(|l| l.lineno > mark && l.lineno < after.lineno && l.event == LONG)
        .max_by_key(|l| l.get_usize("ms").unwrap_or(0))
    else {
        report.note("★ no frame took 50 ms or more while the restyle ran");
        return None;
    };
    let ms = worst.get_usize("ms").unwrap_or(0);
    if ms > HOLD_MS {
        return Some(format!(
            "★ the window stopped painting for {ms} ms while the restyle ran (limit {HOLD_MS} ms): \
             `{}`. The restyle must run in slices across frames (`app::actions::restylejob`). \
             Verdict: `{}`.",
            worst.raw, verdict.raw
        ));
    }
    report.note(format!(
        "★ the longest frame while the restyle ran took {ms} ms (limit {HOLD_MS} ms)"
    ));
    None
}

/// The restyle's summary: every call applied, none run in one go, and spread
/// over frames when the engine time calls for it.
fn batch(
    session: &Session,
    report: &mut CheckReport,
    mark: usize,
    verdict: &TraceLine,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(line) = trace.last_after(BATCH, mark) else {
        return Ok(Some(format!(
            "★★ the restyle's verdict `{}` has no `{BATCH}` summary before it, so it did not run \
             as one sliced job. Trace: {}.",
            verdict.raw,
            session.trace_path().display()
        )));
    };
    let field = |k: &str| line.get_usize(k).unwrap_or(0);
    let (ops, applied, slices, engine) = (
        field("ops"),
        field("applied"),
        field("slices"),
        field("engine_ms"),
    );
    if ops == 0 || applied != ops || !verdict.event.ends_with("applied") {
        return Ok(Some(format!(
            "★★ the restyle applied {applied} of {ops} calls: `{}`, `{}`.",
            line.raw, verdict.raw
        )));
    }
    if line.get("hurried") != Some("-") {
        return Ok(Some(format!(
            "★★ the restyle was run to its end in one frame: `{}`.",
            line.raw
        )));
    }
    if engine > SPREAD_MS && slices < 2 {
        return Ok(Some(format!(
            "★★ {engine} ms of engine time ran in {slices} slice: `{}`.",
            line.raw
        )));
    }
    report.note(format!("★★ `{}`", line.raw));
    Ok(None)
}

/// One Ctrl+Z takes the whole restyle back: the first run reads `before` again.
fn undo_whole(
    session: &Session,
    pointer: &ScriptedPointer,
    report: &mut CheckReport,
    before: f64,
    wanted: f64,
) -> Result<Option<String>> {
    let Some((after, _)) = wait(20_000, || {
        session.settle(4);
        synced(session, 0).map(|s| s.filter(|(size, _)| (size - wanted).abs() < 0.05))
    })?
    else {
        return Ok(Some(format!(
            "★★★ the restyle applied and the panel never read the object at {wanted:.0} pt. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Z", Some("ctrl"))?;
    let undone = wait(30_000, || {
        session.settle(4);
        synced(session, mark)
    })?;
    match undone {
        Some((size, _)) if (size - before).abs() < 0.05 => {
            report.note(format!(
                "★★★ one Ctrl+Z took the object from {after:.2} pt back to {size:.2} pt"
            ));
            Ok(None)
        }
        other => Ok(Some(format!(
            "★★★ one Ctrl+Z left the object's first run at {} where it read {before:.2} pt \
             before the restyle: the change is several undo entries. Trace: {}.",
            other.map_or_else(|| "no re-read".to_owned(), |(s, _)| format!("{s:.2} pt")),
            session.trace_path().display()
        ))),
    }
}
