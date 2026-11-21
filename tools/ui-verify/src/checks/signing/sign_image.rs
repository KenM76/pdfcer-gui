//! `checks::signing::sign_image` — **a visible digital signature carries the
//! picture chosen for it**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/signing/sign_image.md`.

use super::reaching::{engine_fixture, repo_fixture};
use super::{CERT, PASSPHRASE};
use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,1000";
const INVOKE: &str = "file.sign";
const REGION_BODY: &str = "sign-body";
const REGION_CHOOSE: &str = "sign-choose-certificate";
const REGION_PASSPHRASE: &str = "sign-passphrase";
const REGION_OPEN_CERT: &str = "sign-open-certificate";
const REGION_PLACE_BOX: &str = "sign-place-box";
const REGION_PICTURE: &str = "sign-picture";
const REGION_PICTURE_NAME: &str = "sign-picture-name";
const REGION_CONFIRM: &str = "sign-confirm";
const PICTURE_W: u32 = 60;
const PICTURE_H: u32 = 24;

/// See the module documentation.
pub struct AVisibleSignatureCarriesItsPicture;

impl Check for AVisibleSignatureCarriesItsPicture {
    fn name(&self) -> &'static str {
        "a_visible_signature_carries_its_picture"
    }

    fn defect(&self) -> &'static str {
        "the Sign window offers no picture for a visible signature, or the chosen picture never \
         reaches the engine, or the receipt and the written file disagree about whether it was \
         drawn"
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

