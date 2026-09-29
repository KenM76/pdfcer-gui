//! `spots_flattened` — **a page naming more spot inks than the renderer keeps
//! separate says so in the status bar, and a page within the limit does not.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/spots_flattened.md`.

use crate::checks::driving::declared_names;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The status-bar disclosure's published region.
const REGION: &str = "status-group:spots-flattened";
/// The render worker's per-raster line; `spots_flattened=` is the engine's count.
const RASTER_EVENT: &str = "raster-blend-space";
/// Settle rounds to wait for the first raster.
const WAIT_TICKS: usize = 40;

/// See the module documentation.
pub struct ExtraSpotInksAreDisclosed;

impl Check for ExtraSpotInksAreDisclosed {
    fn name(&self) -> &'static str {
        "spots_flattened"
    }

    fn defect(&self) -> &'static str {
        "a page names more spot inks than the renderer keeps as separate colours; the extras \
         are drawn as process colour, so where they overprint or blend the canvas shows the \
         wrong colour, and nothing on screen says so"
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

/// A one-page PDF painting `inks` distinct `/Separation` inks, each twice so a
/// per-fill tally would read 2 rather than 1. Page group CMYK, so the
/// colorant buffer is engaged.
fn spots_pdf(inks: usize) -> Vec<u8> {
    let spaces: String = (0..inks)
        .map(|i| {
            format!(
                "/CS{i} [/Separation /Ink{i} /DeviceCMYK \
                 << /FunctionType 2 /Domain [0 1] /C0 [0 0 0 0] /C1 [0 0.5 1 0] /N 1 >>] "
            )
        })
        .collect();
    let content: String = (0..inks)
        .map(|i| {
            format!(
                "/CS{i} cs 1 scn {x} 10 10 10 re f {x} 30 10 10 re f\n",
                x = 10 + 20 * i
            )
        })
        .collect();
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 200 100] >>".to_owned(),
        format!(
            "<< /Type /Page /Parent 2 0 R /Contents 4 0 R \
             /Group << /S /Transparency /CS /DeviceCMYK >> \
             /Resources << /ColorSpace << {spaces}>> >> >>"
        ),
        format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        ),
    ];
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for off in offsets {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

/// What one launch saw: the engine's count on the last raster, and whether the
/// region was ever declared.
struct Seen {
    counted: Option<u64>,
    shown: bool,
}

fn open_and_look(ctx: &CheckContext, report: &mut CheckReport, inks: usize) -> Result<Seen> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let pdf = ctx.out(&format!("spots_flattened.{inks}-inks.pdf"));
    std::fs::write(&pdf, spots_pdf(inks))
        .map_err(|why| Error::new(format!("could not write {}: {why}", pdf.display())))?;
    report.artifact(pdf.clone());

    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("spots_flattened.{inks}.trace.txt")));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());

    let mut counted = None;
    for _ in 0..WAIT_TICKS {
        session.settle(5);
        let trace = session.trace()?;
        counted = trace
            .last(RASTER_EVENT)
            .and_then(|l| l.get("spots_flattened"))
            .and_then(|v| v.parse().ok());
        if counted.is_some() {
            // One more settle, so the frame after the texture lands has drawn
            // the status bar from it.
            session.settle(10);
            break;
        }
    }
    let trace = session.trace()?;
    let shown = declared_names(&trace, ui_rect, REGION)
        .iter()
        .any(|n| n == REGION);
    Ok(Seen { counted, shown })
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    // --- A: five inks, one beyond the four-plane roster --------------------
    let five = open_and_look(ctx, report, 5)?;
    let Some(n) = five.counted else {
        return Err(Error::new(format!(
            "no `{RASTER_EVENT}` line carrying `spots_flattened=` after opening the five-ink \
             page, so no raster completed or the binary predates the field. Rebuild and re-run."
        )));
    };
    report.note(format!("five inks: engine counted spots_flattened={n}"));
    if n == 0 {
        return Err(Error::new(
            "the engine flattened nothing on a five-ink page. Either the plane roster grew past \
             four or the count is not wired; this check's fixture no longer exercises the line.",
        ));
    }
    if !five.shown {
        return Ok(Some(format!(
            "the engine reported {n} flattened spot ink(s) and `{REGION}` was never declared. \
             The count reached the texture and the status bar did not read it — see \
             `spots_flattened_disclosure`."
        )));
    }
    report.note("★ five inks: the disclosure is on screen");

    // --- B: the control, four inks, nothing flattened ----------------------
    let four = open_and_look(ctx, report, 4)?;
    let Some(n) = four.counted else {
        return Err(Error::new(format!(
            "no `{RASTER_EVENT}` line after opening the four-ink control, so its absence of \
             the disclosure would witness nothing."
        )));
    };
    report.note(format!("four inks: engine counted spots_flattened={n}"));
    if n != 0 {
        return Err(Error::new(format!(
            "the engine flattened {n} ink(s) on a four-ink page, so the control does not \
             control. The plane roster may have shrunk."
        )));
    }
    if four.shown {
        return Ok(Some(format!(
            "`{REGION}` is declared on a page that flattened nothing. A disclosure that shows \
             on a healthy page says nothing on an unhealthy one."
        )));
    }
    report.note("★ four inks: no disclosure, which is correct");
    Ok(None)
}
