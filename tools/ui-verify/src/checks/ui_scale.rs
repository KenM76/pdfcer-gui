//! `ui_scale_resizes_the_chrome` — the UI-scale preference reaches the window,
//! and nothing falls off the edge when it does.
//!
//! # The defect this detects
//!
//! Three of them, and they need one run between them because each is invisible
//! to the tests that cover the other two.
//!
//! 1. **The preference is read and never applied.** `app::prefs` parses
//!    `ui_scale`, `dialogs::settings::appearance` edits it, and `app::frame`
//!    hands it to `Context::set_zoom_factor`. Every one of those has unit
//!    tests and **not one of them can see the chain joined**: the parser's
//!    tests build strings, the control's tests build a `Prefs`, and the frame
//!    hook has no test at all because it needs a live `egui::Context` inside a
//!    real window. This is `HANDOFF.md` §10's *"an `Options` flag that
//!    defaults off will silently neuter a correct decision function"* wearing
//!    a different hat — every part correct, the join unobserved.
//!
//! 2. **Something is clipped at a large scale.** This is the one that needs
//!    pixels and the reason this check exists at all. `MODES_AND_PANELS.md`
//!    states the rule twice over: *layout and clipping defects have exactly
//!    one oracle, a rendered screenshot.* This project has already shipped
//!    **a control laid out below the bottom of its own pane** (the redaction
//!    apply button, `HANDOFF.md` §2 defect 11) and **a two-row ribbon one gap
//!    short** (§10) — both with every unit test green. Doubling every control
//!    in the window is the cheapest possible search for the next one.
//!
//! 3. **The scale is applied to the page as well as the chrome.** It must not
//!    be. `set_zoom_factor` moves `pixels_per_point`, which the canvas already
//!    reads for its raster scale — so the page re-rasterises at the new device
//!    density and stays **the same size relative to the window**. If the
//!    document instead grew with the UI, the setting would be a second page
//!    zoom, which is precisely what its own copy promises it is not: *"It
//!    never changes the page or the file — only the window around them."*
//!
//! # Why it drives the FILE and not the slider
//!
//! The obvious script is: open Settings, drag the slider, capture. This check
//! does not, for two reasons and the second is the important one.
//!
//! The shallow reason is reach — the *pdfcer* group holding `file.settings` is
//! the last group on the File tab, and at the shipped 1100 px window width it
//! falls into the ribbon overflow, so driving it means opening a popup first.
//!
//! The real reason is that **the file is the path an operator's setting
//! actually takes**. A slider drag exercises the draft's live preview, which
//! is one frame of one session. Writing `preferences.txt` and launching
//! exercises the whole chain the operator depends on every morning: parse,
//! normalise, adopt, apply, lay out. If that path is broken, a slider that
//! works is worthless — the operator sets their scale, closes pdfcer, and
//! opens it the next day at 100 %.
//!
//! So this check writes the preference and launches twice, once at 1.0 and
//! once at [`LARGE`]. The slider's live preview is a separate property and is
//! deliberately not covered here; it wants its own check and its own dialog
//! step.
//!
//! # ★ The oracle: a control's SHARE of the window, not its size in points
//!
//! This is the subtle part and the first version of this check got it wrong,
//! so the reasoning is written out rather than assumed.
//!
//! The intuitive oracle is *"the ribbon tab is 28.3 pt at 1.0, so it must be
//! ~51 pt at 1.8"*. **It is not, and a build that made it so would be
//! broken.** `Context::set_zoom_factor` does not enlarge point-sized things in
//! points — it changes how many *pixels* a point is worth:
//!
//! ```text
//!                       base (1.0)        large (1.8)
//!   pixels per point    1.0               1.8
//!   window, pixels      1100 x 800        1100 x 800   (unchanged — the OS
//!                                                       window did not move)
//!   window, POINTS      1100 x 800        611 x 444    (shrinks by 1.8)
//!   ribbon tab, points  28.3 x 24.0       28.3 x 24.0  (UNCHANGED)
//!   ribbon tab, pixels  28.3 x 24.0       51.0 x 43.2  (grows by 1.8)
//!   tab as % of window  2.6 %             4.6 %        (grows by 1.8)
//! ```
//!
//! A control specified as 24 pt tall stays 24 pt tall; that is what "specified
//! in points" means. What changes is the **canvas it is laid out on**, and
//! therefore its share of it. So the measurement that tracks what the operator
//! actually sees — *things got bigger* — is the region's **fraction of the
//! client area**, and that is what [`drive`] asserts.
//!
//! The absolute point sizes are still printed beside the fractions, because
//! when this fails they are what says which of the two regimes the build is
//! in: unchanged points with an unchanged fraction means the zoom factor never
//! moved, and *changed* points would mean something is scaling the widgets
//! themselves, which is a different defect wearing the same symptom.
//!
//! A capture is still taken at both scales, because assertion 2 is about
//! **where** things landed and the screenshot is the artefact a human reads
//! when this fails.
//!
//! # What "clipped" means here, precisely
//!
//! A declared region whose rect is not contained by the client area. That is a
//! narrow test and deliberately so: it catches the redaction-apply defect
//! exactly (a control declared at `y = 801.7` in a body ending at `y = 770.0`)
//! and it cannot produce a false positive from a control that is merely
//! *tight*, because a rect either fits or does not.
//!
//! It does **not** catch a control clipped by a scroll area inside a panel,
//! because such a control is legitimately outside its viewport and the
//! application says so by declaring it there. Naming that limit rather than
//! widening the test: a check that flagged every scrolled-out row would fire
//! on every run and be switched off within a week.

