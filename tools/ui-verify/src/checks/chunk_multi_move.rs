//! `shift_click_builds_a_chunk_set_the_whole_program_honours` — **O215 ask 4,
//! driven: several lines of one text block are selected together, move
//! together, and come back together on one press of undo.**
//!
//! # The request
//!
//! `OPERATOR_REQUESTS.md` **O215** ask 4 — *multi-select*. The operator holds
//! four labels of a note and expects the next drag to take all four, the way
//! every drawing program he uses takes them.
//!
//! # ★★★ The two halves, and why a check that drives only the first is worthless
//!
//! A multi-chunk selection was **representable and reachable** before any of
//! this: `SelectionState::pick_within` has pushed a Shift-clicked part in as
//! its own entry since the Part rung landed, and `normalise` collapses to the
//! Object rung only when entries differ by object or page. So the set could be
//! built by hand and the outlines drew. What no consumer did was *read* it —
//! the drag moved the first entry, and the status line said *1 line of 6* over
//! four outlined lines.
//!
//! ⇒ So this check is in two halves, and the second is the one with teeth:
//!
//! | half | gesture | what it would catch |
//! |---|---|---|
//! | the set is built | Shift-click a second chunk | `pick_within` replacing instead of pushing |
//! | the set is **honoured** | drag it, then undo it | a consumer reading `entries[0]` and moving one line of four |
//!
//! # ★★★ The gap press — the assertion the narrowing exists for
//!
//! Step G presses on the line **between** the two selected ones, which is not
//! selected, and drags.
//!
//! `Grabbable::bounds` at the Part rung is the union of the selected chunks'
//! outlines, so a set built from lines 0 and 2 spans line 1. A `covers`
//! predicate that asks only *"is the topmost object one of mine?"* answers yes
//! for a press anywhere in that span — the whole note is one object — so the
//! press is claimed as a move of the set and `presspick` never gets to re-pick
//! the line the operator aimed at. The operator's experience of that build:
//! he aims at line 1, drags, and lines 0 and 2 move while line 1 stays put.
//!
//! That is **O215 ask 1 failing through a multi-chunk selection**, and no other
//! step here can see it: every other press in this check lands on a chunk that
//! is already held, where the old predicate and the new one agree.
//!
//! # The oracle — four subsystems, and none of them is the canvas agreeing
//! with itself
//!
//! ```text
//! canvas      canvas-selection via=click mod=true sel=2 level=Part first=object:0 part=0
//! status bar  status-rung kind=text part=0 held=2 of=6
//! apply phase move-text-lines page=0 n=2 epoch=1 disclosures=none
//! history     undo kind=MoveTextRun undo_depth=1
//! press pick  selection-set page=0 object=0 part=1 level=part via=press
//! ```
//!
//! | line | question it answers |
//! |---|---|
//! | `canvas-selection mod=` | did the modifier reach the application, or was it a plain click? |
//! | `canvas-selection sel=` | did the Shift-click **add** rather than replace? |
//! | `status-rung held=` | does the surface that tells him what he is holding say **2**? |
//! | `move-text-lines` | did the PLURAL verb run — a singular one here is a set that moved one line |
//! | `undo … undo_depth=1` | did N engine calls fold into **one** undo entry? |
//! | `selection-set … via=press` | did the gap press re-pick the line under it? |
//!
//! ★★★ The last row is on a **different channel** from the first, and it has to
//! be. `canvas-selection` is written by the click path, which runs on the
//! release; a chunk chosen on the press and then dragged never produces a click
//! and never appears there. Reading the gap press off `canvas-selection` sees
//! silence, and reports the fix as missing on the build that has it.
//!
//! ★★ `move-text-lines` and `move-text-line` are distinct event names and the
//! trace matches them exactly, so *"the set moved"* and *"the first entry
//! moved"* cannot be confused. A build that collapsed the set mid-gesture
//! writes the singular line, and this check quotes it.
//!
//! ★★★ `undo_depth=` is read on the press **before** the pop, so it is the
//! depth the operator is acting on. Step C establishes that the log is empty
//! before the drag — without that control, `undo_depth=1` would be satisfied
//! by a build that left one entry of four behind and had four on the log all
//! along.
//!
//! # Fixture — pinned, and `--pdf` is ignored
//!
//! `fixtures/paragraph.pdf` through [`crate::fixture::text_chunk_point`]: one
//! text object of six lines on baselines 16 pt apart, every one of them
//! stating its own position — so the engine refuses none of them and a refusal
//! anywhere in this run is this shell's.
//!
//! ⚠ A missing fixture is a **FAIL**, not a SKIP: it is committed here, so its
//! absence is a broken checkout.
//!
//! # ⚠ What this check can see, and where its reach ends
//!
//! It reads what four subsystems wrote down, not the pixels. That both lines
//! *drew* an outline has one oracle — a rendered screenshot — and it is
//! `text_chunks`' subject rather than this one's.
//!
//! That the ghost travelled over both while the drag was in flight is
//! [`crate::checks::chunk_ghost`]'s, which reads the painter's own count of
//! the outlines it stroked. It is a separate row because the defect it names
//! lies entirely between what this check asserts — the set that was built, and
//! the move the release committed — and passed here undetected.
//!
//! # ⚠ HOW TO FALSIFY THIS CHECK — do this before believing a PASS
//!
//! Three plants, one per half. **Copy each file aside first** and restore from
//! the byte copy; never revert with git, because this project runs parallel
//! tracks and a chained revert discards another track's uncommitted work.
//!
//! 1. **The set is not built.** In `SelectionState::pick_within`, make the
//!    `shift` arm assign `self.entries = vec![entry]` like the plain one. Step
//!    D goes red on `sel=1`, and everything after it is unreachable.
//! 2. **The set is built and not honoured.** In `canvas::moving::eligible`,
//!    delete the `None if lines.len() > 1` arm of the page-object text branch.
//!    Step D still passes; step E goes red naming `move-text-line` — the
//!    singular verb over a selection of two, which is the defect in one line.
//! 3. **The narrowing.** In `canvas::pressing::body_under`, return `true` as
//!    soon as the topmost hit is one of the selection's objects — i.e. delete
//!    the Part-rung chunk test. Steps A–F stay green, because every press in
//!    them is on a held chunk. Step G goes red: the press in the gap is
//!    claimed as a move of the set, so the trace carries `move-text-lines`
//!    where a singular move of the aimed-at line was required.
//! 4. **Prove the plant is in the artifact.** `cargo build --release -p
//!    pdfcer-gui`, then confirm the exe is newer than the source: a stale
//!    binary is the commonest cause of a falsification that "did not
//!    reproduce", and its tell is an **absent** trace line rather than a wrong
//!    one.
//! 5. **Require the `[FAIL]` line**, not the exit code — a SKIP exits the way a
//!    PASS does.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::text_chunks::{
    EXPECTED_CHUNKS, MODE, PAGE_REGION, SELECTION_EVENT, Verdict, press_the_toggle, verdict,
};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, ScreenPoint};
use crate::error::{Error, Result};
use crate::input::{Driver, Key};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The rung one chunk — or a set of them — is selected at, as `canvas::trace`
/// spells it.
const PART_RUNG: &str = "Part"; // ui-text-exempt: a trace token, never displayed

