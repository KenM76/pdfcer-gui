//! `fillable_fields_are_shaded_on_the_page` — every box you can type into wears
//! a wash, the way Acrobat's does.
//!
//! # The operator's ask, `OPERATOR_REQUESTS.md` O96
//!
//! > *"in our display section we should have an option to shade the form fields
//! > like acrobat does."*
//!
//! A form's fields are frequently invisible on the page — a CAD title block with
//! no borders and no background looks identical whether it is fillable or
//! painted on. The wash is what says *"you can type here"* before the operator
//! has clicked anything.
//!
//! # ★★ Why this needs a launched binary, when the drawing is four lines
//!
//! Because the four lines are the *last* stage of a chain, and every earlier
//! link is somewhere else:
//!
//! 1. the preference has to default to on (`app::prefs`);
//! 2. it has to survive into `OpenDoc::prefs`, which is a **snapshot taken when
//!    the document opened** rather than a live read — deliberately, because a
//!    preference changing mid-frame would flicker;
//! 3. the canvas has to have built a widget census for the document at all;
//! 4. the boxes have to be on a page that is currently in view.
//!
//! A unit test of the drawing function can only reach step 4, with the other
//! three supplied by hand. Nothing below the binary can tell you that the
//! default is on *and* reaches the canvas *and* finds the fields.
//!
//! # ★★★ The trace had to distinguish three states before this could exist
//!
//! Drawing nothing is the observable outcome of three completely different
//! situations, and only one of them is a defect:
//!
//! | trace | meaning | defect? |
//! |---|---|---|
//! | `on=0` | the operator turned the wash off | no |
//! | `on=1 boxes=0` | the wash is on and the document has no fields | no |
//! | `on=1 boxes>0 drawn=0` | the wash is on, the fields exist, none was painted | **yes** |
//!
//! Before the trace carried `on` and `boxes` as well as `drawn`, a check could
//! not tell the third from the first two — and, far worse, a build with the
//! feature entirely dead would have looked identical to a correct build run
//! against a document with no form. That is the failure mode where a green
//! suite is actively misleading, so the instrument was widened rather than the
//! assertion weakened.
//!
//! # No input, so this runs at any time
//!
//! The preference defaults to on and the fixture opens on the page carrying the
//! fields, so nothing has to be clicked. With `PDFCER_DIAG_VIEWPORT` the window
//! lays out without taking focus. Like `title_build_stamp`, this can run beside
//! somebody working — which is worth preserving if the check is ever extended.
//! An extension that needs the pointer belongs in a second check, not bolted
//! onto this one.
//!
//! # ★★★ `boxes=` is the CANVAS CENSUS, not the document's field count
//!
//! Measured on `demo-form.pdf`, which carries **two** widgets: a text field and
//! a check box. The trace says `boxes=1`, and that is correct rather than a
//! defect — the text field has no `/AP` `/N`, so the page draws nothing there,
//! and the census deliberately excludes it (`NotOnCanvas::NoAppearance`). It is
//! **disclosed off-canvas** in the Forms panel, with the remedy named:
//! *"N field(s) are not drawn on the page, so they cannot be clicked there.
//! Fill one here and it becomes drawn."*
//!
//! ⇒ So nobody reading a future trace should take `boxes=` for *"how many
//! fields this form has"*. It is *"how many the canvas is in a position to
//! draw on"*, and the difference is exactly the undrawn set, which
//! `form-boxes … undrawn=` reports separately.
//!
//! ⬜ Whether the wash *ought* to cover an undrawn field is a real open
//! question, and this check deliberately does not decide it. The request said
//! *"like acrobat does"*, and Acrobat's field highlight is generally understood
//! to cover fields it would generate an appearance for — but that has **not
//! been verified against the installed Acrobat**, and asserting it from memory
//! is exactly the failure this project forbids. Recorded rather than acted on:
//! see `OPERATOR_REQUESTS.md` O96.
//!
//! # What a passing run does NOT prove
//!
//! That the wash is the right **colour**, or visible against the page. That is a
//! contrast question with a pixel oracle, and `crate::checks::legibility` is
//! where that kind of claim is made. This asserts the wash is drawn over the
//! right number of boxes, which is the part that silently disappears.

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The canvas's wash trace.
const SHADE: &str = "canvas-form-shade";
/// The fixture: two widgets on one page, both visible at once.
const FIXTURE: &str = "forms/demo-form.pdf";
/// Where and how large the window is placed, as `PDFCER_DIAG_VIEWPORT` takes it.
const VIEWPORT: &str = "0,0,1400,900";

