//! `move_line_of_text` — **a drag on one line inside a block of text MOVES
//! it, or says why it cannot.**
//!
//! The driven half of `OPERATOR_REQUESTS.md` **O188**, changes (B) and (C). It
//! is the only thing that can say the feature works, because the defect O188
//! names is a *silence*, and a silence is exactly what a green unit-test suite
//! looks like.
//!
//! # ★★★ A CHECK THAT PINS AN ABSENCE HAS A SHELF LIFE IN DAYS
//!
//! This check first asserted that a single line could NOT be moved, because
//! on the day it was written the engine had no verb for it. The verb landed,
//! and the assertion became a confident statement of the opposite of the
//! truth — green the whole time.
//!
//! ⇒ **A driven check that pins a capability's absence has a shelf life
//! measured in days.** Nothing in this repository changed; a dependency did,
//! and a green suite became a liar. The defence is not to avoid such checks —
//! this one caught a real silence — but to write the absence down where the
//! backlog gate can see it (`ENGINE_BACKLOG.md`) so that the delivery arrives
//! as a failing gate rather than as a quietly wrong assertion.
//!
//! # What it asserts: four drags, four answers, one document
//!
//! A line move succeeds on most lines and refuses on three shapes, and the
//! operator is shown a different sentence for each refusal. So the check
//! drives all four answers against one fixture:
//!
//! | the line under the pointer | what must happen |
//! |---|---|
//! | one written in two pieces, the join having no position of its own | refused, *a piece inside this line has no position* |
//! | one that states its own position, with no successor | **it MOVES** |
//! | one the NEXT line's position is measured from | refused, *moving it would drag that line along too* |
//! | one whose position this document does not state | refused, *there is no position here to change* |
//!
//! ★★★ **The row that MOVES is not a bonus, it is what makes the other three
//! mean something.** A table of refusals alone passes for ever against the
//! build this check was originally written for — the one that refused every
//! line move. A check that cannot fail on the dangerous build is not a check.
//!
//! # ★★★ WHY THIS CHECK EXISTS WHEN FOUR UNIT TESTS ALREADY COVER IT
//!
//! `canvas::moving::tests` asserts, at the seam, that `decline` pushes the
//! right `Action::DeclineOnCanvas`, and that a movable line produces
//! `VectorAction::MoveTextLine`. Those tests are right and they are not
//! evidence, for this project's founding
//! reason (**R1**): they call the function. They cannot see the chain in front
//! of it — whether a real drag at a real rung reaches `decline` at all,
//! whether the apply phase has an arm for the action, whether the status bar
//! draws what the store holds, or whether a mode gate swallows the gesture two
//! layers earlier. Eight green tests once sat in front of a feature that did
//! one step of fourteen.
//!
//! So this drives the OS: a real click into Edit mode, a real chord to arm the
//! Points tool, a real click to select one line, a real press-move-release,
//! and then it reads what three independent subsystems wrote down.
//!
//! # The oracle — three lines, three subsystems, in order (per aim)
//!
//! ```text
//! canvas       canvas-move-declined level=Part sel=1 reason=run-would-move-next …
//! apply phase  canvas-decline-recorded what=text-run-would-drag-next-line
//! status bar   ui-rect name=status-group:decline rect=…
//! ```
//!
//! and, for the row that commits, one line from a fourth subsystem:
//!
//! ```text
//! apply phase  move-text-line page=0 n=1 epoch=3 disclosures=none
//! ```
//!
//! ★ `n=` is the number of show operators the line is written in. The one
//! aim that commits here is a one-piece line, so it is `1`; on his own sheet
//! the same line reports `n=9`.
//!
//! | line | question it answers | who writes it |
//! |---|---|---|
//! | `canvas-move-declined` | did the gesture reach the move rules, and refuse for the reason this check is about? | `crate::canvas::moving`, holding `&OpenDoc` |
//! | `canvas-decline-recorded` | did the refusal's **sentence** cross the `Action` boundary and reach the store? | `crate::app::status::decline::canvas`, holding `&mut` |
//! | `status-group:decline` | was it **drawn**, on a frame, where he could read it? | `crate::app::status::disclosure` |
//!
//! ★★ **That is a chain measurement, not the application agreeing with
//! itself.** The three writers are three subsystems separated by the exact
//! boundary O188's design is about — a canvas gesture holds no `&mut` and can
//! only *ask*. A shell that raised the action and had no apply arm for it
//! writes the first line and not the second. A shell that recorded the
//! sentence into a store the bar never reads writes the first two and not the
//! third.
//!
//! ## ★★★ Why the region alone would have been worthless
//!
//! `status-group:decline` is **one region shared by every decline in the
//! application** — a save that failed, a bookmark that would not move, a zoom
//! with nothing to frame. A check asserting only *"that region is on screen
//! after the drag"* is satisfied just as well by a build that raised the wrong
//! sentence, and by a stale sentence left on the bar by an earlier gesture.
//!
//! > An assertion both outcomes satisfy is not a measurement of which one
//! > shipped. Name what the WRONG mechanism cannot produce.
//!
//! `canvas-decline-recorded what=…` is that name. It carries a **stable
//! token**, not a `Debug` rendering, for the reason this project has recorded
//! twice: a `{:?}` field is a property of how a variant is *spelled*, so a
//! rename turns a driven check into a confident false negative that quotes the
//! truth in its own failure message.
//!
//! # ⚠ HOW TO FALSIFY THIS CHECK — do this before believing a PASS
//!
//! A check that has never been seen to fail is not evidence.
//!
//! 1. **Copy the file aside first.**
//!    `cp crates/pdfcer-gui/src/canvas/moving/mod.rs $SCRATCH/moving.rs.bak`.
//!    **Never `git checkout` to undo it** — this project runs parallel tracks
//!    and that discards another track's uncommitted work.
//! 2. **Plant the silence Ken reported**: in `Refusal::worded`, change
//!    `Self::TextRunCannotMove(RunMoveBlock::NoPositionOfItsOwn)` from
//!    `Some(CanvasDecline::TextRunHasNoPositionOfItsOwn)` to `None`. The
//!    refusal still happens and still traces; it just says nothing, which is
//!    the shape of the defect O188 names.
//! 3. **Prove the plant is in the artifact.** `cargo build --release -p
//!    pdfcer-gui`, then confirm the exe is newer than the source — a stale
//!    binary is the commonest cause of a falsification that "did not
//!    reproduce". Do not grep the exe for the token: `token()` is untouched
//!    by this plant, so the string is still in there and its presence proves
//!    nothing either way.
//! 4. **Require the `[FAIL]` line**, not the exit code — a SKIP exits the way
//!    a PASS does. It must fail on **the last aim only**, naming *the refusal
//!    happened and raised no sentence* and quoting the `canvas-move-declined`
//!    line it saw. The other three aims must still pass: a plant that reddens
//!    all four has broken the drag, not the sentence, and has measured
//!    nothing about this check's discrimination.
//! 5. **Restore from the byte copy**, rebuild, confirm the PASS returns.
//!
//! ★ A second, cheaper plant exercises the third link on its own: in
//! `app::status::decline::show`, return before `disclosure_line` publishes
//! `REGION_DECLINE`. All three declining aims should then fail on the
//! *region* while still reporting the right `canvas-decline-recorded` token —
//! which is the one outcome that separates "recorded but never drawn" from
//! "never recorded".
//!
//! # Fixture — pinned here, not passed on the command line
//!
//! `fixtures/inherited-runs.pdf` at page 0, four aims in PDF user space.
//!
//! ★★★ **NOT `paragraph.pdf`, which every other line-of-text check in this
//! harness uses** — and the reason is the whole argument for a second
//! fixture. `paragraph.pdf` writes a `Tm` in front of all six of its show
//! operators, so every one of its lines states its own position, so
//! `text_run_move_refusal` answers `None` six times out of six and **neither
//! refusal can be reached on it**. A check written against it alone would pass
//! on a build that had deleted the pre-check entirely, for ever.
//!
//! `inherited-runs.pdf` is one `BT`…`ET` block holding **five** `Tj`
//! operators at 12 pt on a 612 × 792 page — three horizontal, then a pair
//! turned a quarter turn — with a positioning operator in front of three of
//! them and **none** in front of the other two. Grouped into visual lines that
//! is **four** lines producing all four of the engine's answers in one
//! document, which is what [`AIMS`] drives.
//! `tools/gen-inherited-runs-fixture.py` builds it and carries the reasoning;
//! `fixtures/inherited-runs.PROVENANCE.md` carries the measured spans the aims
//! in [`AIMS`] were computed from.
//!
//!
//! ⚠ A missing fixture is a **FAIL**, not a SKIP: it is committed to this
//! repository, so its absence is a broken checkout rather than an unavailable
//! precondition, and a SKIP would say the opposite.
//!
//! # ★★ The Points tool, and why there is no double-click here
//!
//! The Part rung on a **text** object is not reachable by double-click, by
//! design: `canvas::clicking`'s O70 arm opens the caret instead, which is the
//! operator's own ruling (*"double-clicking inside the bounding box should
//! edit the text"*). The route that exists is the **Points** tool — chord
//! `A`, labelled *Points* because a draughtsman says point — whose branch
//! takes the click before every other claimant and calls
//! `SelectionState::click_direct`, landing on the Part rung whenever the probe
//! found a part. On a text object a "part" **is** a visual line — which may
//! be written in any number of show operators, and on his own sheet usually
//! is.
//!
//! `deeper_rung_delete::Rung::arms_the_points_tool` carries the measurement
//! that established this, including the trace lines it was read out of.
//!
//! ★ The chord is pressed **before** the click, not after: with the arrow
//! armed, the first click would select the whole text object and the Points
//! tool's own branch would then be entering an object it did not pick.
//!
//! ★★ It also does the check a second favour, and the check depends on it.
//! Arming the tool is a **command**, and `app::status::decline::retire` runs
//! at the top of `dispatch_command` — so any decline left on the bar by an
//! earlier gesture is cleared before this one starts. That is what makes the
//! "nothing on the bar before the drag" control below assertable rather than
//! hopeful.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode whose canvas may select and edit page content.
///
/// The shell's default is Read, where a canvas click on content is refused BY
/// DESIGN. A check that skipped this step would report the mode gate as a
/// selection defect — which `delete_key`'s own header records as having
/// happened.
const MODE: &str = "edit";

