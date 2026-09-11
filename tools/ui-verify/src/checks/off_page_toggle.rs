//! `the_off_page_toggle_is_per_mode_and_remembered` — **the operator's switch
//! for off-sheet content, driven five times in one profile.**
//!
//! # The report
//!
//! Ken, 2026-09-11, `OPERATOR_REQUESTS.md` O175:
//!
//! > *"in our view ribbon area we need an option to show the stuff that is off
//! > page or not (and when not showing the stuff that is off page there
//! > shouldn't be a gap between pages where the stuff is, so it just goes back
//! > to looking before we added the view things that are off the page
//! > feature). by default, read doesn't show off page items, review and edit
//! > do show off page items. these settings can be changed by the user and
//! > their preference is remembered for each read review edit modes."*
//!
//! Six requirements in one paragraph, and **four of them are about state that
//! survives a process**: the per-mode default, the operator's own answer, that
//! the answer is remembered, and that it is remembered *separately per mode*.
//! None of those can be observed in a single launch, and none of them can be
//! observed by a unit test at all — the store, the seed at document open, the
//! re-seed at mode change and the toggle's write are four different files, and
//! a test of any one of them is green while the chain is broken.
//!
//! ⇒ This check is a **ladder of five launches against one profile
//! directory**, which is the only arrangement in which "remembered" means
//! anything. `sandbox` gives each check a private copy of the binary and
//! therefore a private `userdata/`; the five launches below share it, in
//! order, and the later rungs read what the earlier ones wrote.
//!
//! | # | mode | what is invoked | what must be true | which requirement |
//! |---|---|---|---|---|
//! | 1 | Read | nothing but the view | off-page content is **hidden** | the Read default |
//! | 2 | Read | `view.off_page` | it **appears** | the toggle works, from the command the ribbon item raises |
//! | 3 | Read | nothing | it is **still there** | the answer survived the process |
//! | 4 | Edit | `view.off_page` | it was **on before the click** and **off after** | the Edit default, and the toggle in a second mode |
//! | 5 | Read | nothing but the mode | it is **on** | Edit's answer did not touch Read's |
//!
//! Rung 5 is the one that would be missing from a hand-written test of this
//! feature and is the one the operator asked for in as many words: *"remembered
//! for each read review edit modes"*. A single global flag passes rungs 1–4
//! and fails rung 5, and a single global flag is exactly what a first
//! implementation of this reaches for.
//!
//! # ★★★ Why each rung asserts BOTH a trace line and pixels
//!
//! `off_page_visible`'s header carries the argument and it is unchanged here:
//! *layout and clipping defects have exactly one oracle, and it is a rendered
//! screenshot* (`D:/dev/rag/egui/`). The preference chain, though, is
//! invisible in a screenshot — a hidden object and a *missing* object look the
//! same. So each rung reads two independent things:
//!
//! 1. **The decision**, from the trace: `off-page-seed` (document open),
//!    `off-page-mode` (mode change) or `off-page-remembered` (the toggle),
//!    each naming the mode and the answer. This is what says the answer came
//!    from the *preference* rather than from luck.
//! 2. **The consequence**, from the capture: ink, or no ink, at the centre of
//!    a square that lies entirely off the left edge of the sheet.
//!
//! Neither alone is enough, and the failure they catch is different in each
//! direction. A trace-only check passes on a build that resolves the
//! preference correctly and then ignores it — which is the whole of the defect
//! for an operator. A pixel-only check cannot tell "Read hides it" from "the
//! renderer broke", and the report would send the next session into the
//! engine.
//!
//! # ★★ The control that makes a negative rung mean anything
//!
//! Rungs 1 and 4 assert an **absence**, and this suite's memory is explicit
//! that an absence assertion is worth exactly as much as its control: *"a
//! uniform failure at every rung of a sweep is about the probe."* A window
//! that never opened, a capture of the wrong monitor, a document that failed
//! to render — all of them produce a clean patch where the off-page square
//! should be, and all of them would read as a pass.
//!
//! So **every** rung, positive and negative, also measures a patch at the
//! centre of the fixture's **on-page** square, and that patch must be ink. If
//! it is not, the check reports a harness finding and refuses to say anything
//! about the off-page patch — including that it was clean.
//!
//! # ★★★ The gap, which is the operator's OTHER sentence
//!
//! > *"when not showing the stuff that is off page there shouldn't be a gap
//! > between pages where the stuff is, so it just goes back to looking before
//! > we added the view things that are off the page feature."*
//!
//! That is a second consequence, not a restatement of the first, and it is
//! produced by a **different function**: `canvas::tier::decide` widens the
//! raster, `canvas::tier::overhang` widens the layout. A check that watched
//! only the ink would report green on a build that hid the off-sheet square
//! and left the band of grey standing exactly where he said it must not be.
//!
//! So every rung also reads `canvas-pasteboard`, which carries the overhang
//! the layout was actually given, and asserts it: **exactly zero** with the
//! switch off, **strictly positive** with it on. Zero rather than *small*
//! because `overhang` returns early rather than multiplying a measured box by
//! nothing — so there is no rounding to tolerate, and a tolerance here would
//! be a place for a one-pixel band to hide.
//!
//! The fixture is one page, so there is no *between-pages* gap to photograph;
//! what is measured is the reach that creates it, which is the same number for
//! one sheet as for fifty.
//!
//! ⚠ The overhang is in **screen** points, so it only compares across launches
//! while the zoom is held — which `view.zoom_actual` does on every rung. The
//! assertions below are against zero and against zero, which is why that is a
//! footnote rather than a hazard.
//!
//! # Every way this reports SKIP
//!
//! No binary, no diagnostic channel, no `canvas-viewport` region, not enough
//! grey on screen to reach x = −100 at 100 % zoom, or a capture that could not
//! be taken. **Not** any of the five rungs' own assertions: each of those is a
//! failure, and they are the reason this file exists.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::{LRect, PixRect, Pt};
use crate::image::Image;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The tier trace, which also carries the switch's own state.
const HALO: &str = "canvas-halo"; // ui-text-exempt: a trace event name, never displayed

