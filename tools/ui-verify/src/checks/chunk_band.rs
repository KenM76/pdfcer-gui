//! `a_rubber_band_inside_a_note_takes_its_lines` — **O215 ask 4, the band
//! half: a rubber-band drawn inside a text block selects the LINES it reaches,
//! and Shift and Ctrl refine that set the way they refine any other.**
//!
//! # The request
//!
//! `OPERATOR_REQUESTS.md` **O215** ask 4 names three gestures in one breath —
//! *shift-click, ctrl-click and rubber-band*. The first two are driven by
//! [`crate::checks::chunk_multi_move`]. This is the third, and it is the one
//! the operator reaches for when a note has eight labels and he wants five of
//! them: he does not click five times, he sweeps.
//!
//! # The defect it was written against
//!
//! `SelectionState::marquee` hard-set `SelectionLevel::Object`. So a band drawn
//! *inside* a note — with a line of that note already selected — ascended out
//! of the chunk rung and took the whole block. The operator's experience: he
//! sweeps three of six lines and gets all six, at the rung above the one he was
//! working at, and his descent is gone.
//!
//! # The oracle — and the negative that names the defect exactly
//!
//! ```text
//! chunk band   marquee-parts page=0 object=0 mode=touched reached=3 kept=3 combine=replace
//! selection    selection-set page=0 object=0 part=0 level=part held=3 via=band
//! status bar   status-rung kind=text part=0 held=3 of=6
//! apply phase  move-text-lines page=0 n=3 epoch=1 disclosures=none
//! history      undo kind=MoveTextRun undo_depth=1
//! ```
//!
//! ★★★ `marquee-mode` is the **object-rung** band's own line, and the defective
//! build writes it where this check requires `marquee-parts`. The two are
//! mutually exclusive — `take_chunks` returns before `select_with` is reached —
//! so the trace says which of the two bands ran, in one word, and a failure
//! here can quote the defect rather than describe it. A check that read only
//! *"three chunks are selected"* would be satisfied by silence on both.
//!
//! | line | question it answers |
//! |---|---|
//! | `marquee-parts` present | did the band stay at the chunk rung at all? |
//! | `marquee-mode` present | did it ascend and take the whole block — the defect |
//! | `reached=` | did the geometry find the lines the band actually covers? |
//! | `kept=` / `combine=` | did the modifier arm combine rather than replace? |
//! | `status-rung held=` | does the surface that names the next verb's subject agree? |
//! | `move-text-lines` | is a band's set honoured by the same plural verb a Shift-click's is? |
//! | `undo_depth=1` | did N engine calls fold into ONE undo entry? |
//!
//! # The geometry, which is arithmetic and not an estimate
//!
//! `fixtures/paragraph.pdf` — six lines of one text object, baselines 16 pt
//! apart, glyphs about 8.4 pt tall, stated line by line in
//! [`crate::fixture::text_block_target`]:
//!
//! ```text
//! line 0  baseline 700.0   glyph band 700.0 .. 708.4   x 72.0 .. 338.8
//! line 1  baseline 684.0              684.0 .. 692.4     72.0 .. 241.4
//! line 2  baseline 668.0              668.0 .. 676.4     72.0 .. 276.8
//! line 3  baseline 652.0              652.0 .. 660.4     72.0 .. 298.1
//! line 4  baseline 636.0              636.0 .. 644.4     72.0 .. 216.7
//! line 5  baseline 620.0              620.0 .. 628.4     72.0 .. 110.0
//! ```
//!
//! [`WIDE_BAND`] runs from (400, 730) to (60, 666) — **right to left**, so it
//! is a crossing band and reaches whatever it touches. Its floor at y = 666
//! sits 5.6 pt above line 3's ceiling and comfortably inside line 2, so it
//! takes lines 0, 1 and 2 and no others.
//!
//! [`NARROW_BAND`] runs from (400, 644) to (60, 634), 5.2 pt clear of line 3
//! below and 5.6 pt clear of line 5 above, so it reaches line 4 alone.
//!
//! ★★ Crossing rather than enclosing on purpose. An enclosing band would have
//! to discriminate on the lines' **right edges**, which are a claim about what
//! `ObjectModelProvider::text_line_bounds_canvas_of` counts as the end of a
//! line — trailing space, the text object's own width, the advance past the
//! final glyph. The vertical extents are the ones the fixture states, so the
//! bands are aimed along the axis whose numbers are known. The enclosing arm is
//! covered where it can be measured exactly, in `canvas::chunks`' own test of
//! the direction rule.
//!
//! ★★★ Both bands **begin at x = 400**, which is past the right edge of the
//! longest line and past the text object's box. That is a requirement, not a
//! margin: `canvas::pressing::body_under` claims a press inside the block's box
//! but on no line of it as a move of the selection — deliberately, so the white
//! between two lines of a note stays draggable. A band that began there would
//! be a drag of the set, and this check would be measuring the wrong gesture
//! while reading as if it measured this one.
//!
//! # Fixture — pinned, and `--pdf` is ignored
//!
//! ⚠ A missing fixture is a **FAIL**, not a SKIP: it is committed here, so its
//! absence is a broken checkout rather than an unavailable precondition.
//!
//! # ⚠ What this check can see, and where its reach ends
//!
//! It reads what four subsystems wrote down, not the pixels. That the band was
//! *drawn* while the pointer travelled, and that three outlines appeared when
//! it was released, have one oracle — a rendered screenshot — and they are
//! `text_chunks`' subject rather than this one's.
//!
//! # ⚠ HOW TO FALSIFY THIS CHECK — do this before believing a PASS
//!
//! Five plants. **Copy each file aside first** and restore from the byte copy;
//! never revert with git, because this project runs parallel tracks and a
//! chained revert discards another track's uncommitted work.
//!
//! 1. **The band never reaches the chunk rung.** Make `marquee::take_chunks`
//!    return `false` unconditionally. Step C goes red quoting `marquee-mode` —
//!    the object-rung band — which is the original defect exactly.
//! 2. **The direction rule collapses.** Make `chunks::reaches` answer
//!    `band.intersects(chunk)` in both arms. Step C stays green, because it
//!    drives a crossing band; the unit test
//!    `only_a_crossing_band_takes_a_chunk_it_merely_clips` is what catches this
//!    one, and it is named here so a reader does not conclude this check covers
//!    it.
//! 3. **The modifier arms do not combine.** Make `marquee::combined`'s `Add`
//!    arm return `reached.to_vec()`: step C stays green, step D goes red on
//!    `kept=1` where 4 was required — the Shift band throwing away the set it
//!    was meant to extend. Then put the `Subtract` arm back to `held.to_vec()`:
//!    steps C and D stay green, step E goes red on `kept=4` where 3 was
//!    required. The two arms are separately falsifiable because the check
//!    drives them over two different rectangles.
//! 4. **The set is built and not honoured.** Delete the `None if lines.len() >
//!    1` arm of `canvas::moving::eligible`'s page-object text branch. Steps C
//!    to F stay green; step G goes red naming `move-text-line`, the singular
//!    verb over a band of three.
//! 5. **Prove the plant is in the artifact.** `cargo build --release -p
//!    pdfcer-gui` AND `-p ui-verify`, then confirm the exe is newer than the
//!    source: a stale binary is the commonest cause of a falsification that
//!    "did not reproduce", and its tell is an **absent** trace line rather than
//!    a wrong one.
//! 6. **Require the `[FAIL]` line**, not the exit code — a SKIP exits the way a
//!    PASS does.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::text_chunks::{
    EXPECTED_CHUNKS, MODE, PAGE_REGION, SELECTION_EVENT, Verdict, press_the_toggle, verdict,
};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry, ScreenPoint};
use crate::error::{Error, Result};
use crate::input::{Driver, Key};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// `marquee-parts page=… object=… mode=… reached=… kept=… combine=…` —
/// `marquee::take_chunks`' own account of a band that stayed at the chunk rung.
const BAND_PARTS_EVENT: &str = "marquee-parts"; // ui-text-exempt: a trace event name, never displayed