/// `canvas-selection via=… sel=… level=… first=…`.
const SELECTION_EVENT: &str = "canvas-selection";

/// The rung this check must be standing on before it drags.
const PART_LEVEL: &str = "Part";

/// `canvas-move-declined level=… sel=… reason=… detail=…` — written by
/// `canvas::moving::decline`, **on release only**.
///
/// ★ Not per frame. An in-flight drag is re-evaluated 60 times a second and a
/// refusal traced per frame would bury every other event on the channel — the
/// lesson `canvas-pointer` taught when a stationary pointer emitted fifty
/// identical lines in nine seconds.
const MOVE_DECLINED_EVENT: &str = "canvas-move-declined";

/// `canvas-decline-recorded what=…` — the apply phase's line, written once per
/// decline by `app::status::decline::canvas::record_canvas`.
///
/// ★★★ **The per-refusal tokens live in [`AIMS`], not in constants here.** A
/// token hoisted to module scope reads as a property of the check, so it
/// survives the disappearance of the refusal it names — the assertion stays
/// green while the constant becomes the name of nothing. A token that only one
/// row needs belongs in that row, where the answer it describes is visible
/// beside it and a changed answer changes them together.
const RECORDED_EVENT: &str = "canvas-decline-recorded";

/// The `⊗` slot in the status bar. `app::status::decline::show` draws into it
/// and publishes it as a `ui-rect` on the frame it draws.
///
/// ⚠ Read through [`driving::declared`], never with `Trace::last`. The
/// application's `ui-rect` channel is a **change log** — it emits when a rect
/// moves and emits a `ui-rect-gone` when a region stops being drawn — so
/// `.last()` returns a fossil for a region that has been retired, and a caller
/// cannot tell it from a live one. That misreading once produced a confident,
/// detailed, entirely wrong layout-defect report about eighteen ribbon
/// controls.
const DECLINE_REGION: &str = "status-group:decline";

