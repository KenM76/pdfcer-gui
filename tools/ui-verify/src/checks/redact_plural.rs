//! `marking_two_chunks_makes_one_mark_and_two_regions` — **O217's third ask,
//! driven: a redaction gesture over a set of lines is ONE mark holding one
//! region per line.**
//!
//! # The request
//!
//! `OPERATOR_REQUESTS.md` **O217** asks for redaction that addresses a chunk of
//! a text block. Its neighbour `redact_chunk` drives the unit, the gesture and
//! the route against a selection of exactly one line.
//! This row drives the ask that only appears once more than one line is held:
//! **what a single gesture over a set is supposed to produce.**
//!
//! # ★★★ The oracle is TWO fields that pull in OPPOSITE directions
//!
//! | field | required | the wrong build it kills |
//! |---|---|---|
//! | `redact-panel marks=` | before → **before + 1** | one annotation per line: two rows in the review list and two presses of undo for one gesture |
//! | `redact-mark-selection-requested quads=` | **2** | the two regions unioned into one, which destroys the unselected line between them |
//!
//! ⇒ **Neither field alone is an oracle, and that is the whole design of this
//! row.** A census of +1 is satisfied by a build that unioned; `quads=2` is
//! satisfied by a build that wrote two separate annotations of one quad each.
//! Only together do they pin what the operator actually asked for: *one
//! gesture, one mark, one region per line, one undo.*
//!
//! Each is also checked against a **control measured in the same launch** — a
//! singular mark taken first, on one line, through the same menu row. Without
//! it, `quads=2` could be a build that always writes two, and `marks` +1 could
//! be a build whose panel census does not move at all.
//!
//! # ★★ Why chunks 0 and 2, never 0 and 1
//!
//! Adjacent chunks make a union and a pair of regions produce nearly the same
//! `bbox`, so the geometry cannot tell them apart and the check would be
//! deciding on `quads` alone. Two apart leaves chunk **1 unselected between
//! them** — the exact shape of O217's warning — and puts the two aims 32 pt
//! apart on a document whose baselines are 16 pt apart, so an aim off by a few
//! points still lands on the line intended. `chunk_multi_move` picks the same
//! pair for the same reason.
//!
//! # ⚠ What this check CANNOT see
//!
//! The trace publishes the **union** of the regions, not each one. So a PASS
//! here says two regions were built and what they span together; it does not
//! say that each is the box of the line it belongs to, and therefore does not
//! by itself prove chunk 1 survives. The end-to-end proof is the apply report,
//! which lists the text a removal will destroy region by region —
//! `the_apply_report_lists_the_text_it_will_destroy`, on a copy of the fixture.
//!
//! The height comparison below narrows the gap without closing it: two regions
//! spanning three lines of a six-line block have a union at least twice the
//! singular's and well under the block's, which a single unioned rectangle over
//! the whole block would fail. It bounds the answer; it does not name it.
//!
//! # Fixture — pinned, and `--pdf` is ignored
//!
//! `fixtures/paragraph.pdf` through [`crate::fixture::text_chunk_point`]: one
//! text object of six lines on baselines 16 pt apart.
//!
//! ⚠ A missing fixture is a **FAIL**, not a SKIP: it is committed here, so its
//! absence is a broken checkout.
//!
//! # ⚠ HOW TO FALSIFY THIS CHECK — do this before believing a PASS
//!
//! The two assertions need two different plants, and running only one leaves
//! half the oracle unfalsified.
//!
//! 1. **Copy the file aside first.** **Never revert it with git** — this
//!    project runs parallel tracks and a chained revert discards another
//!    track's uncommitted work; restore from the byte copy.
//! 2. **Plant the union**: in `app::actions::redactsel::mark_selection`, fold
//!    the outlines into a single enclosing rectangle before building quads.
//!    `quads` reads 1, `marks` still reads +1, and step G goes red — which is
//!    the field the count alone cannot see.
//! 3. **Plant the split**: make the same function issue one `add_redaction`
//!    per outline instead of one carrying every quad. `quads` still reads 2 —
//!    the regions were right and the GROUPING was not — and `marks` jumps by
//!    2, so step G goes red on the census while the `quads` assertion stays
//!    satisfied. That is the half the count alone cannot see, and a plant that
//!    leaves the other assertion green is the proof it is a second oracle
//!    rather than a restatement of the first.
//! 4. **Prove the plant is in the artifact.** `cargo build --release -p
//!    pdfcer-gui`, then confirm the exe is newer than the source: a stale
//!    binary is the commonest cause of a falsification that "did not
//!    reproduce", and its tell is an **absent** trace line rather than a wrong
//!    one.
//! 5. **Require the `[FAIL]` line**, not the exit code — a SKIP exits the way a
//!    PASS does.
//! 6. **Restore from the byte copy**, rebuild, confirm the PASS returns.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::redact_menu::mark_through_the_menu;
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
use crate::trace::Trace;

