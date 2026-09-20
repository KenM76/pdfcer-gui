//! `redacting_a_clicked_chunk_marks_only_that_chunk` — **O217's first three
//! asks, driven: the redaction verb addresses the unit the operator selected,
//! and it is reachable from the object he selected it on.**
//!
//! # The request
//!
//! `OPERATOR_REQUESTS.md` **O217** asks that redaction address a *chunk* of a
//! text block, by the same gestures that select one, from the place the hand
//! already is. Three claims, and they are separable:
//!
//! | ask | what would satisfy it | what this check measures |
//! |---|---|---|
//! | the unit | a mark bounded by the chunk, not the block | the marked bounds, twice |
//! | the gesture | the click that selects a chunk is the click that aims the mark | the Part rung is standing when the verb runs |
//! | the route | reachable without leaving the canvas (**O53**) | the row is in the object context menu and is pressed |
//!
//! # ★★★ Why the oracle is TWO marks in ONE launch
//!
//! `redact-mark-selection-requested page=N quads=N` is written identically for
//! a chunk-sized mark and a block-sized one — the exact pair this row exists to
//! separate — so a count is an assertion both builds satisfy. The line carries
//! `bbox=llx,lly,urx,ury` for that reason, and this check reads it twice:
//!
//! 1. with the whole text object selected (the **Object** rung), and
//! 2. with one line of it selected (the **Part** rung).
//!
//! The assertion is a comparison between them — the second inside the first,
//! and less than half as tall. **No number is pinned.** A fixture whose
//! leading, page size or text position changed would move both bounds
//! together, and a check built on a literal would go red on a document change
//! while the capability was intact.
//!
//! ⇒ The first mark is therefore not decoration: it is the **control**. Without
//! it there is nothing to say a one-line box is small, because "small" is only
//! meaningful against the block it came out of.
//!
//! # ★★ Why both marks go through the CONTEXT MENU
//!
//! The ribbon route is `redact_selection`'s subject and is already driven
//! there. This row's third ask is the one the ribbon cannot discharge: the
//! operator's hand is on the chunk he just clicked, and a verb that exists only
//! on a tab he has to travel to is the defect **O53** names. Pressing
//! `menu.item.canvas.object.edit.redact_selection` proves the row is drawn,
//! enabled and wired — three things a roster test asserts about the *plan* and
//! none about the running program.
//!
//! # ★ Why the block mark is undone before the chunk mark
//!
//! So the second gesture aims at the same document the first did. A `/Redact`
//! is an annotation the page now carries, and a check whose second click lands
//! on a page the first click changed is measuring two documents. `Ctrl+Z` is
//! also the cheapest possible assertion that marking is undoable at all.
//!
//! # Fixture — pinned, and `--pdf` is ignored
//!
//! `fixtures/paragraph.pdf` through [`crate::fixture::text_chunk_point`]: one
//! text object of six lines on baselines 16 pt apart. Chunk **2** is aimed at —
//! an interior line, so a block-sized mark cannot be mistaken for a chunk-sized
//! one by landing at the same edge.
//!
//! ⚠ A missing fixture is a **FAIL**, not a SKIP: it is committed here, so its
//! absence is a broken checkout.
//!
//! # ⚠ What this check cannot see
//!
//! It reads the bounds the shell *requested*, not the quads the engine stored.
//! `redact_selection` owns the other half — that a mark reaches the document's
//! own census and that nothing is applied — and the two are deliberately not
//! merged: a build that marks the right unit and stores nothing should report
//! both facts, not one.
//!
//! # ⚠ HOW TO FALSIFY THIS CHECK — do this before believing a PASS
//!
//! 1. **Copy the file aside first**: `crates/pdfcer-gui/src/canvas/selection/mod.rs`
//!    to the scratch directory. **Never revert it with git** — this project
//!    runs parallel tracks and a chained revert discards another track's
//!    uncommitted work; restore from the byte copy.
//! 2. **Plant the defect the row reports**: in `SelectionState::outline_rect`,
//!    delete the `part_bounds` arm so every entry answers the whole object's
//!    bounds. The chunk mark becomes block-sized, both bounds become equal, and
//!    step E goes red on the height ratio.
//! 3. **A second plant, for the route**: delete the
//!    `Item::command("edit.redact_selection")` row from `shell::menus`'s
//!    `CANVAS_OBJECT`. Step C goes red — and the roster test in
//!    `shell::menus::tests` goes red with it, which is the pair working as
//!    designed.
//! 4. **Prove the plant is in the artifact.** `cargo build --release -p
//!    pdfcer-gui`, then confirm the exe is newer than the source: a stale
//!    binary is the commonest cause of a falsification that "did not
//!    reproduce", and its tell is an **absent** trace line rather than a wrong
//!    one.
//! 5. **Require the `[FAIL]` line**, not the exit code — a SKIP exits the way a
//!    PASS does.
//! 6. **Restore from the byte copy**, rebuild, confirm the PASS returns.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::text_chunks::{
    EXPECTED_CHUNKS, MODE, PAGE_REGION, SELECTION_EVENT, Verdict, press_the_toggle, verdict,
};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, ScreenPoint};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;
use crate::trace::Trace;

