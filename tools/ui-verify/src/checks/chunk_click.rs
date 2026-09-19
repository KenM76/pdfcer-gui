//! `clicking_a_chunk_selects_that_chunk` — **O215 asks 1 and 6, driven: the
//! left button alone reaches one line of a text block, and it reaches the same
//! one every time.**
//!
//! # The request
//!
//! `OPERATOR_REQUESTS.md` **O215**, in his words:
//!
//! > *"…then let us use our usual mouse selection methods to move the chunks."*
//!
//! with ask 1 — *selecting one chunk is repeatable* — and ask 6 — *all on the
//! left button, the right-click route stays*. Ask 3 (the boxes) is
//! [`crate::checks::text_chunks`]; this is the gesture the boxes made aimable.
//!
//! # ★★★ What "repeatable" means here, and why it needs three clicks
//!
//! His report is that the same gesture *"sometimes takes the whole block"*. So
//! the claim under test is not *a chunk can be selected* — a unit test can say
//! that — but **the same point selects the same chunk on a later visit**. That
//! is a statement about two clicks separated by a third, and it cannot be
//! measured with fewer:
//!
//! | clicks | what it would measure |
//! |---|---|
//! | one | that something was selected |
//! | the same point twice | nothing: the state did not change, and the trace collapses (below) |
//! | A, B, A | that A is reachable, that B is a *different* chunk, and that A comes back |
//!
//! ★★ `canvas-selection` is written through `diag::trace_changed` under its own
//! slot, so **an identical repeat writes nothing**. A check that clicked one
//! chunk twice would read silence and could not tell *it selected the same
//! chunk again* from *the second click did nothing at all* — the two verdicts
//! this whole row exists to separate. Alternating is what makes the channel
//! answer.
//!
//! # ★★ The chunk index is compared, never assumed
//!
//! The check never asserts *the click on the top line selects chunk 0*. Which
//! index the provider gives a line is the provider's business, and pinning it
//! here would make a legitimate change of line granularity look like a
//! selection defect. What is asserted is the shape the operator experiences:
//! two aim points give two **different** indices, and returning to the first
//! gives back the **first** index.
//!
//! # ★ Ask 6 — the left button, and only the left button
//!
//! Every gesture below is [`Driver::click_at`], a plain left click with no
//! modifier. A build where the chunk is reachable only through the context
//! menu passes `chunk_boxes_show_what_a_text_block_is_made_of` and fails here,
//! which is the distinction ask 6 makes.
//!
//! # ★★ What the first click must NOT do
//!
//! It must select the **block**, not a chunk. A build that descended on first
//! contact would make dragging a whole text block unreachable while the boxes
//! are on — a capability regression (**R6**) traded for the one being added, so
//! it is asserted rather than left to be discovered on a CAD sheet.
//!
//! # Fixture — pinned, and `--pdf` is ignored
//!
//! `fixtures/paragraph.pdf` through [`crate::fixture::text_chunk_point`]: one
//! text object of six lines on baselines 16 pt apart. Chunks **0 and 2** are
//! aimed at, 32 pt apart, so an aim off by a few points still lands on the
//! intended line rather than its neighbour.
//!
//! ⚠ A missing fixture is a **FAIL**, not a SKIP: it is committed here, so its
//! absence is a broken checkout.
//!
//! # ⚠ What this check can see, and where its reach ends
//!
//! It reads the selection the application reports, not the pixels. A build
//! whose chunk outline is drawn in the page colour selects correctly and shows
//! nothing; that has one oracle, a rendered screenshot, and it is
//! `text_chunks`'s subject rather than this one's.
//!
//! # ⚠ HOW TO FALSIFY THIS CHECK — do this before believing a PASS
//!
//! 1. **Copy the file aside first**: `crates/pdfcer-gui/src/canvas/selection/mod.rs`
//!    to the scratch directory. **Never revert it with git** — this project
//!    runs parallel tracks and a chained revert discards another track's
//!    uncommitted work; restore from the byte copy.
//! 2. **Plant the defect the operator reported**: in `click_at_object_rung`,
//!    change the `hit.chunk` term of the narrowing condition to `false`. Every
//!    unit test but two stays green, the boxes still draw, and step B goes red
//!    — the second click leaves the selection at the Object rung, which is
//!    *"it took the whole block"*.
//! 3. **A second plant, for steps C and D**: in `canvas::presspick::take`,
//!    delete the `select_part` call and answer `false`. The first descent still
//!    works through the click path; moving between chunks stops, so a run reads
//!    `part=` frozen at its first value.
//! 4. **A third, for the first-click rule**: make `canvas::clicking` set
//!    `chunk: true` unconditionally. Step A goes red on `level=Part` — the
//!    block became undraggable the moment the boxes were switched on.
//! 5. **Prove the plant is in the artifact.** `cargo build --release -p
//!    pdfcer-gui`, then confirm the exe is newer than the source: a stale
//!    binary is the commonest cause of a falsification that "did not
//!    reproduce", and its tell is an **absent** trace line rather than a wrong
//!    one.
//! 6. **Require the `[FAIL]` line**, not the exit code — a SKIP exits the way a
//!    PASS does.
//! 7. **Restore from the byte copy**, rebuild, confirm the PASS returns.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::text_chunks::{
    DECLINED_EVENT, DRAWN_EVENT, EXPECTED_CHUNKS, MODE, PAGE_REGION, SELECTION_EVENT, Verdict,
    press_the_toggle, verdict,
};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, ScreenPoint};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The rung a whole object is selected at, as `canvas::trace` spells it.
const OBJECT_RUNG: &str = "Object"; // ui-text-exempt: a trace token, never displayed