/// The rung one chunk is selected at.
const PART_RUNG: &str = "Part"; // ui-text-exempt: a trace token, never displayed

/// Lowercase, because `canvas-selection` and `selection-set` disagree on the
/// case of this token and both are compared against it.
const PART_RUNG_LOWER: &str = "part"; // ui-text-exempt: a trace token

/// The redaction panel's census: `redact-panel marks=N pages=M epoch=E`.
const PANEL_EVENT: &str = "redact-panel"; // ui-text-exempt: a trace event name

/// The command string that puts the shell in Edit mode with the redaction panel
/// open, handed to the binary as `PDFCER_DIAG_INVOKE`.
///
/// The panel is opened because `marks=` is published **only while it is
/// drawn** — it is the panel's own census, not the document's, so a run that
/// never opened it reads silence and cannot tell a mark that did not happen
/// from a count nobody published.
const INVOKE: &str = "mode.edit,edit.redact";

/// The engine's answer to `Ctrl+Z`, and its refusal.
const UNDO_EVENT: &str = "undo"; // ui-text-exempt: a trace event name
const UNDO_DECLINED_EVENT: &str = "undo-declined"; // ui-text-exempt: a trace event name

/// The two lines marked together.
///
/// **Two apart, deliberately** — see the module header. `BETWEEN` is the line
/// that must never be selected, and is named here so the failure messages can
/// say which line a union would have destroyed.
const PAIR: [usize; 2] = [0, 2];
const BETWEEN: usize = 1;

/// How many regions one gesture over [`PAIR`] must build.
const EXPECTED_QUADS: usize = 2;

/// The least the plural mark's union may be, as a multiple of the singular's.
///
/// Two regions two lines apart span the two lines and the one between them, so
/// their union is about three line heights against the singular's one. Two is
/// the floor: comfortably above anything leading or a rounded trace figure
/// could contribute, and comfortably below the real answer, so the threshold
/// discriminates a second region from a re-measurement of the first.
const MIN_PLURAL_RATIO: f64 = 2.0;

/// The most the plural mark's union may be, as a multiple of the singular's.
///
/// The two aims are two baselines apart, so an honest union is one line height
/// plus 32 pt — between 3.0× and 3.9× a line height of 11 to 16 pt. The whole
/// block is five baselines, so a union spanning it is 80 pt plus a line height,
/// between 6.0× and 8.3×. Five sits between those two ranges: above anything
/// the right answer can produce, below anything the block-sized one can.
///
/// ★ Expressed against the CONTROL rather than against the block's own bounds,
/// because the block's bounds would have to be measured by marking it — a third
/// gesture and a third undo, to bound a number the control already bounds.
const MAX_PLURAL_RATIO: f64 = 5.0;

/// See the module documentation.
pub struct MarkingTwoChunksMakesOneMarkAndTwoRegions;

impl Check for MarkingTwoChunksMakesOneMarkAndTwoRegions {
    fn name(&self) -> &'static str {
        "marking_two_chunks_makes_one_mark_and_two_regions"
    }

    fn defect(&self) -> &'static str {
        "redacting several selected lines in one gesture either unions them into a single \
         rectangle that destroys the unselected text between them, or writes one mark per line so \
         the operator needs one undo for each — neither of which is the gesture he made"
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

/// How many redaction marks the panel says the document has.
fn marks(trace: &Trace) -> Option<usize> {
    trace.last(PANEL_EVENT).and_then(|l| l.get_usize("marks"))
}

/// What the last `canvas-selection` line says the selection is, as
/// `(level, how many held, the raw line)`.
fn selection(trace: &Trace) -> Option<(String, usize, String)> {
    trace.last(SELECTION_EVENT).map(|l| {
        (
            l.get("level").unwrap_or("unstated").to_owned(),
            l.get_usize("sel").unwrap_or(0),
            l.raw.clone(),
        )
    })
}

