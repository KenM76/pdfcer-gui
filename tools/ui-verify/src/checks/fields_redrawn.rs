//! # `checks::fields_redrawn` — a form that asks viewers to redraw its fields
//! says so in the status bar, and the same form without the ask does not.
//!
//! Doc: `docs/modules/ui-verify/checks/fields_redrawn.md`.

use crate::checks::driving::declared_names;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const REGION: &str = "status-group:fields-redrawn";
const RASTER_EVENT: &str = "raster-blend-space";
const COUNT_KEY: &str = "fields_redrawn";
const WAIT_TICKS: usize = 40;

pub struct FieldsRedrawnForNeedAppearancesAreDisclosed;

impl Check for FieldsRedrawnForNeedAppearancesAreDisclosed {
    fn name(&self) -> &'static str {
        "fields_redrawn_for_need_appearances_are_disclosed"
    }

    fn defect(&self) -> &'static str {
        "a form sets /NeedAppearances, so pdfcer draws its fields from their values instead of \
         their saved look and the canvas can differ from another viewer, and nothing on screen \
         says so"
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

/// One page with one text field whose saved look fills its box red and whose
/// value is empty, so no font finding joins the render notes. `need` sets
/// `/NeedAppearances`.
fn form_pdf(need: bool) -> Vec<u8> {
    let ap = "1 0 0 rg 0 0 200 40 re f\n";
    let objects = [
        format!(
            "<< /Type /Catalog /Pages 2 0 R /AcroForm << /Fields [5 0 R] \
             /NeedAppearances {need} /DA (/Helv 12 Tf 0 g) >> >>"
        ),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 400] /Resources << >> \
         /Contents 4 0 R /Annots [5 0 R] >>"
            .to_owned(),
        "<< /Length 0 >>\nstream\n\nendstream".to_owned(),
        "<< /Type /Annot /Subtype /Widget /FT /Tx /T (Stale) /Rect [100 100 300 140] /F 4 \
         /P 3 0 R /DA (/Helv 12 Tf 0 g) /V () /AP << /N 6 0 R >> >>"
            .to_owned(),
        format!(
            "<< /Type /XObject /Subtype /Form /BBox [0 0 200 40] /Resources << >> \
             /Length {} >>\nstream\n{ap}endstream",
            ap.len()
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

/// The engine's count for the first finished raster, and whether the line
/// was declared afterwards.
struct Seen {
    counted: Option<u64>,
    shown: bool,
}

fn open_and_look(ctx: &CheckContext, report: &mut CheckReport, need: bool) -> Result<Seen> {
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
    let pdf = ctx.out(&format!("fields_redrawn.need-{need}.pdf"));
    std::fs::write(&pdf, form_pdf(need))
        .map_err(|why| Error::new(format!("could not write {}: {why}", pdf.display())))?;
    report.artifact(pdf.clone());

    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("fields_redrawn.{need}.trace.txt")));
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
        counted = session
            .trace()?
            .last(RASTER_EVENT)
            .and_then(|l| l.get(COUNT_KEY))
            .and_then(|v| v.parse().ok());
        if counted.is_some() {
            session.settle(10);
            break;
        }
    }
    let shown = declared_names(&session.trace()?, ui_rect, REGION)
        .iter()
        .any(|n| n == REGION);
    Ok(Seen { counted, shown })
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let asked = open_and_look(ctx, report, true)?;
    let Some(n) = asked.counted else {
        return Err(Error::new(format!(
            "no `{RASTER_EVENT}` line carrying `{COUNT_KEY}=` after opening the form, so no \
             raster completed or the binary predates the field. Rebuild and re-run."
        )));
    };
    report.note(format!(
        "/NeedAppearances true: engine counted {COUNT_KEY}={n}"
    ));
    if n == 0 {
        return Err(Error::new(
            "the engine redrew nothing on a form that sets /NeedAppearances over a field with an \
             /AP. Either the render no longer honours the flag or the count is not wired; this \
             fixture no longer exercises the line.",
        ));
    }
    if !asked.shown {
        return Ok(Some(format!(
            "the engine redrew {n} field(s) from their values and `{REGION}` was never declared. \
             The count reached the texture and the status bar did not read it; see \
             `fields_redrawn_disclosure`."
        )));
    }
    report.note("/NeedAppearances true: the disclosure is on screen");

    let control = open_and_look(ctx, report, false)?;
    let Some(n) = control.counted else {
        return Err(Error::new(format!(
            "no `{RASTER_EVENT}` line after opening the control, so its absence of the \
             disclosure would witness nothing."
        )));
    };
    report.note(format!(
        "/NeedAppearances false: engine counted {COUNT_KEY}={n}"
    ));
    if n != 0 {
        return Err(Error::new(format!(
            "the engine redrew {n} field(s) on a form that does not ask it to, so the control \
             does not control."
        )));
    }
    if control.shown {
        return Ok(Some(format!(
            "`{REGION}` is declared on a form whose fields were drawn from their saved look. A \
             disclosure that shows on a healthy page says nothing on an unhealthy one."
        )));
    }
    report.note("/NeedAppearances false: no disclosure, which is correct");
    Ok(None)
}