/// The rung spelled the way `SelectionState::select_part` spells it.
const PART_RUNG_LOWER: &str = "part"; // ui-text-exempt: a trace token, never displayed

/// `selection-set page=… object=… part=… level=… via=…` —
/// `SelectionState::select_part`'s own line, naming the chunk it replaced the
/// entry list with.
///
/// ★★ The only channel on which a press-time re-pick is visible.
/// `canvas-selection` is written by the CLICK path, which runs on the release
/// and reports the selection the click left — so a chunk chosen on the press
/// and then dragged never appears there at all. A check that read
/// `canvas-selection` for the gap press would see silence and report the fix as
/// missing.
const SET_EVENT: &str = "selection-set"; // ui-text-exempt: a trace event name, never displayed

/// `via=` on the line `presspick::take` writes when it re-picks.
const VIA_PRESS: &str = "press"; // ui-text-exempt: a trace token, never displayed

/// `status-rung kind=… part=… held=… of=…` — `app::status::selected`, the
/// sentence the operator reads to learn what the next verb will act on.
const RUNG_EVENT: &str = "status-rung"; // ui-text-exempt: a trace event name, never displayed

/// `move-text-lines page=… n=… epoch=… disclosures=…` — the **plural** verb's
/// success line, written by the apply phase after the engine accepted it.
const MOVED_MANY_EVENT: &str = "move-text-lines"; // ui-text-exempt: a trace event name, never displayed

/// `move-text-line …` — the **singular** twin.
///
/// Read as a failure witness in the multi-chunk steps and as the required
/// answer in the gap-press step, which is the whole reason both names are
/// constants here rather than one being spelled inline.
const MOVED_ONE_EVENT: &str = "move-text-line"; // ui-text-exempt: a trace event name, never displayed

/// `canvas-move-declined level=… sel=… reason=… detail=…`, on release only.
///
/// Read only to improve a failure message: none of this fixture's lines can be
/// refused by the engine, so a refusal here names a rule this shell invented.
const MOVE_DECLINED_EVENT: &str = "canvas-move-declined"; // ui-text-exempt: a trace event name, never displayed

