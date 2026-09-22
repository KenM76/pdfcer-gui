//! `the_off_page_halo_never_costs_the_operator_his_zoom` — **O218/O221's
//! mechanism, driven.**
//!
//! # The report
//!
//! > *"Sometimes when I zoom in I still get the error 'This page could not be
//! > drawn. This zoom is further in than pdfcer can rasterize…' instead of the
//! > rasterizer just stopping at the last zoom level that it accomplished. …
//! > Zooming capability seems to be affected by the number of pdfs I have open
//! > … I think, but I could be wrong and it could be a bit random."*
//!
//! It was not the document count and it was not random. It was **off-page
//! content**, which is common in CAD exports and rare elsewhere, and which the
//! shell displays by default in Review and Edit and not in Read.
//!
//! # ★★★ A region is not automatically small
//!
//! Two different rectangles reach the render path wearing one type, and they
//! behave **oppositely** as the operator zooms:
//!
//! | region | device size as zoom rises |
//! |---|---|
//! | the visible-rect tier's box | **constant** — it is a multiple of the WINDOW |
//! | the off-page halo box | **grows with the zoom**, exactly as the sheet does |
//!
//! `OpenDoc::raster_order_fillable` treated *any* present region as fillable,
//! which is true of the first row and false of the second. The halo box is by
//! construction **larger** than the sheet, so it crosses the rasterizer's
//! `MAX_PIXMAP_EDGE` **first** — on this fixture at about half the zoom the
//! sheet does. Add the one-frame skew (the canvas picks the region at frame
//! N's scale; the settle step orders it at frame N's *new* scale) and a halo
//! validated at one rung is ordered at the next, comes back `BadRasterSize`,
//! and `absorb_render` turns that refusal into the document's **permanent**
//! zoom ceiling.
//!
//! Measured on `fixtures/off-page-object.pdf`, same recipe, two builds:
//!
//! | build | refusal | ceiling learned | where the zoom stopped |
//! |---|---|---|---|
//! | before | `px=23040x12800 scale=64.0 region=1` | `scale=48.0` | **4,800 %** |
//! | after | none | none | **1,677,721,600 %** |
//!
//! # Why this check is headless, and why it is in Edit mode
//!
//! The window is placed off the desktop and no OS input is ever sent to it, so
//! it can run while the operator is working — the zoom verbs arrive through
//! the scripted-keystroke seam instead. `PDFCER_DIAG_INVOKE=mode.edit` is
//! **load-bearing, not decoration**: `OffPagePrefs::default_for_mode` turns
//! off-page display OFF in Read, and a Read-mode run never enters the halo
//! tier at all. A whole eight-rung ladder was once measured in Read mode and
//! recorded a real, precise zero — of the wrong variable.
//!
//! # ★★ The controls, and why they carry no engine constant
//!
//! `tools/ui-verify` has one dependency and cannot import
//! `pdfcer_render::MAX_PIXMAP_EDGE`; a harness constant *naming* an engine
//! constant would be a copy that decays the day the engine's moves. So both
//! controls are **relational**, read from the application's own trace:
//!
//! 1. **The halo box is bigger than the sheet.** Its `box=` against the
//!    `crop=` the same run publishes. That is the whole premise — if the halo
//!    were the smaller rectangle there would be no defect to check.
//! 2. **The application abandoned the halo tier while off-page display was
//!    still on.** A `tier=halo` line followed by a `tier=whole` one is the
//!    application *saying* it crossed the halo's own wall. That is the point
//!    the pre-fix build died at, so a run that never reaches it has not
//!    entered the window where the defect lives and this check would be inert.
//!
//! Both are reported as harness findings rather than as failures: a run that
//! never got into the defect's window has learned nothing about the defect.
//!
//! # The three failures, in causal order
//!
//! 1. **An order the shell could not fill** — any `bad-raster-size`. The root
//!    cause; the message names whether it was a region order or a whole-page
//!    one, because those have different owners.
//! 2. **A permanent ceiling learned** — any `raster-ceiling-learned`. The
//!    effect: the document is pinned for as long as it stays open.
//! 3. **The zoom stopped climbing the moment the halo stopped fitting** — the
//!    operator's own symptom. The last zoom of the run must be strictly above
//!    the zoom the application was at when it abandoned the halo tier. Before
//!    the fix those two figures were the *same number*: fourteen further
//!    chords changed nothing.
//!
//! # Every way this reports SKIP
//!
//! No binary, no diagnostic channel, the seam not delivering its last chord,
//! no document opened, off-page display not coming up, no halo tier entered,
//! the halo box not exceeding the sheet, or the climb never reaching the wall.
//! **Not** a refusal, **not** a learned ceiling and **not** a stalled zoom —
//! those three are the failures this check exists for.
//!
//! # ⚠ What has been falsified, and the one thing that has not
//!
//! A red run proves the check; it does not prove the check's *parts*, because
//! the first arm to fire is the only one that ran. Each was therefore driven on
//! its own, against real evidence rather than a plant:
//!
//! | arm | how it was made to fire | what it said |
//! |---|---|---|
//! | refusal | the pre-fix binary | `px=23040x12800 scale=64.0 region=1` |
//! | ceiling | the pre-fix binary, this arm moved first | `scale=48.0 zoom=64.00 to=48.00` |
//! | stalled zoom | the pre-fix binary, this arm moved first | abandoned at 4,800 %, ended at 4,800 %, 0 further figures |
//! | off-page display off | `mode.read` instead of `mode.edit` | no `offpage=on` on any frame |
//! | no halo tier | `four-pages.pdf`, which has nothing off its sheet | `[tier=whole offpage=on, tier=whole offpage=on]` |
//! | wall never crossed | four rungs instead of twenty-four | scales ordered `[3.32 3.31 4 6 8 16]` |
//!
//! **Not falsified: the halo-bigger-than-the-sheet control.** No run reaches
//! it, because `render::halo::region` only returns a union once the content's
//! reach passes `OVERHANG_TOLERANCE_PTS` beyond the crop box, so a halo that is
//! not the bigger rectangle is not a state the application can be driven into.
//! It is a guard against that invariant changing, not an instrument arm, and it
//! is recorded here as one so a later reader does not quote it as evidence.

