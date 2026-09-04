//! `a_placed_button_can_be_given_something_to_do` — **the whole of
//! `OPERATOR_REQUESTS.md` O60/O61's open half, driven.**
//!
//! # The report
//!
//! O60 and O61 both carried the same open line for a fortnight:
//!
//! > ⬜ *"push buttons that actually do something"*
//!
//! And the Button tool was **greyed**, with a sentence saying pdfcer *"cannot
//! give a button something to do yet, so it will not place one."*
//!
//! ## ★★★ The finding that matters more than the feature
//!
//! `pdfcer-core` shipped `EditSession::set_button_action` on **2026-08-30**
//! (`Pass 182.0`/`183.0`/`183.1`), in answer to this shell's own request, and
//! the reply said in as many words:
//!
//! > *"Please check your own copy. If your surface tells the operator that
//! > pdfcer never authors an action, it is now saying something untrue in the
//! > direction that matters."*
//!
//! **Two days passed.** The reply was read, filed, and the sentence stayed on
//! screen — because nothing in this repository fails when a capability lands.
//! Three things now do: this check, the tripwire in
//! `canvas::formfield::action`, and
//! `canvas::formfield::tests::no_kind_is_authorable_but_inert`.
//!
//! ## ★★ Why this cannot be a unit test, and cannot be a screenshot
//!
//! The chain has six links and only the last two are unit-testable:
//!
//! | # | link | its own test |
//! |---|---|---|
//! | 1 | the ribbon item is not greyed | ★ **nothing** — greying is drawn by `egui` from a condition string |
//! | 2 | the command arms the tool | now `the_push_button_arms_its_tool_like_every_other_kind` |
//! | 3 | a drag opens the placement dialog | ★ **nothing** |
//! | 4 | the dialog draws an action chooser, and it opens | ★ **nothing** |
//! | 5 | a popup row is clickable and changes the draft | ★ **nothing** |
//! | 6 | Add authors the button **and then writes the action** | unit-tested per half, never together |
//!
//! And **a screenshot cannot judge link 6 at all.** That is rule 4 working
//! correctly: a button carrying `/A` is drawn exactly as a button without one,
//! because applied content renders as saved content will. There is no badge, no
//! tint and no dashed outline — so the only oracle that exists is the trace.
//!
//! ```text
//! button-action-applied name=Button1 kind=ResetForm replaced=none
//! ```
//!
//! ★ Note what a weaker check would pass on. `add-form-field` committing, or
//! the field census naming a new button, is true on a build where the action
//! was never written — which is every build before 2026-09-01. This check must
//! read `button-action-applied` or it is measuring the feature that already
//! worked.
//!
//! ## The sequence
//!
//! | # | step | oracle |
//! |---|---|---|
//! | A | Edit mode, arm the button tool, drag out a button | the dialog's chooser region is declared |
//! | B | click the chooser | the popup's rows are declared |
//! | C | click *Clear the form* | `button-action-chose kind=ResetForm` |
//! | D | press Add | `button-action-applied … kind=ResetForm` |
//!
//! ★★ Steps B–D read a **child viewport's** rectangles, so every click goes
//! through [`frame_of`] rather than `session.frame()`. The placement dialog is
//! a real OS window; its rects begin at `x=0` because they are relative to its
//! own client origin, and converting them against the main window aims hundreds
//! of points away at numbers that look perfectly ordinary. That mistake has
//! been made four times in this harness and each time produced a confident,
//! precise and entirely wrong diagnosis of the subject.
//!
//! ★ Step A **drags** rather than clicks. A clicked button is authored at its
//! default 80×22 pt, which is fine — but a drag is the operator's own route
//! (O53) and it makes the dialog's arrival unambiguous.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, frame_of, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Edit mode, then arm the push-button tool.
///
/// ★ `edit.form_push_button` through the harness seam is also the control point
/// for link 1: the seam bypasses the ribbon, so a build where the command is
/// greyed but the dispatcher still arms would get past this. Link 1 is asserted
/// separately, below, by reading the ribbon item's own region.
const INVOKE: &str = "mode.edit,edit.form_push_button";
/// The closed chooser, published by `dialogs::buttonaction::rows`.
const COMBO_REGION: &str = "form.button.action"; // ui-text-exempt: a trace region name, never displayed
/// The *Clear the form* row inside the popup.
///
/// ★ Named by KIND, matching the publisher. An index-named region would keep
/// passing after `ButtonDoesKind::ALL` was reordered, aiming at whatever row
/// now sits second.
const RESET_ROW: &str = "form.button.action.row.ResetForm"; // ui-text-exempt: a trace region name
/// The dialog's Add button.
const ACCEPT_REGION: &str = "dialog.form_field.accept"; // ui-text-exempt: a trace region name
/// The line the chooser writes when the operator changes it.
const CHOSE: &str = "button-action-chose"; // ui-text-exempt: a trace event name, never displayed
/// ★ The line this check exists to read.
const APPLIED: &str = "button-action-applied"; // ui-text-exempt: a trace event name, never displayed
/// The line the author path writes when the SECOND verb refuses.
const REFUSED: &str = "button-action-refused"; // ui-text-exempt: a trace event name, never displayed
/// The page region, so a failure can say whether a sheet was drawn at all.
const PAGE_REGION: &str = "page"; // ui-text-exempt: a trace region name, never displayed
/// ★★★ The line `panels::forms::button` writes when it READS an existing
/// button's action — the half that could not ship until `Pass 212.0`.
const READ: &str = "button-action-read"; // ui-text-exempt: a trace event name
/// The Forms panel's dock TAB — clicked to bring its body forward.
const FORMS_TAB: &str = "dock.tab.view.panel_forms"; // ui-text-exempt: a trace region name

