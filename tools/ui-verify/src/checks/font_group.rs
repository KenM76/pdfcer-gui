//! `the_format_tab_offers_font_controls_for_swept_text` — **the ribbon route
//! to a restyle, and the sentence that tells an operator how to reach it.**
//!
//! # What this is for, and how it differs from `restyle_text`
//!
//!
//! O37 shipped with an admission written into its own row:
//!
//! > You are in Edit mode, so a drag with the Select tool draws a marquee round
//! > objects. Press **T** first — that arms the text tool — then sweep across
//! > the words. ★ **That is a discoverability gap and it is ours, not a
//! > limitation. Nothing on screen tells you to press T.**
//!
//! Three surfaces now do. This check asserts two of them in the state an
//! operator is actually in when they need them, which is the state **before**
//! anything is swept — and that ordering is the whole design of the check.
//!
//! ## ★★★ The two phases, and why the first one has to come first
//!
//! | phase | the operator's state | what must be true |
//! |---|---|---|
//! | 1 | clicked a piece of text with the Select tool; **nothing swept** | the Format tab appears, it carries a **Font** group, and the Properties panel says how to get an operand |
//! | 2 | pressed `T`, swept the words | the ribbon's **Bold** commits a restyle to the document |
//!
//! Phase 1 cannot be reached after phase 2, because a sweep is not undone by
//! clicking again — so a check that swept first would have destroyed the state
//! it is meant to observe. That is not a harness convenience; it is the
//! operator's own sequence. They click the thing they want to change *before*
//! they know a sweep is needed, which is precisely why the gap existed.
//!
//! ## ★★ What phase 1 can and cannot see, said plainly
//!
//! The ribbon publishes `ribbon.item.<id>` for **every** command control,
//! enabled or greyed — deliberately, and `egui_shell::ribbon::control`'s own
//! note says why: *"the question a consumer asks is where is this control, and
//! a control that is greyed is still a control that was drawn somewhere."*
//!
//! So a region tells this check the control is **on screen**. It does not tell
//! it the control is greyed. That is a real limit and it is not papered over:
//! the greying is asserted by
//! `app::conditions::tests::the_font_groups_visibility_follows_the_mode_and_its_enablement_the_sweep`,
//! which reads the registered command's own predicate against the published
//! conditions — the join, not either half. What *this* check adds, and what no
//! unit test can, is that the controls are **drawn at all**, on a real ribbon,
//! in a window, at a real width, on the tab that really appeared.
//!
//! ★ And the appearing is itself under test. The Format tab is contextual: its
//! `visible_when` moved from `selection.any` to `selection.formattable` when
//! the Font group landed, and a build that missed that change shows **no tab**
//! after a sweep and therefore no Font group — a whole feature with no surface,
//! which is exactly the shape of defect this project exists to catch.
//!
//!
//!
//!
//! ★ The answer was in the trace the check was already holding:
//! `pdfcer-diag properties-panel object=832 kind=Path notes=0`. So
//! [`aimed_at_one_text_object`] now reads that line, plus
//! `canvas-selection … sel=N`, and **skips** — never fails — when the click did
//! not leave exactly one text object selected. Phase 2 had this guard from the
//! start (`chars == 0` is a skip, not a failure); phase 1 did not, and the
//! asymmetry is what let a correct program be blamed.
//!
//! ★★ The correct aim for this fixture is `--doc-point 0,1140,62`, which
//! `RESUME.md`'s aim table gives and the sweep did not use: a 5 pt title-block
//! run at PDF (1135.7, 58.4)–(1190.5, 63.4).
//!
//! # The oracle
//!
//! Phase 1: the precondition above, then the regions `ribbon.tab.format`,
//! `ribbon.group.format.font`, the five `ribbon.item.format.*`, and
//! `properties.text` with `properties.text.face` inside it.
//!
//!
//! Phase 2: `text-style-applied … applied=N` **and** the `format-text` label
//! `vector_edit` writes when the edit reached the engine — the same two-line
//! oracle `restyle_text` uses, for its reason: the first without the second is
//! a module that decided to act and whose action never landed.

