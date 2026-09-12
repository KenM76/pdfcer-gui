//! `the_page_preview_limit_is_remembered_and_zero_means_never` — **the two
//! halves of O187, driven.**
//!
//! # The report
//!
//! Ken, 2026-09-12, `OPERATOR_REQUESTS.md` **O187**:
//!
//! > *"the draw page previews timeout needs to be remembered, and setting it to
//! > 0 should set it to infinity (never time out)"*
//!
//! Two sentences, two entirely separate mechanisms, and a build can ship either
//! one without the other:
//!
//! 1. **Remembered** — the Pages panel's previews tick and the time limit
//!    beside it now write through to `preferences.txt` the moment they change,
//!    and are read back once at construction. Before O187 both were
//!    session-only, so an operator who cleared the tick because previews were
//!    slow on their sheet set met them again on the next launch.
//! 2. **Zero means never** — the limit's type became `Option<Duration>`, `None`
//!    is *no limit*, and `0` is the operator's own notation for it in both the
//!    control and the file.
//!
//! ★★ The second is the one most at risk from the first, and that is the whole
//! reason this check exists in the shape it does. Every other small number in
//! that control is raised to a 100 ms floor, because a one-millisecond budget
//! is an off switch wearing a number. `0` has to pass through the same clamp
//! **and come out the other side untouched**, twice — once on the way into the
//! cache and once on the way back out of the file. A clamp that treated it like
//! its neighbours would quietly turn *no limit at all* into the default, and
//! the operator would experience that as *the box refusing to keep the zero he
//! typed*.
//!
//! # ★★★ Why this is a THREE-PROCESS check
//!
//! Because the subject is a value **surviving a process**, and one launch
//! cannot express that. Neither can two, for this particular pair:
//!
//! | Launch | Gesture | What its successor proves |
//! |---|---|---|
//! | 1 | clear the previews tick | the **tick** survived a close |
//! | 2 | type `0` into the limit | the **limit** survived a close, as a zero |
//! | 3 | nothing — it only reads | both, together, from a file written by two different gestures |
//!
//! A two-launch version would have to change both controls in the same process,
//! and then a build that wrote only the last field touched would still pass.
//! Splitting the two gestures across two processes is what makes the third
//! launch's `previews=0 budget_ms=0` a statement about **both** halves of the
//! whole-file write rather than about whichever one happened to go last.
//!
//! # ★★★ Why each process is KILLED and not closed gracefully
//!
//! This is the deliberate opposite of [`super::page_display_pref`], and the
//! contrast is the point. That check presses `Alt+F4` because its subject is a
//! **debounced** write that only an exit hook can rescue — kill it and you are
//! measuring whether the debounce happened to expire, which is true on a slow
//! machine and false on a fast one.
//!
//! O187's write is not debounced. Property 4 of the preference family
//! (`app::actions::prefs`' header) is *one discrete operator decision is one
//! write, **now***. So killing the process is not a shortcut here — it is the
//! assertion. A build that wrote the preference from an exit hook, or from a
//! 750 ms debounce, would pass a graceful-close check and fail the operator the
//! first time the program was closed from the Task Manager or fell over. This
//! check closes in the way that gives the feature no help at all.
//!
//! # The oracles, and why there are three of them
//!
//! | Oracle | Line | Answers |
//! |---|---|---|
//! | the gesture | `page-previews-persisted on=… budget_ms=…` | the verb ran and carried **both** values |
//! | the next launch | `pages-panel … previews=… budget_ms=…` | the file was read back into the live cache |
//! | the file | `page_preview_budget_ms = 0` in `preferences.txt` | the number on disk, independent of either trace |
//!
//! ★ The third is not redundant. The first two are both written by the program
//! under test, in the same run, from values that could in principle both come
//! from the same wrong place. Reading the file is the one observation this
//! harness makes that the application cannot have coloured — an oracle built
//! from the system under test needs an independent calibration, and the file is
//! it.
//!
//! ★★ `budget_ms` is deliberately **not** compared against a hard-coded `2000`
//! anywhere below. The default lives in `thumbnails::PAGE_BUDGET_DEFAULT` and a
//! harness that duplicated it would go stale silently the day it moved. The
//! starting state is asserted as *"not zero"* and the end state as *"zero"*,
//! which is a real change in a known direction and satisfiable by no constant.
//!
//! # What the starting state has to be, and why it is checked rather than assumed
//!
//! `previews=1` and `budget_ms != 0` before the first gesture. Both are the
//! shipped defaults, and the check writes a bare sandbox `preferences.txt`
//! first so they are what it meets — but it **says so out loud** rather than
//! trusting it. A run that began with the tick already clear would clear
//! nothing, find `previews=0` in launch 2, and report a pass for a build that
//! persists nothing whatsoever. *A fixture that defeats a default does not
//! defeat a starting state*: the starting state has to be planted and then
//! confirmed.
//!
//! # ⚠ What is normalised, and the one file that is RESET rather than deleted
//!
//! Only `preferences.txt`, and through [`crate::sandbox::write_prefs`], never
//! `fs::write` and never a delete. The sandbox header carries
//! `ask_default_app = false`; three checks that wrote the file directly
//! re-enabled the O173 startup offer in front of their own launches, and one of
//! them then measured the *dialog's* client area and reported a working
//! preference as broken. A [`RestorePrefs`] guard puts the seed back however
//! this check ends, because a suite that shares state measures the order it ran
//! in.
//!
//! ★★ Safe because the suite is **never** pointed at a published build — that
//! is the standing rule, and a check that rewrites `userdata/preferences.txt`
//! is one of the reasons for it.
//!
//! # Every way this reports SKIP
//!
//! No binary; `--no-input` (this check is entirely pointer and keyboard); the
//! fixture missing; the profile declaring no ui-rect event; the preferences
//! file not writable; the Pages panel's controls not declared; the mode segment
//! not declared; the control chord producing no `chord-command` line, which
//! means no keystroke reached the window and nothing typed below would mean
//! anything; or the starting state not being the shipped default, which means
//! something outside this check seeded the sandbox.