/// The tier that means *the raster was widened to hold off-sheet ink*.
const TIER_HALO: &str = "halo"; // ui-text-exempt: a trace field value, never displayed

/// The LAYOUT half of the switch — how far the canvas reaches past the
/// sheets. The operator's *"there shouldn't be a gap"*, in a number.
const PASTEBOARD: &str = "canvas-pasteboard"; // ui-text-exempt: a trace event name

/// The answer resolved when a document is opened — `app::lifecycle`.
const SEED: &str = "off-page-seed"; // ui-text-exempt: a trace event name

/// The answer applied when the ribbon mode changes — `prefs::offpage::apply_mode`.
const MODE: &str = "off-page-mode"; // ui-text-exempt: a trace event name

/// The answer written when the operator uses the toggle — `prefs::offpage::remember`.
const REMEMBERED: &str = "off-page-remembered"; // ui-text-exempt: a trace event name

/// The region that says a sheet is on screen at all.
const PAGE_REGION: &str = "page"; // ui-text-exempt: a trace region name

/// The scroll area the sheet is drawn inside — the bound an off-page point is
/// converted against.
const VIEWPORT_REGION: &str = "canvas-viewport"; // ui-text-exempt: a trace region name

/// The fixture, shared with the three sibling off-page checks.
const FIXTURE: &str = "fixtures/off-page-object.pdf";

/// Its page box. Transcribed from the file and asserted in this module's tests.
const FIXTURE_PAGE: PageGeometry = PageGeometry {
    width_pt: 200.0,
    height_pt: 200.0,
};

/// **The centre of the square that lies entirely off the sheet.** Ink here
/// when the switch is on; paper when it is off.
const OFF_SHEET_AT: (f64, f64) = (-100.0, 120.0);

/// **The centre of the square that is ON the sheet** — the control, and it is
/// ink on every rung. See the header: without it, a negative rung passes on a
/// window that never drew anything.
const ON_SHEET_AT: (f64, f64) = (70.0, 70.0);

/// Half the side of a sampled patch, in page points.
const PATCH_PT: f64 = 8.0;

/// At or below this on all three channels is ink. The fixture fills with
/// `0 0 0 rg`; the threshold is loose enough for antialiasing and far from any
/// grey the pasteboard is painted in.
const INK: u8 = 96;

/// A patch that should be filled must be at least this dark.
const INK_FRACTION: f64 = 0.90;

/// A patch that should be empty must be at most this dark.
const PAPER_INK_FRACTION: f64 = 0.05;

/// Single-page display, then 100 %.
///
/// The same pair the sibling checks use and for the reason `off_page_visible`
/// measured: fit-page on a 200 × 200 fixture puts x = −100 outside the
/// viewport, the conversion refuses — correctly — and the check SKIPS, which
/// is not red.
///
/// ★ `mode.read` is named EXPLICITLY on every Read rung rather than relied on
/// as the default, because the profile remembers the mode it was last in.
/// Rung 5 follows a rung that ended in Edit, and a rung that assumed Read
/// because the first launch was in Read would be reading Edit's answer while
/// reporting Read's.
const READ: &str = "mode.read,view.page_single,view.zoom_actual";

/// Read, and then the operator's toggle.
const READ_TOGGLED: &str = "mode.read,view.page_single,view.zoom_actual,view.off_page";

/// Edit, and then the operator's toggle.
const EDIT_TOGGLED: &str = "mode.edit,view.page_single,view.zoom_actual,view.off_page";

