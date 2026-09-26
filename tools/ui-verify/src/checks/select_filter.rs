//! `select_filter_changes_what_a_click_hits` — the filter is load-bearing, not
//! decorative.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/select_filter.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// The mode this drives. See the module docs.
const MODE: &str = "edit";

/// The Select button on the status bar.
const FILTER_BUTTON: &str = "status-group:filter";
/// The popup's All button.
const FILTER_ALL: &str = "status-filter-all";
/// The popup's None button.
const FILTER_NONE: &str = "status-filter-none";

/// The canvas's own report of how many things are selected.
const SELECTION_EVENT: &str = "canvas-selection";

/// See the module documentation.
pub struct SelectFilterChangesWhatAClickHits;

impl Check for SelectFilterChangesWhatAClickHits {
    fn name(&self) -> &'static str {
        "select_filter_changes_what_a_click_hits"
    }

    fn defect(&self) -> &'static str {
        "the Select filter is drawn and does not gate the hit test — switching every class off \
         still lets a click select an object, so the control is decorative"
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
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. This check needs a document with objects on it."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space naming a point that sits ON AN \
             OBJECT — a click on blank paper selects nothing whatever the filter says, which \
             would make this check pass without measuring anything.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, the canvas, and \
             two buttons inside a popup. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("select-filter.trace.txt"));
    spec.pdf = Some(pdf.clone());
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
    session.settle(40);
    let driver = Driver::new(session.window());

    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(24);

    // Where on screen the object is. Computed once and reused for all three
    // clicks, so the three are genuinely the same pixel.
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let at = session
        .frame()?
        .to_screen(mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?);

    // --- 0: NORMALISE, because the filter is PERSISTED ----------------------
    //
    // This step exists because its absence made the check poison its own
    // next run, and the second run's failure accused the fixture.
    //
    // The filter is an operator preference and is written to
    // `userdata/select-filter.txt` the moment it changes — correctly, and by
    // design. So a run of this check that ends anywhere other than "All" —
    // including one that FAILS at step 2 and returns early — leaves the next
    // run starting with every class switched off. Step 1 then finds nothing
    // selected and reports SKIPPED with a message blaming --doc-point, which
    // is a confident and completely wrong diagnosis.
    //
    // The general rule this is an instance of: **a driven check that mutates
    // persisted state must establish that state at the START, not assume the
    // default and tidy up at the end.** Tidying at the end only runs when the
    // check passes, which is the case that did not need it.
    set_filter(&session, &driver, ui_rect, FILTER_ALL)?;

    // --- 1: the control point ----------------------------------------------
    driver.click_at(at)?;
    session.settle(24);
    if !selected(&session)? {
        return Err(Error::new(format!(
            "the click at (page {}, {:.1}, {:.1}) selected nothing even with every class \
             enabled, so there is nothing for the filter to take away. That is a fact about the \
             fixture and the point rather than about the filter — aim --doc-point at an object. \
             SKIPPED.",
            target.page + 1,
            target.x,
            target.y,
        )));
    }
    report.note("with every class on, the click selects".to_owned());

    // --- 2: switch everything off ------------------------------------------
    set_filter(&session, &driver, ui_rect, FILTER_NONE)?;
    // Clicking the canvas with nothing selectable must also CLEAR what is
    // already selected — a click on what is now empty paper deselects
    // (convention C6). So this both re-clicks and checks the clear.
    driver.click_at(at)?;
    session.settle(24);
    if selected(&session)? {
        return Ok(Some(format!(
            "★★ THE FILTER IS DECORATIVE. With every class switched off in Select, a click at \
             (page {}, {:.1}, {:.1}) still selects an object. The popup is drawn and the hit \
             test does not consult it — check that `canvas::input::probe` is being handed the \
             filter and that `topmost_allowed` is what answers, rather than \
             `CanvasTargetProvider::hit_test`.",
            target.page + 1,
            target.x,
            target.y,
        )));
    }
    report.note("with every class off, the same click selects nothing".to_owned());

    // --- 3: and back ---------------------------------------------------------
    set_filter(&session, &driver, ui_rect, FILTER_ALL)?;
    driver.click_at(at)?;
    session.settle(24);
    if !selected(&session)? {
        return Ok(Some(format!(
            "★ SWITCHING THE CLASSES BACK ON DID NOT RESTORE SELECTION. The click at (page {}, \
             {:.1}, {:.1}) selected before the filter was touched and does not now, so step 2's \
             result was not the filter working — selection is broken, and a check that stopped \
             after step 2 would have reported this build as passing.",
            target.page + 1,
            target.x,
            target.y,
        )));
    }
    report.note("switching them back on restores it".to_owned());

    Ok(None)
}

/// Is anything selected on the canvas right now?
fn selected(session: &Session) -> Result<bool> {
    let trace = session.trace()?;
    let Some(line) = trace.events(SELECTION_EVENT).last() else {
        // No gesture has reported a selection at all this run. Not selected.
        return Ok(false);
    };
    Ok(line
        .get("sel")
        .and_then(|n| n.parse::<usize>().ok())
        .is_some_and(|n| n > 0))
}

/// Open the Select popup, click one of its two whole-set buttons, and close it.
fn set_filter(session: &Session, driver: &Driver, ui_rect: &str, button: &str) -> Result<()> {
    let frame = session.frame()?;

    let trace = session.trace()?;
    let open_at =
        crate::checks::driving::declared(&trace, ui_rect, FILTER_BUTTON).ok_or_else(|| {
            Error::new(format!(
                "the status bar never published `{FILTER_BUTTON}`, so the Select button cannot \
                 be aimed at. Either the bar is not drawn (no document open?) or the region was \
                 renamed."
            ))
        })?;
    driver.click_at(frame.declared_center(open_at))?;
    session.settle(24);

    let trace = session.trace()?;
    let target = crate::checks::driving::declared(&trace, ui_rect, button).ok_or_else(|| {
        Error::new(format!(
            "the Select popup never published `{button}`. Clicking the button did not open the \
             popup — which is defect O17's first shape, a second `Popup::toggle_id` beside \
             `Popup::menu` cancelling it."
        ))
    })?;
    driver.click_at(frame.declared_center(target))?;
    session.settle(24);
    Ok(())
}
