//! Tools > Compare, driven: the open document is `compare-new.pdf`, the older
//! version `compare-old.pdf` arrives through `PDFCER_DIAG_COMPARE_PATH`, and
//! three checks judge the list, the jump and the marks.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/compare_documents.md`.

use super::dimdrive::{Fixture, launch_with, press, press_on_tab, ui_rect_event};
use super::driving;
use super::multiselect::{after, save, step};
use super::savedannots::{listed_on_page, read};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const NEWER: Fixture = Fixture {
    file: "compare-new.pdf",
    method: "Rebuild it with `python fixtures/compare-pair.PROVENANCE.py fixtures`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const OLDER: &str = "compare-old.pdf";
const COMPARE_PATH_ENV: &str = "PDFCER_DIAG_COMPARE_PATH"; // ui-text-exempt: an environment variable name
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
const ITEM: &str = "ribbon.item.tools.compare"; // ui-text-exempt: a trace region name
const TAB: &str = "ribbon.tab.tools"; // ui-text-exempt: a trace region name
const MARK: &str = "compare.mark"; // ui-text-exempt: a trace region name
const ROW_PREFIX: &str = "compare.row:"; // ui-text-exempt: a trace region name prefix
const LISTED: &str = "compare-listed"; // ui-text-exempt: a trace event name
const JUMPED: &str = "compare-jump"; // ui-text-exempt: a trace event name
const MARKED: &str = "compare-marked"; // ui-text-exempt: a trace event name
const REVEALED: &str = "min-reveal-solved"; // ui-text-exempt: a trace event name

/// The rows the fixture pair must list: `kind@page`, page 1-based. The area
/// rows are counted apart, because a changed line of text also differs in
/// appearance and the engine may report it as an area too.
const TEXT_ROWS: [(&str, usize); 3] = [("replaced@2", 2), ("inserted@2", 1), ("deleted@", 1)];

/// `comparing_lists_every_change`.
pub struct ComparingListsEveryChange;
/// `a_compare_row_shows_its_change`.
pub struct ACompareRowShowsItsChange;
/// `marking_a_comparison_writes_comments`.
pub struct MarkingAComparisonWritesComments;

impl Check for ComparingListsEveryChange {
    fn name(&self) -> &'static str {
        "comparing_lists_every_change"
    }

    fn defect(&self) -> &'static str {
        "Tools > Compare against an older version lists fewer changes than the two files hold, \
         lists a change on a page that did not change, or draws no row to click"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        drive(self, ctx, "compare-list", list_body)
    }
}

impl Check for ACompareRowShowsItsChange {
    fn name(&self) -> &'static str {
        "a_compare_row_shows_its_change"
    }

    fn defect(&self) -> &'static str {
        "Clicking a change in the Compare panel does not turn to its page or does not bring it \
         into view"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        drive(self, ctx, "compare-jump", jump_body)
    }
}

impl Check for MarkingAComparisonWritesComments {
    fn name(&self) -> &'static str {
        "marking_a_comparison_writes_comments"
    }

    fn defect(&self) -> &'static str {
        "Mark changes as comments writes no annotations, writes them to a page that did not \
         change, or takes more than one Undo to take them back"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        drive(self, ctx, "compare-mark", mark_body)
    }
}

type Body =
    fn(&CheckContext, &mut CheckReport, &Session, &ScriptedPointer) -> Result<Option<String>>;

/// Launch the newer file with the older one queued for the pick and two save
/// paths, run Tools > Compare, wait for the list, then drive `body`.
fn drive(check: &dyn Check, ctx: &CheckContext, stem: &str, body: Body) -> CheckReport {
    let mut report = CheckReport::new(check.name(), check.defect());
    let older = ctx.out(&format!("{stem}.older.pdf"));
    let saves = [
        ctx.out(&format!("{stem}.marked.pdf")),
        ctx.out(&format!("{stem}.undone.pdf")),
    ];
    for p in &saves {
        let _ = std::fs::remove_file(p);
    }
    let copied = driving::repo_fixture(OLDER, NEWER.method).and_then(|from| {
        std::fs::copy(from, &older)
            .map(|_| ())
            .map_err(|e| Error::new(format!("copying {OLDER}: {e}")))
    });
    let env = [
        (COMPARE_PATH_ENV, older.display().to_string()),
        (
            SAVE_PATH_ENV,
            format!("{};{}", saves[0].display(), saves[1].display()),
        ),
    ];
    let outcome = copied
        .and_then(|()| launch_with(ctx, &mut report, &NEWER, "mode.edit", stem, &env))
        .and_then(|(session, pointer)| {
            let outcome =
                run_compare(ctx, &mut report, &session, &pointer).and_then(|failed| match failed {
                    Some(why) => Ok(Some(why)),
                    None => body(ctx, &mut report, &session, &pointer),
                });
            let parked = pointer.gone(&session);
            match outcome? {
                Some(failure) => Ok(Some(failure)),
                None => parked.map(|_| None),
            }
        });
    match outcome {
        Ok(Some(failure)) => report.fail(failure),
        Ok(None) => report.pass(),
        Err(why) => report.from_error(&why),
    }
}

/// Press Tools > Compare and wait for the list.
fn run_compare(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    press_on_tab(ctx, session, pointer, ITEM, TAB)?;
    let listed = step!(wait_for(
        session,
        LISTED,
        mark,
        "Tools > Compare was pressed"
    ));
    report.note(format!("`{}`", listed.raw));
    Ok(None)
}

