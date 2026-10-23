//! `a_compacted_copy_treats_each_kind_of_picture_separately` — **with the
//! compact window's pictures split by kind, a picture follows its own kind's
//! settings and no other.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/compact_classes.md`.

use super::compact_target::{drawn, recompressed_after};
use super::dimdrive::{Fixture, press, run_on_with};
use super::stroke_style::into_view_of;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

/// One 400×400 greyscale noise picture drawn at 300 ppi, above the 150 ppi
/// every kind opens on.
const FIXTURE: Fixture = Fixture {
    file: "noisy-photo.pdf",
    method: "Regenerate it with `python fixtures/noisy-photo.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const INVOKE: &str = "mode.edit,file.save_compacted";
/// Mirror `dialogs::compact_images::{REGION_FOLD, REGION_SWITCH}`,
/// `dialogs::compact_classes::REGION_SPLIT` and the per-kind `<stem>.on`.
const FOLD: &str = "compact.pictures";
const SWITCH: &str = "compact.pictures.on";
const SPLIT: &str = "compact.pictures.split";
const COLOUR_ON: &str = "compact.pictures.colour.on";
const GREY_ON: &str = "compact.pictures.grey.on";
const BODY: &str = "compact.body";

/// See the module documentation.
pub struct ACompactedCopyTreatsEachKindOfPictureSeparately;

impl Check for ACompactedCopyTreatsEachKindOfPictureSeparately {
    fn name(&self) -> &'static str {
        "a_compacted_copy_treats_each_kind_of_picture_separately"
    }

    fn defect(&self) -> &'static str {
        "the compact window cannot give colour, grey and black-and-white pictures their own \
         settings: the split is missing, or a picture is changed by another kind's settings \
         or left alone by its own"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on_with(self, ctx, &FIXTURE, INVOKE, "compact-classes", &[], body)
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

/// Press `region` and return the recompression that follows it.
fn after_press(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
) -> Result<std::result::Result<TraceLine, String>> {
    let path = session.trace_path().display().to_string();
    let mark = session.trace()?.mark();
    if !reach(ctx, session, pointer, region)? {
        return Ok(Err(format!(
            "no `{region}` region could be scrolled into view in the compacted-copy window. \
             Trace: {path}."
        )));
    }
    Ok(recompressed_after(session, mark)?.ok_or_else(|| {
        format!("`{region}` was pressed and the copy was not rebuilt within 30 s. Trace: {path}.")
    }))
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
    // Each step: the region pressed, and how many pictures must be resized after it.
    let steps = [
        (SWITCH, 1, "pictures on, one set of choices"),
        (SPLIT, 1, "split by kind; grey opens at 150 ppi"),
        (
            COLOUR_ON,
            1,
            "colour switched off; the grey picture is not colour",
        ),
        (GREY_ON, 0, "grey switched off; the picture's own kind"),
    ];
    for (region, owed, meaning) in steps {
        let line = match after_press(ctx, session, pointer, region)? {
            Ok(line) => line,
            Err(failure) => return Ok(Some(failure)),
        };
        let resized = line.get_usize("resized").unwrap_or(usize::MAX);
        report.note(format!("{meaning}: resized={resized}"));
        if resized != owed {
            return Ok(Some(format!(
                "after `{region}` ({meaning}) the copy resized {resized} picture(s); owed \
                 {owed}. Line: `{}`. Trace: {path}.",
                line.raw
            )));
        }
    }
    Ok(None)
}
