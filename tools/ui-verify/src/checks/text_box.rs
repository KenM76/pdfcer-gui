//! `text_box_takes_a_paragraph` — **multi-line text**, driven end to end.
//!
//! # What this is for
//!
//! The operator, 2026-08-21: *"I should be able to make it multi line."*
//!
//! Until then a text draft was one line and Enter **committed** it, so there
//! was no keystroke that could produce a second line and no gesture that could
//! ask for one.
//!
//! # ★★ Why multi-line needs a box, which is what this check is really about
//!
//! **A PDF has no paragraph.** Each visual line is its own show operator at its
//! own absolute position, so something has to decide where the second line
//! starts — a width to wrap against and a leading to step by. `pdfcer-core`'s
//! `AddTextRequest::with_box` is that, and it needs a rectangle.
//!
//! So the gesture is a **drag**, and the chain has four links no unit test can
//! reach:
//!
//! | # | link | its own test |
//! |---|---|---|
//! | 1 | a drag with the text tool becomes `DragKind::TextBox` rather than a marquee or a sweep | `gesture::meaning` — the decision, not the routing |
//! | 2 | the release converts the band to a page rect and opens a draft anchored to it | nothing |
//! | 3 | **plain Enter INSERTS instead of committing** | nothing — and this is the link that would ship |
//! | 4 | Ctrl+Enter commits, and the wrap rectangle reaches the engine | `canvas::textedit` asserts the ACTION; nothing asserts the engine saw it |
//!
//! ★ **Link 3 is the one that would fail silently and plausibly.** If Enter
//! still commits inside a box, the first Enter ends the draft and everything
//! typed after it goes nowhere — which from a chair is *"multi-line does not
//! work"*, with a perfectly ordinary single-line run left on the page as
//! evidence that something happened.
//!
//! # The oracle carries the LINE COUNT
//!
//! `add-text … n=<hard newlines>`, and that number is the point: a build where
//! Enter committed authors `n=1` and one where the newline survived authors
//! `n=2`. A line saying only *"text was added"* is identical for both, which is
//! `DEFECTS.md` D14's rule — **a trace line must carry the number a wrong build
//! would get wrong.**
//!
//! It counts **hard** newlines rather than laid-out lines, deliberately: the
//! engine wraps to the box's width using the face's own metrics and this
//! harness does not have them, so a wrapped count would be a number neither
//! side could check.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::save_copy::{click_command, click_tab};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode whose canvas may author page content.
const MODE: &str = "edit";
/// The **Edit tab**, which is where the Add-text control lives.
///
/// ★ A mode is not a tab. `click_mode_segment` puts the shell in Edit *mode*,
/// which is what decides `edit_content` — and leaves whichever tab was already
/// showing. The control has to be reached on its own tab, and the first run of
/// this check found that out by reporting the control as undeclared.
const EDIT_TAB: (&str, &str) = ("ribbon.tab.edit", "edit");

/// The ribbon control that arms the ADD-text tool.
///
/// ★★ **Add text, not Edit text**, and the distinction is what keeps two
/// features off one gesture. `edit.add_text` arms
/// `CanvasTool::TextEdit(TextEditKind::Add)`, whose drag draws a box; the
/// separate `view.tool_text` arms `CanvasTool::Text`, whose drag **sweeps** and
/// must go on sweeping, because `text_tool_selects_and_marks_in_edit` depends
/// on it to make a text selection the markup verbs can act on.
///
/// The box was briefly offered from the sweep tool's rung instead, and two unit
/// tests said no — correctly. Two features claiming one drag is a choice
/// somebody has to make, and taking a shipped gesture away to make room is the
/// wrong way to make it.
const TOOL: (&str, &str) = ("ribbon.item.edit.add_text", "edit.add_text");
/// `text-box-open page=… box=… w=… h=…`.
const OPEN_EVENT: &str = "text-box-open";
/// `text-box-declined w=… h=… floor=…`.
const DECLINED_EVENT: &str = "text-box-declined";
/// `add-text page=… n=… epoch=… disclosures=…` — `n` is the hard-newline count.
const APPLIED_EVENT: &str = "add-text";

/// See the module documentation.
pub struct TextBoxTakesAParagraph;

impl Check for TextBoxTakesAParagraph {
    fn name(&self) -> &'static str {
        "text_box_takes_a_paragraph"
    }

    fn defect(&self) -> &'static str {
        "text on the canvas is one line and cannot be made into more — either the text tool has \
         no drag, so there is no rectangle to wrap against, or plain Enter still COMMITS inside \
         one, which ends the draft at the first line break and silently discards everything \
         typed after it"
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

