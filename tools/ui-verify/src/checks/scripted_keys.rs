//! **The keystroke seam is a route to the viewer verbs, and this is what
//! proves it** — a zoom ladder climbed by a window that takes no OS input at
//! all.
//!
//! # What this file is for
//!
//! `app::keyboard::scripted` reads `PDFCER_DIAG_KEYS`, a comma-separated list
//! of chords, and pushes a real `egui::Event::Key` into the frame ahead of the
//! keyboard collector. It exists because the four viewer verbs — zoom in, zoom
//! out, next page, previous page — have **no registered command id**, so
//! `PDFCER_DIAG_INVOKE` cannot ring them, and a window placed off the desktop
//! has no other way to be driven. Its own argument is in that module;
//! `DESIGNS.md` carries why a seam is legitimate there where registering a
//! command would not be.
//!
//! This check is the seam's only assertion. Until it existed the seam was
//! evidenced by a driven run recorded in a commit message, which is a
//! measurement that stops being re-taken the moment it is written down.
//!
//! # ★★★ Why it reads the application's zoom and never counts rungs
//!
//! The seam traces one line per chord:
//!
//! ```text
//! diag-keys index=0 chord=Ctrl++ spelled=yes
//! ```
//!
//! `spelled=yes` means the key was **pushed**, and nothing more. The seam runs
//! before the collector and cannot know what became of the press — and it
//! emitted exactly that line for a rung whose effect was destroyed later in
//! the same frame. Six chords delivered on consecutive frames produced six
//! `spelled=yes` lines and **five** rungs of zoom: the first chord landed
//! before the canvas had been laid out, `FitMode::Page` was still standing,
//! and that frame's fit solve overwrote the zoom the chord had just set.
//!
//! ⇒ *"I asked for N and got N `spelled=yes` lines"* is an assertion both
//! outcomes satisfy. So is an end-state assertion: the broken run and the
//! sound one **both finish at 125 %**, differing only in how many steps they
//! took to get there.
//!
//! What this check reads instead is the application's own `status … zoom=`,
//! **windowed between one rung's `diag-keys` line and the next one's**, so a
//! rung that produced nothing is a failure rather than a silence, and a rung
//! whose effect was undone shows up as a zoom that did not move.
//!
//! # ★★ The unspellable rung, and what it is for
//!
//! One entry in the list is a chord that cannot be spelled. It is there so
//! that `spelled=` is **shown to vary**: a seam that wrote `spelled=yes`
//! unconditionally would satisfy every other assertion here, and the field
//! would be decoration. The same rung doubles as the negative control for the
//! zoom reading — it must move the zoom by nothing at all, which no other rung
//! in the list is allowed to do.
//!
//! # What this does NOT reach, and it is the row next door
//!
//! ⚠ **This says nothing about `OPERATOR_REQUESTS.md` O220.** The operator's
//! report is that `Ctrl`+wheel stops zooming out once the raster refusal has
//! appeared. `Ctrl`+wheel is a continuous `zoom_delta` arriving through the
//! pointer, which is a different route from the discrete keyboard action this
//! ladder climbs; a green run here is not evidence about that route. It also
//! stays entirely below the raster ceiling by choice of fixture, so it is not
//! evidence about O218 or O219 either.
//!
//! It also does not assert that a raster **completed** — only that one was
//! ordered. A check needing a finished raster waits for the application's own
//! `render-async-done`.

use std::path::{Path, PathBuf};

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The document this check opens, pinned, with `--pdf` ignored.
///
/// ★ An A1 CAD sheet fits at about 30 %, so six rungs of zoom-in land at
/// 125 % — a discrete climb that stays entirely under the whole-page raster
/// ceiling. The suite's usual clean control opens near 100 % and the same
/// ladder would run into the refusal, which would make a rung's silence
/// ambiguous between *"the seam did not deliver"* and *"the rasterizer
/// declined"*, and those two send a reader to opposite ends of the program.
const FIXTURE: &str = "a1-titleblock.pdf";

/// The seam's environment variable.
const KEYS_ENV: &str = "PDFCER_DIAG_KEYS";

/// Zoom in one rung, in `app::keyboard::parse_chord`'s grammar.
const ZOOM_IN: &str = "Ctrl++";

/// Zoom out one rung.
const ZOOM_OUT: &str = "Ctrl+-";

/// A chord no `egui::Key` answers to. See the module header — it is the
/// evidence that `spelled=` is read from something rather than printed.
const UNSPELLABLE: &str = "Ctrl+NoSuchKey";

