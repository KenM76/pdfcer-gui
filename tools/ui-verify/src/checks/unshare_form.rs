//! The two driven checks of *"give this page its own copy"* — that the command
//! has an operator route which is **not** the ribbon, and that it tells the
//! truth about the document in front of it.
//!
//! | check | fixture | asserts |
//! |---|---|---|
//! | `the_context_menu_gives_this_page_its_own_copy_of_a_shared_form` | `shared-across-two-pages.pdf` | the row is in the menu, the press copies, and the measurement found the other page |
//! | `the_unshare_declines_when_nothing_else_draws_the_form` | `page-sized-form.pdf` | the press **changes nothing** and says so |
//!
//! # ★★★ Why there are two, and why the first one was not enough
//!
//! Until 2026-08-29 there was one check, it was named
//! `…_of_a_shared_form`, its pass note read *"Every other invocation site keeps
//! naming {original} and is byte-identical"* — and it was pinned to
//! `page-sized-form.pdf`, **a document with exactly one invocation.** There
//! were no other invocation sites. The check asserted a sentence about a
//! population of zero and passed.
//!
//! That is not a cosmetic error in a note. It is the same defect the *feature*
//! had, reproduced in the instrument that was supposed to catch it: the command
//! shipped asserting *"This drawing is drawn on other pages too"* on a document
//! it had never counted, and the check that drove it shipped asserting the same
//! thing about the same absent pages. **A check whose fixture cannot exhibit
//! the condition under test measures nothing**, and this one had a name
//! promising it did.
//!
//! ⇒ So the shared case now uses a fixture that is actually shared, and the
//! unshared case — which is now a **real behaviour**, a worded decline that
//! changes nothing — gets a check of its own rather than being the accident
//! that the shared check happened to be running.
//!
//! # What this is for
//!
//! `EDITABLE_SURFACES.md`, written 2026-08-28, is an audit keyed on the
//! **engine's** verb list rather than on this shell's feature list. It found
//! `unshare_form` implemented in `pdfcer-core` and named nowhere in
//! `crates/pdfcer-gui/src`, and `pdfcer-core` asked for it by name:
//!
//! > *"So you can offer the button. Please un-suppress it rather than leaving
//! > the suppression in place — a control withheld on the strength of a note
//! > that has since been withdrawn is exactly the kind of thing that stays
//! > withheld for months."*
//!
//! ## ★★★ Why the audit's own instrument cannot answer this, which is the
//! ## whole reason this file exists
//!
//! `tools/verb-coverage.py` greps this crate for each engine verb's name.
//! `EDITABLE_SURFACES.md` states the limit of that measurement in as many
//! words:
//!
//! > **A hit means the NAME appears**, not that a reachable operator route
//! > calls it. A call site behind a condition nothing sets is a hit here and
//! > dead in the running program. Only `tools/ui-verify` answers that question.
//!
//! ⇒ So the audit that found this gap **would report it fixed the moment the
//! identifier appeared in a source file**, whether or not a single click could
//! reach it. This check is the other half: the verb is called because a person
//! pressed something.
//!
//! ## ★★★ The route under test is the CONTEXT MENU, deliberately
//!
//! `OPERATOR_REQUESTS.md` **O53** rules that a command must not exist only on
//! the ribbon. That rule is doing more work for this command than for most, and
//! the reason is about *when* the operator needs it rather than about
//! consistency:
//!
//! An operator who needs to unshare is, by construction, **mid-gesture**. They
//! have clicked inside a title block, they are about to type into it, and the
//! moment the choice is worth anything is *before* that keystroke — because
//! afterwards the edit is already in the one shared stream and every sheet has
//! it. The Format contextual tab is the correct second home. The pointer is the
//! first.
//!
//! ★★ And the ribbon route is the one a check could pass on while the useful
//! one was broken: a Format-tab click proves a band item dispatches, which
//! `font_group` and its neighbours already prove for that tab. Nothing before
//! this file had ever **pressed a context-menu row** — see the harness gap
//! below, which is the finding this check turned up.
//!
//! ## ★★★ The harness gap this check found, and had to close
//!
//! `right_clicking_a_form_field_opens_its_menu` was the first driven context
//! menu in this project's history, on 2026-08-28. It asserts that the right
//! menu **resolved** and that it **offered something**, and it stops there.
//!
//! It stopped there because it had to. `shell::menus::MenuHost::attach_with`
//! called `egui_shell::menu::Menu::attach` — the convenience constructor that
//! takes *no optional capabilities at all* — so pdfcer's context menus drew rows
//! and published **no `ui_rect` for any of them**. There was no coordinate to
//! aim at, so no check could press a row, so the entire "does the menu row
//! actually do the thing" question was unaskable.
//!
//! ⇒ That is the same shape `field_menu`'s own header records one layer up:
//! *"a gesture with no driver is a gesture R1 cannot reach, and the gap left no
//! failing test behind to advertise itself."* There the driver was missing;
//! here the **target** was. Both are invisible to a green suite.
//!
//! `MenuHost::attach_with` now supplies a rect sink, so every row of every
//! pdfcer context menu publishes `menu.item.<context>.<command id>` through the
//! same `crate::diag::ui_rect` channel the ribbon and the status bar use. This
//! check is the first consumer; every future menu check inherits it.
//!
//! ★ Publishing is the only possible answer for a popup, and
//! `egui_shell::menu::report`'s header says why: a context menu is drawn **at
//! the pointer**, and `egui` may flip it to any of several alignments to keep
//! it on screen. There is no fraction of the window it can be hard-coded to and
//! no layout a harness could re-derive.
//!
//! ## The oracle: `unshare-form-applied`, and what a wrong build gets wrong
//!
//! `app::actions::xobject` traces
//! `unshare-form-applied page=… original=… copy=… moved=…` on the success path.
//! Three of those four fields are load-bearing here:
//!
//! | field | what a wrong build reports |
//! |---|---|
//! | `original=` | the **innermost** enclosing form instead of the outermost — the operand `EditError::FormNestedInAnotherForm` exists to refuse. On this flat fixture the two coincide, which is why the check also asserts the number against the page's own object list rather than merely against itself |
//! | `copy=` | the same number as `original`, i.e. nothing was allocated |
//! | `moved=` | `0`, i.e. the page's `/XObject` names were not re-pointed and the copy is an orphan |
//!
//! ★★ **The absence of the line is the interesting failure**, not its content.
//! A build where the menu row is greyed, where the dispatcher has no arm, where
//! `containing_form_object` returns the innermost form, or where the engine
//! refuses, all produce **no line at all** — and each of those is a state in
//! which the operator presses a row and the page looks exactly as it did. That
//! is why this check exists and a unit test would not do: on this command,
//! *"nothing visibly happened"* is what **success** looks like too.
//!
//! ## Why these checks pin their own fixtures and ignore `--pdf`
//!
//! `form_selection`'s reason, unchanged and for the same subject: a check whose
//! subject is *"what happens to a form XObject"* cannot take an arbitrary
//! document. On a drawing with no forms — the operator's own SolidWorks export
//! has **zero** — the honest answer is *"there was nothing to unshare"*, which
//! is neither a pass nor a defect.
//!
//! ★★★ And here the fixture is not merely *a* document with a form: **it is the
//! condition under test.** Sharedness is a property of the file and of nothing
//! else, so each of these two checks is defined by which file it opens, and
//! swapping them would swap what each one proves without changing a line of
//! assertion code. Both are read from the read-only corpus at `D:\Dev\pdfcer`.
//!
//! ### `shared-across-two-pages.pdf` — the shared case
//!
//! Two 200 × 200 pt pages, both with `/Resources << /XObject << /Fm0 6 0 R >>
//! >>`, both drawing object 6 at `1 0 0 1 20 20 cm` — a 40 × 40 blue square
//! from (20, 20) to (60, 60). One form, **two invocation sites, on two distinct
//! pages**, which is the smallest honest model of the operator's thirty-six
//! sheet title block.
//!
//! ⇒ The numbers it makes assertable: `places=2`, `pages=2`, **`other=1`**,
//! `moved=1`. A build that had not counted could not produce `other=1`, and a
//! build that counted the wrong thing produces `other=2` — the off-by-one that
//! forgets to subtract the page being unshared, which is invisible on any file
//! with several sheets and wrong on every file with one.
//!
//! ### `page-sized-form.pdf` — the unshared case
//!
//! One 200 × 200 pt page whose only page object is a page-sized form holding
//! three 40 × 40 squares. It is invoked **once**.
//!
//! ★★★ This file used to be the shared check's fixture, and the comment that
//! justified it read: *"It is invoked once, not thirty-six times, and that is
//! fine — `unshare_form` does not require a form to be shared, and refusing to
//! privatise a singly-invoked form would be a rule nobody wrote."* Both
//! sentences were true about the **engine**, which is a verb and does what it
//! is told. Neither was a defence of the **shell**, which had told the operator
//! their drawing was on other pages and dirtied their document to give them a
//! byte-identical clone of it. The rule nobody had written is now written:
//! `crate::…::UnshareRefusal::NotShared`, and this file is the fixture that
//! proves it fires.
//!
//! ## The sequence
//!
//! Steps 1–4 are identical for both checks and live in [`open_and_press`]; only
//! the fixture, the aim point and step 5 differ.
//!
//! | # | step | oracle |
//! |---|---|---|
//! | 1 | open the fixture, click the Edit mode segment | `ribbon-mode` |
//! | 2 | click the centre of a square drawn by the form | `canvas-selection first=leaf:` |
//! | 3 | right-click the same point | `canvas-menu context=canvas.object` |
//! | 4 | the unshare row is on screen and clickable | `menu.item.canvas.object.format.unshare_form` |
//! | 5a | shared: it copies, having measured the fan-out | `unshare-form-measured other=1` then `unshare-form-applied` |
//! | 5b | unshared: it declines, and nothing is edited | `unshare-form-measured other=0` then `unshare-form-declined`, and **no** `unshare-form` funnel line |
//!
//! Step 4 is O53's assertion and step 5 is the audit's. Neither substitutes for
//! the other: a row that is drawn and does nothing passes 4, and a command
//! reachable only from the ribbon would pass 5 if this check pressed a band
//! item instead.
//!
//! ## ★★★ Why the decline needs a POSITIVE oracle, and gets one
//!
//! The obvious way to check a decline is to assert that
//! `unshare-form-applied` did not appear. It is worthless, and it is worthless
//! for the reason this file's own header already gives about the *success*
//! case, read backwards: **every possible breakage produces that same
//! absence.** A greyed row, a dispatcher with no arm, a menu that never opened,
//! a click that missed the square, a build with the feature deleted — all of
//! them are "no applied line", and all of them would pass a check written that
//! way. It would be a check that could only ever succeed.
//!
//! ⇒ `app::actions::xobject::fanout` therefore writes **two** lines the moment
//! the walk runs: `unshare-form-measured` on both paths, carrying the numbers
//! the decision was made from, and `unshare-form-declined … reason=not-shared`
//! on the declining path only. The unshared check asserts both are present,
//! that `other=0`, **and** that the `unshare-form` funnel line — which
//! `vector_edit` writes once per committed edit — is absent. Together those
//! say: the press arrived, the document was measured, the verb chose not to
//! act, and nothing was written. No one of them says it alone.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, ScreenPoint, WindowFrame};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// **The shared fixture: one form, two pages, one invocation each.**
///
/// See the module header. This is the file that makes "shared" a fact rather
/// than a claim — object 6 is drawn from page 0 *and* from page 1, so
/// unsharing page 0 leaves exactly one other page on the original and the
/// disclosure has a real number to state.
const SHARED_FIXTURE: &str = "forms-xobject/shared-across-two-pages.pdf";