/// The rung one chunk is selected at.
const PART_RUNG: &str = "Part"; // ui-text-exempt: a trace token, never displayed

/// Which chunk of the fixture each click aims at, in the order clicked.
///
/// **A, B, A** — the alternation the header argues for. The first two entries
/// are two chunks apart so an aim off by a few points still lands on the
/// intended line.
const AIMS: [usize; 3] = [0, 2, 0];

/// How far apart, in points, the two aim points are.
///
/// Stated rather than recomputed in a message: it is the number that makes
/// *the two lines report the same chunk* a finding about the hit test rather
/// than about the aim.
const AIM_SEPARATION_PT: usize = 32;

/// What one click produced, read off `canvas-selection`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Picked {
    /// `level=` — which rung.
    rung: String,
    /// `part=` — which chunk, or `None` for `part=none`.
    part: Option<usize>,
    /// `first=` — which object, so a click that left the block is visible.
    object: String,
    /// The whole line, so a failure quotes rather than describes.
    raw: String,
}

pub struct ClickingAChunkSelectsThatChunk;

impl Check for ClickingAChunkSelectsThatChunk {
    fn name(&self) -> &'static str {
        "clicking_a_chunk_selects_that_chunk"
    }

    fn defect(&self) -> &'static str {
        "a left click inside a text block cannot reach one line of it, or reaches a different \
         line each visit — so the operator aims at a box he can see and the gesture takes the \
         whole block instead, which is what O215 reports"
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