/// How many zoom-in rungs the ladder climbs before it turns round.
const CLIMB: usize = 6;

/// The application's view-state line, carrying `zoom=` and `fit=`.
const STATUS: &str = "status";

/// The seam's own line, carrying `index=`, `chord=` and `spelled=`.
const KEYS: &str = "diag-keys";

/// The `fit=` a `status` line carries while fit-to-page is standing.
///
/// A discrete zoom verb must clear it, and the check reads it as a fault
/// rather than as a curiosity — see [`walk_the_ladder`].
const FIT_PAGE: &str = "Page";

/// A raster order, carrying `scale=`. Presence is asserted per rung; the
/// figures are reported, not asserted — see the module header.
const SPAWN: &str = "render-spawn";

/// Zoom is traced as a percentage, and the rungs of the ladder are whole
/// points apart. This separates *"moved"* from *"did not move"* with two
/// orders of magnitude to spare either way.
const ZOOM_EPSILON: f32 = 0.01;

/// Off the desktop, at the harness's usual size.
///
/// Nothing is ever aimed at this window and no OS input is sent to it — which
/// is the whole point of the seam — so this check can run while the operator
/// is working.
const OFFSCREEN: &str = "-4200,-4200,1400,900";

/// How long to wait for the last rung to appear before giving up on it.
///
/// The seam paces its chords twenty frames apart and requests a repaint on
/// every frame while chords remain, so on any machine this runs on the whole
/// list is delivered in about a second. The generous deadline is for a loaded
/// machine; exceeding it is reported as a finding rather than a hang.
const RUNG_DEADLINE: std::time::Duration = std::time::Duration::from_secs(30);

/// The list the seam is given, in order.
fn chord_list() -> Vec<&'static str> {
    let mut list: Vec<&'static str> = vec![ZOOM_IN; CLIMB];
    list.push(UNSPELLABLE);
    list.push(ZOOM_OUT);
    list
}

/// What the application did in one rung's window.
struct Rung {
    /// The seam's `index=`, which is the rung's position in the list.
    index: usize,
    /// The seam's `chord=`, echoed back.
    chord: String,
    /// The seam's `spelled=`, which says only that the key was pushed.
    spelled: String,
    /// Where the rung's own line sits in the capture. The window this rung
    /// owns runs from here to the next rung's line.
    lineno: usize,
}

/// See the module documentation.
pub struct TheScriptedKeystrokeSeamClimbsTheZoomLadder;

impl Check for TheScriptedKeystrokeSeamClimbsTheZoomLadder {
    fn name(&self) -> &'static str {
        "the_scripted_keystroke_seam_climbs_the_zoom_ladder"
    }

    fn defect(&self) -> &'static str {
        "the only headless route to the viewer verbs is dead or unpaced. Either the scripted \
         keystroke never reaches the keyboard collector — in which case zoom, zoom-out and \
         paging cannot be driven at all in a window placed off the desktop, and every check \
         that would have used them is unwritable — or a chord is delivered into a frame that \
         then overwrites what it did, which the seam reports as a successful press. A ladder \
         that loses a rung and still arrives at the same zoom is indistinguishable from a \
         sound one by any end-state reading"
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
    let list = chord_list();
    let last_index = list.len() - 1;

    let exe = resolve_exe(ctx)?;
    let session = launch_scripted(ctx, &exe, &list)?;
    report.note(format!(
        "launched {} on fixtures/{FIXTURE} as pid {} with {KEYS_ENV}={}, the window off the \
         desktop and NO OS input sent — which is the condition the seam exists for",
        exe.display(),
        session.pid(),
        list.join(",")
    ));
    report.artifact(session.trace_path().to_path_buf());

    let trace = wait_for_rung(&session, last_index)?;

    // --- the precondition ---------------------------------------------------
    //
    // A launch that opened nothing traces no `status` line, and every reading
    // below would then be legitimately absent. This project has twice produced
    // confident, detailed, entirely wrong defect reports out of exactly that
    // shape.
    if trace.last(STATUS).is_none() {
        return Err(Error::new(format!(
            "the application launched and never traced a `{STATUS}` line, so it opened no \
             document. Any verdict below would be about an empty shell rather than about the \
             seam. Check that fixtures/{FIXTURE} exists and that an absolute path was passed."
        )));
    }

    // --- 1: the seam delivered the list, once each, in order ----------------
    let rungs = read_rungs(&trace);
    if let Some(failure) = check_delivery(&rungs, &list) {
        return Ok(Some(failure));
    }

    // --- 2: ★ `spelled=` is read from something --------------------------
    if let Some(failure) = check_spelling(&rungs, &list) {
        return Ok(Some(failure));
    }

    // --- 3: ★★★ the application's own zoom, rung by rung -------------------
    walk_the_ladder(&trace, &rungs, report)
}

