//! `text_annot_places_and_authors` — a text box drawn on the page, typed into,
//! and actually written to the document.
//!
//! # The gap this closes
//!
//! The operator asked on 2026-08-18 to *"finish adding all of the standard
//! revisioning tools."* Three were registered, drawn on the Markup tab, and
//! had no dispatch arm for the whole life of the project: text box, sticky note
//! and stamp.
//!
//! Their recorded reason was accurate — `canvas::markup`'s own table calls them
//! *"text-bearing, not geometric. A different gesture (place, then type) and a
//! different spec type"* — and building them meant a fourth `CanvasTool`
//! family, a `DragKind` whose release does **not** author, a dialog, two
//! actions and a click path.
//!
//! # Why the release-does-not-author part needs driving
//!
//! It is the whole difference between this family and the seven geometric
//! kinds, and **no unit test in the workspace can see it**. The chain is:
//!
//! 1. a ribbon press arms `CanvasTool::TextAnnot(kind)`;
//! 2. a drag on the canvas rubber-bands a rectangle;
//! 3. the release raises `BeginTextAnnot` and **authors nothing**;
//! 4. a dialog opens and the operator types;
//! 5. Accept raises `CommitTextAnnot`, which reaches the engine.
//!
//! Step 3 is the one worth a harness. A build where the release authored
//! directly would pass every unit test in `canvas::textannot` — the spec
//! builder is pure and correct either way — and would put an **empty box** on
//! the operator's drawing every time they let go of the mouse.
//!
//! So this check asserts, in order: the tool arms, the drag places, **the page
//! is unchanged at that moment**, the dialog appears, and only after typing and
//! accepting does the annotation count go up.
//!
//! # Why the text box and not all three
//!
//! It is the one that exercises every link. The sticky differs only in its
//! placing gesture (a click rather than a drag) and the stamp only in where its
//! words come from (a gallery rather than a field) — both are covered by unit
//! tests over predicates the production code itself branches on
//! (`is_dragged`, `uses_gallery`), which is the shape that cannot drift. What
//! neither of those can cover is the frame-level chain, and that is identical
//! for all three.
//!
//! Stated rather than left implied, because "we drove one of three" read as
//! "we drove them" is exactly the kind of coverage claim this suite exists to
//! stop.

