//! `a_thing_inside_a_wrapped_drawing_can_be_dragged` — **go inside a
//! container, drag what is in it, and the engine gets the edit.**
//!
//! # The request
//!
//! `OPERATOR_REQUESTS.md` **O70**, and before it **O53**:
//!
//! > *"if the engine is capable, I should be able to select the object and do
//! > all of the ordinary editing one would expect a GUI editor to be able to
//! > do."*
//!
//! ## What this is the second half of
//!
//! `a_click_selects_the_whole_drawing_and_a_double_click_goes_inside` proves
//! the operator can *reach* something inside a wrapped drawing. Reaching it and
//! being unable to move it is a worse state than not reaching it, because the
//! selection outline is a promise the gesture then breaks — and that was
//! exactly the state this shell shipped in for the day between the two:
//! `Refusal::InsideForm`, a worded decline, honest and useless.
//!
//! It was honest because no verb existed. `pdfcer-core` Pass 188.0 shipped six
//! (2026-08-31), and `move_objects_in_form` is the first wired.
//!
//! ## ★★★ The oracle, and why `n=` is half of it
//!
//! ```text
//! move-leaves-in-form page=0 n=1 epoch=2 disclosures=0
//! ```
//!
//! The funnel writes that line **after** the engine returns `Ok`, so its
//! presence is the engine's answer rather than the shell's intent. `n=` is the
//! operand count, and it is asserted because the interesting wrong build is not
//! one that refuses — it is one that sends the **page's** index space to a verb
//! expecting the form's. Both are `usize`; `TargetId`'s own header names that
//! failure *"in range and wrong"*, and on the operator's benchmark drawing the
//! two lists hold 129,758 and 10,256 entries, so almost every index is valid in
//! both and means something different in each.
//!
//! ⇒ A refusal would therefore be the *safe* failure. The dangerous one moves
//! something, and this check's fixture is built so that the difference is
//! visible: it has exactly one page object (the container) and three leaves, so
//! an index sent to the wrong space is out of range rather than plausible.
//!
//! ## The sequence
//!
//! | # | step | oracle |
//! |---|---|---|
//! | A | click the drawing | `selection-set … object:` — the container |
//! | B | double-click it | `smart-enter`, then `canvas-selection … first=leaf:N` |
//! | C | drag the leaf's body | `move-leaves-in-form page=0 n=1` |
//! | D | press Delete | `delete-leaves-in-form page=0 n=1` |
//!
//! ★ Step C presses the **bar itself**, not the middle of its bounding box.
//! The fixture's strokes cross, so a press at the box centre could be on a
//! different one — and `canvas::pressing::body_under` requires the press to
//! land on the selected object's own geometry before a drag is a move rather
//! than a marquee.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Content selection needs it.
const MODE: &str = "edit";
/// The selection line the LADDER writes — `canvas-selection … first=leaf:N`.
///
/// ★★ **The third time in one session that a check aimed at the wrong one of
/// two selection lines**, and it is worth pinning here rather than fixing
/// silently. This shell writes two, from two functions, for two different acts:
///
/// | line | written by | for |
/// |---|---|---|
/// | `selection-set … object=N via=…` | `SelectionState::select_only` | naming ONE target directly |
/// | `canvas-selection … first=object:N` | `canvas::trace::selection_event` | a click that walked the ladder |
///
/// The descent goes through the ladder, so it writes the second. A check that
/// reads the first sees nothing and reports the feature missing — which is what
/// this one did on its first run, while the trace four lines further down said
/// `first=leaf:0`.
const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed
/// The line `canvas::smart::enter` writes.
const ENTER: &str = "smart-enter"; // ui-text-exempt: a trace event name, never displayed
/// ★ The line this check exists to read, written by the edit funnel after the
/// engine returned `Ok`.
const MOVED: &str = "move-leaves-in-form"; // ui-text-exempt: a trace event name, never displayed
/// The refusal the shell wrote for the whole of this feature's absence.
const DECLINED: &str = "canvas-decline"; // ui-text-exempt: a trace event name, never displayed
/// The live shape preview's own line — what the operator watches while the
/// drag is in flight.
const PREVIEW: &str = "canvas-shape-preview"; // ui-text-exempt: a trace event name, never displayed
/// The line a DELETE of a form-interior object writes.
const DELETED: &str = "delete-leaves-in-form"; // ui-text-exempt: a trace event name, never displayed
/// `Delete`, for the second half of "the ordinary editing one would expect".
const VK_DELETE: u16 = 0x2E;
/// The page region, so a failure can say whether a sheet was drawn at all.
const PAGE_REGION: &str = "page"; // ui-text-exempt: a trace region name, never displayed

/// The same fixture the descent check uses — one form, three fat strokes.
const FIXTURE: &str = "../../fixtures/form-xobject.pdf";
/// Its page, as the generator writes it.
const FIXTURE_PAGE: PageGeometry = PageGeometry {
    width_pt: 400.0,
    height_pt: 300.0,
};
/// Where to click and press: on the horizontal bar, clear of the other two.
const ON_THE_BAR: (f64, f64) = (100.0, 150.0);
/// Where to drag it, as a page-space offset.
const DRAG_BY: (f64, f64) = (0.0, 40.0);

pub struct AThingInsideAWrappedDrawingCanBeDragged;

