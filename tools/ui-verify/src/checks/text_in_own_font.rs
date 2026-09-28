//! `typing_is_drawn_in_the_runs_own_font` — **a draft on an existing run looks
//! like the run**: with the caret in real page text, the draft is drawn in the
//! run's own font and place, not in an editor box in the shell's font.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/text_in_own_font.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared};
use crate::checks::save_copy::{click_command, click_tab};
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::PixRect;
use crate::image::Image;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

const MODE: &str = "edit";
const EDIT_TAB: (&str, &str) = ("ribbon.tab.edit", "edit");
const EDIT_TEXT_ITEM: (&str, &str) = ("ribbon.item.edit.text", "edit.text");
/// `text-edit-shaped … shaped=0|1` — whether the in-font layout was built.
const SHAPED_EVENT: &str = "text-edit-shaped"; // ui-text-exempt: a trace event name, never displayed
/// The editor's region: the in-font draft's cover, or the shell-font box.
const REGION_BOX: &str = "text-edit.box"; // ui-text-exempt: a trace region name, never displayed
/// Pixels trimmed off every side of the box before comparing, so the accent
/// outline and an end-of-line caret are outside the comparison.
const INSET_PX: u32 = 3;
/// A channel difference below this is anti-aliasing, not a different glyph.
const PIXEL_DIFFERS: u8 = 64;
/// The largest share of the box's pixels that may differ from the page's own
/// rendering while the draft holds the run's original text.
const SAME_AT_MOST: f64 = 0.03;
/// How much the check zooms in on the run before comparing.
const ZOOM_FACTOR: f32 = 6.0;
/// The most Ctrl+wheel notches spent reaching [`ZOOM_FACTOR`].
const ZOOM_SPINS: usize = 40;

/// See the module documentation.
pub struct TypingIsDrawnInTheRunsOwnFont;

impl Check for TypingIsDrawnInTheRunsOwnFont {
    fn name(&self) -> &'static str {
        "typing_is_drawn_in_the_runs_own_font"
    }

    fn defect(&self) -> &'static str {
        "Typing into existing page text shows the draft in an editor box in the program's own \
         font, not in the drawing's font at the text's place — the operator cannot see what the \
         edit will look like until it is committed"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The share of pixels in `region` that have no counterpart within one pixel
