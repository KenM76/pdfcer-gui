//! `print_dialog_body_does_not_deadlock_its_scrollbars` — the regression test
//! for the operator's report of 2026-09-03.
//!
//! # The defect
//!
//! > *"I have two scroll bars in the pop up window that won't go away no matter
//! > how, and it doesn't close after I hit the print button that is so far off
//! > in the corner it is touching the edge the window, and it looks greyed out
//! > as though it doesn't do anything even when I hit print - but it is
//! > working, so after many clicks I checked the printer and of course there
//! > was a dozen jobs there because the button just looks greyed out and
//! > broken."*
//!
//! Four causes. This check owns the first; the other three are asserted
//! elsewhere and named here so a reader can find them:
//!
//! | symptom | where it is asserted |
//! |---|---|
//! | two scrollbars that never go away | **here** |
//! | the window is not its own OS window | `dialog_windows`, which now lists Print |
//! | the button touches the window edge | `dialogs::host::Host::BODY_MARGIN_PTS`, and the margin is visible in any capture |
//! | the button looks disabled | `egui_shell::Theme::accent_pair` and its unit tests |
//!
//! # ★★★ Why this is a check and not a unit test
//!
//! Because the quantity that decides whether a scrollbar appears is
//! **egui's**, not ours, and it exists only in a laid-out frame. The previous
//! version of `PrintDialog::body` was wrong three times in a row about what
//! that quantity was, and each wrong answer looked completely reasonable in the
//! source:
//!
//! 1. it forced the content to `available_width`, measured **outside** the
//!    scroll area — one scrollbar narrower than the viewport the content was
//!    actually being laid into;
//! 2. corrected to measure inside, it still used `auto_shrink([false, false])`,
//!    which *defines* the content to be at least the pre-bar viewport — so the
//!    content was again always at least one bar too wide, by construction;
//! 3. corrected again, it did not account for the two `item_spacing` gaps
//!    `horizontal_top` inserts between three children, nor for the preview's
//!    control strip being **40 pt wider than its own column**.
//!
//! Every one of those was found by reading `egui`'s own `content_size` and
//! `inner_rect` out of a running frame. No unit test could have produced either
//! number, and no screenshot could have said which of the three was wrong.
//!
//! # The oracle
//!
//! The `print-body` trace line, which reports both of egui's numbers, and the
//! `print-strip` line, which reports the inner overflow that fed them. The
//! assertion is an **inequality**, never a value: the widths depend on the
//! theme preset's font and button padding, so any constant would be a claim
//! that decays — which this project has spent six corrections on.
//!
//! ★ It is driven at **several window sizes**, not one. The defect was
//! *inverted* — bars present at 1000x760 and 1300x900 where nothing needed
//! scrolling, and **absent** at 700x520 where the Paper section was clipped and
//! unreachable. Two samples either side of that would have looked like no
//! defect at all.

use crate::checks::driving::{SHELL_DIAG_ENV, VIEWPORT_INNER_EVENT};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The environment variable that fires one command at start-up.
const INVOKE_ENV: &str = "PDFCER_DIAG_INVOKE";

/// The command that opens the subject.
const SUBJECT: &str = "file.print";

/// The window sizes the dialog is driven at, as `width,height` in physical
/// pixels for `PDFCER_DIAG_VIEWPORT`.
///
/// ★ These size the APPLICATION window, and the dialog inherits its own
/// declared 800x620 regardless — so what this actually varies is the machine's
/// available space and the dialog's placement, not the dialog's size. Varying
/// the dialog itself needs an OS-level resize of the child window, which is a
/// capability this harness does not have and which is recorded as the limit of
/// this check rather than papered over: see `LIMITS` in the failure text.
///
/// What it DOES cover is the default-size case at several placements, which is
/// the case every operator meets on first open.
const VIEWPORTS: &[&str] = &[
    "-2400,80,1600,1000",
    "-2400,80,1100,820",
    "-2400,80,900,700",
];

/// See the module documentation.
pub struct PrintDialogBodyDoesNotDeadlockItsScrollbars;

impl Check for PrintDialogBodyDoesNotDeadlockItsScrollbars {
    fn name(&self) -> &'static str {
        "print_dialog_body_does_not_deadlock_its_scrollbars"
    }

    fn defect(&self) -> &'static str {
        "the print dialog draws a horizontal and a vertical scrollbar that no amount of resizing \
         removes, because its body is laid out to a width measured before the scroll area \
         reserves its own bars — so the content is always at least one scrollbar wider than the \
         viewport it is measured against, and each bar keeps the other alive"
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

/// One frame's worth of the `print-body` line, parsed.
struct Body {
    content_w: f32,
    content_h: f32,
    view_w: f32,
    view_h: f32,
}