use std::path::{Path, PathBuf};

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// 200 × 200 pt, one square on the sheet and one square 160 pt off its left
/// edge. The halo union is therefore 360 pt wide against a 200 pt sheet — the
/// 1.8 : 1 ratio that puts the halo's wall at a little over half the sheet's.
///
/// ⚠ Pinned, and `--pdf` is ignored. A document with no off-page content
/// never enters the halo tier, so this check's entire subject would be absent
/// and its controls would turn every run into a SKIP.
const FIXTURE: &str = "off-page-object.pdf";

/// The seam's environment variable.
const KEYS_ENV: &str = "PDFCER_DIAG_KEYS";

/// The command seam's environment variable.
const INVOKE_ENV: &str = "PDFCER_DIAG_INVOKE";

/// ★ Load-bearing. See the module header: off-page display is off in Read, so
/// a Read-mode run cannot reach the tier this check is about.
const INVOKE: &str = "mode.edit";

/// Zoom in one rung, in `app::keyboard::parse_chord`'s grammar.
const ZOOM_IN: &str = "Ctrl++";

/// How many rungs the ladder climbs.
///
/// The fixture's halo wall sits about four rungs above its fit zoom and its
/// sheet's wall about one rung above that, so a dozen would prove the fix.
/// Twice that is deliberate: the rungs past the wall are the ones that were
/// silently doing nothing, and a check that turned round just after the
/// boundary would pass on a build that recovered for one rung and stalled
/// again.
const CLIMB: usize = 24;

/// The application's view-state line, carrying `zoom=`.
const STATUS: &str = "status";

/// The canvas' per-frame line, carrying `crop=` — the page's own box, which is
/// what the halo box is measured against.
const CANVAS: &str = "canvas";

/// The seam's own line, carrying `index=`.
const KEYS: &str = "diag-keys";

/// The canvas' tier decision, carrying `tier=`, `offpage=` and `box=`.
const HALO: &str = "canvas-halo";

/// The preference seam's line, carrying `mode=` and `on=`.
///
/// ⚠ **A one-shot statement about start-up, not about the run.** It is emitted
/// while the document is being opened, which is BEFORE a command from
/// [`INVOKE_ENV`] has fired — so on a fresh profile it says `mode=read on=false`
/// on a run that then enters Edit and displays off-page content perfectly well.
/// Read as the current state it makes a working run look inert, which is
/// exactly what it did here once. The authoritative reading is the `offpage=`
/// field on the canvas' own per-frame [`HALO`] line, because that is the one
/// the canvas acted on. This constant survives only to explain a genuinely
/// off run.
const SEED: &str = "off-page-seed";

