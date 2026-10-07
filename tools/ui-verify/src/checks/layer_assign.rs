//! `layer_assign_moves_the_selection` — a selected page object is put on a
//! layer from the Properties panel's Layer combo, Ctrl+Z takes it off again,
//! and a selected annotation is put on a layer from its right-click Move to
//! layer… window. `paste_goes_on_the_current_layer`,
//! `a_caret_goes_on_the_current_layer` and
//! `a_drawing_goes_on_the_current_layer` — what is added while a layer is
//! current lands on it.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/layer_assign.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, click_mode_segment, declared, declared_in, declared_names,
    declared_or_in_overflow, list, repo_fixture,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

macro_rules! step {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(why) => return Ok(Some(why)),
        }
    };
}

const FIXTURE: &str = "layer-assign.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/layer-assign.PROVENANCE.py`.";
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const PROPERTIES_TAB: &str = "dock.tab.file.properties";
const COMBO: &str = "properties.layer.combo";
const WALLS: &str = "properties.layer.option.Walls";
const MENU_ROW: &str = "menu.item.canvas.markup.format.move_to_layer";
const WINDOW_COMBO: &str = "layer-assign.window.combo";
const WINDOW_NOTES: &str = "layer-assign.window.option.Notes";
const WINDOW_GO: &str = "layer-assign.window.go";
const ASSIGNED: &str = "layer-assigned";
const WALLS_ROW: &str = "panel.layers.row.Walls";
const VIEW_TAB: &str = "ribbon.tab.view";
const LAYERS_ITEM: &str = "ribbon.item.view.panel_layers";
/// The fixture's `Walls` and `Notes` groups, as the trace writes an id.
const WALLS_ID: &str = "layer=6_0";
const NOTES_ID: &str = "layer=7_0";

const MARKUP_TAB: &str = "ribbon.tab.markup";
const INSERT_TEXT: &str = "ribbon.item.markup.insert_text";
const CARET_ACCEPT: &str = "text-annot.accept";
const EDIT_TAB: &str = "ribbon.tab.edit";
const INSERT_IMAGE: &str = "ribbon.item.edit.insert_image";
const IMAGE_INSERT: &str = "insert-image.insert";
const IMAGE_PATH_ENV: &str = "PDFCER_DIAG_IMAGE_PATH"; // ui-text-exempt: an environment variable name
const DRAWING: &str = "vector-art.svg";
/// An empty spot on the page, clear of the box and the annotation; the
/// caret's apex goes here and its 10 pt body hangs below.
const SPOT: (f64, f64) = (150.0, 450.0);

/// The centre of the fixture's unlayered blue box, in page points.
const BOX: (f64, f64) = (250.0, 200.0);
/// A point inside the fixture's `/Square` annotation, in page points.
const ANNOT: (f64, f64) = (600.0, 435.0);

/// Objects pasted while Walls is the current layer land on Walls.
pub struct PasteGoesOnTheCurrentLayer;

impl Check for PasteGoesOnTheCurrentLayer {
    fn name(&self) -> &'static str {
        "paste_goes_on_the_current_layer"
    }

    fn defect(&self) -> &'static str {
        "clicking a layer's name did not make it the layer new content goes on, or objects \
         pasted afterwards did not land on it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match with_drive(ctx, &mut report, draw_on) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// A caret added while Walls is the current layer lands on Walls.
pub struct ACaretGoesOnTheCurrentLayer;

impl Check for ACaretGoesOnTheCurrentLayer {
    fn name(&self) -> &'static str {
        "a_caret_goes_on_the_current_layer"
    }

    fn defect(&self) -> &'static str {
        "an annotation added while a layer is current (here a caret) did not land on that \
         layer: its adder was handed no layer, or the layer did not reach the engine"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match with_drive(ctx, &mut report, caret_on) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Open the Layers panel if needed and click Walls' name; the Walls row after.
fn make_walls_current(d: &Drive<'_>) -> Result<std::result::Result<String, String>> {
    if declared(&d.session.trace()?, d.ui_rect, WALLS_ROW).is_none() {
        for (region, family) in [
            (VIEW_TAB, "ribbon.tab."),
            (LAYERS_ITEM, "ribbon.item.view."),
        ] {
            if let Err(why) = d.press(region, family)? {
                return Ok(Err(why));
            }
        }
        d.session.settle(20);
    }
    let mark = d.session.trace()?.mark();
    if let Err(why) = d.press(WALLS_ROW, "panel.layers.row.")? {
        return Ok(Err(why));
    }
    walls_row(d, mark).map(Ok)
}

