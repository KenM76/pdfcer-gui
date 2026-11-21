//! `a_donor_face_makes_bold_real` — **a font file offered from the
//! Properties panel turns a synthesized Bold into the real face, embedded**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/style_donor.md`.

use crate::checks::two_codes::save;
use crate::checks::word_styles::{Driven, launch_with_env};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::report::CheckReport;

const FIXTURE: &str = "subset-font-floor.pdf";
/// The middle of the page's one run, `ABC` in `pdfceSubsetDemo`.
const IN_RUN: (f64, f64) = (115.2, 612.0);
const INVOKE: &str = "mode.edit,edit.text,file.properties";
const PICKER_ENV: &str = "PDFCER_DIAG_FONT_FILE_PATH";
/// The donor's name: the run's family stem with the Bold the gesture wants.
const DONOR_FILE: &str = "pdfceSubsetDemo-Bold.ttf";
const DONOR_REGION: &str = "properties.text.style-donor"; // ui-text-exempt: a trace region name, never displayed
const BOLD_REGION: &str = "properties.text.bold"; // ui-text-exempt: a trace region name, never displayed
const FORECAST: &str = "text-style-forecast"; // ui-text-exempt: a trace event name, never displayed
const ADDED: &str = "text-style-donor-added"; // ui-text-exempt: a trace event name, never displayed
const LADDER: &str = "text-style-ladder"; // ui-text-exempt: a trace event name, never displayed
const SUPPLIED: &str = "supplied-face"; // ui-text-exempt: the engine's rung token, never displayed
/// The embedded face's name in the saved revision.
const EMBEDDED: &str = "+pdfceSubsetDemo-Bold"; // ui-text-exempt: a needle for the saved bytes

/// See the module documentation.
pub struct ADonorFaceMakesBoldReal;

impl Check for ADonorFaceMakesBoldReal {
    fn name(&self) -> &'static str {
        "a_donor_face_makes_bold_real"
    }

    fn defect(&self) -> &'static str {
        "a font file offered for Bold was not used: the forecast or the restyle stayed \
         synthesized, or the saved file did not embed the offered face"
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

/// The last bold forecast after `mark`, raw, with its rung.
fn bold_forecast(d: &Driven, mark: usize) -> Result<Option<(String, String)>> {
    Ok(d.session
        .trace()?
        .events(FORECAST)
        .filter(|l| l.lineno > mark && l.get("axis") == Some("bold"))
        .last()
        .map(|l| (l.raw.clone(), l.get("rung").unwrap_or_default().to_owned())))
}

/// Whether `bytes`, or any Flate stream in them, names the embedded face.
fn names_face(bytes: &[u8]) -> bool {
    let hit = |b: &[u8]| b.windows(EMBEDDED.len()).any(|w| w == EMBEDDED.as_bytes());
    hit(bytes)
        || bytes
            .windows(6)
            .enumerate()
            .filter(|(_, w)| *w == b"stream")
            .any(|(at, _)| {
                let body = &bytes[at + 6..];
                let start = body.iter().position(|&c| c != b'\r' && c != b'\n');
                start.is_some_and(|s| {
                    miniz_oxide::inflate::decompress_to_vec_zlib(&body[s..])
                        .is_ok_and(|plain| hit(&plain))
                })
            })
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let donor = ctx.out(DONOR_FILE);
    let face = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/augment-fonts/face.ttf");
    std::fs::copy(&face, &donor).map_err(|e| Error::new(format!("copying the donor: {e}")))?;
    let donor_s = donor.display().to_string();
    let (d, doc) = launch_with_env(
        ctx,
        report,
        ("style-donor", FIXTURE),
        INVOKE,
        &[(PICKER_ENV, donor_s.as_str())],
    )?;
    let source_len = std::fs::metadata(&doc).map_or(0, |m| usize::try_from(m.len()).unwrap_or(0));
    if let Err(why) = d.caret_at(IN_RUN)? {
        return Ok(Some(why));
    }
    d.session.settle(20);
    let before = bold_forecast(&d, 0)?;
    if !d.declares(DONOR_REGION)? {
        d.pointer.gone(&d.session)?;
        return Ok(Some(format!(
            "with Bold forecast as {before:?} the Properties panel drew no `{DONOR_REGION}`. \
             Trace: {}.",
            d.path()
        )));
    }
    let mark = d.session.trace()?.mark();
    d.click(DONOR_REGION)?;
    d.session.settle(30);
    let added = d
        .session
        .trace()?
        .last_after(ADDED, mark)
        .map(|l| l.raw.clone());
    let after = bold_forecast(&d, mark)?;
    let mark = d.session.trace()?.mark();
    d.click(BOLD_REGION)?;
    d.session.settle(40);
    let ladder = d
        .session
        .trace()?
        .last_after(LADDER, mark)
        .map(|l| l.raw.clone());
    let (saved, tail) = save(&d, &doc, source_len)?;
    d.pointer.gone(&d.session)?;
    let embedded = names_face(&tail);
    report.note(format!(
        "before: {before:?}; added: {added:?}; after: {after:?}; Bold: {ladder:?}; \
         saved={saved} embedded={embedded}"
    ));
    let mut findings = Vec::new();
    if added.as_deref().is_none_or(|l| !l.contains("donors=1")) {
        findings.push(format!(
            "the button added no donor: `{ADDED}` was {added:?}."
        ));
    }
    if after.as_ref().is_none_or(|(_, rung)| rung != SUPPLIED) {
        findings.push(format!(
            "with the donor offered the Bold forecast was {after:?}; rung `{SUPPLIED}` was owed."
        ));
    }
    if ladder
        .as_deref()
        .is_none_or(|l| !l.contains(&format!("rung={SUPPLIED}")))
    {
        findings.push(format!(
            "Bold did not bind the donor: `{LADDER}` was {ladder:?}."
        ));
    }
    if !saved || !embedded {
        findings.push(format!(
            "the saved revision does not embed `{EMBEDDED}` (saved={saved})."
        ));
    }
    Ok((!findings.is_empty()).then(|| format!("{} Trace: {}.", findings.join(" "), d.path())))
}