use std::path::{Path, PathBuf};

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment, frame_of, stable_rect};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;
use crate::trace::Trace;

/// The mode the Pages panel belongs to, and the one `pages_drag` uses.
const MODE: &str = "review";

/// The previews checkbox — `panels::pages::previews::PREVIEWS_REGION`.
const PREVIEWS: &str = "panel-pages-previews";

/// The time limit beside it — `panels::pages::previews::BUDGET_REGION`.
const BUDGET: &str = "panel-pages-budget";

/// The thumbnail grid. Read only to decide whether the panel is already up, so
/// that a docked-by-default layout is not toggled CLOSED by pressing its own
/// ribbon item.
const GRID: &str = "panel-pages-grid";

/// `pages-panel pages=… current=… selected=… visible=… drawn=… previews=…
/// budget_ms=…`, emitted once per change by `panels::pages`.
const PANEL_EVENT: &str = "pages-panel";

/// `page-previews-persisted on=… budget_ms=…`, emitted by
/// `app::actions::prefs::PrefAction::PagePreviews::apply` **after** the file is
/// written.
const PERSIST_EVENT: &str = "page-previews-persisted";

/// The control chord's own trace line — proof that a keystroke reached the
/// window at all. See [`keyboard_reaches_the_window`].
const CHORD_EVENT: &str = "chord-command";

/// What the control chord resolves to. `Ctrl+2` is bound to the Review mode,
/// which this check is already in by the time it presses it — deliberately, so
/// the probe changes nothing it is about to measure.
const CHORD_ID: &str = "mode.review";

/// The document. Four pages, so the grid has something to draw and the panel
/// is worth having open.
const FIXTURE: &str = "fixtures/four-pages.pdf";

/// The key the preferences file carries the limit under.
const BUDGET_KEY: &str = "page_preview_budget_ms";

/// The key the preferences file carries the tick under.
const PREVIEWS_KEY: &str = "page_previews";

/// See the module documentation.
pub struct ThePagePreviewLimitIsRememberedAndZeroMeansNever;

impl Check for ThePagePreviewLimitIsRememberedAndZeroMeansNever {
    fn name(&self) -> &'static str {
        "the_page_preview_limit_is_remembered_and_zero_means_never"
    }

    fn defect(&self) -> &'static str {
        "the Pages panel's previews tick or its time limit is forgotten when the program \
         closes, or a limit of 0 — the operator's word for `never time out` — is clamped back \
         to a number on the way through the file"
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

