//! **The graphics-pressure instrument is read by something** — the dial in
//! front of O219 and O221, which until now nobody stood in front of.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/graphics_pressure.md`.

use std::path::{Path, PathBuf};

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The document this check opens.
const FIXTURE: &str = "four-pages.pdf";

/// The precondition. A launch that opened nothing traces no `status` line, and
/// every document-dependent reading below would then be legitimately absent.
const STATUS_LINE: &str = "status";

/// The device's real single-axis texture limit, as `side=N`.
const TEXTURE_LIMIT: &str = "gl-max-texture-side";

/// The per-frame drained-error line. Read, reported, never asserted — see the
/// module header.
const PRESSURE: &str = "gl-pressure";

/// `egui`'s `InputState::default().max_texture_side`.
const EGUI_DEFAULT_SIDE: usize = 2048;

/// The floor the standing reading must clear.
const CREDIBLE_FLOOR: usize = 4096;

/// Off the desktop, at the harness's usual size.
const OFFSCREEN: &str = "-4200,-4200,1400,900";

/// See the module documentation.
pub struct TheGraphicsPressureInstrumentReportsARealDeviceLimit;

impl Check for TheGraphicsPressureInstrumentReportsARealDeviceLimit {
    fn name(&self) -> &'static str {
        "the_graphics_pressure_instrument_reports_a_real_device_limit"
    }

    fn defect(&self) -> &'static str {
        "the instrument written for the operator's blank-page report is not wired to the \
         device. It reports `egui`'s pre-backend default of 2048 as though it were a texture \
         limit — a plausible-looking number belonging to no card — so every reading taken from \
         it, and every ceiling that might later be derived from one, is about nothing. Or the \
         instrument is not running at all, in which case the blank page stays unmeasured while \
         the row that owns it reads as instrumented"
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
    let exe = resolve_exe(ctx)?;
    let session = launch_quiet(ctx, &exe, FIXTURE, "graphics_pressure.trace.txt")?;
    report.note(format!(
        "launched {} on fixtures/{FIXTURE} as pid {} — no input is sent and the window is \
         placed off the desktop, so this check does not compete for the machine",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(60);

    let trace = session.trace()?;

    // --- the precondition ---------------------------------------------------
    //
    // Before the presence or absence of a device reading means anything, the
    // application has to have got as far as a document. A mistyped or
    // relative path traces no `status` line at all, and this project has twice
    // produced confident, detailed, entirely wrong defect reports out of
    // exactly that shape.
    if trace.last(STATUS_LINE).is_none() {
        return Err(Error::new(format!(
            "the application launched and never traced a `{STATUS_LINE}` line, so it opened no \
             document and very likely never reached a painted frame. Any verdict below would be \
             about an empty shell. Check that fixtures/{FIXTURE} exists and that an absolute \
             path was passed."
        )));
    }

    // --- 1: the instrument is alive ----------------------------------------
    let sides: Vec<usize> = trace
        .events(TEXTURE_LIMIT)
        .filter_map(|line| line.get_usize("side"))
        .collect();

    if sides.is_empty() {
        let any = trace.events(TEXTURE_LIMIT).count();
        return Ok(Some(format!(
            "the document opened and the build traced {any} `{TEXTURE_LIMIT}` line(s), not one of \
             them carrying a readable `side=` figure. The instrument \
             `render::pressure::trace_texture_limit` \
             is either not called from the frame loop or is not reaching the diagnostic channel, \
             so the device's texture limit is unmeasured — and so is everything O219 and O221 \
             need it for. ★ Check that the call site in `app::frame` survives, and that the \
             diagnostic channel is on in this build."
        )));
    }

    // --- 2: and it is reporting a DEVICE, not a default ------------------
    //
    // The standing value is the last one, because `diag::trace_on_change`
    // emits only when the value differs from the previous under that key.
    let standing = *sides.last().expect("non-empty, tested immediately above");
    if standing < CREDIBLE_FLOOR {
        let stuck_at_default = standing == EGUI_DEFAULT_SIDE;
        let diagnosis = if stuck_at_default {
            format!(
                "★ and {EGUI_DEFAULT_SIDE} is EXACTLY `egui`'s pre-backend default: \
                 `InputState`'s `Default` sets `max_texture_side` to it, and \
                 `InputState::begin_pass` keeps it with `unwrap_or` for as long as \
                 `RawInput::max_texture_side` arrives as `None`. So the overwhelmingly likely \
                 reading is that the backend never supplied the figure and the instrument is \
                 wired to a constant, NOT that this card's limit is {EGUI_DEFAULT_SIDE}"
            )
        } else {
            format!(
                "which is under the plausibility floor of {CREDIBLE_FLOOR} without being the \
                 framework's {EGUI_DEFAULT_SIDE} default, so it is neither a stuck constant nor \
                 a figure any card running this shell has been seen to report"
            )
        };
        return Ok(Some(format!(
            "the standing `{TEXTURE_LIMIT}` reading is side={standing}, {diagnosis}. The \
             sequence traced was {sides:?}. ⚠ If this machine's card genuinely reports \
             {standing}, this check is wrong and the shell is worse than wrong: the whole-page \
             raster tier's edge budget is far above it, so a zoomed page could not be uploaded \
             at all."
        )));
    }

    report.note(format!(
        "the device reports a texture limit of {standing} on its own axis, which clears the \
         {CREDIBLE_FLOOR} plausibility floor and is not the framework's {EGUI_DEFAULT_SIDE} \
         default"
    ));

    // --- reported, not asserted --------------------------------------------
    //
    // The sequence itself is the evidence that the default was SUPERSEDED
    // rather than merely cleared: two or more readings means the field changed
    // under the instrument while it watched.
    if sides.len() > 1 {
        report.note(format!(
            "the reading changed {} time(s) while the instrument watched — {sides:?} — which is \
             the framework default being superseded by the backend's real figure, and is what \
             `trace_on_change` is for",
            sides.len() - 1
        ));
    } else {
        report.note(format!(
            "one reading only ({standing}), so the backend supplied the figure before the first \
             frame this build traced. Nothing is concluded from that either way; the assertion \
             above is on the value, not on the count"
        ));
    }

    let pressure: Vec<String> = trace
        .events(PRESSURE)
        .map(|line| line.raw.clone())
        .collect();
    if pressure.is_empty() {
        report.note(format!(
            "no `{PRESSURE}` line was traced. ★ That is the EXPECTED control result for this \
             rung — one document at fit zoom, no gestures — and it is reported rather than \
             asserted, because a silence here is a fact about how much graphics memory was free \
             on the machine that ran the sweep. ⚠ It is also what a debug build would produce \
             unconditionally, since `egui_glow` clears the error flag before this instrument \
             can read it"
        ));
    } else {
        report.note(format!(
            "{} `{PRESSURE}` line(s) were traced on the control rung, which is NOT the expected \
             result for one document at fit zoom and is worth reading in full: {}",
            pressure.len(),
            pressure.join(" | ")
        ));
    }

    Ok(None)
}

// ---------------------------------------------------------------------------
// launching
// ---------------------------------------------------------------------------

fn resolve_exe(ctx: &CheckContext) -> Result<PathBuf> {
    ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })
}

/// This module's fixture, resolved by [`crate::checks::driving::repo_fixture`].
fn repo_fixture(name: &str) -> Result<PathBuf> {
    crate::checks::driving::repo_fixture(
        name,
        "This check pins its own document and ignores --pdf: its subject is the device, and its \
         reading is only a control for as long as the document on screen is an ordinary one.",
    )
}

/// A launch with the diagnostic channel on, the window off the desktop, and no
/// input ever sent to it.
fn launch_quiet(
    ctx: &CheckContext,
    exe: &Path,
    fixture: &str,
    trace_name: &str,
) -> Result<Session> {
    let mut spec = LaunchSpec::new(exe, ctx.out(trace_name));
    spec.pdf = Some(repo_fixture(fixture)?);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    if let Some(name) = ctx.profile.viewport_env {
        spec.env.push((name.to_owned(), OFFSCREEN.to_owned()));
    }
    // Without this the line above is decoration: `Session::place` would move
    // the window to `(780, 40)` the moment it appeared, onto the desktop the
    // operator is using.
    spec.place = false;
    Session::launch(&spec, ctx.profile.trace_prefix)
}
