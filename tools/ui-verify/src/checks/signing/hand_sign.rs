//! `checks::signing::hand_sign` — a signature drawn with the mouse lands in
//! the clicked box as page ink, undoes in one step, and saves as no digital
//! signature.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/signing/hand_sign.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_in, declared_names, list, repo_fixture,
};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::image::Image;
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "esign-three-boxes.pdf";
const REGION_BOX: &str = "form.sign-box";
const REGION_PAD: &str = "handsign.pad";
const REGION_PLACE: &str = "handsign.place";
const REGION_BODY: &str = "handsign.body";
const REGION_DIGITAL_ID: &str = "handsign.digital-id";
const FILE_TAB: &str = "ribbon.tab.file";
const SAVE_COPY: &str = "ribbon.item.file.save_copy";

/// See the module documentation.
pub struct ADrawnSignatureLandsInItsBox;

impl Check for ADrawnSignatureLandsInItsBox {
    fn name(&self) -> &'static str {
        "a_drawn_signature_lands_in_its_box"
    }

    fn defect(&self) -> &'static str {
        "a click on an empty signature box offers no way to sign by hand, or the drawn \
         signature does not reach the box, or its tag stays, or Ctrl+Z does not take it back, \
         or the saved file carries a digital signature"
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

fn at(x: f32, y: f32) -> WindowPoint {
    WindowPoint::centre_of(LRect::new(Pt::new(x, y), Pt::new(x, y)))
}

/// Pixels of the signature ink: blue well above red and green, and dark.
/// The fixture's border is grey and its captions black, so neither counts.
pub(super) fn ink_pixels(image: &Image, session: &Session, area: LRect) -> Result<usize> {
    let px = session.frame()?.logical_to_capture_pixels(area);
    Ok(image
        .pixels_in(px)
        .filter(|c| c.b > c.r.saturating_add(35) && c.b > c.g.saturating_add(25) && c.r < 110)
        .count())
}

/// The window's own frame, kept as an artefact and loaded for counting.
pub(super) fn shoot(
    ctx: &CheckContext,
    pointer: &ScriptedPointer,
    session: &Session,
    report: &mut CheckReport,
    file: &str,
) -> Result<Image> {
    let path = ctx.out(file);
    pointer.screenshot(session, &path)?;
    let image = Image::load_png(&path)?;
    report.artifact(path);
    Ok(image)
}

