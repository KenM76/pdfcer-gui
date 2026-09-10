//! `stamp_dialog_reopen` — **the second stamp's dialog still has its Add and
//! Cancel buttons on the screen.**
//!
//! # The report this exists for, verbatim
//!
//! `OPERATOR_REQUESTS.md` **O171**, 2026-09-10:
//!
//! > *"After I place the first stamp and go to make a second one the window for
//! > the options pops up but is undersized so I can't see the add or cancel
//! > button. Those buttons should always be available, and if there isn't size
//! > for all the features they get scrolled in their own space."*
//!
//! Two facts in one sentence, and the second is the more important:
//!
//! 1. **The defect is on the SECOND open, not the first.** The window carries
//!    fitting state between openings; the first opening sized itself correctly
//!    and the second inherited a size that no longer fitted the body. A check
//!    that opened the dialog once — and every existing stamp check opens it
//!    once — is structurally incapable of seeing this. That is why this check
//!    exists alongside `stamp_size` rather than inside it.
//! 2. **A dialog whose Accept is off-screen is a transaction that cannot be
//!    finished, only abandoned.** It is not a cosmetic complaint. There is no
//!    keyboard route to Add, so the operator's only exit was the title bar's X.
//!
//! # ★★★ Why this can assert with no size arithmetic anywhere in the file
//!
//! Both buttons are published through `diag::ui_rect_visible`, which emits
//! **nothing** when the rect falls outside its own clip rect. So:
//!
//! > **"was the region declared?" and "was the control on the screen?" are the
//! > same question**, and this check never computes, compares or guesses a
//! > single number.
//!
//! ⚠ That is a property of the application, not of this file, and it was made
//! true on 2026-09-10 in the same revision as this check: `REGION_ACCEPT` used
//! to be a plain `ui_rect`, which publishes a rectangle whether or not anyone
//! can see it. A check reading *that* would have found Accept "declared" on the
//! exact build where the operator could not press it, then clicked a point
//! below the window's own bottom edge — a plausible number, no error anywhere,
//! and a live defect reported as working. `REGION_CANCEL` did not exist at all;
//! his sentence names both buttons, and a harness that could see one of the two
//! would report the row as reachable on a build where half of it had been
//! clipped away.
//!
//! # Phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | Review mode, Markup tab, arm **Stamp** | `markup-tool tool=TextAnnot(..)` |
//! | B | drag a box | `text-annot-open`, and `dialog:text-annot` declared |
//! | C | read the answer row | `text-annot.accept` **and** `text-annot.cancel` declared |
//! | D | press Add | the dialog closes |
//! | E | arm Stamp again, drag a **second** box elsewhere on the page | the dialog opens a second time |
//! | F | read the answer row again | both regions declared — **this is O171** |
//! | G | press Cancel | the check leaves the document as it found it, bar one stamp |
//!
//! ★ Phase C is not redundant with phase F. If the row is missing on the *first*
//! open too then the defect is not the one the operator reported and the fix
//! that was made would be the wrong fix — the message says so, rather than
//! letting the same failure text stand for two different faults.
//!
//! # Rule 15
//!
//! [`BOX_PT`] and [`SECOND_OFFSET_PT`] are **pdf dimensions** — coordinates in
//! the CAD-exported page's own space, used to aim the pointer. Nothing in this
//! module authors a ce dimension.