use std::path::Path;

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The scale the second launch runs at.
///
/// **1.8, not the 2.0 maximum.** Two reasons, and the first is about what the
/// check can conclude:
///
/// * At exactly the maximum, a build that silently clamped to *some* ceiling
///   would produce the same measurement as one that honoured the request. 1.8
///   is inside the range, so the assertion below distinguishes "applied" from
///   "clamped to whatever it felt like".
/// * 1.8 is enough to break a layout that is going to break. The shipped
///   window is 1100 × 800 and the ribbon band is ~103 pt; at 1.8 that is
///   ~186 pt of an 800 pt window, which is where a two-row group with a
///   caption starts competing for space with the canvas.
const LARGE: f32 = 1.8;

/// How far the measured ratio may sit from [`LARGE`] and still pass.
///
/// Generous, and it has to be: a laid-out control's size is not a pure
/// multiple of the scale. Text is measured in whole pixels at the device
/// density, padding is rounded, and `egui` snaps some rects to the pixel grid
/// — so a 30.3 pt tab at 1.8 lands near 54.5 pt rather than at exactly
/// 54.54 pt.
///
/// 12 % is wide enough that no rounding regime trips it and narrow enough that
/// **it cannot be satisfied by the wrong answer**: the two failures worth
/// catching are "no scaling at all" (ratio 1.0, which is 44 % away) and
/// "scaled by the wrong factor" (the nearest plausible wrong factor is the
/// device pixel ratio, 1.0 or 2.0 on this hardware, both far outside).
const RATIO_TOLERANCE: f32 = 0.12;

/// The regions measured for the scaling assertion.
///
/// Ribbon chrome specifically, because it is drawn by `egui-shell` from the
/// theme's own metrics and is therefore the surface a scale change is
/// *supposed* to move. The canvas is deliberately excluded — assertion 3 is
/// that the page does **not** grow, and mixing the two into one list would
/// make a pass ambiguous.
const SCALED_REGIONS: &[&str] = &[
    "ribbon.tab.file",
    "ribbon.modes",
    "ribbon.qat.file.open",
    "status-page-box",
];

/// See the module documentation.
pub struct UiScaleResizesTheChrome;

impl Check for UiScaleResizesTheChrome {
    fn name(&self) -> &'static str {
        "ui_scale_resizes_the_chrome"
    }

    fn defect(&self) -> &'static str {
        "the UI-scale preference is parsed, edited and never handed to egui, so every \
         control is drawn at one size on every machine — or it IS applied and something \
         lands outside the window at a large scale, which no unit test can see"
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

/// One launch at one scale: write the preference, start, read what it declared.
///
/// Returns the declared regions and the client area, which is everything the
/// assertions need. Kept separate from [`drive`] so the two launches cannot
/// drift apart — the failure that would produce is a comparison between two
/// runs configured differently, which would look exactly like a scaling defect.
struct Measured {
    /// Every region the application declared, by name.
    names: Vec<String>,
    /// The client area, in points.
    client: LRect,
    /// Whether the application traced that it applied a scale at start-up.
    traced_initial: bool,
    /// Where the trace went, for the report.
    trace_path: std::path::PathBuf,
}