/// One rung of the ladder.
struct Rung {
    /// Names the rung in notes and artefact filenames.
    label: &'static str,
    /// What `PDFCER_DIAG_INVOKE` carries.
    invoke: &'static str,
    /// The trace event that must state this rung's answer, and the mode it
    /// must name. Which event it is *is* the assertion: a rung that expects
    /// `off-page-seed` is asserting that the answer came from the STORE at
    /// document open, and one that expects `off-page-mode` is asserting it
    /// came from the mode change.
    event: (&'static str, &'static str),
    /// What the answer must be — and therefore whether the off-sheet square
    /// must be painted.
    want_on: bool,
    /// An earlier line in the same run that must also hold, when the rung has
    /// two things to say. Rung 4 uses it to assert Edit's *default* before the
    /// toggle that changes it.
    before: Option<(&'static str, &'static str, bool)>,
    /// What a failure of this rung means, in the operator's terms. Written
    /// into the failure so the report names the requirement rather than the
    /// mechanism.
    means: &'static str,
}

/// The ladder. Order is load-bearing — see the table in the module header.
const LADDER: &[Rung] = &[
    Rung {
        label: "read-default",
        invoke: READ,
        event: (SEED, "read"),
        want_on: false,
        before: None,
        means: "Read is supposed to open with off-page content HIDDEN, and this build showed it. \
                That is the default the operator named first: `by default, read doesn't show off \
                page items`",
    },
    Rung {
        label: "read-toggled-on",
        invoke: READ_TOGGLED,
        event: (REMEMBERED, "read"),
        want_on: true,
        before: None,
        means: "the View ▸ Off-Page Content toggle did not turn off-page content ON in Read. \
                This is the command the ribbon item raises, dispatched through the same choke \
                point a click reaches",
    },
    Rung {
        label: "read-remembered",
        invoke: READ,
        event: (SEED, "read"),
        want_on: true,
        before: None,
        means: "the answer given in the previous launch was FORGOTTEN. The operator asked for \
                `their preference is remembered`, and a preference that lasts until the program \
                closes is not remembered",
    },
    Rung {
        label: "edit-toggled-off",
        invoke: EDIT_TOGGLED,
        event: (REMEMBERED, "edit"),
        want_on: false,
        before: Some((MODE, "edit", true)),
        means: "Edit did not start with off-page content shown, or the toggle did not turn it \
                off there. Both halves are the operator's: `review and edit do show off page \
                items`, and `these settings can be changed by the user`",
    },
    Rung {
        label: "read-unaffected",
        invoke: READ,
        event: (MODE, "read"),
        want_on: true,
        before: None,
        means: "★★★ THE ANSWER IS NOT PER MODE. Turning off-page content off in EDIT also turned \
                it off in READ, which is one global flag wearing the name of three preferences. \
                The operator's sentence is `remembered for each read review edit modes`",
    },
];

/// See the module documentation.
pub struct TheOffPageToggleIsPerModeAndRemembered;

impl Check for TheOffPageToggleIsPerModeAndRemembered {
    fn name(&self) -> &'static str {
        "the_off_page_toggle_is_per_mode_and_remembered"
    }

    fn defect(&self) -> &'static str {
        "off-page content is shown in Read where the operator asked for it to be hidden, or the \
         toggle's answer is forgotten between launches, or one mode's answer overwrites another's \
         — three ways for a per-mode remembered preference to be a single global flag"
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

/// The workspace root, from this crate's manifest directory.
fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// The fraction of `patch` that is ink, and how many pixels that was measured
/// over. `None` means the patch had no area in the capture — a finding about
/// the harness, never "no ink".
fn ink_fraction(image: &Image, patch: PixRect) -> Option<(f64, u64)> {
    let total = u64::from(patch.w) * u64::from(patch.h);
    if total == 0 {
        return None;
    }
    let dark = image
        .pixels_in(patch)
        .filter(|p| p.r <= INK && p.g <= INK && p.b <= INK)
        .count() as u64;
    #[allow(
        clippy::cast_precision_loss,
        reason = "a patch is a few hundred pixels; f64 is exact far past that" // ui-text-exempt: clippy lint justification, never displayed
    )]
    Some((dark as f64 / total as f64, total))
}

/// What one rung measured.
struct Observed {
    /// The last `canvas-halo` line, verbatim.
    halo: String,
    /// Its `tier=` field.
    tier: String,
    /// Its `offpage=` field — the switch's own state, stated rather than
    /// inferred from the tier. See `canvas::trace::halo` on why the two are
    /// not the same fact.
    offpage: String,
    /// Ink fraction at the centre of the off-sheet square.
    off_sheet: f64,
    /// Ink fraction at the centre of the on-sheet square — the control.
    on_sheet: f64,
    /// The last `canvas-pasteboard` line, verbatim.
    pasteboard: String,
    /// Its `overhang=` field, split into x and y. Screen points.
    overhang: (f64, f64),
    /// Where the screenshot was written.
    shot: std::path::PathBuf,
}

