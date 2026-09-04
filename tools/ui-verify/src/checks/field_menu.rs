//! `right_clicking_a_form_field_opens_its_menu` — **the first driven context
//! menu in this project's history.**
//!
//! # What this is for
//!
//! `OPERATOR_REQUESTS.md` **O53**, the standing acceptance criterion:
//!
//! > *"always always always I need objects on the canvas to be clickable and
//! > editable as one would expect given our research of other programs."*
//!
//! Click, drag, grips and Delete all reached a form field by 2026-08-28. The
//! **right-click** did not: `canvas::menus` chose between an object menu and a
//! view menu, and a `/Widget` is neither — it is not in `SelectionState` at all
//! — so right-clicking a text box offered *"zoom to fit width"*.
//!
//! ## ★★★ Why this check is the FIRST of its kind, which is the finding
//!
//! **This harness had driven 92 checks and had never once opened a context
//! menu.** pdfcer has had canvas right-click menus since Phase 1. Everything
//! asserted about them is a unit test over `MenuHost::would_open`, which asks
//! whether the *manifest* would offer something — a real question, and not the
//! same question as *"does a right-click on this pixel open a menu"*.
//!
//! ⇒ There was no `Driver::right_click_at`. A gesture with no driver is a
//! gesture R1 cannot reach, and **the gap left no failing test behind to
//! advertise itself**. It surfaced only because a fourth menu was added and
//! somebody went looking for the driver to exercise it with.
//!
//! ★ The same shape as `DEFECTS.md`'s two headline bugs: invisible to a green
//! suite, obvious within thirty seconds of using the program.
//!
//! ## ★★ The oracle is `canvas-menu context=…`, and it is not a screenshot
//!
//! An `egui` popup is positioned by the pointer and sized by its content, so a
//! harness that clicked *"the second row"* would be encoding a layout, and
//! would silently start choosing the wrong verb the day a menu grows an entry.
//!
//! The application publishes which menu it resolved. That line is the fact
//! under test — *did a right-click on a field produce the FIELD menu* — with no
//! coordinate in it to go stale.
//!
//! ## ★★★ What would be missed without the second assertion
//!
//! `canvas-menu-invoked` is written only when the resolved menu **has something
//! to offer**, and for ten minutes this feature's menu had nothing: both its
//! items were gated on `selection.any`, which is **false** while a form field is
//! selected, so `offers_anything` was false and the popup never opened.
//!
//! ⇒ A check asserting only the context id would have **passed** on that build
//! — the context resolved correctly and no menu appeared. `DEFECTS.md` D1's
//! shape reached through a new door, and it is why both lines are asserted.
//!
//! ## The sequence
//!
//! | # | step | oracle |
//! |---|---|---|
//! | A | place a text field (`edit.form_text_field`, self-accepting dialog) | `form-target` |
//! | B | Escape to disarm, clear on blank paper, click it | `form-field-selected none`, then `form-field-selected field=…` |
//! | C | **right-click it** | `canvas-menu context=canvas.field` |
//! | D | …and the menu had something in it | a `menu.item.canvas.field.*` region per row, inside `menu.body.canvas.field` |
//!
//! ★★★ **D's oracle was `canvas-menu-invoked` and that was a misreading**, kept
//! here because the misreading is instructive. `MenuHost::attach_with` returns
//! *"the commands the operator CHOSE"*, and the line is written only when that
//! vector is non-empty — so it reports an ACTIVATION, not an offer, and a check
//! that opens a menu and presses nothing can never see it. The rows' own
//! published rects are the offer, they name which commands were drawn, and they
//! exist for the same reason this check does: `MenuHost::attach_with` began
//! reporting them on 2026-08-28 precisely so a harness could see a menu.
//!
//! ★ Steps A and B are `widget_move`'s, identical in shape including the
//! Escape — see that file for why the placement tool staying armed is recorded
//! there as a defect rather than as scenery.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// Edit mode, then arm the text-field tool.
const INVOKE: &str = "mode.edit,edit.form_text_field";
/// Makes the placement dialog accept itself, so no dialog driving is needed.
const ACCEPT_ENV: (&str, &str) = ("PDFCER_DIAG_FORM_ACCEPT", "1");
/// The per-widget census line the canvas publishes, carrying each box's rect.
const BOX_LINE: &str = "form-target";
/// The line the canvas writes when a click selects a widget.
const SELECTED: &str = "form-field-selected";
/// `canvas-menu context=… sel=… level=…` — which menu a right-click resolved.
const MENU_EVENT: &str = "canvas-menu";
/// `canvas-menu-invoked context=… tokens=…` — the menu actually offered items.
const INVOKED_EVENT: &str = "canvas-menu-invoked";
/// The context id a form field must resolve to.
const FIELD_CONTEXT: &str = "canvas.field";
/// The page region, so a failure can say whether a sheet was drawn at all.
const PAGE_REGION: &str = "page";

