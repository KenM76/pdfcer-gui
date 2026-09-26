//! **The right-click route to one line inside a block of text** — O188(A),
//! driven end to end on the real binary.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/run_menu_route.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose canvas may select and edit page content.
const MODE: &str = "edit";

/// The fixture, relative to the workspace root. See the module header.
const FIXTURE: &str = "paragraph.pdf";

/// Page index of [`FIXTURE`] this check uses.
const PAGE: usize = 0;

/// Aim, in PDF user space (y up), inside the **third** line of the paragraph.
///
/// See the module header for why the third line and not the first.
const AIM: (f64, f64) = (120.0, 672.0);

/// The size of [`FIXTURE`]'s only page, from its `/MediaBox`.
const PAGE_SIZE: PageGeometry = PageGeometry {
    width_pt: 612.0,
    height_pt: 792.0,
};

/// How many visual LINES [`FIXTURE`]'s single text object holds.
const EXPECTED_RUNS: usize = 6;

/// `canvas-selection via=… sel=… level=… first=…`.
const SELECTION_EVENT: &str = "canvas-selection";

/// The rung the click in step 2 must land on, before the menu is opened.
const OBJECT_LEVEL: &str = "Object";

/// `canvas-menu context=… sel=… level=…` — written by `canvas::menus::attach`
/// on every frame carrying a secondary click.
const MENU_EVENT: &str = "canvas-menu";

/// The context a right-click on a selected text object must resolve.
const OBJECT_CONTEXT: &str = "canvas.object";

/// `text-run-menu pick=line:N/M offered=…` — `canvas::runmenu::trace`, written
/// once per right-click, on the one frame the pointer is still over the text.
const PICK_EVENT: &str = "text-run-menu";

/// `text-run-command pick=… outcome=raised|declined` —
/// `canvas::runmenu::resolve`, written once per press of the row.
const COMMAND_EVENT: &str = "text-run-command";

/// The trace the `!edit_content` arm of the dispatcher writes instead.
const MODE_DECLINE_EVENT: &str = "format-select-text-line-declined";

/// `selection-set page=… object=… part=… level=… via=…` —
/// `SelectionState::select_part`.
const SELECTION_SET_EVENT: &str = "selection-set";

/// The `via=` token this route, and only this route, writes.
const VIA_ROW: &str = "select-text-line";

/// The `level=` token `select_part` writes. Lower case, and deliberately not
/// the same spelling as [`OBJECT_LEVEL`]: that one is a `{:?}` of
/// `SelectionLevel` on the `canvas-selection` line, this one is a literal in
/// `select_part`'s own `format!`. Two spellings of one concept, on two
/// channels, and a harness that assumed they matched would be asserting a
/// coincidence.
const PART_LEVEL_TOKEN: &str = "part";

/// `status-rung kind=text|path part=N held=H of=M` — `app::status::selected`, the
/// clause the status bar appended. See the module header for why the rect is
/// not enough.
const RUNG_EVENT: &str = "status-rung";

/// The `kind=` token for a line of text, as opposed to a part of a shape.
const RUNG_TEXT: &str = "text";

/// The menu row itself, published through `MenuHost::attach_with`'s rect sink.
const ROW_REGION: &str = "menu.item.canvas.object.format.select_text_line";

/// Every row of the canvas object menu, for the failure message.
const ROW_PREFIX: &str = "menu.item.canvas.object.";

/// The status bar's selection readout — `app::status::selected::REGION`.
const SELECTED_REGION: &str = "status-group:selected";

/// Every group of the status bar, for the failure message in step 10.
const STATUS_PREFIX: &str = "status-group:";

/// See the module documentation.
pub struct TheRightClickOffersTheLineYouClicked;

