//! **The right-click route to one line inside a block of text** — O188(A),
//! driven end to end on the real binary.
//!
//! One check: `the_right_click_offers_the_line_you_clicked`.
//!
//! # The operator's report, and what it is actually about
//!
//! `OPERATOR_REQUESTS.md` O188's (A) row, left open after the refusal sentence
//! landed:
//!
//! > ⚠ **(A) is still open.** The delete is reachable only through the Points
//! > tool (`A`, then click); a double-click on text opens the caret instead,
//! > which is O70's ruling and correct. The new sentence tells him Delete
//! > works, but only after he has tried to drag. **A route he can find
//! > *before* failing is owed.**
//!
//! Deleting one line of a title block has worked since 2026-09-05. It could be
//! reached exactly one way: arm the Points tool with the `A` chord **before**
//! clicking, then click the line. Nothing in the program said so. Every route
//! an operator would try first — double-click, drag, the Objects panel's
//! outline — either does something else by design or does nothing at all.
//!
//! ⇒ The fix is a row in the canvas object menu: *Select this line of text*,
//! offered when the right-click landed on a line of a text object that has
//! more than one. This check is the assertion that the row exists, that it is
//! about the line the pointer was on, and that pressing it lands on that line.
//!
//! # ★★★ Why a unit test cannot make this claim, and what it would miss
//!
//! Every mechanism below has unit tests and they were all green while the
//! route did not exist. The reason is that the operand is **parked in
//! `egui::Memory` on the frame of the right-click and read back on the frame
//! of the press**, and there is no in-process test in this project that can
//! own two frames of a real popup:
//!
//! | mechanism | its unit test proves | what it cannot see |
//! |---|---|---|
//! | `canvas::runmenu::pick_at` | the three gates answer correctly for a given provider | that anything calls it on a right-click |
//! | `canvas::runmenu::park`/`parked` | a value survives a round trip through `Id`-keyed memory | that the key is still live by the time the row is pressed |
//! | `shell::menus` item table | the row is registered against `canvas.object` | that `shown_when` ever resolves true in a running frame |
//! | `dispatch::format` | the arm calls `select_part` when `resolve` answers | that `resolve` is reached at all |
//! | `SelectionState::select_part` | the entry list and level are set | that the operator can get there |
//!
//! ★★ The middle row is the one that bites. `MenuHost::with_conditions` sets
//! `canvas.run_select_offered` **per click**, from a pick taken on that click,
//! and a condition that is never published is simply absent — which reads as
//! *false*, which drops the row, **silently and with every test green**. That
//! is R8's mechanism working exactly as designed and it is indistinguishable
//! from the feature not shipping.
//!
//! # The chain, and why each link needs its own assertion
//!
//! The operand crosses four subsystems and three frames. A single end-state
//! assertion (*"the selection is at the Part rung"*) would be satisfied by a
//! build in which the menu row does nothing and the Points tool happened to be
//! armed, so the links are asserted **in order**, each with its own message,
//! and a build that breaks one fails at that one.
//!
//! | # | step | oracle | what a wrong build produces |
//! |---|---|---|---|
//! | 1 | Edit mode | `ribbon-mode-selected` | Read refuses a content click by design (DEFECTS.md D6) |
//! | 2 | click the third line | `canvas-selection … level=Object` | the aim is not on a glyph |
//! | 3 | **control**: the bar has said nothing about a rung | no [`RUNG_EVENT`] line yet | a fossil that would satisfy step 8 |
//! | 4 | right-click the same point | `canvas-menu context=canvas.object` | the view menu, i.e. the hit test missed |
//! | 5 | the pick was taken **on that frame** | `text-run-menu pick=run:N/6 offered=true` | `offered=false`, i.e. nothing to offer |
//! | 6 | **the row is on screen** | `menu.item.canvas.object.format.select_text_line` | O188(A), unfixed: no route |
//! | 7 | the press found the parked operand | `text-run-command pick=run:N/6 outcome=raised` | `outcome=declined`, i.e. the memory key died with the popup |
//! | 8 | the ladder entered the Part rung on **that** run | `selection-set … part=N level=part via=select-text-line` | a different N, i.e. the operand was re-derived and drifted |
//! | 9 | the bar says which line | `status-rung kind=text part=N of=6` | the rung is entered and nothing discloses it |
//! | 10 | the bar drew it | `ui-rect status-group:selected` | a sentence computed and never painted |
//!
//! ★★★ **Steps 8 and 9 carry the same `N` as step 5, and that is the real
//! subject of this file.** Each individual line could be produced by a build
//! that re-picks from scratch at press time — which would be wrong in exactly
//! the way that is hardest to see, because it works on a one-line document and
//! picks the wrong line on a thirty-six sheet title block. Asserting that the
//! index is *the same number all the way through* is what makes this a check
//! of the parked operand rather than three separate checks of three
//! mechanisms.
//!
//! # ★★ Why step 9 needs a trace line and could not use the rect
//!
//! `crate::diag::ui_rect` publishes **where** a label was drawn and never
//! **what it says**. A build that drew the readout and dropped the rung clause
//! publishes a byte-identical region, so an assertion on
//! `status-group:selected` alone is satisfied by both outcomes and measures
//! neither. `status-rung` was added to `app::status::selected` on 2026-09-15
//! for that reason and states the three facts the clause is computed from
//! rather than the English.
//!
//! ⚠ It is emitted through `diag::trace_changed`, which de-duplicates on the
//! rendered line. So step 9 is asserted as *"the count was zero before the
//! press and there is a line after it"* rather than as *"a line follows the
//! mark"* — an earlier gesture producing the identical clause would suppress
//! the later one and a mark-relative assertion would report a defect that is
//! not there. Step 3 is that control, and without it step 9 is vacuous.
//!
//! ★★★ **And WHERE that line is emitted from is part of the oracle, not an
//! implementation detail of the status bar.** It was first written just above
//! the `match` in `with_part`, keyed on the same `PartKind` the clause-building
//! arms are keyed on. That reads as equivalent and is not: falsification recipe
//! (4) below gutted both arms and this check still PASSED, because everything
//! the trace said remained true — it was a statement about the four guards
//! ABOVE the match, not about the sentence. ⇒ The emission was moved inside the
//! two producing arms on 2026-09-15, which is what makes step 9 a measurement
//! of the disclosure rather than of the code path that leads to it.
//!
//! # The fixture, pinned here and not read from `--pdf`
//!
//! `fixtures/paragraph.pdf` — 612 × 792, one `BT`…`ET` block holding **six**
//! `Tj` operators at 12 pt, baselines at 700, 684, 668, 652, 636 and 620, all
//! starting at x = 72. The same file `move_line_of_text` uses, for the same
//! measured reasons: one text object with several runs, legible at fit zoom.
//!
//! ★★★ **Pinned in code**, because the 2026-09-12 sweep hands every chunked
//! check one shared A1 sheet and one shared aim. `deeper_rung_delete` carried
//! a correct fixture table in *prose* for a week and all three of its rungs
//! still ran against a document their own header said they could not use.
//! Knowledge a check cannot run without belongs in the check.
//!
//! ★★ The aim is **(120, 672)** — inside the *third* line, whose baseline is
//! 668 — and the third and not the first on purpose. A build whose pick
//! ignores the pointer and returns run 0 is a real and tempting defect (it is
//! what a `.first()` over the run list does), and it is invisible when the aim
//! is on the top line. So this check asserts `N != 0` as well as `of == 6`.
//!
//! ⚠ A missing fixture is a **FAIL**, not a SKIP: it is committed to this
//! repository, so its absence is a broken checkout rather than an unavailable
//! precondition, and a SKIP would say the opposite.
//!
//! # ⚠ Falsification recipe — run this before believing a PASS
//!
//! A check nobody has seen fail is a claim, not a measurement. Each of these
//! must turn this check red, at the step named:
//!
//! 1. **Delete the row.** Remove the `Item::command("format.select_text_line")`
//!    line from `CANVAS_OBJECT` in `shell::menus`. → step 6 fails, and the
//!    message is the O188(A) report. *(This is the state the program was in
//!    before 2026-09-15, so this is also the recipe that proves the check
//!    would have caught the original defect.)*
//! 2. **Break the parking.** Make the `run_pick` binding in `canvas::menus`
//!    park `RunPick::Elsewhere` unconditionally. → step 5 fails, with the
//!    unreadable-pick message.
//! 3. **Break the hand-off.** Make `dispatch::format`'s arm ignore
//!    `runmenu::resolve`'s run index and call `select_part(page, object, 0, …)`.
//!    → step 8 fails on the INDEX, not on the presence of the line — which is
//!    the assertion this file exists for.
//! 4. **Drop the disclosure.** Replace both clause-building arms of the final
//!    `match` in `app::status::selected::with_part` with `(line, None)`.
//!    → step 9 fails while every other step stays green.
//!
//!    ⚠ **This recipe has already earned its keep, on the day it was
//!    written.** Run against the first version of the oracle it PASSED: the
//!    `status-rung` line was emitted from ABOVE that `match`, keyed on the same
//!    `PartKind` the arms are keyed on, so it went on being written by a build
//!    that disclosed nothing — and the check reported a working route on a
//!    program where the operator stands on one line of six and the status bar
//!    never says so. The emission now lives INSIDE the two arms. An oracle one
//!    statement away from the thing it claims to measure is an assertion both
//!    outcomes satisfy, and the only way to find one is to run the recipe.
//!
//! ★ (3) and (4) are the two that matter. (1) and (2) break loudly and would
//! be noticed by a person opening the menu; (3) and (4) are silent, and a
//! check that cannot distinguish them from a pass is not measuring the
//! feature.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose canvas may select and edit page content.
///
/// The shell's default is Read, where a canvas click on content is refused BY
/// DESIGN — and Read's right-click resolves `canvas.read_object`, a two-row
/// menu (O71) that does not carry this command at all. A check that skipped
/// this step would report the mode gate as a missing row.
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
///
/// Pinned with the fixture rather than read back, for the same reason: a
/// mapping derived from a page size this check did not choose would silently
/// aim somewhere else.
const PAGE_SIZE: PageGeometry = PageGeometry {
    width_pt: 612.0,
    height_pt: 792.0,
};