/// The rung a whole object is selected at, as `canvas::trace` spells it.
const OBJECT_RUNG: &str = "Object"; // ui-text-exempt: a trace token, never displayed

/// The rung one chunk is selected at.
const PART_RUNG: &str = "Part"; // ui-text-exempt: a trace token, never displayed

/// The shell's line for the verb under test, carrying `quads=` and `bbox=`.
const REQUESTED: &str = "redact-mark-selection-requested"; // ui-text-exempt: a trace event name

/// What the canvas writes on every frame carrying a secondary click.
const MENU_EVENT: &str = "canvas-menu"; // ui-text-exempt: a trace event name

/// The context the object menu declares.
const OBJECT_CONTEXT: &str = "canvas.object"; // ui-text-exempt: a trace token

/// The context-menu row this check presses — **O53's half of the row**.
const ROW_REGION: &str = "menu.item.canvas.object.edit.redact_selection";

/// Everything the object menu publishes, for a failure that can name what IS
/// there rather than only what is not.
const ROW_PREFIX: &str = "menu.item.canvas.object.";

/// The engine's answer to `Ctrl+Z`, and its refusal.
const UNDO_EVENT: &str = "undo"; // ui-text-exempt: a trace event name
const UNDO_DECLINED_EVENT: &str = "undo-declined"; // ui-text-exempt: a trace event name

/// Which chunk of the fixture is aimed at.
///
/// **An interior line.** Chunk 0 shares its top edge with the block and chunk 5
/// its bottom, so a build that marked the block while standing on either would
/// produce a box agreeing with the chunk on one side — and a containment test
/// that passes for the wrong reason is worse than none.
const AIM_CHUNK: usize = 2;

/// How much of the block's height a single line is allowed to occupy.
///
/// The fixture holds [`EXPECTED_CHUNKS`] lines, so a correct chunk is about a
/// sixth of its block and the wrong answer is the whole of it. Half is the
/// midpoint between the two, chosen so the threshold cannot be reached by
/// leading, ascenders or a rounded trace figure.
const MAX_CHUNK_SHARE: f64 = 0.5;

/// How far outside the block's bounds a chunk's may fall, in points.
///
/// The trace publishes one decimal place, and the two boxes are computed from
/// the same outlines, so this absorbs rounding and nothing else. A real
/// containment failure is tens of points, not tenths.
const CONTAINMENT_SLACK_PT: f64 = 1.0;

/// A rectangle read off a `bbox=` field, in PDF user space.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Bounds {
    llx: f64,
    lly: f64,
    urx: f64,
    ury: f64,
}

