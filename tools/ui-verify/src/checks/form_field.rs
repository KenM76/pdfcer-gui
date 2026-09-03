//! `form_field` — **place a form field on the page, then click an existing one
//! and get its properties.**
//!
//! The driven assertion for the operator's request of 2026-08-26, in both its
//! halves:
//!
//! > *"when I click one I should be able to click on the canvas to place the
//! > position or drag a box for size then a pop up lets me set the details for
//! > the feature"* … *"when I click on an existing form field on the page it's
//! > properties should come up in our side pane for editing it's properties."*
//!
//! # ★★★ Why this check is the only oracle for most of the feature
//!
//! Everything between a click and an authored field crosses boundaries a unit
//! test cannot: an armed tool in `egui::Memory`, a gesture resolved from a real
//! pointer, a canvas→page transform against a real page, a **second OS window**,
//! and five `pdfcer-core` verbs. The unit tests cover each rule; not one of them
//! covers the sequence, and the sequence is where this project's defects live.
//!
//! The precedent is the shell's own founding defect: the Delete key's guard was
//! *"analysis-confirmed, NOT empirically verified"*, its unit test built a bare
//! context with no widgets, and the condition that broke the real application
//! could not occur in the harness.
//!
//! # ★★ The dialog is answered by a seam, and that is not a shortcut
//!
//! `PDFCER_DIAG_FORM_ACCEPT=1` makes the placement dialog press its own Add on
//! the first frame it is authorable. This harness drives **one** window — the
//! one `Session::launch` found — and the dialog is a deferred viewport with a
//! window of its own, so without the seam everything downstream of placing is
//! unreachable: the five engine verbs, the narrowing in
//! `app::actions::forms::author`, and all four rule-4 disclosures.
//!
//! Two seams already exist for exactly this shape — `PDFCER_DIAG_OPEN_PATH` and
//! `PDFCER_DIAG_INSERT_PATH`, both substituting the answer to a native picker.
//! What this one substitutes is the **operator's press**, not the authoring:
//! it sets the same flag the Add button sets, so the readiness guard, the
//! action, the remembering and the engine call are all the path an operator
//! takes.
//!
//! # ★ The two clicks aim at deliberately different places
//!
//! The first must land on **empty page** — a click on an existing widget would
//! place a field on top of one, which is legal and would make the second phase
//! ambiguous. The second must land on a widget whose canvas rect the
//! application itself published in a `form-box` line, so the check aims at
//! where the program says the box is rather than at where the fixture author
//! thought it would be. That is the `HANDOFF.md` §2 defect-8 rule: a click that
//! hits the field next to the one it aimed at is the same screenshot as a click
//! that worked.
//!
//! # Phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | launch with `mode.edit,edit.form_text_field` | `form-tool-armed kind=Text` |
//! | B | click empty page | `form-field-open kind=Text`, then `add-form-field` succeeded |
//! | C | Escape, then click a published `form-box` | `form-field-selected field=…` |
//! | D | read the properties region | `properties.form_field` declared |

