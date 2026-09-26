//! `chunk_boxes_show_what_a_text_block_is_made_of` — **the boxes O215 asks
//! for, driven: they appear on a click, the switch turns them off, and the
//! switch turns them back on.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/text_chunks.md`.

use crate::checks::driving::{
    INVOKE_EVENT, SHELL_DIAG_ENV, TAB_EVENT, VIEW_TAB, click_mode_segment, declared,
    declared_names, declared_or_in_overflow, list, shell_trace,
};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The mode whose canvas selects page content. Read refuses the click by
/// design, and a check that skipped this step would report the mode gate as a
/// selection defect.
pub(crate) const MODE: &str = "edit";

/// The page region, so a failure can say whether a sheet was drawn at all.
pub(crate) const PAGE_REGION: &str = "page"; // ui-text-exempt: a trace region name, never displayed

/// `canvas-chunks page=N drawn=N` — written when boxes were painted.
pub(crate) const DRAWN_EVENT: &str = "canvas-chunks"; // ui-text-exempt: a trace event name, never displayed

/// `canvas-chunks-declined reason=…` — written when none were.
pub(crate) const DECLINED_EVENT: &str = "canvas-chunks-declined"; // ui-text-exempt: a trace event name, never displayed

/// `text-chunks enabled=…` — written by `canvas::chunks::set_enabled`, whose
/// only caller is the dispatch arm an operator's press runs.
pub(crate) const SWITCH_EVENT: &str = "text-chunks"; // ui-text-exempt: a trace event name, never displayed

/// `canvas-selection … first=` — read only to tell *the click missed* from
/// *the boxes are broken* when the painter says nothing.
pub(crate) const SELECTION_EVENT: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed

/// The ribbon item that carries the toggle.
pub(crate) const TOGGLE_REGION: &str = "ribbon.item.view.text_chunks";
/// The command id the shell reports for it.
pub(crate) const TOGGLE_ID: &str = "view.text_chunks";

/// How many chunks `fixtures/paragraph.pdf`'s one text object holds.
///
/// Six `Tm`-led `Tj` operators on six baselines 16 pt apart. An equality rather
/// than a floor, for the reason in the header.
pub(crate) const EXPECTED_CHUNKS: usize = 6;

/// What the painter last said about the chunk boxes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// `canvas-chunks page=N drawn=N`.
    Drawn {
        /// The page it painted on.
        page: usize,
        /// How many boxes.
        count: usize,
    },
    /// `canvas-chunks-declined reason=…`, carrying a `canvas::chunks::Declined`.
    Declined(String),
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Drawn { page, count } => write!(f, "{DRAWN_EVENT} page={page} drawn={count}"),
            Self::Declined(reason) => write!(f, "{DECLINED_EVENT} reason={reason}"),
        }
    }
}

/// The newest of the two lines after `after`, or `None` if neither was written.
pub(crate) fn verdict(trace: &Trace, after: usize) -> Option<Verdict> {
    let drawn = trace.last_after(DRAWN_EVENT, after);
    let declined = trace.last_after(DECLINED_EVENT, after);
    let newest = match (drawn, declined) {
        (Some(d), Some(x)) if x.lineno > d.lineno => x,
        (Some(d), _) => d,
        (None, Some(x)) => x,
        (None, None) => return None,
    };
    if newest.event == DRAWN_EVENT {
        return Some(Verdict::Drawn {
            page: newest.get_usize("page").unwrap_or(usize::MAX),
            count: newest.get_usize("drawn").unwrap_or(0),
        });
    }
    Some(Verdict::Declined(
        newest.get("reason").unwrap_or("unstated").to_owned(),
    ))
}

pub struct TheChunkBoxesShowWhatATextBlockIsMadeOf;

