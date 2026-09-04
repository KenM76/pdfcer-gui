//! `double_clicking_a_text_box_edits_the_text` — **the last rung of the
//! Smart-Selector chain.**
//!
//! # The request
//!
//! `OPERATOR_REQUESTS.md` **O70**:
//!
//! > *"Selecting a text box or similar item does the same thing, but
//! > double-clicking inside the bounding box should edit the text."*
//!
//! Every other rung of that chain goes **deeper into the geometry** — a
//! container to what is in it, an object to its parts, a part to its points.
//! Text is where the chain means something else: the thing below a text object
//! is not a smaller shape, it is *the words*, and a double-click that descended
//! to a show-operator run would be technically consistent and useless.
//!
//! ## ★★ The tool is armed, and that is the convention rather than a side
//! ## effect
//!
//! Inkscape's selector switches to the text tool on this gesture; Illustrator's
//! does the same. pdfcer could place the caret without arming — its typing path
//! runs whatever tool is selected — and that would leave the operator in a
//! state no other program has: a caret blinking in the page while the arrow is
//! still the tool, so their next click means *select* when everything on screen
//! says they are typing.
//!
//! ⇒ So the check asserts **both**: the caret opened, and the tool that owns
//! carets is the one now pressed.
//!
//! ## The sequence
//!
//! | # | step | oracle |
//! |---|---|---|
//! | A | Edit mode, click the text | a selection line |
//! | B | double-click it | `text-edit-caret kind=Edit page=0 …` |
//! | C | …and the caret tool is armed | the Tool panel draws its `tool.armed` block |
//!
//! ★ Step C reads a **dock region**, not the ribbon's pressed state, and the
//! first run is why: the ribbon shows one tab at a time, so `view.tool_text`
//! was simply not on screen and the check SKIPPED on a build where the feature
//! worked. A region absent because it is on another tab looks exactly like one
//! absent because the feature is missing. The Tool panel is drawn whatever tab
//! is showing, and it is the surface an operator reads while a tool is armed.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Content editing needs it.
const MODE: &str = "edit";
/// The line the caret writes when it opens.
const CARET: &str = "text-edit-caret"; // ui-text-exempt: a trace event name, never displayed
/// The line the descent writes when it goes deeper instead.
const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed
/// The page region, so a failure can say whether a sheet was drawn at all.
const PAGE_REGION: &str = "page"; // ui-text-exempt: a trace region name, never displayed
/// **The Tool panel's armed block** — drawn only while a tool with a gesture is
/// selected, and the arrow is not one.
///
/// ★ Chosen over `ribbon.item.view.tool_text`'s pressed state after the first
/// run SKIPPED on it: the ribbon shows one tab at a time, and this check leaves
/// the operator on whichever tab the mode selector last drew, so the View row
/// is simply not on screen. A region that is absent because it is on another
/// tab looks exactly like one that is absent because the feature is missing.
///
/// The Tool panel is a dock, so it is drawn whatever tab is showing — and it is
/// the surface an operator actually reads while a tool is armed, which makes it
/// the more honest oracle as well as the reachable one.
const ARMED_BLOCK: &str = "tool.armed"; // ui-text-exempt: a trace region name

pub struct DoubleClickingATextBoxEditsTheText;

impl Check for DoubleClickingATextBoxEditsTheText {
    fn name(&self) -> &'static str {
        "double_clicking_a_text_box_edits_the_text"
    }

    fn defect(&self) -> &'static str {
        "double-clicking a piece of text descends into a show-operator run instead of putting a \
         caret in it — technically consistent with the rest of the chain and useless, because the \
         thing below a text object is the words rather than a smaller shape"
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
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is a click and a double-click.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new("no --pdf. This check needs a document with text on its first page.")
    })?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. There is deliberately no default: a double-click on empty page is \
             symptom-identical to one on a piece of text the hit test missed.",
        )
    })?;
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("double-click-text.trace.txt"));
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

    // --- A: select the text -------------------------------------------------
    let at = aim(
        ctx,
        &session,
        page,
        DocPoint::new(target.page, target.x, target.y),
    )?;
    driver.click_at(at)?;
    session.settle(25);

    let selected = session
        .trace()?
        .last(SELECTION)
        .and_then(|l| l.get("first").map(str::to_owned))
        .unwrap_or_else(|| "none".to_owned());
    if selected == "none" {
        return Err(Error::new(format!(
            "the click at ({:.0}, {:.0}) selected nothing, so there is no text box under it. \
             SKIPPED rather than failed: that is a fact about the point this run was given, and \
             a double-click on paper is symptom-identical to one the hit test missed.",
            target.x, target.y
        )));
    }
    report.note(format!("★ the click selected `{selected}`"));

    // --- B: double-click it -------------------------------------------------
    driver.double_click_at(at)?;
    session.settle(35);

    let trace = session.trace()?;
    let Some(caret) = trace.last(CARET) else {
        // ★ Say which of the two happened instead. Descending is the old
        // behaviour and is a different diagnosis from a double-click that did
        // nothing at all — the first sends a reader to `canvas::clicking`'s
        // text arm, the second to whether the gesture arrived.
        let level = trace
            .last(SELECTION)
            .and_then(|l| l.get("level").map(str::to_owned))
            .unwrap_or_else(|| "none".to_owned());
        return Ok(Some(format!(
            "★★★ NO CARET: a double-click on text produced no `{CARET}` line and the selection \
             is at rung `{level}`.\n\
             `Part` means it DESCENDED — the old behaviour, into a show-operator run, which is \
             consistent with the rest of the chain and useless, because the thing below a text \
             object is the words. `Object` means the double-click did not register as one at \
             all. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★★ the caret opened: `{}`", caret.raw));

    // --- C: …and the caret tool is armed ------------------------------------
    if declared(&session.trace()?, ui_rect, ARMED_BLOCK).is_none() {
        return Ok(Some(format!(
            "**THE CARET OPENED AND NO TOOL IS ARMED**: `{ARMED_BLOCK}` is not drawn, so \
             the Tool panel is still showing its idle row. The caret would type — this \
             shell runs its typing path whatever tool is selected — and the operator would \
             be left in a state no other program has: a caret blinking in the page while \
             the arrow is still the tool, so the next click means SELECT when everything \
             on screen says they are typing. Inkscape and Illustrator both switch tools on \
             this gesture. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★★★ …and the caret tool is armed, which is what the convention does");
    Ok(None)
}
