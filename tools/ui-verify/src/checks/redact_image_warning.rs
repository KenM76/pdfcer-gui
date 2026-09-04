//! `marking_over_an_image_says_so_before_apply` — a redaction mark that covers a
//! raster image discloses it when the rectangle is drawn, not when Apply is
//! pressed.
//!
//! # The operator's report, `OPERATOR_REQUESTS.md` O103
//!
//! > *"every time I've tried the redact feature it tells me it can't because
//! > there is objects that weren't redacted."*
//!
//! Reproduced with `pdfcer` alone, so the refusal is the engine's:
//!
//! ```text
//! redaction refused: redaction region on page 1 intersects an image; pdfcer
//! cannot yet destroy image pixels … apply refused rather than producing a
//! false redaction
//! ```
//!
//! ★ The refusal was right. What was wrong is *when* he found out — apply was
//! all-or-nothing for the document, so twelve careful marks and one that grazed
//! a logo produced a single refusal naming no region, after the work rather than
//! during it.
//!
//! # ★★★ THE REFUSAL IS GONE, and this check's subject changed with it
//!
//! `pdfcer-core` **v0.26.0** (`Pass 245.0`, 2026-09-03 — the same day as the
//! report) ships all three asks: the gate is on the samples rather than the
//! bounding boxes, a wholly covered image is removed outright, and an
//! undecodable image now retains just the marks that touch it rather than
//! refusing the document. So a region over an image no longer fails; it
//! **destroys those pixels**.
//!
//! The disclosure did not become unnecessary — it changed subject, and to the
//! more important of the two. A raster redaction is irreversible in a way a text
//! one is not: the samples are overwritten and re-encoded, and what comes back
//! is a black block where the logo was. Learning that while the rectangle is
//! being drawn is the same argument the original made, applied to the opposite
//! outcome.
//!
//! ⇒ **The engine's reply told us to re-word it, by name.** Nothing in this
//! repository could have: a claim about an external limitation, phrased as a UI
//! string, compiles and passes for ever after the limitation lifts. See
//! `crate::checks::driving::declared_or_in_overflow` for the same shape found
//! the same morning in this harness's own code.
//!
//! # ★★★ Why this check asserts BOTH directions, and would be worthless with one
//!
//! A check that only proves *"the warning appears on a document with an image"*
//! passes just as happily on a build that warns about **every** mark on every
//! document. That build is worse than no warning at all: a caveat attached to
//! every action is one an operator learns to scroll past, and the day it is
//! load-bearing they will scroll past it too.
//!
//! So the second half is the real assertion: on a drawing with **no** image the
//! same gesture must produce **`disclosures=none`**. The two halves together
//! say the warning tracks the fact rather than the feature being on.
//!
//! ⇒ This is the same shape as `field_shading`'s three states and the tab-order
//! section's openness: a signal that cannot be absent is not a signal.
//!
//! # What a passing run does NOT prove
//!
//! That the count is right. The warning says how many images the region covers,
//! and this asserts only that it is non-zero on a document that has one and
//! absent on a document that has none — the arithmetic is
//! `app::actions::redactimg`'s and is a straight filter over the object model.
//! It also does not press Apply: what happens there is the engine's, was
//! reproduced with the CLI in O103, and is not this shell's to verify.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode the Redact panel is reachable from.
const MODE: &str = "edit";
/// The command that opens the Redact panel, and the tab it lives on.
const PANEL_ITEM: &str = "ribbon.item.edit.redact";
/// The panel's own region, so the toggle is only pressed when it is closed.
const PANEL: &str = "redact-panel";
/// The **Mark whole page** button.
const WHOLE_PAGE: &str = "redact-whole-page";
/// The funnel's line for a whole-page mark; it carries `disclosures=`.
const MARKED: &str = "redact-mark-page";
/// A fixture that is nothing but a raster image.
const WITH_IMAGE: &str = "fixtures/synthetic-image-only.pdf";
/// A real CAD sheet with no raster image on it at all.
const WITHOUT_IMAGE: &str = "fixtures/a1-titleblock.pdf";
/// The words the disclosure must carry when it fires.
///
/// Matched on the CONSEQUENCE rather than on "image", because the sentence has
/// to tell the operator what will happen to him and not merely what is on the
/// page. A build that said "this region covers 1 image(s)" and stopped would
/// pass a looser check and leave him no wiser about what Apply will do.
///
/// ★★★ **It read `"will be refused"` until 2026-09-03, and the words changed
/// because the OUTCOME did.** `pdfcer-core` v0.26.0 (`Pass 245.0`) destroys
/// image samples under a region instead of refusing the document, so the
/// disclosure stopped being a warning about a failure and became a warning
/// about an irreversible success. This constant is what made that a one-line
/// edit: the check asserts *"the consequence is stated"*, and only the
/// consequence moved.
///
/// ⇒ A check pinned to a whole sentence would have gone red here and read as a
/// regression in the shell, when what had happened is that the engine got
/// better.
const CONSEQUENCE: &str = "destroyed, not hidden";

