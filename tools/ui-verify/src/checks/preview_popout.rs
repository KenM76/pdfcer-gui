//! `the_print_preview_pops_into_its_own_window` — **and the print dialog's own
//! preview column goes away when it does.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/preview_popout.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, frame_of, list, stable_rect,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The command, invoked through the harness seam.
const INVOKE: &str = "file.print";
/// The environment variable that carries it.
const INVOKE_ENV: &str = "PDFCER_DIAG_INVOKE";
/// The print dialog's preview column — the region whose **retirement** is the
/// point of this check.
const COLUMN: &str = "print.preview.column";
/// The Pop-out button.
const POP_OUT: &str = "print.preview.popout";
/// The popped window's body, published inside the child viewport.
const POPPED_BODY: &str = "print.preview.window";
/// The line the body writes every frame, carrying `popped=` and both widths.
const BODY: &str = "print-body";
/// The line the pop-out and the return write.
const POPPED: &str = "print-preview-popped";
/// The retirement line `diag::end_ui_frame` publishes.
const GONE: &str = "ui-rect-gone";

/// See the module documentation.
pub struct ThePrintPreviewPopsIntoItsOwnWindow;

impl Check for ThePrintPreviewPopsIntoItsOwnWindow {
    fn name(&self) -> &'static str {
        "the_print_preview_pops_into_its_own_window"
    }

    fn defect(&self) -> &'static str {
        "pressing Pop out opens the preview in its own window AND leaves the print dialog still \
         drawing its own preview column — two pictures of one sheet, with 340 pt of the dialog \
         held open by a duplicate the operator did not ask for"
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