/// The box and the band above it a signature may rise into: two box heights.
pub(super) fn signing_area(r: LRect) -> LRect {
    LRect::new(Pt::new(r.min.x, r.min.y - r.height()), r.max)
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let source = repo_fixture(FIXTURE, "Run fixtures/esign-three-boxes.PROVENANCE.py.")?;
    let doc = ctx.out("hand-sign-source.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let saved = ctx.out("hand-sign-saved.pdf");
    let _ = std::fs::remove_file(&saved);

    let mut spec = LaunchSpec::new(&exe, ctx.out("hand-sign.trace.txt"));
    spec.pdf = Some(doc.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env.push((
        "PDFCER_DIAG_SAVE_PATH".to_owned(),
        saved.display().to_string(),
    ));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("hand-sign.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);

    let region = |name: &str| -> Result<(LRect, Option<String>)> {
        let trace = session.trace()?;
        declared_in(&trace, ui_rect, name).ok_or_else(|| {
            let prefix = name.rsplit_once('.').map_or(name, |(head, _)| head);
            Error::new(format!(
                "no `{name}` region. Declared under `{prefix}`: {}.",
                list(&declared_names(&trace, ui_rect, prefix))
            ))
        })
    };

    // --- A: the first box is tagged and carries no ink -------------------
    let (first, _) = region(REGION_BOX)?;
    let before = shoot(ctx, &pointer, &session, report, "hand-sign-before.png")?;
    let ink_before = ink_pixels(&before, &session, signing_area(first))?;

    // --- B: a click opens the Sign here window ----------------------------
    pointer.click(&session, WindowPoint::centre_of(first))?;
    session.settle(30);
    if session.trace()?.events("hand-sign-opened").last().is_none() {
        pointer.gone(&session)?;
        return Ok(Some(
            "a click on the tagged box opened no Sign here window (no `hand-sign-opened`)."
                .to_owned(),
        ));
    }

    // --- C: draw two strokes and place ------------------------------------
    let (pad, vp) = region(REGION_PAD)?;
    let (x0, y0, w, h) = (pad.min.x, pad.min.y, pad.width(), pad.height());
    pointer.drag_in(
        &session,
        vp.as_deref(),
        at(x0 + 0.15 * w, y0 + 0.75 * h),
        at(x0 + 0.45 * w, y0 + 0.25 * h),
        12,
        "l",
    )?;
    session.settle(5);
    pointer.drag_in(
        &session,
        vp.as_deref(),
        at(x0 + 0.40 * w, y0 + 0.60 * h),
        at(x0 + 0.85 * w, y0 + 0.55 * h),
        12,
        "l",
    )?;
    session.settle(10);
    // An immediate dialog viewport cannot be photographed (eframe answers a
    // screenshot only for root and deferred viewports), so its layout is
    // read from the regions it declared: every control inside the body.
    let (body, _) = region(REGION_BODY)?;
    let mut outside = Vec::new();
    for name in [REGION_PAD, REGION_PLACE, REGION_DIGITAL_ID] {
        if let Some((r, _)) = declared_in(&session.trace()?, ui_rect, name)
            && !body.contains_rect(r)
        {
            outside.push(name);
        }
    }
    let (place, place_vp) = region(REGION_PLACE)?;
    pointer.click_in(&session, place_vp.as_deref(), WindowPoint::centre_of(place))?;
    session.settle(40);

    let trace = session.trace()?;
    let placed = trace.events("hand-sign-placed").last().map(|l| {
        (
            l.get_usize("strokes"),
            l.get_usize("tagged"),
            l.get_usize("undo_depth"),
        )
    });
    let refused = trace
        .events("hand-sign-refused")
        .last()
        .and_then(|l| l.get("reason").map(str::to_owned));
    let next = declared(&trace, ui_rect, REGION_BOX);
    let after = shoot(ctx, &pointer, &session, report, "hand-sign-after.png")?;
    let ink_after = ink_pixels(&after, &session, signing_area(first))?;

    // --- D: Ctrl+Z takes it back; Ctrl+Shift+Z puts it back ---------------------
    pointer.gone(&session)?;
    pointer.key(&session, None, "Z", Some("ctrl"))?;
    session.settle(30);
    let undone_box = declared(&session.trace()?, ui_rect, REGION_BOX);
    let undone = shoot(ctx, &pointer, &session, report, "hand-sign-undone.png")?;
    let ink_undone = ink_pixels(&undone, &session, signing_area(first))?;
    pointer.key(&session, None, "Z", Some("ctrl+shift"))?;
    session.settle(30);
    let redone_box = declared(&session.trace()?, ui_rect, REGION_BOX);

    // --- E: save a copy ---------------------------------------------------
    let (tab, tab_vp) = region(FILE_TAB)?;
    pointer.click_in(&session, tab_vp.as_deref(), WindowPoint::centre_of(tab))?;
    session.settle(15);
    let (item, item_vp) = region(SAVE_COPY)?;
    pointer.click_in(&session, item_vp.as_deref(), WindowPoint::centre_of(item))?;
    for _ in 0..40 {
        if session.trace()?.events("save-copy").last().is_some() {
            break;
        }
        session.settle(10);
    }
    pointer.gone(&session)?;
    drop(session);

    let source_bytes = std::fs::read(&source).unwrap_or_default();
    let bytes = std::fs::read(&saved).unwrap_or_default();
    let ranges = count(&bytes, b"/ByteRange");
    let tags = count(&bytes, b"/pdfc_HandSig");
    let sig_fields = count(&bytes, b"/FT /Sig");
    report.note(format!(
        "box {first:?}; ink before={ink_before} after={ink_after} undone={ink_undone}; placed \
         (strokes, tagged, undo_depth)={placed:?}; refused={refused:?}; tag after placing at \
         {next:?}, after undo {undone_box:?}, after redo {redone_box:?}; saved {} bytes (source \
         {}), /ByteRange x{ranges}, /FT /Sig x{sig_fields}, /pdfc_HandSig x{tags}",
        bytes.len(),
        source_bytes.len()
    ));

    let mut findings = Vec::new();
    if tags == 0 {
        findings.push(
            "the saved copy holds no `/pdfc_HandSig` tag: the signature is not recorded in the              document, so the box reads unsigned after reopening."
                .to_owned(),
        );
    }
    if !outside.is_empty() {
        findings.push(format!(
            "the Sign here window draws {} outside its own body: clipped or off the window.",
            outside.join(", ")
        ));
    }
    if ink_before != 0 {
        findings.push(format!(
            "{ink_before} ink-coloured pixels in the box before signing: the pixel oracle is \
             counting something other than the signature."
        ));
    }
    match placed {
        Some((Some(strokes), Some(1), Some(_))) if strokes >= 2 => {}
        other => findings.push(format!(
            "placing traced (strokes, tagged, undo_depth)={other:?}, refused={refused:?}; two \
             strokes, tagged with the box's field name, were drawn."
        )),
    }
    if ink_after < 20 {
        findings.push(format!(
            "only {ink_after} ink pixels in or above the box after Place: the signature did not \
             land in the box it was drawn for."
        ));
    }
    match next {
        Some(r) if r.min.y > first.max.y => {}
        other => findings.push(format!(
            "after placing, the first tagged box is {other:?}; the signed box at {first:?} must \
             lose its tag and the next box down carry it."
        )),
    }
    if ink_undone != 0 {
        findings.push(format!(
            "{ink_undone} ink pixels remain after Ctrl+Z: one undo did not remove the signature."
        ));
    }
    if undone_box.is_none_or(|r| (r.min.y - first.min.y).abs() > 1.0) {
        findings.push(format!(
            "after Ctrl+Z the first tag is at {undone_box:?}; the unsigned box at {first:?} must \
             carry it again."
        ));
    }
    if redone_box.is_none_or(|r| r.min.y <= first.max.y) {
        findings.push(format!(
            "after Ctrl+Shift+Z the first tag is at {redone_box:?}; redo must sign the box again."
        ));
    }
    if bytes.len() <= source_bytes.len() {
        findings.push(format!(
            "the saved copy is {} bytes against a source of {}: nothing was written.",
            bytes.len(),
            source_bytes.len()
        ));
    }
    if ranges != 0 {
        findings.push(format!(
            "the saved copy carries {ranges} /ByteRange: a hand signature must never be written \
             as a digital signature."
        ));
    }
    if sig_fields < 3 {
        findings.push(format!(
            "the saved copy names /FT /Sig {sig_fields} times; all three signature fields must \
             survive, so a certificate can still be added later."
        ));
    }
    Ok((!findings.is_empty()).then(|| findings.join("\n")))
}

pub(super) fn count(hay: &[u8], needle: &[u8]) -> usize {
    hay.windows(needle.len()).filter(|w| *w == needle).count()
}
