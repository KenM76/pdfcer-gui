//! `a_page_dragged_between_documents_is_copied` — the operator's whole
//! request, performed end to end.
//!
//! > *"make it so we can open multiple PDFs at once and drag and drop pages
//! > from one thumbnail image sidebar to another … and insert them in between
//! > the pages we've dragged to."*
//!
//! # The gesture, and why it is one gesture
//!
//! Press on a page tile in the document that is open, walk the pointer onto
//! the **other document's tab** and rest there until it springs open, walk on
//! into the page grid that is now showing the other document, and release
//! between two sheets.
//!
//! That is four mechanisms in one held button, and no unit test can reach any
//! of them:
//!
//! | # | mechanism | what a unit test sees |
//! |---|---|---|
//! | 1 | the tile senses a press and publishes a `PageDrag` | a pure function it never calls |
//! | 2 | the drag **survives a document switch** — it lives in `egui::Memory`, not on `PanelsState`, which is reset by an activation | nothing: there is no activation in a unit test |
//! | 3 | a tab under the pointer for `SPRING_DWELL` activates its document | a timer no test advances |
//! | 4 | the release resolves a gap in the *new* document and raises a cross-document insert | two functions, separately tested, that have never met |
//!
//! Mechanism 2 is the one worth naming. `PanelsState::forget_document` is
//! `*self = Self::default()`, and switching documents calls it — so a drag
//! stored on the Pages panel would be **destroyed by the very tab-spring that
//! makes the feature possible**. It is in `egui::Memory` for exactly that
//! reason, and this check is the only thing in the workspace that can observe
//! whether that decision actually holds.
//!
//! # The assertion that says it is a COPY
//!
//! `copied=1` on the release line, and the source document's page count
//! **unchanged**. A cross-document drag does not remove the page from where it
//! came: `crate::app::actions::crossdoc` §2 carries the reason, which is that
//! a move would be two commands on two undo stacks with no single Ctrl+Z able
//! to reverse it.
//!
//! An operator who assumed a move would discover their source drawing intact
//! tomorrow — or, worse, assume it was not and delete the wrong copy. So the
//! caption says *copy* before the button is released, and this check asserts
//! the behaviour matches the promise.
//!
//! # What a passing run does NOT prove
//!
//! That the pages arrived at the right index. `insert-pages-landed at=` is
//! read and cross-checked against the gap the release reported, which rules
//! out the two answers that are wrong by a whole document (the start and the
//! end). It does not verify the *content* of the inserted sheets; that is
//! `pdfcer-core`'s `insert_pages` and it has its own corpus.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use std::time::Duration;

/// The mode the Pages panel is reachable from, matching `pages_drag`.
const MODE: &str = "review";
/// The Pages panel's grid.
const GRID: &str = "panel-pages-grid";
/// The prefix of the per-tile regions.
const TILE: &str = "panel-pages-tile.";
/// The prefix of the per-tab regions.
const TAB: &str = "doc-tab.";
/// The trace line the spring produces when a tab opens under a drag.
const SPRING: &str = "doc-tab-spring";
/// The trace line the release produces.
const RELEASE: &str = "pages-drag-release";
/// The trace line the insert produces once it has landed.
const LANDED: &str = "insert-pages-landed";
/// The picker-answering environment variable. See [`super::document_tabs`].
const OPEN_PATH_ENV: &str = "PDFCER_DIAG_OPEN_PATH";
/// The chord that opens a document.
const CTRL_O: u16 = 0x4F;

/// **How long the pointer rests on the tab**, as a multiple of the
/// application's own `SPRING_DWELL`.
///
/// Twice, and generously. The application's timer runs on `egui`'s input clock
/// and only advances on frames it actually draws; a machine that is
/// rasterizing a dense CAD sheet at the same moment can drop several. Waiting
/// exactly the threshold would make this check a stopwatch race against a
/// renderer, and that is the classic shape of a flake: **a full suite red is
/// not a defect report until the member has been re-run alone.**
const DWELL: Duration = Duration::from_millis(1_400);

/// The trace line the source-side removal produces, on a move.
/// `page-move-take-refused page=… n=… detail=…` — the engine declining the
/// removal half of a move, with its own sentence.
///
/// Distinct from [`TOOK`]'s `removed=0`, and the distinction decides the
/// verdict: `removed=0` says *the source still has them*, which has two causes
/// pointing opposite ways — a build that never attempted the delete (a defect)
/// and an engine that refused it for a reason about the document (not one).
const TOOK_REFUSED: &str = "page-move-take-refused";
const TOOK: &str = "page-move-took";

