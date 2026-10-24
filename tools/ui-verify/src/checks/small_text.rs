//! `small_text_is_the_caption_size` (O303): a Settings description, drawn in
//! `TextStyle::Small` like every other small text, is the theme's ribbon
//! caption size rather than egui's stock 9 pt.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/small_text.md`.

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,1000";
const OPEN_SETTINGS: &str = "file.settings";
/// The galley size of the Settings window's store-location line.
const SMALL_EVENT: &str = "settings-small-text"; // ui-text-exempt: a trace event name
/// The active theme's `ribbon_caption_pts`.
const CAPTION_EVENT: &str = "theme-caption"; // ui-text-exempt: a trace event name
/// egui's stock `Small` size, which O303 replaced.
const STOCK_SMALL: f32 = 9.0;

/// See the module documentation.
pub struct SmallTextIsTheCaptionSize;

impl Check for SmallTextIsTheCaptionSize {
    fn name(&self) -> &'static str {
        "small_text_is_the_caption_size"
    }

    fn defect(&self) -> &'static str {
        "Settings descriptions and every other small text are drawn at egui's stock 9 pt \
         instead of the ribbon caption size"
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

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<Session> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("small-text.trace.txt"));
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), OPEN_SETTINGS.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(40);
    Ok(session)
}

/// The `pts` field of the last `event` line, if one has been written.
fn pts(session: &Session, event: &str) -> Result<Option<f32>> {
    Ok(session
        .trace()?
        .last(event)
        .and_then(|l| l.get("pts"))
        .and_then(|v| v.parse().ok()))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let session = launch(ctx, report)?;
    let mut small = pts(&session, SMALL_EVENT)?;
    for _ in 0..5 {
        if small.is_some() {
            break;
        }
        session.settle(20);
        small = pts(&session, SMALL_EVENT)?;
    }
    let caption = pts(&session, CAPTION_EVENT)?;
    drop(session);

    let small = small.ok_or_else(|| {
        Error::new(format!(
            "`{OPEN_SETTINGS}` was invoked and no `{SMALL_EVENT}` line was written: the Settings \
             window did not draw its description."
        ))
    })?;
    let caption = caption.ok_or_else(|| {
        Error::new(format!(
            "no `{CAPTION_EVENT}` line: the frame never reported its theme's caption size."
        ))
    })?;
    report.note(format!(
        "Settings description drawn at {small} pt; the theme's caption size is {caption} pt"
    ));
    if (small - caption).abs() > 0.01 {
        return Ok(Some(format!(
            "the Settings description is drawn at {small} pt, not the caption size {caption} pt{}.",
            if (small - STOCK_SMALL).abs() < 0.01 {
                " (egui's stock small size)"
            } else {
                ""
            }
        )));
    }
    if caption <= STOCK_SMALL {
        return Ok(Some(format!(
            "the caption size is {caption} pt, no larger than the stock {STOCK_SMALL} pt."
        )));
    }
    Ok(None)
}