/// `undo kind=… undo_depth=…` — the history arm, naming what it is about to
/// take back and **how deep the log was when it did**.
const UNDO_EVENT: &str = "undo"; // ui-text-exempt: a trace event name, never displayed

/// `undo-applied page=… n=… epoch=… disclosures=…` — the engine's half.
const UNDO_APPLIED_EVENT: &str = "undo-applied"; // ui-text-exempt: a trace event name, never displayed

/// `undo-declined reason=empty-stack` — nothing on the log.
const UNDO_DECLINED_EVENT: &str = "undo-declined"; // ui-text-exempt: a trace event name, never displayed

/// The two chunks the set is built from, as positions in the fixture's content
/// stream.
///
/// Two apart, so the aims are 32 pt apart on a document whose baselines are
/// 16 pt apart and an aim off by a few points still lands on the intended line
/// — and, more importantly, so there is an **unselected** line between them for
/// step G to press on.
const PAIR: [usize; 2] = [0, 2];

/// The line between them, which is deliberately never selected.
const BETWEEN: usize = 1;

/// How far apart the two aims are, in PDF user-space points.
///
/// [`crate::fixture::text_chunk_point`] puts the fixture's baselines 16 pt
/// apart, and the pair is two lines apart. Quoted only in the message that
/// reports the two aims collapsing onto one chunk, where the number is what
/// tells a mapping fault from a selection one.
const AIM_SEPARATION_PT: usize = 32;

/// How far each drag travels, in window logical points, on each axis.
///
/// Comfortably past the drag threshold and past `Refusal::NoTravel`'s floor,
/// and small enough that the pointer stays well inside the canvas on a
/// 612 × 792 page at fit zoom.
const DRAG_PX: f32 = 40.0;

/// How deep the undo log must be when the first undo is pressed.
///
/// **One**, and that is the assertion, not a bookkeeping detail: the plural
/// move issues one `move_text_run` per run of every selected line, and
/// `fold_undo` coalesces them into a single entry. A build that skipped the
/// fold moves both lines and then needs one press of undo per line — which the
/// operator experiences as undo not working.
const EXPECTED_DEPTH: usize = 1;

/// What one gesture produced, read off `canvas-selection`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Picked {
    /// `level=` — which rung.
    rung: String,
    /// `sel=` — how many entries the selection holds.
    held: usize,
    /// `mod=` — whether the operator's modifier was held when it landed.
    modifier: bool,
    /// `part=` — which chunk the FIRST entry names, or `None` for `part=none`.
    part: Option<usize>,
    /// The whole line, so a failure quotes rather than describes.
    raw: String,
}

/// See the module documentation.
pub struct ShiftClickBuildsAChunkSetTheWholeProgramHonours;

impl Check for ShiftClickBuildsAChunkSetTheWholeProgramHonours {
    fn name(&self) -> &'static str {
        "shift_click_builds_a_chunk_set_the_whole_program_honours"
    }

    fn defect(&self) -> &'static str {
        "Shift-clicking a second line of a text block does not build a set the rest of the \
         program acts on — the drag moves one line of the several outlined, the status line \
         says one is held while several are, or the move needs one press of undo per line, so \
         the operator selects four labels and gets one"
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

/// What a step measured, or the sentence a FAIL should carry.
///
/// The outer `Result` is this harness's: its `Err` is a SKIP, *the check could
/// not run*. The inner one separates *the check ran and the assertion did not
/// hold* from *the check ran and here is the number it read* — three outcomes,
/// which is what a driven step actually has and what a bare `Option` cannot
/// spell.
type Step<T> = Result<std::result::Result<T, String>>;

/// Click — plainly or with Shift held — and read what the selection became.
///
/// `Ok(None)` is *the application wrote no `canvas-selection` line since the
/// mark taken here*. Every gesture in this check is meant to change the
/// selection, so that silence is a finding; it is returned rather than reported
/// so each step can say what it means where it happened.
fn click_and_read(
    session: &Session,
    driver: &Driver,
    at: ScreenPoint,
    shift: bool,
) -> Result<Option<Picked>> {
    let mark = session.trace()?.mark();
    if shift {
        driver.click_with_modifier(at, Key::Shift)?;
    } else {
        driver.click_at(at)?;
    }
    session.settle(26);
    let trace = session.trace()?;
    Ok(trace.last_after(SELECTION_EVENT, mark).map(|line| Picked {
        rung: line.get("level").unwrap_or("unstated").to_owned(),
        held: line.get_usize("sel").unwrap_or(0),
        modifier: line.get("mod") == Some("true"),
        part: line.get("part").and_then(|p| p.parse().ok()),
        raw: line.raw.clone(),
    }))
}

