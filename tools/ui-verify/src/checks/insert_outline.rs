//! `inserted_pages_bring_their_bookmarks` — **pages inserted from a file
//! bring the bookmarks and named destinations that reach them, after this
//! document's own, and one undo takes all of it away.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/insert_outline.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::{Trace, TraceLine};

/// Two pages, a bookmark aimed at each, and the destination key `sheet`.
const SOURCE: &str = "fixtures/outlined-source.pdf";
/// One page, the bookmark `Target cover`, and its own destination `sheet`.
const TARGET: &str = "fixtures/outlined-target.pdf";
const INSERT_PATH_ENV: &str = "PDFCER_DIAG_INSERT_PATH";
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const MODE_COMMAND: &str = "mode.edit";
const INSERT_ITEM: &str = "ribbon.item.pages.insert_from_file";
const INSERT_BUTTON: &str = "insert-pages.insert";
const BOOKMARKS_PANEL: &str = "view.panel_bookmarks";
/// The funnel's success line, whose `disclosures=` carries the status sentence.
const INSERTED: &str = "insert-pages"; // ui-text-exempt: a trace event name, never displayed
/// `pages::insert_from_view`'s count of what the insert carried.
const CARRIED: &str = "insert-pages-carried"; // ui-text-exempt: a trace event name, never displayed
const EXPECTED: [(&str, &str); 4] = [
    ("outline_items", "2"),
    ("dests", "1"),
    ("dests_renamed", "1"),
    ("outline_dropped", "0"),
];
/// Said by `text::insertoutline::carried` for two bookmarks.
const SAID: &str = "2 bookmarks came across"; // ui-text-exempt: matched against the app's own sentence
const CENSUS: &str = "bookmarks-panel"; // ui-text-exempt: a trace event name, never displayed
const ROW: &str = "bookmark-row"; // ui-text-exempt: a trace event name, never displayed
const CANVAS: &str = "canvas"; // ui-text-exempt: a trace event name, never displayed
const UNDONE: &str = "undo-applied"; // ui-text-exempt: a trace event name, never displayed
/// The outline after the insert, in order: the target's, then the carried two.
const AFTER: [&str; 3] = ["Target cover", "Source sheet A", "Source sheet B"];

/// See the module documentation.
pub struct InsertedPagesBringTheirBookmarks;

