//! `enlarging_pages_reaches_the_recogniser` — File ▸ Recognise text… with
//! *Enlarge each page before reading* ticked and Lanczos chosen shows the
//! page's size before and after, and the run reads every page through the
//! engine's enlargement at ×2: the recogniser itself reports the factor, and
//! the words still come back. Driven with the scripted pointer off the
//! desktop on a copy of `fixtures/synthetic-image-only.pdf`, with `ocrs`.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_upscale.md`.

use crate::checks::ocr_scripted::recognise_choosing;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::report::CheckReport;

const ENGINE: &str = "ocrs";
const ENLARGE: &str = "ocr-upscale"; // ui-text-exempt: a trace region name, never displayed
const LANCZOS: &str = "ocr-upscale-lanczos"; // ui-text-exempt: a trace region name, never displayed
const SIZES: &str = "ocr-upscale-sizes"; // ui-text-exempt: a trace region name, never displayed
const STARTED: &str = "ocr-started"; // ui-text-exempt: a trace event name, never displayed
const RECOGNISED: &str = "ocr-recognised"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct EnlargingPagesReachesTheRecogniser;

impl Check for EnlargingPagesReachesTheRecogniser {
    fn name(&self) -> &'static str {
        "enlarging_pages_reaches_the_recogniser"
    }

    fn defect(&self) -> &'static str {
        "Enlarge each page before reading is ticked and the recogniser reads the page as \
         rendered, the kernel chosen is not the one sent, or the window hides the sizes"
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
    let fixture = crate::fixture::workspace_root()
        .join("fixtures")
        .join("synthetic-image-only.pdf");
    let copy = ctx.out("ocr-upscale.pdf");
    std::fs::copy(&fixture, &copy).map_err(|e| Error::new(format!("copying the fixture: {e}")))?;
    let (trace, path) = recognise_choosing(
        ctx,
        report,
        &copy,
        ENGINE,
        "upscale",
        false,
        &[ENLARGE, LANCZOS],
    )?;
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");
    // Ever drawn: the window closes on Run, which retires the region.
    let drawn = trace
        .lines
        .iter()
        .any(|l| l.event == ui_rect && l.get("name") == Some(SIZES));
    if !drawn {
        return Ok(Some(format!(
            "★ the group was ticked and never drew its `{SIZES}` line. Trace: {path}."
        )));
    }
    let Some(started) = trace.last(STARTED) else {
        return Ok(Some(format!("Run traced no `{STARTED}`. Trace: {path}.")));
    };
    report.note(format!("run: `{}`", started.raw));
    let asked = [
        ("upscale", "2"),
        ("filter", "lanczos"),
        ("magnify", "smooth"),
    ];
    if let Some((key, want)) = asked.iter().find(|(k, v)| started.get(k) != Some(v)) {
        return Ok(Some(format!(
            "★ the run asked for {key}={:?}, not {want}: `{}`. Trace: {path}.",
            started.get(key),
            started.raw
        )));
    }
    let Some(read) = trace.last(RECOGNISED) else {
        return Ok(Some(format!(
            "★ no `{RECOGNISED}` line: the run refused or never ended. Trace: {path}."
        )));
    };
    report.note(format!("read: `{}`", read.raw));
    Ok(
        enlarged_twice(read.get("enlarged"), read.get_usize("recognised"))
            .map(|why| format!("{why} `{}`. Trace: {path}.", read.raw)),
    )
}

/// `enlarged` is `WxH->W2xH2`, the recogniser's own enlargement of the last
/// page; it must be exactly double on both axes, with words read.
fn enlarged_twice(enlarged: Option<&str>, words: Option<usize>) -> Option<String> {
    let sizes = enlarged.and_then(|e| {
        let (from, to) = e.split_once("->")?;
        let pair = |s: &str| -> Option<(u32, u32)> {
            let (w, h) = s.split_once('x')?;
            Some((w.parse().ok()?, h.parse().ok()?))
        };
        Some((pair(from)?, pair(to)?))
    });
    let Some(((w, h), (w2, h2))) = sizes else {
        return Some(
            "★★ the recogniser reports no enlargement: it read the page as rendered.".into(),
        );
    };
    if (w2, h2) != (w * 2, h * 2) {
        return Some(format!(
            "★★ the page went from {w}x{h} to {w2}x{h2}, not twice the size."
        ));
    }
    if words.unwrap_or(0) == 0 {
        return Some("★★ the enlarged page read no words.".into());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::enlarged_twice;

    #[test]
    fn only_an_exact_doubling_with_words_passes() {
        assert_eq!(enlarged_twice(Some("100x50->200x100"), Some(3)), None);
        assert!(enlarged_twice(Some("none"), Some(3)).is_some());
        assert!(enlarged_twice(Some("100x50->300x150"), Some(3)).is_some());
        assert!(enlarged_twice(Some("100x50->200x100"), Some(0)).is_some());
        assert!(enlarged_twice(None, Some(3)).is_some());
    }
}
