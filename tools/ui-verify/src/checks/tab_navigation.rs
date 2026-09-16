//! `tab_moves_between_form_fields` — **Tab pressed inside a form field goes to
//! the next field, not into the ribbon** — `OPERATOR_REQUESTS.md` O204.
//!
//! # The operator's report
//!
//! > *"when I press tab while in a form I end up tabbing through the menus
//! > instead of the form items. The tab should tab through whatever space I
//! > have clicked on (example if I have an object selected on the canvase it
//! > should tab through to the next object as expected, and if I've clicked on
//! > a form item it should tab forward and shift-tab backwards to the next
//! > one."*
//!
//! # Why only a driven run can answer it
//!
//! The mechanism lives in `eframe`'s `raw_input_hook`, which runs **before**
//! `Context::run` and therefore before one line of application `ui` code. A
//! unit test that calls the ring's own step function proves the ring steps; it
//! cannot prove that the Tab press ever reached the ring, because the thing
//! that would steal it — `Focus::begin_pass` latching a focus move out of
//! `RawInput.events` — only exists inside a real frame with real widgets. That
//! is the shape of this project's founding defect: a guard that was
//! *"analysis-confirmed, NOT empirically verified"*, with a unit test whose
//! bare context had no widget the condition could occur in.
//!
//! So the oracle is the operator's sentence, driven: click a field, press Tab,
//! and assert the focus landed on **another field** rather than anywhere else.
//!
//! # The three assertions, and why the claim needs all three
//!
//! 1. `tab-claim scope=field` — the hook took the press. Without this a
//!    passing run could mean egui's own focus walk happened to land somewhere
//!    plausible.
//! 2. `tab-field … from=A to=B` with `to` differing from `from` — the ring
//!    actually moved. A ring of one traces `tab-field-alone` instead, which is
//!    correct behaviour for a one-field form and no evidence at all here.
//! 3. The same, backwards, under Shift — because *"shift-tab backwards"* is
//!    half of what he asked for and a forward-only implementation satisfies
//!    the other half completely.
//!
//! # It needs a document with at least three fillable text fields
//!
//! Two is not enough: with a ring of two, *"Tab moved to the next stop"* and
//! *"Tab wrapped straight back"* produce the same `to=` and the check cannot
//! tell a working ring from one that only ever bounces. `fixtures/
//! three-text-fields.pdf` exists for this, and `the_three_field_fixture_offers
//! _three_clickable_text_boxes` is its guard — **not** one of the engine's own
//! form fixtures, none of which carries a text field with a drawn `/AP`, and
//! an `/AP`-less field is not drawn on the canvas at all.

use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;
use crate::trace::Trace;

/// The fillable census — one line per box this canvas will let him type into.
///
/// Deliberately not `form-target`, which is the *selectable* census: a push
/// button and an undrawn widget are selectable and not fillable, and the field
/// ring walks the fillable set. Aiming from the wrong census would put the
/// click on a widget the ring does not contain and report the miss as a broken
/// Tab.
///
/// Narrowed to `kind=text` below for a second reason: this check needs a box
/// that takes the keyboard on a click. A drop-down is fillable and on the same
/// ring, but clicking it opens an option list instead.
const BOX_LINE: &str = "form-box";
/// A text field took the keyboard.
const FOCUS: &str = "form-focus";
/// The `raw_input_hook` took a Tab press for a canvas ring.
const CLAIM: &str = "tab-claim";
/// The field ring spent one.
const MOVED: &str = "tab-field";
/// The ring had exactly one stop, so the press was spent and focus did not
/// move. Correct behaviour, and useless evidence — reported as a SKIP.
const ALONE: &str = "tab-field-alone";

/// See the module documentation.
pub struct TabMovesBetweenFormFields;

impl Check for TabMovesBetweenFormFields {
    fn name(&self) -> &'static str {
        "tab_moves_between_form_fields"
    }

    fn defect(&self) -> &'static str {
        "Tab pressed while filling a form field walks egui's own focus ring into the ribbon \
         instead of moving to the next field, because `Focus::begin_pass` latches the move out \
         of `RawInput` before any application code runs. Invisible to a unit test of the ring: \
         the ring steps correctly and never receives the press"
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

/// One `form-box` census line, parsed back into a canvas-space centre.
struct Fillable {
    page: usize,
    field: String,
    centre: (f64, f64),
}