impl Check for TheRightClickOffersTheLineYouClicked {
    fn name(&self) -> &'static str {
        "the_right_click_offers_the_line_you_clicked"
    }

    fn defect(&self) -> &'static str {
        "One line inside a block of text can be selected and deleted, and the only way to \
         reach it is to arm the Points tool with a chord BEFORE clicking — a route nothing in \
         the program mentions, so the operator finds it only after a gesture has already \
         failed (O188 (A))"
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

/// The line index and total off a `pick=line:N/M` field.
fn run_of(line: &crate::trace::TraceLine) -> Option<(usize, usize)> {
    let pick = line.get("pick")?;
    let rest = pick.strip_prefix("line:")?;
    let (n, of) = rest.split_once('/')?;
    Some((n.parse().ok()?, of.parse().ok()?))
}

/// Run the sequence.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check selects with a real click, opens a \
             real context menu with a real secondary click, and presses a row in it. Reported \
             as SKIPPED rather than passed: a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its mode segments or its menu rows are. Both are load-bearing here: this \
             check has to leave Read mode, and it has to press a row in a popup whose position \
             depends on where the pointer was.",
            ctx.profile.name
        ))
    })?;

    let pdf = crate::fixture::workspace_root()
        .join("fixtures")
        .join(FIXTURE);
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the fixture is not at {}. It is committed to this repository, so this is a \
             broken checkout rather than an unavailable precondition — reported as a failure \
             for that reason, because a SKIP would say the opposite.",
            pdf.display()
        )));
    }
    report.note(format!(
        "--pdf and --doc-point are IGNORED: pinned to {} at page {PAGE}, ({:.0}, {:.0}) — \
         inside the THIRD of {EXPECTED_RUNS} lines",
        pdf.display(),
        AIM.0,
        AIM.1
    ));

    // --- launch -------------------------------------------------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("run_menu_route.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    // The shell's channel too: `click_mode_segment` reads `egui-shell`'s own
    // trace, and without this the mode click looks like a miss.
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

    let trace = session.trace()?;
    if !trace.started(vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process \
             and this check has no oracle. Captured stderr is at {}.",
            vocab.start_event,
            session.trace_path().display()
        )));
    }

    // --- 1: Edit ------------------------------------------------------------
    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);
    report.note(
        "clicked the Edit mode segment first — the shell's default is Read, whose right-click \
         resolves a different, two-row menu (O71) that does not carry this command",
    );

    // --- 2: select the third line's text object, at the OBJECT rung --------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, PAGE_SIZE, PAGE)?;
    let window_point = mapping.doc_to_window(DocPoint::new(PAGE, AIM.0, AIM.1))?;
    let frame = session.frame()?;
    let target = frame.to_screen(window_point);
    report.note(format!(
        "canvas rect {:?} at zoom {:.3}; page {PAGE} ({:.0}, {:.0}) -> screen ({}, {})",
        mapping.image_rect,
        mapping.zoom,
        AIM.0,
        AIM.1,
        target.x(),
        target.y()
    ));
    driver.click_at(target)?;
    session.settle(15);

    let trace = session.trace()?;
    let Some(selection) = trace.last(SELECTION_EVENT) else {
        return Err(Error::new(format!(
            "the click at document point ({:.1}, {:.1}) on page {PAGE} produced no \
             `{SELECTION_EVENT}` line at all, so the harness has no oracle for what is under \
             the pointer and every step after this would be guesswork. Trace: {}.",
            AIM.0,
            AIM.1,
            session.trace_path().display()
        )));
    };
    if selection.get_usize("sel").unwrap_or(0) == 0 {
        return Err(Error::new(format!(
            "the click at document point ({:.1}, {:.1}) on page {PAGE} selected nothing, so \
             there is no text object to right-click. That is either an aim that is not on a \
             glyph or a broken hit test, and this harness cannot tell them apart — so it \
             declines to file either. The line seen was `{}`. Trace: {}.",
            AIM.0,
            AIM.1,
            selection.raw,
            session.trace_path().display()
        )));
    }
    let level = selection.get("level").unwrap_or("none");
    if level != OBJECT_LEVEL {
        return Err(Error::new(format!(
            "the click landed on the `{level}` rung and this check needs `{OBJECT_LEVEL}`. \
             That is the STARTING state under test, not an incidental precondition: this \
             whole feature exists so an operator holding the whole block can reach one line \
             of it, and a run that already stands where the row would put it cannot measure \
             whether the row put it there. An armed Points tool inherited from an earlier \
             gesture is the ordinary cause. SKIPPED rather than failed. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "the ladder is at the {OBJECT_LEVEL} rung — the whole block of text is held, which is \
         where the operator stands before he asks for one line of it"
    ));

    // --- 3: the control — the bar has said NOTHING about a rung -------------
    //
    // Without this, step 9 is vacuous. `status-rung` goes through
    // `diag::trace_changed`, which de-duplicates on the rendered line, so a
    // line already in the trace could not be told apart from one the press
    // produced.
    //
    // It is asserted rather than assumed because it is a real property of the
    // program: the clause is emitted only from the Part-rung arm of
    // `with_part`, and step 2 has just established that the ladder is at the
    // Object rung. A line here means the bar is disclosing a rung nobody is
    // standing on, which is its own defect and worth filing separately.
    let rung_before = trace.events(RUNG_EVENT).count();
    if rung_before != 0 {
        return Err(Error::new(format!(
            "the status bar has ALREADY disclosed a rung ({rung_before} `{RUNG_EVENT}` \
             line(s)) while the ladder is at the {OBJECT_LEVEL} rung, so its presence after \
             the press would prove nothing. `app::status::selected::with_part` emits that \
             line only from the Part-rung arm, so this should be impossible — a rung clause \
             on a whole-object selection is a disclosure defect in its own right. SKIPPED \
             rather than failed, because this check's subject is a different one and it can \
             no longer measure it. Last line: `{}`. Trace: {}.",
            trace.last(RUNG_EVENT).map_or("?", |l| l.raw.as_str()),
            session.trace_path().display()
        )));
    }
    report.note("control: the status bar has disclosed no rung before the press");

    // --- 4: right-click the same point -------------------------------------
    driver.right_click_at(target)?;
    session.settle(35);

    let trace = session.trace()?;
    let Some(menu) = trace.events(MENU_EVENT).last() else {
        return Ok(Some(format!(
            "THE RIGHT-CLICK RESOLVED NO MENU AT ALL: no `{MENU_EVENT}` line after a secondary \
             click on a selected text object. `canvas::menus::attach` writes that line on \
             every frame carrying a secondary click, so its absence means the click never \
             reached the canvas response. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let context = menu.get("context").unwrap_or_default();
    if context != OBJECT_CONTEXT {
        return Ok(Some(format!(
            "THE RIGHT-CLICK ON A SELECTED BLOCK OF TEXT RESOLVED `{context}`, NOT \
             `{OBJECT_CONTEXT}`: `{}`. The row this check is about lives on the object menu \
             and nowhere else, so no later step can run. Each alternative names a different \
             cause: `canvas.text` is the menu for a caret already open in text (so a previous \
             gesture left one there), `canvas.read_object` is Read's two-row menu (so step 1 \
             did not take), and `canvas.empty` is the view menu a miss resolves (so the \
             right-click hit test disagrees with the left-click one that just succeeded). \
             Trace: {}.",
            menu.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!("the right-click resolved `{}`", menu.raw));

    // --- 5: the pick was taken on that frame -------------------------------
    //
    // This is the frame-scoped half. `canvas::runmenu::pick_at` runs inside
    // the same `if response.secondary_clicked()` block, because this is the
    // only frame on which the pointer is still over the text — every later
    // frame of the popup's life has it on the menu.
    let Some(pick) = trace.events(PICK_EVENT).last() else {
        return Ok(Some(format!(
            "★★★ THE PICK WAS NEVER TAKEN: no `{PICK_EVENT}` line after the right-click, \
             although the object menu resolved. `canvas::menus` calls `runmenu::park` and \
             `runmenu::trace` UNCONDITIONALLY on every secondary click — the `else` arm parks \
             `Elsewhere` — so the absence of this line means the call site is gone, not that \
             the pointer was on paper. Without it the row has no operand, \
             `canvas.run_select_offered` is never published, and the row is dropped silently \
             with every unit test green. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let offered = pick.get("offered") == Some("true");
    let Some((n, of)) = run_of(pick) else {
        return Ok(Some(format!(
            "★★★ THE RIGHT-CLICK ON A LINE OF TEXT PICKED NOTHING: `{}`. The same point had \
             just selected this text object at the {OBJECT_LEVEL} rung, so the pointer IS on \
             the object; a pick of `elsewhere` therefore means the run-level hit test \
             disagrees with the object-level one, and there is no third reading. \
             `runmenu::pick_at` has three gates and each names a distinct cause: the object is \
             not `PartKind::TextLine`; it is a leaf painted from inside a form XObject (the Part \
             rung is unreachable there BY CONSTRUCTION, and this fixture has no forms); or \
             `part_hits_of` found no run under the point. ⚠ Falsify the aim before reading \
             this as a defect: {FIXTURE}'s third line has its baseline at 668 and this check \
             aims at y={:.0}. Trace: {}.",
            pick.raw,
            AIM.1,
            session.trace_path().display()
        )));
    };
    if !offered {
        return Ok(Some(format!(
            "the pick found `line:{n}/{of}` and the row was NOT offered: `{}`. \
             `RunPick::offered` is true for every `TextLine` variant, so a `line:` pick with \
             `offered=false` cannot arise from the current code and means the two have been \
             allowed to disagree. Trace: {}.",
            pick.raw,
            session.trace_path().display()
        )));
    }
    if of != EXPECTED_RUNS {
        return Ok(Some(format!(
            "★★★ THE RUN COUNT IS WRONG: the pick reports `of={of}` and {FIXTURE} holds \
             exactly {EXPECTED_RUNS} `Tj` operators in its one `BT`…`ET` block — baselines at \
             700, 684, 668, 652, 636 and 620. This number is not decorative: it is the \
             denominator the operator is shown in *1 line of {EXPECTED_RUNS}*, so a build that \
             counts wrongly states a wrong fact about his drawing while every other mechanism \
             in this chain still works. `of=1` in particular would also have suppressed the \
             row entirely, by `pick_at`'s second gate. Line: `{}`. Trace: {}.",
            pick.raw,
            session.trace_path().display()
        )));
    }
    if n == 0 {
        return Ok(Some(format!(
            "★★★ THE PICK IGNORED THE POINTER: it returned run 0 — the TOP line, baseline 700 \
             — for a click at y={:.0}, which is inside the third line (baseline 668). That is \
             what a `.first()` over the object's run list produces, and it is invisible on any \
             aim that happens to be on the first line, which is why this check deliberately \
             aims at the third. `part_hits_of` is the same query `canvas::input::probe` asks \
             to enter the Part rung, so if this is really wrong then the Points tool picks the \
             wrong line too. Line: `{}`. Trace: {}.",
            AIM.1,
            pick.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ the right-click picked `line:{n}/{of}` — the line under the pointer, not the first \
         one, and the count matches the fixture's {EXPECTED_RUNS} show operators"
    ));

    // --- 6: THE O188(A) DEFECT ITSELF — is there a route? --------------
    //
    //
    // ⇒ For this command that is the right coverage anyway, because R9 makes
    // the row ABSENT rather than greyed: the condition is not temporary — the
    // pointer either was on a line of a multi-line text object or it was not,
    // and no amount of waiting changes the answer.
    let Some(row) = driving::declared(&trace, ui_rect, ROW_REGION) else {
        return Ok(Some(format!(
            "★★★ THE DEFECT, AND IT IS O188(A) EXACTLY: there is NO ROW in the canvas object \
             menu for selecting the line that was clicked — no `{ROW_REGION}` region after the \
             menu opened, although the pick above found `line:{n}/{of}` on that very frame. \
             Rows the menu DID publish: {}.\n\
             The operator's report is the consequence: the delete is reachable only through \
             the Points tool (`A`, then click) — a chord he has to know BEFORE he clicks, \
             mentioned nowhere, so he finds it only after a gesture has already failed. **A \
             route he can find before failing is owed.**\n\
             Three readings, and all three are defects: the command is not registered at all \
             (R8 — an item naming an unregistered command is dropped before it is drawn, \
             silently and by design); it is registered but `shown_when` resolved false, i.e. \
             `MenuHost::with_conditions` did not publish `canvas.run_select_offered` even \
             though the pick was made, which is the parked-operand hand-off in \
             `canvas::menus`; or `MenuHost::attach_with` has stopped supplying a rect sink, in \
             which case no context-menu row anywhere in this application can be pressed by a \
             check and every other menu check is failing too. Trace: {}.",
            driving::list(&driving::declared_names(&trace, ui_rect, ROW_PREFIX)),
            session.trace_path().display()
        )));
    };
    if !row.is_substantial() {
        return Ok(Some(format!(
            "`{ROW_REGION}` was published at {row:?}, which has no usable area — so the row \
             exists in the plan and was laid out to nothing. A click aimed at a degenerate \
             rectangle proves nothing, and this is itself the finding."
        )));
    }
    report.note(format!(
        "★★★ the row IS in the context menu at {row:?} — the route O188(A) asked for, \
         reachable before any gesture has failed"
    ));

    // --- 7: press it, and follow the operand -------------------------------
    let mark = trace.mark();
    driver.click_at(session.frame()?.declared_center(row))?;
    session.settle(30);
    let after = session.trace()?;

    // 7 — did the parked operand survive the life of the popup?
    let Some(command) = after.last_after(COMMAND_EVENT, mark) else {
        return Ok(Some(format!(
            "THE ROW WAS PRESSED AND THE COMMAND NEVER RAN: no `{COMMAND_EVENT}` line after \
             the press. `canvas::runmenu::resolve` traces UNCONDITIONALLY, on both outcomes, \
             so its absence means the dispatcher was never entered rather than that the pick \
             had gone stale. Ask FIRST whether the press landed: grep the trace for the row's \
             `{}` lines — if the whole menu disappeared in the same frame as a \
             `canvas-pointer` line, the menu died on the pointer MOVE and no button ever went \
             down. That is what happened on 2026-09-12, and it sent the first hour of that \
             investigation to a function that was already correct. Otherwise the chain is \
             `catalog/format.rs` (registration and `enabled_when`) through `app/dispatch.rs` \
             (the `format::handles` funnel) to `dispatch/format.rs`'s arm — which also \
             declines silently when the mode cannot edit content, writing \
             `{MODE_DECLINE_EVENT}`; there are {} such line(s) in this trace. Trace: {}.",
            driving::UI_RECT_GONE_EVENT,
            after.events(MODE_DECLINE_EVENT).count(),
            session.trace_path().display()
        )));
    };
    let outcome = command.get("outcome").unwrap_or("?");
    if outcome != "raised" {
        return Ok(Some(format!(
            "★★★ THE PARKED OPERAND DID NOT SURVIVE THE POPUP: the command ran and declined — \
             `{}`. The pick was `line:{n}/{of}` when the menu opened, and `runmenu::resolve` \
             re-validates it against the CURRENT model at press time, raising nothing when it \
             no longer applies. Four things it re-asks, each a distinct cause: the parked \
             value is `Elsewhere` (the `egui::Memory` key expired with the popup, or nothing \
             parked it); the pick names a different page than the one on screen; the object is \
             no longer `PartKind::TextLine`; or the run index is now out of range. On a static \
             fixture with no edit between the two frames the FIRST is overwhelmingly the \
             likely one — and it is the exact failure this whole mechanism exists to prevent, \
             because it means the menu was right when it was drawn and meaningless when it was \
             pressed. Trace: {}.",
            command.raw,
            session.trace_path().display()
        )));
    }
    let Some((command_n, command_of)) = run_of(command) else {
        return Ok(Some(format!(
            "the command raised and its pick is unreadable: `{}`. Expected `pick=line:N/M`, \
             which `RunPick::word` writes as a stable token precisely so a harness never has \
             to read a `{{:?}}`. Trace: {}.",
            command.raw,
            session.trace_path().display()
        )));
    };
    if (command_n, command_of) != (n, of) {
        return Ok(Some(format!(
            "★★★ THE OPERAND DRIFTED BETWEEN THE MENU AND THE PRESS: the right-click picked \
             `line:{n}/{of}` and the press read back `line:{command_n}/{command_of}`. These are \
             the same parked value read twice and they cannot legitimately differ — nothing \
             edited the document between the two frames. A re-derivation at press time is the \
             cause this is written to catch: it works on a one-line document and picks the \
             wrong line on a thirty-six sheet title block, which is the operator's actual \
             file. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ the press read back the SAME pick the menu was drawn from: `{}`",
        command.raw
    ));

    // 8 — did the ladder actually enter the rung, on that run?
    let Some(set) = after
        .events(SELECTION_SET_EVENT)
        .filter(|l| l.lineno > mark)
        .find(|l| l.get("via") == Some(VIA_ROW))
    else {
        return Ok(Some(format!(
            "★★★ THE COMMAND RAISED AND THE SELECTION DID NOT MOVE: no `{SELECTION_SET_EVENT} \
             … via={VIA_ROW}` line after the press, although `{COMMAND_EVENT}` reported \
             `outcome=raised` — so `runmenu::resolve` answered `Some((object, run))` and \
             `SelectionState::select_part` was not called with it. That is a gap inside one \
             dispatch arm, in `dispatch/format.rs`, between the `if let Some(...)` and the \
             line under it. `{SELECTION_SET_EVENT}` lines that WERE written after the press: \
             {}. Trace: {}.",
            driving::list(
                &after
                    .events(SELECTION_SET_EVENT)
                    .filter(|l| l.lineno > mark)
                    .map(|l| l.raw.clone())
                    .collect::<Vec<_>>()
            ),
            session.trace_path().display()
        )));
    };
    let set_level = set.get("level").unwrap_or("?");
    let set_part = set.get_usize("part");
    if set_level != PART_LEVEL_TOKEN || set_part != Some(n) {
        return Ok(Some(format!(
            "★★★ THE ROW SELECTED THE WRONG THING: `{}`. Expected `part={n} \
             level={PART_LEVEL_TOKEN}` — the line the pointer was on when the menu opened. {} \
             This is the silent half of the feature: the row appears, the press works, the \
             operator watches a highlight land on a line he did not click, and nothing \
             anywhere reports an error. Trace: {}.",
            set.raw,
            if set_part == Some(0) {
                "The selected index is 0, the TOP line, which is what a hard-coded operand \
                 produces — see falsification recipe (3) in this file's header."
            } else {
                "The index does not match the pick, so the operand was re-derived somewhere \
                 between the press and the ladder."
            },
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ the ladder entered the Part rung on the line that was clicked: `{}`",
        set.raw
    ));

    // --- 9: the bar says WHICH line -----------------------------------------
    //
    // Read with `last`, not `last_after` — see step 3 and [`RUNG_EVENT`] for
    // why `trace_changed`'s de-duplication makes a mark-relative assertion the
    // wrong instrument here. Step 3 established the count was zero, so any
    // line at all is one this press produced.
    let Some(rung) = after.last(RUNG_EVENT) else {
        return Ok(Some(format!(
            "★★★ THE OPERATOR IS STANDING ON ONE LINE AND NOTHING SAYS SO: no `{RUNG_EVENT}` \
             line at all, although the ladder is at the Part rung on run {n} of {of}. The \
             status bar goes on naming the object's KIND and SIZE — both facts about the whole \
             block — while Delete would now remove one line of it, so every word on the bar is \
             true about something bigger than what the next keystroke will act on. \
             FIVE readings, and the LAST is the one no other step here can see. \
             `app::status::selected::with_part` returned early through one of its four \
             guards — the level is not `Part`; the first entry carries no `subpath`; \
             `page_object_index` is `None` (a leaf produces no clause BY DESIGN, the Part \
             rung being unreachable inside a form XObject, though this fixture has no \
             forms); or the index failed the bounds check against `part_count` — OR it \
             reached its `match` and the arm that builds the clause no longer builds one. \
             The trace is emitted from INSIDE that arm precisely so that those two are \
             ONE finding: it sat above the `match` for the first four hours it existed, \
             and this check passed on a build with both arms gutted. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let kind = rung.get("kind").unwrap_or("?");
    let rung_part = rung.get_usize("part");
    let rung_of = rung.get_usize("of");
    // HELD is the size of the Part-rung set. This route descends to ONE line,
    // so anything but 1 means the menu command left an earlier set standing and
    // the sentence is about a different operand from the one just chosen.
    let rung_held = rung.get_usize("held");
    if kind != RUNG_TEXT || rung_part != Some(n) || rung_of != Some(of) || rung_held != Some(1) {
        return Ok(Some(format!(
            "★★★ THE BAR DISCLOSED THE WRONG RUNG: `{}`. Expected `kind={RUNG_TEXT} part={n} \
             held=1 of={of}`. {} The clause the operator reads is built from exactly these \
             four numbers, so a wrong one here is a wrong sentence on his screen — and the two \
             wordings are not interchangeable: both rungs offer the same two verbs, drag and \
             Delete, but they name different things to do them to. This one must say *line* of \
             a *block of text*; the path rung says *part* of a *shape*. An operator told he is \
             holding a shape goes looking for corner handles a line of text has not got. \
             Trace: {}.",
            rung.raw,
            if kind == RUNG_TEXT && rung_held != Some(1) {
                "The kind and the index are right and `held` is not, so a Part-rung set built \
                 before this gesture survived a command whose whole meaning is to descend to \
                 the ONE line under the pointer."
            } else if kind == RUNG_TEXT {
                "The kind is right and a number is not, so the clause was computed from a \
                 different selection than the one that was made."
            } else {
                "A block of text was described as a shape, so `part_kind` disagrees with the \
                 pick that was just made from the same object."
            },
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ the status bar disclosed the rung: `{}` — the operator is told he holds one line \
         of {of}, and the hover behind it names both verbs that reach it, drag and Delete, \
         and how to get back",
        rung.raw
    ));

    // --- 10: and it was DRAWN. A sentence nobody paints is still silence. ----
    if driving::declared(&after, ui_rect, SELECTED_REGION).is_none() {
        return Ok(Some(format!(
            "the rung clause was computed and the status bar never drew it. `{}` is in the \
             trace, so `with_part` ran and returned a clause — and no `{SELECTED_REGION}` \
             region is on screen on any later frame, so the label carrying it was not painted. \
             The ordinary cause is the bar SHEDDING that group to fit a narrow window, which \
             `app::status::fitting` does by design and traces separately; on a default-sized \
             window it should not happen. Regions beginning `{STATUS_PREFIX}` that ARE \
             declared: {}. Trace: {}.",
            rung.raw,
            driving::list(&driving::declared_names(&after, ui_rect, STATUS_PREFIX)),
            session.trace_path().display()
        )));
    }
    report.note(
        "★★★ the whole chain held: right-click → pick → row on screen → press → the same run → \
         the Part rung → a sentence on the bar → painted. The route exists, and it is about \
         the line he clicked",
    );

    Ok(None)
}
