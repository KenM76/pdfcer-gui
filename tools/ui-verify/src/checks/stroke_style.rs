//! Properties' "Line and opacity" section, driven on a line drawn under a
//! half-scale CTM and on a picture: width and dash are typed in points and
//! reach the file in the object's own user space, opacity reaches both.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/stroke_style.md`.

use super::dimdrive::{Fixture, click_on, press, run_on, ui_rect_event};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, WindowPoint};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.edit,file.properties";
const SHOWN: &str = "stroke-style-shown"; // ui-text-exempt: a trace event name, never displayed
const CALLS: &str = "stroke-style-calls"; // ui-text-exempt: a trace event name, never displayed
/// Mirrors `panels::properties::strokestyle`'s regions.
const WIDTH: &str = "properties.stroke.width";
const DASH: &str = "properties.stroke.dash";
/// The third entry of `LineStyle::ALL`, the long dash, `[8 4]` points.
const LONG_DASH: &str = "properties.stroke.dash.2";
const LINE_ALPHA: &str = "properties.stroke.line-opacity";
const PICTURE_ALPHA: &str = "properties.stroke.picture-opacity";
/// The Properties tab's body, where the wheel is aimed.
const PANEL_BODY: &str = "dock.body.file.properties";
/// How many wheel steps to spend bringing a row into view.
const SCROLL_NOTCHES: usize = 12;

const HALF_SCALE_LINE: Fixture = Fixture {
    file: "half-scale-line.pdf",
    method: "Rebuild it with `python fixtures/half-scale-line.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 300.0,
    },
};
/// On the line, page space; clear of the picture.
const ON_THE_LINE: (f64, f64) = (200.0, 150.0);
/// Inside the picture, page space.
const ON_THE_PICTURE: (f64, f64) = (80.0, 60.0);
/// The line's CTM scale: user units per point is its inverse.
const SCALE: f64 = 0.5;

/// A width and a dash typed in points land in the line's user space.
pub struct ALineWidthIsTypedInPoints;
/// A picture's opacity is set from the same section.
pub struct APicturesOpacityCanBeSet;

impl Check for ALineWidthIsTypedInPoints {
    fn name(&self) -> &'static str {
        "a_line_width_is_typed_in_points"
    }

    fn defect(&self) -> &'static str {
        "a selected page line offers no width, dash or opacity, or a width typed in points is \
         written in the line's own units, so a line drawn at half scale ends at half the width \
         typed"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &HALF_SCALE_LINE,
            INVOKE,
            "stroke-style-line",
            line,
        )
    }
}

impl Check for APicturesOpacityCanBeSet {
    fn name(&self) -> &'static str {
        "a_pictures_opacity_can_be_set"
    }

    fn defect(&self) -> &'static str {
        "a selected picture offers no opacity, or the value typed never reaches the page"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &HALF_SCALE_LINE,
            INVOKE,
            "stroke-style-picture",
            picture,
        )
    }
}