/// Read the application's own census of where its fillable boxes are.
///
/// The application's numbers, not the fixture's — a check that computed the
/// rect from the PDF would be asserting that two independent derivations agree
/// and would report a disagreement as a Tab failure.
fn fillable(trace: &Trace) -> Vec<Fillable> {
    trace
        .events(BOX_LINE)
        .filter(|l| l.get("kind") == Some("text"))
        .filter_map(|l| {
            let page = l.get("page")?.parse().ok()?;
            let field = l.get("field")?.to_owned();
            // `rect=(x,y)+(w,h)` — canvas space, as the census writes it.
            let raw = l.get("rect")?;
            let (min, size) = raw.split_once(")+(")?;
            let (x, y) = min.trim_start_matches('(').split_once(',')?;
            let (w, h) = size.trim_end_matches(')').split_once(',')?;
            let (x, y): (f64, f64) = (x.trim().parse().ok()?, y.trim().parse().ok()?);
            let (w, h): (f64, f64) = (w.trim().parse().ok()?, h.trim().parse().ok()?);
            Some(Fillable {
                page,
                field,
                centre: (x + w / 2.0, y + h / 2.0),
            })
        })
        .collect()
}

/// The last `tab-field` move, as `(from, to)`.
fn last_move(trace: &Trace) -> Option<(String, String, bool)> {
    let line = trace.events(MOVED).last()?;
    Some((
        line.get("from")?.to_owned(),
        line.get("to")?.to_owned(),
        line.get("backwards") == Some("true"),
    ))
}