use crate::checks::driving::{
    SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, frame_of, list, shell_trace,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// How wide and tall the placed box is, in PDF points.
///
/// Big enough that the drag is unambiguously a drag rather than a click the
/// gesture machine might round to one, and small enough to stay on the page
/// from any `--doc-point` that is itself on it.
const BOX_PT: f64 = 220.0;

/// See the module documentation.
pub struct TextAnnotPlacesAndAuthors;

impl Check for TextAnnotPlacesAndAuthors {
    fn name(&self) -> &'static str {
        "text_annot_places_and_authors"
    }

    fn defect(&self) -> &'static str {
        "the text box control does not arm, the drag authors an EMPTY annotation on release \
         instead of asking for words, the dialog never appears, or accepting it writes \
         nothing to the document"
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
            "input is disabled (--no-input). This check clicks a ribbon control, drags on the \
             canvas and types into a dialog. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. This check needs somewhere on the page to draw the box, and a \
             guessed one can land off the sheet — which is symptom-identical to a drag that \
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("text_annot.trace.txt"));
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

    // --- 1: Review mode and the Markup tab ---------------------------------
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
            "the click on the Markup tab produced no tab-selected line, so nothing below \
             would mean anything.",
        ));
    }

    // --- 2: arm the text box -----------------------------------------------
    let trace = session.trace()?;
    let item = declared(&trace, ui_rect, "ribbon.item.markup.text_box").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.item.markup.text_box` region on the Markup tab. Items declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.item.markup."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(14);
    let trace = session.trace()?;
    if !trace
        .events("markup-tool")
        .any(|l| l.get("tool").is_some_and(|t| t.contains("TextAnnot")))
    {
        return Ok(Some(
            "clicking Markup > Text box traced no `markup-tool tool=TextAnnot(..)` line, so \
             the control armed nothing. Until 2026-08-18 this command had no dispatch arm at \
             all and traced `command-unimplemented`; that is the state this check exists to \
             keep it out of."
                .to_owned(),
        ));
    }
    report.note("Markup > Text box armed the text-annotation tool");

    // --- 3: how many annotations does the page have BEFORE? ----------------
    //
    // The oracle for step 5, taken now so the comparison is against the page
    // as the operator found it.
    let before = annot_count(&session);
    report.note(format!(
        "the page carries {before} annotation(s) before the drag"
    ));

    // --- 4: drag the box, and assert the release AUTHORS NOTHING -----------
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, target.page)?;
    let frame = session.frame()?;
    let from = frame.to_screen(mapping.doc_to_window(target)?);
    let to = frame.to_screen(mapping.doc_to_window(DocPoint {
        page: target.page,
        x: target.x + BOX_PT,
        y: target.y + BOX_PT,
    })?);
    report.note(format!(
        "dragging a {BOX_PT:.0}x{BOX_PT:.0} pt box: screen ({}, {}) -> ({}, {})",
        from.x(),
        from.y(),
        to.x(),
        to.y()
    ));
    driver.drag(from, to)?;
    session.settle(20);

    let trace = session.trace()?;
    if trace.events("text-annot-open").next().is_none() {
        return Ok(Some(
            "the drag completed and no `text-annot-open` line was traced, so the release did \
             not open the dialog. Either the drag never reached `GestureOutcome::TextAnnot` \
             or the action did not reach the dialog host."
                .to_owned(),
        ));
    }
    if declared(&trace, ui_rect, "dialog:text-annot").is_none() {
        return Ok(Some(
            "a `text-annot-open` line was traced and no `dialog:text-annot` region appeared, \
             so the dialog was created and never drawn."
                .to_owned(),
        ));
    }
    // ★★ THE ASSERTION THIS CHECK EXISTS FOR.
    let after_drag = annot_count(&session);
    if after_drag > before {
        return Ok(Some(format!(
            "the RELEASE authored an annotation: the page went from {before} to \
             {after_drag} before the operator typed anything. A text box is not a markup \
             band — releasing the mouse must ask for words, not commit an EMPTY box onto \
             the drawing. Every unit test in `canvas::textannot` passes on a build that \
             does this, because the spec builder is pure and correct either way."
        )));
    }
    report.note(format!(
        "the release authored nothing — still {after_drag} annotation(s) — and opened the \
         dialog instead, which is the property that separates this family from the seven \
         geometric kinds"
    ));

    // --- 5: type, accept, and assert the page gained one -------------------
    let Some(field) = declared(&trace, ui_rect, "text-annot.text") else {
        return Ok(Some(
            "the dialog is open and declared no `text-annot.text` region, so there is \
             nowhere to type the words it is asking for."
                .to_owned(),
        ));
    };
    driver
        .click_at(frame_of(&session, &trace, ui_rect, "text-annot.text")?.declared_center(field))?;
    session.settle(8);
    // Two keys that already exist in `sys::vk`. WHAT is typed does not matter
    // — the Accept control is gated on the field being non-empty and nothing
    // else — so this deliberately does not add letter constants the rest of
    // the harness has no use for.
    for key in [vk::F, vk::DIGIT_2] {
        driver.press(key)?;
        session.settle(6);
    }

    let trace = session.trace()?;
    let Some(accept) = declared(&trace, ui_rect, "text-annot.accept") else {
        return Ok(Some(
            "the dialog declared no `text-annot.accept` region.".to_owned(),
        ));
    };
    driver.click_at(
        frame_of(&session, &trace, ui_rect, "text-annot.accept")?.declared_center(accept),
    )?;
    session.settle(24);

    let after_accept = annot_count(&session);
    if after_accept <= after_drag {
        return Ok(Some(format!(
            "Accept was pressed and the page still carries {after_accept} annotation(s). The \
             tool armed, the drag placed, the dialog opened and typed — and nothing reached \
             the document. The break is between `CommitTextAnnot` and \
             `EditSession::add_text_annotation`."
        )));
    }
    report.note(format!(
        "Accept authored: the page went from {after_drag} to {after_accept} annotation(s)"
    ));
    Ok(None)
}

/// How many text annotations this session has authored so far.
///
/// Counted from the application's own `add-text-annot` trace lines — the label
/// `vector_edit` stamps on the commit — rather than from the file, because the
/// annotation has not been saved and only the session knows about it. That is
/// the same reason `panels::redact`'s census reads the session graph.
///
/// ★ It counts COMMITS, which is exactly the question this check asks: "did
/// the release author?" and "did Accept author?" are both about whether the
/// funnel ran, not about what the page contains. A page census would also
/// answer, and would additionally move if some unrelated arm authored
/// something — a looser oracle for no gain.
///
/// Returns `0` when the trace cannot be read, which is safe here: every
/// comparison is a *difference* between two reads taken the same way, so a
/// build that reports nothing produces equal counts and FAILS the assertions
/// rather than passing them.
fn annot_count(session: &Session) -> usize {
    session
        .trace()
        .map(|t| t.events("add-text-annot").count())
        .unwrap_or(0)
}
