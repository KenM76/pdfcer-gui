//! `set_scale_reads_the_group_it_is_about_to_overwrite` — the Set-scale window
//! names its group, states the scale already set, and keeps what the operator
//! typed when they step out to point at the page.
//!
//! # The three reports this is the oracle for
//!
//! All filed by the operator on 2026-09-13, and all one structural cause:
//!
//! > **O192** — *"the set scale dialogue does not show me the scale that is
//! > already set"*
//!
//! > **O193** — *"there is no group dropdown"*
//!
//! …plus a third he had not got to yet, which this check asserts because it is
//! the one that could cost him work: pressing **Measure it on the drawing…**
//! destroyed everything already entered. The window was closed
//! (`DialogsState::close_scale`) and a fresh one built from
//! `ScaleEntryFields::default()` when the pick completed, so an operator who
//! chose metres, typed a ratio and set the number style before deciding to
//! measure a line came back to a window that had forgotten all four — and one
//! who pressed Escape mid-pick came back to no window at all.
//!
//! The cause of all three was one sentence: `ScaleDialog::show` took a context
//! and an action queue, so the window **physically could not see the document
//! it was editing**, and `dialogs::open` destructured `status` only to prove a
//! document existed and then threw it away.
//!
//! # ★★ Why every part of this needs DRIVING, and unit tests cannot stand in
//!
//! `ScaleEntryFields::for_group` — the inversion that makes O192 possible — has
//! five unit tests including an independently calibrated one, and they would
//! all pass on a build where nothing ever calls it. That is this project's
//! oldest lesson in its sharpest form: *unit tests that call the verb cannot
//! see the chain in front of it.* The chain here is six links and four are
//! frame-level:
//!
//! 1. a ribbon press reaches `open_scale`, which must now **keep** the
//!    `OpenDoc` it proves it has;
//! 2. `ScaleDialog::open` must seed from that document rather than from
//!    `for_group_panel`'s defaults;
//! 3. `DialogsState::show` must pass `doc` through, which it did not before;
//! 4. the picker and the current-scale line must be **drawn**, inside the
//!    window's body, not merely computed;
//! 5. arming the pick must **hide** rather than close;
//! 6. the pick completing must **deliver into** the surviving window rather
//!    than build a new one.
//!
//! Every one of those is an edge read once per frame. Only a running window
//! sees them.
//!
//! # ★★★ The assertion that carries the whole thing: ONE `scale-open`
//!
//! Step 7 counts `scale-open` trace lines across the entire session and
//! requires exactly one. That is the difference between *hidden* and *closed*
//! stated as a number a machine can check, and it is the only assertion here
//! that a plausible-but-wrong build cannot satisfy by accident:
//!
//! - a build that closes and rebuilds emits **two**;
//! - a build that closes and never comes back emits one, and fails step 6
//!   instead;
//! - a build that hides correctly emits one, because the constructor ran once.
//!
//! An assertion on *"the dialog is on screen afterwards"* alone is satisfied by
//! both the fixed build and the broken one, and would have passed throughout
//! the defect's life. Naming what the wrong mechanism **cannot** produce is the
//! rule this check was written under.
//!
//! # The O192 assertion is a ROUND TRIP through the real application
//!
//! Asserting that `scale.current` is drawn proves a label exists; it does not
//! prove the label says anything true, and on a fresh fixture every group is
//! uncalibrated, so a window that invented `1:100` and a window that read the
//! document would print the same defaults. So this check **sets a scale first**
//! — through the two-point calibration, typing a real length, pressing Accept —
//! and then reopens the window and requires the seeded ratio to have **moved**.
//!
//! That is the operator's exact complaint, driven: open it a second time and
//! see the number you set. A build that seeds from defaults reopens at the
//! same ratio it opened at the first time, which is what this measures.
//!
//! # What this check deliberately does NOT assert
//!
//! That the group picker lists more than one group. Creating a second group is
//! `dimension_groups_panel_makes_a_group`'s gesture and its fixture, and
//! duplicating it here would make two checks fail for one cause. What is
//! asserted is that the picker is **drawn inside the window's body** — the
//! failure mode `D:/dev/rag/egui/` records as panels that shipped unreachable
//! in real builds with every gate green.