// ===========================================================================
// The drive
// ===========================================================================

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check is entirely input: it clicks a mode \
             segment, a checkbox and a spinner, and it types a digit. Reported as SKIPPED rather \
             than passed — a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the Pages panel's controls \
             cannot be found.",
            ctx.profile.name
        ))
    })?;
    let pdf = crate::fixture::workspace_root().join(FIXTURE);
    if !pdf.is_file() {
        return Err(Error::new(format!("fixture missing: {}", pdf.display())));
    }
    let userdata = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory to write userdata into"))?
        .join("userdata");

    // ★ The guard is armed BEFORE the first write, so every return below —
    // including the error ones, including a panic — puts the sandbox back.
    let _restore = RestorePrefs(userdata.clone());
    write_prefs(&userdata)?;
    report.note(format!(
        "the sandbox's preferences in {} were reset to the bare seed, so this run starts from \
         the shipped defaults rather than from whatever a previous run left",
        userdata.display()
    ));

    // -----------------------------------------------------------------------
    // Launch 1 — plant the starting state, clear the tick, and be killed.
    // -----------------------------------------------------------------------
    let session = launch(ctx, report, &exe, &pdf, "1")?;
    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    open_panel(&session, &driver, ui_rect)?;

    let trace = session.trace()?;
    let start = panel_state(&trace).ok_or_else(|| {
        Error::new(format!(
            "the Pages panel traced no `{PANEL_EVENT}` line, so it never drew — there is \
             nothing to change and nothing to measure. Trace: {}.",
            session.trace_path().display()
        ))
    })?;
    if !start.previews_on {
        return Err(Error::new(format!(
            "the run began with previews already OFF (`{}`). Clearing the tick would change \
             nothing, launch 2 would find `previews=0` for the wrong reason, and this check \
             would report a pass for a build that persists nothing at all. The reset above \
             should have prevented it — something outside this check is seeding the sandbox.",
            start.raw
        )));
    }
    if start.budget_ms == 0 {
        return Err(Error::new(format!(
            "the run began with the limit already at 0 (`{}`), which is the value this check \
             types in later. It could not then tell a limit that was kept from one that was \
             never changed. The reset above should have prevented it.",
            start.raw
        )));
    }
    report.note(format!(
        "launch 1 started from the shipped defaults: previews on, limit {} ms — a real starting \
         state, planted and confirmed rather than assumed",
        start.budget_ms
    ));

    let before = trace.events(PERSIST_EVENT).count();
    click_region(
        &session,
        &driver,
        ui_rect,
        PREVIEWS,
        "the previews checkbox",
    )?;
    session.settle(16);

    let trace = session.trace()?;
    let Some(persisted) = persisted_after(&trace, before) else {
        return Ok(Some(format!(
            "★★★ THE TICK WAS NOT WRITTEN THROUGH. The previews checkbox was clicked and no \
             `{PERSIST_EVENT}` line followed, so nothing reached `preferences.txt` — this is \
             O187's first half missing. Two readings, and the panel line tells them apart: \
             `{}` is what the panel reported after the click. If `previews=` changed, the \
             control works and the ACTION is not being raised or not being applied — look at \
             `panels::pages::previews::persist` and at `Action::Pref`'s arm in `app::actions::\
             apply`, which is matched ABOVE the `Status::Open` guard on purpose. If `previews=` \
             did not change, the click did not land on the checkbox. Trace: {}.",
            panel_state(&trace).map_or_else(|| "no panel line".to_owned(), |s| s.raw),
            session.trace_path().display()
        )));
    };
    if persisted.on {
        return Ok(Some(format!(
            "the checkbox was clicked and the preference was written as still ON (`{}`), so the \
             verb carried the value from before the change rather than after it. The live half \
             is applied by the panel before the action is raised; an action that read the tick \
             from anywhere other than the cache would produce exactly this.",
            persisted.raw
        )));
    }
    if persisted.budget_ms != start.budget_ms {
        return Ok(Some(format!(
            "★★ THE OTHER HALF WAS LOST IN THE CARRY. Clearing the tick wrote `{}`, but the \
             limit was {} ms before the click and nobody touched it.\n\n\
             `PrefAction::PagePreviews` carries BOTH values on purpose — `Prefs::save` is a \
             whole-file write, so a variant that carried only the field that changed would \
             write the other one from whatever `Prefs` happened to hold. That is the defect \
             this assertion exists for, and it is invisible until the two controls are used in \
             the same session.",
            persisted.raw, start.budget_ms
        )));
    }
    report.note(format!(
        "launch 1 cleared the tick and the file was written in the same breath: `{}` — and it \
         carried the untouched limit with it, which is what makes one gesture one whole-file \
         write",
        persisted.raw
    ));

    // ★★ KILLED, not closed. See the module header: O187's write is not
    // debounced and must not need an exit hook. Dropping the session kills the
    // process, so nothing below can have been rescued on the way out.
    drop(session);
    report.note(
        "launch 1 was KILLED rather than closed — no exit hook ran, so anything the \
                 next launch reads was written at the moment of the gesture",
    );

    // ★★ the file, read BETWEEN the two launches, because the two halves of
    // this check fail in ways that are identical from launch 2 alone. A write
    // that never reached the disk and a read that dropped it both present as
    // `previews=1` in a new process, and they live in different modules.
    //
    // The save failure is SWALLOWED by design — property 4 of the preference
    // family, and correct: a preference is not worth a modal in front of
    // somebody who is in the middle of something. But it means the trace line
    // above is emitted whether or not the file was written, and nothing except
    // the file itself can tell those two apart.
    let after_one = read_prefs(&userdata)?;
    match value_of(&after_one, PREVIEWS_KEY) {
        Some("false") => {
            report.note(
                "and the file on disk carries it: `page_previews = false` — so the write \
                 reached the disk, and anything launch 2 gets wrong from here is on the READ \
                 side",
            );
        }
        other => {
            return Ok(Some(format!(
                "★★★ THE WRITE NEVER REACHED THE DISK. The verb traced \
                 `{PERSIST_EVENT}` — which it does AFTER `Prefs::save`, and which it does \
                 whether or not that save succeeded, because the failure is swallowed on \
                 purpose — and `preferences.txt` holds `{PREVIEWS_KEY} = {}`.\n\n\
                 So the defect is on the WRITE side and not the read side: `Prefs::save` \
                 returned an error nobody looked at, or it wrote somewhere other than {}. \
                 The swallow is correct; it is also why the file is read here rather than \
                 the line above being trusted.",
                other.unwrap_or("no such key"),
                userdata.display()
            )));
        }
    }

    // -----------------------------------------------------------------------
    // Launch 2 — the tick survived; now type the zero.
    // -----------------------------------------------------------------------
    let session = launch(ctx, report, &exe, &pdf, "2")?;
    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    open_panel(&session, &driver, ui_rect)?;

    let trace = session.trace()?;
    let second = panel_state(&trace).ok_or_else(|| {
        Error::new(format!(
            "launch 2's Pages panel traced no `{PANEL_EVENT}` line. Trace: {}.",
            session.trace_path().display()
        ))
    })?;
    if second.previews_on {
        return Ok(Some(format!(
            "★★★ THE TICK WAS FORGOTTEN. It was cleared in launch 1, the file was written \
             (`{PERSIST_EVENT}` was traced), and a new process opened with previews back ON — \
             `{}`.\n\n\
             The write happened, so the defect is on the READ side: `PdfcerApp::new` seeds \
             `pages.cache.force_on(prefs.page_previews)` once, at construction. Either that \
             line is gone, or `prefs::file` is not parsing `{PREVIEWS_KEY}` back out of the \
             file it wrote. Compare the two traces.",
            second.raw
        )));
    }
    if second.budget_ms != start.budget_ms {
        return Ok(Some(format!(
            "the limit changed across the close without anybody touching it: {} ms in launch 1, \
             `{}` in launch 2. Nothing in this check altered it, so either the write carried a \
             different value than it reported or the read is not round-tripping it.",
            start.budget_ms, second.raw
        )));
    }
    report.note(format!(
        "launch 2 opened with the tick still clear and the limit unchanged at {} ms — the first \
         half of O187 survived a kill",
        second.budget_ms
    ));

    keyboard_reaches_the_window(&session, &driver)?;

    let before = session.trace()?.events(PERSIST_EVENT).count();
    // ★ ONE click on the `DragValue`, which is what puts egui into keyboard
    // editing and selects the whole of the displayed text. See `type_the_zero`
    // for why nothing else is pressed first.
    click_region(&session, &driver, ui_rect, BUDGET, "the time-limit spinner")?;
    session.settle(12);
    type_the_zero(&session, &driver)?;

    let trace = session.trace()?;
    let Some(persisted) = persisted_after(&trace, before) else {
        return Ok(Some(format!(
            "★★ TYPING `0` INTO THE LIMIT WROTE NOTHING. No `{PERSIST_EVENT}` line followed the \
             edit, so either the spinner never committed or the commit does not persist.\n\n\
             The commit is on `ended` — `lost_focus` — rather than on `changed`, which is what \
             `app::spinnerdraft` exists for; the Enter keypress is what ends it. The control \
             chord above proved keystrokes reach this window, so `the field never had focus` is \
             the remaining harness-side reading and `panel state: {}` is what the panel \
             reported. Trace: {}.",
            panel_state(&trace).map_or_else(|| "none".to_owned(), |s| s.raw),
            session.trace_path().display()
        )));
    };
    if persisted.budget_ms != 0 {
        return Ok(Some(format!(
            "★★★ ZERO DID NOT SURVIVE THE CONTROL. `0` was typed into the limit and the \
             preference was written as `{}`.\n\n\
             This is O187's second sentence failing at the first hurdle, and the number names \
             the cause:\n\
             • `budget_ms=100` — the zero was clamped to `MIN_PAGE_BUDGET` like any other small \
             value. `budget_from_millis` is the ONE place that is allowed to decide what the \
             number means, and it must answer `None` for 0 before any clamp sees it.\n\
             • the starting {} — the parse rejected the text and the control kept what it had. \
             `previews::parse_budget` has to accept a bare `0`, and it has to accept the word \
             the formatter shows at zero as well.\n\
             • anything else — the spinner committed a value the operator did not type.",
            persisted.raw, start.budget_ms
        )));
    }
    if persisted.on {
        return Ok(Some(format!(
            "the limit was written with the tick back ON (`{}`) — the carry picked up a stale \
             value for the field this gesture did not change, which is the mirror image of the \
             assertion launch 1 makes.",
            persisted.raw
        )));
    }
    report.note(format!(
        "launch 2 typed `0` and the file took it as a zero: `{}` — the clamp that raises every \
         other small value let the sentinel through",
        persisted.raw
    ));
    drop(session);

    // -----------------------------------------------------------------------
    // The file itself — the one oracle the application did not write twice.
    // -----------------------------------------------------------------------
    let prefs_text = read_prefs(&userdata)?;
    match value_of(&prefs_text, BUDGET_KEY) {
        Some("0") => {
            report.note(format!(
                "and the file on disk agrees, independently of either trace: `{BUDGET_KEY} = 0`"
            ));
        }
        other => {
            return Ok(Some(format!(
                "★★ THE TRACE AND THE FILE DISAGREE. `{PERSIST_EVENT}` reported `budget_ms=0` \
                 and `preferences.txt` holds `{BUDGET_KEY} = {}`.\n\n\
                 The trace is written by `PrefAction::apply` immediately after `Prefs::save`, so \
                 the two can only differ if the save wrote something other than what the action \
                 was handed — look at `prefs::file`'s writer for that key. This is exactly why \
                 the file is read here rather than trusted: an oracle built from the system \
                 under test needs an independent calibration.",
                other.unwrap_or("no such key")
            )));
        }
    }

    // -----------------------------------------------------------------------
    // Launch 3 — both halves, from a file written by two different gestures.
    // -----------------------------------------------------------------------
    let session = launch(ctx, report, &exe, &pdf, "3")?;
    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    open_panel(&session, &driver, ui_rect)?;

    let trace = session.trace()?;
    let third = panel_state(&trace).ok_or_else(|| {
        Error::new(format!(
            "launch 3's Pages panel traced no `{PANEL_EVENT}` line. Trace: {}.",
            session.trace_path().display()
        ))
    })?;
    if third.budget_ms != 0 {
        return Ok(Some(format!(
            "★★★ `NEVER` BECAME A NUMBER AGAIN ON THE WAY BACK IN. The file holds \
             `{BUDGET_KEY} = 0` and a new process opened showing `{}`.\n\n\
             The write is correct, so this is the READ path: `PdfcerApp::new` hands \
             `prefs.page_preview_budget_ms` to `thumbnails::budget_from_millis`, which must \
             answer `None` for 0 — and `ThumbnailCache::set_budget` must not clamp a `None` \
             back into a `Some`. `budget_ms={}` in that line is `millis_from_budget` reporting \
             what the cache actually holds, so whatever it says is what the operator's next \
             render will obey.\n\n\
             ⚠ If it reads 100, the floor caught the sentinel. If it reads the compiled-in \
             default, the zero was treated as absent — a parse that maps an unreadable value to \
             the default would do that, and so would a writer that omits the key at zero.",
            third.raw, third.budget_ms
        )));
    }
    if third.previews_on {
        return Ok(Some(format!(
            "the limit survived and the tick did not — `{}`. Two gestures wrote this file in \
             two different processes, and only the second one's field came back, which is what \
             a whole-file write that reconstructs `Prefs` from something other than the live \
             values looks like.",
            third.raw
        )));
    }
    report.note(format!(
        "★★★ launch 3 opened with `{}` — both halves of O187, written by two gestures in two \
         processes that were each killed outright, read back together",
        third.raw
    ));
    if value_of(&prefs_text, PREVIEWS_KEY).is_some() {
        report.note(format!(
            "the file's own words for the same two facts: `{PREVIEWS_KEY} = {}`, \
             `{BUDGET_KEY} = 0`",
            value_of(&prefs_text, PREVIEWS_KEY).unwrap_or("?")
        ));
    }
    Ok(None)
}