// ---------------------------------------------------------------------------
// the seam's own report
// ---------------------------------------------------------------------------

fn read_rungs(trace: &Trace) -> Vec<Rung> {
    trace
        .events(KEYS)
        .map(|line| Rung {
            index: line.get_usize("index").unwrap_or(usize::MAX),
            chord: line.get("chord").unwrap_or_default().to_owned(),
            spelled: line.get("spelled").unwrap_or_default().to_owned(),
            lineno: line.lineno,
        })
        .collect()
}

/// One line per list entry, carrying its own position, in order.
///
/// A seam that delivered a chord twice, skipped one, or renumbered them would
/// pass every zoom assertion below by accident — the windows would still be in
/// ascending order and the zoom would still climb. This is what makes the
/// rung-to-chord mapping real rather than assumed.
fn check_delivery(rungs: &[Rung], list: &[&str]) -> Option<String> {
    if rungs.len() != list.len() {
        return Some(format!(
            "{KEYS_ENV} named {} chord(s) and the seam traced {} `{KEYS}` line(s). Every \
             assertion below reads the application's zoom in the window a rung owns, and a \
             window can only be drawn where a rung is. Traced: [{}]",
            list.len(),
            rungs.len(),
            rungs
                .iter()
                .map(|r| format!("index={} chord={}", r.index, r.chord))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    for (position, (rung, chord)) in rungs.iter().zip(list).enumerate() {
        if rung.index != position {
            return Some(format!(
                "the {position}th `{KEYS}` line carries index={}, so the seam is not numbering \
                 its chords by their position in {KEYS_ENV}. The rung-to-chord mapping every \
                 reading below depends on is therefore unknown",
                rung.index
            ));
        }
        if rung.chord != *chord {
            return Some(format!(
                "rung {position} was asked for `{chord}` and the seam echoed `{}`, so the list \
                 is not being taken in the order it was written",
                rung.chord
            ));
        }
    }
    None
}

/// ★ `spelled=` must vary with the chord, or it is decoration.
fn check_spelling(rungs: &[Rung], list: &[&str]) -> Option<String> {
    for (rung, chord) in rungs.iter().zip(list) {
        let want = if *chord == UNSPELLABLE { "no" } else { "yes" };
        if rung.spelled != want {
            let why = if *chord == UNSPELLABLE {
                format!(
                    "`{UNSPELLABLE}` names no key `egui::Key::from_name` accepts, so \
                     `app::keyboard::parse_chord` must decline it. A seam that reports it as \
                     spelled is writing `spelled=yes` without consulting anything, and the \
                     field says nothing about any of the other rungs either"
                )
            } else {
                format!(
                    "`{chord}` is a spelling `app::keyboard::OWNED` lists for a viewer verb, so \
                     the seam declining it means that verb has no headless route at all"
                )
            };
            return Some(format!(
                "rung {} was `{chord}` and the seam traced spelled={}, wanted spelled={want}. \
                 {why}",
                rung.index, rung.spelled
            ));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// the application's own answer
// ---------------------------------------------------------------------------

/// The last `status … zoom=` the application traced strictly inside a window,
/// with the `fit=` it carried.
fn zoom_within(trace: &Trace, from: usize, to: usize) -> Option<(f32, String)> {
    trace
        .events(STATUS)
        .filter(|l| l.lineno > from && l.lineno < to)
        .filter_map(|l| {
            Some((
                l.get_f32("zoom")?,
                l.get("fit").unwrap_or_default().to_owned(),
            ))
        })
        .last()
}

/// The raster orders inside a window, as their `scale=` figures.
fn spawns_within(trace: &Trace, from: usize, to: usize) -> Vec<f32> {
    trace
        .events(SPAWN)
        .filter(|l| l.lineno > from && l.lineno < to)
        .filter_map(|l| l.get_f32("scale"))
        .collect()
}

fn walk_the_ladder(
    trace: &Trace,
    rungs: &[Rung],
    report: &mut CheckReport,
) -> Result<Option<String>> {
    // ★★★ The baseline, and the first place the unpaced seam went red.
    //
    // A chord delivered before the application has published any view state at
    // all is a chord delivered before the canvas has been laid out — and the
    // fit solve that runs later in that same frame overwrites whatever the
    // chord set. Measured: with the chords on consecutive frames the first
    // `status` line of the run came AFTER the first `diag-keys` line, and the
    // rung was lost in silence.
    let first = rungs.first().expect("delivery checked a non-empty list");
    let Some((baseline, baseline_fit)) = zoom_within(trace, 0, first.lineno) else {
        return Ok(Some(format!(
            "the application had published no `{STATUS}` line at all when the seam delivered \
             rung 0, so the first chord arrived before the canvas had been laid out. Whatever it \
             set was overwritten by that same frame's fit solve, and the seam reported it as \
             spelled. ★ The pacing in `app::keyboard::scripted` is what prevents this, and the \
             load-bearing part of it is that the gap sits in FRONT of the first chord and not \
             only between chords"
        )));
    };
    report.note(format!(
        "the view state before the first chord was zoom={baseline} fit={baseline_fit}, which is \
         the fit-to-page solution for this sheet and the foot of the ladder"
    ));

    let mut previous = baseline;
    let mut series: Vec<String> = vec![format!("{baseline} (fit)")];

    for (position, rung) in rungs.iter().enumerate() {
        let to = rungs
            .get(position + 1)
            .map_or(usize::MAX, |next| next.lineno);
        let seen = zoom_within(trace, rung.lineno, to);
        let spawns = spawns_within(trace, rung.lineno, to);

        if rung.chord == UNSPELLABLE {
            // The negative control. A rung that was never spelled must have
            // done nothing, and a zoom that moved here means something else in
            // the frame is moving it — which would make every other rung's
            // reading a coincidence rather than a consequence.
            if let Some((zoom, _)) = seen
                && (zoom - previous).abs() > ZOOM_EPSILON
            {
                return Ok(Some(format!(
                    "rung {} was the unspellable chord `{UNSPELLABLE}`, which the seam correctly \
                     declined to deliver, and the zoom moved from {previous} to {zoom} anyway. \
                     Nothing pressed a key, so the ladder's other rungs are not evidence that \
                     pressing one does anything",
                    rung.index
                )));
            }
            series.push(format!("{previous} (unspellable, unchanged)"));
            continue;
        }

        let Some((zoom, fit)) = seen else {
            return Ok(Some(format!(
                "rung {} delivered `{}` and the application traced no `{STATUS}` line before the \
                 next rung. The seam says the key was pushed; the application never published a \
                 view state as a result, so either the collector is not seeing the event, the \
                 chord is not reaching `app::keyboard`'s owned table, or the press was consumed \
                 by something in front of it. Series so far: [{}]",
                rung.index,
                rung.chord,
                series.join(" -> ")
            )));
        };

        // ★★ A rung that moved the number while a fit mode is STILL STANDING
        // has not really moved it: the next layout pass solves fit from the
        // canvas rect and writes the zoom again. That is the mechanism behind
        // the swallowed rung, and this is the reading that names it directly
        // rather than inferring it from a number that failed to climb.
        if fit == FIT_PAGE {
            return Ok(Some(format!(
                "rung {} delivered `{}`, the zoom read {zoom}, and the view was still in \
                 fit={FIT_PAGE}. A discrete zoom verb sets an EXPLICIT zoom and must leave the \
                 fit mode; while fit is standing, the next layout pass solves it from the canvas \
                 rect and overwrites whatever the chord set — so this rung's effect has a frame \
                 to live and then goes. Series: [{}]",
                rung.index,
                rung.chord,
                series.join(" -> ")
            )));
        }

        let climbing = rung.chord != ZOOM_OUT;
        let moved_right_way = if climbing {
            zoom > previous + ZOOM_EPSILON
        } else {
            zoom < previous - ZOOM_EPSILON
        };
        if !moved_right_way {
            let direction = if climbing { "IN" } else { "OUT" };
            return Ok(Some(format!(
                "rung {} delivered `{}` — zoom {direction} — and the zoom went from {previous} \
                 to {zoom}. ★ A rung that does not move, or moves the wrong way, is the exact \
                 shape of a chord whose effect was overwritten later in the same frame: the \
                 press was real, was dispatched, and was then discarded by a stage that did not \
                 know a gesture had happened. It is also the shape of a handler that is dead in \
                 one direction only, which is why the ladder turns round at the top. Series: \
                 [{}]",
                rung.index,
                rung.chord,
                series.join(" -> ")
            )));
        }

        if spawns.is_empty() {
            return Ok(Some(format!(
                "rung {} moved the zoom from {previous} to {zoom} and ordered no raster: no \
                 `{SPAWN}` line was traced before the next rung. The number on the status bar \
                 moved without the page being asked to redraw at the new scale, so the operator \
                 would see a stale page under a changed figure",
                rung.index
            )));
        }

        series.push(format!("{zoom}"));
        previous = zoom;
    }

    report.note(format!(
        "the ladder the application actually climbed, read from its own `{STATUS}` lines and \
         windowed by rung: [{}]",
        series.join(" -> ")
    ));
    report.note(format!(
        "each rung ordered a raster in its own window — the `{SPAWN}` scales were [{}] — which \
         is reported and not asserted beyond presence: a spawn is an ORDER to rasterize and not \
         a completed one",
        rungs
            .iter()
            .enumerate()
            .map(|(position, rung)| {
                let to = rungs.get(position + 1).map_or(usize::MAX, |n| n.lineno);
                let scales = spawns_within(trace, rung.lineno, to);
                format!("{:?}", scales)
            })
            .collect::<Vec<_>>()
            .join(", ")
    ));

    Ok(None)
}

// ---------------------------------------------------------------------------
// launching and waiting
// ---------------------------------------------------------------------------

fn resolve_exe(ctx: &CheckContext) -> Result<PathBuf> {
    ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })
}

fn repo_fixture(name: &str) -> Result<PathBuf> {
    crate::checks::driving::repo_fixture(
        name,
        "This check pins its own document and ignores --pdf: the ladder's rungs have to stay \
         under the raster ceiling for a rung's silence to mean what this check reads it as.",
    )
}

/// A launch with the diagnostic channel on, the window off the desktop, the
/// chord list in the environment, and no OS input ever sent to it.
fn launch_scripted(ctx: &CheckContext, exe: &Path, list: &[&str]) -> Result<Session> {
    let mut spec = LaunchSpec::new(exe, ctx.out("scripted_keys.trace.txt"));
    spec.pdf = Some(repo_fixture(FIXTURE)?);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env.push((KEYS_ENV.to_owned(), list.join(",")));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    if let Some(name) = ctx.profile.viewport_env {
        spec.env.push((name.to_owned(), OFFSCREEN.to_owned()));
    }
    // ★ Without this the line above is decoration: `Session::place` would move
    // the window to `(780, 40)` the moment it appeared, onto the desktop the
    // operator is using — and an on-screen window would also be taking his
    // keystrokes, which would make this check's subject ambiguous.
    spec.place = false;
    Session::launch(&spec, ctx.profile.trace_prefix)
}

/// Wait until the seam has delivered the last chord, then let its effect land.
///
/// ★ Not [`Session::settle`]. That polls the frame counter, and this
/// application stops drawing the moment the seam stops asking it to — so a
/// settle asked for more frames than the run has left would spin out its whole
/// cap on every green run. The thing being waited for is a trace line, so the
/// trace line is what is waited for.
fn wait_for_rung(session: &Session, index: usize) -> Result<Trace> {
    let deadline = std::time::Instant::now() + RUNG_DEADLINE;
    loop {
        let trace = session.trace()?;
        let arrived = trace
            .events(KEYS)
            .filter_map(|l| l.get_usize("index"))
            .any(|seen| seen == index);
        if arrived {
            // One short wait, so the rung's own status line and raster order
            // are in the capture before it is read. The seam paces twenty
            // frames apart, so this is the tail of the run and nothing else is
            // still to come.
            session.settle(10);
            return session.trace();
        }
        if std::time::Instant::now() >= deadline {
            let traced: Vec<String> = trace.events(KEYS).map(|l| l.raw.clone()).collect();
            return Err(Error::new(format!(
                "the seam had not delivered rung {index} after {} seconds. ★ The most likely \
                 cause is that it stopped keeping the application awake: it paces on \
                 `ctx.cumulative_pass_nr()`, and a quiescent viewer requests no repaint, so a \
                 gate on a future pass number never opens unless the seam calls \
                 `request_repaint` itself while chords remain. Traced so far: [{}]",
                RUNG_DEADLINE.as_secs(),
                traced.join(" | ")
            )));
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
