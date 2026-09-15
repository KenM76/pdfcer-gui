//! `the_font_controls_are_live_on_the_drawing_you_open` — **O198 claim 3 and
//! claim 4, measured on the operator's own file instead of on a fixture.**
//!
//! # The report this exists to answer
//!
//! `OPERATOR_REQUESTS.md` **O198**, in his words:
//!
//! > *"Also get the font selector and editing tools like [bold] and italic
//! > working. That entire area is always greyed out in the menu, and the
//! > properties area is uneditable too. This is true even when I add a new line
//! > of text."*
//!
//! Two claims, about two surfaces, in one sentence:
//!
//! | claim | surface | what must be true |
//! |---|---|---|
//! | 3 | the ribbon's Format ▸ Font group | all five controls drawn **pressable** |
//! | 4 | the Properties panel | a font editor drawn, with its face row **inside the panel's clip** |
//!
//! ★★★ **And he said it about a drawing, not about a fixture.** The document is
//! `SW41177.pdf` — a SolidWorks export, 36 sheets, 1.8 MB, whose page 0 carries
//! 5,899 paths against 4 text objects and whose labels are 5 pt. Everything
//! that makes that file hard is absent from `fixtures/paragraph.pdf`: the font
//! is subset, the text is a title block rather than a paragraph, the page is
//! 1584 × 1224 pt rather than 612 × 792, and the objects the harness must hit
//! are two screen pixels tall at fit zoom.
//!
//!
//! Driven at `--doc-point 0,1140,62` against a build carrying the uncommitted
//! `panels::properties::tool::Slot` change. Every assertion held:
//!
//! * the click selected exactly one object and it is text;
//! * the Properties panel drew a font editor **with its face row inside the
//!   clip** — claim 4 does not reproduce;
//! * all five Font controls were drawn pressable —
//!   `format.font: enabled=1 live=1, format.font_size: enabled=1 live=1,
//!   format.bold: enabled=1 live=?, format.italic: enabled=1 live=?,
//!   format.font_colour: enabled=1 live=1` — claim 3 does not reproduce.
//!
//! ★★ `live=?` is not a hedge and not a missing measurement. Bold and Italic are
//! ordinary ribbon toggles with a single predicate, so they publish no `live=`
//! field at all, and [`super::font_group::describe`] prints `?` rather than
//! inventing a `1`. The three that DO carry a second predicate all reported
//! `live=1`, which is the reading that matters: the renderer's own read-back
//! resolved a face on a subset SolidWorks font.
//!
//! ★★★ **What this does NOT say is that the operator was wrong.** He is running a
//! published build that predates the `Slot` fix, and on that build the editor
//! drew below the fold of a Properties pane opening on three always-on switches.
//! This check green and his report accurate are the same state of the world one
//! release apart, and the next release is what closes the gap. Keep this check
//! aimed at his file for exactly that reason: it is the thing that will notice
//! if a later panel section takes the top of the pane again.
//!
//! # ★★★ THE TWIN, AND WHY THERE ARE TWO CHECKS AND NOT ONE
//!
//! [`crate::checks::font_group`] asserts the same two surfaces and **pins its
//! fixture**: it opens `fixtures/paragraph.pdf` at a measured point and reads
//! `--pdf` and `--doc-point` only to ignore them. That pinning is correct for
//! what it is for — its subject is a *discoverability route*, and a route has to
//! be asserted on a page whose contents are known, or a red result is a question
//! about the aim rather than an answer about the program.
//!
//! This check is the opposite half, deliberately:
//!
//! | | `font_group` | `font_group_real` |
//! |---|---|---|
//! | document | pinned to a committed fixture | whatever `--pdf` names |
//! | aim | pinned to a measured point | whatever `--doc-point` names |
//! | question | *is the route there at all* | *does it survive HIS file* |
//! | a red result means | the program regressed | the program regressed **or** this file breaks it |
//!
//! ★★ The second row of that last cell is the whole reason to have both. A
//! measurement taken only on a fixture answers *"the feature exists"*, which was
//! never the operator's question: the feature existed, was green, and he could
//! not use it. A measurement taken only on his drawing cannot tell a regression
//! from an aim, which is the mistake that cost this project a day on
//! 2026-08-28. Two checks, two claims, and a reader who compares them gets the
//! diagnosis for free — green here and red there is a fixture problem, red here
//! and green there is something about real drawings.
//!
//! # ★★ What is shared, and why it is shared rather than copied
//!
//! The command list, the region names, the aim guard and the enablement
//! renderer all come from `font_group` as `pub(super)` items. Copying them would
//! have been the ordinary move and this repository has already paid for that
//! move nine times in private `click_tab` helpers and eleven in private
//! `workspace_root` helpers: a duplicated list diverges in silence, and a group
//! measured against a stale copy of its own membership still reports a measured
//! group.
//!
//! # The oracle
//!
//! 1. The click left **exactly one object selected and it is text** — read from
//!    `properties-panel … kind=` and `canvas-selection … sel=`. Anything else is
//!    a **SKIP**, never a failure: it is a statement about where the harness
//!    aimed. See [`super::font_group::aimed_at_one_text_object`].
//! 2. `properties.text` and `properties.text.face` are declared — claim 4.
//! 3. All five `format.*` commands report `enabled=1` and, where they have a
//!    second predicate, `live=1` — claim 3.
//!
//! ★★★ **Step 2 asks `clipped_away` before it reports an absence**, and that is
//! not defensive coding — it is the actual defect O198 claim 4 turned out to be.
//! The font editor drew perfectly and drew *below the fold* of a Properties pane
//! whose first section was three always-on preference switches, so
//! `diag::ui_rect_visible` withheld the rect and the panel looked empty from the
//! trace. A check that reported that as *"the panel drew no editor"* would send
//! a reader to the renderer, which is not where the fix was.
//!
//! # ★★ What this check does NOT do, said so nobody looks for it
//!
//! It does not press Bold and it does not assert that a restyle reached the
//! document. That is `font_group`'s phase 2 and `restyle_text`'s whole subject,
//! both on the fixture, and repeating it here would make this check's red mean
//! four things instead of two. The question here is **reachability**: can the
//! operator, on his own drawing, get to the controls at all. Whether pressing
//! them works is a different question with its own check, and O198's sentence is
//! about the first one — *"that entire area is always greyed out"* is a
//! complaint about a surface, not about an outcome.

