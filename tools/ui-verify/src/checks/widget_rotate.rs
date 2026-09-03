//! `turning_a_field_right_turns_it_right` — the driven proof of
//! `OPERATOR_REQUESTS.md` **O62**'s rotation half.
//!
//! # ★★★ This check exists for ONE arithmetic sign
//!
//! `/MK /R` is **counterclockwise**. The page's `/Rotate` is **clockwise**. The
//! engine flagged this as *"the single most likely thing for a shell to get
//! backwards"*, and the standard makes it easy — the two entries are word for
//! word parallel:
//!
//! | | |
//! |---|---|
//! | `/MK /R` (Table 189) | *"…rotated **counterclockwise** relative to the page…"* |
//! | page `/Rotate` (Table 30) | *"…rotated **clockwise** when displayed or printed…"* |
//!
//! **The direction word is the only difference between those two sentences**,
//! and the *movie* dictionary's `/Rotate` uses the same *"relative to the page"*
//! phrase with the opposite sense — so the phrase carries no convention at all.
//!
//! ⇒ A shell that got this backwards would ship two buttons that both work,
//! both write a legal angle, and both turn the box **the wrong way**. Nothing
//! fails. Every unit test of the arithmetic passes, because the arithmetic is
//! not what is wrong — the *meaning* is.
//!
//! # The oracle, and why a sign is exactly what it reads
//!
//! Press **Turn right** once on a widget at 0, and assert the engine received
//! **270**.
//!
//! - right = clockwise = **−90**
//! - −90 normalised into `0..360` = **270**
//!
//! If the negation were missing, the same press would produce **90** — a legal,
//! successful, silent rotation the wrong way. That single number is the whole
//! subject, and it is why this check asserts a value rather than a change.
//!
//! ★ It reads `rotate-widget-applied … now=`, the **engine's** report, not the
//! shell's request line. The request says what the panel computed; only the
//! applied line says what `rotate_widget` was actually given and accepted.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Edit mode, the text-field tool armed, and the Properties panel open.
///
/// ★★★ The properties panel is **not** opened here, and the first version's
/// attempt to is why.
///
/// `view.panel_properties` is a TOGGLE, and opening it re-docks the canvas
/// narrower — after which the coordinate this check computed for its placement
/// click pointed somewhere else. The symptom was not a missed click: it was
/// *"the window containing (1224, 538) could not be brought to the front"*,
/// three runs running, because the point had moved over a different window
/// entirely.
///
/// ⇒ `D:/dev/rag/egui/` already carries this as **harness coordinates going
/// stale when a dock width changes**. The panel is open in Edit mode by
/// default, so the toggle was never needed — it was doing nothing but moving
/// the canvas out from under the check.
const INVOKE: &str = "mode.edit,edit.form_text_field";

/// The placement dialog accepts itself, so no dialog has to be driven.
const ACCEPT_ENV: (&str, &str) = ("PDFCER_DIAG_FORM_ACCEPT", "1");

/// The per-widget census the canvas publishes.
const BOX_LINE: &str = "form-target";

/// The **Turn right** button's own region.
///
/// ★★ Its own, not a fraction of the row's. The first version took the row's
/// rect and aimed 78 % across it — coordinate arithmetic the harness already
/// has `declared_center` for — and landed outside the window entirely, which
/// surfaced as *"the window could not be brought to the front"* three runs
/// running. A named control is aimed at by name.
const ROTATE_RIGHT_REGION: &str = "properties.widget_edit.rotate_right";

/// The engine's own report.
const APPLIED: &str = "rotate-widget-applied";

/// The properties panel's own body, for scrolling.
///
/// ★★★ The first version of this check did not scroll, and that is why it
/// reported the feature as inert on 2026-08-30 while the feature worked.
///
/// The properties panel on a selected form field is well over a thousand points
/// of content in a dock slot a few hundred tall. The rotation row sits with the
/// geometry rows, **below the fold**, and a control below the fold is published
/// at its *content* position — outside the window entirely. Clicking that
/// coordinate presses whatever is there instead, which is nothing.
///
/// ⇒ Scroll until the row is inside the panel, and if it never gets there say
/// **that**, because a control an operator cannot reach is a control that does
/// not exist whatever its click arm does. `adopt_widget` set this pattern after
/// the same mistake produced three false defect reports in one day.
// ui-text-exempt: trace region name, never displayed
//
/// ★ `file.properties`, NOT `view.panel_properties`. The properties panel is
/// the one command in the ribbon whose id names the *File* group rather than
/// the View group, because it is the document's own properties surface; the
/// dock body takes the command id verbatim. `app::panels` records the same
/// trap. Guessing the obvious name here produced a SKIP that read as
/// "the panel is closed" while it was open the whole time.
const PANEL_BODY: &str = "dock.body.file.properties";

/// How many wheel turns before the check gives up and reports it.
const MAX_SCROLL: usize = 20;

/// Where the field is placed, as page fractions.
const PLACE_AT: (f64, f64) = (0.30, 0.45);

/// See the module documentation.
pub struct TurningAFieldRightTurnsItRight;