/// **The unshared fixture: one form, one page, one invocation.**
///
/// The ordinary shape of a CAD sheet wrapped in a single form, and — until
/// 2026-08-29 — a document the shell would happily clone an object into while
/// telling the operator that every *other* page still shared it. There are no
/// other pages. This is the file the decline is proved on.
const UNSHARED_FIXTURE: &str = "forms-xobject/page-sized-form.pdf";

/// Both fixtures' page box, in PDF points.
///
/// Stated rather than read, for `form_selection`'s reason: each file is a
/// handful of objects of hand-written syntax, and a page size that changed
/// would change what every constant below means. They agree at 200 × 200,
/// which is why one constant serves both.
const PAGE: PageGeometry = PageGeometry {
    width_pt: 200.0,
    height_pt: 200.0,
};

/// The centre of the shared fixture's square (PDF user space).
///
/// Its content stream is `q 1 0 0 1 20 20 cm /Fm0 Do Q` over a form whose
/// `/BBox` is `[0 0 40 40]`, so the square spans (20, 20) → (60, 60) and its
/// centre is (40, 40). Forty points from two page edges at a zoom that fits a
/// 200 pt page to a full-size canvas is a comfortable margin — and the point
/// is well inside the page box, which is what `CanvasMapping::doc_to_window`
/// refuses to clamp for.
const ON_THE_SHARED_SQUARE: (f64, f64) = (40.0, 40.0);

