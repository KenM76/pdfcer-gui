//! `a_drawing_dropped_on_the_thumbnails_becomes_pages` — **drag a PDF from
//! Explorer onto the page grid and its sheets go in where you pointed.**
//!
//! # The request
//!
//! `OPERATOR_REQUESTS.md` **O67**, 2026-08-31:
//!
//! > *"I should be able to drag and drop documents into the thumbnails section
//! > of another pdf to import the pages."*
//!
//! ## ★★★ The one part that cannot be driven, and everything that can
//!
//! A harness moves a pointer and presses keys. It cannot **originate an OLE
//! drag** — that is Explorer's side of a protocol between two processes, and
//! no amount of `SendInput` produces one. So `app::filedrag` carries the same
//! kind of seam `app::dropped` already had (`PDFCER_DIAG_DROP_PATH`), which
//! makes the application behave as though a file had been dropped.
//!
//! ★★ **But the seam alone would test the wrong thing.** This feature is
//! entirely about *where* the file landed, and a drop simulated at startup
//! lands nowhere in particular — so a check built on it would pass on a build
//! that ignored the position completely, which is precisely the build that
//! existed before O67.
//!
//! ⇒ Hence `PDFCER_DIAG_DROP_AFTER_MS`. The drop is held back; the check puts
//! the **real cursor** on the tile it means and waits; and the application then
//! reads the position from the operating system with the same line of code a
//! genuine drop uses (`native_window::cursor_position`). The position is real,
//! the geometry is real, the insert is real. Only the payload is synthetic, and
//! the payload is the one part that cannot be otherwise.
//!
//! ## ★★ Why the pointer is jiggled while waiting
//!
//! `egui` repaints on events. An idle window with a file hovering over it
//! produces exactly one (`HoveredFile`, on entry) — which is why the
//! application requests a repaint while a hover is in flight — but the
//! *simulated* drop has no hover, so an idle application would never reach the
//! frame that fires it. Moving the cursor a point at a time keeps frames
//! coming without leaving the half of the tile the check is aiming at.
//!
//! ## The sequence
//!
//! | # | step | oracle |
//! |---|---|---|
//! | A | open the Pages panel, count the pages | `canvas … pages=N` |
//! | B | park the pointer on the LEFT half of tile 1 | — |
//! | C | the held-back drop fires | `file-dropped … at=x,y` |
//! | D | the panel claims it | `pages-import-dropped files=1 pages=M gap=1` |
//! | E | and the document actually grew | `canvas … pages=N+M` |
//!
//! ★ Step D asserts the **gap**, not merely that an import happened. The left
//! half of tile 1 means *before page 2*, so `gap=1` is the answer that
//! distinguishes "the drop used the pointer" from "the drop appended at the
//! end", and appending is what every position-blind build would do.
//!
//! ★★ Step E is the one that cannot be satisfied by wiring alone. A build that
//! raised the action and never reached the engine passes A–D and fails here.

use std::time::{Duration, Instant};

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode the Pages panel is reachable from, matching `pages_drag`.
const MODE: &str = "review";
/// The Pages panel's grid.
const GRID: &str = "panel-pages-grid"; // ui-text-exempt: a trace region name, never displayed
/// The prefix of the per-tile regions.
const TILE: &str = "panel-pages-tile."; // ui-text-exempt: a trace region prefix, never displayed
/// The seam that makes the application behave as though a file was dropped.
const DROP_PATH_ENV: &str = "PDFCER_DIAG_DROP_PATH"; // ui-text-exempt: an environment variable name
/// How long the application holds that drop back.
const DROP_AFTER_ENV: &str = "PDFCER_DIAG_DROP_AFTER_MS"; // ui-text-exempt: an environment variable name
/// The line the drop itself writes.
const DROPPED: &str = "file-dropped"; // ui-text-exempt: a trace event name, never displayed
/// The line the Pages panel writes when it claims one.
const IMPORTED: &str = "pages-import-dropped"; // ui-text-exempt: a trace event name, never displayed
/// The canvas line, which carries the page count.
const CANVAS: &str = "canvas"; // ui-text-exempt: a trace event name, never displayed