/// The mode seam's line, carrying `to=`. Reported, not asserted — [`HALO`]'s
/// `offpage=` field is the reading that matters and this only says why.
const MODE: &str = "mode-changed";

/// A raster order, carrying `scale=`. Reported, never asserted beyond
/// presence: a spawn is an ORDER to rasterize and not a completed one.
const SPAWN: &str = "render-spawn";

/// The worker refusing an order it could not allocate, carrying `px=`,
/// `scale=` and `region=`.
const BAD_RASTER: &str = "bad-raster-size";

/// The shell turning a refusal into the document's zoom ceiling.
const CEILING: &str = "raster-ceiling-learned";

/// Off the desktop, at the harness's usual size.
const OFFSCREEN: &str = "-4200,-4200,1400,900";

/// How long to wait for the last rung.
///
/// Generous on purpose. The seam paces its chords twenty frames apart and the
/// last rungs of this ladder are drawn at magnifications where a frame is not
/// free; the whole run measured about forty seconds on an idle machine.
/// Exceeding this is reported as a finding rather than as a hang.
const RUNG_DEADLINE: std::time::Duration = std::time::Duration::from_secs(180);

/// See the module documentation.
pub struct TheOffPageHaloNeverCostsTheOperatorHisZoom;

impl Check for TheOffPageHaloNeverCostsTheOperatorHisZoom {
    fn name(&self) -> &'static str {
        "the_off_page_halo_never_costs_the_operator_his_zoom"
    }

    fn defect(&self) -> &'static str {
        "a page with content off the sheet loses its zoom permanently. The off-page halo box is \
         larger than the sheet, so it crosses the rasterizer's pixmap limit first; an order \
         placed for it past that limit is refused, and the shell turns the refusal into the \
         document's zoom ceiling. The operator sees `This zoom is further in than pdfcer can \
         rasterize` at a fraction of the magnification the same drawing reaches in Read mode, \
         and it never comes back while the document is open"
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
    let session = launch_scripted(ctx, &exe)?;
    report.note(format!(
        "launched {} on fixtures/{FIXTURE} as pid {} with {INVOKE_ENV}={INVOKE} and \
         {CLIMB} × `{ZOOM_IN}`, the window off the desktop and NO OS input sent",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());

    let trace = wait_for_rung(&session, CLIMB - 1)?;

    preconditions(&trace)?;
    report.note(format!(
        "the application's own account of the mode it ran in and what it seeded off-page \
         display to: {} | {}",
        trace
            .last(MODE)
            .map_or_else(|| format!("no `{MODE}`"), |l| l.raw.clone()),
        trace
            .last(SEED)
            .map_or_else(|| format!("no `{SEED}`"), |l| l.raw.clone()),
    ));
    let wall = controls(&trace, report)?;

    if let Some(failure) = check_no_refusal(&trace) {
        return Ok(Some(failure));
    }
    if let Some(failure) = check_no_ceiling(&trace) {
        return Ok(Some(failure));
    }
    check_the_climb_continued(&trace, &wall, report)
}

// ---------------------------------------------------------------------------
// preconditions — the run is about the thing this check names
// ---------------------------------------------------------------------------