/// The fixture. See the module header, and
/// `fixtures/inherited-runs.PROVENANCE.md`, for why it is not
/// `paragraph.pdf` like every other line-of-text check in this harness.
const FIXTURE: &str = "inherited-runs.pdf";
/// Page index of [`FIXTURE`] this check uses.
const PAGE: usize = 0;

/// ★★★ **The four aims, and the answer each one must produce.**
///
/// One launch, one fixture, four drags — one per visual LINE of
/// `inherited-runs.pdf`. Every field here is measured; the spans are recorded
/// in `fixtures/inherited-runs.PROVENANCE.md` and pinned by
/// `provider::line::tests::the_local_fixture_gives_all_four_line_move_answers`,
/// which decomposes the same file.
///
/// ★★ **The aim is not asserted directly, and it does not need to be.** The
/// four lines produce four *different* answers, so an aim that landed on the
/// wrong one produces the wrong answer and this check fails — loudly, naming
/// what it got. That is the property a table of four buys that four separate
/// checks against four separate fixtures could not: the discriminating power
/// is in the document, not in the harness's arithmetic.
///
/// ★★★ **Two aims are on rotated text, and that is not decoration.** An
/// inherited show operator advances along the text direction, so a horizontal
/// one always lands on its predecessor's baseline and is always inside its
/// predecessor's line. A line can only BEGIN with an inherited piece when the
/// text is turned — so without rows three and four, `run-has-no-position` and
/// `run-would-move-next` are tokens no document could produce at line
/// granularity, and this check would pass on a build that had deleted both.
const AIMS: [Aim; 4] = [
    Aim {
        what: "the first line, written in two pieces, the second of which has no position",
        at: (87.0, 704.0),
        expect: Expect::Declines {
            reason: "line-piece-has-no-position",
            recorded: "text-line-piece-has-no-position",
        },
    },
    Aim {
        what: "the second line, which states its own position and nothing follows on from it",
        at: (128.0, 664.0),
        expect: Expect::Moves,
    },
    Aim {
        what: "the turned line that the line after it is measured from",
        at: (297.0, 414.0),
        expect: Expect::Declines {
            reason: "run-would-move-next",
            recorded: "text-run-would-drag-next-line",
        },
    },
    Aim {
        what: "the turned line whose position this document does not state",
        at: (297.0, 448.0),
        expect: Expect::Declines {
            reason: "run-has-no-position",
            recorded: "text-run-no-position-of-its-own",
        },
    },
];