// ===========================================================================
// The machinery
// ===========================================================================

/// Launch one process on the fixture and settle it.
///
/// `tag` names the artefact, so a failing run leaves all three traces side by
/// side — which is the first thing anybody reading a failure here will want,
/// because every assertion in this file is a comparison between two launches.
fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    exe: &Path,
    pdf: &Path,
    tag: &str,
) -> Result<Session> {
    let mut spec = LaunchSpec::new(exe, ctx.out(&format!("page-previews-pref.{tag}.trace.txt")));
    spec.pdf = Some(pdf.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    // The shell's own channel, because `click_mode_segment` reads it to tell
    // "the click did not land" from "the switch never reached the process".
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!("launch {tag}: pid {}", session.pid()));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    // Maximized so the dock has room for the Pages panel's controls. A panel
    // squeezed to nothing declares no regions, and the failure would read as
    // the controls being missing.
    session.maximize();
    session.settle(14);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "launch {tag}: the trace has no `{}` line, so the diagnostic switch did not reach \
             the process. Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            session.trace_path().display()
        )));
    }
    Ok(session)
}

/// Put the Pages panel on screen, unless it already is.
///
/// ⚠ The `GRID` test is not an optimisation. The Pages panel is docked by
/// default, and its ribbon item is a **toggle** — pressing it on a panel that
/// is already up would CLOSE the surface under test, and every assertion below
/// would then report a missing control. `pages_drag::open_pages_panel` carries
/// the same guard at its call site and this is the same rule, stated here
/// because this module has three call sites for it.
fn open_panel(session: &Session, driver: &Driver, ui_rect: &str) -> Result<()> {
    if driving::declared(&session.trace()?, ui_rect, GRID).is_some() {
        return Ok(());
    }
    super::pages_drag::open_pages_panel(session, driver, ui_rect)
}