/// Click once and read what the selection became.
///
/// `Ok(None)` means the application wrote no `canvas-selection` line since the
/// mark taken here — which, because every click in this check is meant to
/// change the selection, is itself a finding. It is returned rather than
/// reported so each step can say what that silence means where it happened.
fn click_and_read(session: &Session, driver: &Driver, at: ScreenPoint) -> Result<Option<Picked>> {
    let mark = session.trace()?.mark();
    driver.click_at(at)?;
    session.settle(26);
    let trace = session.trace()?;
    Ok(trace.last_after(SELECTION_EVENT, mark).map(|line| Picked {
        rung: line.get("level").unwrap_or("unstated").to_owned(),
        part: line.get("part").and_then(|p| p.parse().ok()),
        object: line.get("first").unwrap_or("unstated").to_owned(),
        raw: line.raw.clone(),
    }))
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is four left clicks on the page and it \
             needs the pointer and the foreground. Reported as SKIPPED rather than passed: a \
             check that did not run has learned nothing.",
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
    let (pdf, _) = crate::fixture::text_chunk_point(AIMS[0]);
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the text fixture is not at {}. It is committed to this repository, so an absence is \
             a broken checkout rather than an unavailable precondition, and is reported as a \
             failure for that reason — a SKIP would say the opposite.",
            pdf.display()
        )));
    }
    report.note(format!(
        "--pdf and --doc-point are IGNORED: this check pins {} and aims at chunks {AIMS:?} of \
         its one six-line text object",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("chunk-click.trace.txt"));
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

    // --- The precondition: the boxes must be ON -----------------------------
    //
    // Not this check's subject — `text_chunks` owns that — but its premise: the
    // chunk rung is offered exactly where a box is drawn, so a run that began
    // with the switch off would measure the switch and report it as a selection
    // defect. The preference is persisted beside the exe, so a previous run
    // that left it off is a fact about the machine and not about the build.
    if verdict(&session.trace()?, 0) == Some(Verdict::Declined("switched-off".to_owned())) {
        report.note(
            "the chunk boxes were OFF at launch. Turning them on: the chunk rung is offered only \
             where a box is drawn, so this check has no subject without them.",
        );
        if let Some(failure) = press_the_toggle(&session, &driver, ui_rect, true, report)? {
            return Ok(Some(failure));
        }
    }

    // Every aim is converted BEFORE the first click, from one canvas rect. A
    // conversion taken between gestures would silently re-read a rect the
    // selection had changed the scroll of.
    let mut aims = Vec::new();
    for index in AIMS {
        let (_, point) = crate::fixture::text_chunk_point(index);
        aims.push(aim(ctx, &session, page, point)?);
    }

    // --- A: the FIRST click names the block, not a chunk --------------------
    let Some(first) = click_and_read(&session, &driver, aims[0])? else {
        return Ok(Some(format!(
            "★★★ THE CLICK SELECTED NOTHING: no `{SELECTION_EVENT}` line after a left click on \
             the first line of the fixture's text. The document was launched with this file and \
             the canvas is drawing a page, so either the click did not reach the canvas or the \
             hit test found nothing under a point that is inside the glyphs. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if first.rung != OBJECT_RUNG {
        return Ok(Some(format!(
            "★★★ THE FIRST CLICK WENT STRAIGHT INSIDE THE BLOCK: `{}`, where the first click \
             must select the whole text object.\n\
             A build that descends on first contact has made dragging a whole block unreachable \
             while the boxes are on — the capability O215 asks to ADD has been traded for one \
             that already worked (R6). The descent belongs on the second click, which is where \
             the operator has already said which block he means.",
            first.raw
        )));
    }
    let block = first.object.clone();
    report.note(format!(
        "★ the first click named the block: `{}`",
        first.raw
    ));

    // The boxes have to be on the page before the next click, or the operator
    // had nothing to aim at. Asserted rather than assumed: this check's premise
    // is that the rung is offered where a box is.
    match verdict(&session.trace()?, 0) {
        Some(Verdict::Drawn { count, .. }) if count == EXPECTED_CHUNKS => {}
        other => {
            return Ok(Some(format!(
                "★★ THE BLOCK IS SELECTED AND ITS CHUNKS ARE NOT DRAWN: the painter's answer is \
                 {}, and this fixture's one text object holds {EXPECTED_CHUNKS} chunks. This \
                 check's subject — clicking a box you can see — does not exist on this build, \
                 and `chunk_boxes_show_what_a_text_block_is_made_of` is the row that says why. \
                 Trace: {}.",
                other.map_or_else(
                    || format!("silence — neither `{DRAWN_EVENT}` nor `{DECLINED_EVENT}`"),
                    |v| format!("`{v}`")
                ),
                session.trace_path().display()
            )));
        }
    }

    // --- B: the SECOND click, same point, descends to that chunk ------------
    let Some(descended) = click_and_read(&session, &driver, aims[0])? else {
        return Ok(Some(format!(
            "★★★ THE SECOND CLICK CHANGED NOTHING — THE WHOLE BLOCK IS STILL SELECTED: no new \
             `{SELECTION_EVENT}` line after a second left click on the same line of text, so the \
             selection is exactly what the first click left.\n\
             This is the operator's report reached from the measurable side: he aims at a box he \
             can see and the gesture keeps the block. `ClickHit::chunk` carries the permission to \
             narrow, `canvas::clicking` is the only place that sets it, and a build where it is \
             never true selects the block forever. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if descended.rung != PART_RUNG {
        return Ok(Some(format!(
            "★★★ THE SECOND CLICK DID NOT GO INSIDE THE BLOCK: `{}` — expected \
             `level={PART_RUNG}`.\n\
             The selection changed and stayed at the outer rung, which is a click that re-picked \
             the same object rather than one that descended into it. Trace: {}.",
            descended.raw,
            session.trace_path().display()
        )));
    }
    let Some(chunk_a) = descended.part else {
        return Ok(Some(format!(
            "★★ THE RUNG SAYS INSIDE AND NOTHING SAYS WHICH: `{}` — `level={PART_RUNG}` with \
             `part=none`. A selection standing at the Part rung with no part is the state \
             `SelectionState::normalise` exists to prevent, and every verb taking a chunk as its \
             operand would have nothing to act on. Trace: {}.",
            descended.raw,
            session.trace_path().display()
        )));
    };
    if descended.object != block {
        return Ok(Some(format!(
            "★★ THE DESCENT LANDED IN A DIFFERENT OBJECT: the first click selected `{block}` and \
             the second reports `{}`. Two clicks at one point must reach one object, so the hit \
             test is answering differently on the second visit. Line: `{}`.",
            descended.object, descended.raw
        )));
    }
    report.note(format!(
        "★★★ the second click descended to one chunk: `{}`",
        descended.raw
    ));

    // --- C: a click on a DIFFERENT chunk moves to that one ------------------
    let Some(moved) = click_and_read(&session, &driver, aims[1])? else {
        return Ok(Some(format!(
            "★★★ THE SELECTION IS STUCK ON ONE CHUNK: a left click on a different line of the \
             same block wrote no new `{SELECTION_EVENT}` line, so the selection is still chunk \
             {chunk_a}.\n\
             Once inside a block every further click re-picks — `SelectionState::click_inside` \
             is where that is defined. A build that descends once and then freezes gives the \
             operator one chunk per block and a trip out through Escape to reach the next. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let Some(chunk_b) = moved.part.filter(|_| moved.rung == PART_RUNG) else {
        return Ok(Some(format!(
            "★★★ A CLICK ON A NEIGHBOURING CHUNK LEFT THE BLOCK: `{}` — expected \
             `level={PART_RUNG} part=` naming the other line.\n\
             Ascending on a click that landed on a sibling chunk is exactly the report *it takes \
             the whole block*: `canvas::presspick` decides on the PRESS whether the pointer is \
             inside the current selection, and a build that answers no and then re-selects the \
             object has thrown away the rung the operator was standing on. Trace: {}.",
            moved.raw,
            session.trace_path().display()
        )));
    };
    if chunk_b == chunk_a {
        return Ok(Some(format!(
            "★★★ TWO DIFFERENT LINES ARE THE SAME CHUNK: clicking line {} of the fixture and \
             then line {} both report `part={chunk_a}`.\n\
             The two aim points are {AIM_SEPARATION_PT} pt apart on a document whose baselines \
             are 16 pt apart, so this is not an aim that missed. Either the hit test answers one \
             index whatever it is given, or the provider is grouping the block as a single line \
             — the second is the shape `ENGINE_BACKLOG.md` G032 reports from the other side. \
             Line: `{}`.",
            AIMS[0], AIMS[1], moved.raw
        )));
    }
    report.note(format!(
        "★★ a click on a neighbouring line moved to a DIFFERENT chunk: `{}`",
        moved.raw
    ));

    // --- D: …and going back gets the FIRST chunk back -----------------------
    //
    // The row in one assertion. The trace is a change log, so this line exists
    // only because the state actually moved back.
    let Some(again) = click_and_read(&session, &driver, aims[2])? else {
        return Ok(Some(format!(
            "★★★ THE FIRST CHUNK COULD NOT BE SELECTED A SECOND TIME: returning to the point \
             that selected chunk {chunk_a} wrote no new `{SELECTION_EVENT}` line, so the \
             selection is still chunk {chunk_b}.\n\
             That is O215 ask 1 failing directly — the same point does not select the same chunk \
             on a later visit, which is what makes the gesture feel like a coin toss. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if again.rung != PART_RUNG || again.part != Some(chunk_a) {
        return Ok(Some(format!(
            "★★★ THE SAME POINT SELECTED A DIFFERENT CHUNK: the first visit reported \
             `part={chunk_a}` and the second reports `{}`.\n\
             Nothing about the document changed between them — same page, same zoom, same \
             pointer position — so the answer depends on what was selected BEFORE the click. \
             That is the non-determinism O215 ask 1 names, and it is the defect this row exists \
             to hold. Trace: {}.",
            again.raw,
            session.trace_path().display()
        )));
    }
    if again.object != block {
        return Ok(Some(format!(
            "★★ THE RETURN LANDED IN A DIFFERENT OBJECT: `{}`, where the block is `{block}`. The \
             chunk index matches by coincidence, inside something else.",
            again.raw
        )));
    }
    report.note(format!(
        "★★★ …and coming back selected the SAME chunk again: `{}` — the gesture is repeatable, \
         on the left button alone",
        again.raw
    ));
    Ok(None)
}