/// Launch once, drive `rung.invoke`, and measure.
#[allow(clippy::too_many_lines)]
fn one_rung(
    ctx: &CheckContext,
    report: &mut CheckReport,
    exe: &std::path::Path,
    pdf: &std::path::Path,
    ui_rect: &str,
    rung: &Rung,
) -> Result<(Observed, crate::trace::Trace)> {
    let mut spec = LaunchSpec::new(
        exe,
        ctx.out(&format!("off-page-toggle-{}.trace.txt", rung.label)),
    );
    spec.pdf = Some(pdf.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), rung.invoke.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "rung `{}`: pid {} with PDFCER_DIAG_INVOKE={}",
        rung.label,
        session.pid(),
        rung.invoke
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    // Maximising widens the grey the off-sheet square has to fit into.
    session.maximize();
    // ★ A long settle, for `off_page_visible`'s measured reason: the halo
    // cannot appear on the first frame. The page must be decomposed before the
    // shell knows where its ink reaches, and that build happens AFTER the first
    // picture is asked for. The sequence is page → decomposition → halo raster,
    // and this reads the end of it. Four commands also take four frames.
    session.settle(60);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "rung `{}`: the trace has no `{}` line, so the diagnostic switch did not reach the \
             process. Captured stderr is at {}.",
            rung.label,
            ctx.profile.vocab.start_event,
            session.trace_path().display()
        )));
    }
    if declared(&trace, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "rung `{}`: no `{PAGE_REGION}` region, so no sheet is on screen and there is nothing \
             to be off the edge of. Regions beginning `page`: {}.",
            rung.label,
            list(&declared_names(&trace, ui_rect, "page"))
        )));
    }

    let Some(last) = trace.events(HALO).last() else {
        return Err(Error::new(format!(
            "rung `{}`: the application emitted no `{HALO}` line at all, so this is not a wrong \
             tier decision — the trace does not carry one. Either this binary predates \
             `canvas::trace::halo` or the canvas never laid out a page. Trace: {}.",
            rung.label,
            session.trace_path().display()
        )));
    };
    let observed_halo = last.raw.clone();
    let tier = last.get("tier").unwrap_or("?").to_owned();
    let offpage = last.get("offpage").unwrap_or("?").to_owned();

    // ★ The LAYOUT half. A missing line is a SKIP rather than a failure for
    // the same reason the tier line is: a binary that predates the
    // instrument cannot be interrogated with it, and reporting that as a
    // defect would send somebody after a feature that is present.
    let Some(paste) = trace.events(PASTEBOARD).last() else {
        return Err(Error::new(format!(
            "rung `{}`: the application emitted no `{PASTEBOARD}` line, so the layout half of \
             the switch cannot be measured. Either this binary predates \
             `canvas::trace::pasteboard` (2026-09-11) or the canvas never presented a page. \
             Trace: {}.",
            rung.label,
            session.trace_path().display()
        )));
    };
    let observed_paste = paste.raw.clone();
    let overhang = {
        let raw = paste.get("overhang").unwrap_or("?");
        let mut parts = raw.split(',').map(str::parse::<f64>);
        match (parts.next(), parts.next()) {
            (Some(Ok(x)), Some(Ok(y))) => (x, y),
            _ => {
                return Err(Error::new(format!(
                    "rung `{}`: `{PASTEBOARD}` carried an unreadable `overhang=` field: `{raw}`. \
                     The field is two `{{:.3}}` floats separated by a comma; a check cannot \
                     assert on a number it could not parse, and guessing zero would turn a \
                     format change into a silent pass.",
                    rung.label
                )));
            }
        }
    };

    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, FIXTURE_PAGE, 0)?;
    let frame = session.frame()?;
    let viewport = declared(&trace, ui_rect, VIEWPORT_REGION).ok_or_else(|| {
        Error::new(format!(
            "rung `{}`: the application declared no `{VIEWPORT_REGION}` region, so this check has \
             no bound to convert an off-page point against. It cannot fall back to the page's own \
             rect: the point sampled here is outside that by construction.",
            rung.label
        ))
    })?;

    // Both patches as RECTANGLES through the off-page conversion — the corners
    // are converted, not a centre plus a guessed pixel radius, so the patch is
    // the right size at any window scale. The on-sheet control goes through the
    // same conversion as the off-sheet sample deliberately: one conversion, one
    // set of assumptions, so a control that disagrees with the sample is a fact
    // about the page and not about two code paths.
    let patch_of = |at: (f64, f64)| -> Result<PixRect> {
        let lo = mapping
            .doc_to_window_off_page(DocPoint::new(0, at.0 - PATCH_PT, at.1 + PATCH_PT), viewport)?;
        let hi = mapping
            .doc_to_window_off_page(DocPoint::new(0, at.0 + PATCH_PT, at.1 - PATCH_PT), viewport)?;
        Ok(frame.logical_to_capture_pixels(LRect::new(
            Pt::new(lo.x().min(hi.x()), lo.y().min(hi.y())),
            Pt::new(lo.x().max(hi.x()), lo.y().max(hi.y())),
        )))
    };
    let off_patch = patch_of(OFF_SHEET_AT)?;
    let on_patch = patch_of(ON_SHEET_AT)?;

    let shot = ctx.out(&format!("off-page-toggle-{}.png", rung.label));
    let image = crate::capture::window_to_png(&session, &shot)?;
    report.artifact(shot.clone());

    let Some((off_sheet, off_px)) = ink_fraction(&image, off_patch) else {
        return Err(Error::new(format!(
            "rung `{}`: the off-sheet patch clipped to zero area in the capture ({off_patch:?} of \
             a {} x {} image), so nothing was measured. The conversion accepted the point, so \
             this is a capture-geometry disagreement rather than an application defect. \
             Screenshot: {}.",
            rung.label,
            image.width(),
            image.height(),
            shot.display()
        )));
    };
    let Some((on_sheet, on_px)) = ink_fraction(&image, on_patch) else {
        return Err(Error::new(format!(
            "rung `{}`: the ON-SHEET control patch clipped to zero area ({on_patch:?}), so the \
             measurement has no control and this check refuses to report on the off-sheet patch \
             alone. Screenshot: {}.",
            rung.label,
            shot.display()
        )));
    };
    report.note(format!(
        "rung `{}`: {} | {} — off-sheet patch {off_sheet:.3} over {off_px} px, on-sheet \
         control {on_sheet:.3} over {on_px} px",
        rung.label, observed_halo, observed_paste
    ));

    Ok((
        Observed {
            halo: observed_halo,
            tier,
            offpage,
            off_sheet,
            on_sheet,
            pasteboard: observed_paste,
            overhang,
            shot,
        },
        trace,
    ))
}