/// Click a named region, once it has stopped moving.
///
/// ★ Through [`stable_rect`] rather than [`driving::declared`], because raising
/// a dock panel re-lays the dock out over several frames and `ui-rect` is a
/// change log: reading it the frame after a panel opens answers *where that
/// control was*, and a click aimed at a stale coordinate lands on the canvas
/// with no error anywhere. This project has that failure on record twice.
///
/// ★★ And through [`frame_of`] rather than `session.frame()`, which costs
/// nothing on a main-window region and survives the day this panel is allowed
/// to float into its own OS window — at which point its rectangles become
/// relative to *that* window's origin and every click would land hundreds of
/// pixels away, with plausible numbers and no error. Thirteen checks learned
/// that on one day.
fn click_region(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    name: &str,
    what: &str,
) -> Result<()> {
    let Some(rect) = stable_rect(session, ui_rect, name, 10)? else {
        return Err(Error::new(format!(
            "the Pages panel declares no `{name}` region, so {what} is not on screen and cannot \
             be clicked. Reported as SKIPPED rather than as a failure: a control that is not \
             there is a different defect from a preference that is not kept, and this check \
             refuses to name the wrong one."
        )));
    };
    let frame = frame_of(session, &session.trace()?, ui_rect, name)?;
    driver.click_at(frame.declared_center(rect))?;
    Ok(())
}

