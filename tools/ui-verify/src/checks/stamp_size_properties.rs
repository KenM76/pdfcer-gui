//! `stamp_size_in_the_properties_box` — a stamp **already on the page** is given
//! a new label size by typing into the properties panel, and the number is
//! asserted to reach the engine and to come back out again.
//!
//! # The report this closes
//!
//! The operator, 2026-09-09, and again after the first half was fixed:
//!
//! > *"still can't adjust the size of a stamp on the canvas, **or by entering a
//! > different size in the properties box**."*
//!
//! [`super::stamp_size`] closed the first clause: the Size chooser in the
//! *placing* dialog is pressed for real and the number is asserted to arrive at
//! `canvas::textannot::spec`. That check says nothing whatever about a stamp
//! that is already on the page, which is the second clause and the one an
//! operator hits every time they change their mind — the placing dialog is gone
//! by then and never comes back.
//!
//! This file is the payment for the other half. Engine `Pass 292.0` made it
//! possible at all: `TextAnnotStyle` grew `font_size` and `stamp_fit`, and
//! `EditSession::stamp_label_parameters` grew the read half, so a panel can now
//! both show what size a placed stamp's words are and write a new one.
//!
//! # ★★★ Why every unit test in the crate can be green while this fails
//!
//! Because the panel sits at the far end of a chain that no test in the crate
//! can stand at the near end of. The hops, and the test in front of each:
//!
//! | hop | its test | what that test cannot see |
//! |---|---|---|
//! | the session reports a stamp's label parameters | the round-trip test in `markup::tests` | it calls the reader directly; no panel is drawn |
//! | `markup.rs` puts them on the `Reading` | the `Reading` tests | most build a `Reading` by hand and never call `with_stamp_label` |
//! | `size_row` draws a spinner when the label is `Some` | — | **nothing**, because a row is drawn, not returned |
//! | the spinner raises `SetTextAnnotStyle` | — | same |
//! | the action reaches `set_text_annot_style` | the apply-side tests | they raise the verb themselves, in code |
//!
//! Every one of those is green on a build whose properties panel draws the row
//! **below the fold of its own scroller**, or draws it for a `/Text` and not a
//! `/Stamp`, or draws it and never commits because the `size != read_size`
//! guard compares a value that was rounded on the way through. The operator's
//! sentence is a report about the last hop, and only a hand on the control can
//! measure it.
//!
//! ⚠ **The regions this check looks for did not exist until it was written.**
//! `properties.markup.textannot.size` and `.fit` were added to the application
//! on 2026-09-10, in the same hour as this file. That is worth stating plainly:
//! the row shipped instrumented for nothing, which means it shipped
//! **unmeasurable**, which is this project's standing definition of not done.
//!
//! # ★★ The oracle, and why it is two trace lines rather than a screenshot
//!
//! A driven check cannot read a number off a picture. Two lines carry it:
//!
//! ```text
//! pdfcer-diag stamp-label-row size=24.7 source=declared-in-da fit=grow
//! pdfcer-diag set-text-annot-style-applied id=17 subtype=Stamp icon=false colour=false size=true fit=as_requested was_foreign=0 ap=...
//! ```
//!
//! * `stamp-label-row` is what the **panel is displaying**, emitted through
//!   `diag::trace_changed` so a sixty-frame-a-second panel writes one line per
//!   actual change. It is read twice — before the edit and after it — and that
//!   pair is the round trip the operator's sentence is about.
//! * `set-text-annot-style-applied` is what the **engine was asked to do**.
//!   `size=` there is `TextAnnotStyleChange::font_size_written`, a boolean: the
//!   engine's own answer to *"did I write a new size?"*, which is not the same
//!   claim as *"the panel now shows one"*, which is why both are asserted.
//!
//! Neither line existed before this feature. The lesson behind that is on the
//! record: **a trace line must carry the number a wrong build would get wrong**,
//! and before these two, every line this route emitted was byte-identical
//! between a build that carried the operator's number and one that dropped it.
//!
//! # ★★★ The falsifications, which are what make this a test
//!
//! **The size typed is checked against the size already there.** The stamp is
//! placed with the dialog's default (*Fit the box I drew*), so its label size is
//! derived from a 220 pt box and is nothing like [`WANTED_PT`]. If it ever were,
//! the shell's own `size != read_size` guard would correctly decline to write,
//! and this check would report a defect that is not one — so the two are
//! compared, and an equal pair is raised as a **harness** error, named as such,
//! rather than being allowed to look like an application failure. This project
//! has filed four defects that did not exist off one harness input that was
//! wrong.
//!
//! **The row's presence is asserted before anything is typed**, separately from
//! the value travelling. A panel that draws no size row and a panel that draws
//! one and drops the number are the same experience for the operator and two
//! entirely different defects, and a check that only looked at the end state
//! would report the second when it was the first.
//!
//! **The fit chooser is asserted too**, and for its own reason: a build can draw
//! the number and clip the chooser under it, and the operator then has a size
//! they can change with no way to say what should give when the words stop
//! fitting the box. That is a third distinct defect behind one symptom.
//!
//! # What this check does NOT claim
//!
//! It does not assert what `pdfcer-core` paints. `size=true` proves the engine
//! was asked and answered *"I wrote a size"*; `stamp-label-row size=30` proves
//! the size can be read back out of the annotation the engine produced. Whether
//! 30 pt text is legible, or whether the `/Rect` grew to hold it, is the
//! engine's own tests' business — and the box growing is disclosed off-canvas by
//! `annots::textannotstyle` rather than drawn, under R8b rule 4.

