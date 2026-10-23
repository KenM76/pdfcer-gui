//! `scaled_text_shows_and_takes_its_drawn_size` — text written `1 Tf` under
//! an 18-point text matrix, as CAD exports write it, reads 18 in Properties,
//! and typing 24 draws it at 24.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/scaled_text_size.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared_in};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Startup commands: Edit mode with Properties mounted.
const INVOKE: &str = "mode.edit,file.properties";
/// Off-desktop viewport, so the check runs while the operator uses the machine.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// The size field.
const SIZE_REGION: &str = "properties.text.size";
/// The panel's read of the clicked run; `size=` is the drawn size.
const SYNCED: &str = "text-style-synced";
/// The restyle's summary line.
const APPLIED_EVENT: &str = "text-style-applied";
/// The restyle's refusal line.
const DECLINED_EVENT: &str = "text-style-declined";
/// The text matrix's scale, which is the size the text is drawn at.
const DRAWN_PT: f64 = 18.0;
/// What the check types.
const WANTED_PT: f64 = 24.0;
/// Baseline origin of the fixture's one line, in PDF user space.
const ORIGIN: (f64, f64) = (72.0, 700.0);

/// See the module documentation.
pub struct ScaledTextShowsAndTakesItsDrawnSize;

impl Check for ScaledTextShowsAndTakesItsDrawnSize {
    fn name(&self) -> &'static str {
        "scaled_text_shows_and_takes_its_drawn_size"
    }

    fn defect(&self) -> &'static str {
        "CAD text drawn at 13 pt reads `1.0 pt` in Properties, and typing a size writes it as \
         the Tf operand, so the text is redrawn at that many times its scale and vanishes \
         off the page"
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

/// A one-page Letter PDF with one Helvetica line set `1 Tf` under an
/// 18-point text matrix.
fn fixture() -> Vec<u8> {
    let content = format!(
        "BT /F1 1 Tf {DRAWN_PT} 0 0 {DRAWN_PT} {} {} Tm (Scaled label text) Tj ET",
        ORIGIN.0, ORIGIN.1
    );
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

/// The `size=` of the last resolved panel read traced after `mark`.
fn synced_size(session: &Session, mark: usize) -> Result<Option<f64>> {
    let trace = session.trace()?;
    Ok(trace
        .events(SYNCED)
        .filter(|l| l.lineno > mark && l.get("resolved") == Some("1"))
        .last()
        .and_then(|l| l.get("size"))
        .and_then(|v| v.parse().ok()))
}

/// Poll up to `ceiling_ms` for `ready` to answer `Some`.
fn wait<T>(ceiling_ms: u128, mut ready: impl FnMut() -> Result<Option<T>>) -> Result<Option<T>> {
    let started = std::time::Instant::now();
    loop {
        if let Some(v) = ready()? {
            return Ok(Some(v));
        }
        if started.elapsed().as_millis() > ceiling_ms {
            return Ok(None);
        }
    }
}

/// Write the fixture and launch the binary on it.
fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let pdf = ctx.out("scaled-text-size.pdf");
    std::fs::write(&pdf, fixture())
        .map_err(|e| Error::new(format!("cannot write {}: {e}", pdf.display())))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("scaled-text-size.trace.txt"));
    spec.pdf = Some(pdf);
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("scaled-text-size.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(40);
    Ok((session, pointer))
}

/// Click the line as an object and answer the size the panel read.
fn click_text(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<f64>> {
    let vocab = &ctx.profile.vocab;
    let ui_rect = vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    click_mode_segment(session, pointer, ui_rect, "edit")?;
    session.settle(20);
    let page = PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    };
    let mapping = CanvasMapping::from_trace(&session.trace()?, vocab, page, 0)?;
    let at = DocPoint::new(0, ORIGIN.0 + 30.0, ORIGIN.1 + 5.0);
    let mark = session.trace()?.mark();
    pointer.click(session, mapping.doc_to_window(at)?)?;
    wait(10_000, || {
        session.settle(4);
        synced_size(session, mark)
    })
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report)?;
    let Some(read) = click_text(ctx, &session, &pointer)? else {
        return Err(Error::new(format!(
            "a click on the line produced no resolved `{SYNCED}` line, so the panel never \
             read the run. SKIPPED: the aim or the selection failed before this check's \
             subject. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if (read - DRAWN_PT).abs() > 0.05 {
        return Ok(Some(format!(
            "★ Properties reads `{read:.2}` pt for text drawn at {DRAWN_PT} pt (`1 Tf` under a \
             {DRAWN_PT}-point text matrix). `canvas::textedit::pin::inspect` must report \
             `drawn_size`, the Tf operand times `em_scale`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!("★ Properties read {read:.2} pt, the drawn size"));
    type_size(ctx, &session, &pointer, report)
}

/// Type [`WANTED_PT`] into the size field; the restyle's verdict line, or a
/// failure sentence.
fn commit_typed_size(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    mark: usize,
) -> Result<std::result::Result<String, String>> {
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or_default();
    let Some((field, viewport)) = declared_in(&session.trace()?, ui_rect, SIZE_REGION) else {
        return Err(Error::new(format!(
            "no `{SIZE_REGION}` region on screen, so there is nothing to type into. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let vp = viewport.as_deref();
    pointer.click_in(session, vp, WindowPoint::centre_of(field))?;
    session.settle(12);
    pointer.key(session, vp, "A", Some("ctrl"))?;
    pointer.type_text(session, vp, &format!("{WANTED_PT:.0}"))?;
    session.settle(8);
    pointer.key(session, vp, "Enter", None)?;
    let verdict = wait(30_000, || {
        session.settle(4);
        let t = session.trace()?;
        Ok(t.last_after(DECLINED_EVENT, mark)
            .or_else(|| t.last_after(APPLIED_EVENT, mark))
            .map(|l| l.raw.clone()))
    })?;
    Ok(match verdict {
        Some(v) if v.contains(APPLIED_EVENT) => Ok(v),
        Some(v) => Err(format!("the size change was declined: `{v}`")),
        None => Err(format!(
            "{WANTED_PT:.0} was typed and Enter pressed, and no `{APPLIED_EVENT}` or \
             `{DECLINED_EVENT}` line followed. Trace: {}.",
            session.trace_path().display()
        )),
    })
}

/// Type [`WANTED_PT`] and require the panel's re-read to match it.
fn type_size(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    let verdict = match commit_typed_size(ctx, session, pointer, mark)? {
        Ok(v) => v,
        Err(failure) => return Ok(Some(failure)),
    };
    let after = wait(15_000, || {
        session.settle(4);
        synced_size(session, mark)
    })?;
    match after {
        Some(v) if (v - WANTED_PT).abs() <= 0.05 => {
            report.note(format!(
                "★★ typed {WANTED_PT:.0}; the run now reads {v:.2} pt"
            ));
            Ok(None)
        }
        Some(v) => Ok(Some(format!(
            "★ {WANTED_PT:.0} was typed and the run now reads `{v:.2}` pt. A reading of \
             {:.0} is the typed size written as the Tf operand under the {DRAWN_PT}-point \
             matrix, which redraws the text off the page. `StyleChange::for_scale` must \
             divide by the run's `em_scale`. Trace: {}.",
            WANTED_PT * DRAWN_PT,
            session.trace_path().display()
        ))),
        None => Ok(Some(format!(
            "the size applied (`{verdict}`) and the panel never re-read the run, so the \
             result is unmeasured. Trace: {}.",
            session.trace_path().display()
        ))),
    }
}