impl Check for TheChunkBoxesShowWhatATextBlockIsMadeOf {
    fn name(&self) -> &'static str {
        "chunk_boxes_show_what_a_text_block_is_made_of"
    }

    fn defect(&self) -> &'static str {
        "clicking a block of text shows nothing about what it is made of, so the chunk a drag \
         will pick up is an invisible target and the same gesture takes the chunk one time and \
         the whole block the next — or the switch that governs the boxes is registered and \
         cannot be reached, or reaches the dispatch and changes nothing on the page"
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

/// How many times the **shell** has reported [`TOGGLE_ID`] invoked.
///
/// Read from [`shell_trace`]: `ribbon-command-invoked` is `egui-shell`'s line,
/// and `Session::trace` parses only the application's vocabulary.
pub(crate) fn toggle_invokes(session: &Session) -> Result<usize> {
    Ok(shell_trace(session)?
        .events(INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(TOGGLE_ID))
        .count())
}

/// Press the ribbon toggle once, and answer whether two of the three
/// subsystems agree it was pressed. `Ok(Some(_))` is a failure message.
pub(crate) fn press_the_toggle(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    want: bool,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    // The tab first, and only if the item is not already on the band: a tab
    // click is cheap but it is not free.
    if declared(&session.trace()?, ui_rect, TOGGLE_REGION).is_none() {
        let trace = session.trace()?;
        let Some(tab) = declared(&trace, ui_rect, VIEW_TAB.0) else {
            return Ok(Some(format!(
                "no `{}` region, so the tab carrying the toggle is not on screen and the pointer \
                 route to it cannot be measured. Tabs declared: {}.",
                VIEW_TAB.0,
                list(&declared_names(&trace, ui_rect, "ribbon.tab."))
            )));
        };
        driver.click_at(session.frame()?.declared_center(tab))?;
        session.settle(14);
        if !shell_trace(session)?
            .events(TAB_EVENT)
            .any(|l| l.get("tab") == Some(VIEW_TAB.1))
        {
            return Ok(Some(format!(
                "the click on `{}` produced no `{TAB_EVENT} tab={}` line, so the View tab did \
                 not open.",
                VIEW_TAB.0, VIEW_TAB.1
            )));
        }
    }

    let Some(item) = declared_or_in_overflow(session, driver, ui_rect, TOGGLE_REGION)? else {
        return Ok(Some(format!(
            "★★★ THE SWITCH IS REGISTERED AND UNREACHABLE: the View tab declares no \
             `{TOGGLE_REGION}`, on the band, in a collapsed group or in the overflow. The \
             command exists — `shell::commands::catalog::view` registers it and the ledger \
             counts it — so this is a manifest that never names it, which is the one shape \
             **R8** cannot catch by itself: registering a command is how the GUI learns a \
             capability exists, and an item nobody wrote is a capability nobody can use. Items \
             declared: {}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "ribbon.item.view."
            ))
        )));
    };

    // Before/after rather than "did it ever happen": this check presses the
    // same control up to three times in one run.
    let invokes_before = toggle_invokes(session)?;
    let mark = session.trace()?.mark();
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(22);

    if toggle_invokes(session)? <= invokes_before {
        return Ok(Some(format!(
            "★★ THE CONTROL DID NOT TAKE THE CLICK: no new `{INVOKE_EVENT} id={TOGGLE_ID}` line \
             after a click on `{TOGGLE_REGION}`. The item is drawn and the pointer landed on it, \
             so either the item names a command the registry does not hold — in which case the \
             shell drops it and the rect belongs to something else — or its enablement condition \
             is false and it is greyed. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let trace = session.trace()?;
    let Some(switched) = trace.last_after(SWITCH_EVENT, mark) else {
        return Ok(Some(format!(
            "★★★ THE COMMAND ARRIVED AND CHANGED NOTHING: `{INVOKE_EVENT} id={TOGGLE_ID}` was \
             traced and no `{SWITCH_EVENT} enabled=…` line followed it. The dispatch is the only \
             caller of `canvas::chunks::set_enabled`, so the id reached the shell and no arm \
             took it — `app::dispatch::navigate::handles` claiming an id its `match` does not \
             answer is exactly this shape. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let wanted = if want { "true" } else { "false" };
    if switched.get("enabled") != Some(wanted) {
        return Ok(Some(format!(
            "★★ THE SWITCH WENT THE WRONG WAY: expected `{SWITCH_EVENT} enabled={wanted}` and \
             got `{}`. The arm toggles the LIVE answer, so a build reading the persisted one \
             here flips against a value the once-a-frame mirror is about to overwrite. Trace: \
             {}.",
            switched.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!("★ the toggle was pressed: `{}`", switched.raw));
    Ok(None)
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is one click on the page and two or \
             three on the ribbon, and it needs the pointer and the foreground. Reported as \
             SKIPPED rather than passed: a check that did not run has learned nothing.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    // PINNED: `--pdf` and `--doc-point` are read and IGNORED. The header says
    // why the document has to be this one.
    let (pdf, target) = crate::fixture::text_point_target();
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the text fixture is not at {}. It is committed to this repository, so an absence is \
             a broken checkout rather than an unavailable precondition, and is reported as a \
             failure for that reason — a SKIP would say the opposite.",
            pdf.display()
        )));
    }
    report.note(format!(
        "--pdf and --doc-point are IGNORED: this check pins {} at page 0, {:.0}, {:.0} — inside \
         the first of six lines in one text object",
        pdf.display(),
        target.x,
        target.y
    ));
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new("could not read a page size from the fixture. Pass --page-size.")
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("text-chunks.trace.txt"));
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
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    session.settle(45);
    let driver = Driver::new(session.window());

    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "no `{PAGE_REGION}` region, so no sheet is on screen. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // --- A: what state did this run START in? -------------------------------
    //
    // Unanchored, and the header argues it: the painter runs every frame from
    // launch, so the newest of the two lines IS the current state. The switch
    // is persisted beside the exe, so a previous run that ended with the boxes
    // off is a fact about the machine and not about the build.
    let start = verdict(&session.trace()?, 0).ok_or_else(|| {
        Error::new(format!(
            "the painter has said nothing about the chunk boxes since launch — neither \
             `{DRAWN_EVENT}` nor `{DECLINED_EVENT}`. `canvas::painting::draw` calls \
             `draw_chunks` unconditionally, so a silent channel means the canvas is not being \
             painted at all, which is a precondition failure rather than a verdict on this \
             feature. Trace: {}.",
            session.trace_path().display()
        ))
    })?;
    if start == Verdict::Declined("switched-off".to_owned()) {
        report.note(
            "the boxes were OFF at launch — a previous run of this check, or the operator, left \
             the preference that way. Turning them on before measuring; they are left ON however \
             this check ends.",
        );
        if let Some(failure) = press_the_toggle(&session, &driver, ui_rect, true, report)? {
            return Ok(Some(failure));
        }
    }

    // --- B: a click on the text must draw one box per chunk ------------------
    let at = aim(
        ctx,
        &session,
        page,
        DocPoint::new(target.page, target.x, target.y),
    )?;
    let mark = session.trace()?.mark();
    driver.click_at(at)?;
    session.settle(28);

    let trace = session.trace()?;
    let Some(after_click) = verdict(&trace, mark) else {
        let selected = trace
            .last(SELECTION_EVENT)
            .map_or_else(|| "nothing at all".to_owned(), |l| l.raw.clone());
        return Ok(Some(format!(
            "★★★ THE CLICK CHANGED NOTHING ABOUT THE BOXES: neither `{DRAWN_EVENT}` nor \
             `{DECLINED_EVENT}` was written after the click, so the painter's answer is still \
             the one it gave before the gesture, with nothing selected. The selection line says: \
             `{selected}`. If that reports a selection, the painter is not asking \
             `canvas::chunks::outlines`; if it reports none, the click did not select the text. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    match &after_click {
        Verdict::Drawn { page: on, count } if *count == EXPECTED_CHUNKS && *on == target.page => {
            report.note(format!(
                "★★ the click drew one box per chunk: `{after_click}` — the six lines this \
                 fixture's single text object is written in"
            ));
        }
        Verdict::Drawn { page: on, .. } => {
            return Ok(Some(format!(
                "★★ THE BOXES ARE WRONG, NOT ABSENT: `{after_click}`, and this fixture's one \
                 text object holds {EXPECTED_CHUNKS} chunks on page {}.
\
                 `drawn=0` specifically means `overlay::draw_chunk_boxes` was reached and its \
                 loop never ran — the count comes back out of that loop — so the boxes were \
                 computed and none reached the canvas. Any other count is a box walk that \
                 stopped early, or an engine whose line granularity has moved: \
                 `ObjectModelProvider::text_line_count_of` is what `chunks::outlines` asks, and \
                 `text_line_bounds_canvas_of` is what may be answering `None` for some of them. \
                 Painted on page {on}. Trace: {}.",
                target.page,
                session.trace_path().display()
            )));
        }
        Verdict::Declined(reason) => {
            return Ok(Some(format!(
                "★★★ THE CLICK SELECTED AND NO BOXES APPEARED: `{after_click}`.\n\
                 `single-chunk` means the object under the pointer is one line, so this fixture \
                 has changed and the check is measuring nothing. `not-text` means the click \
                 landed on something that is not a text object. `nothing-selected` means the \
                 click did not select at all. `no-provider` means the page decomposition was not \
                 available on that frame, which is the program. Reason: `{reason}`. Trace: {}.",
                session.trace_path().display()
            )));
        }
    }

    // --- C: the switch must turn them off ------------------------------------
    let mark = session.trace()?.mark();
    if let Some(failure) = press_the_toggle(&session, &driver, ui_rect, false, report)? {
        return Ok(Some(failure));
    }
    let off = verdict(&session.trace()?, mark);
    if off != Some(Verdict::Declined("switched-off".to_owned())) {
        // Restore before reporting. This check leaves the boxes on however it
        // ends, and a failure here is exactly the run that would otherwise
        // poison the next one.
        let _ = press_the_toggle(&session, &driver, ui_rect, true, report);
        return Ok(Some(format!(
            "★★★ THE SWITCH DOES NOT GOVERN THE BOXES: the command was invoked, the application \
             wrote `{SWITCH_EVENT} enabled=false`, and the painter's answer is {}.\n\
             The two homes have come apart — `canvas::painting::draw_chunks` reads the LIVE \
             answer through `chunks::enabled`, and a build reading `Prefs::text_chunks` instead \
             is one restart behind its own switch, which looks to an operator like a control \
             that does nothing until tomorrow. Trace: {}.",
            off.map_or_else(
                || format!("silence — neither `{DRAWN_EVENT}` nor `{DECLINED_EVENT}` since"),
                |v| format!("`{v}`")
            ),
            session.trace_path().display()
        )));
    }
    report.note("★★★ the switch turned them off, and the painter honoured it on the next frame");

    // --- D: …and back on, which is also how this check cleans up -------------
    let mark = session.trace()?.mark();
    if let Some(failure) = press_the_toggle(&session, &driver, ui_rect, true, report)? {
        return Ok(Some(failure));
    }
    let back = verdict(&session.trace()?, mark);
    if back
        != Some(Verdict::Drawn {
            page: target.page,
            count: EXPECTED_CHUNKS,
        })
    {
        return Ok(Some(format!(
            "★★ THE BOXES DID NOT COME BACK: the switch was pressed a second time and the \
             painter's answer is {}. A toggle that only turns something off is a control an \
             operator presses once and then restarts the program to undo — and the selection has \
             not changed, so there is nothing else for the difference to be about. Trace: {}.",
            back.map_or_else(|| "silence".to_owned(), |v| format!("`{v}`")),
            session.trace_path().display()
        )));
    }
    report.note("★ …and a second press brought them back, leaving the switch as it was found");
    Ok(None)
}