impl Check for AThingInsideAWrappedDrawingCanBeDragged {
    fn name(&self) -> &'static str {
        "a_thing_inside_a_wrapped_drawing_can_be_dragged"
    }

    fn defect(&self) -> &'static str {
        "an object inside a wrapped drawing can be selected and not moved — the outline follows \
         the pointer, the release refuses, and the operator has an affordance that promises an \
         edit the program will not make"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is two clicks and a drag.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let pdf = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "the fixture {FIXTURE} is missing. Run `python tools/gen-form-xobject-fixture.py`."
        )));
    }
    let page = crate::fixture::page_geometry(&pdf).unwrap_or(FIXTURE_PAGE);

    let mut spec = LaunchSpec::new(&exe, ctx.out("form-leaf-move.trace.txt"));
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

    // --- A and B: get inside the container ---------------------------------
    let at = aim(
        ctx,
        &session,
        page,
        DocPoint::new(0, ON_THE_BAR.0, ON_THE_BAR.1),
    )?;
    driver.click_at(at)?;
    session.settle(25);
    driver.double_click_at(at)?;
    session.settle(30);

    let trace = session.trace()?;
    if trace.events(ENTER).count() == 0 {
        return Err(Error::new(format!(
            "no `{ENTER}` line, so the double-click did not go inside the container. SKIPPED \
             rather than failed: that is `a_click_selects_the_whole_drawing_and_a_double_click_\
             goes_inside`'s subject, and this check cannot reach its own. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let selected = trace
        .last(SELECTION)
        .and_then(|l| l.get("first").map(str::to_owned))
        .filter(|first| first.starts_with("leaf:"));
    if selected.is_none() {
        return Err(Error::new(format!(
            "the descent left no LEAF selected, so there is nothing inside the container to drag. \
             SKIPPED for the reason above. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ inside the container, with something in it selected");

    // --- C: drag it --------------------------------------------------------
    let to = aim(
        ctx,
        &session,
        page,
        DocPoint::new(0, ON_THE_BAR.0 + DRAG_BY.0, ON_THE_BAR.1 + DRAG_BY.1),
    )?;
    driver.drag(at, to)?;
    session.settle(45);

    let trace = session.trace()?;
    let Some(moved) = trace.last(MOVED) else {
        // ★ Name the refusal if there was one. *"The drag was refused"* and
        // *"the drag was dropped"* send a reader to two different files, and
        // this feature's whole history is the first one.
        let refused = trace
            .events(DECLINED)
            .last()
            .map(|l| format!(" The shell declined it: `{}`.", l.raw))
            .unwrap_or_default();
        return Ok(Some(format!(
            "★★★ THE DRAG DID NOT REACH THE ENGINE: no `{MOVED}` line.{refused}\n\
             For the life of this shell that refusal was honest — no verb could address anything \
             inside a form, so `canvas::moving::eligible` answered `Refusal::InsideForm`. \
             `pdfcer-core` Pass 188.0 shipped `move_objects_in_form`; if the decline is still \
             `InsideForm`, the fork in that function has not been taken. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let n = moved.get_usize("n");
    if n != Some(1) {
        return Ok(Some(format!(
            "★★ THE WRONG NUMBER OF OPERANDS: `{}`. One leaf was selected and one must have been \
             sent. A count of zero is a command raised over an empty list; more than one means \
             the selection carried entries the descent did not put there. Trace: {}.",
            moved.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ the engine moved it inside the container: `{}`",
        moved.raw
    ));

    // ★★ **And the operator watched the SHAPE move, not a box round it** —
    // O70's last owed item, 2026-09-01.
    //
    // `canvas::shapes` read `PageObjects::objects` by paint-order index, so a
    // drag inside a container fell back to the outline ghost: the box followed
    // the pointer and the geometry did not. That was the pre-O63 behaviour for
    // one case, correct while the drag could not commit at all, and a visible
    // gap the moment it could.
    //
    // ★ `shapes=` rather than the line's presence. A preview that asked for one
    // object and built none is what a build reading the wrong list produces —
    // and it traces, because the line is written whether or not anything came
    // back.
    let previewed = session
        .trace()?
        .events(PREVIEW)
        .filter_map(|l| l.get_usize("shapes"))
        .max()
        .unwrap_or(0);
    if previewed == 0 {
        return Ok(Some(format!(
            "**THE DRAG SHOWED A BOX, NOT THE SHAPE**: every `{PREVIEW}` line reports \
             zero shapes. The move committed, so the geometry exists and the preview could \
             not find it. `canvas::shapes` reads it through `provider::object_for`, which \
             resolves either index space; a build still reading page objects by paint-order \
             index gets nothing for a leaf and falls back to the outline ghost. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ …and the drag previewed the SHAPE while it was in flight: {previewed} built"
    ));

    // --- D: …and Delete reaches it too --------------------------------------
    //
    // ★ The second of the two acts O53's sentence names — *"all of the ordinary
    // editing one would expect"* — and the one with teeth, because a delete
    // that addressed the wrong index space would remove something the operator
    // did not point at. Driven here rather than in a check of its own because
    // it needs the identical four steps to get inside the container, and a
    // second launch to press one key would be twenty seconds for one assertion.
    driver.press(VK_DELETE)?;
    session.settle(40);

    let trace = session.trace()?;
    let Some(deleted) = trace.last(DELETED) else {
        return Ok(Some(format!(
            "**MOVED BUT NOT DELETABLE**: `{MOVED}` was traced and `{DELETED}` was not, \
             so half of the ordinary editing reaches a thing inside a container and half \
             does not. `canvas::keys` reads `deletable_objects_on`, which answers about \
             the page own paint order and drops every form-interior target; the arm that \
             falls through to `leaf_indices_on` is what closes that. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if deleted.get_usize("n") != Some(1) {
        return Ok(Some(format!(
            "★★ THE DELETE SENT THE WRONG NUMBER OF OPERANDS: `{}`. Trace: {}.",
            deleted.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ …and Delete reaches it as well: `{}`",
        deleted.raw
    ));
    Ok(None)
}