/// `marquee-mode crossing=… mode=… hits=… …` — the **object-rung** band's line.
///
/// ★★★ Read as a failure witness, never as a success one. The two bands are
/// mutually exclusive in one release, so this line appearing where
/// [`BAND_PARTS_EVENT`] was required is the defect this check exists for,
/// quotable rather than describable.
const BAND_OBJECTS_EVENT: &str = "marquee-mode"; // ui-text-exempt: a trace event name, never displayed

/// `selection-set page=… object=… part=… level=… held=… via=…` —
/// `SelectionState::select_parts`' own line.
const SET_EVENT: &str = "selection-set"; // ui-text-exempt: a trace event name, never displayed

/// `via=` on the line a band writes, as against `press` or `click`.
const VIA_BAND: &str = "band"; // ui-text-exempt: a trace token, never displayed

/// The rung spelled the way `canvas::trace` spells it on `canvas-selection`.
const PART_RUNG: &str = "Part"; // ui-text-exempt: a trace token, never displayed

/// The rung spelled the way `SelectionState::select_parts` spells it.
const PART_RUNG_LOWER: &str = "part"; // ui-text-exempt: a trace token, never displayed

/// `status-rung kind=… part=… held=… of=…` — the sentence the operator reads
/// to learn what the next verb will act on.
const RUNG_EVENT: &str = "status-rung"; // ui-text-exempt: a trace event name, never displayed