/// The list names every change of the pair, on page 2 only, and draws a row
/// for each.
fn list_body(
    ctx: &CheckContext,
    _report: &mut CheckReport,
    session: &Session,
    _pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let listed = step!(after(session, LISTED, 0, "the comparison finished"));
    let kinds = kinds_of(&listed);
    for (kind, want) in TEXT_ROWS {
        let got = kinds.iter().filter(|k| k.starts_with(kind)).count();
        if got != want {
            return Ok(Some(format!(
                "`{}` lists {got} `{kind}` row(s); the fixture pair holds {want}.",
                listed.raw
            )));
        }
    }
    if !kinds.contains(&"area@2") {
        return Ok(Some(format!(
            "`{}` lists no area on page 2: the square only the newer page carries was not seen.",
            listed.raw
        )));
    }
    if let Some(wrong) = kinds.iter().find(|k| k.ends_with("@1")) {
        return Ok(Some(format!(
            "`{}` lists `{wrong}`: page 1 is the same in both files.",
            listed.raw
        )));
    }
    let trace = session.trace()?;
    let ui_rect = ui_rect_event(ctx)?;
    for n in 0..kinds.len() {
        if driving::declared(&trace, ui_rect, &format!("{ROW_PREFIX}{n}")).is_none() {
            return Ok(Some(format!(
                "`{ROW_PREFIX}{n}` was never drawn: the list traced {} rows and the panel has \
                 no row {n} to click.",
                kinds.len()
            )));
        }
    }
    Ok(None)
}

/// A row on page 2 turns the view from page 1 and reveals its rectangle.
fn jump_body(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let listed = step!(after(session, LISTED, 0, "the comparison finished"));
    let Some(row) = kinds_of(&listed)
        .iter()
        .position(|k| k.starts_with("replaced@2"))
    else {
        return Ok(Some(format!(
            "`{}` lists no replacement on page 2 to click.",
            listed.raw
        )));
    };
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, &format!("{ROW_PREFIX}{row}"))?;
    session.settle(40);
    let jumped = step!(after(
        session,
        JUMPED,
        mark,
        &format!("row {row} was clicked")
    ));
    report.note(format!("`{}`", jumped.raw));
    if jumped.get("page") != Some("2") {
        return Ok(Some(format!("`{}`: the change is on page 2.", jumped.raw)));
    }
    let revealed = step!(after(
        session,
        REVEALED,
        mark,
        &format!("row {row} was clicked")
    ));
    report.note(format!("`{}`", revealed.raw));
    if revealed.get("page") != Some("1") || revealed.get("why") != Some("compare-row") {
        return Ok(Some(format!(
            "`{}`: the reveal after a row click should be page index 1 for `compare-row`.",
            revealed.raw
        )));
    }
    Ok(None)
}

/// Mark the changes, save: page 2 carries the comments, page 1 none. One
/// Undo, save: they are all gone.
fn mark_body(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, MARK)?;
    let marked = step!(wait_for(
        session,
        MARKED,
        mark,
        "Mark changes as comments was pressed"
    ));
    report.note(format!("`{}`", marked.raw));
    let written: usize = marked
        .get("annotations")
        .and_then(|n| n.parse().ok())
        .unwrap_or(0);
    // Appearance is compared by default, so its paragraph goes with the marks.
    if marked.get("note") != Some("1") {
        return Ok(Some(format!(
            "`{}`: the appearance comparison ran, so its summary note was wanted (note=1).",
            marked.raw
        )));
    }
    if written < 4 || marked.get("top") != Some("mark-comparison") {
        return Ok(Some(format!(
            "`{}`: at least the four text changes as comments, as one undo entry on top of the stack, was \
             wanted.",
            marked.raw
        )));
    }
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let saved = read(&ctx.out("compare-mark.marked.pdf"))?;
    let (first, second) = (listed_on_page(&saved, 0), listed_on_page(&saved, 1));
    report.note(format!(
        "saved: page 1 lists {}, page 2 lists {} ({})",
        first.len(),
        second.len(),
        second
            .iter()
            .map(|a| a.subtype.as_str())
            .collect::<Vec<_>>()
            .join(",")
    ));
    if !first.is_empty() || second.len() != written {
        return Ok(Some(format!(
            "the saved copy lists {} annotation(s) on page 1 and {} on page 2; `{}` wrote \
             {written}, all on page 2.",
            first.len(),
            second.len(),
            marked.raw
        )));
    }
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(40);
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let undone = read(&ctx.out("compare-mark.undone.pdf"))?;
    let left = listed_on_page(&undone, 1).len();
    if left != 0 {
        return Ok(Some(format!(
            "after ONE Undo the saved page 2 still lists {left} of the {written} comments: the \
             marks were not one undo step."
        )));
    }
    Ok(None)
}

/// The `kinds=` list of a `compare-listed` line.
fn kinds_of(line: &TraceLine) -> Vec<&str> {
    line.get("kinds")
        .map(|k| k.split(',').filter(|s| !s.is_empty()).collect())
        .unwrap_or_default()
}

/// Wait up to about ten seconds for `event` after `mark`: the comparison and
/// the marking run off the frame.
fn wait_for(
    session: &Session,
    event: &str,
    mark: usize,
    what: &str,
) -> Result<std::result::Result<TraceLine, String>> {
    for _ in 0..25 {
        if let Some(line) = session.trace()?.last_after(event, mark).cloned() {
            return Ok(Ok(line));
        }
        session.settle(20);
    }
    after(session, event, mark, what)
}