/// **Prove a keystroke reaches this window before typing anything that matters.**
///
/// ★★★ Without this, a build in which the pointer works and the keyboard does
/// not would produce *"typing `0` into the limit wrote nothing"* — a confident,
/// detailed and entirely wrong report naming O187 as the culprit. `find_bar`'s
/// first run did exactly that against a build in which `Ctrl+F` worked.
///
/// `Ctrl+2` is bound to `mode.review` in the application's key table, and this
/// check is already in Review mode by the time it gets here. So the probe is
/// idempotent: it proves the channel without changing a thing the assertions
/// below depend on.
///
/// # Errors
///
/// No new `chord-command` line — reported as a SKIP at the call site, because a
/// check that types into nothing must never name a feature as the culprit.
fn keyboard_reaches_the_window(session: &Session, driver: &Driver) -> Result<()> {
    let before = session.trace()?.events(CHORD_EVENT).count();
    driver.press_chord(&[vk::CONTROL], vk::DIGIT_2)?;
    session.settle(12);
    let probe = session.trace()?;
    let landed = probe
        .events(CHORD_EVENT)
        .skip(before)
        .any(|l| l.get("id") == Some(CHORD_ID));
    if !landed {
        return Err(Error::new(format!(
            "the control chord Ctrl+2 (`{CHORD_ID}`) produced no new `{CHORD_EVENT}` line, so no \
             keystroke reached the application and nothing typed below would mean anything. \
             Reported as SKIPPED rather than as an O187 failure: a check that types into nothing \
             must never name a feature as the culprit."
        )));
    }
    Ok(())
}