/// A small two-tone PNG, written beside the run's other output.
fn picture(ctx: &CheckContext) -> Result<std::path::PathBuf> {
    let mut rgb = Vec::with_capacity((PICTURE_W * PICTURE_H * 3) as usize);
    for y in 0..PICTURE_H {
        for x in 0..PICTURE_W {
            let ink = (8..16).contains(&y) && (4..PICTURE_W - 4).contains(&x);
            rgb.extend_from_slice(if ink {
                &[120, 20, 20]
            } else {
                &[255, 255, 255]
            });
        }
    }
    let png = crate::png::encode_rgb(PICTURE_W, PICTURE_H, &rgb)
        .ok_or_else(|| Error::new("encoding the picture failed."))?;
    let path = ctx.out("sign-image.picture.png");
    std::fs::write(&path, png)
        .map_err(|e| Error::new(format!("writing {}: {e}", path.display())))?;
    Ok(path)
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    out: &std::path::Path,
) -> Result<(Session, ScriptedPointer, &'static str)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let viewport_env = ctx
        .profile
        .viewport_env
        .ok_or_else(|| Error::new("the profile has no viewport variable."))?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let doc = ctx.out("sign-image-source.pdf");
    std::fs::copy(repo_fixture("four-pages.pdf")?, &doc)
        .map_err(|e| Error::new(format!("copying four-pages.pdf: {e}")))?;
    let certificate = engine_fixture(CERT, "the signing certificate")?;
    let image = picture(ctx)?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("sign-image.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1.to_owned()),
        (SHELL_DIAG_ENV.0, SHELL_DIAG_ENV.1.to_owned()),
        (viewport_env, OFFSCREEN.to_owned()),
        ("PDFCER_DIAG_INVOKE", INVOKE.to_owned()),
        (
            "PDFCER_DIAG_CERTIFICATE_PATH",
            certificate.display().to_string(),
        ),
        ("PDFCER_DIAG_IMAGE_PATH", image.display().to_string()),
        ("PDFCER_DIAG_SAVE_PATH", out.display().to_string()),
    ] {
        spec.env.push((k.to_owned(), v));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("sign-image.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer, ui_rect))
}

/// Scroll `region` into the Sign window's body, then click it; returns the
/// viewport it lives in.
fn reach(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    region: &str,
) -> Result<Option<String>> {
    for _ in 0..12 {
        let trace = session.trace()?;
        let (rect, viewport) = declared_in(&trace, ui_rect, region).ok_or_else(|| {
            Error::new(format!(
                "no `{region}` region. Declared under `sign-`: {}.",
                list(&declared_names(&trace, ui_rect, "sign-"))
            ))
        })?;
        let inside = declared(&trace, ui_rect, REGION_BODY)
            .is_none_or(|body| rect.min.y >= body.min.y && rect.max.y <= body.max.y);
        if inside {
            pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
            session.settle(15);
            return Ok(viewport);
        }
        let body = declared(&trace, ui_rect, REGION_BODY).unwrap_or(rect);
        let dy = if rect.max.y > body.max.y {
            -120.0
        } else {
            120.0
        };
        pointer.wheel_in(
            session,
            viewport.as_deref(),
            WindowPoint::centre_of(body),
            dy,
        )?;
        session.settle(8);
    }
    Err(Error::new(format!(
        "`{region}` never scrolled into the Sign window's body."
    )))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let out = ctx.out("sign-image-signed.pdf");
    let _ = std::fs::remove_file(&out);
    let source_len = std::fs::metadata(repo_fixture("four-pages.pdf")?)
        .map_err(|e| Error::new(e.to_string()))?
        .len();
    let (session, pointer, ui_rect) = launch(ctx, report, &out)?;
    reach(&session, &pointer, ui_rect, REGION_CHOOSE)?;
    let vp = reach(&session, &pointer, ui_rect, REGION_PASSPHRASE)?;
    pointer.type_text(&session, vp.as_deref(), PASSPHRASE)?;
    session.settle(10);
    reach(&session, &pointer, ui_rect, REGION_OPEN_CERT)?;
    session.settle(30);
    reach(&session, &pointer, ui_rect, REGION_PLACE_BOX)?;
    reach(&session, &pointer, ui_rect, REGION_PICTURE)?;
    session.settle(20);
    let trace = session.trace()?;
    let chosen = trace.events("sign-picture").last().map(|l| l.raw.clone());
    report.note(format!("picked: {chosen:?}"));
    if declared(&trace, ui_rect, REGION_PICTURE_NAME).is_none() {
        return Ok(Some(format!(
            "★★★ the picture was picked ({chosen:?}) and `{REGION_PICTURE_NAME}` was never \
             declared: the window does not say which picture the box will carry."
        )));
    }
    // The footer, outside the scrolled body.
    pointer.click_region(&session, ui_rect, REGION_CONFIRM)?;
    for _ in 0..40 {
        if session.trace()?.events("sign-applied").last().is_some() {
            break;
        }
        session.settle(10);
    }
    let shot = ctx.out("sign-image-after.png");
    if pointer.screenshot(&session, &shot).is_ok() {
        report.artifact(shot);
    }
    pointer.gone(&session)?;
    verdict(&session.trace()?, &out, source_len, report)
}

/// The receipt's trace and the appended bytes agree that a picture was drawn.
fn verdict(
    trace: &crate::trace::Trace,
    out: &std::path::Path,
    source_len: u64,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let drawn = trace.events("sign-picture-drawn").last();
    report.note(format!("receipt: {:?}", drawn.map(|l| l.raw.clone())));
    let Some(drawn) = drawn else {
        return Ok(Some(
            "★★★ no `sign-picture-drawn` line: the signing never reported on the picture."
                .to_owned(),
        ));
    };
    if drawn.get("asked") != Some("1") || drawn.get("drawn") != Some("1") {
        return Ok(Some(format!(
            "★★★ `{}`: the chosen picture did not reach the engine, or was not drawn.",
            drawn.raw
        )));
    }
    let bytes = std::fs::read(out)
        .map_err(|e| Error::new(format!("the signed file {}: {e}", out.display())))?;
    let start = usize::try_from(source_len).unwrap_or(0).min(bytes.len());
    let appended = &bytes[start..];
    let images = appended.windows(6).filter(|w| w == b"/Image").count();
    report.note(format!("image XObjects in the update: {images}"));
    if images == 0 {
        return Ok(Some(
            "★★★ the receipt says the picture was drawn and the signed file's update holds no \
             image XObject."
                .to_owned(),
        ));
    }
    Ok(None)
}
