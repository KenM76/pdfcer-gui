//! `word_gap_band_re_extracts_the_page` — Edit ▸ Content's *Auto* switch and
//! *Gap width* slider each re-extract the page's text under what they set.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/word_gap_band.md`.

use super::dimdrive::{Fixture, click_on, press, run_on, ui_rect_event};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const FIXTURE: Fixture = Fixture {
    file: "word-gap.pdf",
    method: "Regenerate it: python fixtures/word-gap.PROVENANCE.py.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};

const SWITCH: &str = "ribbon.item.word_gap.auto"; // ui-text-exempt: a trace region name, never displayed
const SLIDER: &str = "ribbon.item.word_gap"; // ui-text-exempt: a trace region name, never displayed
const CONTENT: &str = "ribbon.group.edit.content.collapsed"; // ui-text-exempt: a trace region name, never displayed
const BAND: &str = "word-gap-band"; // ui-text-exempt: a trace event name, never displayed
/// Inside the third line of the six-line block.
const AT: (f64, f64) = (120.0, 668.0);
/// Points from the slider rect's left edge to the click: a quarter of its
/// 120 pt track.
const TRACK_AT: f32 = 30.0;
/// Frames to wait for a page's text to be rebuilt under a new setting.
const REBUILD_POLLS: usize = 10;

/// See the module documentation.
pub struct WordGapBandReExtractsThePage;

impl Check for WordGapBandReExtractsThePage {
    fn name(&self) -> &'static str {
        "word_gap_band_re_extracts_the_page"
    }

    fn defect(&self) -> &'static str {
        "Edit > Content's Auto switch or Gap width slider changes the setting without the \
         page's text being re-extracted under it, so a click still splits words the old way"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &FIXTURE,
            "mode.edit,edit.text",
            "word-gap-band",
            drive,
        )
    }
}

/// The band's state once the page's text answers `want`; the last state seen
/// when it never does.
fn band_after(
    session: &Session,
    mark: usize,
    want: impl Fn(&TraceLine) -> bool,
) -> Result<Option<TraceLine>> {
    let mut last = None;
    for _ in 0..REBUILD_POLLS {
        session.settle(20);
        let trace = session.trace()?;
        last = trace.last_after(BAND, mark).cloned();
        if last.as_ref().is_some_and(&want) {
            break;
        }
    }
    Ok(last)
}

/// `line` says Auto is `auto` and the page's text was built under it: measured
/// under Auto, and at exactly the fixed width with Auto off.
fn built_under(line: &TraceLine, auto: bool) -> bool {
    let measured = line.get("measured") == Some("true");
    line.get("auto") == Some(if auto { "true" } else { "false" })
        && line.get("page") != Some("unbuilt")
        && if auto {
            measured
        } else {
            !measured && line.get("page") == line.get("ratio")
        }
}

/// Press `control` (none: just build the page's text), click the paragraph so
/// the page's text is built, and answer the band once `want` holds.
fn step(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    control: Option<&str>,
    want: impl Fn(&TraceLine) -> bool,
) -> Result<Option<TraceLine>> {
    let mark = session.trace()?.mark();
    match control {
        Some(SLIDER) => press_track(ctx, session, pointer)?,
        Some(control) => press(ctx, session, pointer, control)?,
        None => {}
    }
    click_on(ctx, session, pointer, &FIXTURE, 0, AT)?;
    band_after(session, mark, want)
}

/// Click the slider's track a quarter of the way along. The declared rect also
/// holds the value box and the caption, and its centre lands in the value box,
/// which a click opens for typing rather than moving the knob.
fn press_track(ctx: &CheckContext, session: &Session, pointer: &ScriptedPointer) -> Result<()> {
    let (rect, vp) = declared_in(&session.trace()?, ui_rect_event(ctx)?, SLIDER)
        .ok_or_else(|| Error::new(format!("`{SLIDER}` was never declared.")))?;
    let x = rect.min.x + TRACK_AT;
    let y = (rect.min.y + rect.max.y) / 2.0;
    let at = WindowPoint::centre_of(LRect::new(Pt::new(x, y), Pt::new(x, y)));
    pointer.click_in(session, vp.as_deref(), at)?;
    session.settle(30);
    Ok(())
}

/// The failure for a phase whose band line is `line`, or `None` when `ok`.
fn verdict(line: Option<&TraceLine>, ok: bool, what: &str, path: &str) -> Option<String> {
    let raw = line.map_or("nothing", |l| l.raw.as_str());
    (!ok).then(|| format!("★ {what}: the band reads `{raw}`. Trace: {path}."))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    session.settle(30);
    let path = session.trace_path().display().to_string();
    press(ctx, session, pointer, "ribbon.tab.edit")?;
    let ui_rect = ui_rect_event(ctx)?;
    if declared_in(&session.trace()?, ui_rect, SWITCH).is_none()
        && declared_in(&session.trace()?, ui_rect, CONTENT).is_some()
    {
        press(ctx, session, pointer, CONTENT)?;
    }
    let ratio_of = |l: &TraceLine| l.get("ratio").map(str::to_owned);

    let start = step(ctx, session, pointer, None, |l| built_under(l, false))?;
    report.note(format!("at launch: `{:?}`", start.as_ref().map(|l| &l.raw)));
    let ok = start.as_ref().is_some_and(|l| built_under(l, false));
    if let Some(why) = verdict(
        start.as_ref(),
        ok,
        "at launch Auto is not off with the page at the fixed width",
        &path,
    ) {
        return Ok(Some(why));
    }
    let before = start.as_ref().and_then(ratio_of);

    let moved_ok = |l: &TraceLine| built_under(l, false) && ratio_of(l) != before;
    let moved = step(ctx, session, pointer, Some(SLIDER), moved_ok)?;
    report.note(format!(
        "slider pressed: `{:?}`",
        moved.as_ref().map(|l| &l.raw)
    ));
    let ok = moved.as_ref().is_some_and(moved_ok);
    if let Some(why) = verdict(
        moved.as_ref(),
        ok,
        "after Gap width was pressed the width did not change, or the page was not rebuilt at it",
        &path,
    ) {
        return Ok(Some(why));
    }

    let on = step(ctx, session, pointer, Some(SWITCH), |l| {
        built_under(l, true)
    })?;
    report.note(format!("Auto on: `{:?}`", on.as_ref().map(|l| &l.raw)));
    let ok = on.as_ref().is_some_and(|l| built_under(l, true));
    if let Some(why) = verdict(
        on.as_ref(),
        ok,
        "after Auto was switched on the page was not measured",
        &path,
    ) {
        return Ok(Some(why));
    }

    let off = step(ctx, session, pointer, Some(SWITCH), |l| {
        built_under(l, false)
    })?;
    report.note(format!("Auto off: `{:?}`", off.as_ref().map(|l| &l.raw)));
    let ok = off.as_ref().is_some_and(|l| built_under(l, false));
    Ok(verdict(
        off.as_ref(),
        ok,
        "after Auto was switched off the page was not rebuilt at the fixed width",
        &path,
    ))
}
