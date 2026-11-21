//! `stamp_dialog_reopen` — **the second stamp's dialog still has its Add and
//! Cancel buttons on the screen.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/stamp_dialog_reopen.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, list, shell_trace,
};
use crate::checks::scripteddrive::{ClickNamed, offscreen};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The side, in **pdf** points, of the rectangle dragged for each stamp.
///
/// Matched to `stamp_size`'s, and for its reason: large enough that the gesture
/// machine reads a drag rather than rounding it to a click.
const BOX_PT: f64 = 220.0;

/// How far, in **pdf** points, the second stamp is placed from the first.
const SECOND_OFFSET_PT: f64 = 300.0;

/// **Where the second stamp goes, given where the first one went and how big
/// the page is.**
#[must_use]
fn second_point(target: DocPoint, page: PageGeometry) -> DocPoint {
    fn axis(from: f64, extent: f64) -> f64 {
        let forward = from + SECOND_OFFSET_PT;
        let placed = if forward + BOX_PT <= extent {
            forward
        } else {
            from - SECOND_OFFSET_PT
        };
        placed.clamp(0.0, (extent - BOX_PT).max(0.0))
    }
    DocPoint {
        page: target.page,
        x: axis(target.x, page.width_pt),
        y: axis(target.y, page.height_pt),
    }
}

/// The dialog's body.
const BODY: &str = "dialog:text-annot";
/// Add. `dialogs::textannot::REGION_ACCEPT`.
const ACCEPT: &str = "text-annot.accept";
/// Cancel. `dialogs::textannot::REGION_CANCEL`, added with this check.
const CANCEL: &str = "text-annot.cancel";
/// The ribbon control that arms the stamp tool.
///
/// It is a **toggle**, not a momentary button. See [`arm_stamp`].
const STAMP_ITEM: &str = "ribbon.item.markup.stamp";

/// The trace line naming the armed markup tool. Emitted on change only.
const TOOL_EVENT: &str = "markup-tool";

/// The prefix every text-annotation tool's name carries — `TextAnnot(Stamp)`,
/// `TextAnnot(Note)`, and so on.
const TOOL_TEXT_ANNOT: &str = "TextAnnot";

/// See the module documentation.
pub struct TheSecondStampDialogStillHasItsButtons;

impl Check for TheSecondStampDialogStillHasItsButtons {
    fn name(&self) -> &'static str {
        "the_second_stamp_dialog_still_has_its_buttons"
    }

    fn defect(&self) -> &'static str {
        "the second stamp placed in one session opens an options window whose Add and Cancel \
         are below its own bottom edge — so the operator can neither finish the stamp nor \
         dismiss it except through the title bar's X"
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

/// **The tool the shell says is armed right now, or `None` if it has never
/// said.**
fn armed_tool(session: &Session) -> Result<Option<String>> {
    Ok(session
        .trace()?
        .last(TOOL_EVENT)
        .and_then(|l| l.get("tool"))
        .map(str::to_owned))
}