/// A document is open and off-page display came up.
///
/// ★ Both are reported as harness findings. This project has twice produced
/// confident, detailed and entirely wrong defect reports out of a reading
/// taken on a run where the subject was never present.
fn preconditions(trace: &Trace) -> Result<()> {
    if trace.last(STATUS).is_none() {
        return Err(Error::new(format!(
            "the application launched and never traced a `{STATUS}` line, so it opened no \
             document. Any verdict below would be about an empty shell. Check that \
             fixtures/{FIXTURE} exists and that an absolute path was passed."
        )));
    }

    // ★ Read from the canvas' own tier line and NOT from `off-page-seed` —
    // see that constant. The seed is taken while the document opens, which is
    // before a command from the environment has fired, so on a fresh profile
    // it reports Read and `on=false` on a run that goes on to enter Edit and
    // display off-page content for every frame that matters.
    if !trace.events(HALO).any(|l| l.get("offpage") == Some("on")) {
        let seed = trace.last(SEED).map_or_else(
            || format!("no `{SEED}` line was traced at all"),
            |l| format!("`{}`", l.raw),
        );
        let mode = trace.last(MODE).map_or_else(
            || format!("no `{MODE}` line was traced at all"),
            |l| format!("`{}`", l.raw),
        );
        return Err(Error::new(format!(
            "off-page display was off for every frame of the run: no `{HALO}` line carries \
             `offpage=on`. {seed}; {mode}. `OffPagePrefs::default_for_mode` returns false for \
             Read mode, so either {INVOKE_ENV}={INVOKE} did not land or the preference \
             remembered from a previous run is off. Either way the halo tier is unreachable \
             and this run measures nothing."
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// controls — the run got into the window where the defect lives
// ---------------------------------------------------------------------------

/// Where the application abandoned the halo tier, and what it was at when it
/// did.
struct Wall {
    /// The halo box's longest edge, in page points.
    halo_pts: f64,
    /// The sheet's longest edge, in page points.
    sheet_pts: f64,
    /// Where the `tier=whole` line that follows the halo sits in the capture.
    lineno: usize,
    /// The last zoom the application published at or before that line.
    zoom: f32,
}

/// ★★ Both controls, and both relational — no engine constant appears here.
/// See the module header for why.
fn controls(trace: &Trace, report: &mut CheckReport) -> Result<Wall> {
    let sheet_pts = trace
        .events(CANVAS)
        .filter_map(|l| l.get_f64_list("crop"))
        .filter(|c| c.len() == 4)
        .map(|c| (c[2] - c[0]).max(c[3] - c[1]))
        .find(|longest| longest.is_finite() && *longest > 0.0)
        .ok_or_else(|| {
            Error::new(format!(
                "no `{CANVAS}` line carried a usable `crop=`, so the sheet's own size is \
                 unknown and the halo box cannot be compared against it. That comparison is \
                 this check's premise."
            ))
        })?;

    let halo = trace
        .events(HALO)
        .find(|l| l.get("tier") == Some("halo") && l.get("offpage") == Some("on"))
        .ok_or_else(|| {
            let tiers: Vec<String> = trace
                .events(HALO)
                .map(|l| {
                    format!(
                        "tier={} offpage={}",
                        l.get("tier").unwrap_or_default(),
                        l.get("offpage").unwrap_or_default()
                    )
                })
                .collect();
            Error::new(format!(
                "the canvas never chose the halo tier with off-page display on, so the \
                 rectangle this check is about was never built. Tiers traced: [{}]",
                tiers.join(", ")
            ))
        })?;

    let box_of = halo.get_f64_list("box").filter(|b| b.len() == 4);
    let Some(b) = box_of else {
        return Err(Error::new(format!(
            "the `{HALO} tier=halo` line carried no readable `box=`: {}. The halo rectangle's \
             size is the premise of every reading below.",
            halo.raw
        )));
    };
    let halo_pts = (b[2] - b[0]).max(b[3] - b[1]);

    // Control 1. If the halo were the SMALLER rectangle there would be no
    // defect here to check — it would reach the rasterizer's limit after the
    // sheet did, and the whole-page tier would have refused first.
    if halo_pts <= sheet_pts {
        return Err(Error::new(format!(
            "the halo box's longest edge is {halo_pts:.1} pt against a sheet of \
             {sheet_pts:.1} pt, so it is not the bigger rectangle and cannot reach the \
             rasterizer's limit before the sheet does. This check's premise does not hold on \
             this document — its off-page content is inside the sheet's own extent."
        )));
    }

    // Control 2. The application saying, in its own trace, that the halo
    // stopped fitting: it went back to the whole sheet while off-page display
    // was still on. A run that never reaches this line never entered the
    // window the defect lives in.
    let abandoned = trace
        .events(HALO)
        .find(|l| {
            l.lineno > halo.lineno
                && l.get("tier") == Some("whole")
                && l.get("offpage") == Some("on")
        })
        .ok_or_else(|| {
            Error::new(format!(
                "the canvas entered the halo tier and never left it in {CLIMB} rungs, so the \
                 climb never crossed the halo box's own wall and this run is inert. The halo \
                 box is {halo_pts:.1} pt across; it stops fitting once that times the raster \
                 scale passes the rasterizer's pixmap limit. Raster scales ordered: [{}]",
                spawn_scales(trace)
                    .iter()
                    .map(|s| format!("{s}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            ))
        })?;

    let zoom = zoom_at_or_before(trace, abandoned.lineno).ok_or_else(|| {
        Error::new(format!(
            "the application abandoned the halo tier without ever having published a `{STATUS}` \
             line, so there is no zoom to compare the end of the run against."
        ))
    })?;

    report.note(format!(
        "★ control: the halo box is {halo_pts:.1} pt across against a {sheet_pts:.1} pt sheet \
         — {:.2}× — so it reaches the rasterizer's pixmap limit first, which is this check's \
         premise. The application then abandoned the halo tier at zoom={zoom} with off-page \
         display still on, which is it saying that limit was crossed. That is the point the \
         pre-fix build stopped at.",
        halo_pts / sheet_pts
    ));

    Ok(Wall {
        halo_pts,
        sheet_pts,
        lineno: abandoned.lineno,
        zoom,
    })
}

// ---------------------------------------------------------------------------
// the three failures
// ---------------------------------------------------------------------------

/// **The root cause** — the shell placed an order the rasterizer could not
/// fill.
fn check_no_refusal(trace: &Trace) -> Option<String> {
    let refusal = trace.events(BAD_RASTER).next()?;
    let region = refusal.get("region").unwrap_or_default();
    // ★ The two have different owners and a reader must not be sent to the
    // wrong one. `region=1` is this check's defect; `region=0` is the
    // whole-page wall, which `raster_wall` owns.
    let whose = if region == "1" {
        "a REGION order, which is this check's defect: `OpenDoc::raster_order_fillable` waved \
         the region through because a region was present, and the off-page halo box is a \
         region that GROWS with the zoom. `render::strategy::region_raster_fits` is the \
         predicate that must be consulted before the order is placed"
    } else {
        "a WHOLE-PAGE order, which is a different defect with a different owner — the sheet \
         itself outgrew the rasterizer and the region tier should have taken over before it \
         did. See the `raster_wall` check"
    };
    Some(format!(
        "the shell placed a raster order that could not be allocated: `{}`. {whose}. ★ The \
         operator's report of this is *\"sometimes when I zoom in I still get the error … \
         instead of the rasterizer just stopping at the last zoom level that it accomplished\"*.",
        refusal.raw
    ))
}

/// **The effect** — the document is pinned for as long as it stays open.
fn check_no_ceiling(trace: &Trace) -> Option<String> {
    let learned = trace.events(CEILING).next()?;
    Some(format!(
        "the shell learned a permanent zoom ceiling for this document: `{}`. ⚠ `zoom=` and \
         `to=` on that line are raster SCALES, not percentages — multiply by 100 for the figure \
         the status bar shows. A ceiling is the right answer to a sheet that genuinely cannot \
         be rasterized any further; it is the wrong answer to an order the shell should never \
         have placed, because it outlives the order and pins the document at a fraction of the \
         magnification the same drawing reaches with off-page display off.",
        learned.raw
    ))
}

/// **The symptom, in his words** — zooming stopped working.
fn check_the_climb_continued(
    trace: &Trace,
    wall: &Wall,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let last = trace
        .events(STATUS)
        .filter_map(|l| l.get_f32("zoom"))
        .last()
        .ok_or_else(|| {
            Error::new(format!(
                "the run traced no final `{STATUS}` line, so where the zoom ended is unknown."
            ))
        })?;

    let after: Vec<f32> = trace
        .events(STATUS)
        .filter(|l| l.lineno > wall.lineno)
        .filter_map(|l| l.get_f32("zoom"))
        .collect();
    let distinct = {
        let mut seen: Vec<f32> = Vec::new();
        for z in &after {
            if !seen.iter().any(|s| (s - z).abs() < f32::EPSILON) {
                seen.push(*z);
            }
        }
        seen.len()
    };

    // ⚠ The chords delivered AFTER the wall, not the whole ladder. The
    // difference is the whole sentence: "24 chords were delivered" is true of
    // every run and says nothing, while "14 chords were delivered after the
    // halo stopped fitting and the number never moved again" is the defect.
    let chords_after = trace
        .events(KEYS)
        .filter(|l| l.lineno > wall.lineno)
        .count();
    if last <= wall.zoom {
        return Ok(Some(format!(
            "the zoom stopped climbing the moment the halo box stopped fitting. The \
             application abandoned the halo tier at zoom={} and the run ended at zoom={last}, \
             with {chords_after} further `{ZOOM_IN}` chord(s) delivered after that point and \
             {distinct} distinct zoom figure(s) traced. ★ This is the operator's own symptom \
             — the halo box is {:.1} pt against a {:.1} pt sheet, so it reaches the \
             rasterizer's limit first, and the refusal it earns is absorbed as the DOCUMENT's \
             ceiling rather than declined as an order that should not have been placed. The \
             same drawing with off-page display off climbs on.",
            wall.zoom, wall.halo_pts, wall.sheet_pts
        )));
    }

    let scales = spawn_scales(trace);
    report.note(format!(
        "the climb continued past the halo's wall: zoom={} where the halo was abandoned, \
         zoom={last} at the end of the run — {:.0}× further in, across {distinct} distinct \
         zoom figures. Raster scales ordered, highest last: [{}]",
        wall.zoom,
        f64::from(last) / f64::from(wall.zoom),
        scales
            .iter()
            .rev()
            .take(4)
            .rev()
            .map(|s| format!("{s}"))
            .collect::<Vec<_>>()
            .join(" ")
    ));
    report.note(format!(
        "no `{BAD_RASTER}` and no `{CEILING}` were traced in the whole run, which is the \
         assertion: every order the shell placed could be filled, so nothing was absorbed as a \
         ceiling and nothing was drawn from a refusal"
    ));

    Ok(None)
}

// ---------------------------------------------------------------------------
// reading the trace
// ---------------------------------------------------------------------------

/// The last `status … zoom=` at or before a line.
fn zoom_at_or_before(trace: &Trace, lineno: usize) -> Option<f32> {
    trace
        .events(STATUS)
        .filter(|l| l.lineno <= lineno)
        .filter_map(|l| l.get_f32("zoom"))
        .last()
}

/// Every raster order's `scale=`, in the order they were placed.
fn spawn_scales(trace: &Trace) -> Vec<f32> {
    trace
        .events(SPAWN)
        .filter_map(|l| l.get_f32("scale"))
        .collect()
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
        "This check pins its own document and ignores --pdf: a document with no content off \
         the sheet never enters the halo tier, so the defect's own rectangle would not exist.",
    )
}

/// A launch with the diagnostic channel on, Edit mode asked for, the window off
/// the desktop, the chord list in the environment, and no OS input ever sent.
fn launch_scripted(ctx: &CheckContext, exe: &Path) -> Result<Session> {
    let chords = vec![ZOOM_IN; CLIMB].join(",");
    let mut spec = LaunchSpec::new(exe, ctx.out("off_page_ceiling.trace.txt"));
    spec.pdf = Some(repo_fixture(FIXTURE)?);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env.push((INVOKE_ENV.to_owned(), INVOKE.to_owned()));
    spec.env.push((KEYS_ENV.to_owned(), chords));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    if let Some(name) = ctx.profile.viewport_env {
        spec.env.push((name.to_owned(), OFFSCREEN.to_owned()));
    }
    // ★ Without this the line above is decoration: `Session::place` would move
    // the window onto the desktop the operator is using, where it would also
    // take his keystrokes.
    spec.place = false;
    Session::launch(&spec, ctx.profile.trace_prefix)
}

/// Wait until the seam has delivered the last chord, then let its effect land.
///
/// ★ Not [`Session::settle`] as the primary wait: that polls the frame counter,
/// and this application stops drawing the moment the seam stops asking it to.
/// The thing being waited for is a trace line, so the trace line is what is
/// waited for.
fn wait_for_rung(session: &Session, index: usize) -> Result<Trace> {
    let deadline = std::time::Instant::now() + RUNG_DEADLINE;
    loop {
        let trace = session.trace()?;
        let arrived = trace
            .events(KEYS)
            .filter_map(|l| l.get_usize("index"))
            .any(|seen| seen == index);
        if arrived {
            // ⚠ Longer than the sibling ladder's settle, and for a measured
            // reason: the last rungs here are drawn at magnifications where a
            // frame is not free, and the `status`, `canvas-halo` and
            // `render-spawn` lines this check reads are published across
            // several frames after the chord lands.
            session.settle(20);
            return session.trace();
        }
        if std::time::Instant::now() >= deadline {
            let traced = trace.events(KEYS).count();
            return Err(Error::new(format!(
                "the seam had delivered {traced} of {CLIMB} chords after {} seconds. The seam \
                 paces on `ctx.cumulative_pass_nr()` and keeps the application awake while \
                 chords remain; a stall here is either that pacing failing or frames at the \
                 top of this ladder costing more than this deadline allows.",
                RUNG_DEADLINE.as_secs()
            )));
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