impl Bounds {
    /// Parse `llx,lly,urx,ury`. `None` for `bbox=none`, a short field or any
    /// component that is not a number — all of which mean the same thing to
    /// every caller: **this line cannot say what was marked.**
    fn parse(field: &str) -> Option<Self> {
        let mut parts = field.split(',');
        let mut next = || parts.next()?.parse::<f64>().ok();
        let (llx, lly, urx, ury) = (next()?, next()?, next()?, next()?);
        if parts.next().is_some() {
            return None;
        }
        Some(Self { llx, lly, urx, ury })
    }

    fn height(self) -> f64 {
        self.ury - self.lly
    }

    /// Whether `self` lies inside `outer`, allowing [`CONTAINMENT_SLACK_PT`].
    fn inside(self, outer: Self) -> bool {
        self.llx >= outer.llx - CONTAINMENT_SLACK_PT
            && self.lly >= outer.lly - CONTAINMENT_SLACK_PT
            && self.urx <= outer.urx + CONTAINMENT_SLACK_PT
            && self.ury <= outer.ury + CONTAINMENT_SLACK_PT
    }
}

impl std::fmt::Display for Bounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:.1},{:.1},{:.1},{:.1}",
            self.llx, self.lly, self.urx, self.ury
        )
    }
}

/// See the module documentation.
pub struct RedactingAClickedChunkMarksOnlyThatChunk;

impl Check for RedactingAClickedChunkMarksOnlyThatChunk {
    fn name(&self) -> &'static str {
        "redacting_a_clicked_chunk_marks_only_that_chunk"
    }

    fn defect(&self) -> &'static str {
        "redacting one line of a text block takes the whole block, or cannot be reached from the \
         line the operator just clicked — so the only way to remove a value from a note is to \
         remove the note"
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

/// What the last `canvas-selection` line says the selection is.
///
/// Returned as the raw line beside its rung, because every caller that rejects
/// a rung wants to quote what it got rather than describe it.
fn rung(trace: &Trace) -> Option<(String, String)> {
    trace.last(SELECTION_EVENT).map(|l| {
        (
            l.get("level").unwrap_or("unstated").to_owned(),
            l.raw.clone(),
        )
    })
}

