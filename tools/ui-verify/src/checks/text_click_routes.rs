//! `a_text_tool_click_reaches_what_is_under_it` — with the text tool armed in
//! Edit mode, a click on a scanned page refuses with a Recognise text button
//! that opens the OCR dialog, a click on a form field focuses the field, and a
//! click on a text box comment opens its pop-up; none of the three becomes
//! Add text.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/text_click_routes.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// Edit mode with the Edit Text tool armed, so a click on words opens a caret.
const INVOKE: &str = "mode.edit,edit.text";
const BECAME_ADD: &str = "text-edit-became-add"; // ui-text-exempt: a trace event name, never displayed
const DECLINED: &str = "text-edit-declined"; // ui-text-exempt: a trace event name, never displayed
const ROUTED: &str = "text-click-routed"; // ui-text-exempt: a trace event name, never displayed
const FORM_FOCUS: &str = "form-focus"; // ui-text-exempt: a trace event name, never displayed
const NOTE_TOGGLE: &str = "note-popup-toggle"; // ui-text-exempt: a trace event name, never displayed
const REMEDY: &str = "status-group:decline.remedy"; // ui-text-exempt: a trace region name, never displayed
const OCR_DIALOG: &str = "ocr-dialog"; // ui-text-exempt: a trace region name, never displayed

/// One fixture, the point on page 1 to click, in PDF points, and its tag.
struct Case {
    fixture: &'static str,
    at: (f64, f64),
    tag: &'static str,
}

/// The middle of the scanned page's single full-page image.
const SCAN: Case = Case {
    fixture: "synthetic-image-only.pdf",
    at: (153.0, 198.0),
    tag: "scan",
};
/// Inside `FieldOne`'s widget rectangle.
const FIELD: Case = Case {
    fixture: "three-text-fields.pdf",
    at: (165.0, 237.0),
    tag: "field",
};
/// Inside the FreeText annotation's `/Rect`.
const NOTE: Case = Case {
    fixture: "annots-with-everything.pdf",
    at: (200.0, 420.0),
    tag: "note",
};

/// See the module documentation.
pub struct ATextToolClickReachesWhatIsUnderIt;

impl Check for ATextToolClickReachesWhatIsUnderIt {
    fn name(&self) -> &'static str {
        "a_text_tool_click_reaches_what_is_under_it"
    }

    fn defect(&self) -> &'static str {
        "a text-tool click on a scanned page, a form field or a text box comment silently \
         starts new text there instead of explaining the scan, filling the field or opening \
         the comment"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        for (case, step) in [
            (&SCAN, scan as Step),
            (&FIELD, field as Step),
            (&NOTE, note as Step),
        ] {
            match drive(ctx, &mut report, case, step) {
                Ok(None) => {}
                Ok(Some(failure)) => return report.fail(failure),
                Err(why) => return report.from_error(&why),
            }
        }
        report.pass()
    }
}

type Step = fn(&Drive<'_>, &mut CheckReport) -> Result<Option<String>>;

/// The session, its pointer and what the step needs to read the trace.
struct Drive<'a> {
    session: &'a Session,
    pointer: &'a ScriptedPointer,
    path: String,
    ui_rect: &'static str,
    /// How many `text-edit-became-add` lines preceded the click.
    became_add_before: usize,
}

impl Drive<'_> {
    fn count(&self, event: &str) -> Result<usize> {
        Ok(self.session.trace()?.events(event).count())
    }

    /// The failure when the click turned into Add text, else `None`.
    fn became_add(&self, what: &str) -> Result<Option<String>> {
        if self.count(BECAME_ADD)? > self.became_add_before {
            return Ok(Some(format!(
                "★★★★ a text-tool click on {what} became Add text (`{BECAME_ADD}`). \
                 `canvas::textedit::route::click` is what sends it elsewhere. Trace: {}.",
                self.path
            )));
        }
        Ok(None)
    }
}

