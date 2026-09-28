//! `a_font_file_restyles_swept_text` — **sweep text, press Font file…, pick
//! a font file, and the text is redrawn in that face.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/font_file_face.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::{LRect, PixRect, Pt};
use crate::image::Image;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose canvas may select page content and whose panels carry
/// Properties.
const MODE: &str = "edit";
/// The commands run at startup, in order, before this check touches anything.
const INVOKE: &str = "mode.edit,file.properties";
/// The Text section's own region.
const SECTION_REGION: &str = "properties.text";
/// The Font file… button beside the face chooser.
const BUTTON_REGION: &str = "properties.text.font-file";
/// The canvas viewport, the clip for the pixel comparison.
const VIEWPORT_REGION: &str = "canvas-viewport"; // ui-text-exempt: a trace region name
/// The environment variable that answers the file picker without a dialog.
const PICK_ENV: &str = "PDFCER_DIAG_FONT_FILE_PATH"; // ui-text-exempt: an environment variable name
/// The face picked. Comic Sans: every Windows install carries it, and its
/// letter shapes differ from the fixture's Helvetica at every glyph, so a
/// restyle that reached the page cannot leave the pixels as they were.
const FONT_FILE: &str = r"C:\Windows\Fonts\comic.ttf";
/// The picker's own trace line.
const PICKED_EVENT: &str = "font-file-picked";
/// The `text-style-applied page=… change=… applied=… of=…` line.
const STYLE_EVENT: &str = "text-style-applied";
/// The `text-style-declined …` line.
const DECLINED_EVENT: &str = "text-style-declined";
/// The label `vector_edit` writes when the restyle reached the engine.
const APPLIED: &str = "format-text";
/// The sweep's own oracle, shared with `restyle_text` and `std14_face`.
const SELECTION_EVENT: &str = "canvas-text-selection";
/// How far to sweep along the baseline, in PDF points.
const SWEEP_PT: f64 = 60.0;
/// How far below and above the baseline the compared box reaches, in points.
const DESCENT_PT: f64 = 4.0;
/// Cap height of the fixture's 12 pt line, rounded up.
const ASCENT_PT: f64 = 11.0;
/// How many scroll notches to spend looking for the button below the fold.
const SCROLL_ATTEMPTS: usize = 6;
/// The fraction of the text box's pixels that must change. A different
/// face moves most of the ink; a repaint of the same face moves none.
const MIN_CHANGED: f64 = 0.02;
/// `T`, as a Windows virtual key — the text-sweep tool.
const VK_T: u16 = 0x54;

/// See the module documentation.
pub struct AFontFileRestylesSweptText;

impl Check for AFontFileRestylesSweptText {
    fn name(&self) -> &'static str {
        "a_font_file_restyles_swept_text"
    }

    fn defect(&self) -> &'static str {
        "pdfcer-core can restyle text into a face from a font file, embedding the subset the \
         text needs, and the shell's face chooser offers only faces already in the document or \
         the fourteen standard ones — so the capability is released and unreachable; or the \
         button is there and the restyle never reaches the page"
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

/// Poll until the restyle reports one way or the other.
fn wait_for_verdict(session: &Session) -> Result<u128> {
    const CEILING_MS: u128 = 20_000;
    let started = std::time::Instant::now();
    loop {
        session.settle(4);
        let trace = session.trace()?;
        if trace.last(STYLE_EVENT).is_some() || trace.last(DECLINED_EVENT).is_some() {
            return Ok(started.elapsed().as_millis());
        }
        if started.elapsed().as_millis() > CEILING_MS {
            return Ok(started.elapsed().as_millis());
        }
    }
}