#[allow(
    clippy::too_many_lines,
    reason = "one driven sequence; splitting it would hide the ORDER, which is the subject" // ui-text-exempt: lint justification
)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check clicks a fillable text field and Tabs off it, so it needs a \
             document with at least THREE of them — two cannot distinguish a step from a wrap. \
             `fixtures/three-text-fields.pdf` is the one built for it.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is a click and three keystrokes into a \
             real window. Reported as SKIPPED rather than passed: a check that did not run has \
             learned nothing.",
        ));
    }
    let vocab = &ctx.profile.vocab;
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

    // --- 1: launch ---------------------------------------------------------
    //
    // No mode click. `canvas::forms::overlay` is not mode-gated, so filling a
    // field works in whichever mode the shell starts in, and a mode segment
    // click this check does not need is one more way for it to fail with a
    // message about the wrong thing.
    let mut spec = LaunchSpec::new(&exe, ctx.out("tab_navigation.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
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

    let trace = session.trace()?;
    if !trace.started(vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so {}={} did not reach the process. Trace: {}.",
            vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }

    // --- 2: aim at the first fillable field --------------------------------
    let boxes = fillable(&trace);
    if boxes.len() < 3 {
        return Err(Error::new(format!(
            "this document publishes {} fillable text box(es) and the check needs at least 3. \
             A ring of two cannot tell \"Tab moved to the next stop\" from \"Tab wrapped\", so \
             a run on such a document would report a working ring and a broken one identically. \
             Reported as a SKIP: that is a property of `--pdf`, not the defect under test. \
             Trace: {}.",
            boxes.len(),
            session.trace_path().display()
        )));
    }
    let first = &boxes[0];
    report.note(format!(
        "{} fillable field(s); aiming at {:?} at canvas ({:.1}, {:.1})",
        boxes.len(),
        first.field,
        first.centre.0,
        first.centre.1
    ));

    let mapping = CanvasMapping::from_trace(&trace, vocab, page, first.page)?;
    let frame = session.frame()?;
    let driver = Driver::new(session.window());
    // The census is canvas space; `doc_to_window` takes PDF space. The flip is
    // the mapping's own formula read backwards: `canvas_y = page_height -
    // doc_y`.
    let point = mapping.doc_to_window(DocPoint::new(
        first.page,
        first.centre.0,
        page.height_pt - first.centre.1,
    ))?;
    driver.click_at(frame.to_screen(point))?;
    session.settle(25);

    let trace = session.trace()?;
    let Some(focused) = trace.events(FOCUS).last() else {
        return Ok(Some(format!(
            "clicking a fillable text field focused nothing: no `{FOCUS}` line. Everything below \
             is about where Tab goes FROM a focused field, so this is the precondition failing \
             rather than the feature — the hit test did not reach the form overlay, or the field \
             is not fillable in this build. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let from = focused.get("field").unwrap_or_default().to_owned();
    report.note(format!("the field {from:?} has the keyboard"));

    // --- 3: Tab forward ----------------------------------------------------
    driver.press(vk::TAB)?;
    session.settle(25);

    let trace = session.trace()?;
    if !trace.events(CLAIM).any(|l| l.get("scope") == Some("field")) {
        return Ok(Some(format!(
            "Tab was pressed with a form field focused and the canvas never claimed it: no \
             `{CLAIM} scope=field` line. This is the operator's report verbatim — the press went \
             to egui's own focus walk, which hands it to the first focusable widget of the \
             frame, and that is the ribbon. Look at `canvas::tabnav::claim` and whether \
             `eframe::App::raw_input_hook` still calls it: consuming Tab from `InputState` \
             anywhere inside the frame CANNOT work, because `Focus::begin_pass` reads \
             `RawInput.events` before any application code runs. Trace: {}.",
            session.trace_path().display()
        )));
    }
    if trace.events(ALONE).next().is_some() {
        return Err(Error::new(format!(
            "the ring reported itself as having one stop (`{ALONE}`) on a document whose census \
             named {} fillable fields. The press was claimed and spent correctly, so this is not \
             a defect — but it measures nothing about stepping. Reported as a SKIP. Trace: {}.",
            boxes.len(),
            session.trace_path().display()
        )));
    }
    let Some((moved_from, moved_to, backwards)) = last_move(&trace) else {
        return Ok(Some(format!(
            "the canvas claimed the Tab press and the field ring never spent it: no `{MOVED}` \
             line. The press is gone from egui AND did nothing here, which is worse than the \
             defect reported — Tab now does nothing at all inside a form. Look for \
             `tab-field-unfocused` (the focus was dropped between the hook and the ring) or \
             `tab-field-unringed` (the focused box is on no ring) in the trace: {}.",
            session.trace_path().display()
        )));
    };
    if backwards {
        return Ok(Some(format!(
            "a bare Tab moved the ring BACKWARDS: `{MOVED} backwards=true`. The direction is \
             read from the modifier state winit derives from key events, so this says the \
             harness's Tab arrived carrying Shift, or the two arms are crossed in \
             `canvas::tabnav::claim`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    if moved_to == moved_from {
        return Ok(Some(format!(
            "Tab left the focus where it was: `{MOVED} from={moved_from} to={moved_to}`. On a \
             ring of {} that is a ring that steps onto itself.",
            boxes.len()
        )));
    }
    report.note(format!(
        "Tab moved the keyboard from {moved_from:?} to {moved_to:?}"
    ));

    // --- 4: Shift+Tab back -------------------------------------------------
    //
    // `press_held` with LSHIFT, never `press_chord` and never `SHIFT`. The
    // shell reads the direction from winit's modifier state, which is derived
    // from key EVENTS — `VK_SHIFT` is a key no real keyboard sends, and a
    // modifier that goes down and up inside one frame's event batch can be
    // applied and undone before the key it was meant to carry is dispatched.
    let before = trace.events(MOVED).count();
    driver.press_held(&[vk::LSHIFT], vk::TAB, 1)?;
    session.settle(25);

    let trace = session.trace()?;
    let back: Vec<_> = trace.events(MOVED).skip(before).collect();
    let Some(line) = back.last() else {
        return Ok(Some(format!(
            "Shift+Tab spent nothing: no further `{MOVED}` line after the forward step worked. \
             Half the request — *\"it should tab forward and shift-tab backwards\"* — and a \
             forward-only ring satisfies the other half completely, so this is exactly the \
             failure a hand test would miss. `canvas::tabnav::claim` matches `shift_only()`; if \
             the modifier arrived as something else the press was left to egui. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if line.get("backwards") != Some("true") {
        return Ok(Some(format!(
            "Shift+Tab stepped FORWARD: `{}`. The press was claimed and spent, and the \
             direction was lost between the modifier and the ring. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }
    let returned = line.get("to").unwrap_or_default();
    if returned != moved_from {
        return Ok(Some(format!(
            "Shift+Tab went backwards and landed on {returned:?}, not back on {moved_from:?} \
             where the forward step came from. A backward step that is not the inverse of the \
             forward one is a ring whose two directions disagree about the order, which an \
             operator meets as fields they cannot reach at all. Trace: {}.",
            session.trace_path().display()
        )));
    }

    report.note(format!(
        "★★ Tab moved {moved_from:?} → {moved_to:?} and Shift+Tab returned to {returned:?}, \
         with the canvas claiming both presses off egui's focus walk"
    ));
    Ok(None)
}