/// Right-click at `at`, press the redaction row, and read the bounds it
/// requested.
///
/// The whole gesture is here rather than spelled twice because the two marks
/// differ in exactly one thing — the rung standing when they run — and a second
/// copy of the sequence would be a second place for the route to drift.
///
/// `Ok(Err(..))` is a finding about the program; `Err(..)` is a finding about
/// the run and is reported as SKIPPED.
fn mark_through_the_menu(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    at: ScreenPoint,
    what: &str,
) -> Result<std::result::Result<Bounds, String>> {
    driver.right_click_at(at)?;
    session.settle(35);

    let trace = session.trace()?;
    let Some(menu) = trace.events(MENU_EVENT).last() else {
        return Ok(Err(format!(
            "THE RIGHT-CLICK ON {what} RESOLVED NO MENU AT ALL: no `{MENU_EVENT}` line after a \
             secondary click on the page. `canvas::menus::attach` writes that line on every frame \
             carrying a secondary click, so its absence means the click never reached the canvas \
             response. Trace: {}",
            session.trace_path().display()
        )));
    };
    let context = menu.get("context").unwrap_or_default();
    if context != OBJECT_CONTEXT {
        return Ok(Err(format!(
            "THE RIGHT-CLICK ON {what} RESOLVED `{context}`, NOT `{OBJECT_CONTEXT}`: `{}`.\n\
             A selection standing at either the Object or the Part rung is an OBJECT selection as \
             far as the menu is concerned, so the object menu is the one that must appear. \
             Resolving the view menu here means the secondary hit test lost the selection the \
             left click made. Trace: {}",
            menu.raw,
            session.trace_path().display()
        )));
    }

    let Some(row) = declared(&trace, ui_rect, ROW_REGION) else {
        return Ok(Err(format!(
            "★★★ THE REDACT ROW IS NOT IN THE CANVAS OBJECT MENU: no `{ROW_REGION}` region after \
             the menu opened on {what}. Rows it DID publish: {}.\n\
             Three readings, and all three are defects: `edit.redact_selection` is registered on \
             the Edit ribbon tab only, which is exactly what **O53** forbids; the row is drawn \
             but disabled, because a disabled command is dropped before it is drawn and \
             `selection.any` is not being set for this selection; or `MenuHost::attach_with` has \
             stopped supplying a rect sink, in which case no context-menu row anywhere in this \
             application can be pressed by a check. Trace: {}",
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

    let mark = session.trace()?.mark();
    driver.click_at(session.frame()?.declared_center(row))?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(line) = trace.last_after(REQUESTED, mark) else {
        return Ok(Err(format!(
            "★★★ THE ROW WAS PRESSED AND THE VERB DID NOT RUN: no `{REQUESTED}` line after \
             pressing the redact row on {what}.\n\
             ★★ Ask first whether the press dispatched at all. The menu dies on the pointer MOVE \
             if the canvas is choosing between two responses per frame — egui derives a popup's \
             identity from the response it was attached to — and the tell is the row's \
             `ui-rect-gone` lines arriving in the same frame as `canvas-pointer`, with no button \
             ever going down.\n\
             IF the verb WAS entered, the next suspect is the page filter: \
             `app::actions::redactsel::mark_selection` keeps only outlines whose page is the \
             current one, and a build that kept none writes \
             `redact-mark-selection-declined … reason=no-bounds` instead. Grep for it. Trace: {}",
            session.trace_path().display()
        )));
    };
    let Some(field) = line.get("bbox") else {
        return Ok(Err(format!(
            "★★★ THE VERB RAN AND DID NOT SAY WHAT IT MARKED: `{}` carries no `bbox=` field.\n\
             The count alone is written identically for a chunk-sized mark and a block-sized one, \
             which is the pair this check exists to separate, so without the bounds there is \
             nothing here to measure. Somebody narrowed the trace line; widen it again.",
            line.raw
        )));
    };
    let Some(bounds) = Bounds::parse(field) else {
        return Ok(Err(format!(
            "★★ THE MARKED BOUNDS ARE NOT A RECTANGLE: `{}`.\n\
             `bbox=none` means the union of the marked rectangles was empty on a frame that went \
             on to build quads — which `mark_selection`'s own early return is supposed to make \
             unreachable. Anything else is a malformed field.",
            line.raw
        )));
    };
    Ok(Ok(bounds))
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a line of text, opens a context \
             menu over it and presses a row. Reported as SKIPPED rather than passed: a check that \
             did not run has learned nothing.",
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
    let (pdf, point) = crate::fixture::text_chunk_point(AIM_CHUNK);
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the text fixture is not at {}. It is committed to this repository, so an absence is \
             a broken checkout rather than an unavailable precondition, and is reported as a \
             failure for that reason — a SKIP would say the opposite.",
            pdf.display()
        )));
    }
    report.note(format!(
        "--pdf and --doc-point are IGNORED: this check pins {} and aims at chunk {AIM_CHUNK} of \
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("redact-chunk.trace.txt"));
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

    // --- The precondition: the chunk boxes must be ON -----------------------
    //
    // Not this check's subject — `text_chunks` owns that — but its premise: the
    // chunk rung is offered exactly where a box is drawn, so a run that began
    // with the switch off would measure the switch and report it as a
    // redaction defect. The preference is persisted beside the exe, so a
    // previous run that left it off is a fact about the machine, not the build.
    if verdict(&session.trace()?, 0) == Some(Verdict::Declined("switched-off".to_owned())) {
        report.note(
            "the chunk boxes were OFF at launch. Turning them on: the chunk rung is offered only \
             where a box is drawn, so this check has no subject without them.",
        );
        if let Some(failure) = press_the_toggle(&session, &driver, ui_rect, true, report)? {
            return Ok(Some(failure));
        }
    }

    let at = aim(ctx, &session, page, point)?;

    // --- A: the block ------------------------------------------------------
    driver.click_at(at)?;
    session.settle(26);
    let Some((level, raw)) = rung(&session.trace()?) else {
        return Ok(Some(format!(
            "★★★ THE CLICK SELECTED NOTHING: no `{SELECTION_EVENT}` line after a left click on \
             line {AIM_CHUNK} of the fixture's text. The document was launched with this file and \
             the canvas is drawing a page, so either the click did not reach the canvas or the \
             hit test found nothing under a point that is inside the glyphs. Trace: {}",
            session.trace_path().display()
        )));
    };
    if level != OBJECT_RUNG {
        return Err(Error::new(format!(
            "the first click reported `{raw}`, and this check needs the whole text object as its \
             CONTROL — the block-sized mark it calibrates the chunk-sized one against. SKIPPED \
             rather than failed: a first click that descends is \
             `clicking_a_chunk_selects_that_chunk`'s subject and blaming redaction for it would \
             send the next reader to the wrong file."
        )));
    }
    report.note(format!("★ the first click named the block: `{raw}`"));

    // The boxes have to be on the page before the descent, or the operator had
    // nothing to aim at.
    match verdict(&session.trace()?, 0) {
        Some(Verdict::Drawn { count, .. }) if count == EXPECTED_CHUNKS => {}
        other => {
            return Err(Error::new(format!(
                "the block is selected and its chunks are not drawn: the painter's answer is {}, \
                 and this fixture's one text object holds {EXPECTED_CHUNKS} chunks. The chunk \
                 rung is offered only where a box is, so this check's second mark has no \
                 operand. SKIPPED: `chunk_boxes_show_what_a_text_block_is_made_of` is the row \
                 that owns it. Trace: {}",
                other.map_or_else(|| "silence".to_owned(), |v| format!("`{v}`")),
                session.trace_path().display()
            )));
        }
    }

    // --- B: mark the BLOCK, and read the control --------------------------
    let block = match mark_through_the_menu(&session, &driver, ui_rect, at, "the whole block")? {
        Ok(bounds) => bounds,
        Err(failure) => return Ok(Some(failure)),
    };
    report.note(format!(
        "★★ the block mark: `bbox={block}` — {:.1} pt tall, and this is the CONTROL",
        block.height()
    ));

    // --- C: undo it, so the chunk is marked on the same document -----------
    let mark = session.trace()?.mark();
    driver.press_chord(&[vk::CONTROL], vk::Z)?;
    session.settle(26);
    let trace = session.trace()?;
    if trace.last_after(UNDO_EVENT, mark).is_none() {
        let how = if trace.last_after(UNDO_DECLINED_EVENT, mark).is_some() {
            "the history REFUSED it — `undo-declined` — so marking added nothing to the undo log, \
             and an operator who marks the wrong thing has no way back"
        } else {
            "neither `undo` nor `undo-declined` was written, so the chord did not reach \
             `history_step` at all. `Ctrl+Z` is bound in every mode, so an absent line means the \
             keystroke was swallowed before the keymap — most often by a text field that still \
             has focus"
        };
        return Ok(Some(format!(
            "★★★ THE BLOCK MARK COULD NOT BE UNDONE: {how}. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note("★ undid the block mark — the chunk is marked on the same document");

    // --- D: descend to ONE chunk ------------------------------------------
    //
    // Two clicks, not one: the undo cleared the selection with the epoch, so
    // the ladder starts at the bottom again. The first names the block, the
    // second goes inside it — which is the gesture O217 asks redaction to
    // honour.
    driver.click_at(at)?;
    session.settle(26);
    driver.click_at(at)?;
    session.settle(26);
    let Some((level, raw)) = rung(&session.trace()?) else {
        return Ok(Some(format!(
            "★★ THE SELECTION WENT SILENT AFTER THE UNDO: no `{SELECTION_EVENT}` line from either \
             click. Trace: {}",
            session.trace_path().display()
        )));
    };
    if level != PART_RUNG {
        return Ok(Some(format!(
            "★★★ THE CLICK COULD NOT REACH ONE LINE OF THE BLOCK: `{raw}` — expected \
             `level={PART_RUNG}`.\n\
             This is O217's second ask failing before its first can be measured: the gesture that \
             selects a chunk is the gesture that must aim the mark, and on this build it does not \
             select a chunk. `clicking_a_chunk_selects_that_chunk` is the row that owns the \
             gesture; if it is also red, believe that one first. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note(format!("★★ descended to one line: `{raw}`"));

    // --- E: mark the CHUNK, and compare -----------------------------------
    let chunk = match mark_through_the_menu(&session, &driver, ui_rect, at, "one line")? {
        Ok(bounds) => bounds,
        Err(failure) => return Ok(Some(failure)),
    };
    report.note(format!(
        "★★ the chunk mark: `bbox={chunk}` — {:.1} pt tall",
        chunk.height()
    ));

    // The CONTROL is checked before it is used as one. A block mark with no
    // height makes every ratio below meaningless, and a check that divided by
    // it would report a redaction defect about a degenerate measurement.
    if !block.height().is_finite() || block.height() <= 0.0 {
        return Ok(Some(format!(
            "★★ THE CONTROL HAS NO HEIGHT: the whole-block mark reports `bbox={block}`, which \
             is {:.1} pt tall. Every comparison below is a ratio against that number, so there \
             is nothing here to measure — and a mark with no area would remove nothing when \
             applied, which is its own defect.",
            block.height()
        )));
    }

    if !chunk.inside(block) {
        return Ok(Some(format!(
            "★★★ THE CHUNK'S MARK IS NOT INSIDE ITS BLOCK'S: block `{block}`, chunk `{chunk}`.\n\
             One line of a text object cannot be bounded by anything the object is not, so a box \
             that escapes its parent is not a smaller answer to the same question — it is a \
             different geometry. Suspect the canvas-to-PDF hop before suspecting the rung: \
             `mark_selection` normalises the y flip, and a build that stopped would produce an \
             inside-out rectangle that fails this test and every quad it writes."
        )));
    }

    // Divided only after the containment test above, which is what makes this
    // arithmetic safe: a block whose height is zero, negative or not a number
    // fails that test first, so every value reaching the division is an
    // ordinary rectangle and the quotient cannot be an infinity or a NaN
    // silently comparing as small.
    let share = chunk.height() / block.height();
    if share >= MAX_CHUNK_SHARE {
        return Ok(Some(format!(
            "★★★ REDACTING ONE LINE MARKED THE WHOLE BLOCK: the block's mark is {:.1} pt tall and \
             the chunk's is {:.1} pt — {:.0}% of it, where one of {EXPECTED_CHUNKS} lines should \
             be about a sixth.\n\
             The selection stood at `level={PART_RUNG}` when the verb ran, so the rung is right \
             and the GEOMETRY is wrong: `mark_selection` builds its quads from \
             `SelectionState::outlines()`, and `SelectionState::outline_rect` is the one place \
             that answers `part_bounds` for an entry carrying a subpath. A build that lost that \
             arm answers the whole object's bounds for every entry, which is exactly this \
             number.\n\
             ⇒ This is `OPERATOR_REQUESTS.md` O217 reported from the measurable side: the only \
             way to remove a value from a note is to remove the note. Block `{block}`, chunk \
             `{chunk}`.",
            block.height(),
            chunk.height(),
            share * 100.0
        )));
    }

    report.note(format!(
        "★★★ redacting one line marked {:.0}% of the block's height, from the context menu on the \
         line itself — the unit, the gesture and the route, all three",
        share * 100.0
    ));
    Ok(None)
}