/// Type the zero and commit it.
///
/// ★ **No `Ctrl+A` first, deliberately.** egui's `DragValue` selects the whole
/// of its displayed text the frame its edit gains focus, so the click has
/// already done it — and `Ctrl+A` is bound in this application to a document
/// verb whose guard depends on a text field being focused. Pressing it here
/// would make the check's own setup depend on the very guard that a sibling
/// defect in this project once broke, which is a dependency worth not having
/// when the alternative is nothing at all.
///
/// ★★ `Enter` rather than clicking elsewhere. The commit is on `ended` —
/// `lost_focus` — because `app::spinnerdraft` exists to stop a re-seeded value
/// throwing a drag away, and Enter is the only way to end the edit that does
/// not also press something else.
fn type_the_zero(session: &Session, driver: &Driver) -> Result<()> {
    driver.type_ascii("0")?;
    session.settle(8);
    driver.press(vk::ENTER)?;
    session.settle(18);
    Ok(())
}

/// What the Pages panel last reported about itself.
struct PanelState {
    previews_on: bool,
    budget_ms: usize,
    /// The whole line, so a failure quotes the evidence rather than a
    /// reconstruction of it.
    raw: String,
}

/// Read the last `pages-panel` line.
///
/// ⚠ `last`, and the panel emits through `trace_changed` — one line per change
/// rather than one per frame — so the last one is the current state and not a
/// fossil from the frame the panel happened to be drawn on.
fn panel_state(trace: &Trace) -> Option<PanelState> {
    let line = trace.last(PANEL_EVENT)?;
    Some(PanelState {
        previews_on: line.get("previews") == Some("1"),
        budget_ms: line.get_usize("budget_ms").unwrap_or(0),
        raw: line.raw.clone(),
    })
}

/// What the preference verb last wrote.
struct Persisted {
    on: bool,
    budget_ms: usize,
    raw: String,
}

/// The first `page-previews-persisted` line **after** the first `before` of
/// them, or `None` if the gesture produced no new one.
///
/// ★ Counted rather than compared against an absolute absence, because this
/// check performs two write-through gestures in two processes and a naive
/// `last()` would happily return the previous one. An absence assertion is only
/// as good as when its baseline was taken.
fn persisted_after(trace: &Trace, before: usize) -> Option<Persisted> {
    trace
        .events(PERSIST_EVENT)
        .skip(before)
        .last()
        .map(|l| Persisted {
            // ⚠ `true`/`false`, not `1`/`0`. The SAME fact is spelled two
            // ways in two lines: `PrefAction::apply` formats a Rust `bool`, and
            // the panel line formats `u8::from(..)`. A comparison against `"1"`
            // here is never a compile error and is always false, so it would have
            // reported "the verb wrote the value from before the change" for every
            // build ever made. Measured on this file, first run, 2026-09-12.
            on: l.get("on") == Some("true"),
            budget_ms: l.get_usize("budget_ms").unwrap_or(usize::MAX),
            raw: l.raw.clone(),
        })
}

/// Read the sandbox's preference file.
///
/// # Errors
///
/// Unreadable — reported as a SKIP at every call site, because a file this
/// harness cannot read is a harness problem and must never be named as a defect
/// in the program.
fn read_prefs(userdata: &Path) -> Result<String> {
    std::fs::read_to_string(userdata.join("preferences.txt")).map_err(|e| {
        Error::new(format!(
            "the preferences file could not be read back from {}: {e}. Reported as SKIPPED: a \
             file this harness cannot read is a harness problem, not a defect in the program.",
            userdata.display()
        ))
    })
}

/// The value of `key = value` in a preferences file, ignoring comments.
///
/// Deliberately a five-line parser rather than a call into the application's
/// own reader: the point of reading this file is to observe it with something
/// the program under test did not write. A harness that parsed it with the
/// crate's own parser would agree with the program by construction.
fn value_of<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .find(|(k, _)| k.trim() == key)
        .map(|(_, v)| v.trim())
}