use crate::checks::driving::{
    self, SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The mode whose canvas may select page content, and the only mode the Font
/// group is drawn in — `mode.edit_content` is its `visible_when`.
pub(super) const MODE: &str = "edit";
/// The contextual Format tab's strip region.
pub(super) const FORMAT_TAB: &str = "ribbon.tab.format";
/// The Font group's captioned band.
pub(super) const FONT_GROUP: &str = "ribbon.group.format.font";
/// The Bold control, which is the one this check presses.
const BOLD_ITEM: &str = "ribbon.item.format.bold";
/// Every control the Font group must draw, in manifest order.
///
/// ★ Asserted as a **list**, not as "Bold is there". Three of the five are
/// `Item::Custom`s drawn by `app::fontband`, and a custom item that the
/// manifest names and no renderer matches draws **nothing** while the shell
/// reserves its space — which is the defect `COLOUR_SWATCH` shipped with for
/// the whole of v0.1.0, invisible because a gap in a band looks like a gap in a
/// band. Only naming all five catches it.
pub(super) const FONT_ITEMS: [&str; 5] = [
    "ribbon.item.format.font",
    "ribbon.item.format.font_size",
    "ribbon.item.format.bold",
    "ribbon.item.format.italic",
    "ribbon.item.format.font_colour",
];
/// Every Font-group control's COMMAND id, in the same order as [`FONT_ITEMS`].
///
///
/// This module PINS its fixture: it opens `fixtures/paragraph.pdf` at a
/// measured point and ignores `--pdf` and `--doc-point`, because its subject
/// is a discoverability route and a route needs a known page.
/// `font_group_real` is the twin that does the opposite -- it honours the aim
/// and drives whatever drawing the operator names -- and it shares these two
/// lists, the aim guard and the enablement renderer rather than copying them.
///
/// ★ Copying would have been the ordinary move and it is the one this
/// repository has already paid for nine times over in private `click_tab`
/// helpers: a shared list diverges silently, and a group measured against a
/// stale copy of its own membership reports a measured group.
///
/// # ★★★ A second list, because a region and an enablement are different facts
///
/// [`FONT_ITEMS`] holds published REGION names (`ribbon.item.format.bold`) and
/// answers *where is this control*. These are the ids the same five controls
/// are registered under, and they are what the enablement event is keyed by,
/// because that event is about a COMMAND rather than about a rectangle.
///
/// ★★ The two lists are asserted to line up by
/// [`the_two_font_lists_describe_the_same_five_controls`], which exists because
/// a check that read four regions and five enablements, or five regions and
/// four enablements, would report a measured group either way. The pairing is
/// `ribbon.item.` + the id, and it is spelled out rather than computed so that
/// a rename on either side is a compile-visible edit to a literal instead of a
/// silently-still-passing concatenation.
pub(super) const FONT_COMMANDS: [&str; 5] = [
    "format.font",
    "format.font_size",
    "format.bold",
    "format.italic",
    "format.font_colour",
];

/// The Properties panel's **font editor**, drawn for a clicked text object
/// since O198 (2026-09-14).
///
/// ★★★ THIS CONSTANT REPLACED `ROUTE_REGION`, AND THE SWAP IS THE WHOLE POINT.
///
pub(super) const TEXT_STYLE_REGION: &str = "properties.text";
/// The face control inside the font editor.
///
/// ★★ Asserted ALONGSIDE the section, not instead of it, for the reason
/// `FONT_ITEMS` gives about the ribbon band: a section that draws its heading
/// and then returns before any control is the exact shape of the regression
/// this check exists to catch, and a section-level region cannot see it.
pub(super) const FACE_ROW_REGION: &str = "properties.text.face";
/// The `text-style-applied` summary line.
const STYLE_EVENT: &str = "text-style-applied";
/// The `text-style-declined` line.
const DECLINED_EVENT: &str = "text-style-declined";
/// The label `vector_edit` writes when the restyle reached the engine.
const APPLIED: &str = "format-text";
/// The sweep's own oracle.
const SELECTION_EVENT: &str = "canvas-text-selection";
/// The canvas's report of a selection change — `sel=` is how many entries it
/// holds, which is `panels::properties::text::route`'s single-selection rule
/// expressed as a number the harness can read.
const CANVAS_SELECTION_EVENT: &str = "canvas-selection";
/// The Properties panel's report of the object it is describing —
/// `properties-panel object=N kind=K notes=M`.
///
const PANEL_EVENT: &str = "properties-panel";
/// The `kind=` value [`PANEL_EVENT`] carries for a page-content text object —
/// `summary::ObjectKind::Text` under `{:?}`.
const TEXT_KIND: &str = "Text";
/// How far to sweep along the baseline, in PDF points.
const SWEEP_PT: f64 = 60.0;
/// `T` as a Windows virtual key — the text-sweep tool.
///
/// ★ Pressed only in **phase 2**, and the fact that phase 1 works without it is
/// the point of the check: the whole complaint is that an operator does not
/// know to press it, so the surfaces that tell them must be observed in the
/// state where they have not.
const VK_T: u16 = 0x54;

/// See the module documentation.
pub struct TheFormatTabOffersFontControlsForSweptText;

impl Check for TheFormatTabOffersFontControlsForSweptText {
    fn name(&self) -> &'static str {
        "the_format_tab_offers_font_controls_for_swept_text"
    }

    fn defect(&self) -> &'static str {
        "an operator who wants to change how text looks has to already know to press T and \
         sweep — the ribbon offers no font controls, or offers them on a tab that never appears \
         for a text selection, so a capability the engine has and the panel exposes is \
         unreachable from the surface an operator looks at first"
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

/// Poll until the restyle reports one way or the other, and answer how long it
/// took.
///
/// A bounded poll rather than a fixed sleep, for `restyle_text`'s reason: a
/// restyle re-resolves its pin from a fresh provenance extraction per run, so a
/// sweep across a title-block label is a dozen extractions, and a fixed sleep
/// long enough for the worst case makes every run slow while a pleasant one
/// reads the trace mid-gesture and reports "nothing happened" about a gesture
/// that is still running.
/// **Did the phase-1 click land on one text object?** `Ok(())` if it did; an
/// [`Error`] — which this check's `run` turns into a SKIP — if it did not.
///
/// # ★★★ Why this exists, written on the day it was needed
///
///
/// ★ The trace had the answer on the same frame the check was already reading:
/// `pdfcer-diag properties-panel object=832 kind=Path notes=0`. Nothing new had
/// to be published for this guard; the check simply had to look.
///
/// # ★★ The two facts, and why both are needed
///
/// `route` draws when **exactly one** object is selected **and** it is text.
/// Those are separate refusals with separate causes, so they are read
/// separately:
///
/// | fact | read from | why not the other line |
/// |---|---|---|
/// | how many are selected | `canvas-selection … sel=N` | the panel describes only the FIRST, so it cannot count |
/// | what the first one is | `properties-panel … kind=K` | the canvas line names a `TargetId`, not a kind |
///
/// # ★ An absent `properties-panel` line is "selected nothing", not "unknown"
///
/// `object_section` writes that line unconditionally once it has an object to
/// describe, every frame, through `diag::trace` rather than `trace_changed`. So
/// its absence after a settled click means `object_indices_on` came back empty
/// — no page-content object under the pointer — which is an aim problem of its
/// own and is reported as one.
pub(super) fn aimed_at_one_text_object(
    session: &Session,
    trace: &Trace,
    target: DocPoint,
) -> Result<()> {
    let aim = format!(
        "the --doc-point (page {}, {:.1}, {:.1})",
        target.page, target.x, target.y
    );
    let path = session.trace_path().display();
    match aim_verdict(trace) {
        Aim::OneTextObject => Ok(()),
        Aim::NothingSelected => Err(Error::new(format!(
            "{aim} selected no page-content object, so there is nothing for this check's \
             subject — a piece of text selected as an OBJECT — to be about. No `{PANEL_EVENT}` \
             line, which `panels::properties::mod::object_section` writes every frame it has an \
             object to describe. SKIPPED rather than failed: this says where the harness aimed, \
             not what the program did. Aim at a run of text — `pdfcer extract-text --json` \
             gives the first glyph's x and y of every run. Trace: {path}."
        ))),
        Aim::NotText(kind) => Err(Error::new(format!(
            "{aim} selected a `{kind}`, not text — `{PANEL_EVENT} … kind={kind}`. The Format \
             tab is right to appear (`selection.formattable` is the union of ANY object \
             selection with a live text selection) and `panels::properties::text::route` is \
             right to stay silent, so every oracle below this point would be asserting a \
             sentence about text over a selection that is not text. SKIPPED rather than failed: \
             this is the harness's aim and not the program's behaviour. ★ This is the exact \
             shape of the 2026-08-28 sweep's false alarm — one `--doc-point` was used for the \
             whole suite and it was on a drawing view. Trace: {path}."
        ))),
        Aim::NotAlone(n) => Err(Error::new(format!(
            "{aim} left {n} object(s) selected and `panels::properties::text::route` is \
             deliberately single-selection only — a sentence about *these words* over a mixed \
             selection would describe something the operator did not do. SKIPPED rather than \
             failed: aim at a text run that no other object overlaps. Trace: {path}."
        ))),
    }
}

/// What the phase-1 click left selected, as the four answers that matter.
///
/// Separated from the wording above so the READ is testable without a running
/// program: every variant here is reachable from a three-line trace, and the
/// tests at the foot of this module reach all four. That is the whole point of
/// the split — a guard against a harness misreading its own oracle is worth
/// nothing if the guard itself can only be exercised by driving the mouse.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Aim<'a> {
    /// Exactly one object is selected and it is text. Phase 1 may proceed.
    OneTextObject,
    /// No page-content object is selected — no `properties-panel` line at all.
    NothingSelected,
    /// One object is selected and it is not text; the `kind=` value is carried
    /// so the skip can name it.
    NotText(&'a str),
    /// More than one object is selected (or the count did not parse as one).
    NotAlone(usize),
}

/// Read [`Aim`] out of a settled trace.
///
/// ★ **Order matters, and it is "what" before "how many".** A click that lands
/// on a path inside a marquee of eleven is an aim problem twice over, and the
/// kind is the more useful half to be told about: it names the fixture
/// coordinate that has to change. Reporting "11 objects selected" first would
/// send a reader looking for a stray Shift.
fn aim_verdict(trace: &Trace) -> Aim<'_> {
    let Some(kind) = trace.last(PANEL_EVENT).and_then(|line| line.get("kind")) else {
        return Aim::NothingSelected;
    };
    if kind != TEXT_KIND {
        return Aim::NotText(kind);
    }
    // ★ Absent reads as zero rather than as one. `canvas-selection` is written
    // through `diag::trace_changed`, so a trace with no line at all is a run
    // where the selection never changed — which after a click is a click that
    // selected nothing, and is exactly the state that must not be waved
    // through. A defaulted-to-one guard would pass on silence.
    let selected = trace
        .last(CANVAS_SELECTION_EVENT)
        .and_then(|line| line.get_usize("sel"))
        .unwrap_or(0);
    if selected == 1 {
        Aim::OneTextObject
    } else {
        Aim::NotAlone(selected)
    }
}