/// **The same gesture, with and without Shift.**
///
/// One implementation and two registrations rather than two files, because the
/// only thing that differs is a key held during the drag and two assertions at
/// the end — and a copied file would drift on the twenty-odd things that are
/// the same.
///
/// Both are registered, and the copy one is not redundant. The failure this
/// pair is really shaped to catch is a build where the modifier is read at the
/// **press** instead of the release, or read from the wrong field, or ignored:
/// such a build makes one of the two behave like the other, and only running
/// both can see it. A single check would pass on a build that always copied, or
/// always moved.
pub struct PageDraggedBetweenDocuments {
    /// Whether Shift is held for the whole gesture.
    take: bool,
}

impl PageDraggedBetweenDocuments {
    /// The unmodified drag, which must leave the source document alone.
    pub const COPY: Self = Self { take: false };
    /// The Shift-held drag, which must remove the pages from the source.
    pub const MOVE: Self = Self { take: true };
}

impl Check for PageDraggedBetweenDocuments {
    fn name(&self) -> &'static str {
        if self.take {
            "a_shift_drag_between_documents_moves_the_pages"
        } else {
            "a_page_dragged_between_documents_is_copied"
        }
    }

    fn defect(&self) -> &'static str {
        if self.take {
            "holding Shift while dragging a page into another document still copies it — so \
             Windows' own move modifier is inert, and an operator who used it is left with \
             the sheets in both documents and no indication of which one they meant"
        } else {
            "a page cannot be dragged from one open document into another — the drag does not \
             survive the document switch, or no tab springs open under it, or the release \
             lands nowhere — so combining two drawings means Insert-from-file and a dialog"
        }
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report, self.take) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport, take: bool) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no fixture document. Pass --pdf."))?;
    let second = ctx.second_pdf.clone().ok_or_else(|| {
        Error::new(
            "no second document. Pass --second-pdf <path> — a file DIFFERENT from --pdf. \
             This check drags a page from one document into another and there is no second \
             document to drag between.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check holds the pointer down across three \
             surfaces. Reported as SKIPPED rather than passed: a check that did not run has \
             learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("page_drag_between_documents.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((OPEN_PATH_ENV.to_owned(), second.display().to_string()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;

    // --- 1: two documents open, the second on screen -----------------------
    driver.press_chord(&[crate::input::Key::Ctrl.vk()], CTRL_O)?;
    session.settle(60);
    let trace = session.trace()?;
    let tabs = declared_names(&trace, ui_rect, TAB);
    if tabs.len() < 2 {
        return Err(Error::new(format!(
            "only {} document tab(s) after Ctrl+O with `{OPEN_PATH_ENV}` set to {}. This \
             check needs two documents open and cannot make its own; \
             `two_documents_get_two_tabs` is the check that diagnoses why. Tabs: {}.",
            tabs.len(),
            second.display(),
            list(&tabs)
        )));
    }
    report.note(format!("{} documents open", tabs.len()));

    // --- 2: the Pages panel, showing the ACTIVE (second) document ----------
    //
    // OPEN IT IF IT IS NOT THERE, rather than requiring it to be.
    //
    // The mode's default arrangement is not a guarantee, and a check that
    // refuses when the panel is closed reports nothing at all — it SKIPS its
    // whole sweep on a message about furniture.
    //
    // **A precondition a check ASSUMES is a precondition that eventually stops
    // being true, and the check then reports nothing rather than failing.**
    // Establishing it costs one call to a helper that is already `pub(crate)`
    // for this purpose. The guard is "only if absent", because pressing a panel
    // toggle that is already on CLOSES it.
    if declared(&session.trace()?, ui_rect, GRID).is_none() {
        crate::checks::pages_drag::open_pages_panel(&session, &driver, ui_rect)?;
        session.settle(24);
    }
    let trace = session.trace()?;
    let Some(grid) = declared(&trace, ui_rect, GRID) else {
        return Err(Error::new(format!(
            "no `{GRID}` region after asking the ribbon for the Pages panel, so there are \
             no tiles to drag. Regions beginning `panel-`: {}.",
            list(&declared_names(&trace, ui_rect, "panel-"))
        )));
    };
    let tiles = declared_names(&trace, ui_rect, TILE);
    if tiles.is_empty() {
        return Err(Error::new(
            "the Pages panel is on screen and published no tiles, so there is no page to drag."
                .to_owned(),
        ));
    }
    let from = declared(&trace, ui_rect, &format!("{TILE}0")).ok_or_else(|| {
        Error::new(format!(
            "no `{TILE}0` region — the second document's first page is not on screen. Tiles: \
             {}.",
            list(&tiles)
        ))
    })?;

    // --- 3: the FIRST document's tab, which is the one to spring open ------
    let tab0 = declared(&trace, ui_rect, &format!("{TAB}0")).ok_or_else(|| {
        Error::new(format!(
            "no `{TAB}0` region, so the first document's tab is not drawn and there is \
             nothing to drag onto. Tabs: {}.",
            list(&tabs)
        ))
    })?;

    // --- 4: the gesture ----------------------------------------------------
    //
    // The landing point is computed from the GRID rectangle, and that is a
    // coordinate this check knowingly holds across an act that moves things —
    // the spring re-lays the whole grid for a different document. It survives
    // by construction: the grid rect is the panel's area, and the panel does
    // not resize when the active document changes. A point derived from a
    // *tile* would not survive, because a tile is exactly what gets re-laid.
    //
    // `D:\dev\rag\egui\a_harness_may_hold_a_coordinate_only_until_it_performs_an_act_that_could_move_it.md`
    // is the rule, and it is honoured by NOT asserting on which tile the drop
    // landed on: the release's own `gap=` field is read from the trace
    // afterwards, and the only geometric claim made is that the pointer was
    // inside the grid — which is true of both documents' grids, because the
    // grid rectangle is the panel's and not the document's.
    let frame = session.frame()?;
    let start = frame.declared_center(from);
    let spring_at = frame.declared_center(tab0);
    // THE GRID'S CENTRE, not a point derived from a TILE — step 4's note
    // above carries the reason. A landing point computed from a tile of the
    // *second* document lands where that document has no tile once the spring
    // has re-laid the grid: no gap resolves, no cross-document insert is
    // raised, and the trace reads `pages-drag-release gap=none`. That reports
    // as "the application still believed both ends were the same document",
    // which is a defect report about working code.
    //
    // NEAR THE TOP of the grid, not its centre, because the centre resolves no
    // gap either: the target document here has ONE page, so its single tile
    // sits at the top of a tall grid and the middle is empty space below it —
    // where the grid's tile hit test deliberately raises nothing, because that
    // is where an operator lets go when they mean "put it at the end" and miss.
    // Every non-empty document has a first row at the top of its grid, whatever
    // its page count, which is the property a release point needs.
    let land_at = frame.declared_at(grid, 0.5, 0.12);
    report.note(format!(
        "dragging tile 0 from ({}, {}), resting {} ms on the first document's tab at \
         ({}, {}), releasing at ({}, {})",
        start.x(),
        start.y(),
        DWELL.as_millis(),
        spring_at.x(),
        spring_at.y(),
        land_at.x(),
        land_at.y()
    ));
    driver.drag_via(
        start,
        spring_at,
        DWELL,
        land_at,
        take.then_some(crate::input::Key::Shift),
    )?;
    session.settle(60);

    // --- 5: the tab sprang open --------------------------------------------
    let trace = session.trace()?;
    let Some(sprang) = trace.last(SPRING) else {
        return Ok(Some(format!(
            "the pointer rested {} ms on the first document's tab with a page drag in flight \
             and no `{SPRING}` line was traced. Either the strip does not offer a hovered tab \
             while a drag is in flight, or the dwell timer never ran — a spring measured \
             against a frame clock needs a repaint request, and a stationary pointer produces \
             none.",
            DWELL.as_millis()
        )));
    };
    report.note(format!("the tab sprang open: `{}`", sprang.raw));

    // --- 6: the drag SURVIVED the document switch -----------------------
    let Some(release) = trace.last(RELEASE) else {
        return Ok(Some(format!(
            "the tab sprang open and no `{RELEASE}` line followed, so the drag did not survive \
             the document switch. That is the exact defect `crate::pagedrag` exists to \
             prevent: `PanelsState::forget_document` is `*self = Self::default()` and \
             activating a tab calls it, so a drag stored on the Pages panel would be destroyed \
             by the very spring that makes this feature possible."
        )));
    };
    report.note(format!("the drag survived the switch: `{}`", release.raw));

    // --- 7: it was a COPY, into the other document -------------------------
    if release.get("copied") != Some("1") {
        return Ok(Some(format!(
            "the drag was released in the first document and did not report `copied=1`. A \
             release in a DIFFERENT document from the one the pages came from must raise a \
             cross-document insert; `reordered=` on this line means the application still \
             believed both ends were the same document, which would silently reorder the \
             wrong one. Line: `{}`.",
            release.raw
        )));
    }
    let Some(from_slot) = release.get("from-slot") else {
        return Ok(Some(format!(
            "the release reported a copy and did not say which document the pages came from. \
             Line: `{}`.",
            release.raw
        )));
    };
    if from_slot == "0" {
        return Ok(Some(format!(
            "the release says the pages came from slot 0, which is the document they were \
             dropped INTO. Source and target are the same, so this is a document being copied \
             into itself. Line: `{}`.",
            release.raw
        )));
    }
    report.note(format!("came out of slot {from_slot}"));

    // --- 7b: the MODIFIER reached the application ------------------------
    //
    // Asserted before anything is asked about the source, so a build that
    // ignores Shift fails here — naming the modifier — rather than three steps
    // later with "the source still has its pages", which is the same fact
    // reported as a mystery.
    let wanted = if take { "1" } else { "0" };
    if release.get("take") != Some(wanted) {
        return Ok(Some(format!(
            "Shift was {} for the whole drag and the release reported `take={}`. The modifier \
             is not reaching the drop. It is sampled at the RELEASE — as Windows does — so \
             the likely causes are reading `i.modifiers` from the wrong frame, or latching it \
             at the press. Line: `{}`.",
            if take { "HELD" } else { "not held" },
            release.get("take").unwrap_or("absent"),
            release.raw
        )));
    }
    report.note(if take {
        "Shift was held, and the release asked for a move"
    } else {
        "no modifier, and the release asked for a copy"
    });

    // --- 8: the pages actually arrived -------------------------------------
    let Some(landed) = trace.last(LANDED) else {
        return Ok(Some(format!(
            "the release raised a cross-document insert (`copied=1`) and no `{LANDED}` line \
             followed, so the engine call never ran, was refused, or added nothing. The \
             gesture is complete and both documents are unchanged — which is the worst of the \
             failures available here, because everything the operator saw said it worked."
        )));
    };
    let grew = landed
        .get("pages")
        .zip(landed.get("was"))
        .and_then(|(now, was)| Some((now.parse::<usize>().ok()?, was.parse::<usize>().ok()?)));
    match grew {
        Some((now, was)) if now > was => {
            report.note(format!(
                "the target document went from {was} to {now} pages"
            ));
        }
        Some((now, was)) => {
            return Ok(Some(format!(
                "the insert landed and the target document still has {now} page(s), against \
                 {was} before. A landing line with no growth means `insert_pages` returned \
                 without inserting. Line: `{}`.",
                landed.raw
            )));
        }
        None => {
            return Ok(Some(format!(
                "the `{LANDED}` line does not carry the page counts this check reads. Line: \
                 `{}`.",
                landed.raw
            )));
        }
    }

    // --- 9: the SOURCE, which is where copy and move differ -------------
    //
    // The whole point of the pair. A copy must leave it alone and a move must
    // take the pages out of it, and both assertions are made from the SAME
    // trace line's absence or presence — which is only admissible because the
    // other member of the pair demonstrates, in the same run of the suite,
    // that the line can be produced. See `checks::mod`'s rule 4.
    let took = trace.last(TOOK);
    if take {
        let Some(took) = took else {
            return Ok(Some(format!(
                "Shift was held, the release asked for a move (`take=1`), the pages arrived — \
                 and no `{TOOK}` line followed. The insert half happened and the removal half \
                 did not, so the sheets are now in BOTH documents. That is the one outcome \
                 neither a copy nor a move, and it is the state an operator discovers by \
                 counting."
            )));
        };
        if took.get("removed") == Some("0") {
            // ASK WHY BEFORE ACCUSING, and the commonest why is the fixture.
            //
            // Failing on any `removed=0` — *"the sheets are now in both
            // documents"* — accuses working code. A **one-page** source cannot
            // be moved out of: removing its only sheet would leave a document
            // with no pages, which the engine refuses by name and correctly —
            //
            //   page-move-take-refused page=0 n=1
            //     detail=removing 1 of 1 page(s) would leave the document with none
            //
            // — and the shell notices, words it for the operator, and leaves
            // the sheets where they were. The whole chain behaved.
            //
            // That is this harness's three-state discipline: *a fact about the
            // fixture is not a fact about the build.* The refusal is quoted
            // verbatim rather than paraphrased, for the reason `text_edit_real`
            // records — the engine's sentence IS the diagnosis, and a second
            // account of it would drift.
            if let Some(refused) = trace.last(TOOK_REFUSED) {
                return Err(Error::new(format!(
                    "the source document refused to give up its pages: `{}`. If that says the \
                     document would be left with none, pass a `--second-pdf` with MORE THAN ONE \
                     page — a one-page source cannot be moved out of, by design, and no build \
                     could make it. Reported as SKIPPED rather than FAILED for exactly that \
                     reason.",
                    refused.raw
                )));
            }
            return Ok(Some(format!(
                "the move removed 0 pages from the source, so they are in both documents, and \
                 nothing refused it — so the delete was never attempted rather than declined. \
                 Line: `{}`.",
                took.raw
            )));
        }
        report.note(format!("and the source lost them: `{}`", took.raw));
    } else if let Some(took) = took {
        return Ok(Some(format!(
            "no modifier was held and the source document was edited anyway: `{}`. An \
             unmodified cross-document drag must be a COPY — the source's undo stack, its \
             page count and its unsaved marker are all promised untouched, and an operator \
             who assumed a copy would discover the loss on the drawing they did not have \
             open.",
            took.raw
        )));
    } else {
        report.note("and the source was left alone, as a copy must");
    }
    Ok(None)
}