/// Assert one rung against what it measured. `Ok(None)` is a pass.
fn judge(rung: &Rung, seen: &Observed, trace: &crate::trace::Trace) -> Option<String> {
    // ★★★ THE CONTROL FIRST, on every rung including the positive ones. If the
    // on-sheet square is not painted, nothing measured off the sheet means
    // anything — and on a NEGATIVE rung the absence this check is looking for
    // would be supplied by the broken window rather than by the preference.
    if seen.on_sheet < INK_FRACTION {
        return Some(format!(
            "★★★ THE PROBE IS NOT MEASURING THE PAGE (rung `{}`): the ON-SHEET control patch \
             is only {:.3} ink, where at least {INK_FRACTION:.2} is required. That patch is at \
             ({:.0}, {:.0}) pt, the centre of a 60 pt black square that this fixture paints ON \
             the sheet, and it is not conditional on any setting.\n\n\
             ⇒ Read this as a finding about the HARNESS before reading it as one about the \
             feature: the window may not be where `WindowFrame` thinks it is, the capture may be \
             of the wrong monitor, or the document may not have rendered at all. The off-sheet \
             patch measured {:.3} and whatever that number is, it is not evidence — least of all \
             if this rung expected it to be clean. Screenshot: {}.",
            rung.label,
            seen.on_sheet,
            ON_SHEET_AT.0,
            ON_SHEET_AT.1,
            seen.off_sheet,
            seen.shot.display()
        ));
    }

    // The rung's own trace line: which event it is, and what it says.
    let (event, mode) = rung.event;
    let want = if rung.want_on { "true" } else { "false" };
    let said = trace
        .events(event)
        .filter(|l| l.get("mode") == Some(mode))
        .last();
    let Some(said) = said else {
        let any: Vec<String> = trace.events(event).map(|l| l.raw.clone()).collect();
        return Some(format!(
            "★★★ NO `{event}` LINE FOR MODE `{mode}` (rung `{}`).\n\n\
             {}.\n\n\
             The line is what says the answer came from the preference chain rather than from a \
             default that happens to agree. `{SEED}` is written when a document is opened \
             (`app::lifecycle`), `{MODE}` when the ribbon mode changes (`prefs::offpage::\
             apply_mode`) and `{REMEMBERED}` when the toggle is used (`prefs::offpage::\
             remember`) — so a missing line names which of the three is not wired. Lines of \
             this event seen at all: {}.",
            rung.label,
            rung.means,
            if any.is_empty() {
                "none".to_owned() // ui-text-exempt: report prose, never displayed in the UI
            } else {
                any.join(" | ")
            }
        ));
    };
    if said.get("on") != Some(want) {
        return Some(format!(
            "★★★ THE WRONG ANSWER WAS RESOLVED (rung `{}`): `{}`, where `on={want}` was \
             required.\n\n{}.",
            rung.label, said.raw, rung.means
        ));
    }

    // The rung's optional earlier line — rung 4's "Edit starts with it on".
    if let Some((event, mode, on)) = rung.before {
        let want = if on { "true" } else { "false" };
        let said = trace
            .events(event)
            .filter(|l| l.get("mode") == Some(mode))
            .last();
        match said {
            Some(l) if l.get("on") == Some(want) => {}
            Some(l) => {
                return Some(format!(
                    "★★★ THE MODE'S OWN DEFAULT IS WRONG (rung `{}`): `{}`, where `on={want}` \
                     was required for mode `{mode}` BEFORE the toggle was used.\n\n{}.",
                    rung.label, l.raw, rung.means
                ));
            }
            None => {
                return Some(format!(
                    "★★★ NO `{event}` LINE FOR MODE `{mode}` BEFORE THE TOGGLE (rung `{}`). The \
                     mode change is what applies a mode's answer to every open document; without \
                     it, this rung's toggle acted on whatever the previous mode had left \
                     behind.\n\n{}.",
                    rung.label, rung.means
                ));
            }
        }
    }

    // The switch's own state, as the canvas reports it.
    let want_field = if rung.want_on { "on" } else { "off" };
    if seen.offpage != want_field {
        return Some(format!(
            "★★★ THE CANVAS DISAGREES WITH THE PREFERENCE (rung `{}`): the resolved answer is \
             `on={want}` and the canvas reports `{}`.\n\n\
             The preference reached `prefs::offpage` and did not reach \
             `viewer::ViewState::off_page`, or reached it and was overwritten later in the frame. \
             The seed at document open (`app::lifecycle`) and the re-seed at mode change \
             (`prefs::offpage::apply_mode`) are the only two writers besides the toggle itself.\n\n\
             {}.",
            rung.label, seen.halo, rung.means
        ));
    }

    // ★★★ THE GAP. Asserted before the ink on a positive rung and before the
    // absence of ink on a negative one, because it is the cheaper, more
    // specific oracle: it names the function that got it wrong.
    if rung.want_on {
        if seen.overhang.0 <= 0.0 {
            return Some(format!(
                "★★★ THE LAYOUT WAS NOT WIDENED (rung `{}`): `{}`.\n\n\
                 The switch is on, so `canvas::tier::overhang` should have reported a reach \
                 past the sheet. This fixture's ink starts 160 pt left of the media box and \
                 the zoom is held at 100 %, so the x overhang should be around 160 screen \
                 points — not {:.3}.\n\n\
                 This is the half of the switch the operator described FIRST, and it is a \
                 different function from the one that widens the raster. The two are \
                 adjacent in `canvas::tier` precisely so that one cannot be flipped without \
                 the other.\n\n{}.",
                rung.label, seen.pasteboard, seen.overhang.0, rung.means
            ));
        }
        if seen.tier != TIER_HALO {
            return Some(format!(
                "★★★ THE RASTER WAS NEVER WIDENED (rung `{}`): `{}`.\n\n\
                 The switch is on and the shell still asked for the crop box. On this fixture the \
                 content union is x −160…200 against a 200 × 200 media box, so \
                 `render::halo::region` has a 160 pt overhang to find — far above its 1 pt \
                 tolerance. Suspect `canvas::tier::decide`'s gate substituting `None` for the \
                 content bounds when it should not.\n\n{}.",
                rung.label, seen.halo, rung.means
            ));
        }
        if seen.off_sheet < INK_FRACTION {
            return Some(format!(
                "★★★ OFF-PAGE CONTENT IS SWITCHED ON AND NOT PAINTED (rung `{}`): only \
                 {:.3} of the patch at ({:.0}, {:.0}) pt is ink, where at least {INK_FRACTION:.2} \
                 is required. The on-sheet control is {:.3}, so the probe IS looking at the page \
                 — this is the feature, not the harness.\n\n\
                 The shell asked for the widened box (`{}`), so the decision is right and the \
                 failure is downstream of it. `off_page_visible`'s header lists the three \
                 candidates in order. Screenshot: {}.\n\n{}.",
                rung.label,
                seen.off_sheet,
                OFF_SHEET_AT.0,
                OFF_SHEET_AT.1,
                seen.on_sheet,
                seen.halo,
                seen.shot.display(),
                rung.means
            ));
        }
        return None;
    }

    // The negative direction.
    if seen.overhang != (0.0, 0.0) {
        return Some(format!(
            "★★★ THE GAP IS STILL THERE (rung `{}`): `{}`.\n\n\
             With off-page display switched off the operator asked for the layout to go \
             back to exactly what it was before the feature existed — *\"there shouldn't \
             be a gap between pages where the stuff is\"* — and the canvas is still \
             reserving {:.3} x {:.3} screen points of reach past the sheet.\n\n\
             EXACT zero is required and is achievable: `canvas::tier::overhang` returns \
             early on this condition rather than multiplying a measured box by nothing, so \
             there is no rounding to forgive. A non-zero here means the early return was \
             not taken — the layout is reading a different flag from the raster, which is \
             the half-flipped switch those two functions were put side by side to \
             prevent.\n\n{}.",
            rung.label, seen.pasteboard, seen.overhang.0, seen.overhang.1, rung.means
        ));
    }
    if seen.tier == TIER_HALO {
        return Some(format!(
            "★★★ THE RASTER WAS WIDENED WITH THE SWITCH OFF (rung `{}`): `{}`.\n\n\
             `canvas::tier::decide` hands `None` for the content bounds when `view.off_page` is \
             false, which puts every page back on the whole/region tiers. A `halo` tier here \
             means that gate is not being consulted — and because the SAME condition zeroes \
             `canvas::tier::overhang`, the band of pasteboard the operator asked to be rid of is \
             almost certainly still there too.\n\n{}.",
            rung.label, seen.halo, rung.means
        ));
    }
    if seen.off_sheet > PAPER_INK_FRACTION {
        return Some(format!(
            "★★★ OFF-PAGE CONTENT IS SWITCHED OFF AND STILL PAINTED (rung `{}`): {:.3} of the \
             patch at ({:.0}, {:.0}) pt is ink, where at most {PAPER_INK_FRACTION:.2} is allowed. \
             The on-sheet control is {:.3}, so the window is drawing the document and this is not \
             a probe error.\n\n\
             The tier is `{}`, so the shell did NOT ask for a widened raster — which means the \
             ink at that address came from somewhere else: a stale texture from a previous frame \
             that is still being drawn at the old, wider rectangle, or a region raster whose \
             placement does not follow the narrowed box. Screenshot: {}.\n\n{}.",
            rung.label,
            seen.off_sheet,
            OFF_SHEET_AT.0,
            OFF_SHEET_AT.1,
            seen.on_sheet,
            seen.tier,
            seen.shot.display(),
            rung.means
        ));
    }
    None
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so this check has no bound to \
             convert an off-page point against.",
            ctx.profile.name
        ))
    })?;
    let pdf = workspace_root().join(FIXTURE);
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "the off-page fixture is not at {}. It is 485 bytes of hand-written PDF syntax, \
             shared with `off_page_visible`, `off_page_marquee` and `off_page_press`.",
            pdf.display()
        )));
    }

    // ★★★ THE PROFILE IS SHARED BETWEEN THE RUNGS AND PRIVATE TO THE CHECK.
    // `main` gives every check a `Sandbox` — its own copy of the binary, and
    // therefore its own `userdata/` — and hands it in as `--exe`. That is what
    // makes rungs 3 and 5 mean anything: they read what rungs 2 and 4 wrote,
    // in a preferences file no other check can see and that is deleted when
    // this one ends. Without the sandbox this check would write the operator's
    // own `off_page.read` answer on his own machine.
    for rung in LADDER {
        let (seen, trace) = one_rung(ctx, report, &exe, &pdf, ui_rect, rung)?;
        if let Some(failure) = judge(rung, &seen, &trace) {
            return Ok(Some(failure));
        }
        // ★ A pause between launches, for the reason `main` gives about the
        // suite: window teardown, GPU release and Windows' foreground
        // arbitration all lag process exit, and the next launch starts into
        // that wake. Here the lag matters more than usual, because the next
        // rung reads a preferences file the process that is going away wrote.
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    report.note(
        "★★★ five launches against one profile: Read opened HIDDEN, the toggle SHOWED it, a \
         restart still showed it, Edit opened SHOWN and the toggle HID it, and Read was \
         unaffected by any of that. That is the operator's paragraph in full — the per-mode \
         default, the operator's own answer, the answer surviving the process, and the three \
         modes not sharing one flag"
            .to_owned(),
    );
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::{
        INK_FRACTION, LADDER, MODE, OFF_SHEET_AT, ON_SHEET_AT, PAPER_INK_FRACTION, PATCH_PT,
        REMEMBERED, SEED,
    };

    /// The fixture's off-page square `(left, bottom, right, top)`, transcribed
    /// from its own content stream — `-160 100 120 40 re f`.
    const SQUARE_B: (f64, f64, f64, f64) = (-160.0, 100.0, -40.0, 140.0);

    /// The fixture's on-page square — `40 40 60 60 re f`.
    const SQUARE_A: (f64, f64, f64, f64) = (40.0, 40.0, 100.0, 100.0);

    /// The fixture's media box, which is also its crop box: it declares none.
    const MEDIA: (f64, f64, f64, f64) = (0.0, 0.0, 200.0, 200.0);

    /// ★★★ The off-sheet sample is wholly inside the off-page square, with
    /// margin on every side — otherwise the check measures antialiasing and
    /// its threshold becomes a coin toss.
    #[test]
    fn the_off_sheet_sample_is_inside_the_off_page_square() {
        assert!(OFF_SHEET_AT.0 - PATCH_PT > SQUARE_B.0, "left margin");
        assert!(OFF_SHEET_AT.0 + PATCH_PT < SQUARE_B.2, "right margin");
        assert!(OFF_SHEET_AT.1 - PATCH_PT > SQUARE_B.1, "bottom margin");
        assert!(OFF_SHEET_AT.1 + PATCH_PT < SQUARE_B.3, "top margin");
    }

    /// ★★ …and that square really is off the page, which is the premise of
    /// every rung.
    #[test]
    fn the_off_page_square_is_entirely_off_the_page() {
        assert!(
            SQUARE_B.2 < MEDIA.0,
            "square B's right edge {} must be left of the media box's left edge {}",
            SQUARE_B.2,
            MEDIA.0
        );
    }

    /// ★★★ The CONTROL is inside the on-page square, with margin — and inside
    /// the media box. A control that strayed off the sheet would be switched
    /// off by the very setting it exists to be independent of, and every
    /// negative rung would then report a harness error.
    #[test]
    fn the_control_sample_is_inside_the_on_page_square_and_on_the_sheet() {
        assert!(ON_SHEET_AT.0 - PATCH_PT > SQUARE_A.0, "left margin");
        assert!(ON_SHEET_AT.0 + PATCH_PT < SQUARE_A.2, "right margin");
        assert!(ON_SHEET_AT.1 - PATCH_PT > SQUARE_A.1, "bottom margin");
        assert!(ON_SHEET_AT.1 + PATCH_PT < SQUARE_A.3, "top margin");
        assert!(ON_SHEET_AT.0 - PATCH_PT > MEDIA.0 && ON_SHEET_AT.0 + PATCH_PT < MEDIA.2);
        assert!(ON_SHEET_AT.1 - PATCH_PT > MEDIA.1 && ON_SHEET_AT.1 + PATCH_PT < MEDIA.3);
    }

    /// The two thresholds cannot both be satisfied by one patch, which is what
    /// makes a rung's two directions mutually exclusive rather than merely
    /// differently worded.
    #[test]
    fn the_ink_and_paper_thresholds_do_not_overlap() {
        // A `const` block, so the two thresholds crossing is a BUILD failure
        // rather than a test failure — the suite would otherwise be shipped in
        // a state where a rung could pass in both directions at once, and a
        // check that cannot fail is not evidence.
        const { assert!(PAPER_INK_FRACTION < INK_FRACTION) };
    }

    /// ★★★ The ladder asserts both directions, in both modes, and at least one
    /// rung reads its answer from each of the three writers.
    ///
    /// The failure this catches is a later edit that trims the ladder into
    /// something that still passes: five rungs that all expect `on=true`, or
    /// five that all read `off-page-seed` and therefore never exercise the
    /// mode change. Either would leave a green check over an untested half.
    #[test]
    fn the_ladder_covers_both_directions_and_all_three_writers() {
        assert!(LADDER.iter().any(|r| r.want_on), "no rung expects ON");
        assert!(LADDER.iter().any(|r| !r.want_on), "no rung expects OFF");
        for writer in [SEED, MODE, REMEMBERED] {
            assert!(
                LADDER
                    .iter()
                    .any(|r| r.event.0 == writer || r.before.is_some_and(|b| b.0 == writer)),
                "no rung reads `{writer}`"
            );
        }
        for mode in ["read", "edit"] {
            assert!(
                LADDER.iter().any(|r| r.event.1 == mode),
                "no rung is about mode `{mode}`"
            );
        }
    }

    /// ★★ The independence rung must come AFTER the rung that changes the other
    /// mode's answer, or it proves nothing at all.
    #[test]
    fn the_independence_rung_follows_the_edit_toggle() {
        let edit = LADDER
            .iter()
            .position(|r| r.label == "edit-toggled-off")
            .expect("the Edit toggle rung");
        let read = LADDER
            .iter()
            .position(|r| r.label == "read-unaffected")
            .expect("the independence rung");
        assert!(
            edit < read,
            "`read-unaffected` at {read} must follow `edit-toggled-off` at {edit}, or it is \
             asserting that Read is on before anything has tried to turn it off"
        );
    }
}