fn line(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    click_on(ctx, session, pointer, &HALF_SCALE_LINE, 0, ON_THE_LINE)?;
    let shown = shown_since(session, 0)?;
    report.note(format!("selected: `{shown}`"));
    if !(shown.contains("paths=1 ") && shown.contains("width_pt=2.000")) {
        return Ok(Some(format!(
            "the half-scale line (4 user units, 2 pt on the page) reads `{shown}`; it must read \
             one path at `width_pt=2.000`. Trace: {}.",
            session.trace_path().display()
        )));
    }

    let mark = session.trace()?.mark();
    type_into(ctx, session, pointer, WIDTH, "5")?;
    let trace = session.trace()?;
    let Some(calls) = trace.last_after(CALLS, mark) else {
        return Ok(Some(format!(
            "5 was typed into the width field and nothing was applied (no `{CALLS}`). Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("width applied: `{}`", calls.raw));
    let want_user = 5.0 / SCALE;
    if calls.get("user_widths") != Some(&format!("{want_user:.3}")[..]) {
        return Ok(Some(format!(
            "5 pt on a line drawn at scale {SCALE} must reach the engine as {want_user:.3} user \
             units; the call was `{}`. Trace: {}.",
            calls.raw,
            session.trace_path().display()
        )));
    }
    let shown = shown_since(session, mark)?;
    if !shown.contains("width_pt=5.000") {
        return Ok(Some(format!(
            "the width was applied and the section re-reads `{shown}`; it must read 5 pt. \
             Trace: {}.",
            session.trace_path().display()
        )));
    }

    let mark = session.trace()?.mark();
    into_view(ctx, session, pointer, DASH)?;
    press(ctx, session, pointer, DASH)?;
    press(ctx, session, pointer, LONG_DASH)?;
    let shown = shown_since(session, mark)?;
    report.note(format!("after the long dash: `{shown}`"));
    if !shown.contains("dash=long-dash") {
        return Ok(Some(format!(
            "the long dash ([8 4] pt) was picked and the line re-reads `{shown}`. Written in \
             user units unconverted it reads back at half length, which no entry matches \
             (`dash=foreign`). Trace: {}.",
            session.trace_path().display()
        )));
    }

    let mark = session.trace()?.mark();
    type_into(ctx, session, pointer, LINE_ALPHA, "40")?;
    let shown = shown_since(session, mark)?;
    report.note(format!("after line opacity 40 %: `{shown}`"));
    Ok(
        (!(shown.contains("line_alpha=0.400") && shown.contains("width_pt=5.000"))).then(|| {
            format!(
                "line opacity 40 % was typed and the line re-reads `{shown}`; it must read \
             `line_alpha=0.400` with the width left at 5 pt. Trace: {}.",
                session.trace_path().display()
            )
        }),
    )
}

fn picture(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    click_on(ctx, session, pointer, &HALF_SCALE_LINE, 0, ON_THE_PICTURE)?;
    let shown = shown_since(session, 0)?;
    report.note(format!("selected: `{shown}`"));
    if !(shown.contains("paths=0 ") && shown.contains("pictures=1 ")) {
        return Ok(Some(format!(
            "a click inside the picture shows `{shown}`; it must show one picture and no path. \
             Trace: {}.",
            session.trace_path().display()
        )));
    }
    let mark = session.trace()?.mark();
    type_into(ctx, session, pointer, PICTURE_ALPHA, "50")?;
    let shown = shown_since(session, mark)?;
    report.note(format!("after picture opacity 50 %: `{shown}`"));
    Ok((!shown.contains("picture_alpha=0.500")).then(|| {
        format!(
            "picture opacity 50 % was typed and the picture re-reads `{shown}`. Trace: {}.",
            session.trace_path().display()
        )
    }))
}

/// Click `region`, replace its text with `text`, and press Enter.
fn type_into(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
    text: &str,
) -> Result<()> {
    into_view(ctx, session, pointer, region)?;
    let vp = declared_in(&session.trace()?, ui_rect_event(ctx)?, region).and_then(|(_, vp)| vp);
    press(ctx, session, pointer, region)?;
    pointer.key(session, vp.as_deref(), "A", Some("ctrl"))?;
    pointer.type_text(session, vp.as_deref(), text)?;
    pointer.key(session, vp.as_deref(), "Enter", None)?;
    session.settle(40);
    Ok(())
}

/// Wheel the Properties panel until `region` is on screen; the section sits
/// below the object's geometry rows, under the fold of an off-screen window.
fn into_view(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
) -> Result<()> {
    let ui_rect = ui_rect_event(ctx)?;
    for _ in 0..SCROLL_NOTCHES {
        let trace = session.trace()?;
        if declared_in(&trace, ui_rect, region).is_some() {
            return Ok(());
        }
        let Some((body, vp)) = declared_in(&trace, ui_rect, PANEL_BODY) else {
            break;
        };
        pointer.wheel_in(session, vp.as_deref(), WindowPoint::centre_of(body), -3.0)?;
        session.settle(15);
    }
    Ok(())
}

/// The last `stroke-style-shown` line after trace line `mark`, or a token
/// saying there was none, which every caller's assertion then rejects.
fn shown_since(session: &Session, mark: usize) -> Result<String> {
    Ok(session
        .trace()?
        .last_after(SHOWN, mark)
        .map_or_else(|| format!("<no {SHOWN} line>"), |l| l.raw.clone()))
}
