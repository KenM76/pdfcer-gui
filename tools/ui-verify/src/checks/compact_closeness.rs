//! `a_closeness_target_picks_each_pictures_quality` — **in the compact
//! window, choosing how closely JPEG pictures must match the original sends
//! each through the engine's quality search, and the copy counts it.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/compact_closeness.md`.

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

/// One 400×400 greyscale noise picture, stored exactly, so JPEG changes it.
const FIXTURE: Fixture = Fixture {
    file: "noisy-photo.pdf",
    method: "Regenerate it with `python fixtures/noisy-photo.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const INVOKE: &str = "mode.edit,file.save_compacted";
/// Mirror `dialogs::compact_images::{REGION_FOLD, REGION_SWITCH}` and the
/// `<stem>.format` and `<stem>.target` lists.
const FOLD: &str = "compact.pictures";
const SWITCH: &str = "compact.pictures.on";
const FORMAT_LIST: &str = "compact.pictures.format";
const JPEG: &str = "compact.pictures.format.jpeg";
const TARGET_LIST: &str = "compact.pictures.target";
const CLOSE: &str = "compact.pictures.target.9500";
const BODY: &str = "compact.body";

/// See the module documentation.
pub struct AClosenessTargetPicksEachPicturesQuality;

impl Check for AClosenessTargetPicksEachPicturesQuality {
    fn name(&self) -> &'static str {
        "a_closeness_target_picks_each_pictures_quality"
    }

    fn defect(&self) -> &'static str {
        "the compact window offers no way to ask JPEG pictures to match the original to a \
         closeness, or asking for one does not send them through the engine's quality search"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on_with(self, ctx, &FIXTURE, INVOKE, "compact-closeness", &[], body)
    }
}

/// Press `region`, scrolling it into the window's body first when `in_body`
/// (a list's open choices float above the body); `false` when it never drew.
fn reach(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
    in_body: bool,
) -> Result<bool> {
    if in_body {
        into_view_of(ctx, session, pointer, region, BODY)?;
    }
    if !drawn(ctx, session, region)? {
        return Ok(false);
    }
    press(ctx, session, pointer, region)?;
    Ok(true)
}

/// Open `list`, press `choice`, and return the recompression that follows.
fn choose(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    list: &str,
    choice: &str,
) -> Result<std::result::Result<TraceLine, String>> {
    let path = session.trace_path().display().to_string();
    let mark = session.trace()?.mark();
    for (region, in_body) in [(list, true), (choice, false)] {
        if !reach(ctx, session, pointer, region, in_body)? {
            return Ok(Err(format!(
                "no `{region}` region drew in the compacted-copy window. Trace: {path}."
            )));
        }
    }
    Ok(recompressed_after(session, mark)?.ok_or_else(|| {
        format!("`{choice}` was chosen and the copy was not rebuilt within 30 s. Trace: {path}.")
    }))
}

/// `jpeg`, and `matched + short`, from a recompression line.
fn counts(line: &TraceLine) -> (usize, usize) {
    let n = |key| line.get_usize(key).unwrap_or(usize::MAX);
    (n("jpeg"), n("matched").saturating_add(n("short")))
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
    for region in [FOLD, SWITCH] {
        if !reach(ctx, session, pointer, region, true)? {
            return Ok(Some(format!(
                "no `{region}` region in the window. Trace: {path}."
            )));
        }
    }
    // Each step: the list, the choice, then (JPEG pictures, searched pictures) owed.
    let steps = [
        (FORMAT_LIST, JPEG, (1, 0), "JPEG at one quality"),
        (
            TARGET_LIST,
            CLOSE,
            (1, 1),
            "JPEG searched down to a closeness",
        ),
    ];
    for (list, choice, owed, meaning) in steps {
        let line = match choose(ctx, session, pointer, list, choice)? {
            Ok(line) => line,
            Err(failure) => return Ok(Some(failure)),
        };
        let got = counts(&line);
        report.note(format!("{meaning}: jpeg={} searched={}", got.0, got.1));
        if got != owed {
            return Ok(Some(format!(
                "after `{choice}` ({meaning}) the copy had (jpeg, searched) = {got:?}; owed \
                 {owed:?}. Line: `{}`. Trace: {path}.",
                line.raw
            )));
        }
    }
    Ok(None)
}
