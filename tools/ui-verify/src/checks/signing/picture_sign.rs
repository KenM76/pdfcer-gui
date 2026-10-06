//! `checks::signing::picture_sign` — a picture of a signature chosen on the
//! Sign here window's Picture tab lands in the clicked box as page content
//! with its white cleared, undoes in one step, and saves tagged and as no
//! digital signature.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/signing/picture_sign.md`.

use super::hand_sign::{count, ink_pixels, shoot, signing_area};
use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_in, declared_names, list, repo_fixture,
};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "esign-three-boxes.pdf";
pub(super) const REGION_BOX: &str = "form.sign-box";
const REGION_TAB_PICTURE: &str = "handsign.tab-picture";
const REGION_CHOOSE: &str = "handsign.choose-picture";
pub(super) const REGION_PLACE: &str = "handsign.place";
const FILE_TAB: &str = "ribbon.tab.file";
const SAVE_COPY: &str = "ribbon.item.file.save_copy";
const IMAGE_PATH_ENV: &str = "PDFCER_DIAG_IMAGE_PATH"; // ui-text-exempt: an environment variable name

/// The picture: white paper with a bar in the placement ink colour across
/// its middle, three times as wide as tall.
const PICTURE_W: u32 = 120;
const PICTURE_H: u32 = 40;

/// A drive with the Sign here window open on the Picture tab and a picture
/// chosen.
pub(super) struct Opened {
    pub session: Session,
    pub pointer: ScriptedPointer,
    /// The first signature box, before signing.
    pub first: LRect,
    /// Ink pixels in its signing area before signing.
    pub ink_before: usize,
    /// The `hand-sign-picture-chosen` line's `(read, clear_white, offered)`.
    pub chosen: Option<(Option<usize>, Option<usize>, Option<usize>)>,
    /// Where `PDFCER_DIAG_SAVE_PATH` points.
    pub saved: std::path::PathBuf,
    pub ui_rect: &'static str,
}

impl Opened {
    /// The region `name` and its viewport, or why there is none.
    pub fn region(&self, name: &str) -> Result<(LRect, Option<String>)> {
        let trace = self.session.trace()?;
        declared_in(&trace, self.ui_rect, name).ok_or_else(|| {
            let prefix = name.rsplit_once('.').map_or(name, |(head, _)| head);
            Error::new(format!(
                "no `{name}` region. Declared under `{prefix}`: {}.",
                list(&declared_names(&trace, self.ui_rect, prefix))
            ))
        })
    }

    /// Click the region `name` in its own viewport.
    pub fn press(&self, name: &str, frames: u32) -> Result<()> {
        let (r, vp) = self.region(name)?;
        self.pointer
            .click_in(&self.session, vp.as_deref(), WindowPoint::centre_of(r))?;
        self.session.settle(frames);
        Ok(())
    }
}

/// The picture this check chooses, encoded beside the run's other output.
fn picture(ctx: &CheckContext) -> Result<std::path::PathBuf> {
    let mut rgb = Vec::with_capacity((PICTURE_W * PICTURE_H * 3) as usize);
    for y in 0..PICTURE_H {
        for x in 0..PICTURE_W {
            let ink = (14..26).contains(&y) && (4..PICTURE_W - 4).contains(&x);
            rgb.extend_from_slice(if ink { &[13, 26, 89] } else { &[255, 255, 255] });
        }
    }
    let png = crate::png::encode_rgb(PICTURE_W, PICTURE_H, &rgb)
        .ok_or_else(|| Error::new("encoding the signature picture failed."))?;
    let path = ctx.out("picture-sign.signature.png");
    std::fs::write(&path, png)
        .map_err(|e| Error::new(format!("writing {}: {e}", path.display())))?;
    Ok(path)
}

/// Launch on a copy of the fixture, click the first box, open the Picture
/// tab and choose the picture. `Ok(Err(..))` is a finding.
pub(super) fn open(
    ctx: &CheckContext,
    report: &mut CheckReport,
    stem: &str,
) -> Result<std::result::Result<Opened, String>> {
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
    let doc = ctx.out(&format!("{stem}-source.pdf"));
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let saved = ctx.out(&format!("{stem}-saved.pdf"));
    let _ = std::fs::remove_file(&saved);
    let image = picture(ctx)?;
    report.artifact(image.clone());

    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{stem}.trace.txt")));
    spec.pdf = Some(doc);
    let image = image.display().to_string();
    let saved_s = saved.display().to_string();
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_SAVE_PATH", saved_s.as_str()),
        (IMAGE_PATH_ENV, image.as_str()),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{stem}.pointer.txt")))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let mut opened = Opened {
        session,
        pointer,
        first: LRect::new(Pt::new(0.0, 0.0), Pt::new(0.0, 0.0)),
        ink_before: 0,
        chosen: None,
        saved,
        ui_rect,
    };

    let (first, _) = opened.region(REGION_BOX)?;
    let before = shoot(
        ctx,
        &opened.pointer,
        &opened.session,
        report,
        &format!("{stem}-before.png"),
    )?;
    opened.first = first;
    opened.ink_before = ink_pixels(&before, &opened.session, signing_area(first))?;

    opened
        .pointer
        .click(&opened.session, WindowPoint::centre_of(first))?;
    opened.session.settle(30);
    if opened
        .session
        .trace()?
        .events("hand-sign-opened")
        .last()
        .is_none()
    {
        opened.pointer.gone(&opened.session)?;
        return Ok(Err(
            "a click on the tagged box opened no Sign here window (no `hand-sign-opened`)."
                .to_owned(),
        ));
    }
    if declared(&opened.session.trace()?, ui_rect, REGION_TAB_PICTURE).is_none() {
        opened.pointer.gone(&opened.session)?;
        return Ok(Err(
            "the Sign here window has no Picture tab: a picture of a signature cannot be used."
                .to_owned(),
        ));
    }
    opened.press(REGION_TAB_PICTURE, 20)?;
    opened.press(REGION_CHOOSE, 40)?;
    opened.chosen = opened
        .session
        .trace()?
        .events("hand-sign-picture-chosen")
        .last()
        .map(|l| {
            (
                l.get_usize("read"),
                l.get_usize("clear_white"),
                l.get_usize("offered"),
            )
        });
    Ok(Ok(opened))
}

