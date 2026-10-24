//! `a_paragraph_on_his_drawing_opens_as_one` — on a real drawing, a click in a
//! paragraph with Paragraphs on opens more than the line under the click.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/paragraph_on_drawing.md`.

use super::dimdrive::{Fixture, click_on, launch_path};
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::report::CheckReport;
use crate::trace::TraceLine;

const CARET: &str = "text-edit-caret"; // ui-text-exempt: a trace event name, never displayed
const SWITCHED: &str = "open-paragraphs"; // ui-text-exempt: a trace event name, never displayed
/// Paragraphs at its default (on), then the switch pressed off by command.
const ON: &str = "mode.edit,edit.text";
const OFF: &str = "mode.edit,edit.paragraphs,edit.text";
/// The block must hold at least this many times the line's characters: a
/// paragraph of two short lines still clears it, a line that opened alone
/// cannot.
const MIN_RATIO: usize = 2;

/// See the module documentation.
pub struct AParagraphOnHisDrawingOpensAsOne;

impl Check for AParagraphOnHisDrawingOpensAsOne {
    fn name(&self) -> &'static str {
        "a_paragraph_on_his_drawing_opens_as_one"
    }

    fn defect(&self) -> &'static str {
        "on a real drawing a click in a paragraph opened only the line under it, so the \
         paragraph was edited line by line"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(None) => report.pass(),
            Ok(Some(failure)) => report.fail(failure),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The input: the drawing, the point, and the drawing's page size.
fn inputs(ctx: &CheckContext) -> Result<(std::path::PathBuf, DocPoint, PageGeometry)> {
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. Pass the drawing whose paragraph is to be opened."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new("no --doc-point. Pass PAGE,X,Y in PDF user space inside the paragraph.")
    })?;
    let page = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };
    Ok((pdf, target, page))
}

/// Launch a scratch copy of `pdf` with `invoke`, click `at`, and answer the
/// caret line the click opened and whether Paragraphs was traced off.
fn open_at(
    ctx: &CheckContext,
    report: &mut CheckReport,
    pdf: &std::path::Path,
    at: DocPoint,
    page: PageGeometry,
    invoke: &str,
    stem: &str,
) -> Result<(Option<TraceLine>, bool)> {
    let doc = ctx.out(&format!("{stem}.pdf"));
    std::fs::copy(pdf, &doc).map_err(|e| Error::new(format!("copying {}: {e}", pdf.display())))?;
    let (session, pointer) = launch_path(ctx, report, doc, invoke, stem, &[])?;
    session.settle(30);
    let fixture = Fixture {
        file: "",
        method: "",
        page,
    };
    let mark = session.trace()?.mark();
    let clicked = click_on(ctx, &session, &pointer, &fixture, at.page, (at.x, at.y));
    session.settle(20);
    let trace = session.trace()?;
    pointer.gone(&session)?;
    clicked?;
    let off = trace
        .events(SWITCHED)
        .any(|l| l.get("enabled") == Some("false"));
    Ok((trace.last_after(CARET, mark).cloned(), off))
}

fn len_of(line: &TraceLine) -> Option<usize> {
    line.get("len")?.parse().ok()
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (pdf, at, page) = inputs(ctx)?;
    let (on, _) = open_at(ctx, report, &pdf, at, page, ON, "paragraph-on")?;
    let (off, switched) = open_at(ctx, report, &pdf, at, page, OFF, "paragraph-off")?;
    let raw = |l: &Option<TraceLine>| l.as_ref().map_or("nothing", |l| l.raw.as_str()).to_owned();
    report.note(format!("Paragraphs on: `{}`", raw(&on)));
    report.note(format!("Paragraphs off: `{}`", raw(&off)));

    let Some(line) = off.as_ref().filter(|l| switched && l.get("run").is_some()) else {
        return Err(Error::new(format!(
            "with Paragraphs off the click opened no line (`{}`, switched off: {switched}), so \
             there is no line to compare the paragraph with. Aim --doc-point at text. SKIPPED \
             for that reason.",
            raw(&off)
        )));
    };
    let line_len = len_of(line).unwrap_or(0);
    let Some(block) = on.as_ref().filter(|l| l.get("block").is_some()) else {
        return Ok(Some(format!(
            "★★★★ with Paragraphs on the click opened `{}`, not a paragraph; the same point \
             with Paragraphs off opened a line of {line_len} characters.",
            raw(&on)
        )));
    };
    let block_len = len_of(block).unwrap_or(0);
    Ok((block_len < MIN_RATIO * line_len.max(1)).then(|| {
        format!(
            "★★★ with Paragraphs on the click opened a block of {block_len} characters, under \
             {MIN_RATIO} times the {line_len}-character line under the same point: the block \
             is that line alone. `{}`",
            block.raw
        )
    }))
}