/// How many `Tj` operators [`FIXTURE`]'s single text object holds.
///
/// A hard fact about a 976-byte file committed to this repository, quoted in
/// the module header from its own content stream. It is asserted rather than
/// read back, because `of` is the denominator the operator is shown — *1 line
/// of 6* — and a build that counted wrongly would state a wrong number to him
/// while every mechanism in the chain still worked.
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

/// `text-run-menu pick=run:N/M offered=…` — `canvas::runmenu::trace`, written
/// once per right-click, on the one frame the pointer is still over the text.
const PICK_EVENT: &str = "text-run-menu";

/// `text-run-command pick=… outcome=raised|declined` —
/// `canvas::runmenu::resolve`, written once per press of the row.
const COMMAND_EVENT: &str = "text-run-command";

/// The trace the `!edit_content` arm of the dispatcher writes instead.
///
/// Named only in a failure message, as one of the readings of a press that
/// produced no [`COMMAND_EVENT`] line. A check that did not mention it would
/// send a reader hunting a dispatch gap when the mode had simply changed.
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

/// `status-rung kind=text|path part=N of=M` — `app::status::selected`, the
/// clause the status bar appended. See the module header for why the rect is
/// not enough.
const RUNG_EVENT: &str = "status-rung";

/// The `kind=` token for a line of text, as opposed to a part of a shape.
const RUNG_TEXT: &str = "text";