/// One row of [`AIMS`].
struct Aim {
    /// What the operator would call the thing under the pointer, used in every
    /// note and failure message this check writes. Not a run index: a message
    /// that says *run 1* is a message whoever reads it has to go and decode.
    what: &'static str,
    /// Where to press, in PDF user space (y up).
    at: (f64, f64),
    /// What must happen on release.
    expect: Expect,
}

/// What a drag on one of [`AIMS`] must produce.
///
enum Expect {
    /// The move rules must refuse, with this `reason=` on `canvas-move-declined`
    /// and this `what=` on `canvas-decline-recorded`, and the status bar must
    /// then draw the sentence.
    Declines {
        /// `canvas::moving::Refusal::token`.
        reason: &'static str,
        /// `app::status::decline::canvas::CanvasDecline::token`.
        recorded: &'static str,
    },
    /// The move must COMMIT — the funnel's own line, and no decline anywhere.
    Moves,
}

/// The funnel label `VectorAction::MoveTextLine`'s apply arm passes to
/// `vector_edit_on_page`, which becomes the head of its success line:
/// `move-text-line page=0 n=N epoch=N disclosures=…`.
///
/// ★ `n=` is the number of show operators the line is written in, not `1`.
/// One `move_text_run` is issued per piece and the whole set is folded into
/// one undo step, so a line a producer wrote as nine `Tj`s reports `n=9`.
///
/// ★★ Asserted instead of `canvas-move`, and the difference is the whole
/// point of asserting it. `canvas-move` is written by the canvas when it
/// RAISES the action; this one is written by the apply phase after the engine
/// has accepted the edit and the epoch has moved. A shell that raised a move
/// the engine then refused writes the first and not the second — and that is
/// precisely the build a pre-check regression produces.
const MOVED_EVENT: &str = "move-text-line";