impl Check for TurningAFieldRightTurnsItRight {
    fn name(&self) -> &'static str {
        "turning_a_field_right_turns_it_right"
    }

    fn defect(&self) -> &'static str {
        "the Turn right button turns the box left — /MK /R is counterclockwise while the page's \
         /Rotate is clockwise, the standard's two sentences differ by that one word, and a shell \
         that misses it ships two buttons that both work, both write a legal angle, and both go \
         the wrong way with nothing failing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check places a field, selects it and presses a \
             button. Reported as SKIPPED rather than passed.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. This check needs a page to place a field on."))?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let page: PageGeometry = match ctx.page_size {
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("widget-rotate.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env
        .push((ACCEPT_ENV.0.to_owned(), ACCEPT_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    // ★ Normalise the saved dock layout: `view.panel_properties` is a TOGGLE and
    // the layout persists, so without this the panel alternates open and closed
    // across runs. Same rule, same file, as the bookmark clipboard check.
    if let Some(dir) = exe.parent() {
        let layout = dir.join("userdata").join("layout.ron");
        if layout.exists() {
            let _ = std::fs::remove_file(&layout);
        }
    }

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(60);
    let driver = Driver::new(session.window());

    // --- place a field, so the check does not depend on the fixture ---------
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let at = DocPoint::new(0, PLACE_AT.0 * page.width_pt, PLACE_AT.1 * page.height_pt);
    driver.click_at(session.frame()?.to_screen(mapping.doc_to_window(at)?))?;
    session.settle(35);

    if session.trace()?.events(BOX_LINE).next().is_none() {
        return Err(Error::new(format!(
            "the text-field tool placed nothing: no `{BOX_LINE}` line after a click on the page. \
             That is `field_menu`'s surface, two steps before this check's subject. SKIPPED. \
             Trace: {}",
            session.trace_path().display()
        )));
    }
    // ★ The field authored here is left SELECTED (O53), so the properties panel
    // is already showing its widget section — no selecting click is needed, and
    // adding one would place a second field because the tool stays armed.
    report.note("★ placed a field; it is selected, so the widget section is showing");

    // --- scroll the rotation row into view ----------------------------------
    //
    // ★★ The region is published only when it is VISIBLE, since 2026-08-30. So
    // its absence here is not "the button does not exist" — it is "the button is
    // below the fold", which is a different finding and has a different remedy.
    let panel = declared(&session.trace()?, ui_rect, PANEL_BODY);
    let mut button = declared(&session.trace()?, ui_rect, ROTATE_RIGHT_REGION);
    let mut turns = 0;
    while button.is_none() && turns < MAX_SCROLL {
        let Some(panel) = panel else {
            return Err(Error::new(format!(
                "no `{PANEL_BODY}` region, so there is nowhere to point the wheel. The properties \
                 panel is either closed or docked under a name this check does not know. SKIPPED \
                 rather than reported as a defect in the button."
            )));
        };
        driver.scroll_at(session.frame()?.declared_center(panel), -3)?;
        session.settle(12);
        turns += 1;
        button = declared(&session.trace()?, ui_rect, ROTATE_RIGHT_REGION);
    }
    let Some(button) = button else {
        let names = declared_names(&session.trace()?, ui_rect, "properties.widget_edit");
        return Ok(Some(format!(
            "after {MAX_SCROLL} wheel turns there is still no VISIBLE `{ROTATE_RIGHT_REGION}` \
             region, so the Turn right button cannot be reached by scrolling either. A control an \
             operator cannot get to is a control that does not exist, whatever its click arm does. \
             Visible regions beginning `properties.widget_edit`: {}. Trace: {}",
            list(&names),
            session.trace_path().display()
        )));
    };
    report.note(format!("★ scrolled {turns} wheel turn(s) to reach the row"));
    report.note(format!("the Turn right button is at {button:?}"));
    driver.click_at(session.frame()?.declared_center(button))?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(applied) = trace.events(APPLIED).last() else {
        return Ok(Some(format!(
            "pressing the right-hand rotation button produced no `{APPLIED}` line, so \
             `rotate_widget` was never reached. Either the click landed between the two buttons \
             or the action has no arm. Trace: {}",
            session.trace_path().display()
        )));
    };
    let now = applied.get("now").unwrap_or("?");
    report.note(format!("★ the engine reports now={now}"));

    // ★★★ THE ASSERTION. `Some(270)`, because right is clockwise is −90, and
    // −90 normalised into 0..360 is 270. A missing negation gives `Some(90)` —
    // legal, successful, and the wrong way round.
    if !now.contains("270") {
        return Ok(Some(format!(
            "★★★ TURN RIGHT PRODUCED `now={now}`, AND IT MUST BE 270. `/MK /R` is \
             COUNTERCLOCKWISE (Table 189) while the page's `/Rotate` is clockwise (Table 30) — \
             the two sentences in the standard differ by that one word. A right turn is \
             therefore −90 counterclockwise, which normalises to 270. `now=90` means the \
             negation in `widgetedit::rotation_row` is missing or inverted, and the operator's \
             box turns the wrong way with nothing failing anywhere. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note("★★ 270 — right is clockwise is −90 counterclockwise, and the negation held");

    Ok(None)
}