/// How long the application waits before firing the simulated drop.
///
/// Long enough for the mode click, the panel, and the pointer to be in place —
/// and it is a floor rather than a schedule, because the check then *waits for
/// the trace line* rather than assuming the drop has happened by now.
const DROP_AFTER_MS: u64 = 20_000;
/// How long to keep the pointer parked, waiting for the drop to fire.
const WAIT: Duration = Duration::from_secs(35);
/// Where across the tile the pointer parks.
///
/// A quarter, so the LEFT half is unambiguous: the panel resolves the nearer
/// vertical edge, and a point near the middle is where a rounding difference
/// between the application's `f32` rectangle and this harness's reading could
/// flip the answer — `pages_drag`'s reasoning, mirrored.
const PARK_ACROSS: f32 = 0.25;
/// The tile the drop is aimed at: the second page, so the resolved gap is 1.
///
/// ★ Not tile 0. Its left edge is gap 0, which is also what a build that
/// defaulted to `Start` would produce, and this check must not have a passing
/// answer that a position-blind build can reach.
const TILE_INDEX: usize = 1;
/// The gap that must come out of it.
const WANT_GAP: usize = 1;

pub struct ADrawingDroppedOnTheThumbnails;

impl Check for ADrawingDroppedOnTheThumbnails {
    fn name(&self) -> &'static str {
        "a_drawing_dropped_on_the_thumbnails_becomes_pages"
    }

    fn defect(&self) -> &'static str {
        "a PDF dropped on the page thumbnails opens in a new tab instead of importing its pages \
         — or it imports them at the end of the document however carefully the operator aimed, \
         because the toolkit discards the drop point and nothing asks the operating system for it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "one gesture with five oracles; splitting it would put the setup in one function and the assertions in another, and every one of them reads a rectangle the setup resolved" // ui-text-exempt: a lint justification, never displayed
)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check parks a real cursor on a real tile, \
             which is the entire point of it.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. This check needs a document to import INTO."))?;
    // ★ The dropped file must be a DIFFERENT document. pdfcer activates the tab
    // a path is already open in rather than opening it twice, so dropping the
    // open document on its own thumbnails would be testing a case with a
    // different correct answer.
    let source = ctx.second_pdf.clone().ok_or_else(|| {
        Error::new(
            "no --second-pdf. This check drops one document onto another's thumbnails, so it \
             needs two, and the same path twice would be a different subject.",
        )
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("drop-onto-thumbnails.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((DROP_PATH_ENV.to_owned(), source.display().to_string()));
    spec.env
        .push((DROP_AFTER_ENV.to_owned(), DROP_AFTER_MS.to_string()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {}, with {} held back for {DROP_AFTER_MS} ms",
        exe.display(),
        session.pid(),
        source.display()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- A: the panel, and the page count before anything happens ----------
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    if declared(&session.trace()?, ui_rect, GRID).is_none() {
        crate::checks::pages_drag::open_pages_panel(&session, &driver, ui_rect)?;
    }
    let trace = session.trace()?;
    if declared(&trace, ui_rect, GRID).is_none() {
        return Err(Error::new(format!(
            "no `{GRID}` region after asking for the Pages panel, so there are no thumbnails to \
             drop onto. Regions beginning `panel-`: {}.",
            list(&declared_names(&trace, ui_rect, "panel-"))
        )));
    }
    let Some(before) = page_count(&trace) else {
        return Err(Error::new(format!(
            "no `{CANVAS}` line carrying a page count, so this check cannot tell whether the \
             document grew."
        )));
    };
    let Some(tile) = declared(&trace, ui_rect, &format!("{TILE}{TILE_INDEX}")) else {
        return Err(Error::new(format!(
            "no `{TILE}{TILE_INDEX}` region — the second page's tile is not on screen. Tiles \
             declared: {}.",
            list(&declared_names(&trace, ui_rect, TILE))
        )));
    };
    report.note(format!(
        "the Pages panel is open on a {before}-page document; aiming at the left quarter of tile \
         {TILE_INDEX}"
    ));

    // --- B: park the pointer, and keep frames coming -----------------------
    let park = session.frame()?.declared_at(tile, PARK_ACROSS, 0.5);
    driver.move_to(park)?;

    // --- C: wait for the held-back drop ------------------------------------
    let deadline = Instant::now() + WAIT;
    let mut jiggle = 0i32;
    let landed = loop {
        if let Some(line) = session.trace()?.last(DROPPED) {
            break Some(line.get("at").unwrap_or("unknown").to_owned());
        }
        if Instant::now() >= deadline {
            break None;
        }
        // ★ One point back and forth. See the module header: an idle egui
        // application does not draw, and a drop that is never polled for never
        // fires. Both positions are deep inside the same half of the tile.
        jiggle = 1 - jiggle;
        #[allow(
            clippy::cast_precision_loss,
            reason = "a zero or a one" // ui-text-exempt: a lint justification, never displayed
        )]
        let nudge = jiggle as f32;
        driver.move_to(
            session
                .frame()?
                .declared_at(tile, PARK_ACROSS + nudge * 0.02, 0.5),
        )?;
        session.settle(8);
    };
    let Some(at) = landed else {
        return Ok(Some(format!(
            "★★★ NO DROP AT ALL: the application never traced `{DROPPED}` in {} s with \
             `{DROP_PATH_ENV}` set and `{DROP_AFTER_ENV}={DROP_AFTER_MS}`. Either the seam is \
             gone, or `app::filedrag::poll` is not being called at the top of the frame. Trace: \
             {}.",
            WAIT.as_secs(),
            session.trace_path().display()
        )));
    };
    if at == "unknown" {
        return Ok(Some(format!(
            "★★★ THE DROP HAD NO POSITION: `{DROPPED} … at=unknown`, so \
             `native_window::cursor_position` answered `None` while the pointer was demonstrably \
             on the window. Without a position this feature cannot exist — every drop would have \
             to fall through to opening the file. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ the drop landed, and it carries a position: at={at}"
    ));
    session.settle(30);

    // --- D: the panel claimed it, at the gap the pointer named -------------
    let trace = session.trace()?;
    let Some(imported) = trace.last(IMPORTED) else {
        return Ok(Some(format!(
            "★★★ THE PANEL DID NOT CLAIM IT: the drop landed at {at} — on the Pages panel — and \
             no `{IMPORTED}` line followed, so the file has fallen through to opening in a tab. \
             That is the behaviour O67 asked to replace. Look at `panels::pages::import::claim`: \
             it declines when the point is outside the panel rectangle, when the file has no \
             readable pages, or when it is not a PDF at all. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let gap = imported.get_usize("gap");
    if gap != Some(WANT_GAP) {
        return Ok(Some(format!(
            "★★★ THE PAGES WENT SOMEWHERE ELSE: the pointer was in the LEFT quarter of tile \
             {TILE_INDEX}, which names gap {WANT_GAP}, and the import reports gap={}. {} \
             Trace: {}.",
            gap.map_or_else(|| "none".to_owned(), |g| g.to_string()),
            if gap == Some(before) {
                "★★ AND IT IS THE END OF THE DOCUMENT, which is what a position-blind build \
                 produces: the pointer was never consulted, and every drop appends."
            } else {
                "The nearer-edge rule in `panels::pages::tile` resolved a different boundary \
                 than the one under the pointer."
            },
            session.trace_path().display()
        )));
    }
    let imported_pages = imported.get_usize("pages").unwrap_or(0);
    report.note(format!(
        "★★ the panel claimed it: {imported_pages} page(s) at gap {WANT_GAP}"
    ));

    // --- E: …and the document actually grew --------------------------------
    let deadline = Instant::now() + Duration::from_secs(20);
    let after = loop {
        let now = page_count(&session.trace()?);
        if now.is_some_and(|n| n != before) || Instant::now() >= deadline {
            break now;
        }
        session.settle(8);
    };
    let want = before + imported_pages;
    if after != Some(want) {
        return Ok(Some(format!(
            "★★★ THE ENGINE NEVER GOT IT: the panel raised the import and the document still \
             reports {} pages where {want} were expected ({before} + {imported_pages}). The \
             action was raised and did not land — `PageAction::InsertPagesFromFile` refused, or \
             the position was out of range. Trace: {}.",
            after.map_or_else(|| "an unreported number of".to_owned(), |n| n.to_string()),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ the document grew from {before} to {want} pages, at the gap the operator pointed at"
    ));
    Ok(None)
}

/// The page count the application last reported, from its canvas line.
fn page_count(trace: &crate::trace::Trace) -> Option<usize> {
    trace.last(CANVAS)?.get_usize("pages")
}