/// in the other image, either way round. A counterpart is a pixel whose
/// largest channel difference is below [`PIXEL_DIFFERS`].
///
/// The page and the draft are drawn by two rasterisers, so glyph edges differ
/// in anti-aliasing by up to a pixel even when the glyphs agree. A different
/// typeface, size or place moves ink by more than that.
fn share_differing(a: &Image, b: &Image, region: PixRect) -> f64 {
    let has_match = |from: &Image, to: &Image, x: u32, y: u32| {
        let Some(p) = from.pixel(x, y) else {
            return false;
        };
        (x.saturating_sub(1)..=x + 1).any(|qx| {
            (y.saturating_sub(1)..=y + 1).any(|qy| {
                to.pixel(qx, qy)
                    .is_some_and(|q| crate::checks::driving::delta(p, q) < u16::from(PIXEL_DIFFERS))
            })
        })
    };
    let (mut n, mut differ) = (0_u64, 0_u64);
    for y in region.y..region.y + region.h {
        for x in region.x..region.x + region.w {
            n += 1;
            if !has_match(a, b, x, y) || !has_match(b, a, x, y) {
                differ += 1;
            }
        }
    }
    if n == 0 {
        1.0
    } else {
        differ as f64 / n as f64
    }
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let pdf = ctx.pdf.clone().ok_or_else(|| Error::new("no --pdf."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. It must name page text: a click on empty page opens a new-text \
             draft, which is drawn in the shell's font by design.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input); reported as SKIPPED rather than passed.",
        ));
    }
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
        None => crate::fixture::page_geometry(&pdf)
            .ok_or_else(|| Error::new("cannot read a page size. Pass --page-size WxH."))?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("text_in_own_font.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);
    click_tab(&session, &driver, ui_rect, EDIT_TAB)?;
    session.settle(14);
    click_command(&session, &driver, ui_rect, EDIT_TEXT_ITEM, 18)?;

    // Zoom in on the run: at fit-page a drawing's text is a few pixels tall,
    // too small for a glyph-by-glyph comparison.
    let aim_now = || {
        crate::checks::text_selection::aim(
            ctx,
            &session,
            page,
            DocPoint::new(target.page, target.x, target.y),
        )
    };
    let start = crate::checks::scale_aim::current_zoom(&session)?;
    for _ in 0..ZOOM_SPINS {
        if crate::checks::scale_aim::current_zoom(&session)? >= start * ZOOM_FACTOR {
            break;
        }
        driver.scroll_at_held(aim_now()?, &[vk::CONTROL], 1, 1)?;
        session.settle(10);
    }
    session.settle(60);
    report.note(format!(
        "zoomed from {start:.3} to {:.3}",
        crate::checks::scale_aim::current_zoom(&session)?
    ));

    // The page's own rendering of the run, before any draft covers it.
    let before_path = ctx.out("own-font.before.png");
    let before = crate::capture::window_to_png(&session, &before_path)?;
    report.artifact(before_path);

    let at = crate::checks::text_selection::aim(
        ctx,
        &session,
        page,
        DocPoint::new(target.page, target.x, target.y),
    )?;
    driver.click_at(at)?;
    session.settle(24);
    if session.trace()?.last("text-edit-caret").is_none() {
        return Err(Error::new(
            "the click resolved no run, so there is no draft to look at. The --doc-point does \
             not name page text on this build's reading of the fixture.",
        ));
    }
    // Caret to the start: an X typed there moves every glyph after it, a
    // change the comparison cannot miss, and the caret line sits in the inset.
    driver.press(vk::HOME)?;
    session.settle(8);

    // One character added: the draft now differs from the page.
    driver.type_ascii("X")?;
    session.settle(30);
    let typed_path = ctx.out("own-font.typed.png");
    let typed = crate::capture::window_to_png(&session, &typed_path)?;
    report.artifact(typed_path);
    let trace = session.trace()?;
    let shaped = trace.last(SHAPED_EVENT).map(|l| {
        (
            l.get("shaped").map(str::to_owned),
            l.get("refused").map(str::to_owned),
            l.get("chars").map(str::to_owned),
        )
    });
    report.note(format!("after typing X: {SHAPED_EVENT} = {shaped:?}"));
    let Some((Some(flag), refused, _)) = shaped else {
        return Ok(Some(format!(
            "no `{SHAPED_EVENT}` line after typing into a run: the in-font layout was never \
             asked for, so the draft can only be the shell-font box."
        )));
    };
    if flag != "1" {
        return Ok(Some(format!(
            "`{SHAPED_EVENT} shaped={flag} refused={refused:?}`: the engine did not lay the \
             draft out, and the shell-font box was drawn instead."
        )));
    }

    // Back to the original text: now the in-font draft must look like the page.
    driver.press(vk::BACKSPACE)?;
    session.settle(30);
    let same_path = ctx.out("own-font.original.png");
    let same = crate::capture::window_to_png(&session, &same_path)?;
    report.artifact(same_path);

    let trace = session.trace()?;
    let body = declared(&trace, ui_rect, REGION_BOX).ok_or_else(|| {
        Error::new(format!(
            "no `{REGION_BOX}` region with a draft open: nothing says where the draft was drawn."
        ))
    })?;
    let px = session.frame()?.logical_to_capture_pixels(body);
    let inner = PixRect {
        x: px.x + INSET_PX,
        y: px.y + INSET_PX,
        w: px.w.saturating_sub(2 * INSET_PX),
        h: px.h.saturating_sub(2 * INSET_PX),
    };
    if inner.w == 0 || inner.h == 0 {
        return Err(Error::new(format!(
            "the `{REGION_BOX}` region {px:?} is too small to compare once inset."
        )));
    }
    if crate::pixels::region_not_uniform(&before, inner).is_uniform() {
        return Err(Error::new(format!(
            "the page under `{REGION_BOX}` {inner:?} carries no ink before the click, so the \
             comparison below has nothing to match. The --doc-point is not on visible text."
        )));
    }

    let typed_share = share_differing(&before, &typed, inner);
    let same_share = share_differing(&before, &same, inner);
    report.note(format!(
        "pixels differing from the page render inside {inner:?}: with X typed {:.1} %, with \
         the original text {:.1} % (at most {:.1} % allowed)",
        typed_share * 100.0,
        same_share * 100.0,
        SAME_AT_MOST * 100.0
    ));
    // Calibration: the comparison must be able to see a one-character change.
    if typed_share <= SAME_AT_MOST {
        return Err(Error::new(format!(
            "an extra X changed only {:.1} % of the box, no more than the {:.1} % allowed for \
             'looks the same': this comparison cannot tell a draft from the page here.",
            typed_share * 100.0,
            SAME_AT_MOST * 100.0
        )));
    }
    if same_share > SAME_AT_MOST {
        return Ok(Some(format!(
            "with the original text back in the draft, {:.1} % of the box differs from how the \
             page itself draws that text (at most {:.1} % allowed). The draft is not drawn in \
             the run's own font and place. Compare own-font.before.png with \
             own-font.original.png.",
            same_share * 100.0,
            SAME_AT_MOST * 100.0
        )));
    }
    driver.press(vk::ESCAPE)?;
    session.settle(10);
    Ok(None)
}