fn wait_for_verdict(session: &Session) -> Result<u128> {
    const CEILING_MS: u128 = 20_000;
    let started = std::time::Instant::now();
    loop {
        session.settle(4);
        let trace = session.trace()?;
        if trace.last(STYLE_EVENT).is_some() || trace.last(DECLINED_EVENT).is_some() {
            return Ok(started.elapsed().as_millis());
        }
        if started.elapsed().as_millis() > CEILING_MS {
            return Ok(started.elapsed().as_millis());
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
    // ★ PINNED: `--pdf` and `--doc-point` are read and IGNORED here.
    //
    //
    // `fixture::text_point_target` holds the document, the point, and the
    // measurement behind both. Read its doc comment before changing either.
    let (pdf, target) = crate::fixture::text_point_target();
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the text fixture is not at {}. It is committed to this repository, so an \
             absence is a broken checkout rather than an unavailable precondition, and is \
             reported as a failure for that reason - a SKIP would say the opposite.",
            pdf.display()
        )));
    }
    report.note(format!(
        "--pdf and --doc-point are IGNORED: this check pins {} at page 0, 120, 704 - \
         eight characters into a 12 pt line on a 612 × 792 page",
        pdf.display()
    ));
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a page object, clicks a ribbon \
             tab, sweeps the pointer and presses a button, and none of that can be simulated \
             from the trace.",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("font-group.trace.txt"));
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
    session.settle(20);

    // ★★★ BRING THE PROPERTIES PANE TO THE FRONT OF ITS TAB GROUP.
    //
    // The dock mounts several panes per slot and draws only the ACTIVE tab's
    // body, so a pane that exists and is not in front publishes nothing —
    // which is indistinguishable, from here, from a panel that had nothing to
    // say about the selection.
    //
    //
    // ⇒ An absent `properties-panel` line means "selected nothing" ONLY once
    // the pane is in front. Establishing that is this check's job, not the
    // reader's.
    if let Some(tab) = declared(&session.trace()?, ui_rect, "dock.tab.file.properties") {
        driver.click_at(session.frame()?.declared_center(tab))?;
    }
    session.settle(20);

    // =======================================================================
    // PHASE 1 — click the text as an OBJECT, and nothing else.
    //
    // This is the state O37 admitted to: the operator has clicked the thing
    // they want to change, and the program has to tell them what to do next.
    // =======================================================================
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let frame = session.frame()?;
    // ★ A little way ALONG the baseline and a little above it, not at the
    // baseline's left end. `--doc-point` names the first glyph's origin, which
    // is the bottom-left corner of the first character — a point on the very
    // edge of the ink and, on a six-point label, a click that can land in the
    // paper beside it. Two points in and two points up is inside the glyph box
    // of any text this fixture carries.
    let on_text = frame.to_screen(mapping.doc_to_window(DocPoint::new(
        target.page,
        target.x + 2.0,
        target.y + 2.0,
    ))?);
    driver.click_at(on_text)?;
    session.settle(24);

    let trace = session.trace()?;
    // ★★★ THE PRECONDITION, BEFORE ANY ORACLE — what did the click actually
    // select?
    //
    //
    // ★ Note where the guard has to sit: **before** the Format tab is asserted,
    // not after. The tab's `visible_when` is `selection.formattable`, which is
    // true of *any* selection, so a click on a path raises the tab and every
    // later step then runs on a wrong premise. Putting the precondition first
    // also buys the tab's own failure message a sharper claim — see below.
    //
    // ★★ SKIPPED, never failed. This is `restyle_text`'s rule and phase 2's,
    // one paragraph down: a `--doc-point` that is not on text is the harness's
    // aim, and a harness that reports its own aim as the program's behaviour is
    // worse than one that reports nothing.
    aimed_at_one_text_object(&session, &trace, target)?;
    report.note("the click selected exactly one object and it is a text object");

    let Some(_tab) = declared(&trace, ui_rect, FORMAT_TAB) else {
        let shot = ctx.out("font_group.no-format-tab.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ CLICKING A PIECE OF TEXT RAISED NO FORMAT TAB: no `{FORMAT_TAB}` region.\n\
             One candidate, and the aim is not it: the precondition above read the trace and \
             found exactly one selected object of kind `Text`, so the click landed on ink and \
             the tab has a subject. That leaves **`selection.formattable` is not published**, \
             which is the condition the tab's `visible_when` names; it is the union of the \
             object selection and a live text selection, and a build that spelled it either way \
             round loses one of the tab's two subjects. Tabs declared: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab.")),
            session.trace_path().display()
        )));
    };
    report.note("★ clicking a piece of text raised the contextual Format tab");

    // ★★ The Properties panel's font editor, read BEFORE the ribbon tab is
    // clicked, because clicking a tab does not disturb the panel and reading it
    // first keeps the two surfaces independent.
    //
    // ★★★ THIS IS THE PANEL HALF OF O198, AND IT IS AN ASSERTION ABOUT A CLICK.
    //
    let missing = [TEXT_STYLE_REGION, FACE_ROW_REGION]
        .into_iter()
        .find(|region| driving::declared(&trace, ui_rect, region).is_none());
    if let Some(region) = missing {
        let shot = ctx.out("font_group.no-text-style-section.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★★ A PIECE OF TEXT IS SELECTED AND THE PROPERTIES PANEL DRAWS NO FONT EDITOR: no \
             `{region}` region.\n\
             `panels::properties::text::section` opens by asking \
             `app::textoperand::Cache::resolve` which runs a restyle would act on, and returns \
             without drawing when the answer is nothing. ★★ The two candidates this message \
             would otherwise lead with are ALREADY RULED OUT by the precondition above, which \
             read the trace and found `properties-panel ... kind=Text` over a selection of \
             exactly one: the object IS text by `summary::object_kind`, and it is not a \
             multi-selection. What is left. (1) **The object's byte span holds no placeable \
             show operator**, so `canvas::textedit::pin::object_text` found no runs - the one \
             direction in which the condition `selection.text_runs` and the operand are \
             allowed to disagree, and it is documented as harmless because the verbs decline. \
             Look for a `text-operand-resolved ... runs=0` line in the trace; if it is there, \
             this is a fixture problem and not a defect. (2) **The resolver was never called**, \
             which shows as no `text-operand-resolved` line at all. (3) **The region was drawn \
             and not declared**: `diag::ui_rect_visible` withholds a rect whose section is less \
             than 60 % inside its clip, which is what a Properties pane taller than its dock \
             slot produces - the screenshot beside this report settles that one by eye. \
             Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(
        "★★★ the Properties panel drew the font editor for a CLICKED text object, with nothing swept",
    );

    // The Format tab is contextual and is not the active tab merely by
    // appearing — the band draws whichever tab is active, so its contents are
    // unobservable until it is clicked. That is correct behaviour and not a
    // defect: a tab that stole focus on every selection would move the ribbon
    // under the operator's hand.
    let tab_rect = declared(&trace, ui_rect, FORMAT_TAB).expect("checked above");
    driver.click_at(session.frame()?.declared_center(tab_rect))?;
    session.settle(20);

    let trace = session.trace()?;
    if declared(&trace, ui_rect, FONT_GROUP).is_none() {
        let shot = ctx.out("font_group.no-group.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ THE FORMAT TAB CARRIES NO FONT GROUP: no `{FONT_GROUP}` region.\n\
             The most likely cause is `mode.edit_content`, which every one of the group's five \
             items names as its `visible_when` — a group all of whose items are hidden is not \
             drawn at all, by design, so an unpublished condition removes the whole band \
             silently. Groups declared: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.group.")),
            session.trace_path().display()
        )));
    }
    let missing: Vec<&str> = FONT_ITEMS
        .into_iter()
        .filter(|name| {
            driving::declared_or_in_overflow(&session, &driver, ui_rect, name)
                .ok()
                .flatten()
                .is_none()
        })
        .collect();
    if !missing.is_empty() {
        let shot = ctx.out("font_group.missing-items.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ THE FONT GROUP IS DRAWN AND {} OF ITS FIVE CONTROLS ARE MISSING: {}.\n\
             Three of the five — the face chooser, the size field and the colour swatch — are \
             `Item::Custom`s drawn by `app::fontband`, and a custom kind the manifest names and \
             no renderer matches draws NOTHING while the shell reserves its space. That is a \
             gap in a band, which looks like a gap in a band; `manifest::COLOUR_SWATCH`'s own \
             note records it shipping that way for the whole of v0.1.0. The other two are \
             ordinary command items and their absence would mean the manifest lost them. Items \
             declared: {}. Trace: {}.",
            missing.len(),
            list_of(&missing),
            list(&declared_names(&trace, ui_rect, "ribbon.item.format.")),
            session.trace_path().display()
        )));
    }
    let states = driving::enablement(&session)?;
    let unpressable: Vec<&str> = FONT_COMMANDS
        .into_iter()
        .filter(|id| !states.get(*id).is_some_and(|s| s.pressable()))
        .collect();
    if !unpressable.is_empty() {
        let shot = ctx.out("font_group.greyed.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★★★ THE FONT GROUP DREW ALL {} OF ITS CONTROLS AND {} OF THEM COULD NOT BE PRESSED: {}.\n\
             That is `OPERATOR_REQUESTS.md` O198 claim 3 reproducing — *\"the font selector and \
             editing tools ... that entire area is always greyed out in the menu\"* — on a frame \
             in which exactly one text object is selected and nothing is swept, which is the \
             state O198 made these controls act in.\n\
             What each one said: {}.\n\
             ★★ READ THE TWO NUMBERS BEFORE BLAMING EITHER SIDE. `enabled=0` is the command's \
             own predicate over the published conditions refusing, and points at \
             `app::conditions` and the `selection.text_runs` join. `enabled=1 live=0` is the \
             renderer refusing AFTER the condition agreed, and points at `app::fontband`'s \
             second predicate — the `resolved(doc, draft)` read-back — which is a text \
             extraction that can come back empty for a font the extractor cannot resolve while \
             every condition about the selection stays true. A missing entry altogether means \
             the control never published an enablement, which for the three custom ones means \
             `app::fontband::draw` returned before its report.\n\
             Trace: {}.",
            FONT_COMMANDS.len(),
            unpressable.len(),
            list_of(&unpressable),
            describe(&states),
            session.trace_path().display()
        )));
    }
    let disagreeing: Vec<&str> = FONT_COMMANDS
        .into_iter()
        .filter(|id| states.get(*id).is_some_and(|s| s.disagrees()))
        .collect();
    report.note(format!(
        "★★★ all {} Font controls were drawn PRESSABLE with nothing swept: {}",
        FONT_COMMANDS.len(),
        describe(&states)
    ));
    debug_assert!(
        disagreeing.is_empty(),
        "unreachable: a disagreeing control is not pressable"
    );

    // =======================================================================
    // PHASE 2 — arm the text tool, sweep, and press the ribbon's Bold.
    // =======================================================================
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let frame = session.frame()?;
    let start =
        frame.to_screen(mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?);
    let end = frame.to_screen(mapping.doc_to_window(DocPoint::new(
        target.page,
        target.x + SWEEP_PT,
        target.y,
    ))?);
    driver.press(VK_T)?;
    session.settle(16);
    driver.drag(start, end)?;
    session.settle(24);

    let trace = session.trace()?;
    let swept = trace
        .events(SELECTION_EVENT)
        .last()
        .and_then(|l| l.get("chars"))
        .and_then(|n| n.parse::<usize>().ok())
        .unwrap_or(0);
    if swept == 0 {
        return Err(Error::new(format!(
            "the drag from (page {}, {:.1}, {:.1}) rightwards {SWEEP_PT} pt selected no text, \
             so there was nothing for the Font group to act on. SKIPPED rather than failed: \
             this says the --doc-point is not on text, which is the harness's aim and not the \
             program's behaviour. Trace: {}.",
            target.page,
            target.x,
            target.y,
            session.trace_path().display()
        )));
    }
    report.note(format!("the sweep selected {swept} character(s)"));

    // ★ Re-find Bold rather than reusing the phase-1 rect. The sweep can change
    // the band: `selection.any` may have gone (a text press clears the object
    // selection on some routes), which changes nothing about the Font group but
    // could reflow the Selection group beside it — and a stale rect is how a
    // check clicks whatever happens to be at those coordinates now. `D:/dev/rag/egui/`
    // carries the general form of this: harness coordinates go stale when a
    // layout changes, and the symptom is a click that lands on the wrong thing
    // and a failure that blames the feature.
    let bold = driving::declared_or_in_overflow(&session, &driver, ui_rect, BOLD_ITEM)?
        .ok_or_else(|| {
            Error::new(format!(
                "no `{BOLD_ITEM}` region after the sweep, though it was there before it. \
                 SKIPPED rather than failed: a button that was never pressed proves nothing \
                 about pressing it. Trace: {}.",
                session.trace_path().display()
            ))
        })?;
    driver.click_at(session.frame()?.declared_center(bold))?;

    let waited = wait_for_verdict(&session)?;
    report.note(format!(
        "the restyle took {waited} ms of wall clock — one provenance extraction per run"
    ));

    let trace = session.trace()?;
    if let Some(declined) = trace.events(DECLINED_EVENT).last() {
        return Ok(Some(format!(
            "the ribbon's Bold was pressed and the restyle DECLINED: `{}`.\n\
             That is the program answering rather than staying silent, so the whole chain — \
             tab, group, custom renderer, token, dispatch arm, action — works, and the answer \
             is what is worth reading. A refusal means neither `set_synthetic` nor the \
             `set_font` retry took, which is either an unpinnable run or the engine naming a \
             real face that then failed glyph coverage (filed and confirmed, 2026-08-27). \
             Trace: {}.",
            declined.raw,
            session.trace_path().display()
        )));
    }
    let Some(applied) = trace.events(STYLE_EVENT).last() else {
        let shot = ctx.out("font_group.no-effect.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ THE RIBBON'S BOLD WAS PRESSED AND NOTHING HAPPENED AND NOTHING WAS DECLINED: no \
             `{STYLE_EVENT}` line and no `{DECLINED_EVENT}` line.\n\
             The panel's Bold and this one raise the same action through different routes, so \
             the candidates are the parts they do NOT share. (1) **The control was greyed** — \
             `selection.text` is its enable predicate and the sweep above set it, so this would \
             mean the condition and the sweep disagree. (2) **`dispatch::format` does not claim \
             the id**, in which case the token reached the dispatcher and fell through; \
             `handles` is the list. (3) **The click missed** — the region was declared, so the \
             screenshot beside this report settles it. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★ the ribbon's Bold committed a restyle: `{}`",
        applied.raw
    ));

    let n: usize = applied
        .get("applied")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if n == 0 {
        return Ok(Some(format!(
            "the restyle reported `applied=0`, so the arm decided to act and every run it \
             tried refused without saying so. That is worse than a decline: `{}`. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    if trace.last(APPLIED).is_none() {
        return Ok(Some(format!(
            "the ribbon's Bold computed `{}` and no `{APPLIED}` line followed, so the action \
             was raised and its apply arm never ran. Nothing reached the document, which from a \
             chair is indistinguishable from the button doing nothing. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note(
        "★★★ the whole ribbon route works: a click on text raised the tab, the tab carried the \
         Font group, and its Bold restyled the swept text in the open document",
    );
    Ok(None)
}

/// Join borrowed names for a failure message.
///
/// `driving::list` takes owned `String`s and `driving::list_str` takes a slice
/// of `&str` — this is the latter, spelled locally only because the filter
/// above produces a `Vec<&str>` and handing it straight over reads better than
/// a collect-into-owned at the call site.
fn list_of(names: &[&str]) -> String {
    driving::list_str(names)
}

/// Render the five controls' enablement for a failure message.
///
/// ★ One line, both numbers, every id — including the ones that PASSED. A
/// message that lists only the offenders leaves a reader unable to tell
/// *"three of five are dead"* from *"three of five never reported"*, and those
/// two want different investigations. `live=?` marks a control whose renderer
/// has no second predicate, which is not the same as one whose second predicate
/// said no.
pub(super) fn describe(states: &std::collections::BTreeMap<String, driving::Enablement>) -> String {
    FONT_COMMANDS
        .into_iter()
        .map(|id| match states.get(id) {
            None => format!("{id}=NEVER-REPORTED"),
            Some(s) => format!(
                "{id}: enabled={} live={}",
                u8::from(s.enabled),
                match s.live {
                    None => "?".to_owned(),
                    Some(live) => u8::from(live).to_string(),
                }
            ),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★ The region list and the command list describe the same five controls.
    ///
    /// Two hand-written lists over one set is the shape a completeness check
    /// goes blind in: a sixth control added to the band and to one list reads as
    /// a measured group from either end. The pairing is mechanical — a region
    /// is `ribbon.item.` followed by the command id — so it can be asserted
    /// even though neither list is derived from the other.
    #[test]
    fn the_two_font_lists_describe_the_same_five_controls() {
        let paired: Vec<String> = FONT_COMMANDS
            .into_iter()
            .map(|id| format!("ribbon.item.{id}"))
            .collect();
        assert_eq!(paired, FONT_ITEMS.to_vec(), "{paired:?}");
    }

    /// The trace of the state the check is FOR: one text object clicked with
    /// the Select tool, nothing swept.
    const ON_TEXT: &str = "pdfcer-diag canvas-selection via=click mod=false sel=1 level=Object \
                           first=object:412\n\
                           pdfcer-diag properties-panel object=412 kind=Text notes=0";

    /// ★★★ The regression this guard was written for, in three lines.
    ///
    /// These are the actual values from a run at `--doc-point 0,300,500` — a
    /// drawing view on `SW41177.pdf` — where the check reported the program's
    /// correct silence as O37's complaint returning.
    /// The verdict must be a SKIP that names the kind, so the reader is sent to
    /// the fixture coordinate rather than to `panels::properties::text`.
    #[test]
    fn a_click_that_landed_on_a_path_is_the_harnesss_aim_and_not_a_defect() {
        let trace = Trace::parse(
            "pdfcer-diag canvas-selection via=click mod=false sel=1 level=Object \
             first=object:832\n\
             pdfcer-diag properties-panel object=832 kind=Path notes=0",
            "pdfcer-diag",
        );
        assert_eq!(aim_verdict(&trace), Aim::NotText("Path"));
    }

    /// The state every phase-1 oracle below the guard is entitled to assume.
    #[test]
    fn one_text_object_is_the_state_the_check_may_proceed_from() {
        let trace = Trace::parse(ON_TEXT, "pdfcer-diag");
        assert_eq!(aim_verdict(&trace), Aim::OneTextObject);
    }

    /// A click on blank paper: `object_section` has no object to describe, so
    /// it writes no line at all. Absent must not read as "unknown, carry on".
    #[test]
    fn no_properties_panel_line_means_the_click_selected_nothing() {
        let trace = Trace::parse(
            "pdfcer-diag canvas-selection via=click mod=false sel=0 level=Object first=none",
            "pdfcer-diag",
        );
        assert_eq!(aim_verdict(&trace), Aim::NothingSelected);
        // ★ And an EMPTY trace too — a run where the click never reached the
        // canvas is the same absence and must skip rather than proceed.
        assert_eq!(
            aim_verdict(&Trace::parse("", "pdfcer-diag")),
            Aim::NothingSelected
        );
    }

    /// `route` is single-selection by design, so a marquee that caught a label
    /// and ten paths is an aim problem even though the FIRST object is text.
    ///
    /// ★ This is the case `properties-panel` alone cannot see: it describes the
    /// first selected object and says nothing about how many there are, which
    /// is why the count is read from `canvas-selection` instead of inferred.
    #[test]
    fn a_multi_selection_whose_first_object_is_text_still_skips() {
        let trace = Trace::parse(
            "pdfcer-diag canvas-selection via=marquee mod=false sel=11 level=Object \
             first=object:412\n\
             pdfcer-diag properties-panel object=412 kind=Text notes=0",
            "pdfcer-diag",
        );
        assert_eq!(aim_verdict(&trace), Aim::NotAlone(11));
    }

    /// A `properties-panel` line with no readable count beside it must not be
    /// waved through as one.
    ///
    /// `canvas-selection` is written through `diag::trace_changed`, so a run
    /// that never changed its selection carries no line — and defaulting that
    /// to one would let the guard pass on silence, which is the failure mode
    /// the guard exists to end.
    #[test]
    fn a_missing_selection_count_is_zero_and_not_one() {
        let trace = Trace::parse(
            "pdfcer-diag properties-panel object=412 kind=Text notes=0",
            "pdfcer-diag",
        );
        assert_eq!(aim_verdict(&trace), Aim::NotAlone(0));
    }

    /// The event and field names are the program's, and a rename on either side
    /// silently turns this guard into "always skip".
    ///
    /// Pinned here rather than trusted: the two lines are quoted verbatim from
    /// `canvas::trace` and `panels::properties::mod::object_section`, and the
    /// kind spelling is `summary::ObjectKind::Text` under `{:?}`.
    #[test]
    fn the_oracle_names_are_the_ones_the_program_writes() {
        assert_eq!(CANVAS_SELECTION_EVENT, "canvas-selection");
        assert_eq!(PANEL_EVENT, "properties-panel");
        assert_eq!(TEXT_KIND, "Text");
        let trace = Trace::parse(ON_TEXT, "pdfcer-diag");
        assert!(
            trace.last(PANEL_EVENT).is_some(),
            "the panel line must parse under the profile's `pdfcer-diag` prefix"
        );
        assert_eq!(
            trace
                .last(CANVAS_SELECTION_EVENT)
                .and_then(|line| line.get_usize("sel")),
            Some(1)
        );
    }
}
