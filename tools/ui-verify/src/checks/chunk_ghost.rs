//! `dragging_a_chunk_shows_where_it_is_going` — **O215 ask 5: dragging a line
//! of a note previews where it will land, and dragging several previews all of
//! them.**
//!
//! # The request
//!
//! `OPERATOR_REQUESTS.md` **O215** ask 5 asks for a live preview — *"the chunk
//! follows the pointer"*. The full ask is the glyphs travelling; this row
//! measures the floor beneath it, which is that **anything at all** moves on
//! screen while the pointer does.
//!
//! # The defect it was written against
//!
//! `overlay::draw_move_ghost` opened with `if !outline { return; }`, and
//! `canvas::pressing::grabbable` sets that flag only at the **object** rung. So
//! at the chunk rung the ghost was withheld, `draw_selection` was gated on the
//! same flag so no outline was drawn either, and `painting::draw_chunks` paints
//! the chunk boxes at their *undisplaced* positions. The operator's experience
//! of dragging a line of a note was therefore that **nothing whatsoever moved**
//! until he let go.
//!
//! ★★★ The gate was not arbitrary. `OPERATOR_REQUESTS.md` **O63** is *"it just
//! had a perimeter box around it"* — dragging a path **node** must not draw a
//! perimeter box, because `ShapePreview` already shows the real anchors
//! travelling and a box on top of that is noise. The correct condition is
//! therefore *is the real geometry already travelling*, not *is this an inner
//! rung*. Those two coincided exactly until a text chunk became selectable, and
//! a text chunk is an inner rung with **no path geometry at all**.
//!
//! # The oracle — and the negative that names the defect exactly
//!
//! ```text
//! fixed build   canvas-move-ghost boxes=3 rung=part suppressed=no
//! defect        canvas-move-ghost boxes=0 rung=part suppressed=o63
//! ```
//!
//! Both were measured by driving the release binary. They are mutually
//! exclusive and one field apart, so a failure here quotes the defect rather
//! than describing it.
//!
//! | field | question it answers |
//! |---|---|
//! | `boxes=` | did the painter draw anything? It is the painter's claim about itself, counted in the loop that strokes |
//! | `rung=` | read from `SelectionState::level`, **not** from the flag being tested — a field that restated its own gate could not witness the gate being wrong |
//! | `suppressed=` | `o63` names the one correct withholding; `no` is every other frame |
//!
//! ★★ `boxes=` is asserted against `held=` on `status-rung`, because a preview
//! of *one* box while *three* lines are held is the ask failing in the way the
//! operator would actually meet it — he sweeps three labels, drags, and watches
//! one of them move.
//!
//! # Where this check's reach ends
//!
//! It reads what the painter wrote down, not the pixels. That the outline is
//! **visible** — not clipped, not drawn in the backdrop colour — has one oracle,
//! a rendered screenshot, and it is not this row's subject.
//!
//! It does not drive the object rung either. `overlay::ghost_is_owed` answers
//! `true` whenever `outline` is, so the narrowing cannot reach the object rung
//! at all: that arm is additive by the shape of the boolean rather than by
//! measurement, and a driven assertion would be theatre. Its truth table is
//! pinned by that function's own test.
//!
//! The `suppressed=o63` arm needs a **path node** under the pointer, which this
//! fixture has none of; it is listed below as a plant instead.
//!
//! # Fixture — pinned, and `--pdf` is ignored
//!
//! `fixtures/paragraph.pdf`, six lines of one text object, baselines 16 pt
//! apart. The geometry table lives in
//! [`crate::checks::chunk_band`]'s header, which is where the bands were
//! derived; [`WIDE_BAND`] below is that check's wide band verbatim, and it
//! reaches lines 0, 1 and 2.
//!
//! ⚠ A missing fixture is a **FAIL**, not a SKIP: it is committed here, so its
//! absence is a broken checkout rather than an unavailable precondition.
//!
//! # ⚠ HOW TO FALSIFY THIS CHECK — do this before believing a PASS
//!
//! **Copy each file aside first** and restore from the byte copy; never revert
//! with git, because this project runs parallel tracks and a chained revert
//! discards another track's uncommitted work.
//!
//! 1. **Restore the defect.** Make `overlay::ghost_is_owed` answer `outline`
//!    alone. Steps C and F both go red quoting
//!    `boxes=0 … suppressed=o63`.
//!
//!    ★★★ `a_rubber_band_inside_a_note_takes_its_lines` **passed under that
//!    build**, and so did every other row in the roster. It asserts what the
//!    band selected and what the release committed, and the defect lies
//!    entirely between the two. That is why this row is owed and why its
//!    absence was not visible as a gap.
//! 2. **Break the count without breaking the draw.** Make `boxes` in
//!    `draw_move_ghost` a constant `1`. Step C stays green — one line is held —
//!    and step F goes red on `boxes=1` where `held=3`. The two arms are
//!    separately falsifiable because they hold different numbers of lines.
//! 3. **Break the rung field.** Make `part_rung` in `draw_move_ghost` read
//!    `!outline` instead of `selection.level()`. Every step here stays green
//!    today, which is the point: the two agree on this build. Then apply plant
//!    1 on top and step C reports `rung=object` — a defective build describing
//!    itself as a healthy one. Restore plant 3 before believing anything.
//! 4. **Prove the O63 arm still suppresses.** Select a path node on a vector
//!    drawing and drag it; `canvas-move-ghost boxes=0 … suppressed=o63` must
//!    appear. This check cannot drive that — its fixture has no path — so the
//!    arm is guarded by `overlay`'s
//!    `the_ghost_is_withheld_only_for_a_preview_that_has_something_in_it`,
//!    whose middle row is the trap: a preview that EXISTS and is empty, which
//!    is what a text object produces, must not count as geometry travelling.
//! 5. **Prove the plant is in the artifact.** `cargo build --release -p
//!    pdfcer-gui` AND `-p ui-verify`, then confirm the exe is newer than the
//!    source: a stale binary is the commonest cause of a falsification that
//!    "did not reproduce", and its tell is an **absent** trace line rather than
//!    a wrong one.
//! 6. **Require the `[FAIL]` line**, not the exit code — a SKIP exits the way a
//!    PASS does.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::text_chunks::{
    MODE, PAGE_REGION, SELECTION_EVENT, Verdict, press_the_toggle, verdict,
};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry, ScreenPoint, WindowFrame};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// `canvas-move-ghost boxes=… rung=… suppressed=…` — `overlay::draw_move_ghost`'s
/// own account of what it drew, written on the frames a move is in flight.
const GHOST_EVENT: &str = "canvas-move-ghost"; // ui-text-exempt: a trace event name, never displayed