use crate::checks::driving::{
    SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, frame_of, list, shell_trace,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The side, in **pdf** points, of the rectangle dragged for each stamp.
///
/// Matched to `stamp_size`'s, and for its reason: large enough that the gesture
/// machine reads a drag rather than rounding it to a click.
const BOX_PT: f64 = 220.0;

/// How far, in **pdf** points, the second stamp is placed from the first.
///
/// ★ It has to miss the first stamp's rectangle. A second drag that starts
/// **inside** an annotation that already exists is a different gesture — the
/// canvas reads it as grabbing that object — and the dialog would then never
/// open, which this check would report as O171 recurring. A harness with a bad
/// input produces defects that do not exist.
const SECOND_OFFSET_PT: f64 = 300.0;

/// The dialog's body.
const BODY: &str = "dialog:text-annot";
/// Add. `dialogs::textannot::REGION_ACCEPT`.
const ACCEPT: &str = "text-annot.accept";
/// Cancel. `dialogs::textannot::REGION_CANCEL`, added with this check.
const CANCEL: &str = "text-annot.cancel";
/// The ribbon control that arms the stamp tool.
const STAMP_ITEM: &str = "ribbon.item.markup.stamp";

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

/// **Arm the stamp tool from the Markup tab.**
///
/// Called before *each* placement rather than once, and that is deliberate: the
/// markup tools disarm themselves after a placement in some modes and not in
/// others, and a check that assumed the wrong one would report *"the second
/// dialog never opened"* about a shell behaving exactly as designed. Re-arming
/// an already-armed tool costs one click and closes the question.
///
/// # Errors
///
/// If the ribbon item is not on the tab, or the click armed nothing.
fn arm_stamp(session: &Session, driver: &Driver, ui_rect: &str) -> Result<()> {
    let trace = session.trace()?;
    let item = declared(&trace, ui_rect, STAMP_ITEM).ok_or_else(|| {
        Error::new(format!(
            "no `{STAMP_ITEM}` region on the Markup tab. Items declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.item.markup."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(14);
    Ok(())
}

/// **Drag one stamp box.**
///
/// It does not assert that the dialog opened — each caller says that in its own
/// words, because *"the FIRST drag opened nothing"* and *"the SECOND drag
/// opened nothing"* are different defects and one shared sentence standing for
/// both is how a fix gets aimed at the wrong one.
///
/// # Errors
///
/// If the canvas mapping cannot be read, or the pointer cannot be driven.
fn place(
    session: &Session,
    driver: &Driver,
    ctx: &CheckContext,
    page: PageGeometry,
    at: DocPoint,
) -> Result<()> {
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, at.page)?;
    let frame = session.frame()?;
    let from = frame.to_screen(mapping.doc_to_window(at)?);
    let to = frame.to_screen(mapping.doc_to_window(DocPoint {
        page: at.page,
        x: at.x + BOX_PT,
        y: at.y + BOX_PT,
    })?);
    driver.drag(from, to)?;
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
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a ribbon control, drags twice on \
             the canvas and presses buttons in a dialog. Reported as SKIPPED rather than passed.",
        ));
    }
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

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- A: Review mode, the Markup tab, the stamp tool --------------------
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, "review")?;
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.markup").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.markup` region after switching to Review. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
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
    arm_stamp(&session, &driver, ui_rect)?;
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
    place(&session, &driver, ctx, page, target)?;
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
    // ★ Asserted on the first opening TOO, and the message says which. If the
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
    if crate::capture::window_to_png(&session, &shot).is_ok() {
        report.artifact(shot);
    }

    // --- D: press Add, which is what the operator did ----------------------
    let accept = declared(&trace, ui_rect, ACCEPT)
        .ok_or_else(|| Error::new("Add stopped being declared between two reads"))?;
    driver.click_at(frame_of(&session, &trace, ui_rect, ACCEPT)?.declared_center(accept))?;
    session.settle(24);
    if declared(&session.trace()?, ui_rect, BODY).is_some() {
        report.note(
            "the first dialog still declares a body after Add was pressed; that may simply be \
             the last frame before it closed, and the second placement below will say.",
        );
    }

    // --- E: the SECOND stamp, which is the whole point ---------------------
    arm_stamp(&session, &driver, ui_rect)?;
    let second = DocPoint {
        page: target.page,
        x: target.x + SECOND_OFFSET_PT,
        y: target.y + SECOND_OFFSET_PT,
    };
    report.note(format!(
        "placing a second stamp {SECOND_OFFSET_PT:.0} pt away from the first, which clears its \
         rectangle — a drag that started inside the first stamp would be read as grabbing it"
    ));
    place(&session, &driver, ctx, page, second)?;
    let trace = session.trace()?;
    if trace.events("text-annot-open").count() < 2 {
        return Ok(Some(
            "the second drag traced no further `text-annot-open` line, so the dialog never \
             opened a second time. Either the tool did not re-arm or the drag landed on the \
             first stamp instead of on empty paper."
                .to_owned(),
        ));
    }
    if declared(&trace, ui_rect, BODY).is_none() {
        return Ok(Some(format!(
            "the second `text-annot-open` was traced and no `{BODY}` region followed it."
        )));
    }

    let shot = ctx.out("stamp-dialog-second.png");
    if crate::capture::window_to_png(&session, &shot).is_ok() {
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
    driver.click_at(frame_of(&session, &trace, ui_rect, CANCEL)?.declared_center(cancel))?;
    session.settle(16);
    Ok(None)
}
