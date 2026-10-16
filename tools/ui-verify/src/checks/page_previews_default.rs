//! `a_fresh_profile_draws_page_previews_with_no_limit` — with no preference
//! written, the Pages panel comes up with its preview time limit at `0`, the
//! operator's word for *no limit* (O225).
//!
//! Design and rationale: `docs/modules/ui-verify/checks/page_previews_default.md`.

use super::dimdrive::{Fixture, launch_with};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::{Error, Result};
use crate::report::CheckReport;

const PANEL_EVENT: &str = "pages-panel"; // ui-text-exempt: a trace event name, never displayed
const BUDGET_KEY: &str = "page_preview_budget_ms"; // ui-text-exempt: a preferences key, never displayed

const FIXTURE: Fixture = Fixture {
    file: "four-pages.pdf",
    method: "It is checked in; restore it from git.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};

/// A fresh profile's page previews have no time limit.
pub struct AFreshProfileDrawsPagePreviewsWithNoLimit;

impl Check for AFreshProfileDrawsPagePreviewsWithNoLimit {
    fn name(&self) -> &'static str {
        "a_fresh_profile_draws_page_previews_with_no_limit"
    }

    fn defect(&self) -> &'static str {
        "a fresh profile's Pages panel limits how long page previews may take to draw, where \
         the operator asked for no limit by default"
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
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let userdata = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata");
    crate::sandbox::reset_prefs(&userdata)
        .map_err(|e| Error::new(format!("could not reset preferences: {e}")))?;
    let written = std::fs::read_to_string(userdata.join("preferences.txt")).unwrap_or_default();
    if written.contains(BUDGET_KEY) {
        return Err(Error::new(format!(
            "the reset preferences still name `{BUDGET_KEY}`, so the launch would read a \
             planted value rather than the default"
        )));
    }
    let (session, pointer) = launch_with(
        ctx,
        report,
        &FIXTURE,
        "mode.review",
        "previews-default",
        &[],
    )?;
    session.settle(60);
    let line = session.trace()?.last(PANEL_EVENT).map(|l| l.raw.clone());
    pointer.gone(&session)?;
    report.note(format!("panel: {line:?}"));
    let Some(line) = line else {
        return Err(Error::new(format!(
            "the Pages panel traced no `{PANEL_EVENT}` line in Review mode, so its limit was \
             never shown. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let budget = line
        .split_whitespace()
        .find_map(|w| w.strip_prefix("budget_ms="));
    Ok((budget != Some("0")).then(|| {
        format!(
            "a fresh profile's Pages panel traced `{line}`; want budget_ms=0, no limit. Trace: \
             {}.",
            session.trace_path().display()
        )
    }))
}