/// How far the drag travels, in window logical points, on each axis.
///
/// ★ Comfortably past any drag threshold and past
/// `canvas::moving::Refusal::NoTravel`'s floor, and small enough that the
/// pointer stays well inside the canvas on a 612 × 792 page at fit zoom. The
/// destination does not matter: the release is refused before any geometry is
/// computed, so this only has to be *a drag* rather than *a click*.
const DRAG_PX: f32 = 60.0;

/// See the module documentation.
pub struct DraggingOneLineOfTextMovesItOrSaysWhy;

impl Check for DraggingOneLineOfTextMovesItOrSaysWhy {
    fn name(&self) -> &'static str {
        "dragging_one_line_of_text_moves_it_or_says_why"
    }

    fn defect(&self) -> &'static str {
        "Dragging one line inside a block of text does nothing and says nothing — either the \
         move never reaches `move_text_run` on a line the engine would accept, or a line the \
         engine refuses is refused in silence, and in both cases the operator is left with a \
         gesture that did not happen and no way on screen to learn why"
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
        "--pdf and --doc-point are IGNORED: pinned to {} at page {PAGE}, {} aims",
        pdf.display(),
        AIMS.len()
    ));

    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check arms a tool with a real chord, \
             selects with a real click and drags with a real press-move-release. Reported as \
             SKIPPED rather than passed: a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so this check can neither \
             leave Read mode nor read the status bar's decline slot.",
            ctx.profile.name
        ))
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

    // --- launch -------------------------------------------------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("move_line_of_text.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
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
    let driver = Driver::new(session.window());

    // --- 1: Edit ------------------------------------------------------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    // --- 2: read the canvas mapping -----------------------------------------
    //
    // ★★★ **The Points tool is armed inside [`one_aim`], NOT once here, and the
    // reason is measured rather than stylistic.** This check's first shape armed
    // it once before the loop, on the reasoning that a tool stays armed and the
    // two later rows would then not get a free `decline::retire` from a fresh
    // command. The first driven run refuted that in one line: the ladder was at
    // `Object` on row 1, so the click had gone to the arrow tool.
    //
    // The cause is `canvas::keys`' own documented Escape contract — *"pressing
    // Escape twice puts the tool down; pressing it once corrects a mis-aimed
    // pick"*. Row 1 presses Escape with **nothing selected**, so there is no
    // pick to correct and that single press is the one that puts the tool down.
    // Arming before a clearing Escape disarms; arming after it does not.
    //
    // ★★ The general shape, worth carrying: **a setup step that runs once and a
    // clearing step that runs per row are in a race, and the clearing step
    // wins.** The fix is not to drop the clearing step — it is what keeps each
    // row's status-bar control honest — but to put every precondition it
    // destroys downstream of it.
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, PAGE)?;
    let frame = session.frame()?;

    // --- 3: each aim in turn ------------------------------------------------
    //
    // ★★★ **The rows share a launch and are otherwise independent**, which is
    // the shape a suite that shares state taught this project to insist on. Any
    // row may FAIL, and a FAIL stops the check — but no row leaves a
    // precondition behind for the next one, because each re-reads the trace
    // from its own mark and each asserts its own empty-slot control before it
    // presses.
    //
    // ★★ The third row EDITS THE DOCUMENT, and it runs last for that reason.
    // A committed move changes the page, and every aim above it is computed
    // from the page as loaded. Running the control first would have moved a
    // line the two refusal rows are then aiming at.
    for aim in &AIMS {
        if let Some(failure) = one_aim(&session, &driver, &mapping, &frame, ui_rect, aim, report)? {
            return Ok(Some(failure));
        }
    }

    Ok(None)
}