/// The rung spelled the way `canvas::trace` spells it on `canvas-selection`.
const PART_RUNG: &str = "Part"; // ui-text-exempt: a trace token, never displayed

/// The rung spelled the way `canvas::trace::move_ghost` spells it.
const PART_RUNG_LOWER: &str = "part"; // ui-text-exempt: a trace token, never displayed

/// `suppressed=` on a frame where the ghost was drawn.
const NOT_SUPPRESSED: &str = "no"; // ui-text-exempt: a trace token, never displayed

/// `suppressed=` on the one frame where withholding is correct — a path node,
/// whose real anchors are already travelling.
const SUPPRESSED_O63: &str = "o63"; // ui-text-exempt: a trace token, never displayed

/// `marquee-parts page=… object=… mode=… reached=… kept=… combine=…` —
/// `marquee::take_chunks`' own line, read here only to establish step E's set.
const BAND_PARTS_EVENT: &str = "marquee-parts"; // ui-text-exempt: a trace event name, never displayed

/// `status-rung kind=… part=… held=… of=…` — the sentence that names what the
/// next verb will act on, and the source of the count `boxes=` is measured
/// against.
const RUNG_EVENT: &str = "status-rung"; // ui-text-exempt: a trace event name, never displayed

/// `move-text-line …` — the singular verb, read as evidence the preview was of
/// a move that could actually happen.
const MOVED_ONE_EVENT: &str = "move-text-line"; // ui-text-exempt: a trace event name, never displayed

/// `move-text-lines page=… n=… epoch=… disclosures=…` — the plural twin.
const MOVED_MANY_EVENT: &str = "move-text-lines"; // ui-text-exempt: a trace event name, never displayed