use crate::checks::driving::{
    self, SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, frame_of, list, shell_trace,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The side, in PDF points, of the rectangle dragged for the stamp.
///
/// The same 220 pt [`super::stamp_size`] uses, for the same two reasons:
/// unambiguously a drag rather than a click the gesture machine might round to
/// one, and small enough to stay on the sheet from any `--doc-point` that is
/// itself on it.
const BOX_PT: f64 = 220.0;

/// The label size this check types into the properties box.
///
/// ★ **30, and the number is load-bearing in three directions.** It must not be
/// `12` (`StampStyle::default()`'s flat size, which a build that threw the typed
/// value away and re-defaulted would produce); it must not be `24` (what
/// [`super::stamp_size`] presses in the placing dialog, so a build that somehow
/// replayed the placing choice would be caught); and it must not be whatever a
/// 220 pt box derives, which is **asserted at run time** rather than assumed —
/// see the module header's first falsification.
const WANTED_PT: f64 = 30.0;

/// How close two point sizes have to be before this check calls them the same.
///
/// ★ Half a point, and it is a tolerance rather than an equality because the
/// number makes a round trip through a `/DA` string and back. The engine writes
/// what it was given; the reader parses what it finds. A build that wrote 30 and
/// read back `29.999999` is not the defect the operator reported, and a check
/// that failed on it would send whoever read the report into the wrong file.
/// Anything a *wrong* build produces here — a dropped edit leaving the derived
/// size, a re-default to 12 — is tens of points away, not tenths.
const SAME_PT: f64 = 0.5;

/// The label-size spinner's region, as `panels::properties::markup::textannot`
/// publishes it.
const SIZE_REGION: &str = "properties.markup.textannot.size";

/// The fit chooser's region, published beside [`SIZE_REGION`].
const FIT_REGION: &str = "properties.markup.textannot.fit";

/// The dock tab that has to be brought forward before either is drawn.
///
/// ⚠ A dock draws only its **active** tab. In Review the right dock opens on
/// Comments, so a check that read the trace without pressing this would report
/// *"the panel published no size row"* about a build whose panel is perfect.
/// That mistake has been made on this project three times, in three checks, and
/// produced zero application defects.
const PROPERTIES_TAB_REGION: &str = "dock.tab.file.properties";

/// The line the canvas writes when a click selects an annotation.
const SELECT_EVENT: &str = "annot-select";

/// What the properties panel reports it is **displaying**.
const ROW_EVENT: &str = "stamp-label-row";

/// What the engine reports it **did**.
const APPLIED_EVENT: &str = "set-text-annot-style-applied";

/// See the module documentation.
pub struct StampSizeInThePropertiesBox;

impl Check for StampSizeInThePropertiesBox {
    fn name(&self) -> &'static str {
        "stamp_size_in_the_properties_box"
    }

    fn defect(&self) -> &'static str {
        "a stamp already on the page cannot be given a new label size from the properties panel \
         — the row is not drawn, is drawn out of reach below the panel's fold, or is drawn and \
         the typed number never reaches the engine, leaving every placed stamp stuck at the size \
         it was created with"
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
            "input is disabled (--no-input). This check drags a stamp onto the page, selects it, \
             brings a dock tab forward and types into a spinner. Reported as SKIPPED rather than \
             passed, because a check that cannot touch the control it is named after has \
             measured nothing.",
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. This check needs somewhere on the page to draw the stamp, and a \
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("stamp_size_properties.trace.txt"));
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
    driving::click_mode_segment(&session, &driver, ui_rect, "review")?;
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
            "the click on the Markup tab produced no tab-selected line, so nothing below it \
             would mean anything.",
        ));
    }

    // --- 2: place a stamp, at whatever size the dialog defaults to ----------
    //
    // ★ The placing size is deliberately left alone. This check is about the
    // OTHER route, and pressing the dialog's chooser here would leave it unable
    // to tell "the properties box wrote 30" from "the placing dialog did".
    let trace = session.trace()?;
    let item = declared(&trace, ui_rect, "ribbon.item.markup.stamp").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.item.markup.stamp` region on the Markup tab. Items declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.item.markup."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(14);

    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, target.page)?;
    let frame = session.frame()?;
    let from = frame.to_screen(mapping.doc_to_window(target)?);
    let to = frame.to_screen(mapping.doc_to_window(DocPoint {
        page: target.page,
        x: target.x + BOX_PT,
        y: target.y + BOX_PT,
    })?);
    driver.drag(from, to)?;
    session.settle(20);

    let trace = session.trace()?;
    if trace.events("text-annot-open").next().is_none() {
        return Ok(Some(
            "the drag completed and no `text-annot-open` line was traced, so the release did not \
             open the stamp dialog. Nothing below this point could run."
                .to_owned(),
        ));
    }
    let Some(accept) = declared(&trace, ui_rect, "text-annot.accept") else {
        return Ok(Some(format!(
            "the stamp dialog is open and declares no `text-annot.accept` region. Regions the \
             dialog declared: {}.",
            list(&declared_names(&trace, ui_rect, "text-annot."))
        )));
    };
    driver.click_at(
        frame_of(&session, &trace, ui_rect, "text-annot.accept")?.declared_center(accept),
    )?;
    session.settle(24);
    if session.trace()?.events("stamp-style").next().is_none() {
        return Ok(Some(
            "Accept was pressed and no `stamp-style` line was traced, so no stamp was authored \
             and there is nothing to restyle."
                .to_owned(),
        ));
    }
    report.note("a stamp was placed at the dialog's default size");

    // --- 3: put the tool down, then select the stamp ------------------------
    //
    // ★★★ THE TOOL MUST GO DOWN FIRST. With the stamp tool still armed, a click
    // on the page starts a SECOND stamp — and the check would then report that
    // the mark could not be selected, about a build whose selection works.
    if !driving::arm_select_from_ribbon(&session, &driver, ui_rect, report)? {
        driver.press(crate::sys::vk::V)?;
        session.settle(12);
    }
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, target.page)?;
    let frame = session.frame()?;
    let centre = frame.to_screen(mapping.doc_to_window(DocPoint {
        page: target.page,
        x: target.x + BOX_PT / 2.0,
        y: target.y + BOX_PT / 2.0,
    })?);
    driver.click_at(centre)?;
    session.settle(24);
    if session.trace()?.events(SELECT_EVENT).last().is_none() {
        return Ok(Some(format!(
            "THE PLACED STAMP COULD NOT BE SELECTED: a click at the centre of the box that was \
             just dragged produced no `{SELECT_EVENT}` line. Every route to the properties panel \
             runs through a selection, so this is upstream of the feature under test. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ the placed stamp was selected");

    // --- 4: bring the Properties tab forward --------------------------------
    let trace = session.trace()?;
    let Some(tab) = declared(&trace, ui_rect, PROPERTIES_TAB_REGION) else {
        return Ok(Some(format!(
            "the right dock declared no `{PROPERTIES_TAB_REGION}`. Tabs declared: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "dock.tab.")),
            session.trace_path().display()
        )));
    };
    driver.click_at(session.frame()?.declared_at(tab, 0.5, 0.5))?;
    session.settle(24);

    // --- 5: ★★★ IS THE SIZE ROW EVEN THERE? --------------------------------
    let trace = session.trace()?;
    let Some(field) = declared(&trace, ui_rect, SIZE_REGION) else {
        let row = trace.events(ROW_EVENT).last().map(|l| l.raw.clone());
        return Ok(Some(format!(
            "★★★ THERE IS NO LABEL-SIZE FIELD FOR A PLACED STAMP. No `{SIZE_REGION}` region \
             after the Properties tab was brought forward — which is the operator's sentence, \
             verbatim: he cannot enter a different size in the properties box.\n\
             The panel's own row line last said: {}. If there is no such line at all, `size_row` \
             returned before drawing anything, and its two guards say which: the selection is \
             not a `/Stamp`, or `EditSession::stamp_label_parameters` answered `None` for this \
             one — the honest answer for a stamp whose appearance paints no text, but this \
             stamp's appearance paints the word pdfcer just put there. If the line IS present, \
             the row was computed and not drawn, or is scrolled below the properties panel's \
             fold: the region is published through `ui_rect_visible`, which is deliberately \
             silent for a rectangle outside the panel's clip.\n\
             Regions beginning `properties.`: {}. Trace: {}.",
            row.as_deref().unwrap_or("<no row line at all>"),
            list(&declared_names(&trace, ui_rect, "properties.")),
            session.trace_path().display()
        )));
    };
    report.note("★★ the properties panel drew a label-size field for the placed stamp");

    // ★ A SEPARATE defect, asserted separately. See the module header: a size
    // the operator can change with no way to say what gives when the words stop
    // fitting is a different failure from no size at all.
    if declared(&trace, ui_rect, FIT_REGION).is_none() {
        return Ok(Some(format!(
            "the label-size field is on screen and `{FIT_REGION}` is not, so the fit chooser \
             that qualifies it was not drawn — or was drawn below the panel's fold. The operator \
             can ask for words that no longer fit the box and has no way to say whether the box \
             should grow, the words should shrink, or the ends should be cut. Regions beginning \
             `properties.markup.textannot`: {}.",
            list(&declared_names(
                &trace,
                ui_rect,
                "properties.markup.textannot"
            ))
        )));
    }

    // --- 6: what does it SAY, before anything is typed? ---------------------
    let Some(before) = row_size(&session) else {
        return Ok(Some(format!(
            "the label-size field is on screen and no `{ROW_EVENT} size=` line was traced, so \
             what the control is displaying cannot be observed. That line is written by \
             `size_row` through `diag::trace_changed` on the frame the row is drawn; its absence \
             means the region was published by something other than the row, or the diagnostic \
             channel is off for this process. ⚠ Reported rather than assumed: a check that \
             narrated an absence it had not measured is this project's standing lesson about \
             unevidenced excuses."
        )));
    };
    report.note(format!("the field is seeded at {before} pt from the file"));

    // ★★★ THE HARNESS'S OWN FALSIFICATION. See the module header.
    if (before - WANTED_PT).abs() < SAME_PT {
        return Err(Error::new(format!(
            "the placed stamp's label is already {before} pt, which is what this check types. \
             The shell's `size != read_size` guard would correctly decline to write, and the \
             check would report a defect that is not one. This is a HARNESS condition, not an \
             application failure: change `WANTED_PT`, or pass a `--doc-point` whose {BOX_PT} pt \
             box derives a different size."
        )));
    }

    // --- 7: type a new size ------------------------------------------------
    //
    // ★★ A CLICK, then select-all, then the digits, then Enter. egui's
    // `DragValue` enters keyboard-edit mode on `clicked()` and selects its own
    // text as it does; the explicit Ctrl+A is belt and braces, because a build
    // that lost the select-all would otherwise append the digits to what was
    // there and this check would type `24.730` — a number in range, committed,
    // and nothing like what was asked for.
    //
    // ⚠ Enter and not a click elsewhere. The shell commits on `drag_stopped()
    // || lost_focus()`, and clicking away to blur would also land a click on
    // whatever is under it — on this panel, another row.
    //
    // ★★★ The anchor is taken BEFORE the click, not before the Enter. A
    // `DragValue` that has been clicked is already in keyboard-edit mode and the
    // row keeps drawing while it is; anchoring later would let a line the click
    // itself provoked count as evidence that the *typing* landed.
    let mark = session.trace()?.mark();
    let frame = session.frame()?;
    driver.click_at(frame.declared_center(field))?;
    session.settle(12);
    driver.press_chord(&[crate::sys::vk::CONTROL], crate::sys::vk::A)?;
    driver.type_ascii(&format!("{WANTED_PT:.0}"))?;
    session.settle(8);
    driver.press(crate::sys::vk::ENTER)?;
    session.settle(30);

    // --- 8: did the ENGINE get it? -----------------------------------------
    let trace = session.trace()?;
    let applied: Vec<String> = trace.events(APPLIED_EVENT).map(|l| l.raw.clone()).collect();
    // ⚠ `last_after`, never `last`. This verb is reachable from the colour and
    // icon rows of the same panel, and a stray earlier call would leave a line
    // that satisfies the assertion below without the typed size having gone
    // anywhere. The list quoted on failure is deliberately the WHOLE capture's,
    // because "there are three of these and none since you typed" is the fact a
    // reader needs and a filtered list hides.
    let Some(last) = trace.last_after(APPLIED_EVENT, mark) else {
        return Ok(Some(format!(
            "{WANTED_PT:.0} was typed into the label-size field and Enter pressed, and no \
             `{APPLIED_EVENT}` line was traced — so `SetTextAnnotStyle` never reached the \
             engine. The commit is guarded by `(drag_stopped() || lost_focus()) && size != \
             read_size`: either Enter did not surrender focus, or the parsed value came back \
             equal to the one already there. The field was seeded at {before}. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if last.get("size") != Some("true") {
        return Ok(Some(format!(
            "the engine was called and reports `size={:?}`, not `size=true`. That field is \
             `TextAnnotStyleChange::font_size_written` — the engine's own answer to *did I write \
             a new size* — so the action reached it carrying `font_size: None`. That is the \
             silent decline `TextAnnotStyle`'s contract makes possible: a field left `None` is a \
             field left alone, and it compiles. Lines traced: {}.",
            last.get("size"),
            list(&applied)
        )));
    }
    report.note("the engine reports it wrote a new label size");

    // --- 9: ★★★ and does it come BACK? -------------------------------------
    //
    // The round trip is the operator's sentence. A build that writes a `/DA` its
    // own reader cannot then parse leaves the panel showing the old number after
    // a successful edit — which reads, to the person in front of it, as exactly
    // the thing he reported.
    let Some(after) = row_size_since(&session, mark) else {
        let still = row_size(&session);
        return Ok(Some(format!(
            "the engine reported it wrote a new label size and the properties row never \
             re-reported. It last said {still:?} pt, from before the edit. `stamp-label-row` \
             is a `trace_changed` slot written on every frame the row is drawn, so silence \
             here is not *the value is unchanged* — it is **the row was not drawn again at \
             all**. ⇒ Look at the selection, not at the size: the most likely reading is \
             that re-baking the appearance replaced the annotation and the panel is now \
             looking at nothing, which leaves the operator staring at a properties box that \
             emptied itself the moment he pressed Enter."
        )));
    };
    if (after - WANTED_PT).abs() >= SAME_PT {
        return Ok(Some(format!(
            "{WANTED_PT:.0} pt was typed, the engine reported it wrote a size, and the \
             properties box now reads {after} pt. The number did not survive the round trip \
             through the file: either the shell sent something other than what was typed, or \
             `stamp_label_parameters` reads back something other than what was written. It was \
             {before} pt before the edit."
        )));
    }
    report.note(format!(
        "★★★ the properties box reads {after} pt after the edit — typed, written, and read back"
    ));
    Ok(None)
}

/// The size the properties panel is currently **displaying**, in points.
///
/// # ★★ Why the panel's own line and not the `ui-rect` line
///
/// `ui-rect` carries a name and a rectangle and no text whatsoever, so a check
/// that tried to read the number out of it would answer `None` on every build
/// that has ever existed — and would then go on to say the field *"reads
/// nothing"*, narrating an absence it never measured. The application grew
/// `stamp-label-row` for this. Every caller here treats `None` as a **failure to
/// observe** and reports it as such, never as a reading.
/// # ★★★ `last`, and why a fossil is the RIGHT reading here
///
/// This suite's standing hazard is reading a whole capture's `last()` and
/// getting a line the surface stopped emitting some time ago — three wrong
/// defect reports came from exactly that. `stamp-label-row` is the one shape
/// where the fossil is the answer: it is written through `diag::trace_changed`,
/// a **state slot**, so the newest line is by construction what the row is
/// displaying now, and silence means *nothing changed* rather than *nobody is
/// looking*. Reading only lines newer than some anchor would turn "the panel
/// still shows the old number" — which is the operator's complaint, exactly —
/// into "the panel reports nothing", and send the reader to the wrong file.
///
/// [`row_size_since`] is the companion for the one question this cannot answer:
/// *did the row redraw at all since I pressed Enter?*
fn row_size(session: &Session) -> Option<f64> {
    parse_row(session.trace().ok()?.last(ROW_EVENT))
}

/// The size the row **re-reported after** line `after`, or `None` if it has not
/// re-reported since.
///
/// ★★ The distinction [`row_size`] cannot draw. After a successful edit
/// there are three outcomes and only two of them are visible to a whole-capture
/// read: the row re-reported a new size (good), the row re-reported the same
/// size (a failed round trip), or **the row said nothing at all** — which means
/// the panel did not redraw it, and points at the selection rather than at the
/// size. Separating the third is what stops one message being written about two
/// different defects.
fn row_size_since(session: &Session, after: usize) -> Option<f64> {
    parse_row(session.trace().ok()?.last_after(ROW_EVENT, after))
}

/// The `size=` field off a `stamp-label-row` line, as a number.
fn parse_row(line: Option<&crate::trace::TraceLine>) -> Option<f64> {
    line?.get("size")?.parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::{FIT_REGION, SAME_PT, SIZE_REGION, WANTED_PT};

    /// ★ **The number typed is not one another route could have produced.**
    ///
    /// `12` is `StampStyle::default()`; `24` is what `stamp_size` presses in the
    /// placing dialog. A build that ignored this field and re-defaulted, or one
    /// that somehow replayed the placing choice, would trace a size that looked
    /// like a pass if [`WANTED_PT`] were either of them. Asserted here because
    /// the reasoning lives in a doc comment, and doc comments do not fail.
    #[test]
    fn the_typed_size_collides_with_nothing_else_in_the_suite() {
        assert!(
            (WANTED_PT - 12.0).abs() > SAME_PT,
            "12 pt is StampStyle::default()"
        );
        assert!(
            (WANTED_PT - 24.0).abs() > SAME_PT,
            "24 pt is what the placing-dialog check presses"
        );
        assert!(
            (4.0..=144.0).contains(&WANTED_PT),
            "outside the spinner's own range the value would be clamped on the way in, and the \
             check would compare what it typed against what egui allowed"
        );
    }

    /// ★★ **The two regions are distinct and share the row's prefix.**
    ///
    /// They are the harness's hand-written copies of two constants in the
    /// application, and a hand-written copy is exactly where two ends drift.
    /// Equal names would make the fit assertion pass on a build that draws only
    /// the spinner — the third defect the module header names, silently
    /// unmeasured.
    #[test]
    fn the_two_regions_are_distinct_and_belong_to_the_same_row() {
        assert_ne!(SIZE_REGION, FIT_REGION);
        for region in [SIZE_REGION, FIT_REGION] {
            assert!(
                region.starts_with("properties.markup.textannot."),
                "`{region}` must sit under the subsection that publishes it, or the failure \
                 message's list of `properties.markup.textannot` regions will not contain the \
                 name it is telling the reader to look for"
            );
        }
    }
}