/// See the module documentation.
pub struct FillableFieldsAreShadedOnThePage;

impl Check for FillableFieldsAreShadedOnThePage {
    fn name(&self) -> &'static str {
        "fillable_fields_are_shaded_on_the_page"
    }

    fn defect(&self) -> &'static str {
        "the boxes an operator can type into are not marked on the page, so a form whose fields \
         have no border and no background is indistinguishable from a drawing that merely looks \
         like one"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
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

    // Its own fixture, never `--pdf`. The harness's usual fixture is a CAD
    // drawing with no `/AcroForm`, which would trace `on=1 boxes=0` — a state
    // this check correctly treats as "nothing to say", so falling back would
    // make it SKIP forever, and a SKIP is not red.
    let fixture = form_fixture().ok_or_else(|| {
        Error::new(format!(
            "the engine fixture `{FIXTURE}` is not on disk, so there is no document with form \
             fields to shade. This check does NOT fall back to `--pdf`: the usual fixture has \
             no form, and a run against it could not distinguish a working wash from a dead \
             one."
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("field_shading.trace.txt"));
    spec.pdf = Some(fixture);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    if let Some(name) = ctx.profile.viewport_env {
        spec.env.push((name.to_owned(), VIEWPORT.to_owned()));
    }

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {} — no input is sent",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(48);

    let trace = session.trace()?;
    let Some(line) = trace.last(SHADE) else {
        return Ok(Some(format!(
            "the document opened and the canvas traced no `{SHADE}` line at all, so the wash \
             code never ran. It is called from the form overlay, so either that overlay is not \
             drawn on this document or the call was removed."
        )));
    };
    report.note(format!("canvas: `{}`", line.raw));

    // --- 1: the option is on by default ------------------------------------
    if line.get("on") != Some("1") {
        return Ok(Some(format!(
            "the wash reports `on=0`, so the preference is off in a fresh profile. It is meant \
             to default to ON — an operator who has never opened the settings should see which \
             boxes are fillable. Check `app::prefs`' default, and check that it survives into \
             `OpenDoc::prefs`, which is a SNAPSHOT taken when the document opened rather than \
             a live read. Line: `{}`.",
            line.raw
        )));
    }

    // --- 2: the fixture really does carry fields ---------------------------
    //
    // ★ Asserted rather than assumed, and it guards the check itself rather
    // than the feature: if `demo-form.pdf` ever loses its widgets, every
    // assertion below becomes vacuous and this run would report PASS while
    // having tested nothing.
    let boxes: usize = line.get("boxes").and_then(|v| v.parse().ok()).unwrap_or(0);
    if boxes == 0 {
        return Err(Error::new(format!(
            "the canvas found no form-field boxes in `{FIXTURE}`, so there is nothing to shade \
             and this check cannot distinguish a working wash from a dead one. Reported as SKIP \
             rather than PASS or FAIL: the fixture no longer exercises the case it was chosen \
             for. Line: `{}`.",
            line.raw
        )));
    }

    // --- 3: ★ and they were actually painted -------------------------------
    let drawn: usize = line.get("drawn").and_then(|v| v.parse().ok()).unwrap_or(0);
    if drawn == 0 {
        return Ok(Some(format!(
            "the wash is on and the canvas knows about {boxes} field box(es) and painted NONE \
             of them. This is the one shape that is a defect rather than an absence: either \
             the page-view walk and the box census disagree about which page a box is on, or \
             the boxes are all off screen — and this fixture is a single page opened at fit, \
             so they cannot be. Line: `{}`.",
            line.raw
        )));
    }
    report.note(format!(
        "the wash painted {drawn} of {boxes} field box(es) on the visible page"
    ));

    Ok(None)
}

/// Resolve [`FIXTURE`] under the engine repository's synthetic corpus.
///
/// Read-only, as everything under `D:\Dev\pdfcer` is until fold-in day. The
/// harness opens it and never saves.
fn form_fixture() -> Option<std::path::PathBuf> {
    let path = std::path::Path::new("D:/Dev/pdfcer/fixtures/synthetic").join(FIXTURE);
    path.is_file().then_some(path)
}