use crate::checks::driving::{self, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The commands rung on startup, in order, one per frame.
///
/// ★ The list form of `PDFCER_DIAG_INVOKE`, which exists because arming a form
/// tool takes two commands: the arm declines without `edit_content`, so Edit
/// mode has to be entered first. Using it here rather than clicking a mode
/// segment also removes a whole class of flake from this check — a mode segment
/// click that misses is a failure about the ribbon, not about forms.
///
/// ★★★ **`file.properties` is in the middle of the list, and phases D–F cannot
/// run without it.** Edit mode's default dock puts Properties in a TABBED stack
/// with Comments, Forms, Redact, Dimension groups and Attachments
/// (`app::modes::defaults`), and a tabbed stack draws only its **active** tab.
/// On 2026-08-29's sweep the active tab of that stack was not Properties, so
/// `panels::properties::formfield::section` never ran on a single frame, no
/// `properties.form_field` region was ever declared, and phase D would have
/// reported a properties pane that "did not draw" about a pane that was never
/// asked to draw. `file.properties` is `show_panel`, not a toggle — it mounts
/// the panel and brings it to the front of whatever stack holds it, from any
/// mode (`app::tests` asserts exactly that), so ringing it is idempotent.
///
/// ★ It goes AFTER `mode.edit`, because a mode change re-applies that mode's
/// default arrangement and would undo it, and BEFORE `edit.form_text_field`, so
/// that nothing runs after the tool is armed that could put it down.
const INVOKE: &str = "mode.edit,file.properties,edit.form_text_field";

/// The seam that answers the placement dialog. See the module header.
const ACCEPT_ENV: (&str, &str) = ("PDFCER_DIAG_FORM_ACCEPT", "1");

/// Traced when a form tool is armed.
const ARMED: &str = "form-tool-armed";
/// Traced when the placement dialog opens.
const OPENED: &str = "form-field-open";
/// Traced by the `vector_edit` funnel for the authoring verb.
const AUTHORED: &str = "add-form-field";
/// Traced when a click selects an existing field.
const SELECTED: &str = "form-field-selected";
/// The census line naming every **selectable** widget, in canvas space.
///
/// ★ `form-target`, not `form-box`. The two censuses describe different sets
/// and the difference is exactly what form authoring added: `form-box` lists
/// what a click can FILL, which excludes a drop-down, a push button and any
/// widget with no appearance. Aiming at that list would make this check unable
/// to reach three of the five kinds it exists to verify — and on a fixture whose
/// only text field is undrawn, unable to reach anything at all.
const BOX_LINE: &str = "form-target";
/// The properties section's published region.
const PROPERTIES_REGION: &str = "properties.form_field";
/// ★★★ **A window tall enough that a form field's properties fit in the
/// Properties pane.**
///
/// The harness's default window gives the Properties panel about **180 points**
/// of dock slot — it shares the right column with the Tool and Objects panels —
/// and a selected form field now draws about **450 points** of content there:
/// the read-only facts, the rename box, seven editable properties, the two
/// delete buttons, and the box's own four numbers.
///
/// ★★ **That is a real finding about the product and it is recorded here rather
/// than absorbed.** An operator on a 1,100 × 800 window has to scroll a
/// 180-point window through 450 points of pane to reach the controls that move
/// a box, and *"I clicked the field and there is nothing there"* is what that
/// looks like from a chair. It is written up in `FEATURES.md`; the remedy is a
/// layout decision (collapsible groups, a taller default slot, or the dialog
/// route O39 already uses for placement) and it is the operator's call, not
/// this check's.
///
/// What this check does about it is drive at a window with room, which is
/// `read_mode_chrome`'s precedent and its reasoning: a check's job is to
/// exercise the feature, and a check that failed because the *window* was small
/// would be reporting the wrong subject. The scroll loop below is kept anyway
/// and still works — a taller window makes it need fewer notches, not none.
const VIEWPORT: &str = "0,0,1400,1300";

/// **The Properties panel's own dock slot** — the scroll anchor.
///
/// `egui_shell::dock` publishes `dock.body.<panel command id>` for the body of
/// every mounted pane, and that rect is the visible slot by construction: it is
/// what the dock gave the panel, before the panel scrolled anything inside it.
/// See [`scroll_to`] for the three content rects that were tried first and how
/// each of them failed.
const PANE_REGION: &str = "dock.body.file.properties";
/// The editable-properties section, added with `EditSession::edit_field` on
/// 2026-08-27.
///
/// ★★★ Its own region, distinct from [`PROPERTIES_REGION`], and that is the
/// whole point of adding it. The section above it — the read-only facts, the
/// rename box, the delete buttons — drew perfectly well for a day while the
/// panel told the operator that required, read-only and the tooltip *"can only
/// be set when a field is placed. To change one, delete this field and place a
/// new one."* A check asserting only `properties.form_field` passed on that
/// build, correctly, because what it asserts was true.
///
/// So the two are separate names for the two separate claims: *"clicking a
/// field describes it"* and *"clicking a field lets you change it"*.
const EDITABLE_REGION: &str = "properties.field_edit";
/// The Required checkbox — the single control an operator reaches for first,
/// and the one O39's row named by name.
const REQUIRED_REGION: &str = "properties.field_edit.required";
/// The `edit-field` label `vector_edit` writes when the change reached the
/// engine.
///
/// ★★ Named after the ENGINE verb, so the line says which crate did the work —
/// the convention `format-text` follows, and the one that was learned the hard
/// way when a module's summary line and `vector_edit`'s label shared a name and
/// a check read the wrong one.
const EDIT_APPLIED: &str = "edit-field";
/// How many notches to spend looking for the editable properties below the
/// Properties panel's fold. `restyle_text` spends the same number looking for
/// Bold, in the same panel, for the same reason.
const SCROLL_ATTEMPTS: usize = 6;
// ★★★ `properties.widget_edit` is deliberately NOT a constant here, and the
// reason is a finding rather than tidiness.
//
// It was one, used as this step's scroll anchor, and the wheel went nowhere.
// That section's rect is published with the **ungated** `ui_rect` — correctly,
// so it survives being taller than its slot — which means it is published even
// when the section is **entirely off screen**. Aiming `declared_center` at it
// then aims the wheel outside the panel, and outside the window.
//
// ⇒ **An ungated region is a bad scroll anchor precisely because it is
// ungated.** A scroll anchor has to be something *known visible*, and the only
// thing that guarantees that is the visibility gate. So this step anchors on
// `properties.form_field`, whose rect is the panel's own `max_rect` and is
// therefore always inside the panel by construction — the same anchor the
// field-scoped step above uses, and it is not a coincidence that the two
// working anchors are the two whose rects cannot leave the viewport.
/// Its X spinner, which this check scrubs.
const WIDGET_X_REGION: &str = "properties.widget_edit.geometry.x";
/// Its Apply button.
const WIDGET_APPLY_REGION: &str = "properties.widget_edit.apply";
/// The `edit-widget` label `vector_edit` writes when the move reached the
/// engine.
const WIDGET_APPLIED: &str = "edit-widget";

/// Placing a form field, and selecting one that already exists.
pub struct FormFieldPlaceAndSelect;

impl Check for FormFieldPlaceAndSelect {
    fn name(&self) -> &'static str {
        "form_field"
    }

    fn defect(&self) -> &'static str {
        "the five form-field commands arm a tool and a click on the page places nothing — or a \
         field is placed and clicking it again offers no way to rename or delete it, leaving \
         every form pdfcer authors editable only by authoring it again"
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

/// One `form-box` census line, parsed back into a canvas rectangle.
struct PlacedBox {
    page: usize,
    field: String,
    /// Canvas-space centre — what the click aims at.
    centre: (f64, f64),
}

/// Read the application's own census of where the form's boxes are.
///
/// ★★ The application's numbers, not the fixture's. `canvas/forms.rs` publishes
/// one line per widget precisely so a harness can aim at where the program says
/// the box is; a check that computed the rect from the PDF would be asserting
/// that two independent derivations agree, and would report a disagreement as a
/// hit-test failure.
fn placed_boxes(trace: &Trace) -> Vec<PlacedBox> {
    trace
        .events(BOX_LINE)
        .filter_map(|l| {
            let page = l.get("page")?.parse().ok()?;
            let field = l.get("field")?.to_owned();
            // `rect=(x,y)+(w,h)` — the canvas rect, as the census writes it.
            let raw = l.get("rect")?;
            let (min, size) = raw.split_once(")+(")?;
            let (x, y) = min.trim_start_matches('(').split_once(',')?;
            let (w, h) = size.trim_end_matches(')').split_once(',')?;
            let (x, y): (f64, f64) = (x.trim().parse().ok()?, y.trim().parse().ok()?);
            let (w, h): (f64, f64) = (w.trim().parse().ok()?, h.trim().parse().ok()?);
            Some(PlacedBox {
                page,
                field,
                centre: (x + w / 2.0, y + h / 2.0),
            })
        })
        .collect()
}

// ★★★ `scroll_to` was HERE until 2026-08-28 and now lives in
// `checks::driving`. It was written in this file because two copies of one loop
// in one check forced the extraction; a THIRD caller — the Settings dialog's
// heading sweep — is what moved it to where the shared helpers live.
//
// That check's own note had said the fix for its coverage gap was *"a real
// piece of work"*, and it was, until this existed. The gap it named — five of
// seven groups never measured — closed for the cost of an import.
//
// `driving`'s existing occupants make the same argument at length:
// *"a rule stated twice is a rule that drifts"*, and this file's own header
// records what that drift looked like when `declared_or_in_overflow` gained a
// third case and a hand-rolled copy did not.

/// Run the four phases.
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
            "no --pdf. Pass a document that already carries an /AcroForm — phase C clicks an \
             existing field, and on a drawing with no form there is nothing to click and this \
             check would silently measure only half of itself.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is three real clicks, a keystroke and \
             the foreground. Reported as SKIPPED rather than passed: a check that did not run \
             has learned nothing.",
        ));
    }
    let vocab = &ctx.profile.vocab;
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its controls are.",
            ctx.profile.name
        ))
    })?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space naming EMPTY page — somewhere the \
             fixture has no form widget. There is deliberately no default: a placement click \
             that landed on an existing field would still open the dialog, so the check would \
             pass while aiming at the wrong thing.",
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

    // --- A: launch with the tool already armed -----------------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("form_field.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env
        .push((ACCEPT_ENV.0.to_owned(), ACCEPT_ENV.1.to_owned()));
    // ★ See `VIEWPORT`: the default window's Properties slot is shorter than a
    // selected field's properties, which is a finding about the product and a
    // wrong subject for this check to fail on.
    if let Some(name) = ctx.profile.viewport_env {
        spec.env.push((name.to_owned(), VIEWPORT.to_owned()));
    }
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {} with PDFCER_DIAG_INVOKE={INVOKE} and {}={}",
        exe.display(),
        session.pid(),
        ACCEPT_ENV.0,
        ACCEPT_ENV.1
    ));
    report.artifact(session.trace_path().to_path_buf());
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
    let Some(armed) = trace.last(ARMED) else {
        return Ok(Some(format!(
            "no `{ARMED}` line. The two commands `{INVOKE}` were rung and the text-field tool \
             was not armed, so nothing below could work. Either the command has no arm, or Edit \
             mode was not entered and the arm declined on `edit_content`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if armed.get("kind") != Some("Text") {
        return Ok(Some(format!(
            "`{ARMED}` reports kind={:?}, not Text — `edit.form_text_field` armed the wrong \
             tool, which would place the wrong control for every one of the five commands.",
            armed.get("kind")
        )));
    }
    report.note("the text-field tool is armed");

    // --- B: click empty page, and the field is authored --------------------
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let window_point = mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?;
    let frame = session.frame()?;
    let driver = Driver::new(session.window());
    report.note(format!(
        "clicking empty page at PDF ({}, {}) on page {}",
        target.x, target.y, target.page
    ));
    driver.click_at(frame.to_screen(window_point))?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(opened) = trace.last(OPENED) else {
        return Ok(Some(format!(
            "the click placed nothing: no `{OPENED}` line. The tool was armed (phase A proved \
             it), so the failure is between the pointer and the action — the gesture did not \
             resolve to a form placement, or the click never reached the canvas. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "the dialog opened for kind={:?} named {:?}",
        opened.get("kind"),
        opened.get("name")
    ));
    // ★ The generated name is asserted non-empty because it is what makes the
    // dialog acceptable: `is_authorable` refuses a blank one, so a naming bug
    // would make the seam below do nothing and the failure would read as "the
    // engine refused" rather than "the name was never generated".
    if opened.get("name").is_none_or(str::is_empty) {
        return Ok(Some(
            "the dialog opened with no generated field name. Nothing can be authored without \
             one — Add stays greyed — so a placement would silently do nothing."
                .to_owned(),
        ));
    }
    if !trace
        .events(AUTHORED)
        .any(|l| !l.raw.contains("refused") && !l.raw.contains("failed"))
    {
        return Ok(Some(format!(
            "the dialog opened and accepted and no field was authored: no clean `{AUTHORED}` \
             line. This is the half no unit test reaches — five engine verbs behind one \
             narrowing. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("a field was authored");

    // --- C: disarm, then click a field that already exists -----------------
    //
    // ★ Escape first. The tool stays armed after a placement, exactly as a
    // markup pen does, so a second click without this would place a SECOND
    // field rather than select one — and the check would fail with a message
    // about selection when the cause was arming.
    driver.press(crate::sys::vk::ESCAPE)?;
    session.settle(12);

    let trace = session.trace()?;
    let boxes = placed_boxes(&trace);
    let Some(existing) = boxes.iter().find(|b| b.page == target.page) else {
        return Err(Error::new(format!(
            "the application published no `{BOX_LINE}` line for page {}, so this check has no \
             widget to aim at. Either the fixture's form has no widget on that page, or the \
             census stopped being written. Reported as a SKIP rather than a failure because a \
             fixture with no field on the clicked page is a harness problem, not a defect.",
            target.page
        )));
    };
    report.note(format!(
        "aiming at the field {:?} the application placed at canvas ({:.1}, {:.1})",
        existing.field, existing.centre.0, existing.centre.1
    ));

    // ★★★ CLEAR THE SELECTION ON BLANK PAPER FIRST, AND ASSERT THAT IT
    // CLEARED. `checks::formaim`'s header carries the whole finding.
    //
    // The field authored above is ALREADY SELECTED — `app::actions::forms`
    // selects what it places (`OPERATOR_REQUESTS.md` O53) — and
    // `canvas::forms::select_click` raises its action and writes its trace line
    // **only on a change**. So the previous shape of this phase clicked a field
    // that was already selected and then demanded the program announce a
    // selection that had not moved. It failed a full sweep with a sentence
    // about a click that had landed dead centre of the widget: the trace shows
    // the click resolving to page (1221.00, 1151.52) inside a rect spanning
    // x ∈ [1140.6, 1300.6], y ∈ [1141.8, 1161.8].
    //
    // ⇒ The clearing click is not a workaround; it is the missing half of the
    // observation. `select_click`'s own table says a primary click on blank
    // paper CLEARS, so this phase now asserts both rows of it, and the clearing
    // line is what makes the naming line's absence admissible evidence
    // afterwards — `crate::checks` rule 4.
    let widgets = crate::checks::formaim::targets(&trace);
    let blank =
        crate::checks::formaim::blank_canvas_point(&widgets, page, target.page, existing.centre)
            .ok_or_else(|| {
                Error::new(format!(
                    "no blank paper could be found on page {} near the field this check placed: \
                     every candidate around ({:.1}, {:.1}) is inside one of the {} widget(s) the \
                     canvas named, or off the sheet. Without a clearing click the field stays \
                     selected from authoring and the selecting click below changes nothing to \
                     observe. Reported as a SKIP: that is a property of `--pdf`, not the defect \
                     under test.",
                    target.page + 1,
                    existing.centre.0,
                    existing.centre.1,
                    widgets.len()
                ))
            })?;
    let blank_point = mapping.doc_to_window(DocPoint::new(
        target.page,
        blank.0,
        page.height_pt - blank.1,
    ))?;
    driver.click_at(frame.to_screen(blank_point))?;
    session.settle(20);

    let trace = session.trace()?;
    // ★ `field=` absent IS the cleared line: the application writes
    // `form-field-selected none`, with no key/value pairs at all, for a cleared
    // selection and `field=…` for every other one.
    if !trace.events(SELECTED).any(|l| l.get("field").is_none()) {
        return Ok(Some(format!(
            "a click on blank paper at canvas ({:.1}, {:.1}) cleared nothing: no `{SELECTED} \
             none` line. A primary click on paper is an unambiguous deselect and \
             `canvas::forms::select_click` traces it, so this says the click never reached the \
             form surface — and without it the selection below cannot change, so its absence \
             would say nothing about the hit test. Trace: {}.",
            blank.0,
            blank.1,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "a click on blank paper at canvas ({:.1}, {:.1}) cleared the selection the placement \
         left behind",
        blank.0, blank.1
    ));

    // The census is canvas space; `doc_to_window` takes PDF space. The flip is
    // the one arithmetic this check does, and it is the mapping's own formula
    // read backwards: `canvas_y = page_height - doc_y`.
    let doc_y = page.height_pt - existing.centre.1;
    let point = mapping.doc_to_window(DocPoint::new(existing.page, existing.centre.0, doc_y))?;
    driver.click_at(frame.to_screen(point))?;
    session.settle(25);

    let trace = session.trace()?;
    let Some(selected) = trace
        .events(SELECTED)
        .filter(|l| l.get("field").is_some())
        .last()
    else {
        return Ok(Some(format!(
            "clicking an existing form field selected nothing: no `{SELECTED}` line with a \
             field name, on a run where the clearing click above proved the selection channel \
             is live. In Edit mode a click on a widget must select it for its properties; what \
             an operator meets if this regresses is a field they can see, can place, and \
             cannot rename or delete. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "selected {:?} on page {:?}",
        selected.get("field"),
        selected.get("page")
    ));

    // --- D: and the properties section is actually on screen ---------------
    //
    // ★ The region, not just the trace line. A selection that no panel drew is
    // the whole defect restated: the operator clicked, something was recorded,
    // and nothing appeared. This is the difference between "the model changed"
    // and "the operator can see it", and only the second is the feature.
    let drawn = trace
        .events(ui_rect)
        .any(|l| l.get("name") == Some(PROPERTIES_REGION));
    if !drawn {
        return Ok(Some(format!(
            "the field was selected and the `{PROPERTIES_REGION}` region was never declared, so \
             the properties section did not draw. The most likely causes are that the \
             Properties panel is not open in this layout — which this check cannot open and \
             should learn to — or that the section returned early. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("the form-field properties section drew");

    // --- E: …and it lets the operator CHANGE something --------------------
    //
    // ★★★ The assertion this check was missing for a day, and the day is the
    // argument for it. Steps A–D all passed on a build whose properties pane
    // was **read-only** and which told the operator, in shipped UI text, to
    // *"delete this field and place a new one"* — a destructive workaround for
    // a capability the engine already had. Every one of those steps was
    // asserting something true; none of them asked whether the pane could edit.
    // ★★ SCROLL FOR IT, the way an operator would. Found by driving: the
    // Properties panel is a `ScrollArea` whose dock slot is shorter than its
    // content, and on the operator's own drawing the form-field section runs
    // Name / Type / Page / Rename before it reaches anything editable — so the
    // new controls sit below the fold at the shipped layout.
    //
    // That is not a product defect and it is not a harness convenience either:
    // `restyle_text` had to learn the same thing for the Bold button, and both
    // are the same fact about this panel. A check that failed here would be
    // reporting "the controls are missing" about controls that are present and
    // one notch away.
    //
    // ★ It scrolls **at the section it can already see**, not at a guessed
    // point: `properties.form_field` is declared, so its centre is a coordinate
    // inside the scroll area rather than over the canvas or another panel.
    let mut required = None;
    for attempt in 0..SCROLL_ATTEMPTS {
        let trace = session.trace()?;
        if let Some(rect) = driving::declared(&trace, ui_rect, REQUIRED_REGION) {
            required = Some(rect);
            if attempt > 0 {
                report.note(format!(
                    "the editable properties were below the panel's fold; {attempt} scroll \
                     notch(es) brought them into view"
                ));
            }
            break;
        }
        let Some(anchor) = driving::declared(&trace, ui_rect, PROPERTIES_REGION) else {
            return Err(Error::new(format!(
                "the form-field section stopped being visible while scrolling for its editable \
                 properties, so there is nothing left to aim at. Trace: {}.",
                session.trace_path().display()
            )));
        };
        driver.scroll_at(session.frame()?.declared_center(anchor), -1)?;
        session.settle(12);
    }

    let trace = session.trace()?;
    let Some(required) = required else {
        let shot = ctx.out("form_field.not-editable.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ A FIELD IS SELECTED AND ITS PROPERTIES CANNOT BE CHANGED: no `{REQUIRED_REGION}` \
             region, though `{PROPERTIES_REGION}` drew.\n\
             `panels::properties::fieldedit` draws Required, Read only and a tooltip for every \
             field type, so the section is either not being called or returned early. What an \
             operator meets if this regresses is the state O39 shipped in: a pane that describes \
             their field and offers no way to change it, under a sentence advising them to \
             delete it and start again, after {SCROLL_ATTEMPTS} scroll notches looking for \
             them. Regions declared: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "properties.")),
            session.trace_path().display()
        )));
    };
    // ★★ This block used to FAIL when `EDITABLE_REGION` was absent while a
    // control inside it was present, on the reasoning that it *"should be
    // impossible — the section publishes its own rect after its controls."*
    //
    // **That was an invariant asserted without being measured, and the first
    // driven run refuted it.** The section published through
    // `diag::ui_rect_visible`, which suppresses a region less than 60 % inside
    // the clip; a seven-control section in a short dock slot is never 60 %
    // inside anything. So the controls were visible, the section was not, and
    // the check called the correct state impossible.
    //
    // The product side was fixed — a section rect is not a surface anybody
    // samples, so it takes the plain `ui_rect` now — and the assertion is kept
    // as a **note rather than a failure**, because that is what it can honestly
    // be. If it goes missing again the cause is a publishing convention, not a
    // broken feature, and a check that failed on it would be reporting the
    // wrong subject.
    if driving::declared(&trace, ui_rect, EDITABLE_REGION).is_none() {
        report.note(format!(
            "note: `{REQUIRED_REGION}` drew and `{EDITABLE_REGION}` did not. That is a \
             publishing-convention question rather than a defect in the feature — see this \
             check's own comment and `panels::properties::fieldedit`'s"
        ));
    }

    // --- F: press it, and the change reaches the document ------------------
    //
    // ★★ A checkbox, deliberately, and not the tooltip box or the max-length
    // spinner. It is **one click with a binary outcome**: a text box needs
    // typing and a focus loss to commit, and a spinner needs a scrub that has
    // to be reconciled against a speed constant — either of which makes a
    // failure ambiguous between the program and the harness's own arithmetic.
    // The same argument `restyle_text` makes for pressing Bold rather than
    // scrubbing the size.
    //
    // ★ It also toggles a real flag on a real field and leaves it toggled. That
    // is a side effect on the fixture in `--out`, not on the operator's file —
    // which is why the suite is driven against a copy of the exe and a fixture,
    // and why `CONTINUE.md` says never to drive the published build.
    let before = trace.events(EDIT_APPLIED).count();
    driver.click_at(session.frame()?.declared_center(required))?;
    session.settle(24);

    let trace = session.trace()?;
    if trace.events(EDIT_APPLIED).count() <= before {
        let shot = ctx.out("form_field.edit-did-nothing.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ REQUIRED WAS TICKED AND NOTHING REACHED THE DOCUMENT: no new `{EDIT_APPLIED}` \
             line.\n\
             Three candidates. (1) **The click missed** — the region was declared, so the \
             screenshot beside this report settles it. (2) **The press raised no action**, which \
             on a plain `ui.checkbox` means the section re-drew between the press and the read. \
             (3) **`edit_field` refused** — the engine checks its gates against the RESULTING \
             field, so a refusal can name a property the request never mentioned; the status \
             bar carries the sentence and the trace carries the decline. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★★ ticking Required reached the document through `edit_field`");

    // --- G: the BOX moves ---------------------------------------------------
    //
    // ★★★ The widget half, and it is a different verb with a different scope.
    // `edit_field` writes one property and every placement follows;
    // `edit_widget` writes one placement. A check that exercised only the first
    // would pass on a build where the second was never wired, which is the
    // state this shell was in an hour before this was written.
    //
    // ★ Scrubbed, not typed. `geometry_fields`' own header argues for a scrub
    // over a double-click-and-type: typing into an `egui::DragValue` needs a
    // focus dance the harness has no reliable way to drive, while a horizontal
    // drag is one gesture. The arithmetic does not have to be reconciled here
    // because the assertion is *"a move reached the document"*, not *"it moved
    // by exactly N points"* — the engine's own tests own the second question,
    // and a check that asserted it would be pinning `SPEED`.
    // ★ Scroll again. The widget-scoped controls sit below the field-scoped
    // ones, which were themselves below the fold — so one scroll reaches the
    // first set and not the second, and the first run of this step reported the
    // box controls missing while `properties.widget_edit` was in the very same
    // trace. The anchor is that section's own rect, which is why it is
    // published with the ungated `ui_rect`.
    let spinner = driving::scroll_to(
        &session,
        &driver,
        ui_rect,
        PANE_REGION,
        WIDGET_X_REGION,
        SCROLL_ATTEMPTS,
        report,
    )?;
    let trace = session.trace()?;
    // ★ The rect is deliberately discarded and only its PRESENCE is used: the
    // spinner is re-found after the Apply scroll below, because scrolling for
    // Apply moves it. Named `_` so that is a statement rather than an oversight.
    let Some(_) = spinner else {
        let shot = ctx.out("form_field.no-box-controls.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ THE FIELD'S BOX CANNOT BE MOVED: no `{WIDGET_X_REGION}` region.\n\
             `panels::properties::widgetedit` draws X, Y, Width, Height and an Apply for every \
             widget whose `/Rect` reads back. Two candidates: the section is not being called, \
             or the widget's `/Rect` was `None` — which is the malformed case only, because a \
             zero-area rect normalises to a `Rect` rather than to `None`. What an operator \
             meets if this regresses is a form field a few points out of place and no way to \
             fix it but deleting and re-placing it. Regions declared: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "properties.")),
            session.trace_path().display()
        )));
    };
    // ★ Apply sits below the four spinners, so reaching them is not reaching
    // it — the same one-more-notch lesson this file has now learned three
    // times, at three different depths of one pane. Scrolled for by the same
    // helper rather than assumed.
    let apply = driving::scroll_to(
        &session,
        &driver,
        ui_rect,
        PANE_REGION,
        WIDGET_APPLY_REGION,
        SCROLL_ATTEMPTS,
        report,
    )?
    .ok_or_else(|| {
        Error::new(format!(
            "the box's spinners drew and its Apply never came into view. SKIPPED rather than \
             failed: a button that was never pressed proves nothing about pressing it. \
             Trace: {}.",
            session.trace_path().display()
        ))
    })?;
    // ★ Re-find the spinner AFTER that scroll. The rect read before it names a
    // position the content has since left, and a drag aimed at it lands on
    // whatever is there now — the staleness `D:/dev/rag/egui/` records as the
    // commonest harness defect in a scrolled panel.
    let trace = session.trace()?;
    let spinner = driving::declared(&trace, ui_rect, WIDGET_X_REGION).ok_or_else(|| {
        Error::new(format!(
            "the X spinner left the view while scrolling for Apply, so there is nothing to \
             scrub. Trace: {}.",
            session.trace_path().display()
        ))
    })?;

    // ★★ Apply must be GREYED before anything is typed — R9's temporarily
    // unavailable case, and the assertion that a driven check can make where a
    // unit test cannot: it is the join between `WidgetPropsDraft::differs` and
    // the button's `add_enabled`. A build whose epsilon was wrong would render
    // Apply live the moment the pane opened on any widget whose box is not
    // exactly hundredths, which is most real documents.
    //
    // Observed by PRESSING it and asserting nothing happened, because the
    // region is published for a greyed control exactly as for a live one —
    // `egui_shell::ribbon::control`'s note gives the reason and the diag
    // channel follows it.
    let before_widget = trace.events(WIDGET_APPLIED).count();
    driver.click_at(session.frame()?.declared_center(apply))?;
    session.settle(16);
    if session.trace()?.events(WIDGET_APPLIED).count() > before_widget {
        return Ok(Some(format!(
            "★ APPLY COMMITTED A MOVE WITH NOTHING TYPED: an `{WIDGET_APPLIED}` line appeared \
             after pressing Apply on an untouched box.\n\
             The button is `add_enabled(draft.differs())`, so this means `differs()` answered \
             true for a box nobody changed — which on a real document is what an epsilon that \
             is too tight produces, because a `/Rect` read out of a file routinely carries more \
             than the two decimals the spinners show. The operator sees a program that thinks \
             they have unsaved changes they never made. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("Apply is dead until a number moves");

    // Now move it, and press.
    // ★ `offset_from`, not arithmetic on a `ScreenPoint`'s fields — `coords`'
    // standing rule is that *a coordinate is produced by a conversion and never
    // assembled*, and this is the one sanctioned displacement: a drag in screen
    // pixels from a point the application itself published.
    let frame = session.frame()?;
    let from = frame.declared_center(spinner);
    let to = frame.offset_from(from, 40.0, 0.0);
    driver.drag(from, to)?;
    session.settle(20);

    let trace = session.trace()?;
    let apply = driving::declared(&trace, ui_rect, WIDGET_APPLY_REGION).ok_or_else(|| {
        Error::new(format!(
            "Apply stopped being declared after the scrub. Trace: {}.",
            session.trace_path().display()
        ))
    })?;
    let before_widget = trace.events(WIDGET_APPLIED).count();
    driver.click_at(session.frame()?.declared_center(apply))?;
    session.settle(24);

    let trace = session.trace()?;
    if trace.events(WIDGET_APPLIED).count() <= before_widget {
        let shot = ctx.out("form_field.box-did-not-move.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ THE BOX'S X WAS SCRUBBED AND APPLY REACHED NOTHING: no new `{WIDGET_APPLIED}` \
             line.\n\
             Three candidates. (1) **The scrub did not move the value** — a `DragValue` needs \
             the press and the move in one gesture, and the screenshot beside this report \
             shows what the spinner reads. (2) **Apply stayed greyed**, which means \
             `differs()` answered false for a value that changed — an epsilon too loose. (3) \
             **`edit_widget` refused**; the status bar carries the sentence. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let line = trace.events(WIDGET_APPLIED).last().map(|l| l.raw.clone());
    report.note(format!(
        "★★★ the box moved, through `edit_widget`: {}",
        line.unwrap_or_default()
    ));
    Ok(None)
}