use crate::checks::driving::{
    SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, frame_of, list, shell_trace,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// How far apart the two picked points are, in PDF points along x.
///
/// The same span `measure_calibrate` uses, and deliberately so: the two checks
/// drive the same gesture and a reader comparing their traces should not have
/// to account for a different number. See that module for why 400 pt.
const SPAN_PT: f64 = 400.0;

/// The real-world length typed into the calibrated window, as a bare number.
///
/// ★ No unit suffix and no punctuation. `Driver::type_ascii` sends key events
/// and refuses punctuation, and this crate's own note on that is a finding
/// rather than an excuse — so the value is chosen to be typeable rather than
/// realistic. The group's display unit supplies the unit, which is the ordinary
/// behaviour of the field: an operator typing `100` into a metric group means
/// a hundred metres.
///
/// ### It is APPENDED, not entered into an empty field
///
/// Measured 2026-09-13: `ScaleEntryFields::default()` seeds
/// `real_length_text` with `"1"`, and `reseed` deliberately carries that text
/// across because it belongs to the reference line rather than to the group.
/// Clicking the field puts the caret at the end, so what is actually committed
/// is **`1100`** in the group's display unit — millimetres, on a fresh
/// document.
///
/// That is recorded rather than corrected. Clearing the field would mean
/// sending Ctrl+A, and Ctrl+A on a build where focus has silently gone
/// elsewhere is *Select All* on the page — a gesture with consequences, in a
/// check whose subject is not selection. The assertion below needs the scale
/// to MOVE, not to land on a predicted number, so the exact value is not
/// load-bearing and the check keeps working if the default text ever changes.
const TYPED_REAL_LENGTH: &str = "100";

/// How far the seeded ratio must move before this check believes the window
/// read the document.
///
/// ★ A **relative** floor rather than an absolute one, and generous. The point
/// is not to predict the arithmetic — `ScaleEntryFields::for_group` has five
/// unit tests for that, including one calibrated by hand against the engine's
/// own documented formula. The point is to distinguish *"seeded from the
/// document"* from *"seeded from `for_group_panel`'s constant"*, and those two
/// differ by orders of magnitude for any real calibration.
///
/// A tight bound here would be this check re-deriving the inversion, which is
/// an oracle built from the system under test.
const RATIO_MUST_MOVE_BY: f64 = 0.10;

/// See the module documentation.
pub struct SetScaleReadsTheGroupItIsAboutToOverwrite;

impl Check for SetScaleReadsTheGroupItIsAboutToOverwrite {
    fn name(&self) -> &'static str {
        "set_scale_reads_the_group_it_is_about_to_overwrite"
    }

    fn defect(&self) -> &'static str {
        "the Set-scale window does not say which dimension group it is aimed at, does not \
         show the scale already set on it, and throws away everything typed when the \
         operator steps out to measure a line on the drawing"
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

/// The seeded ratio's real side, from the most recent `scale-seeded` line.
///
/// ★ Two plain numeric keys rather than one `1:100` field, because a check that
/// has to split a packed field is a check that can report the opposite of the
/// truth while quoting the truth in its own message. `dialogs::scale`'s
/// `reseed` emits them separately for this reader.
fn seeded_ratio_real(trace: &Trace) -> Option<f64> {
    trace
        .events("scale-seeded")
        .filter_map(|l| l.get("ratio_real"))
        .filter_map(|v| v.parse::<f64>().ok())
        .last()
}

/// Open the Set-scale window from Measure ▸ Scale on a build already in Review
/// mode with the Measure tab showing.
fn click_set_scale(session: &Session, driver: &Driver, ui_rect: &str) -> Result<()> {
    let trace = session.trace()?;
    let item = declared(&trace, ui_rect, "ribbon.item.measure.set_scale").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.item.measure.set_scale` region on the Measure tab. Items declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.item.measure."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(16);
    Ok(())
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
            "input is disabled (--no-input). This check clicks a mode segment, a ribbon tab, \
             three ribbon or dialog controls, two points on the page, and types into a field. \
             Reported as SKIPPED rather than passed: a check that did not run has learned \
             nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. This check needs somewhere on the page to measure FROM, and a \
             guessed one can land off the sheet — which is symptom-identical to a pick that \
             never registered.",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("scale_reads_the_group.trace.txt"));
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

    // --- 1: Review mode and the Measure tab --------------------------------
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, "review")?;
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.measure").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.measure` region after switching to Review. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    if !shell_trace(&session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some("measure"))
    {
        return Err(Error::new(
            "the click on the Measure tab produced no tab-selected line, so nothing below \
             would mean anything.",
        ));
    }

    // --- 2: open the window ------------------------------------------------
    click_set_scale(&session, &driver, ui_rect)?;
    let trace = session.trace()?;
    let Some(body) = declared(&trace, ui_rect, "dialog:set-scale") else {
        return Ok(Some(
            "`measure.set_scale` was clicked and no `dialog:set-scale` region appeared, so the \
             Set-scale dialog did not open."
                .to_owned(),
        ));
    };
    report.note("the Set-scale window opened from Measure > Scale");

    // --- 3: O193 — it says which group it is aimed at -----------------------
    let Some(picker) = declared(&trace, ui_rect, "scale.group") else {
        return Ok(Some(format!(
            "the Set-scale window declared no `scale.group` region, so it does not say which \
             dimension group it is about to recalibrate. This is O193 exactly — *\"there is no \
             group dropdown\"* — and it is not cosmetic: the window is opened either from the \
             ribbon (the group being drawn into) or from the Manage-groups panel (the row \
             selected there), those are routinely different groups, and an unnamed window can \
             be aimed somewhere the operator is not looking. Regions the window did declare: \
             {}.",
            list(&declared_names(&trace, ui_rect, "scale."))
        )));
    };
    if !body.contains_rect(picker) {
        return Ok(Some(format!(
            "`scale.group` was declared at {picker:?}, outside the window body at {body:?}. A \
             control drawn beyond its container is one the operator cannot click, and it is \
             the failure this suite exists for: every gate can be green while a panel ships \
             unreachable."
        )));
    }
    report.note("the group picker is drawn, inside the window body");

    // --- 4: O192 — it states the scale already set --------------------------
    let Some(current) = declared(&trace, ui_rect, "scale.current") else {
        return Ok(Some(format!(
            "the Set-scale window declared no `scale.current` region, so it does not state the \
             scale already set on the group. This is O192 — *\"the set scale dialogue does not \
             show me the scale that is already set\"*. The invisible half is worse than the \
             visible one: a window pre-filled with a plausible wrong number lets the operator \
             press Accept without touching a control and silently recalibrate the drawing. \
             Regions the window did declare: {}.",
            list(&declared_names(&trace, ui_rect, "scale."))
        )));
    };
    if !body.contains_rect(current) {
        return Ok(Some(format!(
            "`scale.current` was declared at {current:?}, outside the window body at {body:?} \
             — so the sentence stating the group's scale exists and cannot be read."
        )));
    }
    let Some(first_ratio) = seeded_ratio_real(&trace) else {
        return Ok(Some(
            "the Set-scale window opened and traced no `scale-seeded` line, so its entry \
             controls were not seeded from the document at all. The label may be drawn; the \
             numbers under it are still the constructor's defaults, which is the half of O192 \
             that can damage a file."
                .to_owned(),
        ));
    };
    report.note(format!(
        "the window seeded its entry controls from the document at ratio 1:{first_ratio}"
    ));

    // --- 5: step out to the page -------------------------------------------
    let Some(button) = declared(&trace, ui_rect, "scale.calibrate") else {
        return Ok(Some(
            "the Set-scale window declared no `scale.calibrate` region, so there is no route \
             from it into the two-point calibration."
                .to_owned(),
        ));
    };
    driver.click_at(
        frame_of(&session, &trace, ui_rect, "scale.calibrate")?.declared_center(button),
    )?;
    session.settle(16);
    let trace = session.trace()?;
    if !trace
        .events("scale-calibrate")
        .any(|l| l.get("armed") == Some("true"))
    {
        return Ok(Some(
            "the calibrate button was clicked and no `scale-calibrate armed=true` line was \
             traced, so the request never reached `app::frame` and the pick was never armed."
                .to_owned(),
        ));
    }
    if declared(&trace, ui_rect, "dialog:set-scale").is_some() {
        return Ok(Some(
            "the pick was armed and the Set-scale window is still on screen, sitting over the \
             page the operator has just been asked to click two points on. The window is \
             supposed to step aside for the duration — `ScaleDialog::hidden`."
                .to_owned(),
        ));
    }
    report.note("the window stepped aside and the two-point pick armed");

    // --- 6: pick two points ------------------------------------------------
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, target.page)?;
    for (label, doc) in [
        ("A", target),
        (
            "B",
            DocPoint {
                page: target.page,
                x: target.x + SPAN_PT,
                y: target.y,
            },
        ),
    ] {
        let window = mapping.doc_to_window(doc)?;
        let screen = session.frame()?.to_screen(window);
        report.note(format!(
            "pick {label}: document ({:.1}, {:.1}) -> screen ({}, {})",
            doc.x,
            doc.y,
            screen.x(),
            screen.y()
        ));
        driver.click_at(screen)?;
        session.settle(14);
    }
    let trace = session.trace()?;
    let Some(measured) = trace
        .events("scale-calibrate")
        .filter_map(|l| l.get("measured_pt"))
        .filter_map(|v| v.parse::<f64>().ok())
        .last()
    else {
        return Ok(Some(
            "two points were clicked on the page and no `scale-calibrate measured_pt=` line \
             was traced, so the pick never completed. Everything below is about what the \
             window does with a measurement, and there is none."
                .to_owned(),
        ));
    };
    if declared(&trace, ui_rect, "dialog:set-scale").is_none() {
        return Ok(Some(format!(
            "the pick measured {measured:.2} pt and the Set-scale window did not come back, so \
             the operator is holding a measurement with nowhere to say what it represents."
        )));
    }

    // --- 7: ★★★ HIDDEN, NOT CLOSED — the assertion the wrong build fails ----
    let opens = trace.events("scale-open").count();
    let delivered = trace.events("scale-delivered").count();
    if delivered != 1 {
        return Ok(Some(format!(
            "the pick measured {measured:.2} pt and `scale-delivered` was traced {delivered} \
             times, where it must be traced once. That event comes only from \
             `ScaleDialog::deliver_measured`, which only a window that SURVIVED the pick can \
             be sent -- so zero means the measurement went to a window built fresh from \
             defaults, discarding the unit, ratio and number style the operator had already \
             chosen. It is asserted beside the constructor count below because a count of one \
             construction, on its own, is also satisfied by a window that simply never came \
             back."
        )));
    }
    if opens != 1 {
        return Ok(Some(format!(
            "the Set-scale window's constructor ran {opens} times across one calibration, and \
             it must run once. More than one means the window was DESTROYED when the operator \
             asked to measure on the drawing and a fresh one was built from defaults when the \
             pick completed — so the unit, the ratio and the number style they had already \
             chosen were silently discarded, and pressing Escape mid-pick stranded them with \
             no window at all. This is asserted as a count rather than as *\"the window is \
             back\"* because both the fixed build and the broken one put a window back on the \
             screen; only the count tells them apart."
        )));
    }
    report.note(format!(
        "the window was hidden rather than closed: one `scale-open` across the whole \
         calibration, and it came back carrying {measured:.2} pt"
    ));

    // --- 8: commit a scale, so there is one to read back -------------------
    let trace = session.trace()?;
    let Some(field) = declared(&trace, ui_rect, "scale.real_length") else {
        return Ok(Some(format!(
            "the window came back after measuring {measured:.2} pt and declared no \
             `scale.real_length` region, so it returned on the RATIO path and the operator is \
             asked for a ratio after doing the work of picking two points."
        )));
    };
    let frame = frame_of(&session, &trace, ui_rect, "scale.real_length")?;
    driver.click_at(frame.declared_center(field))?;
    session.settle(8);
    driver.type_ascii(TYPED_REAL_LENGTH)?;
    session.settle(10);
    let trace = session.trace()?;
    let Some(accept) = declared(&trace, ui_rect, "dialog:set-scale.accept") else {
        return Ok(Some(
            "the window declared no `dialog:set-scale.accept` region, so there is no way to \
             commit the calibration it just took."
                .to_owned(),
        ));
    };
    driver.click_at(
        frame_of(&session, &trace, ui_rect, "dialog:set-scale.accept")?.declared_center(accept),
    )?;
    session.settle(20);
    let trace = session.trace()?;
    if !trace
        .events("scale-commit")
        .any(|l| l.get("group").is_some())
    {
        return Ok(Some(format!(
            "Accept was pressed on a window holding a {measured:.2} pt reference line and \
             `{TYPED_REAL_LENGTH}`, and no `scale-commit` line was traced. Without a committed \
             scale there is nothing for the window to read back, so the O192 assertion below \
             cannot be made — and this is reported as a failure rather than skipped because a \
             calibration that silently commits nothing is the defect it would be hiding."
        )));
    }
    report.note("the calibration committed, so the group now has a scale to read back");

    // --- 9: ★ O192 proper — reopen, and see the number you set --------------
    click_set_scale(&session, &driver, ui_rect)?;
    let trace = session.trace()?;
    if declared(&trace, ui_rect, "dialog:set-scale").is_none() {
        return Ok(Some(
            "the Set-scale window would not reopen after a calibration was committed.".to_owned(),
        ));
    }
    let Some(second_ratio) = seeded_ratio_real(&trace) else {
        return Ok(Some(
            "the Set-scale window reopened on a calibrated group and traced no `scale-seeded` \
             line, so it did not read the group's scale."
                .to_owned(),
        ));
    };
    let moved = (second_ratio - first_ratio).abs();
    let floor = first_ratio.abs().max(1.0) * RATIO_MUST_MOVE_BY;
    if moved <= floor {
        return Ok(Some(format!(
            "a scale was committed to the group and the Set-scale window reopened seeded at \
             ratio 1:{second_ratio}, which is where it opened before anything was calibrated \
             (1:{first_ratio}). The window is still showing its constructor's defaults rather \
             than the document's number — O192, with the round trip closed against it. \
             Pressing Accept on that window without touching a control recalibrates the \
             drawing to a number pdfcer invented."
        )));
    }
    report.note(format!(
        "reopening the window seeded it at 1:{second_ratio}, moved from 1:{first_ratio} by the \
         calibration — so the window reads the group it is about to overwrite"
    ));
    Ok(None)
}