#[allow(clippy::too_many_lines)]
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
        .ok_or_else(|| Error::new("no --pdf. This check needs any document with a page."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space — the box is dragged out from \
             there. Unlike the selection checks this one does not need content under the point, \
             only room to the right and below it.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, arms a tool, \
             drags a rectangle and types. Reported as SKIPPED rather than passed.",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("text-box.trace.txt"));
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

    // --- 1: Edit, then arm the text tool ------------------------------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);
    click_tab(&session, &driver, ui_rect, EDIT_TAB)?;
    click_command(&session, &driver, ui_rect, TOOL, 16)?;
    let trace = session.trace()?;
    if !trace
        .events("text-edit-tool")
        .any(|l| l.get("tool") == Some("TextEdit(Add)"))
    {
        return Ok(Some(format!(
            "ADD TEXT WAS INVOKED AND THE TOOL DID NOT ARM: no `text-edit-tool \
             tool=TextEdit(Add)`. Either the dispatch arm declined — a `command-declined \
             id=edit.add_text reason=mode-cannot-edit-content` line would say so, and this \
             check is in `{MODE}` — or the arming was not reached. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("the Add-text tool is armed");

    // --- 2: drag a box ------------------------------------------------------
    //
    // ★ Built from DOCUMENT points through the frame's own mapping, not from
    // screen pixels: the box has to be big enough to hold two lines at the
    // pen's default 12 pt, and "big enough" is a fact about the page rather
    // than about the window. 200 × 60 pt is four default lines wide and three
    // tall, comfortably above the 12 pt floor `begin_box` refuses under.
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let frame = session.frame()?;
    let from =
        frame.to_screen(mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?);
    let to = frame.to_screen(mapping.doc_to_window(DocPoint::new(
        target.page,
        target.x + 200.0,
        // DOWN the page, which is a SMALLER y: PDF user space runs upward.
        // Getting this backwards draws the box off the top of the sheet, where
        // it is clipped and the drag looks like it did nothing.
        target.y - 60.0,
    ))?);
    driver.drag(from, to)?;
    session.settle(20);

    let trace = session.trace()?;
    let Some(opened) = trace.events(OPEN_EVENT).last() else {
        if let Some(declined) = trace.events(DECLINED_EVENT).last() {
            return Err(Error::new(format!(
                "the drag produced a box too small to type into: `{}`. That is a fact about \
                 where --doc-point sits and how much room is left on the page, not about the \
                 build — aim somewhere with 200 x 60 pt of space below and to the right. \
                 SKIPPED for that reason.",
                declined.raw
            )));
        }
        return Ok(Some(format!(
            "★ A DRAG WITH THE TEXT TOOL OPENED NO BOX: no `{OPEN_EVENT}` line.\n\
             The text tool must answer a drag as well as a click — a click places a caret for \
             one line, a drag draws a rectangle for a paragraph. If `press_kind`'s caret rung \
             still returns `drag: None`, the drag fell through to a marquee or a text sweep and \
             the operator gets a selection where they asked for a text box. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the drag opened a box: `{}`", opened.raw));

    // --- 3: ★★ type two lines, with a plain Enter between them --------------
    //
    // The link that would ship. If Enter still commits, the draft ends here and
    // the second line's keystrokes land nowhere.
    for key in [vk::A, vk::D] {
        driver.press(key)?;
        session.settle(4);
    }
    driver.press(vk::ENTER)?;
    session.settle(6);
    for key in [vk::A, vk::D] {
        driver.press(key)?;
        session.settle(4);
    }
    session.settle(10);

    // --- 4: Ctrl+Enter commits ---------------------------------------------
    //
    // ★ The escape hatch the plain Enter gave up. Every in-place editor that
    // takes multi-line input needs one, and this is the old shell's own choice
    // carried across: *"in box mode a plain Enter is a paragraph break;
    // Ctrl+Enter accepts."*
    driver.press_chord(&[vk::CONTROL], vk::ENTER)?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(applied) = trace.events(APPLIED_EVENT).last() else {
        return Ok(Some(format!(
            "the box took keystrokes and CTRL+ENTER COMMITTED NOTHING: no `{APPLIED_EVENT}` \
             line. Either the chord is not wired as the box's accept, or `commit_into`'s \
             `Anchor::Box` arm is missing and the draft was dropped. Trace: {}.",
            session.trace_path().display()
        )));
    };

    // --- 5: ★★★ and it authored TWO lines, not one --------------------------
    let lines: usize = applied
        .get("n")
        .and_then(|v| v.parse().ok())
        .unwrap_or_default();
    report.note(format!(
        "★ the commit reached the engine: `{}`",
        applied.raw
    ));
    if lines < 2 {
        return Ok(Some(format!(
            "★ THE PARAGRAPH WAS AUTHORED AS {lines} LINE(S), AND IT MUST BE 2.\n\
             A plain Enter inside a text box is a PARAGRAPH BREAK, not a commit. `n=1` means \
             the Enter committed the draft instead — so the first line was written, the second \
             line's keystrokes went nowhere, and the operator is left with an ordinary \
             single-line run as evidence that something happened.\n\
             Look at `canvas::textedit::typing`'s Enter arm: it must insert when the anchor is \
             `Anchor::Box` and the command modifier is absent, and commit otherwise. `n=0` \
             would mean the text never reached the request at all. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ the paragraph survived as {lines} lines — a plain Enter inside the box broke the \
         line instead of ending the draft"
    ));
    Ok(None)
}