/// **Arm the stamp tool from the Markup tab — but only if it is not already
/// armed.**
///
///
/// It was written to click Markup > Stamp before *each* placement, on the
/// reasoning that *"markup tools disarm themselves after a placement in some
/// modes and not in others… re-arming an already-armed tool costs one click and
/// closes the question."*
///
///
/// ```text
/// egui-shell-diag ribbon-command-invoked id=markup.stamp handler=522
/// pdfcer-diag      markup-tool tool=Select
/// ```
///
/// — the command fired and the tool went to **Select**. The shell had kept the
/// stamp tool armed across the first placement (which is the right behaviour,
/// and the behaviour Acrobat has: the tool is sticky, and clicking the armed
/// tool again puts it down). So the defensive second click *disarmed* it, the
/// second drag was a text selection, no dialog opened, and the check reported
/// *"the second drag traced no further `text-annot-open`"* — a true sentence
/// about a build in which the operator's own route works.
///
/// The lesson is the standing one and it earned another instance: **a driven
/// failure is a claim about the check too.** The check's own failure message
/// offered two hypotheses — *"either the tool did not re-arm or the drag landed
/// on the first stamp"* — and the truth was a third the author had ruled out by
/// construction.
///
/// ⇒ So this now READS the state before acting. Clicking is reserved for the
/// case where the tool genuinely is not armed, which keeps the original
/// intention (do not assume stickiness) without the toggle's side effect.
///
/// # Errors
///
/// If the ribbon item is not on the tab, or the click armed nothing.
fn arm_stamp(session: &Session, driver: &ScriptedPointer, ui_rect: &str) -> Result<()> {
    // Already armed? Then there is nothing to do, and doing it would undo it.
    if armed_tool(session)?.is_some_and(|t| t.starts_with(TOOL_TEXT_ANNOT)) {
        return Ok(());
    }
    let trace = session.trace()?;
    let item =
        crate::checks::driving::declared_or_in_overflow(session, driver, ui_rect, STAMP_ITEM)?
            .ok_or_else(|| {
                Error::new(format!(
                    "no `{STAMP_ITEM}` region on the Markup tab. Items declared: {}.",
                    list(&declared_names(&trace, ui_rect, "ribbon.item.markup."))
                ))
            })?;
    driver.click(session, WindowPoint::centre_of(item))?;
    session.settle(14);
    // And say so if the click did not arm it. Without this the failure
    // surfaces one step later, as "the dialog never opened", which reads as a
    // dialog defect rather than a tool one.
    match armed_tool(session)? {
        Some(t) if t.starts_with(TOOL_TEXT_ANNOT) => Ok(()),
        Some(t) => Err(Error::new(format!(
            "clicking `{STAMP_ITEM}` left the tool at `{t}`, not \
             `{TOOL_TEXT_ANNOT}(..)`. The ribbon item is a toggle: if the tool \
             was already armed, this click put it down. Nothing below can \
             open a dialog."
        ))),
        None => Err(Error::new(format!(
            "clicking `{STAMP_ITEM}` traced no `{TOOL_EVENT}` line at all, so \
             the control armed nothing."
        ))),
    }
}

/// **Drag one stamp box.**
fn place(
    session: &Session,
    driver: &ScriptedPointer,
    ctx: &CheckContext,
    page: PageGeometry,
    at: DocPoint,
) -> Result<()> {
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, at.page)?;
    let from = mapping.doc_to_window(at)?;
    let to = mapping.doc_to_window(DocPoint {
        page: at.page,
        x: at.x + BOX_PT,
        y: at.y + BOX_PT,
    })?;
    driver.drag(session, from, to, 8)?;
    session.settle(24);
    Ok(())
}

