//! `the_tags_panel_outlines_the_picked_element` and
//! `the_tags_panel_says_a_file_is_untagged` — the Tags panel lists the
//! structure tree, picking a cell, a row and the table outlines each one's
//! content on every page it spans, and a file with no tags says so.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/tags_pick.md`.

use super::dimdrive::{Fixture, launch_with, press, ui_rect_event};
use super::driving::{declared, declared_names, declared_since};
use super::multiselect::{after, step};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const SPLIT: Fixture = Fixture {
    file: "tags-split-table.pdf",
    method: "Rebuild it with `python fixtures/tags-split-table.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const UNTAGGED: Fixture = Fixture {
    file: "ruled-table.pdf",
    method: "Rebuild it with `python fixtures/ruled-table.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const OPEN: &str = "view.panel_tags";
const ROW: &str = "tags.row."; // ui-text-exempt: a trace region name prefix
const OUTLINE: &str = "canvas.tag-outline.1"; // ui-text-exempt: a trace region name
const STATE: &str = "tags-panel"; // ui-text-exempt: a trace event name
const PICK: &str = "tags-pick"; // ui-text-exempt: a trace event name
/// The fixture's first cell, `A1`, is set at (72, 700) in 12 pt Helvetica.
const A1: (f64, f64) = (72.0, 700.0);
/// The second column's text starts here.
const B_X: f64 = 300.0;

/// `the_tags_panel_outlines_the_picked_element`.
pub struct TheTagsPanelOutlinesThePickedElement;
/// `the_tags_panel_says_a_file_is_untagged`.
pub struct TheTagsPanelSaysAFileIsUntagged;

impl Check for TheTagsPanelOutlinesThePickedElement {
    fn name(&self) -> &'static str {
        "the_tags_panel_outlines_the_picked_element"
    }

    fn defect(&self) -> &'static str {
        "The Tags panel draws no tree for a tagged file, or picking a cell, a row or the table \
         outlines the wrong content, misses a page the element spans, or draws no outline"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        drive(self, ctx, &SPLIT, "tags-pick", pick_body)
    }
}

impl Check for TheTagsPanelSaysAFileIsUntagged {
    fn name(&self) -> &'static str {
        "the_tags_panel_says_a_file_is_untagged"
    }

    fn defect(&self) -> &'static str {
        "The Tags panel on a file with no structure tree draws rows or says nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        drive(self, ctx, &UNTAGGED, "tags-untagged", untagged_body)
    }
}

type Body =
    fn(&CheckContext, &mut CheckReport, &Session, &ScriptedPointer) -> Result<Option<String>>;