/// `move-text-lines page=… n=… epoch=… disclosures=…` — the **plural** verb.
const MOVED_MANY_EVENT: &str = "move-text-lines"; // ui-text-exempt: a trace event name, never displayed

/// `move-text-line …` — the **singular** twin, read only as a failure witness.
const MOVED_ONE_EVENT: &str = "move-text-line"; // ui-text-exempt: a trace event name, never displayed

/// `canvas-move-declined level=… sel=… reason=… detail=…`, on release only.
const MOVE_DECLINED_EVENT: &str = "canvas-move-declined"; // ui-text-exempt: a trace event name, never displayed

/// `undo kind=… undo_depth=…`.
const UNDO_EVENT: &str = "undo"; // ui-text-exempt: a trace event name, never displayed

/// `undo-applied page=… n=… epoch=… disclosures=…` — the engine's half.
const UNDO_APPLIED_EVENT: &str = "undo-applied"; // ui-text-exempt: a trace event name, never displayed

/// `undo-declined reason=empty-stack` — nothing on the log.
const UNDO_DECLINED_EVENT: &str = "undo-declined"; // ui-text-exempt: a trace event name, never displayed

/// The combining arm a plain band is required to report, as `Combine::label`
/// spells it.
const REPLACE: &str = "replace"; // ui-text-exempt: a trace token, never displayed

/// The arm a Shift band is required to report.
const ADD: &str = "add"; // ui-text-exempt: a trace token, never displayed

/// The arm a Ctrl band is required to report.
const SUBTRACT: &str = "subtract"; // ui-text-exempt: a trace token, never displayed

/// The direction word a right-to-left band is required to report.
const TOUCHED: &str = "touched"; // ui-text-exempt: a trace token, never displayed

/// The chunk the descent aims at, and the one the set is dragged by.
///
/// Line 0, which [`WIDE_BAND`] also reaches — so the drag in step G begins on
/// a line the band selected, which is the ordinary case and the one this check
/// means to measure. Pressing on an *unselected* line is a different gesture
/// with its own row, in [`crate::checks::chunk_multi_move`].
const ANCHOR_CHUNK: usize = 0;

/// The band that takes lines 0, 1 and 2 — see the module header's table.
///
/// Right to left, so it is a crossing band. `(from, to)` in PDF user space on
/// page 0.
const WIDE_BAND: (DocPoint, DocPoint) = (
    DocPoint::new(0, 400.0, 730.0),
    DocPoint::new(0, 60.0, 666.0),
);

/// How many lines [`WIDE_BAND`] reaches.
const WIDE_REACH: usize = 3;

/// The band that takes line 4 alone.
const NARROW_BAND: (DocPoint, DocPoint) = (
    DocPoint::new(0, 400.0, 644.0),
    DocPoint::new(0, 60.0, 634.0),
);

/// How many lines [`NARROW_BAND`] reaches.
const NARROW_REACH: usize = 1;

/// How far the set is dragged, in window logical points, on each axis.
///
/// Comfortably past the drag threshold and past `Refusal::NoTravel`'s floor,
/// and small enough that the pointer stays well inside the canvas at fit zoom.
const DRAG_PX: f32 = 40.0;

/// How deep the undo log must be when undo is pressed.
///
/// **One**, and that is the assertion: the plural move issues one
/// `move_text_run` per run of every selected line and `fold_undo` coalesces
/// them into a single entry. A build that skipped the fold moves three lines
/// and then needs one press of undo per line, which the operator experiences as
/// undo not working.
const EXPECTED_DEPTH: usize = 1;

/// See the module documentation.
pub struct ARubberBandInsideANoteTakesItsLines;

