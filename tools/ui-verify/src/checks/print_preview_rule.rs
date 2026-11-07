//! `print_preview_follows_the_paper_rule` — the Print dialog's preview, and the
//! spool that shares its options, obey paper's rule rather than the screen's.
//!
//! Driven off the desktop with the scripted pointer: `file.print` is rung on a
//! page built here, and the oracle is the share of the preview's raster that is
//! not white, read from `print-preview-render ink= of=`. See
//! `docs/modules/ui-verify/checks/print_preview_rule.md` for what each wrong share
//! means.

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const RENDER: &str = "print-preview-render";
const OPEN: &str = "print-open";
/// A square of each, in points on a 200-point page, as a share of the page: the
/// page's own content, an annotation flagged for paper and not the screen, an
/// annotation with `Print` clear, and content in a layer whose `Print` usage is off.
const CONTENT: f64 = 900.0 / 40_000.0;
const PAPER_ONLY: f64 = 900.0 / 40_000.0;
const SCREEN_ONLY: f64 = 3_600.0 / 40_000.0;
const NOT_ON_PAPER_LAYER: f64 = 6_400.0 / 40_000.0;
/// How far a share may sit from its sum: edge pixels, nothing a square could hide in.
const SLACK: f64 = 0.005;

/// See the module documentation.
pub struct PrintPreviewFollowsThePaperRule;

impl Check for PrintPreviewFollowsThePaperRule {
    fn name(&self) -> &'static str {
        "print_preview_follows_the_paper_rule"
    }

    fn defect(&self) -> &'static str {
        "Print draws what the screen draws: an annotation that is not for paper comes out, one \
         that is only for paper does not, or a layer the file keeps off paper is printed"
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

/// A form XObject's dictionary and stream, filled black over `side` points.
fn black_form(side: u32) -> String {
    let body = format!("0 g 0 0 {side} {side} re f");
    format!(
        "<< /Type /XObject /Subtype /Form /BBox [0 0 {side} {side}] /Length {} >>\nstream\n{body}\nendstream",
        body.len()
    )
}

/// One page holding the four squares `CONTENT` to `NOT_ON_PAPER_LAYER` name, none
/// overlapping. `/Watermark` paints under every scope the dialog opens on but
/// fields-only, so the annotations' flags are all that decide them.
fn page_pdf() -> Vec<u8> {
    let content = "0 g 10 10 30 30 re f /OC /G0 BDC 0 g 50 10 80 80 re f EMC";
    let annot = |rect: &str, flags: u32, form: u32| {
        format!(
            "<< /Type /Annot /Subtype /Watermark /Rect [{rect}] /F {flags} /AP << /N {form} 0 R >> >>"
        )
    };
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R /OCProperties << /OCGs [5 0 R] /D << /ON [5 0 R] \
         /AS [<< /Event /Print /OCGs [5 0 R] /Category [/Print] >>] >> >> >>"
            .to_owned(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 200 200] >>".to_owned(),
        "<< /Type /Page /Parent 2 0 R /Contents 4 0 R /Annots [6 0 R 7 0 R] \
         /Resources << /Properties << /G0 5 0 R >> >> >>"
            .to_owned(),
        format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        ),
        "<< /Type /OCG /Name (Screen only) /Usage << /Print << /PrintState /OFF >> >> >>"
            .to_owned(),
        annot("10 150 40 180", 36, 8),
        annot("100 130 160 190", 0, 9),
        black_form(30),
        black_form(60),
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

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe, or build the default."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let doc = ctx.out("print_preview_rule.pdf");
    std::fs::write(&doc, page_pdf())
        .map_err(|e| Error::new(format!("writing the fixture: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("print_preview_rule.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", "file.print"),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("print_preview_rule.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(90);
    Ok((session, pointer))
}

/// Which mistake `share` of ink is, or `None` when it is the right picture.
fn wrong(share: f64) -> Option<&'static str> {
    let is = |want: f64| (share - want).abs() <= SLACK;
    let right = CONTENT + PAPER_ONLY;
    if is(right) {
        None
    } else if is(right + SCREEN_ONLY) {
        Some("an annotation with Print clear reached the preview")
    } else if is(right + NOT_ON_PAPER_LAYER) {
        Some("a layer whose Print usage is off reached the preview")
    } else if is(CONTENT + SCREEN_ONLY) {
        Some("the annotations followed the screen's rule instead of paper's")
    } else if is(CONTENT) {
        Some("an annotation flagged for paper alone did not reach the preview")
    } else {
        Some("no one mistake accounts for this share")
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report)?;
    let trace = session.trace()?;
    if trace.last(OPEN).is_none() {
        return Err(Error::new(format!(
            "`file.print` was rung and no `{OPEN}` line followed. That is \
             `print_dialog_reaches_the_spooler`'s subject."
        )));
    }
    let Some(line) = trace.last(RENDER) else {
        return Ok(Some(format!(
            "★★★ the dialog opened and no `{RENDER}` line followed: nothing was previewed."
        )));
    };
    report.note(line.raw.clone());
    let (Some(ink), Some(of)) = (line.get_usize("ink"), line.get_usize("of")) else {
        return Ok(Some(format!(
            "★★★ `{RENDER}` does not state `ink=` and `of=`: `{}`.",
            line.raw
        )));
    };
    #[allow(clippy::cast_precision_loss)]
    let share = ink as f64 / of.max(1) as f64;
    if let Some(mistake) = wrong(share) {
        return Ok(Some(format!(
            "★★★ {mistake}: {:.2}% of the preview is ink where {:.2}% belongs. `{}`",
            share * 100.0,
            (CONTENT + PAPER_ONLY) * 100.0,
            line.raw
        )));
    }
    pointer.gone(&session)?;
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_mistake_has_its_own_name() {
        let right = CONTENT + PAPER_ONLY;
        assert_eq!(wrong(right), None);
        for share in [
            right + SCREEN_ONLY,
            right + NOT_ON_PAPER_LAYER,
            CONTENT + SCREEN_ONLY,
            CONTENT,
            0.5,
        ] {
            assert!(wrong(share).is_some(), "{share}");
        }
    }
}
