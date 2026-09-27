//! `a_typed_password_is_not_saved_unless_asked` — filling a password field
//! stores nothing and says so, and the Forms panel's button stores it on request.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/password_fill.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::forms_spotlight::open_from_tab;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The mode the canvas fill is driven in.
const MODE: &str = "read";
/// The mode the Forms panel is reachable from.
const PANEL_MODE: &str = "review";
/// The ribbon item that opens the Forms panel.
const PANEL_ITEM: &str = "ribbon.item.view.panel_forms";
/// The Forms panel's dock body.
const PANEL_BODY: &str = "dock.body.view.panel_forms";
/// The panel's *save it anyway* button.
const STORE_REGION: &str = "forms.fill.store_password";

/// `form-box page=… field=… widget=… kind=… rect=(x,y)+(w,h)`.
const BOX_LINE: &str = "form-box";
/// `form-focus page=… field=… widget=…`.
const FOCUS_LINE: &str = "form-focus";
/// `form-fill-text commands=… … password_withheld=…`.
const FILL_LINE: &str = "form-fill-text";
/// The same line for the verb that stores the plaintext.
const STORE_LINE: &str = "form-fill-storing-password";
/// The status bar's fill-disclosure group.
const BAR_REGION: &str = "status-group:fill-disclosure";

/// `Enter` as a Windows virtual key.
const VK_RETURN: u16 = 0x0D;

/// The value typed in.
const TYPED: &str = "4711";

/// The fixture: one text field with the Password flag and no stored value.
const FIXTURE: &str = "password-field.pdf";

/// See the module documentation.
pub struct ATypedPasswordIsNotSavedUnlessAsked;

impl Check for ATypedPasswordIsNotSavedUnlessAsked {
    fn name(&self) -> &'static str {
        "a_typed_password_is_not_saved_unless_asked"
    }

    fn defect(&self) -> &'static str {
        "a password typed into a form field either lands in the saved file as plain text, or is \
         silently dropped with the box reading empty and nothing saying why, or cannot be saved \
         even when the operator asks for it"
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

/// The first `form-box` line's page, field and canvas-space centre.
fn first_box(trace: &Trace) -> Option<(usize, String, (f64, f64))> {
    trace.events(BOX_LINE).find_map(|l| {
        let page: usize = l.get("page")?.parse().ok()?;
        let field = l.get("field")?.to_owned();
        let (min, size) = l.get("rect")?.split_once(")+(")?;
        let (x, y) = min.trim_start_matches('(').split_once(',')?;
        let (w, h) = size.trim_end_matches(')').split_once(',')?;
        let x: f64 = x.trim().parse().ok()?;
        let y: f64 = y.trim().parse().ok()?;
        let w: f64 = w.trim().parse().ok()?;
        let h: f64 = h.trim().parse().ok()?;
        Some((page, field, (x + w / 2.0, y + h / 2.0)))
    })
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    // `--pdf` is ignored: the check needs a password field, which only this
    // fixture has.
    let pdf = crate::fixture::workspace_root()
        .join("fixtures")
        .join(FIXTURE);
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the fixture is not at {}. It is committed with its .PROVENANCE.py, so an absence \
             is a broken checkout.",
            pdf.display()
        )));
    }
    report.note(format!(
        "--pdf is IGNORED: this check pins {}",
        pdf.display()
    ));
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a field and types into it.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    let page: PageGeometry = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new(format!("cannot read a page size from {}.", pdf.display())))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("password_fill.trace.txt"));
    spec.pdf = Some(pdf);
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
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(40);

    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    // --- Fill the field on the canvas ---------------------------------------
    let trace = session.trace()?;
    let Some((box_page, field, (cx, cy))) = first_box(&trace) else {
        return Ok(Some(format!(
            "no `{BOX_LINE}` line: the application drew no fillable widget on a document with \
             one. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "the application drew field {field:?} on page {box_page}"
    ));
    // Census is canvas space (y down); the mapping takes PDF space (y up).
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, box_page)?;
    let point = mapping.doc_to_window(DocPoint::new(box_page, cx, page.height_pt - cy))?;
    driver.click_at(session.frame()?.to_screen(point))?;
    session.settle(25);
    if session.trace()?.last(FOCUS_LINE).is_none() {
        return Ok(Some(format!(
            "the click on the field placed no caret: no `{FOCUS_LINE}` line. Trace: {}.",
            session.trace_path().display()
        )));
    }

    driver.type_ascii(TYPED)?;
    session.settle(20);
    let mark = session.trace()?.mark();
    driver.press(VK_RETURN)?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(fill) = trace.last_after(FILL_LINE, mark) else {
        return Ok(Some(format!(
            "the commit wrote nothing: no `{FILL_LINE}` line after the Enter. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the commit applied: `{}`", fill.raw));
    if fill.get("password_withheld") != Some("true") {
        return Ok(Some(format!(
            "the fill of a password field did not report `password_withheld=true`: `{}`. \
             Either the engine stored the typed value in the file as plain text, or this shell \
             dropped the engine's report.",
            fill.raw
        )));
    }
    let drawn = trace
        .events(ui_rect)
        .any(|l| l.lineno > mark && l.get("name") == Some(BAR_REGION));
    if !drawn {
        return Ok(Some(format!(
            "the engine withheld the password and the status bar said nothing: no \
             `{BAR_REGION}` region after the fill. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("the status bar said the password was not saved");

    // --- Save it anyway, from the Forms panel -------------------------------
    click_mode_segment(&session, &driver, ui_rect, PANEL_MODE)?;
    session.settle(20);
    if declared(&session.trace()?, ui_rect, PANEL_BODY).is_none() {
        open_from_tab(&session, &driver, ui_rect, "view", PANEL_ITEM)?;
        session.settle(24);
    }
    let trace = session.trace()?;
    let Some(button) = declared(&trace, ui_rect, STORE_REGION) else {
        return Ok(Some(format!(
            "the Forms panel offers no way to save the withheld password: no `{STORE_REGION}` \
             region. Regions beginning `forms.`: {}.",
            list(&declared_names(&trace, ui_rect, "forms."))
        )));
    };
    let mark = trace.mark();
    driver.click_at(session.frame()?.declared_center(button))?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(store) = trace.last_after(STORE_LINE, mark) else {
        return Ok(Some(format!(
            "the save-it-anyway button was clicked and no `{STORE_LINE}` line followed, so the \
             storing verb never ran. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the storing fill applied: `{}`", store.raw));
    if store.get("commands") != Some("1") || store.get("password_withheld") != Some("false") {
        return Ok(Some(format!(
            "the storing fill ran but did not store: `{}`. Expected one command and \
             `password_withheld=false`.",
            store.raw
        )));
    }
    Ok(None)
}