impl Check for InsertedPagesBringTheirBookmarks {
    fn name(&self) -> &'static str {
        "inserted_pages_bring_their_bookmarks"
    }

    fn defect(&self) -> &'static str {
        "pages inserted from another file arrive without the bookmarks and named destinations \
         that pointed at them, or arrive with them out of order, or leave them behind on undo"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch(ctx, &mut report)
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer));
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Launch on a copy of [`TARGET`], off the desktop, with the Insert-pages
/// picker answered by [`SOURCE`].
fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe, or build the default."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let target = repo_fixture(TARGET)?;
    let source = repo_fixture(SOURCE)?;
    let pdf = ctx.out("insert-outline-target.pdf");
    std::fs::copy(&target, &pdf).map_err(|e| Error::new(format!("copying {TARGET}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("insert-outline.trace.txt"));
    spec.pdf = Some(pdf);
    let source = source.to_string_lossy().into_owned();
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", MODE_COMMAND),
        (INSERT_PATH_ENV, source.as_str()),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("insert-outline.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    Ok((session, pointer))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    crate::checks::reaching::raise_dock_tab(session, pointer, ui_rect, BOOKMARKS_PANEL)?;
    session.settle(20);
    let before = outline(&session.trace()?);
    report.note(format!("before the insert the panel lists {before:?}"));
    if before != AFTER[..1] {
        return Err(Error::new(format!(
            "before the insert the Bookmarks panel must list only `{}`; it listed {before:?}, \
             so the panel is not on screen or {TARGET} has changed. SKIPPED rather than failed.",
            AFTER[0]
        )));
    }
    insert(session, pointer, ui_rect)?;
    if let Some(failure) = carried(report, &session.trace()?)? {
        return Ok(Some(failure));
    }
    let after = outline(&session.trace()?);
    report.note(format!("after the insert the panel lists {after:?}"));
    if after != AFTER {
        return Ok(Some(format!(
            "after the insert the Bookmarks panel must list {AFTER:?} — the target's bookmark, \
             then the two carried ones in their source order; it listed {after:?}."
        )));
    }
    undone(report, session, pointer)
}

/// Open Insert pages from the Pages tab and press Insert.
fn insert(session: &Session, pointer: &ScriptedPointer, ui_rect: &str) -> Result<()> {
    crate::checks::forms_spotlight::open_from_tab(session, pointer, ui_rect, "pages", INSERT_ITEM)?;
    session.settle(24);
    let trace = session.trace()?;
    let Some((rect, viewport)) = declared_in(&trace, ui_rect, INSERT_BUTTON) else {
        return Err(Error::new(format!(
            "the Insert-pages window declared no `{INSERT_BUTTON}` region, so it did not open or \
             did not read {SOURCE}. Regions beginning `insert-pages`: {}.",
            list(&declared_names(&trace, ui_rect, "insert-pages"))
        )));
    };
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(40);
    Ok(())
}

/// `None` when the insert carried exactly the two bookmarks and the renamed
/// destination, and said so.
fn carried(report: &mut CheckReport, trace: &Trace) -> Result<Option<String>> {
    let Some(inserted) = trace.last(INSERTED) else {
        return Ok(Some(format!(
            "Insert was pressed and the funnel traced no `{INSERTED}` line: no pages arrived."
        )));
    };
    let disclosures = inserted.get("disclosures").unwrap_or_default();
    report.note(format!("inserted: {disclosures}"));
    let Some(line) = trace.last(CARRIED) else {
        return Ok(Some(format!(
            "the insert applied and traced no `{CARRIED}` line, so the build under test does \
             not call `insert_pages_with_options` — it is older than the wiring, or the call \
             was reverted."
        )));
    };
    report.note(format!("carried: `{}`", line.raw));
    let wrong: Vec<_> = EXPECTED
        .iter()
        .filter(|(key, want)| line.get(key) != Some(*want))
        .collect();
    if !wrong.is_empty() {
        // A dropped outline reads `outline_items=0 dests=0`, which no carrying
        // insert of this source can produce.
        return Ok(Some(format!(
            "★★★ the insert must carry `outline_items=2 dests=1 dests_renamed=1 \
             outline_dropped=0`; it traced `{}`. Wrong: {wrong:?}.",
            line.raw
        )));
    }
    Ok((!disclosures.contains(SAID)).then(|| {
        format!(
            "the counts are right and the status sentence does not say `{SAID}`: \
             {disclosures:?}. The operator is not told what came with the pages."
        )
    }))
}

/// One Ctrl+Z removes the two pages and the two carried bookmarks together.
fn undone(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let undos = trace.events(UNDONE).count();
    let mark = trace.mark();
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(40);
    let trace = session.trace()?;
    if trace.events(UNDONE).count() != undos + 1 {
        return Ok(Some(format!("one Ctrl+Z did not trace one `{UNDONE}`.")));
    }
    let pages = trace
        .last_after(CANVAS, mark)
        .and_then(|c| c.get("pages").map(str::to_owned));
    let left = outline(&trace);
    report.note(format!(
        "after one undo: pages={pages:?}, bookmarks {left:?}"
    ));
    if pages.as_deref() != Some("1") || left != AFTER[..1] {
        return Ok(Some(format!(
            "★★★ one Ctrl+Z must leave one page and only `{}`; it left pages={pages:?} and \
             {left:?}. The pages and their bookmarks are not one undo step.",
            AFTER[0]
        )));
    }
    Ok(None)
}

/// The titles of the last COMPLETE frame's bookmark rows, top to bottom.
///
/// The panel traces one `bookmarks-panel` census and then one `bookmark-row`
/// per row every frame, so a whole frame is the rows between the last two
/// censuses; the rows after the last census may be a frame still being
/// written.
fn outline(trace: &Trace) -> Vec<String> {
    let census: Vec<usize> = positions(trace, CENSUS);
    let [.., from, to] = census[..] else {
        return Vec::new();
    };
    trace.lines[from..to]
        .iter()
        .filter(|l| l.event == ROW)
        .filter_map(|l| l.get("title").map(str::to_owned))
        .collect()
}

fn positions(trace: &Trace, event: &str) -> Vec<usize> {
    trace
        .lines
        .iter()
        .enumerate()
        .filter(|(_, l): &(usize, &TraceLine)| l.event == event)
        .map(|(i, _)| i)
        .collect()
}

/// A fixture relative to this repository's root; the check builds its own
/// subject, so a missing one is a harness error.
fn repo_fixture(rel: &str) -> Result<std::path::PathBuf> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(rel);
    if path.is_file() {
        Ok(path)
    } else {
        Err(Error::new(format!(
            "the fixture `{rel}` is not on disk; regenerate it with its PROVENANCE script."
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::{AFTER, outline};
    use crate::trace::Trace;

    fn row(title: &str) -> String {
        format!("pdfcer-diag bookmark-row id=5 level=0 title={title:?} page=Some(1)")
    }

    #[test]
    fn a_frame_is_the_rows_between_the_last_two_censuses() {
        let census = "pdfcer-diag bookmarks-panel page=1 items=3";
        let text = [
            census.to_owned(),
            row(AFTER[0]),
            census.to_owned(),
            row(AFTER[0]),
            row(AFTER[1]),
            row(AFTER[2]),
            census.to_owned(),
            row(AFTER[0]),
        ]
        .join("\n");
        let trace = Trace::parse(&text, "pdfcer-diag");
        assert_eq!(outline(&trace), AFTER);
    }
}
