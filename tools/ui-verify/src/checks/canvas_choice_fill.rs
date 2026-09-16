//! `a_drop_down_can_be_answered_on_the_page` — **clicking a combo box or a
//! list box on the page opens its options and picking one writes the field** —
//! `FORMS_PARITY.md` §8.1 row 2.
//!
//! # What was wrong, and why the wrongness was argued rather than forgotten
//!
//! `canvas::forms` §5 used to carry a fifth reason a field is not offered on
//! the page: *"a choice field would need a dropdown anchored to the page,
//! which is a second popup surface with its own placement rules and no gesture
//! the panel does not already have."* Both clauses were wrong. The placement
//! rules are the ones the page-anchored context menu already has, and the
//! panel's gesture is **not** the same one: the panel is reached by finding a
//! row in a list of field names, and the operator reaching for a drop-down on
//! a drawing has the drop-down under the pointer already.
//!
//! So a drop-down was selectable on the page and not answerable there, and the
//! selectable census said so in the trace on every run without any check
//! reading it.
//!
//! # Why a unit test cannot close this row
//!
//! `canvas::forms::choosing` has unit tests over the whole pure half: which
//! exports a pick produces, which row the list opens on, and where the
//! highlight goes. What they cannot see is the **chain in front of the verb** —
//! that the click reaches the form overlay at all, that the popup is drawn
//! somewhere a press can land, that the press lands on the row the operator
//! aimed at rather than on the page behind it, and that the arrow key reaches
//! the list instead of scrolling the canvas. Every one of those is a fact about
//! a laid-out frame and a real pointer.
//!
//! The half-plane constraint the popup uses exists precisely because the naive
//! version fails here and nowhere else: a page-anchored popup handed
//! `constrain_to(screen)` slides back over its own anchor and then takes that
//! anchor's clicks, so the operator's pick re-opens the list instead of
//! answering it. A unit test of the pick arithmetic passes throughout.
//!
//! # The sequence, and what each step rules out
//!
//! | step | oracle | what a wrong build produces |
//! |---|---|---|
//! | click `ComboOne` | `form-choice-open row=1` | `row=0` — the list opens at the top rather than on the answer the document already holds |
//! | `ArrowDown` | **no** `form-choice-pick` | a pick — Windows changes a closed combo's value on an arrow press, and doing that here writes a document edit and an undo entry for a key pressed to *look* |
//! | `Enter` | `form-choice-pick row=2 selected=1 multi=false` | `row=1` — the arrow never reached the list; or no line at all — Enter did not answer |
//! | | `form-set-choice commands=1` | the command was built and the engine refused it, which is what a value outside `/Opt` produces |
//! | click `ListMulti`, three arrows, `Enter` | `selected=3 multi=true` | `selected=1` — the pick replaced the two values already in `/V` instead of adding to them |
//! | `Escape` | `left=false` | `left=true` — the list closed itself after the tick, so answering three options is three separate gestures |
//! | `Escape` | `left=true` | the ring is never given up and the field keeps the keyboard |
//!
//! The `selected=` field is a **count**, never a value: which option an
//! operator picked is their answer to the form, so the trace carries how many
//! and the row index and not the text. That is why the multi-select assertion
//! reads `selected=3` rather than naming a weekday.
//!
//! # Why this fixture and no other
//!
//! `fixtures/all-field-kinds.pdf` is the only document in the corpus with a
//! choice field at all — which is why *"a drop-down cannot be answered on the
//! canvas"* was never going to be caught by a driven check before it existed.
//! It needs two of them and they must differ in the right way: `ComboOne` is
//! single-select with a `/V` that is **not** the first option, and `ListMulti`
//! is `MultiSelect` with **two** values already in `/V`. A single-select field
//! cannot tell "added to the selection" from "replaced it", and a field
//! answered with its own first option cannot tell "opens on the answer" from
//! "opens at the top".
//!
//! # It sends real input, so it takes the screen
//!
//! No `PDFCER_DIAG_VIEWPORT`: an off-screen window is where its sibling
//! `option_arrows` runs, and it can, because it presses nothing. Four clicks
//! and six keystrokes need the window where the OS will deliver them.

