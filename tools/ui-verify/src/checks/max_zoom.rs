//! `the_zoom_readout_opens_the_maximum_zoom_popup` — a readout that turned into
//! a button, proved to actually do something.
//!
//! # Why this exists
//!
//! `OPERATOR_REQUESTS.md` O24, and the operator on 2026-08-22:
//!
//! > *"put the max zoom setting on the bar at the bottom."*
//!
//! The status bar's zoom readout is now a button that opens a list of maximums.
//! ★ That is precisely the shape this project has shipped broken twice in one
//! week: the **Select** popup went out with a double toggle that made its button
//! do nothing, green on 1,628 unit tests, 17 gates and a smoke launch that
//! confirmed the button's rect was drawn in the right place. Every one of those
//! observed the *button*, and the button was never the broken part.
//!
//! So this asserts the thing an operator would: **the popup opens, and it
//! contains its rows.**
//!
//! # ★★ What it deliberately does not assert, and why that is honest
//!
//! It does not click a row and check the preference changed. That would be the
//! stronger claim, and it needs the preferences file — which is written to a
//! shared profile directory the harness would then be mutating under a running
//! application. `select_filter_changes_what_a_click_hits` learned that lesson
//! the expensive way: it left a persisted filter behind, its next run started
//! with everything switched off, and it blamed the fixture.
//!
//! The gap is covered from the other side instead. `app::status::maxzoom`'s
//! unit tests assert every preset is a value the preference accepts and that
//! the default is one of the rows, and `app::prefs` asserts the round trip
//! through the file. What no unit test can see is whether the popup opens at
//! all — and that is exactly what this check is for.

use crate::checks::driving;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// The zoom group's declared region — what the readout sits inside.
const ZOOM_REGION: &str = "status-group:zoom";

/// Prefix of the popup's per-row regions.
const ROW_PREFIX: &str = "status-maxzoom-row";

/// See the module documentation.
pub struct TheZoomReadoutOpensTheMaximumZoomPopup;

impl Check for TheZoomReadoutOpensTheMaximumZoomPopup {
    fn name(&self) -> &'static str {
        "the_zoom_readout_opens_the_maximum_zoom_popup"
    }

    fn defect(&self) -> &'static str {
        "the status bar's zoom readout is drawn as a button and clicking it opens nothing — a \
         control that looks available and is inert, which is how the Select popup shipped"
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
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new("no --pdf. The status bar draws no controls with nothing open.")
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks the zoom readout. Reported as \
             SKIPPED rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("max-zoom.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- the control point: nothing is open yet -----------------------------
    //
    // ★ Without this, "rows are declared" after the click could be rows that
    // were always declared, and the check would pass on a build where the popup
    // was permanently open — a different defect wearing the same green tick.
    let trace = session.trace()?;
    if !driving::live_names(&trace, ui_rect, ROW_PREFIX).is_empty() {
        return Ok(Some(
            "the maximum-zoom popup's rows are already declared before anything was clicked, so \
             the popup is open from the first frame. It should open on a click."
                .to_owned(),
        ));
    }

    let zoom = driving::declared(&trace, ui_rect, ZOOM_REGION).ok_or_else(|| {
        Error::new(format!(
            "the status bar never declared `{ZOOM_REGION}`, so the readout cannot be aimed at. \
             Either no document is open or the region was renamed."
        ))
    })?;

    // ★ The readout is the MIDDLE of the zoom group — `− ⟨percent⟩ +` — so the
    // group's centre is the readout. Aiming at an edge would hit a step button
    // and zoom instead, which would then report the popup as not opening.
    let frame = session.frame()?;
    driver.click_at(frame.declared_center(zoom))?;
    session.settle(24);

    let trace = session.trace()?;
    let rows = driving::live_names(&trace, ui_rect, ROW_PREFIX);
    if rows.is_empty() {
        return Ok(Some(
            "★★ CLICKING THE ZOOM READOUT OPENED NOTHING. No `status-maxzoom-row` region was \
             declared after the click, so the popup did not draw. ★ This is the shape the Select \
             popup shipped in: `Popup::menu` already toggles on click, so a second \
             `Popup::toggle_id` beside it cancels the first and the button does nothing — check \
             for one before looking anywhere else."
                .to_owned(),
        ));
    }

    report.note(format!("the popup opened with {} row(s)", rows.len()));
    Ok(None)
}