/// **Is the answer row on the screen?** `None` when it is, a sentence when it
/// is not.
///
/// `which` names the opening — *first* or *second* — because the same words
/// standing for two different faults is how a fix gets aimed at the wrong one.
fn answer_row_missing(trace: &crate::trace::Trace, ui_rect: &str, which: &str) -> Option<String> {
    let mut absent = Vec::new();
    for (region, what) in [(ACCEPT, "Add"), (CANCEL, "Cancel")] {
        if declared(trace, ui_rect, region).is_none() {
            absent.push(format!("{what} (`{region}`)"));
        }
    }
    if absent.is_empty() {
        return None;
    }
    Some(format!(
        "the {which} stamp dialog opened and did NOT declare {}. Those regions are published \
         through `diag::ui_rect_visible`, which stays silent when the rect is outside its own \
         clip rect — so the control is on the window and off the screen, which is exactly the \
         operator's report of 2026-09-10. The answer row must be allocated out of the window's \
         rectangle BEFORE the body, never after it: see `dialogs::host::Host::scrolled`, and \
         `dialogs::host::fit` for the size the window inherited from the previous opening. \
         Regions declared: {}.",
        absent.join(" or "),
        list(&declared_names(trace, ui_rect, "text-annot"))
    ))
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no fixture document. Pass --pdf."))?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. This check needs somewhere on the page to drag, and a guessed one \
             can land off the sheet — which is symptom-identical to a drag that never \
             registered.",
        )
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("stamp_dialog_reopen.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let pointer = offscreen(ctx, &mut spec, "stamp_dialog_reopen")?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    let driver = &pointer;

    // --- A: Review mode, the Markup tab, the stamp tool --------------------
    crate::checks::driving::click_mode_segment(&session, driver, ui_rect, "review")?;
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.markup").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.markup` region after switching to Review. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click(&session, WindowPoint::centre_of(tab))?;
    session.settle(14);
    if !shell_trace(&session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some("markup"))
    {
        return Err(Error::new(
            "the click on the Markup tab produced no tab-selected line, so nothing below would \
             mean anything.",
        ));
    }
    arm_stamp(&session, driver, ui_rect)?;
    if !session
        .trace()?
        .events("markup-tool")
        .any(|l| l.get("tool").is_some_and(|t| t.contains("TextAnnot")))
    {
        return Ok(Some(
            "clicking Markup > Stamp traced no `markup-tool tool=TextAnnot(..)` line, so the \
             control armed nothing and neither dialog below can open."
                .to_owned(),
        ));
    }
    report.note("Markup > Stamp armed the annotation tool");

    // --- B/C: the FIRST opening --------------------------------------------
    place(&session, driver, ctx, page, target)?;
    let trace = session.trace()?;
    if trace.events("text-annot-open").next().is_none() {
        return Ok(Some(
            "the first drag completed and traced no `text-annot-open` line, so the release did \
             not open the stamp dialog at all. That is a different defect from O171 and it \
             blocks this check rather than being reported as it."
                .to_owned(),
        ));
    }
    if declared(&trace, ui_rect, BODY).is_none() {
        return Ok(Some(format!(
            "a `text-annot-open` line was traced and no `{BODY}` region appeared, so the dialog \
             was created and never drawn."
        )));
    }
    // Asserted on the first opening TOO, and the message says which. If the
    // row is already gone here then the operator's report is being reproduced
    // by the wrong mechanism, and a fix aimed at the reopening path would leave
    // the real cause in place.
    if let Some(failed) = answer_row_missing(&trace, ui_rect, "FIRST") {
        return Ok(Some(format!(
            "{failed}\n\n⚠ This failed on the FIRST opening. The operator's report was about \
             the SECOND — so whatever is wrong here is NOT the fit state carried between \
             openings, and `dialogs::host::fit` is the wrong place to look."
        )));
    }
    report.note("the first stamp dialog has both Add and Cancel on the screen");

    let shot = ctx.out("stamp-dialog-first.png");
    if driver.screenshot(&session, &shot).is_ok() {
        report.artifact(shot);
    }

    // --- D: press Add, which is what the operator did ----------------------
    let accept = declared(&trace, ui_rect, ACCEPT)
        .ok_or_else(|| Error::new("Add stopped being declared between two reads"))?;
    driver.click_named(&session, &trace, ui_rect, ACCEPT, accept)?;
    session.settle(24);
    if declared(&session.trace()?, ui_rect, BODY).is_some() {
        report.note(
            "the first dialog still declares a body after Add was pressed; that may simply be \
             the last frame before it closed, and the second placement below will say.",
        );
    }

    // --- E: the SECOND stamp, which is the whole point ---------------------
    arm_stamp(&session, driver, ui_rect)?;
    let second = second_point(target, page);
    report.note(format!(
        "placing a second stamp {SECOND_OFFSET_PT:.0} pt away from the first at \
         ({:.0}, {:.0}), which clears its rectangle — a drag that started inside the first \
         stamp would be read as grabbing it",
        second.x, second.y
    ));
    place(&session, driver, ctx, page, second)?;
    let trace = session.trace()?;
    if trace.events("text-annot-open").count() < 2 {
        return Ok(Some(
            "the second drag traced no further `text-annot-open` line, so the dialog never \
             opened a second time.\n\n★ Read the trace before believing any of the obvious \
             causes: the tool did not re-arm (look for a second \
             `markup-tool` line retiring it), or the drag landed on the first stamp (compare \
             the two boxes), or the press was delivered and the canvas ignored it (look for \
             `canvas-gesture started=1`)."
                .to_owned(),
        ));
    }
    if declared(&trace, ui_rect, BODY).is_none() {
        return Ok(Some(format!(
            "the second `text-annot-open` was traced and no `{BODY}` region followed it."
        )));
    }

    let shot = ctx.out("stamp-dialog-second.png");
    if driver.screenshot(&session, &shot).is_ok() {
        report.artifact(shot);
    }

    // --- F: THE ASSERTION THIS CHECK EXISTS FOR ----------------------------
    if let Some(failed) = answer_row_missing(&trace, ui_rect, "SECOND") {
        return Ok(Some(failed));
    }
    report.note("the second stamp dialog also has both Add and Cancel on the screen — O171");

    // --- G: leave by the route that authors nothing further ----------------
    let cancel = declared(&trace, ui_rect, CANCEL)
        .ok_or_else(|| Error::new("Cancel stopped being declared between two reads"))?;
    driver.click_named(&session, &trace, ui_rect, CANCEL, cancel)?;
    session.settle(16);
    driver.gone(&session)?;
    Ok(None)
}