/// The centre of the unshared fixture's middle square (PDF user space,
/// 80,80 → 120,120).
///
/// The **middle** one, exactly as `form_selection` aims: it is furthest from
/// every page edge, so a small error in the coordinate hop lands on paper
/// rather than off-window — and a failure then reads "selected nothing" rather
/// than "the click went outside the client area", which are different
/// diagnoses.
///
/// ★ It matters twice here rather than once, because the same point is
/// right-clicked. A popup opened near an edge is repositioned by `egui`, which
/// is exactly the case the published rect exists to survive — but a check
/// should not be *testing* that incidentally while trying to test something
/// else.
const ON_A_SQUARE: (f64, f64) = (100.0, 100.0);

/// `canvas-selection … first=object:N|leaf:N|none` — what a click selected.
const SELECTION_EVENT: &str = "canvas-selection";
/// The field naming which index space the selection landed in.
const FIRST_FIELD: &str = "first";
/// `canvas-menu context=…` — which menu a right-click resolved.
const MENU_EVENT: &str = "canvas-menu";
/// The context a selected page object must resolve to.
const OBJECT_CONTEXT: &str = "canvas.object";
/// The published rect of the row both checks press.
const ROW_REGION: &str = "menu.item.canvas.object.format.unshare_form";
/// The prefix every context-menu row publishes under.
const ROW_PREFIX: &str = "menu.item.canvas.object.";
/// `unshare-form-applied page=… original=… copy=… moved=…` — the success line.
const APPLIED: &str = "unshare-form-applied";
/// `unshare-form-measured page=… form=… places=… pages=… other=… lower_bound=…`
/// — the document walk's verdict, written on **both** paths.
///
/// ★★ This is the line that did not exist before 2026-08-29, and its absence
/// is the whole defect in one fact: nothing in the command's chain had ever
/// asked how many times the form was invoked, so there was no number anywhere
/// to trace.
const MEASURED: &str = "unshare-form-measured";
/// `unshare-form-declined page=… form=… reason=… places=…` — the decline's own
/// positive oracle. See the module header for why the absence of [`APPLIED`]
/// cannot serve in its place.
const DECLINED: &str = "unshare-form-declined";
/// The `vector_edit` funnel label, written **once per committed edit** as
/// `unshare-form page=… n=… epoch=… disclosures=…`.
///
/// The unshared check asserts this line is ABSENT, which is its proof that the
/// document was not touched: no `edit_epoch` bump, no undo entry, no dirty
/// flag. `Trace::events` matches the whole first token, so this never collides
/// with the three suffixed names above — the property
/// `tools/gates/check-trace-names.py` exists to hold.
const FUNNEL: &str = "unshare-form";
/// `other=…` on [`MEASURED`] — how many pages OTHER than this one draw the
/// form. The number the decision is made from and the number the disclosure
/// states.
const OTHER_FIELD: &str = "other";