/// The fraction of pixels in `region` that differ between two captures.
fn changed(a: &Image, b: &Image, region: PixRect) -> f64 {
    let mut total = 0_u32;
    let mut moved = 0_u32;
    for (p, q) in a.pixels_in(region).zip(b.pixels_in(region)) {
        total += 1;
        let d =
            p.r.abs_diff(q.r)
                .max(p.g.abs_diff(q.g))
                .max(p.b.abs_diff(q.b));
        if d > 48 {
            moved += 1;
        }
    }
    f64::from(moved) / f64::from(total.max(1))
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    // PINNED: `--pdf` and `--doc-point` are read and IGNORED, as in `std14_face`.
    let (pdf, target) = crate::fixture::text_point_target();
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the text fixture is not at {}. It is committed to this repository, so an absence \
             is a broken checkout and is reported as a failure.",
            pdf.display()
        )));
    }
    if !std::path::Path::new(FONT_FILE).is_file() {
        return Err(Error::new(format!(
            "{FONT_FILE} is not on this machine, so there is no font file to pick. SKIPPED: \
             the precondition is the machine's, not the program's."
        )));
    }
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check sweeps the pointer across text and \
             presses a button, and neither can be simulated from the trace.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("font-file-face.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env.push((PICK_ENV.to_owned(), FONT_FILE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with {PICK_ENV}={FONT_FILE}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- 1: Edit mode, then sweep across the text ---------------------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let frame = session.frame()?;
    let start =
        frame.to_screen(mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?);
    let end = frame.to_screen(mapping.doc_to_window(DocPoint::new(
        target.page,
        target.x + SWEEP_PT,
        target.y,
    ))?);
    driver.press(VK_T)?;
    session.settle(16);
    driver.drag(start, end)?;
    session.settle(24);

    let trace = session.trace()?;
    let swept = trace
        .events(SELECTION_EVENT)
        .last()
        .and_then(|l| l.get("chars"))
        .and_then(|n| n.parse::<usize>().ok())
        .unwrap_or(0);
    if swept == 0 {
        return Err(Error::new(format!(
            "the drag selected no text, so there was no run for the button to be about. \
             SKIPPED: that is the harness's aim, not the program's behaviour. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!("the sweep selected {swept} character(s)"));

    // --- 2: the button, scrolling as an operator would ----------------------
    let mut button = None;
    for _ in 0..SCROLL_ATTEMPTS {
        let trace = session.trace()?;
        if let Some(rect) = driving::declared(&trace, ui_rect, BUTTON_REGION) {
            button = Some(rect);
            break;
        }
        let Some(section) = driving::declared(&trace, ui_rect, SECTION_REGION) else {
            return Err(Error::new(format!(
                "{swept} character(s) are selected and no `{SECTION_REGION}` region is on \
                 screen. SKIPPED: `restyle_text` owns that defect. Trace: {}.",
                session.trace_path().display()
            )));
        };
        driver.scroll_at(session.frame()?.declared_center(section), -1)?;
        session.settle(12);
    }
    let Some(button) = button else {
        let seen = driving::live_names(&session.trace()?, ui_rect, SECTION_REGION);
        return Ok(Some(format!(
            "★ THE FONT FILE BUTTON IS NOT ON SCREEN: no `{BUTTON_REGION}` region after \
             scrolling the Properties panel {SCROLL_ATTEMPTS} times, with the Text section \
             drawn. `face_row` draws it beside the face chooser; regions beginning \
             `{SECTION_REGION}`: {seen:?}. Trace: {}.",
            session.trace_path().display()
        )));
    };

    // --- 3: the pixels before ------------------------------------------------
    let trace = session.trace()?;
    let frame = session.frame()?;
    let canvas = driving::declared(&trace, ui_rect, VIEWPORT_REGION).ok_or_else(|| {
        Error::new(format!(
            "no `{VIEWPORT_REGION}` region, so there is no clip for the text box. SKIPPED."
        ))
    })?;
    let canvas_px = frame.logical_to_capture_pixels(canvas);
    let a = mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y - DESCENT_PT))?;
    let b = mapping.doc_to_window(DocPoint::new(
        target.page,
        target.x + SWEEP_PT,
        target.y + ASCENT_PT,
    ))?;
    let text_l = LRect::new(
        Pt::new(a.x().min(b.x()), a.y().min(b.y())),
        Pt::new(a.x().max(b.x()), a.y().max(b.y())),
    );
    let r = frame.logical_to_capture_pixels(text_l);
    let x0 = r.x.max(canvas_px.x);
    let y0 = r.y.max(canvas_px.y);
    let x1 = (r.x + r.w).min(canvas_px.x + canvas_px.w);
    let y1 = (r.y + r.h).min(canvas_px.y + canvas_px.h);
    if x1 <= x0 || y1 <= y0 {
        return Err(Error::new(format!(
            "the swept text box lies outside the canvas viewport, so there are no pixels to \
             compare. SKIPPED. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let text_px = PixRect::new(x0, y0, x1 - x0, y1 - y0);
    let before_shot = ctx.out("font_file_face_before.png");
    let before = crate::capture::window_to_png(&session, &before_shot)?;
    report.artifact(before_shot);

    // --- 4: press the button -----------------------------------------------
    driver.click_at(frame.declared_center(button))?;
    let waited = wait_for_verdict(&session)?;
    report.note(format!("the restyle answered in {waited} ms"));

    let trace = session.trace()?;
    let Some(picked) = trace.events(PICKED_EVENT).last() else {
        return Ok(Some(format!(
            "★ THE BUTTON WAS PRESSED AND NO FILE WAS ASKED FOR: no `{PICKED_EVENT}` line. \
             Either the click missed the button, or the press raised no \
             `StyleChange::FaceFile(None)`, or `textstyle::apply` did not route it to \
             `files::pick_font_file`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if picked.get("source") != Some("env") {
        return Ok(Some(format!(
            "the picker answered `{}`, not from {PICK_ENV}: a native dialog opened, which this \
             check cannot drive. Trace: {}.",
            picked.raw,
            session.trace_path().display()
        )));
    }
    if let Some(declined) = trace.events(DECLINED_EVENT).last() {
        return Ok(Some(format!(
            "★ A FONT FILE WAS PICKED AND THE RESTYLE DECLINED: `{}`.\n\
             A `font-file=` decline is `plan_subset` refusing the file — {FONT_FILE} is an \
             ordinary TrueType face, so that is an engine regression to report. Any other \
             decline is `format_text` refusing the embedded plan, whose refusal the line \
             names. Trace: {}.",
            declined.raw,
            session.trace_path().display()
        )));
    }
    let Some(applied) = trace.events(STYLE_EVENT).last() else {
        return Ok(Some(format!(
            "a file was picked (`{}`) and neither `{STYLE_EVENT}` nor `{DECLINED_EVENT}` \
             followed within the ceiling. Trace: {}.",
            picked.raw,
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the restyle reported `{}`", applied.raw));
    if applied.get("change") != Some("face-file") {
        return Ok(Some(format!(
            "the last restyle was `{}`, not `change=face-file`, so something other than the \
             button restyled the text. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    let n: usize = applied
        .get("applied")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if n == 0 || trace.last(APPLIED).is_none() {
        return Ok(Some(format!(
            "the restyle reported `{}` and {} `{APPLIED}` line, so nothing reached the \
             document. Trace: {}.",
            applied.raw,
            if trace.last(APPLIED).is_some() {
                "a"
            } else {
                "no"
            },
            session.trace_path().display()
        )));
    }

    // --- 5: the pixels after -------------------------------------------------
    //
    // The trace says the engine accepted it; only the page says the saved
    // content now draws in another face. The selection highlight is present
    // in both captures, so it cancels.
    session.settle(24);
    let after_shot = ctx.out("font_file_face_after.png");
    let after = crate::capture::window_to_png(&session, &after_shot)?;
    report.artifact(after_shot);
    let fraction = changed(&before, &after, text_px);
    report.note(format!(
        "{:.1}% of the text box's pixels changed ({}×{} px)",
        fraction * 100.0,
        text_px.w,
        text_px.h
    ));
    if fraction < MIN_CHANGED {
        return Ok(Some(format!(
            "★★ THE ENGINE ACCEPTED THE RESTYLE AND THE PAGE LOOKS THE SAME: only {:.1}% of \
             the swept text's pixels changed, under {:.0}%. Either the page was not \
             re-rendered after the edit, or the embedded face was written and the renderer \
             drew the old one. The two screenshots beside this report are the evidence. \
             Trace: {}.",
            fraction * 100.0,
            MIN_CHANGED * 100.0,
            session.trace_path().display()
        )));
    }
    report.note(
        "★★★ the swept text was redrawn in a face taken from a font file, through \
         `format_text` with an embedded subset",
    );
    Ok(None)
}