use crate::checks::driving::repo_fixture;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;
use crate::trace::Trace;

/// The fillable census. Filtered on `kind=choice`, the token
/// `canvas::forms::kind_label` writes for a combo box or a list box.
const BOX_LINE: &str = "form-box";
/// A click opened the option list. Carries the row it opened on.
const OPEN: &str = "form-choice-open";
/// A row was picked. Carries the row index, the resulting count, and whether
/// the field is multi-select.
const PICK: &str = "form-choice-pick";
/// Escape was spent — on the list (`left=false`) or on the ring
/// (`left=true`).
const ESCAPE: &str = "form-choice-escape";
/// `panels::forms::edit::apply`'s line for `FormEdit::SetChoice`, which is the
/// only evidence that the engine took the command rather than refusing it.
const APPLIED: &str = "form-set-choice";
/// The open list's own state, de-duplicated by `trace_changed`. The only
/// oracle for a highlight that moved without writing.
const STATE: &str = "form-choice-state";

/// Nine fields, ten widgets, every widget carrying an appearance stream.
const FIXTURE: &str = "all-field-kinds.pdf";
/// Why no other document substitutes for it.
const METHOD: &str = "This check needs a single-select choice field whose stored value is NOT its \
                      first option, and a MultiSelect one with two values already stored. No \
                      other fixture in the corpus carries a choice field at all. See \
                      `fixtures/all-field-kinds.PROVENANCE.py`.";

/// The single-select combo. `/Opt [(Red)(Green)(Blue)]`, `/V (Green)`.
const SINGLE: &str = "ComboOne";
/// The row `SINGLE`'s list must open on — `Green`, the value the document
/// holds, and deliberately not index 0.
const SINGLE_OPEN_ROW: usize = 1;
/// The row one `ArrowDown` from there — `Blue`.
const SINGLE_PICK_ROW: usize = 2;

/// The multi-select list. `/Opt [(Mon)(Tue)(Wed)(Thu)(Fri)]`, `/V [(Mon)(Wed)]`.
const MULTI: &str = "ListMulti";
/// Three `ArrowDown` presses from `Mon` reach `Thu`, which is not ticked.
const MULTI_ARROWS: usize = 3;
/// `Mon`, `Wed` and the newly ticked `Thu`.
const MULTI_WANT_COUNT: usize = 3;

/// See the module documentation.
pub struct ADropDownCanBeAnsweredOnThePage;

impl Check for ADropDownCanBeAnsweredOnThePage {
    fn name(&self) -> &'static str {
        "a_drop_down_can_be_answered_on_the_page"
    }

    fn defect(&self) -> &'static str {
        "a combo box or list box on the page can be selected and not answered: clicking it does \
         nothing an operator can use, so the only way to set a choice field is to find its row \
         by name in the Forms panel. Invisible to a unit test of the pick arithmetic, which \
         never learns whether the click reached the overlay or the popup landed anywhere a \
         press could hit"
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

/// One `form-box` census line for a choice field, as a canvas-space centre.
struct ChoiceBox {
    page: usize,
    field: String,
    centre: (f64, f64),
}

/// Read the application's own census of where its choice boxes are.
///
/// The application's numbers and not the fixture's, for `tab_navigation`'s
/// reason: a check that derived the rect from the PDF would be asserting that
/// two independent derivations agree, and would report a disagreement between
/// them as a broken drop-down.
fn choice_boxes(trace: &Trace) -> Vec<ChoiceBox> {
    trace
        .events(BOX_LINE)
        .filter(|l| l.get("kind") == Some("choice"))
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
            Some(ChoiceBox {
                page,
                field,
                centre: (x + w / 2.0, y + h / 2.0),
            })
        })
        .collect()
}