/// The menu row itself, published through `MenuHost::attach_with`'s rect sink.
///
/// ★ Publishing is the only possible answer for a popup: a context menu is
/// drawn **at the pointer** and `egui` may flip it to any of several
/// alignments to keep it on screen. There is no fraction of the window it can
/// be hard-coded to and no layout a harness could re-derive.
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

/// The run index and total off a `pick=run:N/M` field.
///
/// `None` for `pick=elsewhere` and for anything malformed. The caller
/// distinguishes those from the raw line rather than from this return: a parse
/// failure and an honest *"the pointer was not on a line"* are different
/// findings and must not share a message.
///
/// ★ Parsed rather than `Debug`-matched. A `{:?}` rendering of the pick would
/// make this harness depend on a Rust enum's formatting, which is the defect
/// recorded as *never `Debug`-format a field a machine reads* — a check there
/// reported the opposite of the truth while quoting the truth in its own
/// message. `RunPick::word` writes `run:N/M` as a deliberate, stable token for
/// exactly this reason.
fn run_of(line: &crate::trace::TraceLine) -> Option<(usize, usize)> {
    let pick = line.get("pick")?;
    let rest = pick.strip_prefix("run:")?;
    let (n, of) = rest.split_once('/')?;
    Some((n.parse().ok()?, of.parse().ok()?))
}

