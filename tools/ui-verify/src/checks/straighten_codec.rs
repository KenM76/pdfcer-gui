//! `straighten_scans_stores_the_scan_as_chosen` — the Straighten scans
//! window's *Store the straightened scan* choice reaches the saved file. Two
//! launches on copies of `fixtures/skewed-scans.pdf`, whose scans are
//! Flate-compressed grey: left at *As the original*, the saved copy holds no
//! JPEG stream; with *JPEG* chosen, each of the two straightened pages' new
//! picture is a `/DCTDecode` stream. Driven with the scripted pointer off the
//! desktop; judged in the bytes of the saved copy.

use super::dimdrive::{Fixture, launch_with, press_on_tab};
use crate::checks::straighten_scans::{await_finished, click, open_dialog};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::{Error, Result};
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "skewed-scans.pdf",
    method: "Rebuild it with `python fixtures/skewed-scans.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
const JPEG: &str = "deskew.codec.jpeg"; // ui-text-exempt: a trace region name, never displayed
const COMMIT: &str = "deskew.commit"; // ui-text-exempt: a trace region name, never displayed
const FILE_TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name, never displayed
const SAVE_COPY: &str = "ribbon.item.file.save_copy"; // ui-text-exempt: a trace region name, never displayed
const STARTED: &str = "deskew-started"; // ui-text-exempt: a trace event name, never displayed
const SAVED: &str = "save-copy"; // ui-text-exempt: a trace event name, never displayed
const DCT: &[u8] = b"/DCTDecode";
/// The fixture's tilted pages, each straightened once.
const STRAIGHTENED: usize = 2;

/// See the module documentation.
pub struct StraightenScansStoresTheScanAsChosen;

impl Check for StraightenScansStoresTheScanAsChosen {
    fn name(&self) -> &'static str {
        "straighten_scans_stores_the_scan_as_chosen"
    }

    fn defect(&self) -> &'static str {
        "Straighten scans ignores its storage choice: JPEG chosen and the saved scans are not \
         JPEG, or left as the original and a lossless scan comes back JPEG"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let judged = straighten(ctx, &mut report, "keep", false).and_then(|keep| {
            let jpeg = straighten(ctx, &mut report, "jpeg", true)?;
            Ok(judge(keep, jpeg))
        });
        match judged {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// What one launch left: its start line's codec word and the saved copy's
/// count of JPEG streams, or the failure sentence.
type Launch = std::result::Result<(String, usize), String>;

fn judge(keep: Launch, jpeg: Launch) -> Option<String> {
    let (keep, jpeg) = match (keep, jpeg) {
        (Err(why), _) | (_, Err(why)) => return Some(why),
        (Ok(k), Ok(j)) => (k, j),
    };
    if keep.0 != "keep" || jpeg.0 != "jpeg" {
        return Some(format!(
            "the runs started with codec `{}` and `{}`; the window's choice did not reach the \
             run (expected `keep`, then `jpeg`).",
            keep.0, jpeg.0
        ));
    }
    if keep.1 != 0 {
        return Some(format!(
            "left as the original, the Flate scans came back as {} JPEG stream(s).",
            keep.1
        ));
    }
    (jpeg.1 < STRAIGHTENED).then(|| {
        format!(
            "with JPEG chosen the saved copy holds {} JPEG stream(s), not one for each of the \
             {STRAIGHTENED} straightened pages.",
            jpeg.1
        )
    })
}

/// Open Straighten scans, pick JPEG when asked, run it, save a copy.
fn straighten(
    ctx: &CheckContext,
    report: &mut CheckReport,
    tag: &str,
    jpeg: bool,
) -> Result<Launch> {
    let stem = format!("straighten-codec.{tag}");
    let saved = ctx.out(&format!("{stem}.saved.pdf"));
    let _ = std::fs::remove_file(&saved);
    let env = [(SAVE_PATH_ENV, saved.display().to_string())];
    let (session, pointer) = launch_with(ctx, report, &FIXTURE, "mode.read", &stem, &env)?;
    let outcome = (|| -> Result<Launch> {
        open_dialog(&session, &pointer)?;
        if jpeg {
            click(&session, &pointer, JPEG)?;
        }
        click(&session, &pointer, COMMIT)?;
        if await_finished(&session)?.is_none() {
            return Ok(Err(format!(
                "{tag}: Straighten was pressed and the run never finished. Trace: {}.",
                session.trace_path().display()
            )));
        }
        let trace = session.trace()?;
        let started = trace.events(STARTED).last().cloned();
        let codec = started
            .as_ref()
            .and_then(|l| l.get("codec"))
            .unwrap_or("absent")
            .to_owned();
        if let Some(line) = &started {
            report.note(format!("{tag}: `{}`", line.raw));
        }
        let mark = trace.mark();
        press_on_tab(ctx, &session, &pointer, SAVE_COPY, FILE_TAB)?;
        session.settle(40);
        if session.trace()?.last_after(SAVED, mark).is_none() {
            return Ok(Err(format!(
                "{tag}: Save a copy was pressed and no `{SAVED}` followed. Trace: {}.",
                session.trace_path().display()
            )));
        }
        let bytes = std::fs::read(&saved)
            .map_err(|e| Error::new(format!("cannot read {}: {e}", saved.display())))?;
        let count = bytes.windows(DCT.len()).filter(|w| *w == DCT).count();
        report.note(format!(
            "{tag}: the saved copy holds {count} JPEG stream(s)"
        ));
        report.artifact(saved.clone());
        Ok(Ok((codec, count)))
    })();
    let parked = pointer.gone(&session);
    let outcome = outcome?;
    parked?;
    Ok(outcome)
}