/// Every choice field the census named, for a failure message.
fn named(boxes: &[ChoiceBox]) -> String {
    if boxes.is_empty() {
        return "none".to_owned();
    }
    boxes
        .iter()
        .map(|b| b.field.as_str())
        .collect::<Vec<_>>()
        .join(", ")
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
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is four clicks and six keystrokes into a \
             real window. Reported as SKIPPED rather than passed: a check that did not exercise \
             its subject has learned nothing about it.",
        ));
    }
    // Its own fixture, ignoring `--pdf`. The sweep's shared aim document has no
    // choice field, so honouring `--pdf` would make this check SKIP in every
    // sweep — the failure mode `sweep-full.sh`'s own header is written against.
    let pdf = repo_fixture(FIXTURE, METHOD)?;
    let vocab = &ctx.profile.vocab;
    let page: PageGeometry = crate::fixture::page_geometry(&pdf).ok_or_else(|| {
        Error::new(format!(
            "cannot read a page size from {}. The fixture is pinned, so this is a broken fixture \
             rather than a missing argument.",
            pdf.display()
        ))
    })?;

    // --- 1: launch ---------------------------------------------------------
    //
    // No mode click: `canvas::forms::overlay` is not mode-gated, so a field is
    // answerable in whichever mode the shell starts in. A mode segment click
    // this check does not need is one more way for it to fail with a message
    // about the wrong thing.
    let mut spec = LaunchSpec::new(&exe, ctx.out("canvas_choice_fill.trace.txt"));
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
        "launched {} as pid {} on {FIXTURE}",
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

    // --- 2: the census offers choice boxes at all --------------------------
    //
    // The row's own subject, stated as a precondition. Before row 2 the census
    // contained no `kind=choice` line by construction — `classify` refused
    // every choice field with `NotOffered` — so an empty set here is the defect
    // and not a property of the document, and it is reported as a FAIL.
    let boxes = choice_boxes(&trace);
    let Some(single) = boxes.iter().find(|b| b.field == SINGLE) else {
        return Ok(Some(format!(
            "★★★ NO CHOICE BOX ON THE PAGE. The fillable census named {} as `kind=choice`, and \
             {SINGLE} is not among them. If the list is empty, \
             `canvas::forms::boxes::classify` is still refusing every choice field with \
             `NotOffered` — the fifth reason `canvas::forms` §5 used to carry — and the field is \
             selectable and unanswerable, which is the whole of what this check exists to catch. \
             If the list is non-empty and {SINGLE} is missing, the fixture changed. Trace: {}.",
            named(&boxes),
            session.trace_path().display()
        )));
    };
    let Some(multi) = boxes.iter().find(|b| b.field == MULTI) else {
        return Err(Error::new(format!(
            "the census named {} as choice boxes and {MULTI} is not among them, so the \
             multi-select half cannot be measured. Reported as a SKIP: that is a property of the \
             document, not of the gesture. {METHOD}",
            named(&boxes)
        )));
    };
    report.note(format!(
        "{} choice box(es) on the page: {}",
        boxes.len(),
        named(&boxes)
    ));

    let mapping = CanvasMapping::from_trace(&trace, vocab, page, single.page)?;
    let frame = session.frame()?;
    let driver = Driver::new(session.window());
    // The census is canvas space; `doc_to_window` takes PDF space. The flip is
    // the mapping's own formula read backwards: `canvas_y = page_height -
    // doc_y`.
    let aim = |b: &ChoiceBox| {
        let point = mapping.doc_to_window(DocPoint::new(
            b.page,
            b.centre.0,
            page.height_pt - b.centre.1,
        ))?;
        Ok::<_, Error>(frame.to_screen(point))
    };

    // --- 3: the click opens the list, on the answered row ------------------
    driver.click_at(aim(single)?)?;
    session.settle(25);

    let trace = session.trace()?;
    let Some(opened) = trace
        .events(OPEN)
        .filter(|l| l.get("field") == Some(SINGLE))
        .last()
    else {
        return Ok(Some(format!(
            "clicking {SINGLE} opened nothing: no `{OPEN}` line for it. The box is in the \
             fillable census, so `classify` offers it and `canvas::forms::click` did not route \
             the press to `choosing::focus_choice` — check that its `BoxKind::Choice` arm is \
             still there, and that the click reached the form overlay at all rather than the \
             page's own selection. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("open: `{}`", opened.raw));
    match opened.get_usize("row") {
        Some(row) if row == SINGLE_OPEN_ROW => {}
        Some(row) => {
            return Ok(Some(format!(
                "{SINGLE}'s list opened on row {row} and its stored value is row \
                 {SINGLE_OPEN_ROW}. `/V` holds the second of three options, so row 0 means \
                 `first_selected` was not asked or its answer was discarded, and the operator \
                 arrows away from the top of the list rather than from their own answer. Trace: \
                 {}.",
                session.trace_path().display()
            )));
        }
        None => {
            return Ok(Some(format!(
                "the `{OPEN}` line carries no readable `row=`: `{}`. Trace: {}.",
                opened.raw,
                session.trace_path().display()
            )));
        }
    }

    // --- 4: an arrow moves the highlight and writes NOTHING ----------------
    //
    // The conservative half, and the one a hand test never notices: a build
    // that answered the form on an arrow press looks *more* responsive. What it
    // actually does is put a document edit and an undo entry behind a key the
    // operator pressed to read the options.
    driver.press(vk::ARROW_DOWN)?;
    session.settle(25);

    let trace = session.trace()?;
    if let Some(early) = trace.events(PICK).last() {
        return Ok(Some(format!(
            "an `ArrowDown` ANSWERED THE FORM: `{}`. Moving the highlight must not write. \
             Windows changes a closed combo's value on an arrow press and this build copied it, \
             so every keystroke spent looking at the options is a document edit and an undo \
             entry. `choosing::choose` must reach `chosen` only from Enter, Space or a click on \
             a row. Trace: {}.",
            early.raw,
            session.trace_path().display()
        )));
    }
    // The other half of the same press, and the half the absence above cannot
    // state: a build whose arrow key does nothing at all also writes nothing.
    // `form-choice-state` is the highlight itself, so this is the move measured
    // rather than inferred from what did not happen.
    let moved = trace
        .events(STATE)
        .filter(|l| l.get("field") == Some(SINGLE))
        .last();
    let Some(moved) = moved else {
        return Ok(Some(format!(
            "no `{STATE}` line for {SINGLE} at all, so the focused choice field drew no frame \
            after its list opened. `choosing::choose` is not running: either \
            `canvas::forms::editor` stopped routing `BoxKind::Choice` to it, or the stored focus \
            was dropped before it could. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if moved.get("open") != Some("true") {
        return Ok(Some(format!(
            "an `ArrowDown` CLOSED {SINGLE}'s list: `{}`. An arrow moves the highlight inside an \
            open list and nothing else; closing it on the way makes the next Enter answer a list \
            the operator can no longer see. Trace: {}.",
            moved.raw,
            session.trace_path().display()
        )));
    }
    match moved.get_usize("hl") {
        Some(hl) if hl == SINGLE_PICK_ROW => {}
        Some(hl) => {
            return Ok(Some(format!(
                "an `ArrowDown` left {SINGLE}'s highlight on row {hl}; it opened on row \
                {SINGLE_OPEN_ROW} and one press down is row {SINGLE_PICK_ROW}: `{}`. Either the \
                key never reached `choosing::arrow` — egui surrenders focus on a bare arrow \
                unless the field locks it, which is `canvas::forms::keyboard_box` — or `step` \
                wrapped or stalled. Trace: {}.",
                moved.raw,
                session.trace_path().display()
            )));
        }
        None => {
            return Ok(Some(format!(
                "the `{STATE}` line carries no readable `hl=`: `{}`. Trace: {}.",
                moved.raw,
                session.trace_path().display()
            )));
        }
    }
    report.note(format!(
        "an ArrowDown moved the highlight to row {SINGLE_PICK_ROW} and wrote nothing"
    ));

    // --- 5: Enter answers, on the row the arrow moved to -------------------
    driver.press(vk::ENTER)?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(pick) = trace
        .events(PICK)
        .filter(|l| l.get("field") == Some(SINGLE))
        .last()
    else {
        return Ok(Some(format!(
            "`Enter` on an open option list answered nothing: no `{PICK}` line for {SINGLE}. The \
             list opened, so the popup exists; either `Enter` never reached it — the canvas or \
             the ribbon took the press — or the row it resolved was out of range. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("pick: `{}`", pick.raw));
    match pick.get_usize("row") {
        Some(row) if row == SINGLE_PICK_ROW => {}
        Some(row) if row == SINGLE_OPEN_ROW => {
            return Ok(Some(format!(
                "`Enter` picked row {row}, the row the list OPENED on, so the `ArrowDown` before \
                 it never reached the list. The arrow went somewhere else — the canvas's own \
                 scroll, most likely — and the operator can see the options and not walk them. \
                 Trace: {}.",
                session.trace_path().display()
            )));
        }
        Some(row) => {
            return Ok(Some(format!(
                "`Enter` picked row {row}; one `ArrowDown` from row {SINGLE_OPEN_ROW} is row \
                 {SINGLE_PICK_ROW}. Trace: {}.",
                session.trace_path().display()
            )));
        }
        None => {
            return Ok(Some(format!(
                "the `{PICK}` line carries no readable `row=`: `{}`.",
                pick.raw
            )));
        }
    }
    if pick.get("selected") != Some("1") || pick.get("multi") != Some("false") {
        return Ok(Some(format!(
            "a single-select pick produced `{}`, and it must produce exactly one value with \
             `multi=false`. A count above one on a single-select field is the pick being treated \
             as an addition, and `set_choice_value` refuses that by name \
             (`ChoiceRequiresMultiSelect`) — so the operator's pick would be built and thrown \
             away. Trace: {}.",
            pick.raw,
            session.trace_path().display()
        )));
    }

    // --- 6: the engine took the command ------------------------------------
    //
    // Asked separately because a pick that is built and refused traces `PICK`
    // exactly as a pick that lands does. The refusal is the live failure mode
    // for choice fields: any value outside `/Opt` is rejected by name, so a
    // build that carried a stored-but-unlisted value into the command would
    // fail every pick on such a field while tracing a correct-looking pick.
    let Some(applied) = trace.events(APPLIED).last() else {
        return Ok(Some(format!(
            "the pick was traced and the engine never applied it: no `{APPLIED}` line. \
             `panels::forms::edit::apply` traces `{APPLIED}-refused` for a declined verb and \
             `{APPLIED} commands=0` for one that changed nothing — look for either in the \
             trace: {}.",
            session.trace_path().display()
        )));
    };
    if applied.get("commands") == Some("0") {
        return Ok(Some(format!(
            "the engine applied the pick and it changed nothing: `{}`. `/V` held row \
             {SINGLE_OPEN_ROW} and the pick was row {SINGLE_PICK_ROW}, so a zero-command apply \
             means the values handed to `set_choice_value` were the ones already stored — the \
             display half was sent, or the old selection was. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!("the engine took it: `{}`", applied.raw));

    // --- 7: the multi-select half ------------------------------------------
    //
    // A different field and not the same one twice: a single-select pick
    // REPLACES and a multi-select pick ADDS, and a build that replaced in both
    // cases satisfies every assertion above. `/V` already holds two values, so
    // the count after one tick is the whole oracle.
    driver.click_at(aim(multi)?)?;
    session.settle(25);

    let trace = session.trace()?;
    if !trace.events(OPEN).any(|l| l.get("field") == Some(MULTI)) {
        return Ok(Some(format!(
            "clicking {MULTI} opened nothing: no `{OPEN}` line for it, on a run where {SINGLE} \
             opened correctly. A list box and a combo box take the same gesture here — both are \
             `BoxKind::Choice` — so this is the click missing the box rather than the kind being \
             unhandled. Trace: {}.",
            session.trace_path().display()
        )));
    }
    for _ in 0..MULTI_ARROWS {
        driver.press(vk::ARROW_DOWN)?;
        session.settle(8);
    }
    driver.press(vk::ENTER)?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(tick) = trace
        .events(PICK)
        .filter(|l| l.get("field") == Some(MULTI))
        .last()
    else {
        return Ok(Some(format!(
            "`Enter` on {MULTI}'s open list answered nothing: no `{PICK}` line for it. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("tick: `{}`", tick.raw));
    if tick.get("multi") != Some("true") {
        return Ok(Some(format!(
            "{MULTI} was picked as `{}`, and it carries the MultiSelect flag. The kind is read \
             from `/Ff` in `classify`, so `multi=false` here means the flag is not being read, \
             and every tick on this field will be refused by the engine as \
             `ChoiceRequiresMultiSelect` the moment it names more than one value. Trace: {}.",
            tick.raw,
            session.trace_path().display()
        )));
    }
    if tick.get_usize("selected") != Some(MULTI_WANT_COUNT) {
        return Ok(Some(format!(
            "a tick on an unticked row of {MULTI} produced `{}`, and it must produce \
             {MULTI_WANT_COUNT} values: the two `/V` already holds plus the one just ticked. A \
             count of 1 is a multi-select pick that REPLACED the selection instead of adding to \
             it, which silently unticks answers the operator had already given. A count of 2 is \
             an arrow that did not move, so the tick landed on a row already selected and \
             removed it. Trace: {}.",
            tick.raw,
            session.trace_path().display()
        )));
    }

    // --- 8: the list survived the tick, and Escape gives it up -------------
    //
    // `left=false` is the assertion. A multi-select list that closes after each
    // tick makes choosing three options three separate gestures, and the only
    // outside evidence that it stayed open is that the first Escape was spent
    // on the LIST rather than on the ring.
    driver.press(vk::ESCAPE)?;
    session.settle(20);

    let trace = session.trace()?;
    let Some(first_escape) = trace.events(ESCAPE).last() else {
        return Ok(Some(format!(
            "`Escape` after a tick was spent on nothing: no `{ESCAPE}` line. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if first_escape.get("left") != Some("false") {
        return Ok(Some(format!(
            "the option list CLOSED ITSELF after a multi-select tick: the first `Escape` was \
             spent giving up the field (`{}`) rather than closing the list, which is only \
             possible if the list was already shut. Ticking three options then costs three \
             clicks into the box, and the operator cannot see which rows they have already \
             ticked while ticking the next. `choosing::choose` keeps the list open with \
             `state.open = *multi`. Trace: {}.",
            first_escape.raw,
            session.trace_path().display()
        )));
    }
    let before = trace.events(ESCAPE).count();
    driver.press(vk::ESCAPE)?;
    session.settle(20);

    let trace = session.trace()?;
    let Some(second_escape) = trace.events(ESCAPE).skip(before).last() else {
        return Ok(Some(format!(
            "a second `Escape` was spent on nothing: the field still holds the keyboard and \
             there is no way off it but a click elsewhere. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if second_escape.get("left") != Some("true") {
        return Ok(Some(format!(
            "a second `Escape` did not give up the field: `{}`. The two rungs are the same key \
             and must resolve innermost first — close the list, then leave the ring. Trace: {}.",
            second_escape.raw,
            session.trace_path().display()
        )));
    }

    report.note(format!(
        "★★ {SINGLE} opened on its own stored row, an arrow moved without writing, Enter wrote \
         row {SINGLE_PICK_ROW} and the engine took it; {MULTI} added a third value to the two it \
         already held and kept its list open"
    ));
    Ok(None)
}