/// Ascend to the top of the ladder, then descend to the one chunk under `at`.
///
/// Returns the chunk index the descent reached. Two clicks, because the chunk
/// rung is entered on the second — `chunk_click` owns that claim and it is
/// assumed here rather than re-filed, but the RUNG is asserted: a set built at
/// the Object rung is a different selection and every assertion below would
/// then be about the wrong thing.
///
/// The leading Escapes make this callable both at the start of the run and
/// again after a committed edit, without inheriting a rung.
fn descend(session: &Session, driver: &Driver, at: ScreenPoint) -> Step<usize> {
    driver.press(vk::ESCAPE)?;
    session.settle(12);
    driver.press(vk::ESCAPE)?;
    session.settle(12);

    let Some(_first) = click_and_read(session, driver, at, false)? else {
        return Ok(Err(format!(
            "★★ THE FIRST CLICK SELECTED NOTHING: no `{SELECTION_EVENT}` line after a left \
             click inside the fixture's text. Without a block there is nothing to descend \
             into. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let Some(second) = click_and_read(session, driver, at, false)? else {
        return Ok(Err(format!(
            "★★ THE SECOND CLICK CHANGED NOTHING: the whole block is still selected after a \
             second left click on the same line, so the ladder never reached the chunk rung. \
             `clicking_a_chunk_selects_that_chunk` is the row that owns that descent and it \
             should be read first — this check cannot build a set of chunks on a build that \
             cannot select one. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if second.rung != PART_RUNG {
        return Ok(Err(format!(
            "★★ THE DESCENT DID NOT REACH ONE LINE: `{}` — expected `level={PART_RUNG}`. \
             Every assertion in this check is about a set of CHUNKS, and a selection standing \
             at `{}` is a set of objects.",
            second.raw, second.rung
        )));
    }
    let Some(part) = second.part else {
        return Ok(Err(format!(
            "the descent reached the chunk rung and named no chunk: `{}`. `part=none` at \
             `level={PART_RUNG}` is a selection standing inside something it cannot identify, \
             and every later comparison in this check is against that index.",
            second.raw
        )));
    };
    Ok(Ok(part))
}

/// Descend onto `aims[0]`, then Shift-add the chunk under `aims[1]`.
///
/// Returns the first chunk's index — the one `canvas-selection`'s `part=` names
/// for the rest of the gesture, since that field reports the FIRST entry.
fn build_the_pair(
    session: &Session,
    driver: &Driver,
    aims: &[ScreenPoint],
    report: &mut CheckReport,
) -> Step<usize> {
    let first = match descend(session, driver, aims[0])? {
        Ok(part) => part,
        Err(failure) => return Ok(Err(failure)),
    };
    report.note(format!("descended to chunk {first}"));

    let Some(pair) = click_and_read(session, driver, aims[1], true)? else {
        return Ok(Err(format!(
            "★★★ THE SHIFT-CLICK CHANGED NOTHING: no `{SELECTION_EVENT}` line after a \
             Shift-click on a different line of the same block, so the selection is exactly \
             what the plain click left it.\n\
             That is O215 ask 4 at its most direct — the operator holds one label, \
             Shift-clicks a second, and the program does not notice. \
             `canvas::presspick::press_selects` declines on Shift precisely so the CLICK path \
             owns the extend gesture; a build where neither owns it writes nothing here. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    if !pair.modifier {
        return Ok(Err(format!(
            "★★ THE MODIFIER DID NOT ARRIVE WITH THE CLICK: `{}` — `mod=false` on a click \
             this harness sent with Shift held.\n\
             Read before the rest: this is a claim about the HARNESS as much as about the \
             application. `Driver::click_with_modifier` holds the key down across the press \
             and the release, and a synthetic press that arrives between the key-down and the \
             application's next input pass is an unmodified click. Everything below would \
             then be measuring a plain second click and reporting it as a Shift-click.",
            pair.raw
        )));
    }
    if pair.rung != PART_RUNG {
        return Ok(Err(format!(
            "★★★ THE SHIFT-CLICK LEFT THE CHUNK RUNG: `{}` — expected `level={PART_RUNG}`.\n\
             `SelectionState::normalise` collapses to the Object rung when entries differ by \
             object or page, and both of these chunks are in one object on one page, so it \
             must not fire here. A build that ascends on a Shift-click has made the set \
             unbuildable: the operator's second click takes his first selection away.",
            pair.raw
        )));
    }
    if pair.held != 2 {
        return Ok(Err(format!(
            "★★★ THE DEFECT: THE SHIFT-CLICK REPLACED THE SELECTION INSTEAD OF ADDING TO IT \
             — `sel={}`, expected 2. Line: `{}`.\n\
             `SelectionState::pick_within` is the one place that decides this, and its `shift` \
             arm pushes the entry in — or removes it, if it was already there — where the plain \
             arm assigns a one-entry list. A `sel=1` here means the extend gesture is taking \
             the plain arm, so every chunk the operator adds costs him the one before it.",
            pair.held, pair.raw
        )));
    }
    report.note(format!(
        "★★★ the Shift-click ADDED a second chunk rather than replacing the first: `{}`",
        pair.raw
    ));
    Ok(Ok(first))
}

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is a run of left clicks, a Shift-click, \
             two press-move-release drags and a chord, and it needs the pointer and the \
             foreground. Reported as SKIPPED rather than passed: a check that did not run has \
             learned nothing.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    // ★ PINNED: `--pdf` and `--doc-point` are read and IGNORED. The header says
    // why the document has to be this one.
    let (pdf, _) = crate::fixture::text_chunk_point(PAIR[0]);
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the text fixture is not at {}. It is committed to this repository, so an absence \
             is a broken checkout rather than an unavailable precondition, and is reported as a \
             failure for that reason — a SKIP would say the opposite.",
            pdf.display()
        )));
    }
    report.note(format!(
        "--pdf and --doc-point are IGNORED: this check pins {} and builds a set from chunks \
         {PAIR:?} of its one six-line text object, pressing on chunk {BETWEEN} between them",
        pdf.display()
    ));
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new("could not read a page size from the fixture. Pass --page-size.")
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("chunk-multi-move.trace.txt"));
    spec.pdf = Some(pdf);
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
    session.settle(45);
    let driver = Driver::new(session.window());

    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "no `{PAGE_REGION}` region, so no sheet is on screen. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // --- the precondition: the chunk boxes must be ON -----------------------
    //
    // Not this check's subject, but its premise: the chunk rung is offered
    // exactly where a box is drawn, so a run that began with the switch off
    // would measure the switch and report it as a multi-selection defect. The
    // preference is persisted beside the exe, so a previous run that left it
    // off is a fact about the machine and not about the build.
    if verdict(&session.trace()?, 0) == Some(Verdict::Declined("switched-off".to_owned())) {
        report.note(
            "the chunk boxes were OFF at launch. Turning them on: the chunk rung is offered \
             only where a box is drawn, so this check has no subject without them.",
        );
        if let Some(failure) = press_the_toggle(&session, &driver, ui_rect, true, report)? {
            return Ok(Some(failure));
        }
    }

    // Every aim is converted BEFORE the first gesture, from one canvas rect. A
    // conversion taken between gestures would silently re-read a rect an edit
    // or a scroll had moved — and this check commits two edits.
    let mut aims = Vec::new();
    for index in [PAIR[0], PAIR[1], BETWEEN] {
        let (_, point) = crate::fixture::text_chunk_point(index);
        aims.push(aim(ctx, &session, page, point)?);
    }
    let frame = session.frame()?;

    // --- C: the control — the undo log is EMPTY -----------------------------
    //
    // ★★★ Step F asserts `undo_depth=1`, and that number means *the move added
    // exactly one entry* only if the log was empty before it. Established by
    // pressing undo and requiring a decline, because the depth is not published
    // any other way: `history_step` traces it, and `history_step` only runs
    // when the operator asks for a step.
    //
    // A worded decline lands on the status bar as a result. Harmless, and
    // deliberately not cleared: nothing below reads that slot.
    let mark = session.trace()?.mark();
    driver.press_chord(&[vk::CONTROL], vk::Z)?;
    session.settle(22);
    let trace = session.trace()?;
    if let Some(taken) = trace.last_after(UNDO_EVENT, mark) {
        return Err(Error::new(format!(
            "the document already has something on its undo log — `{}` — so this run cannot \
             measure whether the multi-chunk move adds exactly one entry. The fixture is \
             opened read-only and untouched by the steps above, so this is either a build \
             that records an edit for opening a file or a harness step that edited it. \
             SKIPPED rather than failed: the subject is a different one and it can no longer \
             be measured. Trace: {}.",
            taken.raw,
            session.trace_path().display()
        )));
    }
    if trace.last_after(UNDO_DECLINED_EVENT, mark).is_none() {
        return Err(Error::new(format!(
            "pressing undo on a freshly opened document wrote neither `{UNDO_EVENT}` nor \
             `{UNDO_DECLINED_EVENT}`, so the chord did not reach `history_step` at all and \
             this check cannot establish its control. `Ctrl+Z` is bound in every mode — it is \
             on no tab — so an absent line means the keystroke was swallowed before the \
             keymap. SKIPPED. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("control: the undo log is empty before anything is dragged");

    // --- which chunks the two aims actually reach ---------------------------
    //
    // ★★ Measured, never assumed. Step G asserts that the press in the gap
    // re-picked a chunk that is NOT one of the two held, and that comparison
    // needs both indices — `canvas-selection`'s `part=` reports the FIRST entry
    // only, so the Shift-added one is never on that line. One plain descent onto
    // the second aim is the cheapest honest way to learn it, and it costs two
    // clicks on a document nothing has edited yet.
    let chunk_b = match descend(&session, &driver, aims[1])? {
        Ok(part) => part,
        Err(failure) => return Ok(Some(failure)),
    };

    // --- A, B, D: descend, then Shift-add a second chunk --------------------
    let chunk_a = match build_the_pair(&session, &driver, &aims, report)? {
        Ok(part) => part,
        Err(failure) => return Ok(Some(failure)),
    };
    if chunk_a == chunk_b {
        return Err(Error::new(format!(
            "both aims reach chunk {chunk_a}, so there is no second chunk to add and no \
             unselected line between them. The two aims are {AIM_SEPARATION_PT} pt apart on a \
             document whose lines are 16 pt apart, so this is a mapping or a zoom that \
             collapsed them rather than a defect in the selection. SKIPPED: the subject \
             cannot be reached."
        )));
    }
    report.note(format!(
        "the two aims reach chunks {chunk_a} and {chunk_b}, with at least one unselected line \
         between them"
    ));

    // --- the surface that tells him what he is holding ----------------------
    //
    // ★★ Read with `last`, not `last_after`: `status-rung` goes through
    // `diag::trace_changed`, keyed on the rendered line, so the bar re-states
    // itself only when the clause CHANGES. `held=1` was written by the descent
    // and `held=2` by the Shift-click; the newest line is the current state,
    // which is what a check about disclosure wants.
    let trace = session.trace()?;
    let Some(rung) = trace.last(RUNG_EVENT) else {
        return Ok(Some(format!(
            "★★ THE STATUS BAR SAYS NOTHING ABOUT THE RUNG: no `{RUNG_EVENT}` line anywhere in \
             the run, although two chunks of a text block are selected. \
             `app::status::selected::with_part` emits it from the two producing arms, so its \
             absence means the bar drew no rung clause — the operator is holding a set and \
             the only surface that names what the next verb will act on is silent. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let held = rung.get_usize("held");
    let of = rung.get_usize("of");
    if held != Some(2) {
        return Ok(Some(format!(
            "★★★ THE BAR SAYS HE IS HOLDING {} LINE(S) AND HE IS HOLDING 2: `{}`.\n\
             Confidently wrong beats silent at nothing, and this is the wrong half: the \
             outlines say two, the next drag takes two, and the sentence says one. \
             `text::status::selection::selection_part_of_text` takes the count as an argument \
             precisely so it cannot be spelled `1` — a `held=1` here means its caller is \
             passing the FIRST entry's index space instead of the size of the set.",
            held.map_or_else(|| "an unstated number of".to_owned(), |h| h.to_string()),
            rung.raw
        )));
    }
    if of != Some(EXPECTED_CHUNKS) {
        return Ok(Some(format!(
            "the bar says the block has {} lines and this fixture's one text object has \
             {EXPECTED_CHUNKS}: `{}`. The set size is right and the total is not, so the \
             clause reads *2 lines of {}* — a denominator from somewhere other than the \
             object under the selection.",
            of.map_or_else(|| "an unstated number of".to_owned(), |o| o.to_string()),
            rung.raw,
            of.map_or_else(|| "?".to_owned(), |o| o.to_string())
        )));
    }
    report.note(format!(
        "★★ the status bar discloses the whole set, not the first entry: `{}`",
        rung.raw
    ));

    // --- E: drag the set ----------------------------------------------------
    //
    // From inside the FIRST chunk, which is held — so `covers` claims the press
    // and no re-pick is even attempted. That is the ordinary case and it is
    // deliberately not the interesting one; step G presses where the set is
    // not.
    let mark = session.trace()?.mark();
    driver.drag(aims[0], frame.offset_from(aims[0], DRAG_PX, DRAG_PX))?;
    session.settle(30);
    let trace = session.trace()?;

    if let Some(one) = trace.last_after(MOVED_ONE_EVENT, mark) {
        return Ok(Some(format!(
            "★★★ THE DEFECT: THE SET WAS SELECTED AND ONE LINE MOVED — `{}`, the SINGULAR \
             verb, over a selection of two.\n\
             This is O215 ask 4 as the operator meets it: he Shift-clicks four labels, watches \
             four outline, drags, and one moves. `canvas::moving::eligible` chooses between \
             `MoveSubject::TextLine` and `MoveSubject::TextLines` on \
             `SelectionState::selected_parts_on(...).len()`, and a singular line here means it \
             read `entered_object` — the FIRST entry — instead of the set. Trace: {}.",
            one.raw,
            session.trace_path().display()
        )));
    }
    let Some(moved) = trace.last_after(MOVED_MANY_EVENT, mark) else {
        let declined = trace
            .last_after(MOVE_DECLINED_EVENT, mark)
            .map(|l| l.raw.clone());
        return Ok(Some(format!(
            "★★★ DRAGGING A SET OF TWO CHUNKS MOVED NOTHING: no `{MOVED_MANY_EVENT}` line \
             follows the release, so the edit never reached `EditSession::move_text_run` \
             through the funnel. {} Every line of this fixture states its own position, so \
             `text_run_move_refusal` answers `None` for all six and the engine would accept \
             the move — which means a refusal here is this shell's and not the document's. \
             Trace: {}.",
            declined.map_or_else(
                || "No `canvas-move-declined` line either, so the gesture did not reach the \
                    move rules at all — the press may have been claimed as a marquee, or the \
                    travel may have been below the drag threshold."
                    .to_owned(),
                |raw| format!("It was REFUSED instead: `{raw}`.")
            ),
            session.trace_path().display()
        )));
    };
    // `n=` is the number of show operators rewritten, which is one per RUN and
    // not one per line — so it is asserted as a floor rather than as an
    // equality. Two is the minimum a two-line set can produce, and one would
    // mean a set of two resolved to a single run.
    let pieces = moved.get_usize("n").unwrap_or(0);
    if pieces < 2 {
        return Ok(Some(format!(
            "the plural verb ran and rewrote {pieces} piece(s): `{}`. A set of two lines \
             resolves to at least two runs, so a lower count means \
             `ObjectModelProvider::text_line_runs` answered for one of them and not the other \
             — the set reached the apply phase and was spent on part of itself.",
            moved.raw
        )));
    }
    report.note(format!(
        "★★★ the drag moved the WHOLE set through the plural verb: `{}`",
        moved.raw
    ));

    // --- F: one press of undo, and the whole set comes back -----------------
    let mark = session.trace()?.mark();
    driver.press_chord(&[vk::CONTROL], vk::Z)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(undo) = trace.last_after(UNDO_EVENT, mark) else {
        return Ok(Some(format!(
            "★★ UNDO DID NOTHING AFTER THE MOVE: no `{UNDO_EVENT}` line, and {}. The move \
             committed a moment ago, so the log cannot be empty. Trace: {}.",
            trace.last_after(UNDO_DECLINED_EVENT, mark).map_or_else(
                || "no `undo-declined` line either, so the chord did not reach the keymap"
                    .to_owned(),
                |l| format!("it DECLINED instead: `{}`", l.raw)
            ),
            session.trace_path().display()
        )));
    };
    let depth = undo.get_usize("undo_depth");
    if depth != Some(EXPECTED_DEPTH) {
        return Ok(Some(format!(
            "★★★ THE MOVE LEFT {} ENTRIES ON THE UNDO LOG AND IT MUST LEAVE {EXPECTED_DEPTH}: \
             `{}`.\n\
             The log was measured empty before the drag, so every entry on it now was put \
             there by the one gesture. `fold_undo` coalesces the run of `move_text_run` calls \
             into a single `CommandKind::MoveTextRun` entry for exactly this reason: an \
             operator who moved four labels with one drag presses undo once, and a build that \
             needs four presses has made undo look broken on the gesture that most needs it.",
            depth.map_or_else(|| "an unstated number of".to_owned(), |d| d.to_string()),
            undo.raw
        )));
    }
    if trace.last_after(UNDO_APPLIED_EVENT, mark).is_none() {
        return Ok(Some(format!(
            "★★ THE UNDO WAS DECIDED AND NEVER APPLIED: `{}` with no `{UNDO_APPLIED_EVENT}` \
             line after it. The two lines are written by two subsystems — the history arm \
             decides and names the kind, `vector_edit` bumps the epoch and drops the page \
             texture — and a build that writes the first alone has restored nothing the \
             operator can see. Trace: {}.",
            undo.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ one press of undo took the whole set back, off one log entry: `{}`",
        undo.raw
    ));

    // --- G: the gap press ---------------------------------------------------
    //
    // ★★★ The step the Part-rung narrowing in `pressing::body_under` exists
    // for, and the only one that can see it: every press above lands on a chunk
    // that is already held, where the object-granular predicate and the
    // chunk-granular one agree.
    //
    // The set is rebuilt from scratch rather than assumed to have survived the
    // undo. An epoch bump invalidates every epoch-keyed cache in the shell, and
    // what a selection does across one is not this check's subject — asserting
    // it here would file a cache question as a multi-selection defect.
    if let Err(failure) = build_the_pair(&session, &driver, &aims, report)? {
        return Ok(Some(failure));
    }
    let mark = session.trace()?.mark();
    driver.drag(aims[2], frame.offset_from(aims[2], DRAG_PX, DRAG_PX))?;
    session.settle(30);
    let trace = session.trace()?;

    if let Some(many) = trace.last_after(MOVED_MANY_EVENT, mark) {
        return Ok(Some(format!(
            "★★★ THE DEFECT: A PRESS ON AN UNSELECTED LINE MOVED THE SELECTED ONES INSTEAD — \
             `{many}`.\n\
             Two chunks were held with one unselected line between them, and the drag began \
             inside that unselected line. `Grabbable::bounds` at the chunk rung is the UNION \
             of the held outlines, so it spans the gap; a `covers` predicate that asks only \
             whether the topmost object is one of the selection's answers yes for the whole \
             block and claims the press as a move of the set. \
             `canvas::pressing::body_under` narrows that question to the chunk under the \
             point for exactly this reason. What the operator sees on the failing build: he \
             aims at one label, drags, and two OTHER labels move while the one under his \
             pointer stays where it was. Trace: {trace}.",
            many = many.raw,
            trace = session.trace_path().display()
        )));
    }
    let Some(one) = trace.last_after(MOVED_ONE_EVENT, mark) else {
        let declined = trace
            .last_after(MOVE_DECLINED_EVENT, mark)
            .map(|l| l.raw.clone());
        return Ok(Some(format!(
            "★★ THE PRESS IN THE GAP MOVED NOTHING AT ALL: neither `{MOVED_ONE_EVENT}` nor \
             `{MOVED_MANY_EVENT}` follows the release. {} The narrowing is meant to hand the \
             press back to `presspick::take`, which re-picks the chunk under it and leaves the \
             drag a singular move — a build that instead declines the press has traded one \
             defect for a gesture that does nothing. Trace: {}.",
            declined.map_or_else(
                || "No `canvas-move-declined` line either.".to_owned(),
                |raw| format!("It was REFUSED: `{raw}`.")
            ),
            session.trace_path().display()
        )));
    };
    // ★★★ The re-pick is read on `selection-set`, NOT on `canvas-selection`.
    //
    // `canvas-selection` is written by the click path, which runs on the
    // RELEASE. A chunk chosen on the press and dragged never produces a click
    // at all, so that line is silent for this gesture — and a check that read
    // it here would report the fix as missing on the build that has it.
    // `SelectionState::select_part` writes `selection-set … via=press`, which is
    // `presspick::take`'s own account of what it did.
    let Some(repick) = trace
        .events(SET_EVENT)
        .filter(|l| l.lineno > mark && l.get("via") == Some(VIA_PRESS))
        .last()
    else {
        return Ok(Some(format!(
            "★★★ THE PRESS IN THE GAP MOVED ONE LINE WITHOUT RE-PICKING IT — `{}` with no \
             `{SET_EVENT} … via={VIA_PRESS}` line before it.\n\
             So the line that moved was chosen by something other than the selection the \
             operator can see, and the outlines on screen still say two chunks are held while \
             a third one moved. `presspick::take` is the only caller that writes that line, \
             and it runs only when `covers` declined the press — an absent line means the \
             press was claimed as a move of the set and the singular verb came from \
             somewhere else entirely. Trace: {}.",
            one.raw,
            session.trace_path().display()
        )));
    };
    if repick.get("level") != Some(PART_RUNG_LOWER) {
        return Ok(Some(format!(
            "the press re-picked at `{}` and the chunk rung is `{PART_RUNG_LOWER}`: `{}`. A \
             re-pick that lands on another rung has answered a different question from the \
             one the operator asked by aiming at a line.",
            repick.get("level").unwrap_or("an unstated rung"),
            repick.raw
        )));
    }
    let Some(picked) = repick.get_usize("part") else {
        return Ok(Some(format!(
            "the press re-picked and named no chunk: `{}`. Without the index there is nothing \
             to compare against the two that were held, which is the whole assertion of this \
             step.",
            repick.raw
        )));
    };
    // ★★ Compared against the two measured indices, never pinned to a literal.
    // Which index the provider gives a line is the provider's business; pinning
    // one here would make a legitimate change of line granularity read as a
    // selection defect.
    if picked == chunk_a || picked == chunk_b {
        return Ok(Some(format!(
            "the press in the gap re-picked chunk {picked}, which is one of the two that were \
             already held ({chunk_a} and {chunk_b}): `{}`.\n\
             The aim was meant to land on an unselected line between them, so either the \
             chunk boxes have moved under the aims or `chunks::under` is answering with a \
             neighbour. Either way this step is no longer pressing where it thinks it is, and \
             it can no longer tell the narrowing from its absence.",
            repick.raw
        )));
    }
    report.note(format!(
        "★★★ the press in the gap re-picked chunk {picked} — neither of the two held — and \
         moved that one alone: `{}` then `{}`",
        repick.raw, one.raw
    ));
    Ok(None)
}