/// Ctrl+Z, count the box's ink, Ctrl+Shift+Z, and save a copy. Returns the
/// ink left after undo and the saved bytes.
pub(super) fn undo_and_save(
    ctx: &CheckContext,
    report: &mut CheckReport,
    opened: &Opened,
    stem: &str,
) -> Result<(usize, Vec<u8>)> {
    let (session, pointer) = (&opened.session, &opened.pointer);
    pointer.gone(session)?;
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(30);
    let undone = shoot(ctx, pointer, session, report, &format!("{stem}-undone.png"))?;
    let ink_undone = ink_pixels(&undone, session, signing_area(opened.first))?;
    pointer.key(session, None, "Z", Some("ctrl+shift"))?;
    session.settle(30);
    opened.press(FILE_TAB, 15)?;
    let (item, item_vp) = opened.region(SAVE_COPY)?;
    pointer.click_in(session, item_vp.as_deref(), WindowPoint::centre_of(item))?;
    for _ in 0..40 {
        if session.trace()?.events("save-copy").last().is_some() {
            break;
        }
        session.settle(10);
    }
    pointer.gone(session)?;
    Ok((ink_undone, std::fs::read(&opened.saved).unwrap_or_default()))
}

/// See the module documentation.
pub struct APictureSignatureLandsInItsBox;

impl Check for APictureSignatureLandsInItsBox {
    fn name(&self) -> &'static str {
        "a_picture_signature_lands_in_its_box"
    }

    fn defect(&self) -> &'static str {
        "the Sign here window offers no Picture tab, or the chosen picture does not reach the \
         box, or its white paper is not cleared, or Ctrl+Z does not take it back, or the saved \
         file carries no hand-signature tag or a digital signature"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let opened = match open(ctx, report, "picture-sign")? {
        Ok(opened) => opened,
        Err(finding) => return Ok(Some(finding)),
    };
    opened.press(REGION_PLACE, 40)?;
    let trace = opened.session.trace()?;
    let placed = trace.events("hand-sign-placed").last().map(|l| {
        (
            l.get("via").map(str::to_owned),
            l.get_usize("tagged"),
            l.get_usize("clear_white"),
        )
    });
    let refused = trace
        .events("hand-sign-refused")
        .last()
        .and_then(|l| l.get("reason").map(str::to_owned));
    let after = shoot(
        ctx,
        &opened.pointer,
        &opened.session,
        report,
        "picture-sign-after.png",
    )?;
    let ink_after = ink_pixels(&after, &opened.session, signing_area(opened.first))?;
    let (ink_undone, bytes) = undo_and_save(ctx, report, &opened, "picture-sign")?;
    let (first, ink_before, chosen) = (opened.first, opened.ink_before, opened.chosen);
    drop(opened);

    let tags = count(&bytes, b"/pdfc_HandSig");
    let ranges = count(&bytes, b"/ByteRange");
    let keys = count(&bytes, b"/Mask");
    report.note(format!(
        "box {first:?}; chosen (read, clear_white, offered)={chosen:?}; placed (via, tagged, \
         clear_white)={placed:?}; refused={refused:?}; ink before={ink_before} \
         after={ink_after} undone={ink_undone}; saved {} bytes, /pdfc_HandSig x{tags}, /Mask \
         x{keys}, /ByteRange x{ranges}",
        bytes.len()
    ));

    let mut findings = Vec::new();
    if chosen != Some((Some(1), Some(1), Some(1))) {
        findings.push(format!(
            "choosing the picture traced (read, clear_white, offered)={chosen:?}; a picture that \
             read, with white offered and cleared by default, was expected."
        ));
    }
    if ink_before != 0 {
        findings.push(format!(
            "{ink_before} ink-coloured pixels in the box before signing: the pixel oracle is \
             counting something other than the signature."
        ));
    }
    match &placed {
        Some((Some(via), Some(1), Some(1))) if via == "picture" => {}
        other => findings.push(format!(
            "placing traced (via, tagged, clear_white)={other:?}, refused={refused:?}; a picture \
             placed with white cleared and tagged with the box's field name was expected."
        )),
    }
    if ink_after < 20 {
        findings.push(format!(
            "only {ink_after} ink pixels in or above the box after Place: the picture did not \
             land in the box."
        ));
    }
    if ink_undone != 0 {
        findings.push(format!(
            "{ink_undone} ink pixels remain after Ctrl+Z: one undo did not remove the picture."
        ));
    }
    if tags == 0 {
        findings.push(
            "the saved copy holds no `/pdfc_HandSig` tag: the box reads unsigned after reopening."
                .to_owned(),
        );
    }
    if keys == 0 {
        findings.push(
            "the saved copy holds no colour-key `/Mask`: the picture's white paper covers the \
             form under it."
                .to_owned(),
        );
    }
    if ranges != 0 {
        findings.push(format!(
            "the saved copy carries {ranges} /ByteRange: a picture signature must never be \
             written as a digital signature."
        ));
    }
    Ok((!findings.is_empty()).then(|| findings.join("\n")))
}
