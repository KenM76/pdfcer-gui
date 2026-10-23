//! `a_compacted_copy_lists_each_picture_before_and_after` — **once the compact
//! window has worked out the copy, it lists each changed picture, before and
//! after, before anything is saved.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/compact_compare.md`.

use super::compact_target::{drawn, recompressed_after};
use super::dimdrive::{Fixture, press, run_on_with};
use super::stroke_style::into_view_of;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// One 400×400 greyscale noise picture drawn at 300 ppi, above the 150 ppi
/// the pictures switch opens on, so switching it on resizes the picture.
const FIXTURE: Fixture = Fixture {
    file: "noisy-photo.pdf",
    method: "Regenerate it with `python fixtures/noisy-photo.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const INVOKE: &str = "mode.edit,file.save_compacted";
/// Mirror `dialogs::compact_images::{REGION_FOLD, REGION_SWITCH}` and
/// `dialogs::compact_compare::{REGION, REGION_TABLE}`.
const FOLD: &str = "compact.pictures";
const SWITCH: &str = "compact.pictures.on";
const COMPARE: &str = "compact.compare";
const TABLE: &str = "compact.compare.table";
const BODY: &str = "compact.body";

/// See the module documentation.
pub struct ACompactedCopyListsEachPictureBeforeAndAfter;

impl Check for ACompactedCopyListsEachPictureBeforeAndAfter {
    fn name(&self) -> &'static str {
        "a_compacted_copy_lists_each_picture_before_and_after"
    }

    fn defect(&self) -> &'static str {
        "the compact window does not list the pictures the copy changes, before and after, \
         before it is saved"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on_with(self, ctx, &FIXTURE, INVOKE, "compact-compare", &[], body)
    }
}

/// Scroll `region` into view and press it; `false` when it never drew.
fn reach(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
) -> Result<bool> {
    into_view_of(ctx, session, pointer, region, BODY)?;
    if !drawn(ctx, session, region)? {
        return Ok(false);
    }
    press(ctx, session, pointer, region)?;
    Ok(true)
}

fn body(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    if recompressed_after(session, 0)?.is_none() {
        return Ok(Some(format!(
            "the compacted-copy window never finished its first recompression. Trace: {path}."
        )));
    }
    if !reach(ctx, session, pointer, FOLD)? {
        return Ok(Some(format!(
            "no `{FOLD}` fold in the window. Trace: {path}."
        )));
    }
    let mark = session.trace()?.mark();
    if !reach(ctx, session, pointer, SWITCH)? {
        return Ok(Some(format!(
            "no `{SWITCH}` switch in the window. Trace: {path}."
        )));
    }
    let Some(line) = recompressed_after(session, mark)? else {
        return Ok(Some(format!(
            "pictures were switched on and the copy was not rebuilt within 30 s. Trace: {path}."
        )));
    };
    let compared = line.get_usize("compared");
    report.note(format!("pictures on: compared={compared:?}"));
    if compared != Some(1) {
        return Ok(Some(format!(
            "with pictures on, the copy should compare the one resized picture; it compared \
             {compared:?}. Line: `{}`. Trace: {path}.",
            line.raw
        )));
    }
    if !reach(ctx, session, pointer, COMPARE)? {
        return Ok(Some(format!(
            "the copy compared a picture and no `{COMPARE}` fold drew under the result. \
             Trace: {path}."
        )));
    }
    into_view_of(ctx, session, pointer, TABLE, BODY)?;
    if !drawn(ctx, session, TABLE)? {
        return Ok(Some(format!(
            "`{COMPARE}` was pressed and no `{TABLE}` table drew. Trace: {path}."
        )));
    }
    report.note("the before-and-after table drew");
    Ok(None)
}