fn measure(
    ctx: &CheckContext,
    report: &mut CheckReport,
    scale: f32,
    tag: &str,
) -> Result<(Measured, crate::trace::Trace)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no fixture document. Pass --pdf: this check needs a window with a document in \
             it, because a document-less window draws less chrome and the point is the \
             chrome.",
        )
    })?;

    write_preference(&exe, scale)?;

    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("ui_scale.{tag}.trace.txt")));
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
    report.note(format!(
        "launched at ui_scale = {scale:.2} as pid {}",
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    // Generous: a scale change forces a full re-layout AND a re-raster of the
    // page at the new device density, and on the benchmark sheet that is over
    // a second. Measuring a half-laid-out window would report a clipped
    // control that is merely not finished yet.
    session.settle(60);

    let trace = session.trace()?;
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot \
             say where its controls are and there is nothing to measure.",
            ctx.profile.name
        ))
    })?;

    // ★ The client area in the APPLICATION's points, which is not the
    // harness's points.
    //
    // `WindowFrame::scale` is the **OS** device-pixel ratio, measured from the
    // window. The application's `ui-rect` events are in `egui` logical points,
    // and egui's points-per-pixel is `native_pixels_per_point × zoom_factor`.
    // So at a zoom factor of 1.8 a 1100 px window is 611 pt wide to the
    // application, not 1100.
    //
    // Getting this wrong would make assertion 2 fire on every region in a
    // scaled window — every rect would appear to be outside a client area
    // measured 1.8× too large — which is the most confident possible way to
    // report a defect that is entirely the harness's arithmetic.
    let frame = session.frame()?;
    let points_per_pixel = frame.scale * scale;
    let client = LRect::new(
        crate::geom::Pt::new(0.0, 0.0),
        crate::geom::Pt::new(
            frame.client_size.0 as f32 / points_per_pixel,
            frame.client_size.1 as f32 / points_per_pixel,
        ),
    );
    let traced_initial = trace.events("ui-scale-initial").next().is_some();
    let names = declared_names(&trace, ui_rect, "");
    // ★ The capture. Assertion 2 is about WHERE things landed, and a rect list
    // in a failure string is not something a human can judge — the screenshot
    // is. Taken before the session is dropped, because dropping it closes the
    // window.
    let png = ctx.out(&format!("ui_scale.{tag}.png"));
    match crate::capture::window_to_png(&session, &png) {
        Ok(_) => {
            report.artifact(png);
        }
        Err(e) => {
            report.note(format!(
                "could not capture the window at {scale:.2}: {e}. The measurements below \
                 still hold — they come from the application's own declarations — but the \
                 artefact a reader would use to judge a layout failure is missing."
            ));
        }
    }
    let path = session.trace_path().to_path_buf();
    // The session is dropped here, which closes the window. Everything the
    // assertions need has been read out first, deliberately: holding two
    // windows open at once would leave the second one behind whichever the
    // desktop decided to raise, and a capture would then be of the wrong one.
    drop(session);
    Ok((
        Measured {
            names,
            client,
            traced_initial,
            trace_path: path,
        },
        trace,
    ))
}