use crate::checks::driving::{
    SHELL_DIAG_ENV, click_mode_segment, clipped_away, declared, declared_names, list,
};
use crate::checks::font_group::{
    FACE_ROW_REGION, FONT_COMMANDS, FONT_GROUP, FORMAT_TAB, MODE, TEXT_STYLE_REGION,
    aimed_at_one_text_object, describe,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The Properties pane's tab header in the dock.
///
/// ★★★ Clicked before anything is asserted about the panel, and the reason is
/// worth stating every time it appears: **a dock draws only its ACTIVE tab's
/// body.** A pane that exists and is behind another publishes nothing at all, so
/// a check reading the trace sees an absence that is indistinguishable from a
/// panel with nothing to say. Three checks in this suite have reported an
/// application defect that was this, and no application defect was present in
/// any of the three.
const PROPERTIES_TAB: &str = "dock.tab.file.properties";

/// See the module documentation.
pub struct TheFontControlsAreLiveOnTheDrawingYouOpen;

impl Check for TheFontControlsAreLiveOnTheDrawingYouOpen {
    fn name(&self) -> &'static str {
        "the_font_controls_are_live_on_the_drawing_you_open"
    }

    fn defect(&self) -> &'static str {
        "an operator clicks a piece of text on his own drawing and the font selector, the size \
         box, Bold, Italic and the colour swatch are all greyed, with the Properties panel \
         offering nothing editable either — so a capability the engine has, the conditions \
         publish and a fixture proves is unreachable on the file he actually works on"
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

/// The whole run. `Ok(None)` passes, `Ok(Some(_))` fails, `Err(_)` SKIPs.
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    // ★★★ THE AIM IS REQUIRED, AND ITS ABSENCE IS A SKIP THAT SAYS WHAT TO PASS.
    //
    // There is deliberately no default here and no fallback to the fixture. A
    // defaulted point would be a guess about where a document keeps a piece of
    // text, and a click on blank paper is symptom-identical to a hit test that
    // does not work — which is the confusion behind a defect this project filed
    // and then retracted. The twin check exists precisely so that the
    // "does the feature exist at all" question has a pinned answer; this one
    // has nothing useful to say without an aim.
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check is the one that asks whether the font controls survive a REAL \
             drawing, so it has no fixture to fall back on — the fallback is a different check \
             (`the_format_tab_offers_font_controls_for_swept_text`), which pins \
             `fixtures/paragraph.pdf` on purpose. Pass the drawing and a point on a run of text \
             in it; `RESUME.md`'s aim table has both.",
        )
    })?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(format!(
            "no --doc-point. {} is open but nothing says where in it a piece of text is, and a \
             click on blank paper reports identically to a hit test that does not work. \
             `pdfcer extract-text --json` gives the first glyph's x and y of every run; \
             `RESUME.md`'s aim table gives `0,1140,62` for the SolidWorks drawing.",
            pdf.display()
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a dock tab, clicks a piece of \
             text on the page and clicks a ribbon tab, and none of those can be simulated from \
             the trace.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    let page: PageGeometry = match ctx.page_size {
        Some((width_pt, height_pt)) => PageGeometry {
            width_pt,
            height_pt,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };
    report.note(format!(
        "driving {} at page {}, {:.1}, {:.1} on a {:.0} × {:.0} pt page",
        pdf.display(),
        target.page,
        target.x,
        target.y,
        page.width_pt,
        page.height_pt
    ));

    let mut spec = LaunchSpec::new(&exe, ctx.out("font-group-real.trace.txt"));
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
    // ★ A longer settle than the fixture checks use. This document is 1.8 MB of
    // dense vector content across 36 sheets and its first page is 5,899 paths;
    // `BENCHMARK.md` measures the first full render of a sheet of that class in
    // the hundreds of milliseconds, and a harness that started clicking before
    // the first frame would be reporting on a window that had not laid out.
    session.settle(60);
    let driver = Driver::new(session.window());

    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(24);

    // Raise the Properties pane — see [`PROPERTIES_TAB`].
    if let Some(tab) = declared(&session.trace()?, ui_rect, PROPERTIES_TAB) {
        driver.click_at(session.frame()?.declared_center(tab))?;
    }
    session.settle(24);

    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let frame = session.frame()?;
    // ★ Two points in and two points up from the named origin. `--doc-point`
    // names the first glyph's origin, which is the bottom-left corner of the
    // first character and therefore a point on the very edge of the ink. On a
    // 5 pt title-block label that is a click which can land in the paper beside
    // the letter.
    let on_text = frame.to_screen(mapping.doc_to_window(DocPoint::new(
        target.page,
        target.x + 2.0,
        target.y + 2.0,
    ))?);
    driver.click_at(on_text)?;
    session.settle(32);

    let trace = session.trace()?;
    // ★★★ THE PRECONDITION, BEFORE ANY ORACLE. Shared with the twin, which is
    // the point of sharing it: a guard that skips on a bad aim is only useful if
    // every check that can be handed a bad aim uses the same one, and this is
    // the check most likely to be handed one, because its aim comes from the
    // command line.
    aimed_at_one_text_object(&session, &trace, target)?;
    report.note("the click selected exactly one object and it is a text object");

    // =======================================================================
    // CLAIM 4 — the Properties panel offers something editable.
    // =======================================================================
    // Owned rather than borrowed only because `driving::list` takes `&[String]`,
    // which is the shape `declared_names` returns and therefore the shape every
    // other caller in this suite already has.
    let missing: Vec<String> = [TEXT_STYLE_REGION, FACE_ROW_REGION]
        .into_iter()
        .filter(|region| declared(&trace, ui_rect, region).is_none())
        .map(str::to_owned)
        .collect();
    if !missing.is_empty() {
        let shot = ctx.out("font_group_real.no-editor.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        // ★★★ Clipping FIRST. This is the shape O198 claim 4 actually had: the
        // editor drew, and drew below the fold. See the module header.
        let clipped: Vec<String> = missing
            .iter()
            .filter_map(|region| clipped_away(&trace, ui_rect, region.as_str()))
            .collect();
        if !clipped.is_empty() {
            return Ok(Some(format!(
                "★★★ THE PROPERTIES PANEL'S FONT EDITOR DREW OFF THE EDGE OF THE PANEL: {}.\n\
                 This is NOT a missing editor and the fix is NOT in the renderer. The section \
                 measured the rectangle it was about to publish against the clip it had, found \
                 too little of it inside, and declined to publish — which is `shown=` below the \
                 floor, and which an operator sees as a panel with nothing usable in it. That \
                 is `OPERATOR_REQUESTS.md` O198 claim 4 word for word: *\"the properties area \
                 is uneditable\"*.\n\
                 ★★ Look at what is ABOVE it in the screenshot beside this report. A section \
                 that draws with no reference to the selection, sitting above the sections that \
                 describe the selection, pushes them out of the pane on any dock narrower or \
                 shorter than the author's — and `panels::properties::tool::Slot` is where that \
                 ordering is decided. Trace: {}.",
                clipped.join(" | "),
                session.trace_path().display()
            )));
        }
        return Ok(Some(format!(
            "★★ A PIECE OF TEXT IS SELECTED ON {} AND THE PROPERTIES PANEL DRAWS NO FONT \
             EDITOR: no {} region, and nothing was reported clipped either.\n\
             The precondition above has already ruled out the two candidates this message would \
             otherwise lead with: it read the trace and found `properties-panel … kind=Text` \
             over a selection of exactly one, so the object IS text and it is not a \
             multi-selection. What is left, in the order worth checking. (1) **The object's \
             byte span holds no placeable show operator**, so the operand resolver found no \
             runs — look for `text-operand-resolved … runs=0`. On a SolidWorks export that is a \
             real possibility and it is a FIXTURE finding, not a defect: the verbs decline \
             honestly. (2) **The resolver was never called**, which shows as no \
             `text-operand-resolved` line at all, and IS a defect. (3) The section returned \
             early because a stale text SELECTION from an earlier gesture is live — but this \
             check sweeps nothing, so that would mean a selection surviving a document open. \
             Regions declared: {}. Trace: {}.",
            pdf.display(),
            list(&missing),
            list(&declared_names(&trace, ui_rect, "properties.")),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ the Properties panel drew a font editor with a face row for a CLICKED text object \
         on {}, with nothing swept",
        pdf.display()
    ));

    // =======================================================================
    // CLAIM 3 — the ribbon's Font group is pressable.
    // =======================================================================
    let Some(tab_rect) = declared(&trace, ui_rect, FORMAT_TAB) else {
        let shot = ctx.out("font_group_real.no-format-tab.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ CLICKING A PIECE OF TEXT RAISED NO FORMAT TAB: no `{FORMAT_TAB}` region.\n\
             The aim is not the candidate — the precondition above found exactly one selected \
             object of kind `Text`, so the tab has a subject. That leaves \
             `selection.formattable` unpublished, which is the condition the tab's \
             `visible_when` names. Tabs declared: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab.")),
            session.trace_path().display()
        )));
    };
    // The Format tab is contextual and is not the ACTIVE tab merely by
    // appearing. A ribbon draws only its active tab's band, so the Font group's
    // controls — and therefore their enablement lines — do not exist until it is
    // clicked. That is correct behaviour: a tab that stole focus on every
    // selection would move the ribbon under the operator's hand.
    driver.click_at(session.frame()?.declared_center(tab_rect))?;
    session.settle(24);

    let trace = session.trace()?;
    if declared(&trace, ui_rect, FONT_GROUP).is_none() {
        let shot = ctx.out("font_group_real.no-group.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ THE FORMAT TAB CARRIES NO FONT GROUP: no `{FONT_GROUP}` region.\n\
             A group all of whose items are hidden is not drawn at all, by design, so a single \
             unpublished condition removes the whole band silently — and every one of the five \
             names `mode.edit_content` as its `visible_when`. Groups declared: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.group.")),
            session.trace_path().display()
        )));
    }

    let states = crate::checks::driving::enablement(&session)?;
    let unpressable: Vec<String> = FONT_COMMANDS
        .into_iter()
        .filter(|id| !states.get(*id).is_some_and(|s| s.pressable()))
        .map(str::to_owned)
        .collect();
    if !unpressable.is_empty() {
        let shot = ctx.out("font_group_real.greyed.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★★★ THE FONT GROUP DREW ALL {} OF ITS CONTROLS ON {} AND {} OF THEM COULD NOT BE \
             PRESSED: {}.\n\
             That is `OPERATOR_REQUESTS.md` O198 claim 3 reproducing, on his own drawing — \
             *\"the font selector and editing tools like [bold] and italic … that entire area is \
             always greyed out in the menu\"* — on a frame in which exactly one text object is \
             selected and nothing is swept.\n\
             What each one said: {}.\n\
             ★★ READ THE TWO NUMBERS BEFORE BLAMING EITHER SIDE. `enabled=0` is the command's \
             own predicate over the published conditions refusing, and points at \
             `app::conditions` and the `selection.text_runs` join. `enabled=1 live=0` is the \
             RENDERER refusing after the condition agreed, and points at `app::fontband`'s \
             second predicate — the `resolved(doc, draft)` read-back, which runs a provenance \
             extraction and can come back empty for a subset font the extractor cannot resolve \
             while every condition about the selection stays true. ★★★ That second shape is the \
             one to expect HERE and not on the fixture: a SolidWorks export's faces are subset \
             and non-standard, and this check exists because a fixture cannot produce that \
             state. A missing entry altogether means the control never published an enablement, \
             which for the three custom ones means `app::fontband::draw` returned before its \
             report.\n\
             Trace: {}.",
            FONT_COMMANDS.len(),
            pdf.display(),
            unpressable.len(),
            list(&unpressable),
            describe(&states),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ all {} Font controls were drawn PRESSABLE on {} with nothing swept: {}",
        FONT_COMMANDS.len(),
        pdf.display(),
        describe(&states)
    ));
    Ok(None)
}