/// Write the sandbox's preference file back to the bare seed.
///
/// ★★★ Through `sandbox::write_prefs`, never `fs::write`, and never a delete.
/// The header it prepends carries `ask_default_app = false`; three checks that
/// wrote the file directly re-enabled the O173 startup offer in front of their
/// own launches, and deleting it does the same thing by omission — every absent
/// key takes its compiled-in default, and that one's is `true`.
///
/// # Errors
///
/// The directory could not be created or the file could not be written. A SKIP
/// at the call site: a preference that could not be written means the check
/// never began.
fn write_prefs(userdata: &Path) -> Result<()> {
    crate::sandbox::reset_prefs(userdata).map_err(|e| {
        Error::new(format!(
            "could not reset the preferences in {}: {e}. Reported as SKIPPED — a starting state \
             that could not be planted means the check never began.",
            userdata.display()
        ))
    })
}

/// Put the sandbox back to the bare seed when the check ends, however it ends.
///
/// ★ A guard rather than a line at the end, because there are a dozen returns
/// above and the one that gets forgotten is the one that leaves
/// `page_preview_budget_ms = 0` behind for every check that runs afterwards —
/// which would arm an unbounded thumbnail render in whatever check drew a Pages
/// panel next. A suite that shares state measures the order it ran in.
///
/// Reset rather than deleted, for the reason `ui_scale` records: a *missing*
/// file exercises the absent-file path, which is a different state and not the
/// one the other checks were written against.
///
/// Failure to restore is warned about rather than fatal — this type runs during
/// unwinding as well as on the ordinary path, and a harness that turned its own
/// housekeeping problem into a verdict would be reporting itself as a defect in
/// the program.
struct RestorePrefs(PathBuf);

impl Drop for RestorePrefs {
    fn drop(&mut self) {
        if let Err(e) = crate::sandbox::reset_prefs(&self.0) {
            eprintln!(
                "ui-verify: WARNING — could not reset {} ({e}). A later check may be measuring a \
                 page-preview preference this one left behind.",
                self.0.display()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The value typed in is the one value that is not a quantity.**
    ///
    /// ★ Pinned because every other number this control accepts is clamped to a
    /// floor, and a check that typed `1` would pass against a build with O187's
    /// second half missing entirely — `1` and `100` are both *a limit*, and the
    /// operator would never know. Only `0` can tell the two builds apart.
    #[test]
    fn the_check_types_the_one_value_that_is_a_sentinel() {
        // The constant this file asserts on, spelled once here so a future edit
        // that softened it to `100` has to argue with this comment first.
        assert_eq!(
            0_usize, 0,
            "the limit asserted at the end of this check is zero, and zero is not a small number"
        );
    }

    /// The three regions named are the panel's, not the ribbon's.
    ///
    /// ★ A check that named `ribbon.item.view.panel_pages` as its control would
    /// be asserting that a menu entry exists, which is true in every build that
    /// has ever shipped and says nothing about the preference.
    #[test]
    fn the_regions_named_are_the_panels_own_controls() {
        for name in [PREVIEWS, BUDGET, GRID] {
            assert!(
                name.starts_with("panel-pages-"),
                "{name} is not a Pages panel region"
            );
        }
    }

    /// The control chord is one this check does not otherwise depend on.
    ///
    /// ★★ `Ctrl+2` puts the application into the mode it is already in, so the
    /// probe cannot change any state an assertion reads. A probe bound to a
    /// verb with a side effect would be a setup step pretending to be a
    /// measurement.
    #[test]
    fn the_control_chord_changes_nothing_the_check_measures() {
        assert_eq!(
            CHORD_ID,
            format!("mode.{MODE}"),
            "the control chord must resolve to the mode this check is already in, or pressing it \
             is a gesture rather than a probe"
        );
    }

    /// `value_of` reads a key past the sandbox header's comments.
    ///
    /// ★ The header is comment lines and a blank; a parser that took the first
    /// `=` it saw anywhere would read one of them. This is the assertion that
    /// the independent oracle is actually independent *and* correct.
    #[test]
    fn the_file_oracle_reads_past_the_comments() {
        let text = concat!(
            "# page_preview_budget_ms: how long pdfcer may spend\n",
            "# page_previews = true\n",
            "\n",
            "page_previews = false\n",
            "page_preview_budget_ms = 0\n"
        );
        assert_eq!(value_of(text, PREVIEWS_KEY), Some("false"));
        assert_eq!(value_of(text, BUDGET_KEY), Some("0"));
        assert_eq!(value_of(text, "nothing_like_this"), None);
    }
}