impl Check for ARubberBandInsideANoteTakesItsLines {
    fn name(&self) -> &'static str {
        "a_rubber_band_inside_a_note_takes_its_lines"
    }

    fn defect(&self) -> &'static str {
        "A rubber-band drawn inside a text block cannot reach its lines — it ascends out of the \
         chunk rung and takes the whole block, so an operator who sweeps three labels of a note \
         gets all of them and loses the rung he was working at"
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

/// What one band produced, read off [`BAND_PARTS_EVENT`].
#[derive(Debug, Clone)]
struct Swept {
    /// `reached=` — how many chunks the band's geometry covered.
    reached: usize,
    /// `kept=` — how many the combining arm left held.
    kept: usize,
    /// `combine=` — which arm ran.
    combine: String,
    /// `mode=` — `touched` or `enclosed`.
    mode: String,
    /// The whole line, so a failure quotes rather than describes.
    raw: String,
}

/// Sweep one band, with `modifier` held throughout, and read what it did.
///
/// `Ok(Err(_))` is the FAIL sentence for a band that never reached the chunk
/// rung at all — the defect — quoting the object-rung line when the build wrote
/// one.
fn sweep(
    session: &Session,
    driver: &Driver,
    band: (ScreenPoint, ScreenPoint),
    modifier: Option<Key>,
    label: &str,
) -> Step<Swept> {
    let mark = session.trace()?.mark();
    match modifier {
        Some(key) => driver.drag_with_modifier(band.0, band.1, key)?,
        None => driver.drag(band.0, band.1)?,
    }
    session.settle(30);
    let trace = session.trace()?;
    let Some(line) = trace.last_after(BAND_PARTS_EVENT, mark) else {
        let ascended = trace
            .last_after(BAND_OBJECTS_EVENT, mark)
            .map(|l| l.raw.clone());
        return Ok(Err(format!(
            "★★★ THE DEFECT: THE {label} BAND DID NOT STAY INSIDE THE NOTE — no \
             `{BAND_PARTS_EVENT}` line follows the release.\n\
             {}\n\
             `SelectionState::marquee` hard-set the Object rung, which is why a band drawn \
             between two lines of a block used to take the block. `marquee::take_chunks` is \
             the fork that keeps it at the chunk rung, and it runs only where the chunk boxes \
             are drawn and a chunk is entered — both of which the steps above established. \
             Trace: {}.",
            ascended.map_or_else(
                || format!(
                    "No `{BAND_OBJECTS_EVENT}` line either, so no band ran at all: the press \
                     was claimed as something else — a move of the selection, most likely, \
                     which would mean it landed inside the text object's box rather than clear \
                     of it."
                ),
                |raw| format!(
                    "It ASCENDED to the object rung instead and swept whole objects: `{raw}`. \
                     That is the operator sweeping three lines of a note and getting the note."
                )
            ),
            session.trace_path().display()
        )));
    };
    Ok(Ok(Swept {
        reached: line.get_usize("reached").unwrap_or(0),
        kept: line.get_usize("kept").unwrap_or(0),
        combine: line.get("combine").unwrap_or("unstated").to_owned(),
        mode: line.get("mode").unwrap_or("unstated").to_owned(),
        raw: line.raw.clone(),
    }))
}

/// Assert one band's whole account of itself in one place.
fn judge(
    swept: &Swept,
    label: &str,
    reached: usize,
    kept: usize,
    combine: &str,
) -> std::result::Result<(), String> {
    if swept.mode != TOUCHED {
        return Err(format!(
            "the {label} band was drawn RIGHT TO LEFT and reported `mode={}`: `{}`. A \
             right-to-left band is a crossing one at every rung of this application, so a band \
             that enclosed here has taken its direction from somewhere other than the drag — \
             and every count below is then about a different gesture.",
            swept.mode, swept.raw
        ));
    }
    if swept.combine != combine {
        return Err(format!(
            "★★ THE {label} BAND'S MODIFIER DID NOT ARRIVE WITH IT: `combine={}`, expected \
             `{combine}`. Line: `{}`.\n\
             Read before the counts: this is a claim about the HARNESS as much as about the \
             application. `Driver::drag_with_modifier` holds the key down across the press, \
             the walk and the release, and a synthetic press that arrives between the key-down \
             and the application's next input pass is an unmodified drag. A `replace` where an \
             `add` was required makes everything below a measurement of a plain band.",
            swept.combine, swept.raw
        ));
    }
    if swept.reached != reached {
        return Err(format!(
            "the {label} band reached {} line(s) and the fixture's geometry says it covers \
             {reached}: `{}`.\n\
             The bands are aimed along the vertical axis, whose extents `text_block_target` \
             states line by line, with at least 5 pt of clearance from the nearest line each \
             band must miss. An off-by-one here is a question about \
             `ObjectModelProvider::text_line_bounds_canvas_of` — what it counts as the top and \
             the bottom of a line — or about the aim conversion. It is NOT a question about \
             the combining arms, which `kept=` answers separately below.",
            swept.reached, swept.raw
        ));
    }
    if swept.kept != kept {
        return Err(format!(
            "★★★ THE {label} BAND REACHED {reached} LINE(S) AND LEFT {} HELD, WHERE {kept} WAS \
             REQUIRED: `{}`.\n\
             `reached` and `kept` disagree, so the geometry found the right lines and the \
             `{combine}` arm of `marquee::combined` did the wrong thing with them. That is \
             O215 ask 4's refinement half: the operator sweeps three labels, Shift-sweeps a \
             fourth, and watches the first three disappear.",
            swept.kept, swept.raw
        ));
    }
    Ok(())
}