/// `canvas-move-declined level=… sel=… reason=… detail=…`, on release only.
const MOVE_DECLINED_EVENT: &str = "canvas-move-declined"; // ui-text-exempt: a trace event name, never displayed

/// The chunk step B descends to, and step C drags.
///
/// **Line 4, not line 0**, and the reason is step E: the band in step D sweeps
/// lines 0, 1 and 2, and its rectangle is stated in document space against
/// their *original* baselines. Dragging line 0 first would move the very lines
/// the band is aimed at, and the band would then reach a set nobody can state.
/// Line 4 is clear of the band by 5.2 pt below and 5.6 pt above.
const ANCHOR_CHUNK: usize = 4;

/// The band that takes lines 0, 1 and 2 — [`crate::checks::chunk_band`]'s
/// [`WIDE_BAND`](crate::checks::chunk_band) verbatim, and its header is where
/// the arithmetic is.
///
/// Right to left, so it is a crossing band. `(from, to)` in PDF user space on
/// page 0.
const WIDE_BAND: (DocPoint, DocPoint) = (
    DocPoint::new(0, 400.0, 730.0),
    DocPoint::new(0, 60.0, 666.0),
);

/// How many lines [`WIDE_BAND`] reaches, and therefore how many outlines the
/// ghost owes in step F.
const WIDE_REACH: usize = 3;

/// How far each drag travels, in window logical points, on each axis.
///
/// Comfortably past the drag threshold and past `Refusal::NoTravel`'s floor,
/// and small enough that the pointer stays well inside the canvas at fit zoom.
const DRAG_PX: f32 = 40.0;

/// How long the pointer rests at the halfway point of each drag.
///
/// The ghost is drawn on the frames a move is in flight, so the gesture has to
/// spend frames in flight. A press-and-release with no dwell relies on the
/// walk's intermediate steps landing on repaints, which is scheduler luck.
const DWELL_MS: u64 = 220;

/// See the module documentation.
pub struct DraggingAChunkShowsWhereItIsGoing;

impl Check for DraggingAChunkShowsWhereItIsGoing {
    fn name(&self) -> &'static str {
        "dragging_a_chunk_shows_where_it_is_going"
    }

    fn defect(&self) -> &'static str {
        "Dragging a line of a note previews nothing — no ghost, no outline, and the chunk boxes \
         stay where they were — so the operator moves text blind and learns where it went only \
         after he lets go"
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
/// not run*. The inner one separates *the assertion did not hold* from *here is
/// the number it read*.
type Step<T> = Result<std::result::Result<T, String>>;