/// Walls current; Markup > Insert text; a caret at `SPOT`; Escape; a click on
/// the caret. The engine's own `/OC` on the selected caret is the oracle.
fn caret_on(d: &Drive<'_>, report: &mut CheckReport) -> Result<Option<String>> {
    let chosen = step!(make_walls_current(d)?);
    step!(d.press(MARKUP_TAB, "ribbon.tab.")?);
    step!(d.press(INSERT_TEXT, "ribbon.item.markup.")?);
    let mark = d.session.trace()?.mark();
    d.pointer.click(d.session, d.at(SPOT)?)?;
    d.session.settle(30);
    let Some((accept, viewport)) = declared_in(&d.session.trace()?, d.ui_rect, CARET_ACCEPT) else {
        return Ok(Some(format!(
            "a click with Insert text armed opened no window: no `{CARET_ACCEPT}`."
        )));
    };
    d.pointer
        .type_text(d.session, viewport.as_deref(), "on walls")?;
    d.session.settle(20);
    // The window is its own viewport; Add is clicked there.
    d.pointer.click_in(
        d.session,
        viewport.as_deref(),
        WindowPoint::centre_of(accept),
    )?;
    d.session.settle(20);
    let trace = d.session.trace()?;
    let on = trace
        .last_after("add-caret-annot-on-layer", mark)
        .map(|l| l.raw.clone());
    let placed = trace
        .last_after("caret-annot-placed", mark)
        .map(|l| l.raw.clone());
    d.pointer.key(d.session, None, "Escape", None)?;
    d.session.settle(10);
    let mark = d.session.trace()?.mark();
    d.pointer.click(d.session, d.at((SPOT.0, SPOT.1 - 5.0))?)?;
    d.session.settle(20);
    let selected = d
        .session
        .trace()?
        .last_after("annot-select", mark)
        .map(|l| l.raw.clone());
    let row = walls_row(d, mark)?;
    report.note(format!(
        "chosen: `{chosen}`; on: {on:?}; placed: {placed:?}; selected: {selected:?}; Walls \
         after: `{row}`"
    ));
    let mut findings = Vec::new();
    if !chosen.contains("current=1") {
        findings.push("clicking Walls' name did not make it current (current=1).");
    }
    if on.is_none() {
        findings.push("the caret's add traced no `add-caret-annot-on-layer`.");
    }
    if placed.is_none() {
        findings.push("no `caret-annot-placed`: the engine never authored the caret.");
    }
    if selected.is_none() {
        findings.push("a click on the placed caret traced no `annot-select`.");
    } else if !row.contains("highlighted=true") {
        findings.push("the placed caret, selected, is not on Walls (highlighted=true).");
    }
    Ok((!findings.is_empty()).then(|| {
        format!(
            "{} Trace: {}.",
            findings.join(" "),
            d.session.trace_path().display()
        )
    }))
}

/// A drawing goes on the current layer: Walls current; Edit > Insert image
/// with an SVG; Insert. The placed drawing is selected, and Walls' row says
/// the selection is on it.
pub struct ADrawingGoesOnTheCurrentLayer;

impl Check for ADrawingGoesOnTheCurrentLayer {
    fn name(&self) -> &'static str {
        "a_drawing_goes_on_the_current_layer"
    }

    fn defect(&self) -> &'static str {
        "an SVG or EMF drawing inserted while a layer is current lands on no layer: the add \
         was handed no layer, or the layer did not reach add_svg_on_layer"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match with_drive(ctx, &mut report, drawing_on) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drawing_on(d: &Drive<'_>, report: &mut CheckReport) -> Result<Option<String>> {
    let chosen = step!(make_walls_current(d)?);
    step!(d.press(EDIT_TAB, "ribbon.tab.")?);
    let Some(item) = declared_or_in_overflow(d.session, d.pointer, d.ui_rect, INSERT_IMAGE)? else {
        return Ok(Some(format!("the Edit tab declares no `{INSERT_IMAGE}`.")));
    };
    d.pointer.click(d.session, WindowPoint::centre_of(item))?;
    d.session.settle(40);
    let Some((insert, viewport)) = declared_in(&d.session.trace()?, d.ui_rect, IMAGE_INSERT) else {
        return Ok(Some(format!("Insert image declared no `{IMAGE_INSERT}`.")));
    };
    let mark = d.session.trace()?.mark();
    d.pointer.click_in(
        d.session,
        viewport.as_deref(),
        WindowPoint::centre_of(insert),
    )?;
    d.session.settle(60);
    let trace = d.session.trace()?;
    let on = trace
        .last_after("add-svg-on-layer", mark)
        .map(|l| l.raw.clone());
    let selected = trace
        .last_after("selection-set", mark)
        .map(|l| l.raw.clone());
    let row = walls_row(d, mark)?;
    report.note(format!(
        "chosen: `{chosen}`; on: {on:?}; selected: {selected:?}; Walls after: `{row}`"
    ));
    let mut findings = Vec::new();
    if !chosen.contains("current=1") {
        findings.push("clicking Walls' name did not make it current (current=1).");
    }
    if on.is_none() {
        findings.push("the drawing's add traced no `add-svg-on-layer`.");
    }
    if selected.is_none() {
        findings.push("the placed drawing was not selected (`selection-set`).");
    } else if !row.contains("highlighted=true") {
        findings.push("the placed drawing, selected, is not on Walls (highlighted=true).");
    }
    Ok((!findings.is_empty()).then(|| {
        format!(
            "{} Trace: {}.",
            findings.join(" "),
            d.session.trace_path().display()
        )
    }))
}

