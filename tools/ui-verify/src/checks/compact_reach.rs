//! `size_reduction_is_on_the_ribbon_and_the_tab_menu` — **File ▸ Save's
//! Reduce file size opens the compact window at the Standard preset, and a
//! document tab's right-click Fit to a size opens it with the size target on;
//! each reaches the engine.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/compact_reach.md`.

use super::compact_target::{WAIT_S, drawn, recompressed_after};
use super::dimdrive::{Fixture, press, press_on_tab, reveal, run_on_with, ui_rect_event};
use crate::checks::driving::{declared_in, declared_names, declared_or_in_overflow, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, WindowPoint};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// A 300 ppi noise picture: Standard scales pictures above 225 ppi, and noise
/// keeps the plain copy above any opening target.
const FIXTURE: Fixture = Fixture {
    file: "noisy-photo.pdf",
    method: "`python fixtures/noisy-photo.PROVENANCE.py`",
    page: PageGeometry {
        width_pt: 200.0,
        height_pt: 200.0,
    },
};
const INVOKE: &str = "mode.edit";
const FILE_TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name
const REDUCE: &str = "ribbon.item.file.reduce_size"; // ui-text-exempt: a trace region name
const FIT_RIBBON: &str = "ribbon.item.file.fit_size"; // ui-text-exempt: a trace region name
const TAB: &str = "doc-tab.0"; // ui-text-exempt: a trace region name
const MENU: &str = "menu.item.document.tab."; // ui-text-exempt: a trace region name prefix
const ROWS: [&str; 3] = ["file.save_compacted", "file.reduce_size", "file.fit_size"];
/// Mirrors `dialogs::compact::REGION_CANCEL`.
const CANCEL: &str = "compact.cancel";
const RECOMPRESSED: &str = "compact-recompressed"; // ui-text-exempt: a trace event name

/// See the module documentation.
pub struct SizeReductionIsOnTheMenus;

impl Check for SizeReductionIsOnTheMenus {
    fn name(&self) -> &'static str {
        "size_reduction_is_on_the_ribbon_and_the_tab_menu"
    }

    fn defect(&self) -> &'static str {
        "the size-reduction commands are missing from File ▸ Save or the document tab's \
         right-click menu, or they open the compact window without the starting point they \
         name"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on_with(self, ctx, &FIXTURE, INVOKE, "compact-reach", &[], body)
    }
}

fn body(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if let Some(failure) = from_the_ribbon(ctx, report, session, pointer)? {
        return Ok(Some(failure));
    }
    press(ctx, session, pointer, CANCEL)?;
    session.settle(10);
    from_the_tab_menu(ctx, report, session, pointer)
}

/// File ▸ Save ▸ Reduce file size: the window opens at Standard, and the
/// engine scales the picture under it with no size target.
fn from_the_ribbon(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let ui_rect = ui_rect_event(ctx)?;
    reveal(ctx, session, pointer, FIT_RIBBON, FILE_TAB)?;
    for item in [FIT_RIBBON, REDUCE] {
        if declared_or_in_overflow(session, pointer, ui_rect, item)?.is_none() {
            return Ok(Some(format!(
                "★★ FILE ▸ SAVE HAS NO `{item}`, on the tab or in its overflow. Regions \
                 beginning `ribbon.item.file.`: {}. Trace: {path}.",
                list(&declared_names(
                    &session.trace()?,
                    ui_rect,
                    "ribbon.item.file."
                ))
            )));
        }
    }
    let mark = session.trace()?.mark();
    press_on_tab(ctx, session, pointer, REDUCE, FILE_TAB)?;
    let Some(line) = recompressed_after(session, mark)? else {
        return Ok(Some(format!(
            "★★ REDUCE FILE SIZE OPENED NO COMPACT WINDOW: no `{RECOMPRESSED}` line in \
             {WAIT_S} s. Trace: {path}."
        )));
    };
    report.note(format!("Reduce file size: `{}`", line.raw));
    let standard = line.get("preset") == Some("standard")
        && line.get_usize("resized").unwrap_or(0) >= 1
        && line.get("target") == Some("none");
    Ok((!standard).then(|| {
        format!(
            "★★★ REDUCE FILE SIZE DID NOT START FROM STANDARD: `{}`. Standard scales the \
             fixture's 300 ppi picture to 150 ppi, with no size target. Trace: {path}.",
            line.raw
        )
    }))
}

/// A document tab's right-click menu carries all three, and Fit to a size
/// opens the window with the target on and met.
fn from_the_tab_menu(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let ui_rect = ui_rect_event(ctx)?;
    let Some((tab, _)) = declared_in(&session.trace()?, ui_rect, TAB) else {
        return Ok(Some(format!(
            "the document tab `{TAB}` was never drawn. Trace: {path}."
        )));
    };
    pointer.right_click(session, WindowPoint::centre_of(tab))?;
    session.settle(10);
    for row in ROWS {
        if !drawn(ctx, session, &format!("{MENU}{row}"))? {
            return Ok(Some(format!(
                "★★ THE DOCUMENT TAB'S MENU HAS NO `{row}`. Rows: {}. Trace: {path}.",
                list(&declared_names(&session.trace()?, ui_rect, MENU))
            )));
        }
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, &format!("{MENU}file.fit_size"))?;
    let Some(line) = recompressed_after(session, mark)? else {
        return Ok(Some(format!(
            "★★ FIT TO A SIZE OPENED NO COMPACT WINDOW: no `{RECOMPRESSED}` line in {WAIT_S} s. \
             Trace: {path}."
        )));
    };
    report.note(format!("Fit to a size: `{}`", line.raw));
    let max = line.get_usize("max").unwrap_or(0);
    let bytes = line.get_usize("bytes").unwrap_or(0);
    let fit = line.get("target") == Some("met")
        && line.get("preset") == Some("lossless")
        && max > 0
        && bytes > 0
        && bytes <= max;
    Ok((!fit).then(|| {
        format!(
            "★★★ FIT TO A SIZE DID NOT OPEN WITH THE TARGET ON: `{}`. It starts from Lossless \
             with the target at half the plain copy, which a lower JPEG quality meets. \
             Trace: {path}.",
            line.raw
        )
    }))
}