/// Drive one row of [`AIMS`]: select the line, drag it, and assert what the row
/// says must happen.
///
/// Same three-way return as [`drive`]: `Err` SKIP, `Ok(Some(_))` FAIL,
/// `Ok(None)` pass.
///
/// # ★★ Why the control is INSIDE this function and not once at the top
///
/// Because `status-group:decline` is one region shared by every decline in the
/// application, and two of the three rows put a sentence in it. A control taken
/// once, before the loop, would be a baseline for row 1 and a fossil for rows 2
/// and 3 — the exact shape of *an absence assertion is only as good as when
/// its baseline was taken*, which cost this project a green check over a
/// planted defect. Each row establishes its own.
///
/// The clearing gesture is a press of **Escape**. It ascends the selection
/// ladder, which is what this row needs: every row starts from no selection and
/// descends to the Part rung by its own click, so no row inherits a rung.
///
///
/// This function's first shape treated an empty status-bar slot as a
/// **precondition** and skipped the row when the slot was still on screen,
/// reasoning that *"Escape is a command, and `decline::retire` runs at the top
/// of `dispatch_command`."* The first driven run refuted it: row 2 skipped with
/// the slot still live, and the trace carries no `ui-rect-gone
/// name=status-group:decline` anywhere — so the sentence really was still
/// drawn.
///
/// The cause is that **Escape is claimed by the canvas before it can become a
/// command.** With a selection standing, `canvas::keys` spends it ascending the
/// ladder, so `dispatch_command` never runs and neither does `retire`.
/// `canvas::keys`' own header states the contract plainly — *"pressing Escape
/// twice puts the tool down; pressing it once corrects a mis-aimed pick"* —
/// and neither of those is a command.
///
/// ★★ **That is behaviour, not a defect, and the fix was to stop needing it.**
/// Both of this check's sentences are `still_true` for the reason
/// `decline::fresh` calls *"the FILE cannot change under it"*: they report a
/// property of the document, so no later moment makes them stale and they are
/// meant to outlive a deselection. A check that demanded an empty slot was
/// asserting a retirement policy this project deliberately does not have.
///
/// ★★★ **So the per-row link is a MARK, not an absence.** Every assertion below
/// is anchored at a `Trace::mark` taken immediately before this row's drag and
/// reads `last_after`. A sentence left standing by an earlier row cannot
/// satisfy them, because the events that carry it are behind the mark. The
/// slot's live-ness is still read — and reported — but as a note, which is
/// the difference between measuring a thing and gating on it.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn one_aim(
    session: &Session,
    driver: &Driver,
    mapping: &CanvasMapping,
    frame: &crate::coords::WindowFrame,
    ui_rect: &'static str,
    aim: &Aim,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let what = aim.what;

    // --- clear whatever the previous row left ------------------------------
    driver.press(vk::ESCAPE)?;
    session.settle(14);
    let before = session.trace()?;
    if driving::declared(&before, ui_rect, DECLINE_REGION).is_some() {
        report.note(format!(
            "a decline from an earlier row is still on the status bar as this row begins \
             — measured rather than assumed, and NOT a precondition: see this function's \
             header for the Escape contract that makes it expected. Row: {what}."
        ));
    }

    // --- arm the Points tool ------------------------------------------------
    //
    // ★★★ After the Escape above, never before it: see step 2 in [`drive`] for
    // the measurement. On a text object this is what makes a single click land
    // on the Part rung at all — a double-click opens the caret and never
    // touches the ladder, and the arrow tool's click selects the whole block.
    driver.press(vk::A)?;
    session.settle(10);

    // --- select the line ----------------------------------------------------
    let window_point = mapping.doc_to_window(DocPoint::new(PAGE, aim.at.0, aim.at.1))?;
    let at = frame.to_screen(window_point);
    driver.click_at(at)?;
    session.settle(14);

    let trace = session.trace()?;
    let selected = trace
        .last(SELECTION_EVENT)
        .and_then(|l| l.get_usize("sel"))
        .unwrap_or(0);
    if selected == 0 {
        return Err(Error::new(format!(
            "the click at document point ({:.1}, {:.1}) on page {PAGE} selected nothing, so \
             there is no line to drag — row: {what}. That is either an aim that is not on a \
             glyph or a broken hit test, and this harness cannot tell them apart, so it \
             declines to file either. The spans the aims were computed from are in \
             `fixtures/inherited-runs.PROVENANCE.md` and are the first thing to re-measure. \
             Trace: {}.",
            aim.at.0,
            aim.at.1,
            session.trace_path().display()
        )));
    }
    let level = trace
        .last(SELECTION_EVENT)
        .and_then(|l| l.get("level").map(str::to_owned))
        .unwrap_or_else(|| "none".to_owned());
    if level != PART_LEVEL {
        return Err(Error::new(format!(
            "the selection ladder is at `{level}` and this row needs `{PART_LEVEL}`, so no \
             single LINE of text is selected and the drag below would be testing the Object \
             rung — which moves the whole text block and is a different verb entirely. The \
             Points tool should land directly on the Part rung; on a text object a part is a \
             show operator. Row: {what}. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- drag ---------------------------------------------------------------
    let mark = trace.mark();
    driver.drag(at, frame.offset_from(at, DRAG_PX, DRAG_PX))?;
    session.settle(26);
    let after = session.trace()?;

    match aim.expect {
        Expect::Moves => moved(session, &after, mark, ui_rect, what, report),
        Expect::Declines { reason, recorded } => declined(
            session, &after, mark, ui_rect, aim, reason, recorded, report,
        ),
    }
}

/// Assert the CONTROL row: the drag committed a `move_text_run`.
///
/// ★★★ **This is the row that proves the other two are measuring something.**
/// It is also the whole of O188's move half as the operator experiences it —
/// he drags one label in a title block and the label moves.
fn moved(
    session: &Session,
    after: &crate::trace::Trace,
    mark: usize,
    ui_rect: &'static str,
    what: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let Some(line) = after.last_after(MOVED_EVENT, mark) else {
        let declined = after
            .last_after(MOVE_DECLINED_EVENT, mark)
            .map(|l| l.raw.clone());
        return Ok(Some(format!(
            "★★★ THE DEFECT: dragging {what} did not move it. No `{MOVED_EVENT}` line \
             follows the release, so the edit never reached `EditSession::move_text_run` \
             through the funnel. {} This line states its own position and has no successor \
             that depends on it, so `text_run_move_refusal` answers `None` for it and the \
             engine would accept the move — which means the refusal, if there was one, is \
             this shell's and not the document's. That is `OPERATOR_REQUESTS.md` O188 \
             unfixed. Trace: {}.",
            declined.map_or_else(
                || "No `canvas-move-declined` line either, so the gesture did not reach the \
                    move rules at all — the press may have landed on paper and become a \
                    marquee, or the travel may have been below the drag threshold."
                    .to_owned(),
                |raw| format!("It was REFUSED instead: `{raw}`.")
            ),
            session.trace_path().display()
        )));
    };
    report.note(format!("dragging {what} moved it: `{}`", line.raw));

    // ★★ And it must NOT have worded a refusal at the operator. A build that
    // both moved the line and put a sentence on the bar is worse than one that
    // did neither, because the operator is told his edit did not happen while
    // looking at it having happened.
    if driving::declared(after, ui_rect, DECLINE_REGION).is_some() {
        return Ok(Some(format!(
            "the drag on {what} COMMITTED (`{}`) and the status bar is showing a decline \
             anyway. Whatever that sentence says, it contradicts the page in front of the \
             operator — `fuzzy, never sneaky` cuts both ways, and a refusal displayed over \
             a successful edit is the worst-reading half of it. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }
    report.note("★★ and no decline was drawn over it — the page and the status bar agree");
    Ok(None)
}

/// Assert a REFUSING row, in the three links the refusal has to survive.
///
/// A build that breaks any one link fails at that link and says which, rather
/// than at a summary: refused for the right reason, worded, and drawn.
#[allow(clippy::too_many_arguments)]
fn declined(
    session: &Session,
    after: &crate::trace::Trace,
    mark: usize,
    ui_rect: &'static str,
    aim: &Aim,
    reason: &str,
    recorded: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let what = aim.what;

    // --- a: did the gesture reach the move rules at all? --------------------
    let Some(line) = after.last_after(MOVE_DECLINED_EVENT, mark) else {
        let moved = after.last_after(MOVED_EVENT, mark).map(|l| l.raw.clone());
        if let Some(raw) = moved {
            return Ok(Some(format!(
                "★★★ THE DEFECT, in the direction that damages a document: the drag on \
                 {what} COMMITTED — `{raw}`. The engine's own guard says this line cannot \
                 move without carrying another line with it, or has no position to change, \
                 so a commit here means `canvas::moving::eligible` is no longer asking \
                 `text_run_move_refusal` before it draws the ghost. The operator moved one \
                 label and something he did not select moved too, silently. Trace: {}.",
                session.trace_path().display()
            )));
        }
        return Err(Error::new(format!(
            "the drag on {what} produced no `{MOVE_DECLINED_EVENT}` line and no \
             `{MOVED_EVENT}` line, so the release never reached `canvas::moving::drag` and \
             this row never exercised its subject. Two ordinary causes this harness cannot \
             tell apart from the trace alone: the press landed on paper rather than on the \
             selected line, so the gesture became a marquee; or the travel was below the \
             drag threshold. SKIPPED rather than failed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let got = line.get("reason").unwrap_or("?");
    if got != reason {
        return Ok(Some(format!(
            "the drag on {what} was refused for `{got}`, and the engine's guard says it must \
             be `{reason}`. The line seen was `{}`. This is a FAILURE and not a skip: the \
             two refusals of a line move are two different facts about the document and the \
             operator is shown a different sentence for each, so refusing for the wrong one \
             tells him the wrong thing about his own drawing. If the aim is what moved, the \
             spans are in `fixtures/inherited-runs.PROVENANCE.md`. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "dragging {what} was refused, for the reason under test: `{}`",
        line.raw
    ));

    // --- b: ★★★ THE O188 DEFECT ITSELF. Did the refusal raise a sentence? ---
    let at_decline = line.lineno;
    let found = after
        .events(RECORDED_EVENT)
        .find(|l| l.lineno > at_decline && l.get("what") == Some(recorded));
    if found.is_none() {
        let other = after
            .events(RECORDED_EVENT)
            .find(|l| l.lineno > at_decline)
            .map(|l| l.raw.clone());
        return Ok(Some(format!(
            "★★★ THE DEFECT: the drag on {what} was refused and the operator was told \
             NOTHING. The refusal is correct and is not the bug — the engine's own guard \
             says this move cannot be performed, and `canvas::moving::eligible` asks it \
             before the ghost slides, which is what keeps the preview truthful. **The bug \
             is the silence after it**: no `{RECORDED_EVENT} what={recorded}` line follows \
             `{}`, so `Refusal::worded` returned `None`, no `Action::DeclineOnCanvas` was \
             raised, and the status bar's decline slot was never given a sentence to draw. \
             {} This is O188 and it is the founding defect class of this project: the \
             operator drags, nothing moves, and nothing says why. Trace: {}.",
            line.raw,
            other.map_or_else(
                || "No decline of any kind was recorded after the refusal.".to_owned(),
                |raw| format!("A DIFFERENT decline was recorded instead: `{raw}`.")
            ),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "the refusal's sentence crossed the `Action` boundary and reached the store: \
         `{RECORDED_EVENT} what={recorded}`"
    ));

    // --- c: was it DRAWN? A sentence in a store nobody reads is silence. ----
    if driving::declared(after, ui_rect, DECLINE_REGION).is_none() {
        return Ok(Some(format!(
            "the refusal on {what} recorded its sentence and the status bar never drew it. \
             `{RECORDED_EVENT} what={recorded}` is in the trace, so the gesture refused and \
             the apply phase wrote the store — and no `{DECLINE_REGION}` region is on \
             screen on any later frame, which means `decline::live` filtered it out (a \
             `still_true` predicate that retires the sentence on the frame it is written) \
             or `decline::line` has no catalog entry for it. The operator sees exactly what \
             O188 reported: a drag that did nothing, in silence. Regions beginning \
             `status-group:` that ARE declared: {}. Trace: {}.",
            list(&driving::declared_names(after, ui_rect, "status-group:")),
            session.trace_path().display()
        )));
    }
    report.note(format!("★★ refused, worded and drawn, all three — {what}"));
    Ok(None)
}

/// Render a list of region names for a failure message, or say there were
/// none.
///
/// ★ A failure message that prints `[]` for an empty list makes the reader
/// wonder whether the query was wrong. Saying *none* answers that.
fn list(names: &[String]) -> String {
    if names.is_empty() {
        "none".to_owned()
    } else {
        names.join(", ")
    }
}