/// Where the field is placed, as page fractions.
///
/// ★ Well inside the sheet, so the placement lands on paper and the right-click
/// that follows is nowhere near an edge, where a popup would be repositioned.
const PLACE_AT: (f64, f64) = (0.30, 0.55);

/// See the module documentation.
pub struct RightClickingAFormFieldOpensItsMenu;

impl Check for RightClickingAFormFieldOpensItsMenu {
    fn name(&self) -> &'static str {
        "right_clicking_a_form_field_opens_its_menu"
    }

    fn defect(&self) -> &'static str {
        "a form field can be clicked, dragged, resized and deleted and has no CONTEXT MENU — a \
         right-click on a text box resolves the view menu and offers four zoom levels, because a \
         /Widget is in neither the object selection nor the caret and the canvas had only those \
         two questions to ask"
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

/// Each widget's canvas-space centre, from the census the canvas publishes.
fn boxes(trace: &Trace) -> Vec<(usize, String, (f64, f64))> {
    trace
        .events(BOX_LINE)
        .filter_map(|l| {
            let page: usize = l.get("page")?.parse().ok()?;
            let field = l.get("field")?.to_owned();
            // `rect=(x,y)+(w,h)` — the canvas rect, as the census writes it.
            let (min, size) = l.get("rect")?.split_once(")+(")?;
            let (x, y) = min.trim_start_matches('(').split_once(',')?;
            let (w, h) = size.trim_end_matches(')').split_once(',')?;
            let (x, y): (f64, f64) = (x.trim().parse().ok()?, y.trim().parse().ok()?);
            let (w, h): (f64, f64) = (w.trim().parse().ok()?, h.trim().parse().ok()?);
            Some((page, field, (x + w / 2.0, y + h / 2.0)))
        })
        .collect()
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check places a field with a click, selects it \
             with another, and then right-clicks it — the right-click being the gesture under \
             test, and the first secondary click this harness has ever sent.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. This check needs a page to place a field on."))?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. This check places its field in page fractions \
                 and needs the box to turn them into points. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("field-menu.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env
        .push((ACCEPT_ENV.0.to_owned(), ACCEPT_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with PDFCER_DIAG_INVOKE={INVOKE}",
        exe.display(),
        session.pid()
    ));
    session.settle(45);
    let driver = Driver::new(session.window());

    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "the application declared no `{PAGE_REGION}` region, so no sheet is on screen. \
             Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // --- A: place a field ---------------------------------------------------
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let frame = session.frame()?;
    let at = DocPoint::new(0, PLACE_AT.0 * page.width_pt, PLACE_AT.1 * page.height_pt);
    driver.click_at(frame.to_screen(mapping.doc_to_window(at)?))?;
    session.settle(35);

    let trace = session.trace()?;
    let placed = boxes(&trace);
    let Some((_, field, centre)) = placed.iter().find(|(p, _, _)| *p == 0).cloned() else {
        return Ok(Some(format!(
            "THE TEXT-FIELD TOOL PLACED NOTHING: a click on the page produced no `{BOX_LINE}` \
             line for page 1, so `edit.form_text_field` did not arm or the placement dialog did \
             not accept. This is two steps BEFORE the one under test. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★ placed the field {field:?} at canvas ({:.1}, {:.1})",
        centre.0, centre.1
    ));

    // --- B: disarm, CLEAR, then select it ----------------------------------
    //
    // ★ Escape first. The tool stays armed after a placement, exactly as a
    // markup pen does, so a second click without this would place a SECOND
    // field rather than select one — and the check would fail with a message
    // about selection when the cause was arming.
    driver.press(crate::sys::vk::ESCAPE)?;
    session.settle(12);

    // ★★★ AND THEN A CLICK ON BLANK PAPER — `checks::formaim`'s header carries
    // the trace lines that put it here, and `widget_move`'s phase B is the same
    // step for the same reason.
    //
    // A field this check authored is ALREADY SELECTED (`OPERATOR_REQUESTS.md`
    // O53), and `canvas::forms::select_click` traces only on a CHANGE. So
    // clicking it without clearing first asks the program to announce a
    // selection that has not moved, and the check fails naming a click that in
    // fact landed dead centre. Clearing makes the assertion answerable and adds
    // an observation: the clearing line proves the selection channel is live
    // before its silence is read as a failure — `crate::checks` rule 4.
    let boxes = crate::checks::formaim::targets(&trace);
    let blank =
        crate::checks::formaim::blank_canvas_point(&boxes, page, 0, centre).ok_or_else(|| {
            Error::new(format!(
                "no blank paper could be found on page 1 near the field this check placed: every \
                 candidate around ({:.1}, {:.1}) is inside one of the {} widget(s) the canvas \
                 named, or off the sheet. Without a clearing click the field stays selected from \
                 authoring and the selecting click below changes nothing to observe. Reported as \
                 a SKIP: this is a property of the document, not of the menu under test.",
                centre.0,
                centre.1,
                boxes.len()
            ))
        })?;
    let blank_point = mapping.doc_to_window(DocPoint::new(0, blank.0, page.height_pt - blank.1))?;
    driver.click_at(session.frame()?.to_screen(blank_point))?;
    session.settle(20);

    let trace = session.trace()?;
    // ★ `field=` absent IS the cleared line: the application writes
    // `form-field-selected none` with no key/value pairs for a cleared
    // selection, and `field=…` for every other one.
    if !trace.events(SELECTED).any(|l| l.get("field").is_none()) {
        return Ok(Some(format!(
            "THE SELECTION COULD NOT BE CLEARED: a click on blank paper at canvas ({:.1}, {:.1}) \
             produced no `{SELECTED} none` line. A primary click on paper is an unambiguous \
             deselect and `canvas::forms::select_click` traces it, so this says the click never \
             reached the form surface at all — which would make the assertion below unreadable \
             either way. Trace: {}.",
            blank.0,
            blank.1,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ the selection was cleared on blank paper at canvas ({:.1}, {:.1})",
        blank.0, blank.1
    ));

    // The census is canvas space and `doc_to_window` takes PDF space, so the
    // flip is the one arithmetic this check does: `canvas_y = height - doc_y`.
    let doc_y = page.height_pt - centre.1;
    let field_point = mapping.doc_to_window(DocPoint::new(0, centre.0, doc_y))?;
    let field_screen = session.frame()?.to_screen(field_point);
    driver.click_at(field_screen)?;
    session.settle(25);

    let trace = session.trace()?;
    if !trace
        .events(SELECTED)
        .any(|l| l.get("field").is_some_and(|f| f == field))
    {
        return Ok(Some(format!(
            "THE FIELD COULD NOT BE SELECTED: a click at its centre produced no `{SELECTED}` \
             line naming {field:?}, on a frame where the clearing click above proved the \
             selection channel is live. So this says the click missed the widget's rect or the \
             tool was still armed. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!("★ selected {field:?}"));

    // --- C: RIGHT-CLICK it, which is the gesture under test -----------------
    driver.right_click_at(field_screen)?;
    session.settle(35);

    let trace = session.trace()?;
    let Some(menu) = trace.events(MENU_EVENT).last() else {
        return Ok(Some(format!(
            "★★★ THE RIGHT-CLICK RESOLVED NO MENU AT ALL: no `{MENU_EVENT}` line after a \
             secondary click on a selected form field.\n\
             `canvas::menus::attach` writes that line on every frame carrying a secondary click, \
             so its absence means the click never reached the canvas response. Suspect the \
             HARNESS before the menu — this is the first check in the suite to send a secondary \
             button, so `sys::mouse_button_secondary` has never been exercised anywhere else. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let context = menu.get("context").unwrap_or_default();
    if context != FIELD_CONTEXT {
        return Ok(Some(format!(
            "★★★ THE RIGHT-CLICK ON A FORM FIELD RESOLVED `{context}`, NOT `{FIELD_CONTEXT}`: \
             `{}`.\n\
             **This is the exact state the feature shipped in until 2026-08-28.** A `/Widget` is \
             in neither `SelectionState` nor the caret draft, so `attach` fell through to the \
             object hit test and then to the view menu, and the operator got four zoom levels on \
             a right-click of a text box.\n\
             `canvas::forms::right_click_hits_a_field` decides this, and it is a HIT TEST rather \
             than a read of `doc.selected_field` on purpose: the selection raised by this very \
             click is applied at the END of the frame, and egui opens the popup ON the click. \
             Trace: {}.",
            menu.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ the right-click resolved the field menu: `{}`",
        menu.raw
    ));

    // --- D: …and it had something to offer ----------------------------------
    //
    // ★★★ The second assertion, and the one the first cannot see past.
    // `canvas-menu-invoked` is written only when the resolved menu
    // `offers_anything`; a menu whose every item is DISABLED resolves its
    // context correctly and never opens.
    //
    // Not hypothetical — it is how this feature was built. Both items were
    // gated on `selection.any`, which is false while a form field is selected,
    // so the popup would not have appeared and step C would still have passed.
    // ★★★ THE ORACLE IS THE MENU'S OWN PUBLISHED ROWS, NOT `canvas-menu-invoked`,
    // AND THE SWAP IS A CORRECTION — 2026-08-29.
    //
    // This phase read `canvas-menu-invoked` and its comment said that line is
    // *"written only when the resolved menu `offers_anything`"*. It is not.
    // `MenuHost::attach_with` returns `Vec<HandlerToken>` — `egui-shell`'s own
    // words: *"report the commands the operator CHOSE … the returned tokens are
    // intent"* — and `canvas::menus` writes `canvas-menu-invoked` only when that
    // vector is non-empty. So the line means **a row was activated**, which this
    // check never does, and the assertion could not have held on any build.
    //
    // It was never caught because phase B failed first for a whole sweep: the
    // field authored in phase A is already selected (`checks::formaim`), so the
    // selecting click changed nothing and the check stopped before ever reaching
    // this line. Two independent defects, the second hidden behind the first.
    //
    // ⇒ The right oracle was added on the same day the menu was: since
    // 2026-08-28 `MenuHost::attach_with` reports every row's rect through
    // `crate::diag::ui_rect` as `menu.body.<context>` and
    // `menu.item.<context>.<command id>`. Those regions ARE the menu having
    // something in it — they exist only for rows that were laid out — and they
    // name WHICH commands were offered, which `canvas-menu-invoked` never did.
    //
    // ★ Admissible as evidence because phase C proved the menu resolved on this
    // very frame (`canvas-menu context=canvas.field`): the absence below can
    // only mean "resolved and drew no rows", never "no right-click happened".
    // `crate::checks` rule 4.
    let rows = declared_names(&trace, ui_rect, &format!("menu.item.{FIELD_CONTEXT}."));
    if rows.is_empty() {
        return Ok(Some(format!(
            "★★★ THE FIELD MENU RESOLVED AND OFFERED NOTHING: `{}`, and no `menu.item.{FIELD_CONTEXT}.*` region was ever declared.
             Every item is disabled or withheld, so `Menu::attach` declines to open a popup — correctly, per the empty-menu rule. Its two items are `format.properties` and `format.delete`: the first is gated on `selection.actionable`, which `app::conditions` sets for `doc.selected_field.is_some()` and which `canvas::menus::attach` must CORRECT for this frame through `MenuHost::with_conditions` — the frame's own snapshot predates the selection — and the second is `shown_when(selection.delete_permitted)`, corrected on the same frame from `formfield::document_refuses_delete`. A correction that forgot either produces exactly this. Regions beginning `menu.`: {}. Trace: {}.",
            menu.raw,
            list(&declared_names(&trace, ui_rect, "menu.")),
            session.trace_path().display()
        )));
    }
    if declared_names(&trace, ui_rect, &format!("menu.body.{FIELD_CONTEXT}")).is_empty() {
        return Ok(Some(format!(
            "the field menu declared {} row region(s) and no `menu.body.{FIELD_CONTEXT}` region, so rows were laid out and the popup frame that holds them was not. That is a reporting split rather than a menu defect — `MenuHost::attach_with` publishes both through one sink — and it is reported rather than ignored because every later check that wants to PRESS a row will aim inside the body rect. Rows seen: {}.",
            rows.len(),
            list(&rows)
        )));
    }
    report.note(format!(
        "★★★ the field menu opened with {} row(s): {}",
        rows.len(),
        list(&rows)
    ));

    // ★★ `canvas-menu-invoked` is REPORTED and never asserted, and the
    // distinction is the point: it appears only once a row has been activated,
    // so on this check — which opens the menu and presses nothing — it is
    // correctly absent. Recorded so a later check that does press a row has the
    // line named in one place.
    let invoked = trace
        .events(INVOKED_EVENT)
        .filter(|l| l.get("context") == Some(FIELD_CONTEXT))
        .count();
    report.note(format!(
        "{invoked} `{INVOKED_EVENT}` line(s) — this check activates no row, so zero is correct"
    ));

    // ★ Escape, so the popup is not left over the page for whatever runs next
    // in the sweep. `right_click_at` deliberately does not do this itself —
    // leaving it open is what lets a screenshot show one — so the caller says
    // so, here, where the reason is local.
    driver.press(crate::sys::vk::ESCAPE)?;
    session.settle(10);
    Ok(None)
}