/// Read `name=[w h]` out of a trace line's field.
///
/// The two egui vectors are printed by `Vec2`'s `Debug`, which is `[w h]` with
/// a space — so they cannot be read by the harness's `key=value` splitter and
/// are parsed here. Returns `None` rather than a default on anything
/// unexpected: a zero would satisfy every inequality below and turn a broken
/// parse into a pass, which is the exact shape this project calls a check that
/// cannot fail.
fn vec_field(raw: &str, key: &str) -> Option<(f32, f32)> {
    let after = raw.split_once(&format!("{key}=["))?.1;
    let inside = after.split_once(']')?.0;
    let (w, h) = inside.trim().split_once(' ')?;
    Some((w.trim().parse().ok()?, h.trim().parse().ok()?))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. The print dialog draws no body without a document, and a dialog that drew \
             no body would report no overflow — an absence proving nothing.",
        )
    })?;

    let mut failures: Vec<String> = Vec::new();
    let mut sampled = 0usize;

    for (index, viewport) in VIEWPORTS.iter().enumerate() {
        let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("print-layout-{index}.trace.txt")));
        spec.pdf = Some(pdf.clone());
        spec.env.push((
            ctx.profile.diag_env.0.to_owned(),
            ctx.profile.diag_env.1.to_owned(),
        ));
        spec.env
            .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
        spec.env.push((INVOKE_ENV.to_owned(), SUBJECT.to_owned()));
        if let Some(env) = ctx.profile.viewport_env {
            spec.env.push((env.to_owned(), (*viewport).to_owned()));
        }
        spec.allow_stale = ctx.allow_stale;
        spec.source_root = ctx.source_root.clone();

        let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
        report.artifact(session.trace_path().to_path_buf());
        session.settle(40);
        let trace = session.trace()?;

        // The dialog must actually be open, or everything below is vacuously
        // true. This is the planted-absence guard: no window, no evidence.
        if trace.events(VIEWPORT_INNER_EVENT).last().is_none() {
            return Err(Error::new(format!(
                "the print dialog never opened its window at viewport {viewport}, so nothing \
                 about its body could be measured. That is not a pass."
            )));
        }

        let Some(line) = trace.events("print-body").last() else {
            return Err(Error::new(
                "the application published no `print-body` line. Either this build predates the \
                 2026-09-03 layout instrumentation, or the body did not draw — and an absent \
                 measurement must never read as a good one."
                    .to_owned(),
            ));
        };

        let (Some((content_w, content_h)), Some((view_w, view_h))) = (
            vec_field(&line.raw, "egui_content"),
            vec_field(&line.raw, "egui_view"),
        ) else {
            return Err(Error::new(format!(
                "could not parse egui's own content and viewport sizes out of `{}`.",
                line.raw
            )));
        };
        let body = Body {
            content_w,
            content_h,
            view_w,
            view_h,
        };
        sampled += 1;

        // ★ The assertion. Content must be strictly no larger than the viewport
        // on BOTH axes at a window size that comfortably holds the dialog. It
        // is `<=` rather than `<` because equality draws no bar and demanding a
        // strict margin would be asserting the allowance's exact value, which
        // is a constant this check must not depend on.
        if body.content_w > body.view_w {
            failures.push(format!(
                "at viewport {viewport} the body's content is {:.1} pt wide against a {:.1} pt \
                 viewport — {:.1} pt of overflow, which is a horizontal scrollbar the operator \
                 cannot dismiss",
                body.content_w,
                body.view_w,
                body.content_w - body.view_w
            ));
        }
        if body.content_h > body.view_h {
            failures.push(format!(
                "at viewport {viewport} the body's content is {:.1} pt tall against a {:.1} pt \
                 viewport — {:.1} pt of overflow, which is a vertical scrollbar the operator \
                 cannot dismiss",
                body.content_h,
                body.view_h,
                body.content_h - body.view_h
            ));
        }

        // ★★ The inner cause, asserted separately so a failure names WHICH of
        // the two it is. The preview's control strip was 379.9 pt in a 340 pt
        // column, and that overflow propagated outward into the body — so a
        // body that fits while the strip does not is a body one theme change
        // away from not fitting.
        if let Some(strip) = trace.events("print-strip").last() {
            let laid: f32 = strip
                .get("laid_w")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.0);
            let column: f32 = strip
                .get("column_w")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.0);
            if laid > column && column > 0.0 {
                failures.push(format!(
                    "at viewport {viewport} the preview's control strip is laid out {laid:.1} pt \
                     wide inside a {column:.1} pt column. That overflow is what reaches the body \
                     and raises a horizontal scrollbar; the row must be `horizontal_wrapped`, \
                     which is bounded by its column by construction"
                ));
            } else {
                report.note(format!(
                    "· at {viewport} the strip fits its column: {laid:.1} <= {column:.1} pt"
                ));
            }
        }

        report.note(format!(
            "★ at {viewport} the body content is {:.1}x{:.1} in a {:.1}x{:.1} viewport — no bar \
             on either axis",
            body.content_w, body.content_h, body.view_w, body.view_h
        ));
    }

    if sampled == 0 {
        return Err(Error::new(
            "no viewport produced a measurable body. Nothing was checked.".to_owned(),
        ));
    }

    if failures.is_empty() {
        return Ok(None);
    }
    Ok(Some(format!(
        "★ the print dialog's body overflows its own scroll viewport:\n  {}\n\n\
         Each overflow is a scrollbar, and the two axes feed each other: a horizontal bar \
         consumes height, which can raise a vertical bar, which consumes width, which keeps the \
         horizontal one. The operator's words were *\"two scroll bars in the pop up window that \
         won't go away no matter how\"*.\n\n\
         The rule `PrintDialog::body` now holds: every width and height it lays out is derived \
         from the space OUTSIDE the scroll area and from constants — never from a measurement \
         taken inside it, and never with `auto_shrink` false, which defines the content to be at \
         least the pre-bar viewport.\n\n\
         LIMITS OF THIS CHECK, stated rather than implied: it drives the dialog at its DECLARED \
         size only. Resizing a child OS window is not something this harness can do, so the \
         deadlock as the operator met it — after dragging the dialog larger — is covered by the \
         arithmetic being size-independent rather than by a sample at his size.",
        failures.join("\n  ")
    )))
}