/// Press `Ctrl+Z` once and require the history to take it.
///
/// Returns the failure prose rather than reporting it, so each caller can say
/// which of its two undos went wrong.
fn undo_once(session: &Session, driver: &Driver, what: &str) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    driver.press_chord(&[vk::CONTROL], vk::Z)?;
    session.settle(26);
    let trace = session.trace()?;
    if trace.last_after(UNDO_EVENT, mark).is_some() {
        return Ok(None);
    }
    let how = if trace.last_after(UNDO_DECLINED_EVENT, mark).is_some() {
        "the history REFUSED it — `undo-declined` — so marking added nothing to the undo log, and \
         an operator who marks the wrong thing has no way back"
    } else {
        "neither `undo` nor `undo-declined` was written, so the chord did not reach `history_step` \
         at all. `Ctrl+Z` is bound in every mode, so an absent line means the keystroke was \
         swallowed before the keymap — most often by a text field that still has focus"
    };
    Ok(Some(format!(
        "★★★ {what} COULD NOT BE UNDONE: {how}. Trace: {}",
        session.trace_path().display()
    )))
}

/// Ascend out of whatever rung is standing, then click twice to reach the chunk
/// under `at`.
///
/// Two clicks because the chunk rung is entered on the second — `chunk_click`
/// owns that claim and it is assumed here rather than re-filed. The leading
/// Escapes make this callable again after a mark and an undo, without
/// inheriting a rung.
fn descend_to_a_chunk(
    session: &Session,
    driver: &Driver,
    at: ScreenPoint,
) -> Result<std::result::Result<String, String>> {
    driver.press(vk::ESCAPE)?;
    session.settle(12);
    driver.press(vk::ESCAPE)?;
    session.settle(12);
    driver.click_at(at)?;
    session.settle(26);
    driver.click_at(at)?;
    session.settle(26);

    let Some((level, _held, raw)) = selection(&session.trace()?) else {
        return Ok(Err(format!(
            "★★ THE SELECTION WENT SILENT: no `{SELECTION_EVENT}` line from either click on the \
             fixture's text. Trace: {}",
            session.trace_path().display()
        )));
    };
    if !level.eq_ignore_ascii_case(PART_RUNG_LOWER) {
        return Ok(Err(format!(
            "★★★ THE CLICK COULD NOT REACH ONE LINE OF THE BLOCK: `{raw}` — expected \
             `level={PART_RUNG}`.\n\
             This is O217's gesture failing before this row's subject can be measured at all. \
             `clicking_a_chunk_selects_that_chunk` is the row that owns the gesture; if it is \
             also red, believe that one first. Trace: {}",
            session.trace_path().display()
        )));
    }
    Ok(Ok(raw))
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks two lines of text, Shift-clicks to \
             build a set, opens a context menu over it and presses a row. Reported as SKIPPED \
             rather than passed: a check that did not run has learned nothing.",
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
            "the text fixture is not at {}. It is committed to this repository, so an absence is \
             a broken checkout rather than an unavailable precondition, and is reported as a \
             failure for that reason — a SKIP would say the opposite.",
            pdf.display()
        )));
    }
    report.note(format!(
        "--pdf and --doc-point are IGNORED: this check pins {} and marks chunks {} and {} of its \
         one six-line text object, leaving chunk {BETWEEN} between them unselected",
        pdf.display(),
        PAIR[0],
        PAIR[1]
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("redact-plural.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    // ★ Normalise the saved dock layout, or a panel TOGGLE alternates between
    // opening and closing across runs — a check that passes on odd-numbered
    // runs is worse than one that never passes. The application writes this
    // file; deleting it is putting the machine back, not editing the build.
    if let Some(dir) = exe.parent() {
        let layout = dir.join("userdata").join("layout.ron");
        if layout.exists() {
            let _ = std::fs::remove_file(&layout);
        }
    }

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    session.settle(60);
    let driver = Driver::new(session.window());

    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);
    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "no `{PAGE_REGION}` region, so no sheet is on screen. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // --- A: the census, before anything is marked --------------------------
    let before = marks(&session.trace()?).ok_or_else(|| {
        Error::new(format!(
            "the redaction panel published no `{PANEL_EVENT} … marks=` line, so half this check's \
             oracle is unreadable — it could not tell one mark from two. The panel is opened by \
             `{INVOKE}` at launch and publishes its census only while drawn; `redaction` is the \
             row that owns that surface. SKIPPED. Trace: {}",
            session.trace_path().display()
        ))
    })?;
    report.note(format!(
        "the document starts with {before} redaction mark(s)"
    ));

    // ★★★ EVERY AIM IS CONVERTED HERE, after the panel is open and before the
    // first gesture. Opening the redaction panel MOVES THE CANVAS RECT — the
    // dock takes width from it — so an aim computed before the panel opened
    // names a point on the old canvas and lands on the wrong line, or off the
    // page. It is the same stale-rect trap the egui RAG records against the
    // dock, and it is silent: the click still reaches the canvas.
    let aims = {
        let mut out = Vec::new();
        for index in PAIR {
            let (_, point) = crate::fixture::text_chunk_point(index);
            out.push(aim(ctx, &session, page, point)?);
        }
        out
    };

    // --- B: the precondition — the chunk boxes must be ON ------------------
    //
    // Not this check's subject, but its premise: the chunk rung is offered
    // exactly where a box is drawn, so a run that began with the switch off
    // would measure the switch and report it as a redaction defect. The
    // preference is persisted beside the exe, so a previous run that left it
    // off is a fact about the machine, not the build.
    if verdict(&session.trace()?, 0) == Some(Verdict::Declined("switched-off".to_owned())) {
        report.note(
            "the chunk boxes were OFF at launch. Turning them on: the chunk rung is offered only \
             where a box is drawn, so this check has no subject without them.",
        );
        if let Some(failure) = press_the_toggle(&session, &driver, ui_rect, true, report)? {
            return Ok(Some(failure));
        }
    }

    // --- C: the CONTROL — one line, marked singly --------------------------
    match descend_to_a_chunk(&session, &driver, aims[0])? {
        Ok(raw) => {
            report.note(format!("★ descended to one line: `{raw}`"));
        }
        Err(failure) => return Ok(Some(failure)),
    }
    match verdict(&session.trace()?, 0) {
        Some(Verdict::Drawn { count, .. }) if count == EXPECTED_CHUNKS => {}
        other => {
            return Err(Error::new(format!(
                "a line is selected and the block's chunks are not drawn: the painter's answer is \
                 {}, and this fixture's one text object holds {EXPECTED_CHUNKS} chunks. The \
                 Shift-click below has nothing to add to a set without them. SKIPPED: \
                 `chunk_boxes_show_what_a_text_block_is_made_of` is the row that owns it. Trace: \
                 {}",
                other.map_or_else(|| "silence".to_owned(), |v| format!("`{v}`")),
                session.trace_path().display()
            )));
        }
    }

    let singular = match mark_through_the_menu(&session, &driver, ui_rect, aims[0], "one line")? {
        Ok(marked) => marked,
        Err(failure) => return Ok(Some(failure)),
    };
    report.note(format!(
        "★★ the CONTROL — one line marked singly: `quads={}`, `bbox={}`, {:.1} pt tall",
        singular.quads,
        singular.bbox,
        singular.bbox.height()
    ));
    if singular.quads != 1 {
        return Ok(Some(format!(
            "★★★ MARKING ONE LINE BUILT {} REGIONS, NOT ONE: `bbox={}`.\n\
             This is the control for everything below, and it is measured first precisely so that \
             a `quads=2` on the plural mark cannot be a build that always writes two. A singular \
             selection producing more than one region means `mark_selection` is not reading the \
             selection's outlines — suspect `SelectionState::outlines` before the redaction verb.",
            singular.quads, singular.bbox
        )));
    }
    if !singular.bbox.height().is_finite() || singular.bbox.height() <= 0.0 {
        return Ok(Some(format!(
            "★★ THE CONTROL HAS NO HEIGHT: the one-line mark reports `bbox={}`, which is {:.1} pt \
             tall. The plural mark's span is measured as a ratio against that number, so there is \
             nothing here to compare it with — and a mark with no area would remove nothing when \
             applied, which is its own defect.",
            singular.bbox,
            singular.bbox.height()
        )));
    }

    // --- D: undo it, so the set is marked on the same document -------------
    if let Some(failure) = undo_once(&session, &driver, "THE CONTROL MARK")? {
        return Ok(Some(failure));
    }
    let cleared = marks(&session.trace()?);
    if cleared != Some(before) {
        return Ok(Some(format!(
            "★★ THE UNDO WAS TAKEN AND THE PANEL STILL COUNTS {}: it read {before} before the \
             control mark and must read {before} again after undoing it.\n\
             The history accepted the step — `undo` was written — so this is the panel's census \
             disagreeing with the document, and it is the field the plural assertion below is \
             about to be measured on. A census that does not fall on undo cannot be trusted to \
             rise by exactly one.",
            cleared.map_or_else(|| "nothing".to_owned(), |n| n.to_string())
        )));
    }
    report.note("★ undid the control mark — the set is marked on the same document");

    // --- E: build the SET: descend to chunk 0, Shift-click chunk 2 ---------
    match descend_to_a_chunk(&session, &driver, aims[0])? {
        Ok(raw) => {
            report.note(format!("★ re-descended to chunk {}: `{raw}`", PAIR[0]));
        }
        Err(failure) => return Ok(Some(failure)),
    }
    let mark = session.trace()?.mark();
    driver.click_with_modifier(aims[1], Key::Shift)?;
    session.settle(26);
    let trace = session.trace()?;
    let Some(line) = trace.last_after(SELECTION_EVENT, mark) else {
        return Err(Error::new(format!(
            "the Shift-click wrote no `{SELECTION_EVENT}` line, so this run never built a set and \
             has no operand. SKIPPED rather than failed: the gesture is \
             `shift_click_builds_a_chunk_set_the_whole_program_honours`'s subject, and blaming \
             redaction for it would send the next reader to the wrong file. Trace: {}",
            session.trace_path().display()
        )));
    };
    let held = line.get_usize("sel").unwrap_or(0);
    let level = line.get("level").unwrap_or("unstated").to_owned();
    let modifier = line.get("mod") == Some("true");
    if held != 2 || !level.eq_ignore_ascii_case(PART_RUNG_LOWER) || !modifier {
        return Err(Error::new(format!(
            "the Shift-click did not leave two chunks held: `{}` — expected `sel=2`, \
             `level={PART_RUNG}` and `mod=true`.\n\
             SKIPPED, not failed, and the distinction matters: building the set is \
             `shift_click_builds_a_chunk_set_the_whole_program_honours`'s subject. If that row is \
             red, believe it first — this one has no operand until it is green. Trace: {}",
            line.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!("★★ the set is built: `{}`", line.raw));

    // --- F: mark the SET, in ONE gesture -----------------------------------
    let plural = match mark_through_the_menu(
        &session,
        &driver,
        ui_rect,
        aims[0],
        "two lines held together",
    )? {
        Ok(marked) => marked,
        Err(failure) => return Ok(Some(failure)),
    };
    report.note(format!(
        "★★ the plural mark: `quads={}`, `bbox={}`, {:.1} pt tall",
        plural.quads,
        plural.bbox,
        plural.bbox.height()
    ));

    // --- G: the oracle, both halves ----------------------------------------
    if plural.quads != EXPECTED_QUADS {
        let reading = if plural.quads < EXPECTED_QUADS {
            format!(
                "★★★ THE TWO LINES WERE UNIONED INTO {} REGION: `bbox={}`.\n\
                 Chunk {BETWEEN} lies BETWEEN the two that were selected and was never part of \
                 the selection, so a single rectangle spanning both covers it — and applying that \
                 mark destroys text the operator did not choose. That is the failure O217 warns \
                 about, and it is silent, because the panel's count and the review list look \
                 exactly the same either way.\n\
                 `app::actions::redactsel::mark_selection` builds its quads from \
                 `SelectionState::outlines()`; a build that folded them into an enclosing \
                 rectangle before writing the annotation produces exactly this.",
                plural.quads, plural.bbox
            )
        } else {
            format!(
                "★★ THE GESTURE BUILT {} REGIONS FOR {} SELECTED LINES: `bbox={}`.\n\
                 The control mark on one line built exactly one, so the outline-per-line mapping \
                 is right for a singleton and wrong for a set. Suspect `SelectionState::outlines` \
                 returning a rectangle per RUN rather than per chunk before suspecting the \
                 redaction verb.",
                plural.quads,
                PAIR.len(),
                plural.bbox
            )
        };
        return Ok(Some(reading));
    }

    let after = marks(&session.trace()?);
    if after != Some(before + 1) {
        let reading = match after {
            Some(n) if n > before + 1 => format!(
                "★★★ ONE GESTURE ADDED {} MARKS: the panel counted {before} before and {n} after.\n\
                 The operator made one gesture and the document now holds one annotation per \
                 line: {} rows in the review list to read, and {} presses of `Ctrl+Z` to take \
                 back what he did once. The regions are right — `quads={}` — so this is the \
                 GROUPING: `mark_selection` is issuing one `add_redaction` per outline where it \
                 must issue one carrying every quad.",
                n - before,
                n - before,
                n - before,
                plural.quads
            ),
            Some(n) => format!(
                "★★★ THE MARK DID NOT REACH THE DOCUMENT: the panel counted {before} before the \
                 gesture and {n} after.\n\
                 The verb ran and said what it wanted — `quads={}`, `bbox={}` — so the request was \
                 built and the document did not take it. `redact_selection` is the row that owns \
                 the shell-to-document hop; if it is also red, believe that one first.",
                plural.quads, plural.bbox
            ),
            None => format!(
                "★★ THE PANEL STOPPED PUBLISHING ITS CENSUS: it read {before} before the gesture \
                 and wrote no `{PANEL_EVENT} … marks=` line after it.\n\
                 The census is written on change, so silence here is either a panel that closed \
                 mid-run or a count that did not move — and those are opposite findings. Read the \
                 trace for `{PANEL_EVENT}` before choosing between them."
            ),
        };
        return Ok(Some(reading));
    }

    // Safe arithmetic: the control's height was tested finite and positive
    // above, before this ratio could be reached.
    let ratio = plural.bbox.height() / singular.bbox.height();
    if ratio < MIN_PLURAL_RATIO {
        return Ok(Some(format!(
            "★★★ THE PLURAL MARK SPANS ONE LINE, NOT THREE: the control is {:.1} pt tall and the \
             two-line mark is {:.1} pt — {ratio:.1}× it, where regions on chunks {} and {} span \
             those two lines and chunk {BETWEEN} between them, about three line heights.\n\
             `quads` reads {} and the census rose by one, so the shape of the request is right \
             and its GEOMETRY is not: both regions are being built from the same outline. \
             Suspect `SelectionState::outlines` answering the FIRST entry twice.",
            singular.bbox.height(),
            plural.bbox.height(),
            PAIR[0],
            PAIR[1],
            plural.quads
        )));
    }

    if ratio > MAX_PLURAL_RATIO {
        return Ok(Some(format!(
            "★★★ THE PLURAL MARK SPANS THE WHOLE BLOCK, NOT THREE LINES OF IT: the \
             control is {:.1} pt tall and the two-line mark is {:.1} pt — {ratio:.1}× it, \
             where regions on chunks {} and {} span those two lines and chunk {BETWEEN} \
             between them, about three line heights.\n\
             `quads` reads {}, so this is not the union defect — it is two regions each \
             grown beyond the line it belongs to, and applying the mark destroys text \
             below chunk {} that was never selected. Suspect `SelectionState::outlines` \
             answering the BLOCK's bounds once per selected chunk.",
            singular.bbox.height(),
            plural.bbox.height(),
            PAIR[0],
            PAIR[1],
            plural.quads,
            PAIR[1]
        )));
    }

    report.note(format!(
        "★★★ one gesture over chunks {} and {} produced ONE mark holding {} regions, spanning \
         {ratio:.1}× one line, and one press of undo takes it back — the unit, the grouping and \
         the undo, all three",
        PAIR[0], PAIR[1], plural.quads
    ));

    // --- H: one gesture, ONE undo ------------------------------------------
    //
    // ★★ Asserted last and asserted at all: `marks` rising by one proves the
    // document holds a single annotation, and it does NOT prove the history
    // holds a single entry. A build that folded the annotation and not the
    // undo log reads +1 here and still costs the operator two presses.
    if let Some(failure) = undo_once(&session, &driver, "THE PLURAL MARK")? {
        return Ok(Some(failure));
    }
    let restored = marks(&session.trace()?);
    if restored != Some(before) {
        return Ok(Some(format!(
            "★★★ ONE GESTURE NEEDED MORE THAN ONE UNDO: the panel counted {} after a single \
             `Ctrl+Z`, and it read {before} before the gesture.\n\
             The mark is one annotation by the census and more than one entry on the history, so \
             taking back one action takes more than one press — which the operator experiences as \
             undo not working. `fold_undo` is the mechanism that coalesces a plural edit into one \
             entry; a build that skipped it for redaction produces exactly this.",
            restored.map_or_else(|| "nothing".to_owned(), |n| n.to_string())
        )));
    }
    report.note("★★★ one press of `Ctrl+Z` took the whole plural mark back");

    Ok(None)
}