/// See the module documentation.
pub struct MarkingOverAnImageSaysSoBeforeApply;

impl Check for MarkingOverAnImageSaysSoBeforeApply {
    fn name(&self) -> &'static str {
        "marking_over_an_image_says_so_before_apply"
    }

    fn defect(&self) -> &'static str {
        "a redaction mark that covers a raster image is authored in silence, so the operator \
         does not learn until he opens the saved file that those image samples were destroyed \
         and overwritten rather than hidden — a raster redaction is irreversible in a way a \
         text one is not"
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
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check presses a panel button. Reported as \
             SKIPPED rather than passed: a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    // --- 1: a document that HAS an image ----------------------------------
    let with = mark_whole_page(ctx, &exe, ui_rect, WITH_IMAGE, report)?;
    let Some(line) = with else {
        return Ok(Some(format!(
            "no `{MARKED}` line after pressing Mark whole page on `{WITH_IMAGE}`, so nothing \
             was marked at all and this check never reached its subject."
        )));
    };
    report.note(format!("with an image: `{line}`"));
    if !line.contains(CONSEQUENCE) {
        return Ok(Some(format!(
            "a whole-page mark on `{WITH_IMAGE}` — which is nothing BUT a raster image — was \
             authored without saying that applying it will be refused. The operator finds out \
             at Apply, for the whole document, with no indication of which region caused it. \
             Line: `{line}`."
        )));
    }

    // --- 2: ★★★ and one that does NOT ------------------------------------
    let without = mark_whole_page(ctx, &exe, ui_rect, WITHOUT_IMAGE, report)?;
    let Some(line) = without else {
        return Ok(Some(format!(
            "no `{MARKED}` line after pressing Mark whole page on `{WITHOUT_IMAGE}`."
        )));
    };
    report.note(format!("without an image: `{line}`"));
    if line.contains(CONSEQUENCE) {
        return Ok(Some(format!(
            "a whole-page mark on `{WITHOUT_IMAGE}` — a CAD sheet with NO raster image on it — \
             was warned about anyway. A caveat attached to every mark is one an operator learns \
             to scroll past, and the day it is load-bearing they will scroll past it too. The \
             warning must track the fact, not the feature being switched on. Line: `{line}`."
        )));
    }
    report.note("…and a sheet with no image is marked in silence, which is the half that makes the warning worth reading");

    Ok(None)
}

/// Launch on `fixture`, open the Redact panel, press **Mark whole page**, and
/// return the funnel's line for it.
fn mark_whole_page(
    ctx: &CheckContext,
    exe: &std::path::Path,
    ui_rect: &str,
    fixture: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let path = std::path::Path::new(fixture);
    if !path.is_file() {
        return Err(Error::new(format!(
            "the fixture `{fixture}` is not on disk, so this check cannot establish the \
             document it needs."
        )));
    }
    let mut spec = LaunchSpec::new(exe, ctx.out("redact-image-warning.trace.txt"));
    spec.pdf = Some(path.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    // Maximised, because the Edit tab's later groups fold away at the harness's
    // default width — three checks skipped for a fortnight on exactly that.
    session.maximize();
    session.settle(20);
    let driver = Driver::new(session.window());

    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(14);

    // ★ Only if it is not already there: a panel toggle that is already on
    // CLOSES the thing this check needs.
    if declared(&session.trace()?, ui_rect, PANEL).is_none() {
        let trace = session.trace()?;
        let tab = declared(&trace, ui_rect, "ribbon.tab.edit").ok_or_else(|| {
            Error::new(format!(
                "no `ribbon.tab.edit` region. Tabs declared: {}.",
                list(&declared_names(&trace, ui_rect, "ribbon.tab."))
            ))
        })?;
        driver.click_at(session.frame()?.declared_center(tab))?;
        session.settle(14);
        let item = crate::checks::driving::declared_or_in_overflow(
            &session, &driver, ui_rect, PANEL_ITEM,
        )?
        .ok_or_else(|| {
            Error::new(format!(
                "no `{PANEL_ITEM}` region on the Edit tab or in its overflow. Items declared: \
                 {}.",
                list(&declared_names(
                    &session.trace().unwrap_or_default(),
                    ui_rect,
                    "ribbon.item.edit."
                ))
            ))
        })?;
        driver.click_at(session.frame()?.declared_center(item))?;
        session.settle(24);
    }

    let trace = session.trace()?;
    let button = declared(&trace, ui_rect, WHOLE_PAGE).ok_or_else(|| {
        Error::new(format!(
            "no `{WHOLE_PAGE}` region, so the Redact panel is not on screen or has no Mark \
             whole page control. Regions beginning `redact-`: {}.",
            list(&declared_names(&trace, ui_rect, "redact-"))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(button))?;
    session.settle(30);

    Ok(session.trace()?.last(MARKED).map(|e| e.raw.clone()))
}