/// Write `ui_scale = <scale>` into the profile's own preferences file.
///
/// # ★ It writes the WHOLE file, and that is deliberate
///
/// Not a surgical edit of one line. `Prefs::write_to_string` is what the
/// application itself writes, so a file assembled any other way would be
/// testing the parser against a shape the writer never produces — and the
/// round-trip property is already unit-tested, so the interesting question
/// here is whether the *shipped* shape is honoured end to end.
///
/// # ★ The file IS restored to 1.0 at the end, and the first version's
/// argument for not restoring it was wrong
///
/// That argument ran: *"this harness already writes `layout.ron` and
/// `recent.txt` and does not restore those either, and a check that tidied up
/// would hide the state the next check inherits — which is exactly the failure
/// `delete_key` had."*
///
/// It sounds like the lesson from `delete_key` and it is the inverse of it.
/// The lesson there was **a check must establish the state it depends on**.
/// Leaving 1.8 behind does not help another check establish anything; it
/// silently changes the coordinate system every other check measures in. The
/// measured cost, on the first full run after this check was added:
///
/// ```text
///   before   20 passed, 0 failed, 4 skipped
///   after     3 passed, 1 failed, 21 skipped
/// ```
///
/// Twenty-one checks stopped being able to begin, because they were written
/// against a 1100 pt window and met a 611 pt one. The check that was supposed
/// to find layout defects had created the largest one in the suite's history.
///
/// So it restores — and *restoring* is not *hiding*. The distinction that
/// matters is **who owns the state**: `layout.ron` and `recent.txt` are
/// written by the APPLICATION as a consequence of being driven, and erasing
/// those would hide what driving it does. `preferences.txt` here is written by
/// the HARNESS as an input, and an input a check injects is one it owes the
/// suite a return to neutral on.
///
/// Restored to 1.0 rather than deleted, deliberately: 1.0 is a value the check
/// has just proved the application honours, and a *missing* file exercises the
/// absent-file path instead — a different state, and not the one the other
/// checks were written against.
///
/// # Errors
///
/// The directory could not be created or the file could not be written. Both
/// are SKIPs at the call site: a preference that could not be written means
/// the check never began, and reporting that as a scaling failure would name
/// the wrong subsystem.
fn write_preference(exe: &Path, scale: f32) -> Result<()> {
    let dir = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory to write userdata into"))?
        .join("userdata");
    std::fs::create_dir_all(&dir)
        .map_err(|e| Error::new(format!("could not create {}: {e}", dir.display())))?;
    let path = dir.join("preferences.txt");
    // Only the one key. Every other preference is absent, which the loader
    // treats as "use the default" — the same state a first run produces, so
    // this isolates the variable under test from anything a previous check
    // happened to leave behind.
    let body = format!("# written by ui-verify's ui_scale check\nui_scale = {scale:.2}\n");
    std::fs::write(&path, body)
        .map_err(|e| Error::new(format!("could not write {}: {e}", path.display())))?;
    Ok(())
}