/// Run the sequence.
///
/// The three-way return is the SKIP/FAIL/PASS rule made structural: `Err` is a
/// precondition that was absent (SKIP), `Ok(Some(_))` is an assertion that did
/// not hold (FAIL), `Ok(None)` is a pass.
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

    // PINNED: `--pdf` and `--doc-point` are read and IGNORED here. See the
    // module header for what a prose fixture table cost on 2026-09-12.
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
    // ★★★ Without this, step 9 is vacuous. `status-rung` goes through
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
    // ★★ This is the frame-scoped half. `canvas::runmenu::pick_at` runs inside
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
             not `PartKind::Run`; it is a leaf painted from inside a form XObject (the Part \
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
            "the pick found `run:{n}/{of}` and the row was NOT offered: `{}`. \
             `RunPick::offered` is true for every `Run` variant, so a `run:` pick with \
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
        "★★ the right-click picked `run:{n}/{of}` — the line under the pointer, not the first \
         one, and the count matches the fixture's {EXPECTED_RUNS} show operators"
    ));

    // --- 6: ★★★ THE O188(A) DEFECT ITSELF — is there a route? --------------
    //
    // ★★ What a missing rect does and does NOT prove, taken from
    // `unshare_form`'s own correction of 2026-09-12: `menu/plan.rs` keeps a
    // DISABLED command as a slot and `menu/render.rs` reports its rect with no
    // reference to `enabled`. What `plan` drops is a command that is **not
    // registered**, or one whose `shown_when` resolved false. So this
    // assertion catches "no row" and does not catch "greyed row"; the greying
    // has no check, which is stated rather than quietly left out.
    //
    // ⇒ For this command that is the right coverage anyway, because R9 makes
    // the row ABSENT rather than greyed: the condition is not temporary — the
    // pointer either was on a line of a multi-line text object or it was not,
    // and no amount of waiting changes the answer.
    let Some(row) = driving::declared(&trace, ui_rect, ROW_REGION) else {
        return Ok(Some(format!(
            "★★★ THE DEFECT, AND IT IS O188(A) EXACTLY: there is NO ROW in the canvas object \
             menu for selecting the line that was clicked — no `{ROW_REGION}` region after the \
             menu opened, although the pick above found `run:{n}/{of}` on that very frame. \
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
             `{}`. The pick was `run:{n}/{of}` when the menu opened, and `runmenu::resolve` \
             re-validates it against the CURRENT model at press time, raising nothing when it \
             no longer applies. Four things it re-asks, each a distinct cause: the parked \
             value is `Elsewhere` (the `egui::Memory` key expired with the popup, or nothing \
             parked it); the pick names a different page than the one on screen; the object is \
             no longer `PartKind::Run`; or the run index is now out of range. On a static \
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
            "the command raised and its pick is unreadable: `{}`. Expected `pick=run:N/M`, \
             which `RunPick::word` writes as a stable token precisely so a harness never has \
             to read a `{{:?}}`. Trace: {}.",
            command.raw,
            session.trace_path().display()
        )));
    };
    if (command_n, command_of) != (n, of) {
        return Ok(Some(format!(
            "★★★ THE OPERAND DRIFTED BETWEEN THE MENU AND THE PRESS: the right-click picked \
             `run:{n}/{of}` and the press read back `run:{command_n}/{command_of}`. These are \
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
    if kind != RUNG_TEXT || rung_part != Some(n) || rung_of != Some(of) {
        return Ok(Some(format!(
            "★★★ THE BAR DISCLOSED THE WRONG RUNG: `{}`. Expected `kind={RUNG_TEXT} part={n} \
             of={of}`. {} The clause the operator reads is built from exactly these three \
             numbers, so a wrong one here is a wrong sentence on his screen — and the two \
             wordings are not interchangeable: the text one deliberately does NOT say *drag it \
             to move it*, because `pdfcer-core` has a verb that moves a subpath and none that \
             moves one show operator (request G017). Trace: {}.",
            rung.raw,
            if kind == RUNG_TEXT {
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
         of {of}, and the hover behind it says what Delete will do and how to get back",
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