fn drive(
    check: &dyn Check,
    ctx: &CheckContext,
    fixture: &Fixture,
    stem: &str,
    body: Body,
) -> CheckReport {
    let mut report = CheckReport::new(check.name(), check.defect());
    let outcome =
        launch_with(ctx, &mut report, fixture, OPEN, stem, &[]).and_then(|(session, pointer)| {
            let outcome = body(ctx, &mut report, &session, &pointer);
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

fn untagged_body(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    _pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let state = step!(after(session, STATE, 0, "the Tags panel was opened"));
    report.note(format!("`{}`", state.raw));
    if state.get("state") != Some("untagged") {
        return Ok(Some(format!(
            "`{}` on a file with no structure tree; expected `state=untagged`.",
            state.raw
        )));
    }
    let rows = declared_names(&session.trace()?, ui_rect_event(ctx)?, ROW);
    if !rows.is_empty() {
        return Ok(Some(format!("an untagged file drew tag rows: {rows:?}.")));
    }
    Ok(None)
}

/// One `p:llx,lly,urx,ury` box of a `tags-pick` line; page 1-based.
#[derive(Debug, Clone, Copy)]
struct Box {
    page: usize,
    llx: f64,
    lly: f64,
    urx: f64,
    ury: f64,
}

fn boxes(at: &str) -> Vec<Box> {
    at.split(';')
        .filter_map(|b| {
            let (page, r) = b.split_once(':')?;
            let v: Vec<f64> = r.split(',').filter_map(|c| c.parse().ok()).collect();
            let [llx, lly, urx, ury] = v[..] else {
                return None;
            };
            Some(Box {
                page: page.parse().ok()?,
                llx,
                lly,
                urx,
                ury,
            })
        })
        .collect()
}

fn pick_body(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let state = step!(after(session, STATE, 0, "the Tags panel was opened"));
    report.note(format!("`{}`", state.raw));
    if state.get("state") != Some("read") {
        return Ok(Some(format!("`{}`; expected `state=read`.", state.raw)));
    }
    let rows = declared_names(&session.trace()?, ui_rect_event(ctx)?, ROW);
    report.note(format!("rows {rows:?}"));
    let first = |kind: &str| {
        rows.iter()
            .find(|r| r.ends_with(&format!(".{kind}")))
            .cloned()
    };
    let (Some(cell), Some(row), Some(table)) = (first("TD"), first("TR"), first("Table")) else {
        return Ok(Some(format!(
            "the tree drew no TD, TR and Table rows: {rows:?}."
        )));
    };
    let mark = session.trace()?.mark();
    let cell = step!(pick(ctx, report, session, pointer, &cell));
    if declared_since(&session.trace()?, ui_rect_event(ctx)?, OUTLINE, mark).is_none() {
        return Ok(Some(format!("picking a cell drew no `{OUTLINE}`.")));
    }
    let Some(c) = one_on_page_one(&cell) else {
        return Ok(Some(format!(
            "the cell has {cell:?}; expected one box on page 1."
        )));
    };
    if (c.llx - A1.0).abs() > 3.0 || c.lly > A1.1 + 1.0 || c.ury < A1.1 + 1.0 || c.urx > B_X {
        return Ok(Some(format!(
            "the cell's box {c:?} does not hold `A1` at {A1:?} alone."
        )));
    }
    let row = step!(pick(ctx, report, session, pointer, &row));
    let Some(r) = one_on_page_one(&row) else {
        return Ok(Some(format!(
            "the row has {row:?}; expected one box on page 1."
        )));
    };
    if r.llx > c.llx + 0.5 || r.urx < B_X || r.lly > c.lly + 0.5 || r.ury < c.ury - 0.5 {
        return Ok(Some(format!(
            "the row's box {r:?} does not hold its first cell {c:?} and reach the second column."
        )));
    }
    let table = step!(pick(ctx, report, session, pointer, &table));
    let pages: Vec<usize> = table.iter().map(|b| b.page).collect();
    if pages != [1, 2] {
        return Ok(Some(format!(
            "the table, one row on each page, has boxes on pages {pages:?}; expected [1, 2]."
        )));
    }
    // Page 1 holds only row 1, so the table's box there is the row's and the
    // region is not re-emitted; it must still be live.
    if declared(&session.trace()?, ui_rect_event(ctx)?, OUTLINE).is_none() {
        return Ok(Some(format!(
            "with the table picked no `{OUTLINE}` is drawn."
        )));
    }
    Ok(None)
}

fn one_on_page_one(b: &[Box]) -> Option<Box> {
    match b {
        [only] if only.page == 1 => Some(*only),
        _ => None,
    }
}

/// Press the row `region` and read the `tags-pick` that follows.
fn pick(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
) -> Result<std::result::Result<Vec<Box>, String>> {
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, region)?;
    session.settle(20);
    Ok(
        after(session, PICK, mark, &format!("`{region}` was pressed"))?.map(|line| {
            report.note(format!("`{}`", line.raw));
            line.get("at").map(boxes).unwrap_or_default()
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::boxes;

    #[test]
    fn a_pick_line_reads_one_box_per_page() {
        let b = boxes("1:72.0,697.4,86.0,709.0;2:72.0,697.4,316.0,709.0");
        assert_eq!(b.len(), 2);
        assert_eq!((b[0].page, b[1].page), (1, 2));
        assert!((b[1].urx - 316.0).abs() < 1e-9);
        assert!(boxes("").is_empty());
    }
}