/// The last `layer-row` line for Walls after `mark`, raw.
fn walls_row(d: &Drive<'_>, mark: usize) -> Result<String> {
    Ok(d.session
        .trace()?
        .lines
        .iter()
        .rfind(|l| l.lineno > mark && l.event == "layer-row" && l.raw.contains("name=\"Walls\""))
        .map(|l| l.raw.clone())
        .unwrap_or_default())
}

fn draw_on(d: &Drive<'_>, report: &mut CheckReport) -> Result<Option<String>> {
    let chosen = step!(make_walls_current(d)?);
    d.pointer.click(d.session, d.at(BOX)?)?;
    d.session.settle(20);
    d.pointer.key(d.session, None, "C", Some("ctrl"))?;
    d.session.settle(10);
    let mark = d.session.trace()?.mark();
    d.pointer.key(d.session, None, "V", Some("ctrl"))?;
    d.session.settle(40);
    // The paste lands 5 pt up and on top, so a click on the box picks it.
    d.pointer.click(d.session, d.at(BOX)?)?;
    d.session.settle(20);
    let pasted = walls_row(d, mark)?;
    report.note(format!(
        "after the click: `{chosen}`; after the paste: `{pasted}`"
    ));
    let mut findings = Vec::new();
    if !chosen.contains("current=1") {
        findings.push("clicking Walls' name did not make it current (current=1).");
    }
    if !pasted.contains("highlighted=true") {
        findings.push("the pasted object, clicked, is not on Walls (highlighted=true).");
    }
    Ok((!findings.is_empty()).then(|| {
        format!(
            "{} Trace: {}.",
            findings.join(" "),
            d.session.trace_path().display()
        )
    }))
}

/// See the module documentation.
pub struct LayerAssignMovesTheSelection;

impl Check for LayerAssignMovesTheSelection {
    fn name(&self) -> &'static str {
        "layer_assign_moves_the_selection"
    }

    fn defect(&self) -> &'static str {
        "a selected object or annotation cannot be put on a layer: the Properties panel shows no \
         Layer combo, the right-click menu has no Move to layer…, or a choice never reaches \
         set_objects_layer / set_annotation_layer, or Ctrl+Z does not take the move back"
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

struct Drive<'a> {
    session: &'a Session,
    pointer: &'a ScriptedPointer,
    ui_rect: &'static str,
    mapping: CanvasMapping,
}