/// Drag from `from` by [`DRAG_PX`] on both axes, resting halfway, and read the
/// ghost line the drag wrote.
///
/// `Ok(Err(_))` is the FAIL sentence, and it quotes the defect by name when the
/// build wrote `suppressed=o63` where a ghost was owed.
fn drag_and_read_ghost(
    session: &Session,
    driver: &Driver,
    frame: &WindowFrame,
    from: ScreenPoint,
    expected_boxes: usize,
    label: &str,
) -> Step<()> {
    let to = frame.offset_from(from, DRAG_PX, DRAG_PX);
    let via = frame.offset_from(from, DRAG_PX / 2.0, DRAG_PX / 2.0);
    let mark = session.trace()?.mark();
    driver.drag_via(
        from,
        via,
        std::time::Duration::from_millis(DWELL_MS),
        to,
        None,
    )?;
    session.settle(30);
    let trace = session.trace()?;

    let Some(line) = trace.last_after(GHOST_EVENT, mark) else {
        return Ok(Err(format!(
            "★★ {label}: DRAGGING {expected_boxes} LINE(S) OF THE NOTE PREVIEWED NOTHING — no \
             `{GHOST_EVENT}` line at all. The painter writes that line on every frame a move is \
             in flight, whether it draws or withholds, so an ABSENCE is not the O63 withholding: \
             it is `overlay::draw_move_ghost` never being reached, which means no move was in \
             flight and the press was read as something else. Trace: {}.",
            session.trace_path().display()
        )));
    };

    if line.get("suppressed") == Some(SUPPRESSED_O63) {
        return Ok(Err(format!(
            "★★★ {label}: THE DEFECT, EXACTLY — `{}`. The ghost was withheld at the chunk rung, \
             so a drag of a line of a note moves nothing on screen until the operator lets go. \
             `suppressed={SUPPRESSED_O63}` is correct for a path NODE, whose real anchors are \
             already travelling; a text chunk has no path geometry, so withholding leaves the \
             gesture with no feedback whatsoever. The condition in `draw_move_ghost` is *is the \
             real geometry already travelling*, and it must be read off the shape preview's \
             `shapes` being NON-EMPTY — a preview that exists and is empty, which is what \
             `shapes::transformed` returns for a text object, shows the operator nothing and may \
             not stand in for the ghost.",
            line.raw
        )));
    }
    if line.get("suppressed") != Some(NOT_SUPPRESSED) {
        return Ok(Err(format!(
            "{label}: `{}` — expected `suppressed={NOT_SUPPRESSED}` or \
             `suppressed={SUPPRESSED_O63}` and got neither. The field has two values and this \
             check knows both of them, so a third is a change to the trace vocabulary that \
             nobody told the harness about.",
            line.raw
        )));
    }
    if line.get("rung") != Some(PART_RUNG_LOWER) {
        return Ok(Err(format!(
            "{label}: `{}` — expected `rung={PART_RUNG_LOWER}`. Every assertion in this check is \
             about a preview at the CHUNK rung; a preview reported at the object rung means the \
             descent was lost between the selection and the drag, and the number of boxes below \
             is then a claim about the wrong subject.",
            line.raw
        )));
    }
    let Some(boxes) = line.get_usize("boxes") else {
        return Ok(Err(format!(
            "{label}: `{}` carries no readable `boxes=`. That field is the painter's claim about \
             its own stroke loop and it is the only thing here that witnesses a draw rather than \
             an intention.",
            line.raw
        )));
    };
    if boxes != expected_boxes {
        return Ok(Err(format!(
            "★★ {label}: `{}` — the preview drew {boxes} outline(s) where {expected_boxes} \
             line(s) are held. The operator sweeps {expected_boxes} labels of a note, drags, and \
             watches {boxes} of them move. `draw_move_ghost` strokes one box per \
             `SelectionState::outlines` entry, so a short count is that iterator disagreeing with \
             what the status bar says is held.",
            line.raw
        )));
    }
    Ok(Ok(()))
}

/// Read `held=` off the status line, which is the number `boxes=` is measured
/// against.
fn held_now(session: &Session, mark: usize, label: &str) -> Step<usize> {
    let trace = session.trace()?;
    let Some(line) = trace.last_after(RUNG_EVENT, mark) else {
        return Ok(Err(format!(
            "{label}: no `{RUNG_EVENT}` line, so the surface that tells the operator what the \
             next verb acts on said nothing. Without it there is no independent count for the \
             preview's `boxes=` to be measured against, and this check would be comparing the \
             painter with itself."
        )));
    };
    let Some(held) = line.get_usize("held") else {
        return Ok(Err(format!(
            "{label}: `{}` carries no readable `held=`.",
            line.raw
        )));
    };
    Ok(Ok(held))
}