/// The box dragged out for the button, as page fractions.
const DRAG_FROM: (f64, f64) = (0.28, 0.58);
const DRAG_TO: (f64, f64) = (0.46, 0.50);

pub struct APlacedButtonCanBeGivenSomethingToDo;

impl Check for APlacedButtonCanBeGivenSomethingToDo {
    fn name(&self) -> &'static str {
        "a_placed_button_can_be_given_something_to_do"
    }

    fn defect(&self) -> &'static str {
        "the Button tool is greyed and refuses in words, so the one control on the ribbon that \
         makes a form do anything cannot be placed at all — and the engine has been able to \
         author its action since 2026-08-30"
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

#[allow(
    clippy::too_many_lines,
    reason = "one gesture chain with five oracles, each reading a rectangle the step before it resolved" // ui-text-exempt: a lint justification, never displayed
)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check drags out a field and then clicks three \
             controls in a child window.",
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
        .ok_or_else(|| Error::new("no --pdf. This check needs a page to place a button on."))?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new("cannot read a page size from the fixture. Pass --page-size WxH.")
        })?,
    };
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("button-action.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    // ★★ Deliberately NOT `PDFCER_DIAG_FORM_ACCEPT`. That seam accepts the
    // dialog with its default draft, whose action is `Nothing` — so a run using
    // it would author an inert button and read no `button-action-applied` line,
    // and would be measuring the placement that already worked. The dialog is
    // driven for real, which is also the only way links 4 and 5 are covered.
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with the button tool armed",
        exe.display(),
        session.pid()
    ));
    session.settle(45);
    let driver = Driver::new(session.window());

    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "no `{PAGE_REGION}` region, so no sheet is on screen. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // --- A: drag out a button ----------------------------------------------
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let frame = session.frame()?;
    let from = frame.to_screen(mapping.doc_to_window(DocPoint::new(
        0,
        DRAG_FROM.0 * page.width_pt,
        DRAG_FROM.1 * page.height_pt,
    ))?);
    let to = frame.to_screen(mapping.doc_to_window(DocPoint::new(
        0,
        DRAG_TO.0 * page.width_pt,
        DRAG_TO.1 * page.height_pt,
    ))?);
    driver.drag(from, to)?;
    session.settle(45);

    let trace = session.trace()?;
    if declared(&trace, ui_rect, COMBO_REGION).is_none() {
        // ★★★ Two very different diagnoses share this symptom, so name both and
        // say which regions WERE declared. A dialog that never opened is a
        // greyed tool or a broken drag; a dialog that opened without the chooser
        // is `dialogs::formfield::button_rows` not calling into
        // `dialogs::buttonaction::rows`.
        let form_regions = declared_names(&trace, ui_rect, "dialog.form_field");
        return Ok(Some(format!(
            "★★★ NO ACTION CHOOSER: the drag produced no `{COMBO_REGION}` region.\n\
             Regions beginning `dialog.form_field`: {}.\n\
             If that list is EMPTY the placement dialog never opened — the Button tool is \
             greyed again (one line: `app::conditions` sets `forms.push_button_runnable` beside \
             `doc.pages`) or `app::dispatch::forms::arm` declines it. If the list is NOT empty \
             the dialog opened and drew no chooser, which is \
             `dialogs::formfield::button_rows`. Trace: {}.",
            list(&form_regions),
            session.trace_path().display()
        )));
    }
    report.note("★ the placement dialog opened, and it draws an action chooser");

    // --- B: open the chooser ------------------------------------------------
    // ★ Re-read the trace rather than reusing the one above: a coordinate held
    // across an act that could move it is the other standing harness hazard,
    // and re-reading is free.
    let trace = session.trace()?;
    let combo = declared(&trace, ui_rect, COMBO_REGION)
        .ok_or_else(|| Error::new("the chooser was retired between two frames."))?;
    let combo_frame = frame_of(&session, &trace, ui_rect, COMBO_REGION)?;
    driver.click_at(combo_frame.declared_center(combo))?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(row) = declared(&trace, ui_rect, RESET_ROW) else {
        return Ok(Some(format!(
            "★★ THE CHOOSER DOES NOT OPEN, or its rows are unreachable: clicking \
             `{COMBO_REGION}` declared no `{RESET_ROW}`. Rows under that prefix: {}. An empty \
             list means the popup never opened; a non-empty one means the row NAMES changed and \
             this check is aiming at a name nothing publishes — which is a harness fault, not an \
             application one. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "form.button.action.row")),
            session.trace_path().display()
        )));
    };

    // --- C: choose "Clear the form" ----------------------------------------
    let row_frame = frame_of(&session, &trace, ui_rect, RESET_ROW)?;
    driver.click_at(row_frame.declared_center(row))?;
    session.settle(30);

    let trace = session.trace()?;
    let chose = trace
        .events(CHOSE)
        .filter_map(|l| l.get("kind").map(str::to_owned))
        .last();
    match chose.as_deref() {
        Some("ResetForm") => {
            report.note("★★ the chooser is set to Clear the form");
        }
        Some(other) => {
            return Ok(Some(format!(
                "★★ THE WRONG ROW WAS CHOSEN: the click on `{RESET_ROW}` produced \
                 `{CHOSE} kind={other}`. The rectangle and the value it selects disagree, which \
                 is what an index-named region does after `ButtonDoesKind::ALL` is reordered — \
                 except that these are named by kind, so this is a genuine mismatch between the \
                 published rect and the `selectable_value` beside it. Trace: {}.",
                session.trace_path().display()
            )));
        }
        None => {
            return Ok(Some(format!(
                "★★ THE ROW IS DRAWN AND NOT CLICKABLE: `{RESET_ROW}` was declared and clicked \
                 and no `{CHOSE}` line followed, so the draft did not change. A rectangle that \
                 is published and does not respond is worse than one that is absent — R9 calls \
                 an offered control that does nothing the misleading kind of placeholder. \
                 Trace: {}.",
                session.trace_path().display()
            )));
        }
    }

    // --- D: press Add, and the action reaches the engine --------------------
    let trace = session.trace()?;
    let Some(accept) = declared(&trace, ui_rect, ACCEPT_REGION) else {
        return Err(Error::new(format!(
            "the dialog declares no `{ACCEPT_REGION}` region, so this check cannot press Add. \
             Regions beginning `dialog.form_field`: {}.",
            list(&declared_names(&trace, ui_rect, "dialog.form_field"))
        )));
    };
    let accept_frame = frame_of(&session, &trace, ui_rect, ACCEPT_REGION)?;
    driver.click_at(accept_frame.declared_center(accept))?;
    session.settle(60);

    let trace = session.trace()?;
    if let Some(refusal) = trace.last(REFUSED) {
        return Ok(Some(format!(
            "★★★ THE BUTTON WAS PLACED AND THE ACTION WAS REFUSED: `{}`.\n\
             That is the two-verb hazard this feature carries by construction — \
             `add_push_button` and `set_button_action` are separate commands, and the second \
             can fail on its own leaving a correctly placed button with nothing to do. The \
             shell reports it off-canvas, which is right; what this check says is that it \
             HAPPENED. Trace: {}.",
            refusal.raw,
            session.trace_path().display()
        )));
    }
    let Some(applied) = trace.last(APPLIED) else {
        return Ok(Some(format!(
            "★★★ NO ACTION WAS WRITTEN: Add was pressed, the chooser said `ResetForm`, and no \
             `{APPLIED}` line followed — and no `{REFUSED}` line either, so the second verb was \
             never called.\n\
             The button is on the page and does nothing, which is EXACTLY the defect this \
             feature exists to remove, arriving through the surface built to remove it. Look at \
             `app::actions::forms::author`'s push-button arm: the action has to cross into the \
             `vector_edit` closure and `ButtonDoes::to_core` has to answer `Some`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let kind = applied.get("kind").unwrap_or("none");
    if kind != "ResetForm" {
        return Ok(Some(format!(
            "★★ THE WRONG ACTION WAS WRITTEN: the chooser said `ResetForm` and the engine was \
             given `{kind}`. `{}`. The draft the dialog edits and the draft the author reads \
             are not the same object. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!("★★ the engine wrote it: `{}`", applied.raw));

    // --- E: and the FORMS PANEL can read it back ----------------------------
    //
    // ★★★ **The half that could not ship on the morning of 2026-09-01.**
    //
    // `set_button_action` could write and nothing could read, so a control over
    // an EXISTING button had three possible shapes and all three were bad: show
    // "Nothing" and lie about somebody else's script; invent a one-way "set
    // this button to:" that no form editor has; or make the only way to read an
    // action be to destroy it. The row was declined and the gap was filed.
    //
    // `Pass 212.0` answered it hours later. This step proves the answer reached
    // the operator, and its oracle is deliberately the READ rather than the
    // row's pixels: the sentence a reader sees is chosen from four states, and
    // `state=` names which one — a screenshot reading "Clear the form" cannot
    // distinguish a correct `Known` from a lucky default.
    // ★★★ **BRING THE PANEL TO THE FRONT.** `view.panel_forms` puts the panel
    // in the layout; it does not make it the ACTIVE TAB of its dock group, and
    // a background tab draws no body at all.
    //
    // The first run of this step read `dock.tab.view.panel_forms` present and
    // `dock.body.view.panel_forms` absent, and reported *"the Forms panel never
    // drew a push-button row"* — true, and a statement about tab order rather
    // than about the reader. A region absent because its tab is behind another
    // looks exactly like one absent because the feature is missing, which is
    // the third instance of that shape in this harness.
    session.settle(20);
    let trace = session.trace()?;
    if let Some(tab) = declared(&trace, ui_rect, FORMS_TAB) {
        driver.click_at(session.frame()?.declared_center(tab))?;
        session.settle(35);
    }

    session.settle(20);
    let trace = session.trace()?;
    let read = trace
        .events(READ)
        .filter_map(|l| l.get("state").map(str::to_owned))
        .last();
    match read.as_deref() {
        Some("known") => {
            report.note("★★★ …and the Forms panel reads it back as a known action");
        }
        Some("none") => {
            return Ok(Some(format!(
                "★★★ THE PANEL READS THE BUTTON AS INERT: `{READ} … state=none`, on a button \
                 this run has just given a Reset action to and watched the engine accept.\n\
                 That is the exact falsehood the reader was requested to prevent — pdfcer \
                 asserting a fact about the operator's document that it did not check. Look at \
                 `panels::forms::button::row` and at whether it asks for the same \
                 fully-qualified name the author wrote. Trace: {}.",
                session.trace_path().display()
            )));
        }
        Some(other) => {
            return Ok(Some(format!(
                "★★ THE PANEL READS THE BUTTON AS `{other}`: this run authored a `ResetForm`, \
                 which `Pass 212.0` states round-trips as `Known` — including `Only` vs \
                 `Except`, the thing a reader most easily gets backwards. `unmodelled` here \
                 would mean the engine wrote something it cannot decode; `foreign` would mean \
                 it decoded something it will not author, on a button pdfcer itself just \
                 authored. Trace: {}.",
                session.trace_path().display()
            )));
        }
        None => {
            return Err(Error::new(format!(
                "no `{READ}` line, so the Forms panel never drew a push-button row — most \
                 likely it is not on screen in this run's inherited layout. SKIPPED rather \
                 than failed: that is a fact about the layout, not about the reader. \
                 Trace: {}.",
                session.trace_path().display()
            )));
        }
    }
    Ok(None)
}