/// What one check needs to know about the document it opens.
///
/// ★ Its existence is the point made in the module header: these two checks
/// differ in **which file they open**, and almost nowhere else. Bundling the
/// three facts that vary keeps [`open_and_press`] identical for both, so a
/// change to the gesture sequence cannot be made for one case and forgotten
/// for the other.
struct Scenario {
    /// Path under `D:/Dev/pdfcer/fixtures/synthetic`.
    fixture: &'static str,
    /// Page-space point to click and right-click. Must land on something the
    /// form draws, because the operand is a leaf.
    aim_at: (f64, f64),
    /// Basename for this check's captured trace, so two checks in one run do
    /// not overwrite each other's artifact.
    trace_name: &'static str,
}

/// The shared fixture's scenario.
const SHARED: Scenario = Scenario {
    fixture: SHARED_FIXTURE,
    aim_at: ON_THE_SHARED_SQUARE,
    trace_name: "unshare_form_shared.trace.txt",
};

/// The unshared fixture's scenario.
const UNSHARED: Scenario = Scenario {
    fixture: UNSHARED_FIXTURE,
    aim_at: ON_A_SQUARE,
    trace_name: "unshare_form_unshared.trace.txt",
};

/// See the module documentation.
pub struct TheContextMenuGivesThisPageItsOwnCopyOfASharedForm;