/// Two clicks at `at`, which selects the object and then descends into the
/// chunk under the pointer.
fn descend(session: &Session, driver: &Driver, at: ScreenPoint) -> Step<usize> {
    driver.press(vk::ESCAPE)?;
    session.settle(12);
    driver.press(vk::ESCAPE)?;
    session.settle(12);

    let mark = session.trace()?.mark();
    driver.click_at(at)?;
    session.settle(26);
    driver.click_at(at)?;
    session.settle(26);
    let trace = session.trace()?;
    let Some(line) = trace.last_after(SELECTION_EVENT, mark) else {
        return Ok(Err(format!(
            "★★ TWO CLICKS INSIDE THE FIXTURE'S TEXT SELECTED NOTHING: no `{SELECTION_EVENT}` \
             line. Without a chunk entered there is nothing to drag, and a preview of nothing is \
             not this check's subject. `clicking_a_chunk_selects_that_chunk` is the row that owns \
             the descent and it should be read first. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if line.get("level") != Some(PART_RUNG) {
        return Ok(Err(format!(
            "★★ THE DESCENT DID NOT REACH ONE LINE: `{}` — expected `level={PART_RUNG}`. A drag \
             at the object rung is a different gesture, and the ghost is never withheld there, so \
             this check would report green on a build with the defect intact.",
            line.raw
        )));
    }
    let Some(part) = line.get_usize("part") else {
        return Ok(Err(format!(
            "the descent reached the chunk rung and named no chunk: `{}`.",
            line.raw
        )));
    };
    Ok(Ok(part))
}

/// The whole gesture sequence. See the module documentation.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is four clicks, one band sweep and two \
             press-move-dwell-release drags, and it needs the pointer and the foreground. \
             Reported as SKIPPED rather than passed: a check that did not run has learned \
             nothing.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    // ★ PINNED: `--pdf` and `--doc-point` are read and IGNORED. The band's
    // rectangle is stated against this document's baselines.
    let (pdf, anchor) = crate::fixture::text_chunk_point(ANCHOR_CHUNK);
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the text fixture is not at {}. It is committed to this repository, so an absence is \
             a broken checkout rather than an unavailable precondition, and is reported as a \
             failure for that reason — a SKIP would say the opposite.",
            pdf.display()
        )));
    }
    report.note(format!(
        "--pdf and --doc-point are IGNORED: this check pins {} and drags lines of its one \
         six-line text object",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("chunk-ghost.trace.txt"));
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
    let frame = session.frame()?;

    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "no `{PAGE_REGION}` region, so no sheet is on screen. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // --- the precondition: the chunk boxes must be ON -----------------------
    //
    // Not this check's subject, but its premise: the chunk rung is offered only
    // where a box is drawn, so a run that began with the switch off would
    // measure the switch and report it as a preview defect. The preference is
    // persisted beside the exe, so a previous run that left it off is a fact
    // about the machine and not about the build.
    if verdict(&session.trace()?, 0) == Some(Verdict::Declined("switched-off".to_owned())) {
        report.note(
            "the chunk boxes were OFF at launch. Turning them on: the chunk rung is offered only \
             where a box is drawn, so this check has no subject without them.",
        );
        if let Some(failure) = press_the_toggle(&session, &driver, ui_rect, true, report)? {
            return Ok(Some(failure));
        }
    }

    // Every aim is converted BEFORE the first gesture, from one canvas rect. A
    // conversion taken between gestures would silently re-read a rect a
    // committed move had already changed.
    let at_anchor = aim(ctx, &session, page, anchor)?;
    let wide = (
        aim(ctx, &session, page, WIDE_BAND.0)?,
        aim(ctx, &session, page, WIDE_BAND.1)?,
    );
    // ★★ The direction is the gesture's meaning, and it is asserted about the
    // SCREEN points rather than assumed from the document ones. The band is
    // written right-to-left so that it is a crossing band; a mapping that
    // mirrored the x axis would turn it into an enclosing one without changing
    // a word of the assertions below.
    if wide.0.x() <= wide.1.x() {
        return Err(Error::new(format!(
            "the band's start point did not convert to the RIGHT of its end point on screen — \
             {:?} → {:?}. SKIPPED: the crossing gesture step D means to drive cannot be driven \
             through this mapping.",
            wide.0, wide.1
        )));
    }

    // --- B: descend to ONE line --------------------------------------------
    //
    // The mark is taken BEFORE the descent because `status-rung` is written
    // when the selection changes; a mark taken afterwards would find silence
    // and read it as the status line saying nothing.
    let mark = session.trace()?.mark();
    let part = match descend(&session, &driver, at_anchor)? {
        Ok(part) => part,
        Err(why) => return Ok(Some(why)),
    };
    if part != ANCHOR_CHUNK {
        return Ok(Some(format!(
            "the descent reached chunk {part} where the aim was chunk {ANCHOR_CHUNK}. Step D's \
             band is stated against the baselines of lines 0 to 2 and step C is about to move \
             this one, so a drag of the wrong line would leave the band reaching a set nobody \
             can state."
        )));
    }
    let held = match held_now(&session, mark, "B")? {
        Ok(held) => held,
        Err(why) => return Ok(Some(why)),
    };
    if held != 1 {
        return Ok(Some(format!(
            "B: the status line says {held} line(s) are held after a descent onto one. Step C's \
             `boxes=` is measured against this number, so it has to be the one the descent \
             produced."
        )));
    }
    report.note("one line of the note is selected, at the chunk rung");

    // --- C: drag it, and require a preview ----------------------------------
    let mark = session.trace()?.mark();
    if let Err(why) = drag_and_read_ghost(&session, &driver, &frame, at_anchor, 1, "C")? {
        return Ok(Some(why));
    }
    report.note("C: dragging one line previewed one outline travelling with the pointer");

    // --- D: and require that the preview was of a move that could happen ----
    //
    // ★★ A ghost is a promise. A build that drew one and then declined the
    // release would show the operator the line moving and put it back, which is
    // a worse defect than drawing nothing — so the preview is only honoured if
    // the release committed.
    let trace = session.trace()?;
    if trace.last_after(MOVED_ONE_EVENT, mark).is_none() {
        let declined = trace
            .last_after(MOVE_DECLINED_EVENT, mark)
            .map(|l| l.raw.clone())
            .unwrap_or_else(|| "and nothing was written at all".to_owned());
        return Ok(Some(format!(
            "★★ C previewed a move that never happened: no `{MOVED_ONE_EVENT}` after the release \
             — {declined}. The ghost promised the operator the line would land where he let go. \
             A preview of a refusal is worse than no preview: he watches the text move and then \
             finds it back where it started. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- E: build a set of THREE with a band --------------------------------
    //
    // ★★ The selection is deliberately NOT cleared first. `marquee::take_chunks`
    // offers the chunk rung only to a band drawn while a chunk of that object
    // is already selected — a band drawn from nothing is the object-rung
    // gesture, which writes `marquee-mode` and ascends. Line 4 is still held
    // from step C, so this band stays where this check's subject is.
    let mark = session.trace()?.mark();
    driver.drag(wide.0, wide.1)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(band) = trace.last_after(BAND_PARTS_EVENT, mark) else {
        return Err(Error::new(format!(
            "the band never reached the chunk rung: no `{BAND_PARTS_EVENT}`. SKIPPED rather than \
             failed — that is `a_rubber_band_inside_a_note_takes_its_lines`' subject, and it is \
             this step's precondition rather than its assertion. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if band.get_usize("kept") != Some(WIDE_REACH) {
        return Err(Error::new(format!(
            "the band did not keep {WIDE_REACH} lines: `{}`. SKIPPED: the set \
             this step needs was not built, and which lines a band reaches belongs to \
             `a_rubber_band_inside_a_note_takes_its_lines`.",
            band.raw
        )));
    }
    let held = match held_now(&session, mark, "E")? {
        Ok(held) => held,
        Err(why) => return Ok(Some(why)),
    };
    if held != WIDE_REACH {
        return Ok(Some(format!(
            "E: the band kept {WIDE_REACH} lines and the status line says {held} are held. Step \
             F measures the preview against what the operator is TOLD is held, so the two have to \
             agree before the preview can be judged."
        )));
    }
    report.note(format!(
        "E: a band swept {WIDE_REACH} lines of the note into one set"
    ));

    // --- F: drag the set, and require THREE outlines ------------------------
    //
    // The press begins on line 0, which the band selected — the ordinary case.
    // Pressing on an UNSELECTED line is a different gesture with its own row in
    // `chunk_multi_move`.
    let on_first = aim(ctx, &session, page, crate::fixture::text_chunk_point(0).1)?;
    let mark = session.trace()?.mark();
    if let Err(why) = drag_and_read_ghost(&session, &driver, &frame, on_first, WIDE_REACH, "F")? {
        return Ok(Some(why));
    }
    let trace = session.trace()?;
    if trace.last_after(MOVED_MANY_EVENT, mark).is_none() {
        let declined = trace
            .last_after(MOVE_DECLINED_EVENT, mark)
            .map(|l| l.raw.clone())
            .unwrap_or_else(|| "and nothing was written at all".to_owned());
        return Ok(Some(format!(
            "★★ F previewed a move of {WIDE_REACH} lines that never happened: no \
             `{MOVED_MANY_EVENT}` after the release — {declined}. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "F: dragging {WIDE_REACH} lines previewed {WIDE_REACH} outlines travelling together, and \
         the release moved all {WIDE_REACH}"
    ));

    Ok(None)
}