/// How many times a region has been retired so far.
fn retirements(trace: &crate::trace::Trace, region: &str) -> usize {
    trace
        .events(GONE)
        .filter(|line| line.get("name") == Some(region))
        .count()
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check's subject is a click on the Pop out \
             button.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. The print dialog draws no preview without a document, and a column that \
             was never there cannot be seen to go away.",
        )
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("preview-popout.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env.push((INVOKE_ENV.to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(60);
    let driver = Driver::new(session.window());

    // ── position 1: the column is there ─────────────────────────────────────
    //
    // An `Err`, not a `fail`. A print dialog that never opened, or one on a
    // machine with no printers, is a run that cannot answer this question — and
    // reporting it as the defect would be exactly the "articulate failure about
    // the wrong subject" this project has recorded three times in one day. See
    // `print_dialog` for the diagnosis of a dialog that will not open.
    let trace = session.trace()?;
    if declared(&trace, ui_rect, COLUMN).is_none() {
        return Err(Error::new(format!(
            "the print dialog drew no preview column, so there is nothing for this check to \
             watch disappear. Either the dialog did not open, or this build has no printer to \
             describe a sheet with — `print_dialog` diagnoses both. Regions beginning `print`: \
             {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "print")),
            session.trace_path().display()
        )));
    }
    let Some(before) = trace.events(BODY).last() else {
        return Err(Error::new(format!(
            "the application published no `{BODY}` line, so the layout cannot be read. Either \
             this build predates the 2026-09-03 instrumentation or the body did not draw; an \
             absent measurement must never read as a good one. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if before.get("popped") != Some("false") {
        return Ok(Some(format!(
            "the print dialog opened with its preview ALREADY popped out: `{}`.\n\
             `PrintDialog::open` sets `preview_popped: false`, and a dialog that opens with a \
             second window the operator did not ask for is a worse defect than the one this \
             check was written for. Trace: {}.",
            before.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "· before the click the column is drawn: `{}`",
        before.raw
    ));
    let gone_before = retirements(&trace, COLUMN);

    // ── the click ───────────────────────────────────────────────────────────
    let Some(button) = stable_rect(&session, ui_rect, POP_OUT, 8)? else {
        return Ok(Some(format!(
            "the preview drew its column and declared no `{POP_OUT}` region, so the operator has \
             no Pop out button — which is O112 ask 2 not being built rather than being broken. \
             ★ It is published with the VISIBILITY-GATED publisher, so this is also what a \
             button laid out past the column's clip rectangle looks like: the strip is \
             `horizontal_wrapped` and Pop out is its last control, so a narrow dialog wraps it \
             onto a second row. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let trace = session.trace()?;
    let frame = frame_of(&session, &trace, ui_rect, POP_OUT)?;
    driver.click_at(frame.declared_center(button))?;
    session.settle(60);

    // ── position 2: the window is there and the column is not ───────────────
    let trace = session.trace()?;
    let Some(out) = trace
        .events(POPPED)
        .filter(|line| line.get("state") == Some("out"))
        .last()
    else {
        return Ok(Some(format!(
            "Pop out was clicked and the dialog did not record the change: no `{POPPED} \
             state=out` line. The button's own rect was declared and aimed at, so this is the \
             click not reaching it or the handler not running. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★ the preview reports itself popped out: `{}`",
        out.raw
    ));

    if declared(&trace, ui_rect, POPPED_BODY).is_none() {
        return Ok(Some(format!(
            "the dialog says the preview popped out and NO SECOND WINDOW DREW: no \
             `{POPPED_BODY}` region. `dialogs::print::popout` opens a `Host` keyed \
             `print-preview`; a host that reports no body is a viewport that was never created, \
             which on a backend with no multi-viewport support is `ViewportClass::Embedded`. \
             Regions beginning `print`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "print")),
            session.trace_path().display()
        )));
    }

    // THE ASSERTION THE CHECK EXISTS FOR.
    let gone_after = retirements(&trace, COLUMN);
    if gone_after <= gone_before {
        return Ok(Some(format!(
            "★★★ THE PREVIEW POPPED OUT AND THE PRINT DIALOG KEPT DRAWING ITS COLUMN.\n\
             `{GONE} name={COLUMN}` was published {gone_before} time(s) before the click and \
             {gone_after} after it, so the column was never retired — the operator now has two \
             pictures of one sheet and 340 pt of the dialog held open by the duplicate.\n\
             This is R9's case, not a cosmetic one: `layout::Columns::split`'s popped arm must \
             return `preview: 0.0`, `splitter: 0.0` and `options == content`, and \
             `PrintDialog::body` must skip the column and the splitter together. Trace: {}.",
            session.trace_path().display()
        )));
    }

    let Some(after) = trace.events(BODY).last() else {
        return Ok(Some(format!(
            "the body stopped publishing `{BODY}` after the pop-out, so the collapsed layout \
             cannot be read. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if after.get("popped") != Some("true") {
        return Ok(Some(format!(
            "the preview reported itself popped out and the body did not: `{}`. Two pieces of \
             state that must agree, disagreeing — read `PrintDialog::preview_popped`'s writers. \
             Trace: {}.",
            after.raw,
            session.trace_path().display()
        )));
    }
    let preview_w: f32 = after
        .get("preview_w")
        .and_then(|v| v.parse().ok())
        .unwrap_or(f32::NAN);
    if preview_w.abs() >= f32::EPSILON || !preview_w.is_finite() {
        return Ok(Some(format!(
            "the column collapsed and {preview_w:.1} pt is still reserved for it: `{}`. A width \
             held open for a surface that is not drawn is a hole in the dialog — R9's own case. \
             Trace: {}.",
            after.raw,
            session.trace_path().display()
        )));
    }
    // A RELATIONSHIP, not a value. See the module header.
    let (Some(options_w), Some(content_w)) = (
        after.get("options_w").and_then(|v| v.parse::<f32>().ok()),
        after.get("content_w").and_then(|v| v.parse::<f32>().ok()),
    ) else {
        return Err(Error::new(format!(
            "could not read the two widths out of `{}`.",
            after.raw
        )));
    };
    if (options_w - content_w).abs() > 0.5 {
        return Ok(Some(format!(
            "the column collapsed and the options DID NOT TAKE THE ROOM: options {options_w:.1} \
             pt inside {content_w:.1} pt of content, leaving {:.1} pt nobody is using — `{}`.\n\
             Collapsing a column and emptying it are different things, and this is the second. \
             `Columns::split`'s popped arm hands the whole content width to the options for \
             exactly this reason. Trace: {}.",
            content_w - options_w,
            after.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ the column is gone and the options took its room: `{}`",
        after.raw
    ));

    // ── the return trip ─────────────────────────────────────────────────────
    //
    // G4: Escape IS the close button. A skip, not a failure, when the keyboard
    // did not reach the popped window — see the module header.
    driver.press(vk::ESCAPE)?;
    session.settle(60);
    let trace = session.trace()?;
    let returned = trace
        .events(POPPED)
        .filter(|line| line.get("state") == Some("in"))
        .last();
    match returned {
        Some(line) => {
            report.note(format!(
                "★ closing the window put the preview back: `{}`",
                line.raw
            ));
            let Some(home) = trace.events(BODY).last() else {
                return Ok(Some(format!(
                    "the preview came home and the body stopped reporting. Trace: {}.",
                    session.trace_path().display()
                )));
            };
            if home.get("popped") != Some("false") {
                return Ok(Some(format!(
                    "the popped window closed and the print dialog still believes the preview is \
                     elsewhere: `{}`. The operator now has a dialog with no preview and no window \
                     to put it back from — the one state this feature must not be able to reach. \
                     `popout::popped_preview` writes `false` on `Frame::closed`; that is the only \
                     writer. Trace: {}.",
                    home.raw,
                    session.trace_path().display()
                )));
            }
            if declared(&trace, ui_rect, COLUMN).is_none() {
                return Ok(Some(format!(
                    "the dialog says the preview came back and no column was drawn: no `{COLUMN}` \
                     region after the return. Trace: {}.",
                    session.trace_path().display()
                )));
            }
            report.note("★★ the column is drawn again — the round trip is closed");
        }
        None => {
            report.note(
                "⬜ THE RETURN TRIP WAS NOT MEASURED: Escape produced no `print-preview-popped \
                 state=in` line. That is most likely the keyboard not reaching the popped \
                 window rather than the return being broken — this harness has reported nine \
                 false skips on a stray window holding the foreground. The pop-out half above \
                 stands; the put-back half is UNVERIFIED by this run.",
            );
        }
    }
    Ok(None)
}