fn launch(
    ctx: &CheckContext,
    case: &Case,
) -> Result<(Session, ScriptedPointer, std::path::PathBuf)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let source = repo_fixture(case.fixture, "It is committed under fixtures/.")?;
    let doc = ctx.out(&format!("text-click-{}.pdf", case.tag));
    std::fs::copy(&source, &doc)
        .map_err(|e| Error::new(format!("copying {}: {e}", case.fixture)))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("text-click-{}.trace.txt", case.tag)));
    spec.pdf = Some(doc.clone());
    for (k, v) in [
        ctx.profile.diag_env,
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", INVOKE),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(
        &mut spec,
        ctx.out(&format!("text-click-{}.pointer.txt", case.tag)),
    )?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer, doc))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    case: &Case,
    step: Step,
) -> Result<Option<String>> {
    let (session, pointer, doc) = launch(ctx, case)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let page = crate::fixture::page_geometry(&doc)
        .ok_or_else(|| Error::new(format!("could not read a page size from {}.", case.fixture)))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let d = Drive {
        session: &session,
        pointer: &pointer,
        path: session.trace_path().display().to_string(),
        ui_rect: ctx
            .profile
            .vocab
            .ui_rect_event
            .ok_or_else(|| Error::new("the profile names no ui-rect trace event."))?,
        became_add_before: session.trace()?.events(BECAME_ADD).count(),
    };
    pointer.click(
        &session,
        mapping.doc_to_window(DocPoint::new(0, case.at.0, case.at.1))?,
    )?;
    session.settle(25);
    let outcome = step(&d, report)?;
    pointer.gone(&session)?;
    Ok(outcome)
}

/// ★ A scan refuses as a picture of text, offers Recognise text, and the
/// button opens the OCR dialog.
fn scan(d: &Drive<'_>, report: &mut CheckReport) -> Result<Option<String>> {
    if let Some(f) = d.became_add("a scanned page")? {
        return Ok(Some(f));
    }
    let trace = d.session.trace()?;
    let Some(declined) = trace.events(DECLINED).last() else {
        return Ok(Some(format!(
            "★★★★ a text-tool click on a scanned page said nothing: no `{DECLINED}` line. \
             Trace: {}.",
            d.path
        )));
    };
    if declined.get("reason") != Some("PictureOfText") {
        return Ok(Some(format!(
            "★★★ the scanned page refused for the wrong reason: `{}`; expected \
             `reason=PictureOfText`. Trace: {}.",
            declined.raw, d.path
        )));
    }
    let Some((rect, viewport)) = declared_in(&trace, d.ui_rect, REMEDY) else {
        return Ok(Some(format!(
            "★★★ the picture-of-text refusal drew no Recognise text button (`{REMEDY}`). \
             Trace: {}.",
            d.path
        )));
    };
    d.pointer
        .click_in(d.session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    d.session.settle(30);
    if declared(&d.session.trace()?, d.ui_rect, OCR_DIALOG).is_none() {
        return Ok(Some(format!(
            "★★★ the Recognise text button beside the refusal opened no `{OCR_DIALOG}`. \
             Trace: {}.",
            d.path
        )));
    }
    report.note("a scan refuses as a picture of text; its button opens Recognise text");
    Ok(None)
}

/// ★ A form field takes the click and focuses; no caret opens.
fn field(d: &Drive<'_>, report: &mut CheckReport) -> Result<Option<String>> {
    if let Some(f) = d.became_add("a form field")? {
        return Ok(Some(f));
    }
    let trace = d.session.trace()?;
    let routed = trace.events(ROUTED).any(|l| l.get("to") == Some("field"));
    let focused = trace.events(FORM_FOCUS).next().is_some();
    if !(routed && focused) {
        return Ok(Some(format!(
            "★★★★ a text-tool click on a form field did not reach it: routed to the field \
             {routed}, `{FORM_FOCUS}` {focused}. Trace: {}.",
            d.path
        )));
    }
    report.note("a form field took the text-tool click and focused");
    Ok(None)
}

/// ★ A text box comment opens its pop-up; no caret opens.
fn note(d: &Drive<'_>, report: &mut CheckReport) -> Result<Option<String>> {
    if let Some(f) = d.became_add("a text box comment")? {
        return Ok(Some(f));
    }
    let trace = d.session.trace()?;
    let routed = trace.events(ROUTED).any(|l| l.get("to") == Some("note"));
    let toggled = trace.events(NOTE_TOGGLE).next().is_some();
    if !(routed && toggled) {
        return Ok(Some(format!(
            "★★★★ a text-tool click on a text box comment did not open it: routed to the note \
             {routed}, `{NOTE_TOGGLE}` {toggled}. Trace: {}.",
            d.path
        )));
    }
    report.note("a text box comment took the text-tool click and opened its pop-up");
    Ok(None)
}
