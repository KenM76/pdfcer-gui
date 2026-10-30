//! `print_opens_on_the_documents_own_settings` — a PDF that sets its print
//! presets in `/ViewerPreferences` opens the Print dialog on them.
//!
//! Driven off the desktop with no input: `file.print` is rung through
//! `PDFCER_DIAG_INVOKE` on `print-presets.pdf`, which asks for actual size
//! (enforced), pages 2-3, two copies, long-edge duplex and tray by size.
//! Oracles: the `print-presets` line (five settings used, scale locked, range
//! `2-3`) and the `print-open` line built after it carrying the same values.

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "print-presets.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/print-presets.PROVENANCE.py`.";
const PRESETS: &str = "print-presets";
const OPEN: &str = "print-open";

/// `print-open` fields and the values the fixture asks for.
const EXPECTED: [(&str, &str); 4] = [
    ("scale", "actual"),
    ("copies", "2"),
    ("duplex", "long-edge"),
    ("tray", "true"),
];

/// See the module documentation.
pub struct PrintOpensOnTheDocumentsOwnSettings;

impl Check for PrintOpensOnTheDocumentsOwnSettings {
    fn name(&self) -> &'static str {
        "print_opens_on_the_documents_own_settings"
    }

    fn defect(&self) -> &'static str {
        "the Print dialog ignores the print settings a PDF carries (actual size, page range, \
         copies, two-sided, tray by size)"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let doc = ctx.out("print-presets.pdf");
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("print_presets.trace.txt"));
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
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(90);

    let trace = session.trace()?;
    let Some(open) = trace.last(OPEN) else {
        return Err(Error::new(format!(
            "`file.print` was rung and no `{OPEN}` line followed, so the dialog never opened. \
             That is `print_dialog_reaches_the_spooler`'s subject. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(open.raw.clone());
    let Some(presets) = trace.last(PRESETS) else {
        return Ok(Some(format!(
            "★★★ the dialog opened on a document that sets five print presets and traced no \
             `{PRESETS}` line: the document's own settings were never read. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(presets.raw.clone());
    let mut wrong: Vec<String> = Vec::new();
    for (field, want) in [("used", "5"), ("locked", "true"), ("range", "2-3")] {
        let got = presets.get(field).unwrap_or("<absent>");
        if got != want {
            wrong.push(format!("{PRESETS} {field}={got}, want {want}"));
        }
    }
    for (field, want) in EXPECTED {
        let got = open.get(field).unwrap_or("<absent>");
        if got != want {
            wrong.push(format!("{OPEN} {field}={got}, want {want}"));
        }
    }
    if wrong.is_empty() {
        return Ok(None);
    }
    Ok(Some(format!(
        "★★★ the dialog did not open on the document's print settings: {}. Trace: {}.",
        wrong.join("; "),
        session.trace_path().display()
    )))
}