/// Read back the selection the last band left, off [`SET_EVENT`].
fn held_after_band(session: &Session, mark: usize, label: &str) -> Step<usize> {
    let trace = session.trace()?;
    let Some(line) = trace
        .events(SET_EVENT)
        .filter(|l| l.lineno > mark && l.get("via") == Some(VIA_BAND))
        .last()
    else {
        return Ok(Err(format!(
            "★★ THE {label} BAND DECIDED AND NEVER SET THE SELECTION: no `{SET_EVENT} … \
             via={VIA_BAND}` line, although `{BAND_PARTS_EVENT}` reported what it had chosen. \
             The two lines are written by `marquee::take_chunks` and by \
             `SelectionState::select_parts` in that order, so the first without the second is a \
             band that worked out its answer and dropped it — the operator sweeps and nothing \
             changes. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if line.get("level") != Some(PART_RUNG_LOWER) {
        return Ok(Err(format!(
            "the {label} band set the selection at `{}` and the chunk rung is \
             `{PART_RUNG_LOWER}`: `{}`. A band that lands on another rung has answered a \
             different question from the one the operator asked by sweeping inside a block.",
            line.get("level").unwrap_or("an unstated rung"),
            line.raw
        )));
    }
    let Some(held) = line.get_usize("held") else {
        return Ok(Err(format!(
            "the {label} band set a selection that does not say how many chunks it holds: \
             `{}`. `held=` is what the status bar and the move rules read to tell a set from a \
             single line.",
            line.raw
        )));
    };
    Ok(Ok(held))
}

/// Ascend to the top of the ladder, then descend to the one chunk under `at`.
///
/// Two clicks, because the chunk rung is entered on the second. The leading
/// Escapes make this callable without inheriting a rung from whatever ran
/// before it.
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
             line. Without a chunk entered there is no note for a band to be drawn *inside*, \
             and `marquee::take_chunks` declines on exactly that. \
             `clicking_a_chunk_selects_that_chunk` is the row that owns the descent and it \
             should be read first. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if line.get("level") != Some(PART_RUNG) {
        return Ok(Err(format!(
            "★★ THE DESCENT DID NOT REACH ONE LINE: `{}` — expected `level={PART_RUNG}`. Every \
             assertion in this check is about a band drawn at the CHUNK rung; a band drawn at \
             the object rung is the gesture `marquee_band` already owns.",
            line.raw
        )));
    }
    let Some(part) = line.get_usize("part") else {
        return Ok(Err(format!(
            "the descent reached the chunk rung and named no chunk: `{}`. `part=none` at \
             `level={PART_RUNG}` is a selection standing inside something it cannot identify.",
            line.raw
        )));
    };
    Ok(Ok(part))
}

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is two clicks, three press-move-release \
             band sweeps — two of them with a modifier held throughout — one drag and a chord, \
             and it needs the pointer and the foreground. Reported as SKIPPED rather than \
             passed: a check that did not run has learned nothing.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    // ★ PINNED: `--pdf` and `--doc-point` are read and IGNORED. The header's
    // geometry table is why the document has to be this one.
    let (pdf, anchor) = crate::fixture::text_chunk_point(ANCHOR_CHUNK);
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the text fixture is not at {}. It is committed to this repository, so an absence \
             is a broken checkout rather than an unavailable precondition, and is reported as a \
             failure for that reason — a SKIP would say the opposite.",
            pdf.display()
        )));
    }
    report.note(format!(
        "--pdf and --doc-point are IGNORED: this check pins {} and sweeps two bands across its \
         one six-line text object",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("chunk-band.trace.txt"));
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
    // Not this check's subject, but its premise: `take_chunks` declines where
    // no box is drawn, so a run that began with the switch off would measure
    // the switch and report it as a band defect. The preference is persisted
    // beside the exe, so a previous run that left it off is a fact about the
    // machine and not about the build.
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
    // conversion taken between gestures would silently re-read a rect the
    // committed edit had moved.
    let at_anchor = aim(ctx, &session, page, anchor)?;
    let wide = (
        aim(ctx, &session, page, WIDE_BAND.0)?,
        aim(ctx, &session, page, WIDE_BAND.1)?,
    );
    let narrow = (
        aim(ctx, &session, page, NARROW_BAND.0)?,
        aim(ctx, &session, page, NARROW_BAND.1)?,
    );
    // ★★ The direction is the gesture's meaning, and it is asserted about the
    // SCREEN points rather than assumed from the document ones. Both bands are
    // written right-to-left in document space precisely so they are crossing
    // bands; a mapping that mirrored the x axis would turn them into enclosing
    // ones without changing a single assertion's wording below.
    if wide.0.x() <= wide.1.x() || narrow.0.x() <= narrow.1.x() {
        return Err(Error::new(format!(
            "the bands' start points did not convert to the RIGHT of their end points on \
             screen — wide {:?} → {:?}, narrow {:?} → {:?}. SKIPPED: the crossing gesture this \
             check means to drive cannot be driven through this mapping.",
            wide.0, wide.1, narrow.0, narrow.1
        )));
    }
    let frame = session.frame()?;

    // --- A: the control — the undo log is EMPTY -----------------------------
    //
    // ★★★ Step H asserts `undo_depth=1`, and that number means *the move added
    // exactly one entry* only if the log was empty before it. Established by
    // pressing undo and requiring a decline, because the depth is not published
    // any other way.
    let mark = session.trace()?.mark();
    driver.press_chord(&[vk::CONTROL], vk::Z)?;
    session.settle(22);
    let trace = session.trace()?;
    if let Some(taken) = trace.last_after(UNDO_EVENT, mark) {
        return Err(Error::new(format!(
            "the document already has something on its undo log — `{}` — so this run cannot \
             measure whether the band's move adds exactly one entry. The fixture is opened \
             read-only and untouched by the steps above. SKIPPED rather than failed: the \
             subject is a different one and it can no longer be measured. Trace: {}.",
            taken.raw,
            session.trace_path().display()
        )));
    }
    if trace.last_after(UNDO_DECLINED_EVENT, mark).is_none() {
        return Err(Error::new(format!(
            "pressing undo on a freshly opened document wrote neither `{UNDO_EVENT}` nor \
             `{UNDO_DECLINED_EVENT}`, so the chord did not reach `history_step` at all and this \
             check cannot establish its control. `Ctrl+Z` is bound in every mode, so an absent \
             line means the keystroke was swallowed before the keymap. SKIPPED. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("control: the undo log is empty before anything is swept or dragged");

    // --- B: enter the note --------------------------------------------------
    let entered = match descend(&session, &driver, at_anchor)? {
        Ok(part) => part,
        Err(failure) => return Ok(Some(failure)),
    };
    report.note(format!("descended to chunk {entered}"));

    // --- C: the plain band --------------------------------------------------
    let mark = session.trace()?.mark();
    let plain = match sweep(&session, &driver, wide, None, "PLAIN")? {
        Ok(swept) => swept,
        Err(failure) => return Ok(Some(failure)),
    };
    if let Err(failure) = judge(&plain, "PLAIN", WIDE_REACH, WIDE_REACH, REPLACE) {
        return Ok(Some(failure));
    }
    match held_after_band(&session, mark, "PLAIN")? {
        Ok(held) if held == WIDE_REACH => {
            report.note(format!(
                "★★★ the band stayed INSIDE the note and took its lines: `{}`",
                plain.raw
            ));
        }
        Ok(held) => {
            return Ok(Some(format!(
                "the band chose {WIDE_REACH} chunks and set a selection of {held}: `{}`. \
                 `marquee::take_chunks` hands its own `kept` list straight to `select_parts`, \
                 so the two counts disagreeing means the list was filtered between the decision \
                 and the selection — by `normalise`, most likely, which collapses to the Object \
                 rung when entries differ by object or page and must not fire for lines of one \
                 block on one page.",
                plain.raw
            )));
        }
        Err(failure) => return Ok(Some(failure)),
    }

    // --- D: Shift refines it upward -----------------------------------------
    let mark = session.trace()?.mark();
    let added = match sweep(&session, &driver, narrow, Some(Key::Shift), "SHIFT")? {
        Ok(swept) => swept,
        Err(failure) => return Ok(Some(failure)),
    };
    let grown = WIDE_REACH + NARROW_REACH;
    if let Err(failure) = judge(&added, "SHIFT", NARROW_REACH, grown, ADD) {
        return Ok(Some(failure));
    }
    match held_after_band(&session, mark, "SHIFT")? {
        Ok(held) if held == grown => {
            report.note(format!(
                "★★ a Shift band ADDED to the set rather than replacing it: `{}`",
                added.raw
            ));
        }
        Ok(held) => {
            return Ok(Some(format!(
                "the Shift band chose {grown} chunks and set a selection of {held}: `{}`.",
                added.raw
            )));
        }
        Err(failure) => return Ok(Some(failure)),
    }

    // --- E: Ctrl refines it back down ---------------------------------------
    //
    // ★★ The SAME band as step D, so the subtraction's required answer is the
    // set step C left. A build whose subtract arm cleared instead would leave
    // nothing; one that added instead would leave the grown set; both are a
    // count away from the required one, on a line that names which arm ran.
    let mark = session.trace()?.mark();
    let removed = match sweep(&session, &driver, narrow, Some(Key::Ctrl), "CTRL")? {
        Ok(swept) => swept,
        Err(failure) => return Ok(Some(failure)),
    };
    if let Err(failure) = judge(&removed, "CTRL", NARROW_REACH, WIDE_REACH, SUBTRACT) {
        return Ok(Some(failure));
    }
    match held_after_band(&session, mark, "CTRL")? {
        Ok(held) if held == WIDE_REACH => {
            report.note(format!(
                "★★ a Ctrl band SUBTRACTED what it reached and left the rest: `{}`",
                removed.raw
            ));
        }
        Ok(held) => {
            return Ok(Some(format!(
                "the Ctrl band chose {WIDE_REACH} chunks and set a selection of {held}: `{}`.",
                removed.raw
            )));
        }
        Err(failure) => return Ok(Some(failure)),
    }

    // --- F: the surface that tells him what he is holding -------------------
    //
    // ★★ Read with `last`, not `last_after`: `status-rung` goes through
    // `diag::trace_changed`, keyed on the rendered line, so the bar re-states
    // itself only when the clause CHANGES. The newest line is the current
    // state, which is what a check about disclosure wants.
    let trace = session.trace()?;
    let Some(rung) = trace.last(RUNG_EVENT) else {
        return Ok(Some(format!(
            "★★ THE STATUS BAR SAYS NOTHING ABOUT THE RUNG: no `{RUNG_EVENT}` line anywhere in \
             the run, although {WIDE_REACH} lines of a text block are selected by band. The \
             operator is holding a set and the only surface that names what the next verb will \
             act on is silent. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if rung.get_usize("held") != Some(WIDE_REACH) {
        return Ok(Some(format!(
            "★★★ THE BAR SAYS HE IS HOLDING {} LINE(S) AND THE BAND LEFT HIM {WIDE_REACH}: \
             `{}`.\n\
             Confidently wrong beats silent at nothing, and this is the wrong half: the \
             outlines say {WIDE_REACH}, the next drag takes {WIDE_REACH}, and the sentence says \
             otherwise. The count reaches `selection_part_of_text` as an argument precisely so \
             it cannot be spelled `1`.",
            rung.get_usize("held")
                .map_or_else(|| "an unstated number of".to_owned(), |h| h.to_string()),
            rung.raw
        )));
    }
    if rung.get_usize("of") != Some(EXPECTED_CHUNKS) {
        return Ok(Some(format!(
            "the bar says the block has {} lines and this fixture's one text object has \
             {EXPECTED_CHUNKS}: `{}`. The set size is right and the total is not, so the \
             denominator comes from somewhere other than the object under the selection.",
            rung.get_usize("of")
                .map_or_else(|| "an unstated number of".to_owned(), |o| o.to_string()),
            rung.raw
        )));
    }
    report.note(format!(
        "★★ the status bar discloses the whole band, not its first line: `{}`",
        rung.raw
    ));

    // --- G: the set the band built moves as one -----------------------------
    let mark = session.trace()?.mark();
    driver.drag(at_anchor, frame.offset_from(at_anchor, DRAG_PX, DRAG_PX))?;
    session.settle(30);
    let trace = session.trace()?;
    if let Some(one) = trace.last_after(MOVED_ONE_EVENT, mark) {
        return Ok(Some(format!(
            "★★★ THE BAND SELECTED {WIDE_REACH} LINES AND ONE MOVED — `{}`, the SINGULAR \
             verb.\n\
             A set built by band is the same selection a set built by Shift-click is, and \
             `canvas::moving::eligible` chooses between `MoveSubject::TextLine` and \
             `MoveSubject::TextLines` on `SelectionState::selected_parts_on(...).len()`. A \
             singular verb here means the drag read `entered_object` — the first entry — \
             instead of the set. Trace: {}.",
            one.raw,
            session.trace_path().display()
        )));
    }
    let Some(moved) = trace.last_after(MOVED_MANY_EVENT, mark) else {
        let declined = trace
            .last_after(MOVE_DECLINED_EVENT, mark)
            .map(|l| l.raw.clone());
        return Ok(Some(format!(
            "★★★ DRAGGING THE BAND'S SET MOVED NOTHING: no `{MOVED_MANY_EVENT}` line follows \
             the release. {} Every line of this fixture states its own position, so \
             `text_run_move_refusal` answers `None` for all six and the engine would accept the \
             move — a refusal here is this shell's and not the document's. Trace: {}.",
            declined.map_or_else(
                || "No `canvas-move-declined` line either, so the gesture did not reach the \
                    move rules at all — the press may have been claimed as another band, or the \
                    travel may have been below the drag threshold."
                    .to_owned(),
                |raw| format!("It was REFUSED instead: `{raw}`.")
            ),
            session.trace_path().display()
        )));
    };
    // `n=` counts show operators rewritten, which is one per RUN and not one
    // per line, so it is asserted as a floor. `WIDE_REACH` is the minimum a
    // three-line set can produce; a lower count is a set spent on part of
    // itself.
    let pieces = moved.get_usize("n").unwrap_or(0);
    if pieces < WIDE_REACH {
        return Ok(Some(format!(
            "the plural verb ran and rewrote {pieces} piece(s) for a band of {WIDE_REACH} \
             lines: `{}`. A set of {WIDE_REACH} lines resolves to at least {WIDE_REACH} runs, \
             so a lower count means `ObjectModelProvider::text_line_runs` answered for some of \
             them and not the others — the set reached the apply phase and was spent on part of \
             itself.",
            moved.raw
        )));
    }
    report.note(format!(
        "★★★ the drag moved the WHOLE band through the plural verb: `{}`",
        moved.raw
    ));

    // --- H: one press of undo, and the whole band comes back ----------------
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
    if undo.get_usize("undo_depth") != Some(EXPECTED_DEPTH) {
        return Ok(Some(format!(
            "★★★ THE MOVE LEFT {} ENTRIES ON THE UNDO LOG AND IT MUST LEAVE {EXPECTED_DEPTH}: \
             `{}`.\n\
             The log was measured empty before the sweep, so every entry on it now was put \
             there by the one drag. `fold_undo` coalesces the run of `move_text_run` calls into \
             a single entry for exactly this reason: an operator who swept three labels and \
             moved them with one drag presses undo once.",
            undo.get_usize("undo_depth")
                .map_or_else(|| "an unstated number of".to_owned(), |d| d.to_string()),
            undo.raw
        )));
    }
    if trace.last_after(UNDO_APPLIED_EVENT, mark).is_none() {
        return Ok(Some(format!(
            "★★ THE UNDO WAS DECIDED AND NEVER APPLIED: `{}` with no `{UNDO_APPLIED_EVENT}` \
             line after it. The history arm decides and names the kind; `vector_edit` bumps the \
             epoch and drops the page texture. A build that writes the first alone has restored \
             nothing the operator can see. Trace: {}.",
            undo.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ one press of undo took the whole band back, off one log entry: `{}`",
        undo.raw
    ));
    Ok(None)
}