/// Run both launches and compare them.
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    // ★ Restore the profile to 1.0 on EVERY path out of this check.
    //
    // A guard rather than a line at the end, because there are eleven returns
    // below — three SKIPs and eight FAILs — and the one that gets forgotten is
    // the one that leaves the whole suite measuring a 611 pt window. See
    // `write_preference` for what that cost when it was not restored at all.
    //
    // Failure to restore is REPORTED and does not change the verdict: the
    // check's own assertions are about the application, and a harness that
    // downgraded a real pass because it could not tidy up would be reporting
    // its own housekeeping as a defect in the program.
    struct RestoreScale<'a>(Option<&'a Path>);
    impl Drop for RestoreScale<'_> {
        fn drop(&mut self) {
            if let Some(exe) = self.0
                && let Err(e) = write_preference(exe, 1.0)
            {
                eprintln!(
                    "ui-verify: WARNING — could not restore ui_scale to 1.0 ({e}). Every later check will measure a scaled window. Fix by deleting userdata/preferences.txt beside the binary."
                );
            }
        }
    }
    let exe_for_restore = ctx.resolve_exe();
    let _restore = RestoreScale(exe_for_restore.as_deref());

    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let (base, base_trace) = measure(ctx, report, 1.0, "base")?;
    let (big, big_trace) = measure(ctx, report, LARGE, "large")?;

    // --- assertion 0: `pixels_per_point` actually moved ---------------------
    //
    // ★ Measured from the CLIENT AREA IN POINTS, not from a trace line, and
    // the first version of this check got that wrong too.
    //
    // The obvious oracle is *"did the application trace that it changed the
    // zoom factor?"*. It is the wrong question, and the reason is a good one:
    // once the scale is applied at start-up (`lib.rs`, before the first frame,
    // so a scaled profile does not flash at 1.0 for one frame), the per-frame
    // hook correctly finds no delta and correctly says nothing. **A build that
    // traced nothing would then be the CORRECT build**, and asserting on the
    // line would fail the fix.
    //
    // The client area in points is the property itself rather than a report of
    // it. The OS window is the same 1100 x 800 px at both scales — the harness
    // never resizes it — so if `pixels_per_point` moved by 1.8, the window
    // must be 1.8x SMALLER in points. Nothing can fake that, and it holds
    // whichever code path set the factor.
    //
    // `ui-scale-initial` is still read, as corroboration and for the report.
    let client_ratio = base.client.width() / big.client.width();
    report.note(format!(
        "client area {:.0}x{:.0} pt → {:.0}x{:.0} pt (the OS window is 1100x800 px at both \
         scales, so a smaller area in POINTS is `pixels_per_point` having moved) — x{:.2}",
        base.client.width(),
        base.client.height(),
        big.client.width(),
        big.client.height(),
        client_ratio
    ));
    if (client_ratio - LARGE).abs() > RATIO_TOLERANCE {
        return Ok(Some(format!(
            "the window is {client_ratio:.2}x smaller in points at ui_scale = {LARGE:.2} \
             when it should be {LARGE:.2}x smaller. A ratio of 1.00 means \
             `pixels_per_point` never moved, so the preference did not reach \
             `Context::set_zoom_factor` at all — the candidates are the start-up call in \
             `lib.rs` and the per-frame hook in `app::frame` step 0b. Trace: {}.",
            big.trace_path.display()
        )));
    }
    report.note(match (base.traced_initial, big.traced_initial) {
        (true, true) => {
            "both launches traced `ui-scale-initial`, so the preference was applied before \
             the first frame and no launch flashed at the wrong size"
        }
        _ => {
            "at least one launch traced no `ui-scale-initial` line. Not a failure — the \
             measurement above already proves the factor moved — but it means the \
             start-up path did not run and the scale was applied by the per-frame hook \
             instead, which costs one frame at the wrong size on every launch"
        }
    });

    // --- assertion 1: the chrome grew, by about the right factor ------------
    let mut measured = 0usize;
    for name in SCALED_REGIONS {
        let (Some(a), Some(b)) = (
            declared(&base_trace, ui_rect, name),
            declared(&big_trace, ui_rect, name),
        ) else {
            // Not a failure: a region this build does not declare is a region
            // this check has nothing to say about, and naming it would be a
            // claim about a control that may legitimately not exist in this
            // mode. The count below is what turns "all of them missing" into
            // a SKIP.
            continue;
        };
        if a.width() <= 0.0 || a.height() <= 0.0 {
            continue;
        }
        // ★ The region's SHARE of the client area, not its size in points.
        // See the module header's table for why the point sizes are expected
        // to be unchanged and why a build in which they grew would be broken
        // in a different way.
        let share_a = (
            a.width() / base.client.width(),
            a.height() / base.client.height(),
        );
        let share_b = (
            b.width() / big.client.width(),
            b.height() / big.client.height(),
        );
        let ratio_w = share_b.0 / share_a.0;
        let ratio_h = share_b.1 / share_a.1;
        report.note(format!(
            "  {name:<28} {:.1}x{:.1} pt (both) · {:.2}%x{:.2}% → {:.2}%x{:.2}% of the window \
             (x{ratio_w:.2}, y{ratio_h:.2})",
            a.width(),
            a.height(),
            share_a.0 * 100.0,
            share_a.1 * 100.0,
            share_b.0 * 100.0,
            share_b.1 * 100.0,
        ));
        measured += 1;
        for (axis, ratio) in [("width", ratio_w), ("height", ratio_h)] {
            if (ratio - LARGE).abs() > RATIO_TOLERANCE {
                return Ok(Some(format!(
                    "`{name}` takes {ratio:.2}x as much of the window's {axis} at \
                     ui_scale = {LARGE:.2}, when it should take {LARGE:.2}x (tolerance \
                     {RATIO_TOLERANCE:.2}). Its size in points is {:.1}x{:.1} at the base \
                     scale and {:.1}x{:.1} at the large one — those SHOULD be equal, and \
                     which of the two numbers moved says which defect this is. \
                     A share ratio near 1.0 with unchanged points means the zoom factor \
                     never reached `egui`, so `pixels_per_point` did not move and the \
                     window is the same size in points. Changed POINTS mean something is \
                     rescaling the widgets themselves rather than the point, which is a \
                     different fault with the same symptom. A ratio near the display's \
                     device pixel ratio means the native value is being used in place of \
                     the preference — `set_zoom_factor` MULTIPLIES the native value and \
                     must not replace it. Traces: {} and {}.",
                    a.width(),
                    a.height(),
                    b.width(),
                    b.height(),
                    base.trace_path.display(),
                    big.trace_path.display()
                )));
            }
        }
    }
    if measured == 0 {
        return Err(Error::new(format!(
            "none of the {} measured regions was declared at both scales, so there is \
             nothing to compare. The application declared {} region(s) at the large scale. \
             This is a harness/application mismatch rather than a scaling result: either \
             the region names moved, or the window came up without its ribbon.",
            SCALED_REGIONS.len(),
            big.names.len()
        )));
    }
    report.note(format!(
        "{measured} of {} chrome regions scaled by ~{LARGE:.2}",
        SCALED_REGIONS.len()
    ));

    // --- assertion 2: nothing landed outside the window ---------------------
    //
    // The reason this check takes two launches instead of one. See the header
    // on what "clipped" means and what it deliberately does not cover.
    let mut clipped = Vec::new();
    for name in &big.names {
        let Some(rect) = declared(&big_trace, ui_rect, name) else {
            continue;
        };
        if !rect.is_substantial() {
            continue;
        }
        // A small slack, in points. A rect that overhangs by a fraction of a
        // point is a rounding artefact of the layout, not a control the
        // operator cannot reach — and this assertion must not be the kind that
        // fires on a build with nothing wrong with it, because the first thing
        // that happens to such an assertion is that it gets switched off.
        const SLACK: f32 = 1.0;
        let outside = rect.min.x < -SLACK
            || rect.min.y < -SLACK
            || rect.max.x > big.client.max.x + SLACK
            || rect.max.y > big.client.max.y + SLACK;
        if outside {
            clipped.push(format!("{name} at {rect:?}"));
        }
    }
    if !clipped.is_empty() {
        let listed: Vec<&str> = clipped.iter().take(8).map(String::as_str).collect();
        return Ok(Some(format!(
            "at ui_scale = {LARGE:.2}, {} declared region(s) lie outside the {:.0}x{:.0} pt \
             client area — the operator cannot reach them and no unit test can see it. This \
             is the redaction-apply defect's exact shape (a control declared at y = 801.7 in \
             a body ending at y = 770.0), reached by scaling instead of by adding copy. \
             Offenders: {}{}. Traces: {}.",
            clipped.len(),
            big.client.width(),
            big.client.height(),
            listed.join("; "),
            if clipped.len() > listed.len() {
                " …"
            } else {
                ""
            },
            big.trace_path.display()
        )));
    }
    report.note(format!(
        "all {} declared region(s) fit inside the {:.0}x{:.0} pt client area at {LARGE:.2}x",
        big.names.len(),
        big.client.width(),
        big.client.height()
    ));

    // --- assertion 3: the PAGE did not grow with the chrome -----------------
    //
    // The setting's own copy promises this: "It never changes the page or the
    // file — only the window around them." A `page` region that scaled with
    // the chrome would mean the preference had become a second document zoom,
    // which is the one thing an operator reading that sentence would not
    // expect.
    //
    // Reported rather than asserted when the region is absent at either scale,
    // because a fit-to-page document legitimately re-fits when the viewport
    // changes size, and this check cannot separate "grew because of the zoom
    // factor" from "re-fitted because the canvas got smaller". What it CAN
    // say is the direction: the chrome took more room, so a page that is
    // fitting must have got SMALLER, never larger.
    match (
        declared(&base_trace, ui_rect, "page"),
        declared(&big_trace, ui_rect, "page"),
    ) {
        (Some(a), Some(b)) if a.width() > 0.0 => {
            let ratio = b.width() / a.width();
            report.note(format!(
                "  {:<28} {:.1} pt wide → {:.1} pt wide   (x{ratio:.2})",
                "page",
                a.width(),
                b.width()
            ));
            if ratio > 1.0 + RATIO_TOLERANCE {
                return Ok(Some(format!(
                    "the PAGE grew by {ratio:.2} when the UI scale went to {LARGE:.2}. The \
                     setting's own copy promises it 'never changes the page or the file — \
                     only the window around them', and a page that grows with the chrome has \
                     become a second document zoom. Note the chrome takes MORE of the window \
                     at a larger scale, so a fitted page should get smaller, not larger."
                )));
            }
        }
        _ => {
            report.note(
                "the `page` region was not declared at both scales, so assertion 3 (the page \
                 does not grow with the chrome) was not evaluated. Reported rather than \
                 silently skipped.",
            );
        }
    }

    Ok(None)
}