impl Check for TheContextMenuGivesThisPageItsOwnCopyOfASharedForm {
    fn name(&self) -> &'static str {
        "the_context_menu_gives_this_page_its_own_copy_of_a_shared_form"
    }

    fn defect(&self) -> &'static str {
        "`EditSession::unshare_form` has no operator route, so a title block invoked from \
         thirty-six sheets can be edited in place and cannot be privatised first — the operator \
         has decision 076's default and no option at all, which is the state R206 exists to \
         prevent"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_shared(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// See the module documentation.
pub struct TheUnshareDeclinesWhenNothingElseDrawsTheForm;

impl Check for TheUnshareDeclinesWhenNothingElseDrawsTheForm {
    fn name(&self) -> &'static str {
        "the_unshare_declines_when_nothing_else_draws_the_form"
    }

    fn defect(&self) -> &'static str {
        "\"Give this page its own copy\" succeeds on a form nothing else draws — allocating an \
         object, privatising /Resources, committing an undo entry and dirtying a clean document \
         to produce a byte-identical clone — and then tells the operator that every other page \
         still shares the original, about a file that has no other page"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_unshared(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Resolve a fixture under the engine repository's synthetic corpus.
///
/// ★ The path is derived, not configured, and `None` rather than a panic —
/// `form_selection`'s helper verbatim in shape, for the reason its own docs
/// give: `D:\Dev\pdfcer` is READ-ONLY to this project, and a missing corpus is a
/// SKIP with a reason rather than a crash mid-suite.
fn engine_fixture(rel: &str) -> Option<std::path::PathBuf> {
    let path = std::path::Path::new("D:/Dev/pdfcer/fixtures/synthetic").join(rel);
    path.is_file().then_some(path)
}

/// What survives [`open_and_press`] when the gesture sequence completed.
///
/// The `Session` travels because the process must stay alive for the caller to
/// read the trace it wrote — dropping it kills the application, and a trace
/// read afterwards would be whatever happened to be flushed.
struct Pressed {
    /// The running application, still open on the fixture.
    session: Session,
    /// How many [`APPLIED`] lines existed **before** the row was clicked, so a
    /// caller reads the new one rather than a stale one.
    applied_before: usize,
    /// Likewise for [`FUNNEL`]: the unshared check's "nothing was edited"
    /// assertion is about lines added by *this* press.
    funnel_before: usize,
}

/// Steps 1–4, identical for both checks: open the fixture, leave Read mode,
/// select a leaf inside the form, open the context menu on it, and press the
/// unshare row.
///
/// Returns `Ok(Ok(Pressed))` when the row was pressed, `Ok(Err(failure))` when
/// an assertion up to and including step 4 did not hold, and `Err(skip)` when a
/// precondition was absent. The three-way split is the suite's SKIP/FAIL/PASS
/// rule made structural: an author who reaches for `?` gets a SKIP, which is
/// the safe default — the unsafe default would be a pass.
#[allow(clippy::too_many_lines)]
fn open_and_press(
    ctx: &CheckContext,
    report: &mut CheckReport,
    scenario: &Scenario,
) -> Result<std::result::Result<Pressed, String>> {
    let vocab = &ctx.profile.vocab;

    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check cannot be performed without \
             clicking and right-clicking. Reported as SKIPPED rather than passed: a check that \
             did not run has learned nothing.",
        ));
    }
    let fixture = engine_fixture(scenario.fixture).ok_or_else(|| {
        Error::new(format!(
            "the engine's form fixture is not at D:/Dev/pdfcer/fixtures/synthetic/{}. This check \
             pins it and ignores --pdf: the fixture IS the condition under test — whether the \
             form is drawn on more than one page is a property of the file and of nothing else — \
             and on an arbitrary document the honest answer is 'there was nothing to unshare', \
             which is neither a pass nor a defect.",
            scenario.fixture
        ))
    })?;
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its mode segments or its menu rows are. Both are load-bearing here: this \
             check has to leave Read mode, and it has to press a row in a popup whose position \
             depends on where the pointer was.",
            ctx.profile.name
        ))
    })?;

    // --- launch ------------------------------------------------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out(scenario.trace_name));
    spec.pdf = Some(fixture.clone());
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
    report.note(format!(
        "launched {} as pid {} on {}",
        exe.display(),
        session.pid(),
        fixture.display()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process and \
             this check has no oracle. Captured stderr is at {}.",
            vocab.start_event,
            session.trace_path().display()
        )));
    }

    // --- 1: leave Read, where a content click is refused BY DESIGN ---------
    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, "edit")?;
    report.note(
        "clicked the Edit mode segment first — the shell's default mode is Read, where a canvas \
         click on content is refused by design (DEFECTS.md D6)",
    );

    // --- 2: select something INSIDE the form -------------------------------
    //
    // ★ The operand this command derives from is a LEAF, and nothing else will
    // do: `format.unshare_form`'s dispatch arm reads the selection's first leaf
    // and asks for that leaf's outermost enclosing form. Selecting the form
    // itself would leave `selection.in_form` false and grey the row — which is
    // correct behaviour and would look exactly like the defect.
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, PAGE, 0)?;
    report.note(format!(
        "canvas rect {:?} at zoom {:.3}",
        mapping.image_rect, mapping.zoom
    ));
    let frame = session.frame()?;
    let target = aim(&mapping, &frame, scenario.aim_at)?;
    report.note(format!(
        "the square's centre (page 0, {:.1}, {:.1}) -> screen ({}, {})",
        scenario.aim_at.0,
        scenario.aim_at.1,
        target.x(),
        target.y()
    ));
    driver.click_at(target)?;
    session.settle(15);

    let after = session.trace()?;
    let Some(first) = last_first(&after) else {
        return Err(Error::new(format!(
            "the click produced no `{SELECTION_EVENT} … {FIRST_FIELD}=` line, so the harness has \
             no oracle for what is selected and everything after it would be guesswork. \
             `a_click_inside_a_form_selects_what_is_drawn_there` is the check that owns this \
             step; if it is also failing, fix that one first. Trace: {}",
            session.trace_path().display()
        )));
    };
    if !first.starts_with("leaf:") {
        return Err(Error::new(format!(
            "the click on the square selected `{FIRST_FIELD}={first}`, and this check needs a \
             LEAF — an object painted from inside the form — because that is the only operand \
             `format.unshare_form` can derive its form from. Reported as SKIPPED rather than \
             failed: the failure is in the deep hit test, which \
             `a_click_inside_a_form_selects_what_is_drawn_there` owns, and blaming the unshare \
             for it would send the next reader to the wrong file. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ selected {FIRST_FIELD}={first} — inside the form"
    ));

    // --- 3: right-click the same point -------------------------------------
    driver.right_click_at(target)?;
    session.settle(35);

    let trace = session.trace()?;
    let Some(menu) = trace.events(MENU_EVENT).last() else {
        return Ok(Err(format!(
            "THE RIGHT-CLICK RESOLVED NO MENU AT ALL: no `{MENU_EVENT}` line after a secondary \
             click on a selected leaf. `canvas::menus::attach` writes that line on every frame \
             carrying a secondary click, so its absence means the click never reached the canvas \
             response. Trace: {}",
            session.trace_path().display()
        )));
    };
    let context = menu.get("context").unwrap_or_default();
    if context != OBJECT_CONTEXT {
        return Ok(Err(format!(
            "THE RIGHT-CLICK ON A FORM-INTERIOR OBJECT RESOLVED `{context}`, NOT \
             `{OBJECT_CONTEXT}`: `{}`. A leaf IS an object selection, so the object menu is the \
             one that must appear; resolving the view menu here would mean the right-click hit \
             test cannot see inside a form, which is the same defect as \
             `a_click_inside_a_form_selects_what_is_drawn_there` reaching through a second door. \
             Trace: {}",
            menu.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!("★★ the right-click resolved `{}`", menu.raw));

    // --- 4: the row is on screen, and O53 is satisfied ---------------------
    //
    // ★★★ This assertion is about the ROUTE, not about the verb, and it is the
    // one that fails if `format.unshare_form` is registered on the ribbon only.
    // A greyed row publishes no rect either — `plan::resolve` drops a disabled
    // command before it is drawn — so this also catches a build where
    // `selection.in_form` is not published for a leaf selection.
    //
    // ★★ It is also the assertion R9 is defended by. The correct-looking
    // "improvement" to this feature is to grey the row when the form is not
    // shared — and that would need a whole-document page walk in a per-frame
    // condition, sixty times a second, to learn an answer that moves only when
    // the document does. The row therefore stays LIVE on both fixtures, and the
    // unshared check asserts exactly that before asserting the worded decline.
    let Some(row) = declared(&trace, ui_rect, ROW_REGION) else {
        return Ok(Err(format!(
            "★★★ THE UNSHARE ROW IS NOT IN THE CANVAS OBJECT MENU: no `{ROW_REGION}` region \
             after the menu opened. Rows it DID publish: {}.\n\
             Four readings, and all four are defects: the command is registered on the Format \
             ribbon tab only (O53); the row is drawn but disabled, because `selection.in_form` \
             is not set for a leaf selection and a disabled command is dropped before it is \
             drawn; somebody greyed it on 'the form is not shared', which R9 forbids because the \
             condition is a document walk and conditions run per frame — decline in words \
             instead; or `MenuHost::attach_with` has stopped supplying a rect sink, in which \
             case no context-menu row anywhere in this application can be pressed by a check. \
             Trace: {}",
            list(&declared_names(&trace, ui_rect, ROW_PREFIX)),
            session.trace_path().display()
        )));
    };
    if !row.is_substantial() {
        return Ok(Err(format!(
            "`{ROW_REGION}` was published at {row:?}, which has no usable area — so the row \
             exists in the plan and was laid out to nothing. A click aimed at a degenerate \
             rectangle proves nothing, and this is itself the finding."
        )));
    }
    report.note(format!(
        "★★★ the unshare row is in the CONTEXT MENU at {row:?} — O53's requirement that a \
         command must not exist only on the ribbon, and R9's that it stays live rather than \
         being greyed on a fact only a document walk knows"
    ));

    // --- 5: press it -------------------------------------------------------
    let before = session.trace()?;
    let applied_before = before.events(APPLIED).count();
    let funnel_before = before.events(FUNNEL).count();
    driver.click_at(session.frame()?.declared_center(row))?;
    session.settle(30);

    Ok(Ok(Pressed {
        session,
        applied_before,
        funnel_before,
    }))
}

/// Read the `other=` field off the last [`MEASURED`] line, which is the number
/// the verb's whole decision was made from.
///
/// `None` means the line is absent or malformed, and both callers treat that as
/// the same finding: **the walk did not run**, which is the state the feature
/// shipped in and the state these checks exist to prevent returning to.
fn other_pages(trace: &Trace) -> Option<usize> {
    trace.last(MEASURED).and_then(|l| l.get_usize(OTHER_FIELD))
}

/// The failure text for a press that produced no [`MEASURED`] line.
///
/// Shared by both checks because it is one defect with one diagnosis, and a
/// second copy of the paragraph would be a second place for it to drift.
fn no_measurement(trace_path: &std::path::Path) -> String {
    format!(
        "★★★ THE COMMAND DID NOT MEASURE THE DOCUMENT: no `{MEASURED}` line after the row was \
         pressed.\n\
         This is the defect of 2026-08-28 exactly: nothing in the chain from \
         `catalog/format.rs` (gates on `selection.in_form`) through `conditions.rs` (defines it \
         as 'a leaf id is in the selection for this page') through `dispatch/format.rs` (adds \
         'the leaf resolves to a containing form') to the engine's `unshare_form` (guards \
         encryption, certification, /Size, form-not-on-page and nesting) asks whether the form \
         is invoked more than once. Without that walk the command cannot decline on an unshared \
         form and cannot state a true number on a shared one, and it will do neither silently. \
         `app::actions::xobject::fanout` is the function that must run, on the press, before \
         `vector_edit` is called. Trace: {}",
        trace_path.display()
    )
}

/// The shared case: the row copies, and the disclosure has a real number
/// behind it.
fn drive_shared(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let pressed = match open_and_press(ctx, report, &SHARED)? {
        Ok(pressed) => pressed,
        Err(failure) => return Ok(Some(failure)),
    };
    let session = &pressed.session;
    let trace = session.trace()?;

    // --- the measurement ---------------------------------------------------
    let Some(other) = other_pages(&trace) else {
        return Ok(Some(no_measurement(session.trace_path())));
    };
    if other != 1 {
        return Ok(Some(format!(
            "★★★ THE FAN-OUT WAS COUNTED WRONG: `{}` reports {OTHER_FIELD}={other}, and this \
             fixture has exactly ONE other page drawing the form — two pages, one invocation \
             each, so unsharing page 0 leaves page 1 on the original.\n\
             `{OTHER_FIELD}=2` is the off-by-one that forgets to subtract the page being \
             unshared, which is invisible on any file with several sheets and wrong on every \
             file with one — it is how a single-page document gets told that one other page \
             shares its drawing. `{OTHER_FIELD}=0` means the walk found the form on this page \
             only, which on this fixture means it never read page 1. Trace: {}",
            trace
                .last(MEASURED)
                .map_or(String::new(), |l| l.raw.clone()),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ `{}` — the document was walked and the other page was counted",
        trace
            .last(MEASURED)
            .map_or(String::new(), |l| l.raw.clone())
    ));

    // --- the edit ----------------------------------------------------------
    let applied: Vec<_> = trace.events(APPLIED).collect();
    let Some(line) = applied.get(pressed.applied_before) else {
        return Ok(Some(format!(
            "★★★ THE ROW WAS PRESSED ON A GENUINELY SHARED FORM AND NOTHING WAS UNSHARED: no \
             new `{APPLIED}` line.\n\
             **This is what the defect looks like from the operator's chair, and it looks like \
             success**: the copy `unshare_form` makes is byte-identical to the original, so a \
             page that WAS unshared renders pixel-for-pixel as one that was not. Nothing on \
             screen distinguishes them.\n\
             The measurement above ran and found {OTHER_FIELD}={other}, so the walk is not the \
             problem. Candidate causes, in the order they are worth checking: \
             `app::actions::xobject::fanout` declined on a document it should have proceeded on \
             (look for a `{DECLINED}` line — on this fixture that is itself the defect); \
             `containing_form_object` returned the INNERMOST enclosing form, which \
             `EditError::FormNestedInAnotherForm` refuses by name; or the engine declined for a \
             document-wide reason, in which case `app::status::decline` is carrying a worded \
             sentence and the status bar is showing it. Trace: {}",
            session.trace_path().display()
        )));
    };
    report.note(format!("★★★ `{}`", line.raw));

    let original = line.get("original").unwrap_or_default();
    let copy = line.get("copy").unwrap_or_default();
    let moved = line.get("moved").unwrap_or_default();

    if copy == original || copy.is_empty() {
        return Ok(Some(format!(
            "THE COPY IS NOT A COPY: `{}` reports copy={copy:?} against original={original:?}. \
             `unshare_form` allocates a new object number and clones the stream into it; two \
             equal numbers mean the page was re-pointed at the object it already named, which is \
             a no-op wearing a success line.",
            line.raw
        )));
    }
    if moved != "1" {
        return Ok(Some(format!(
            "THE PAGE'S REFERENCES DID NOT MOVE AS EXPECTED: `{}` reports moved={moved:?}, and \
             this fixture's page 0 invokes its one form under exactly one name.\n\
             `0` means the copy was allocated and nothing was re-pointed at it — an orphan \
             object in the file and no change to what the page draws, which is strictly worse \
             than refusing. A number above 1 means the page's /XObject dictionary named the form \
             more than once, which this seven-object hand-written fixture does not do, so it \
             says the count is being derived from something other than this page's own names — \
             page 1's invocation being counted into page 0's move is the specific way that goes \
             wrong here, and it would mean the verb had unshared BOTH pages onto one copy.",
            line.raw
        )));
    }
    report.note(format!(
        "★★★ page 0 now names its own copy of form {original}: object {copy}, {moved} reference \
         moved. Page 1 — the one other page the walk counted — keeps naming {original} and is \
         byte-identical."
    ));

    Ok(None)
}

/// The unshared case: the row declines in words, and the document is untouched.
fn drive_unshared(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let pressed = match open_and_press(ctx, report, &UNSHARED)? {
        Ok(pressed) => pressed,
        Err(failure) => return Ok(Some(failure)),
    };
    let session = &pressed.session;
    let trace = session.trace()?;

    // --- the measurement ran, and found nothing else ------------------------
    let Some(other) = other_pages(&trace) else {
        return Ok(Some(no_measurement(session.trace_path())));
    };
    if other != 0 {
        return Ok(Some(format!(
            "★★ THE WALK COUNTED PAGES THIS FIXTURE DOES NOT HAVE: `{}` reports \
             {OTHER_FIELD}={other} on a ONE-PAGE document whose single form is invoked once. \
             Any non-zero answer here is a counting bug — most likely the page being unshared is \
             not being subtracted from the set, which would make every single-sheet drawing look \
             shared and would restore the defect this check exists for. Trace: {}",
            trace
                .last(MEASURED)
                .map_or(String::new(), |l| l.raw.clone()),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ `{}` — the document was walked and no other page draws the form",
        trace
            .last(MEASURED)
            .map_or(String::new(), |l| l.raw.clone())
    ));

    // --- it declined, and said so -------------------------------------------
    //
    // ★★★ The positive half. See the module header: proving a decline from the
    // absence of `unshare-form-applied` would pass on every build where the
    // press never arrived at all.
    let Some(declined) = trace.last(DECLINED) else {
        return Ok(Some(format!(
            "★★★ THE COMMAND MEASURED {OTHER_FIELD}={other} AND DID NOT DECLINE: no \
             `{DECLINED}` line.\n\
             The walk ran, it found that nothing else on this document draws the form, and the \
             verb went ahead anyway. What the operator gets for that press is an allocated \
             object holding a byte-identical clone, a rewritten page /Resources, an undo entry \
             to step back over, a DIRTY DOCUMENT where a clean one was open — and a status line \
             asserting that every other page still shares the original, about a file with one \
             page.\n\
             `app::actions::xobject::fanout` must return `None` on this measurement, having \
             recorded `UnshareRefusal::NotShared`, and `unshare` must return before \
             `vector_edit` is called. Trace: {}",
            session.trace_path().display()
        )));
    };
    let reason = declined.get("reason").unwrap_or_default();
    if reason != "not-shared" {
        return Ok(Some(format!(
            "THE COMMAND DECLINED FOR THE WRONG REASON: `{}` reports reason={reason:?}, and on \
             this fixture the only correct one is `not-shared`. Any other value means the \
             document is being refused for a fault it does not have — this file is unencrypted, \
             unsigned, structurally sound and hand-written — and the operator would be shown a \
             sentence about a problem instead of the one sentence that is true here, which is \
             that the drawing already belongs to this page alone.",
            declined.raw
        )));
    }
    report.note(format!(
        "★★★ `{}` — the decline is worded, not silent",
        declined.raw
    ));

    // --- and NOTHING was edited ---------------------------------------------
    //
    // ★★★ The clause that makes the decline worth having. A build that worded
    // a refusal and performed the edit anyway would pass every assertion above.
    let applied_now = trace.events(APPLIED).count();
    if applied_now > pressed.applied_before {
        return Ok(Some(format!(
            "★★★ IT DECLINED AND COPIED ANYWAY: a new `{APPLIED}` line appeared alongside the \
             `{DECLINED}` one. The operator is now looking at a status bar saying nothing \
             happened, over a document that has an extra object in it and is marked dirty — \
             which is worse than either behaviour on its own, because the sentence and the file \
             now disagree. Trace: {}",
            session.trace_path().display()
        )));
    }
    let funnel_now = trace.events(FUNNEL).count();
    if funnel_now > pressed.funnel_before {
        return Ok(Some(format!(
            "★★★ THE DOCUMENT WAS EDITED DESPITE THE DECLINE: a new `{FUNNEL}` line appeared. \
             `vector_edit` writes that line once per COMMITTED edit — it carries the bumped \
             `epoch` — so its presence means an undo entry exists and the document is dirty. \
             Declining is only better than performing the copy if it actually changes nothing; \
             a save prompt the operator did not earn is most of what this check is protecting \
             them from. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note(
        "★★★ no `unshare-form-applied` and no `unshare-form` funnel line: nothing was \
         allocated, no /Resources was rewritten, no undo entry was pushed and the document is \
         as clean as it was before the press",
    );

    Ok(None)
}

/// A page-space point, through the mapping and the window frame, to a desktop
/// point.
///
/// Its own function so the click and the right-click cannot hop differently —
/// the class of error `crate::coords` exists to prevent. Both gestures in this
/// check aim at the *same* screen point, and that is load-bearing: the menu
/// must open over the thing that was selected, not over a second guess at where
/// it is.
fn aim(mapping: &CanvasMapping, frame: &WindowFrame, point: (f64, f64)) -> Result<ScreenPoint> {
    let window = mapping.doc_to_window(DocPoint {
        page: 0,
        x: point.0,
        y: point.1,
    })?;
    Ok(frame.to_screen(window))
}

/// The `first=` value of the most recent `canvas-selection` line, if any.
///
/// ★ The **last** line rather than a count of new ones, for
/// `form_selection::last_first`'s reason: `canvas-selection` is emitted through
/// `diag::trace_changed`, so a click producing the same selection as the
/// previous one emits nothing, and a consumer that counted lines would read a
/// legitimate no-change as a dropped event.
fn last_first(trace: &Trace) -> Option<String> {
    trace
        .last(SELECTION_EVENT)
        .and_then(|l| l.get(FIRST_FIELD))
        .map(str::to_owned)
}
