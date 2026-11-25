//! `a_moved_or_deleted_sheet_keeps_its_label` — on `fixtures/drawing-sheets.pdf`
//! (`A-101 … A-104`), Pages ▸ Move down and then Delete under the default
//! *Each page keeps its label* setting: the moved sheet carries its label, the
//! deleted one takes only its own away, and a second process opening the saved
//! copy reads the same labels.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/page_labels_follow.md`.

use super::dimdrive::{launch_path, launch_with, press_on_tab};
use super::info_entries::SHEETS;
use super::multiselect::{after, save, step};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.review"; // ui-text-exempt: a ribbon command id
const PAGES_TAB: &str = "ribbon.tab.pages"; // ui-text-exempt: a trace region name
const MOVE_DOWN: &str = "ribbon.item.pages.move_down"; // ui-text-exempt: a trace region name
const DELETE: &str = "ribbon.item.pages.delete"; // ui-text-exempt: a trace region name
const LABELS: &str = "page-labels-now"; // ui-text-exempt: a trace event name
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
/// The fixture's labels, and the order a move of sheet 1 down leaves.
const OPENED: [&str; 4] = ["A-101", "A-102", "A-103", "A-104"];
const MOVED: [&str; 4] = ["A-102", "A-101", "A-103", "A-104"];

/// See the module documentation.
pub struct AMovedOrDeletedSheetKeepsItsLabel;

impl Check for AMovedOrDeletedSheetKeepsItsLabel {
    fn name(&self) -> &'static str {
        "a_moved_or_deleted_sheet_keeps_its_label"
    }

    fn defect(&self) -> &'static str {
        "moving or deleting a sheet renumbers the drawing set: A-101 moved down reads A-102, \
         or deleting a sheet relabels every sheet after it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let saved = ctx.out("page-labels-follow.saved.pdf");
        let _ = std::fs::remove_file(&saved);
        let env = [(SAVE_PATH_ENV, saved.display().to_string())];
        let outcome = launch_with(
            ctx,
            &mut report,
            &SHEETS,
            INVOKE,
            "page-labels-follow",
            &env,
        )
        .and_then(|(session, pointer)| {
            let outcome = drive(ctx, &mut report, &session, &pointer);
            let parked = pointer.gone(&session);
            match outcome? {
                Some(failure) => Ok(Some(failure)),
                None => parked.map(|_| None),
            }
        });
        let outcome = match outcome {
            Ok(None) => reopen(ctx, &mut report, &saved),
            other => other,
        };
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// A `page-labels-now` line's labels.
fn labels(line: &crate::trace::TraceLine) -> Vec<String> {
    line.get("labels")
        .filter(|l| !l.is_empty())
        .map(|l| l.split('|').map(str::to_owned).collect())
        .unwrap_or_default()
}

/// `after` is `before` with exactly one label taken out.
fn one_removed(before: &[&str], after: &[String]) -> bool {
    after.len() + 1 == before.len()
        && (0..before.len()).any(|gone| {
            before
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != gone)
                .map(|(_, l)| *l)
                .eq(after.iter().map(String::as_str))
        })
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let opened = step!(after(session, LABELS, 0, "the drawing set was opened"));
    report.note(format!("opened: `{}`", opened.raw));
    if labels(&opened) != OPENED {
        return Ok(Some(format!(
            "the fixture should open labelled {}; it traced `{}`.",
            OPENED.join("|"),
            opened.raw
        )));
    }

    let mark = session.trace()?.mark();
    press_on_tab(ctx, session, pointer, MOVE_DOWN, PAGES_TAB)?;
    let moved = step!(after(
        session,
        LABELS,
        mark,
        "Move down was pressed on sheet 1"
    ));
    report.note(format!("moved: `{}`", moved.raw));
    if moved.get("why") != Some("reorder") || labels(&moved) != MOVED {
        return Ok(Some(format!(
            "moving sheet A-101 down must leave {}; it traced `{}`.",
            MOVED.join("|"),
            moved.raw
        )));
    }

    let mark = session.trace()?.mark();
    press_on_tab(ctx, session, pointer, DELETE, PAGES_TAB)?;
    let deleted = step!(after(session, LABELS, mark, "Delete was pressed"));
    report.note(format!("deleted: `{}`", deleted.raw));
    let left = labels(&deleted);
    if deleted.get("why") != Some("delete") || !one_removed(&MOVED, &left) {
        return Ok(Some(format!(
            "deleting one sheet of {} must take out only its own label; it traced `{}`.",
            MOVED.join("|"),
            deleted.raw
        )));
    }
    write_expected(ctx, &left)?;
    save(ctx, session, pointer)
}

/// The delete's labels, kept for the reopen to compare against.
fn write_expected(ctx: &CheckContext, left: &[String]) -> Result<()> {
    std::fs::write(ctx.out("page-labels-follow.expected.txt"), left.join("|"))
        .map_err(|e| crate::error::Error::new(format!("cannot keep the expected labels: {e}")))
}

/// A second process opening the saved copy reads the labels the delete left.
fn reopen(
    ctx: &CheckContext,
    report: &mut CheckReport,
    saved: &std::path::Path,
) -> Result<Option<String>> {
    let want = std::fs::read_to_string(ctx.out("page-labels-follow.expected.txt"))
        .map_err(|e| crate::error::Error::new(format!("no expected labels kept: {e}")))?;
    let (session, pointer) = launch_path(
        ctx,
        report,
        saved.to_path_buf(),
        INVOKE,
        "page-labels-follow.reopen",
        &[],
    )?;
    let opened = after(&session, LABELS, 0, "the saved copy was opened");
    let _ = pointer.gone(&session);
    let opened = step!(opened);
    report.note(format!("reopened: `{}`", opened.raw));
    if labels(&opened).join("|") != want {
        return Ok(Some(format!(
            "a second process opening the saved copy reads `{}`; `{want}` was left by the \
             delete.",
            opened.raw
        )));
    }
    Ok(None)
}