impl Drive<'_> {
    /// Click a declared region, or the failure naming the regions under `family`.
    fn press(&self, region: &str, family: &str) -> Result<std::result::Result<(), String>> {
        let trace = self.session.trace()?;
        let Some(r) = declared(&trace, self.ui_rect, region) else {
            return Ok(Err(format!(
                "no `{region}` region. Regions beginning `{family}`: {}. Trace: {}.",
                list(&declared_names(&trace, self.ui_rect, family)),
                self.session.trace_path().display()
            )));
        };
        self.pointer
            .click(self.session, WindowPoint::centre_of(r))?;
        self.session.settle(20);
        Ok(Ok(()))
    }

    fn at(&self, p: (f64, f64)) -> Result<WindowPoint> {
        self.mapping.doc_to_window(DocPoint::new(0, p.0, p.1))
    }

    /// The `layer-assigned` line after `mark`, or the failure.
    fn assigned(&self, mark: usize, what: &str) -> Result<std::result::Result<String, String>> {
        let trace = self.session.trace()?;
        Ok(match trace.last_after(ASSIGNED, mark) {
            Some(line) => Ok(line.raw.clone()),
            None => Err(format!(
                "{what}, and no `{ASSIGNED}` line followed: the engine was never asked. \
                 Refusals: {}. Trace: {}.",
                trace
                    .last_after("layer-assign-refused", mark)
                    .map_or("none", |l| l.raw.as_str()),
                self.session.trace_path().display()
            )),
        })
    }
}

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("layer_assign.trace.txt"));
    spec.pdf = Some(repo_fixture(FIXTURE, METHOD)?);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    let drawing = repo_fixture(DRAWING, "See fixtures/vector-art.PROVENANCE.md.")?;
    spec.env
        .push((IMAGE_PATH_ENV.to_owned(), drawing.display().to_string()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("layer_assign.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(40);
    Ok((session, pointer))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    with_drive(ctx, report, |d, report| {
        if let Some(failure) = objects(d, report)? {
            return Ok(Some(failure));
        }
        annotation(d, report)
    })
}

/// Launch on the fixture in Edit mode and run `body` against it.
fn with_drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    body: impl FnOnce(&Drive<'_>, &mut CheckReport) -> Result<Option<String>>,
) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report)?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile names no ui-rect trace event."))?;
    click_mode_segment(&session, &pointer, ui_rect, "edit")?;
    session.settle(20);
    let pdf = repo_fixture(FIXTURE, METHOD)?;
    let page = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let d = Drive {
        session: &session,
        pointer: &pointer,
        ui_rect,
        mapping,
    };
    body(&d, report)
}

/// The blue box onto Walls from the combo; Ctrl+Z; onto Walls again.
fn objects(d: &Drive<'_>, report: &mut CheckReport) -> Result<Option<String>> {
    d.pointer.click(d.session, d.at(BOX)?)?;
    d.session.settle(20);
    if declared(&d.session.trace()?, d.ui_rect, COMBO).is_none() {
        step!(d.press(PROPERTIES_TAB, "dock.tab.")?);
        d.session.settle(10);
    }
    let mut first = String::new();
    for round in ["the first choice", "the choice after Ctrl+Z"] {
        step!(d.press(COMBO, "properties.")?);
        let mark = d.session.trace()?.mark();
        step!(d.press(WALLS, "properties.layer.")?);
        let line = step!(d.assigned(mark, &format!("Walls was chosen ({round})"))?);
        report.note(format!("{round}: `{line}`"));
        if !(line.contains("kind=objects")
            && line.contains(" moved=1 ")
            && line.ends_with(WALLS_ID))
        {
            return Ok(Some(format!(
                "{round} did not move the one selected object: `{line}`."
            )));
        }
        if first.is_empty() {
            first = line;
            let mark = d.session.trace()?.mark();
            d.pointer.key(d.session, None, "Z", Some("ctrl"))?;
            d.session.settle(30);
            if d.session
                .trace()?
                .last_after("undo-applied", mark)
                .is_none()
            {
                return Ok(Some(
                    "Ctrl+Z after the move traced no `undo-applied`: the move left no undo entry."
                        .to_owned(),
                ));
            }
        }
    }
    report.note("the object went onto Walls, came off with Ctrl+Z, and went on again");
    Ok(None)
}

/// The annotation onto Notes through the right-click Move to layer… window.
fn annotation(d: &Drive<'_>, report: &mut CheckReport) -> Result<Option<String>> {
    let point = d.at(ANNOT)?;
    d.pointer.click(d.session, point)?;
    d.session.settle(20);
    if d.session.trace()?.events("annot-select").last().is_none() {
        return Ok(Some(
            "a click inside the /Square annotation traced no `annot-select`.".to_owned(),
        ));
    }
    d.pointer.right_click(d.session, point)?;
    d.session.settle(20);
    step!(d.press(MENU_ROW, "menu.item.canvas.")?);
    step!(d.press(WINDOW_COMBO, "layer-assign.")?);
    let mark = d.session.trace()?.mark();
    step!(d.press(WINDOW_NOTES, "layer-assign.")?);
    step!(d.press(WINDOW_GO, "layer-assign.")?);
    let line = step!(d.assigned(mark, "Notes was chosen in the window and Move pressed")?);
    report.note(format!("annotation: `{line}`"));
    if !(line.contains("kind=annotation")
        && line.contains("Square")
        && line.contains("changed=true")
        && line.ends_with(NOTES_ID))
    {
        return Ok(Some(format!(
            "the window's Move did not put the Square annotation on Notes: `{line}`."
        )));
    }
    d.pointer.gone(d.session)?;
    Ok(None)
}
