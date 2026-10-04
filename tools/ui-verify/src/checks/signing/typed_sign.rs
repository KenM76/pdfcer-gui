//! `checks::signing::typed_sign` — a name typed on the Sign here window's
//! Type tab lands in the clicked box as page text in an embedded handwriting
//! face, undoes in one step, and saves as no digital signature.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/signing/typed_sign.md`.

use super::hand_sign::{count, ink_pixels, shoot, signing_area};
use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_in, declared_names, list, repo_fixture,
};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "esign-three-boxes.pdf";
const REGION_BOX: &str = "form.sign-box";
const REGION_BODY: &str = "handsign.body";
const REGION_TAB_TYPE: &str = "handsign.tab-type";
const REGION_NAME: &str = "handsign.name";
const REGION_PREVIEW: &str = "handsign.preview";
const REGION_PLACE: &str = "handsign.place";
const FILE_TAB: &str = "ribbon.tab.file";
const SAVE_COPY: &str = "ribbon.item.file.save_copy";
/// A made-up name: eleven characters, a space, upper and lower case.
const NAME: &str = "Pat Example";

/// See the module documentation.
pub struct ATypedSignatureLandsInItsBox;

impl Check for ATypedSignatureLandsInItsBox {
    fn name(&self) -> &'static str {
        "a_typed_signature_lands_in_its_box"
    }

    fn defect(&self) -> &'static str {
        "the Sign here window offers no Type tab, or the typed name does not reach the box, or \
         it is not written in an embedded face, or Ctrl+Z does not take it back, or the saved \
         file carries a digital signature"
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
    let doc = ctx.out("typed-sign-source.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let saved = ctx.out("typed-sign-saved.pdf");
    let _ = std::fs::remove_file(&saved);

    let mut spec = LaunchSpec::new(&exe, ctx.out("typed-sign.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("typed-sign.pointer.txt"))?;
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
    let before = shoot(ctx, &pointer, &session, report, "typed-sign-before.png")?;
    let ink_before = ink_pixels(&before, &session, signing_area(first))?;

    // --- B: a click opens the Sign here window; choose Type ---------------
    pointer.click(&session, WindowPoint::centre_of(first))?;
    session.settle(30);
    if session.trace()?.events("hand-sign-opened").last().is_none() {
        pointer.gone(&session)?;
        return Ok(Some(
            "a click on the tagged box opened no Sign here window (no `hand-sign-opened`)."
                .to_owned(),
        ));
    }
    let Some((tab, tab_vp)) = declared_in(&session.trace()?, ui_rect, REGION_TAB_TYPE) else {
        pointer.gone(&session)?;
        return Ok(Some(
            "the Sign here window has no Type tab. It is drawn only when Segoe Script, Ink Free \
             or Segoe Print is installed and embeddable; on this machine one is expected."
                .to_owned(),
        ));
    };
    pointer.click_in(&session, tab_vp.as_deref(), WindowPoint::centre_of(tab))?;
    session.settle(20);

    // --- C: type a name and place -----------------------------------------
    let (name, name_vp) = region(REGION_NAME)?;
    pointer.click_in(&session, name_vp.as_deref(), WindowPoint::centre_of(name))?;
    session.settle(5);
    pointer.type_text(&session, name_vp.as_deref(), NAME)?;
    session.settle(20);
    // An immediate dialog viewport cannot be photographed, so its layout is
    // read from the regions it declared: every control inside the body.
    let (body, _) = region(REGION_BODY)?;
    let mut outside = Vec::new();
    for name in [REGION_TAB_TYPE, REGION_NAME, REGION_PREVIEW, REGION_PLACE] {
        match declared_in(&session.trace()?, ui_rect, name) {
            Some((r, _)) if body.contains_rect(r) => {}
            _ => outside.push(name),
        }
    }
    let (place, place_vp) = region(REGION_PLACE)?;
    pointer.click_in(&session, place_vp.as_deref(), WindowPoint::centre_of(place))?;
    session.settle(40);

    let trace = session.trace()?;
    let requested = trace
        .events("hand-sign-requested")
        .last()
        .and_then(|l| l.get("via").map(str::to_owned));
    let placed = trace.events("hand-sign-placed").last().map(|l| {
        (
            l.get("via").map(str::to_owned),
            l.get_usize("chars"),
            l.get_usize("tagged"),
            l.get("face").map(str::to_owned),
        )
    });
    let refused = trace
        .events("hand-sign-refused")
        .last()
        .and_then(|l| l.get("reason").map(str::to_owned));
    let next = declared(&trace, ui_rect, REGION_BOX);
    let after = shoot(ctx, &pointer, &session, report, "typed-sign-after.png")?;
    let ink_after = ink_pixels(&after, &session, signing_area(first))?;

    // --- D: Ctrl+Z takes it back -------------------------------------------
    pointer.gone(&session)?;
    pointer.key(&session, None, "Z", Some("ctrl"))?;
    session.settle(30);
    let undone_box = declared(&session.trace()?, ui_rect, REGION_BOX);
    let undone = shoot(ctx, &pointer, &session, report, "typed-sign-undone.png")?;
    let ink_undone = ink_pixels(&undone, &session, signing_area(first))?;
    pointer.key(&session, None, "Z", Some("ctrl+shift"))?;
    session.settle(30);

    // --- E: save a copy ---------------------------------------------------
    let (file_tab, file_vp) = region(FILE_TAB)?;
    pointer.click_in(
        &session,
        file_vp.as_deref(),
        WindowPoint::centre_of(file_tab),
    )?;
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

    let bytes = std::fs::read(&saved).unwrap_or_default();
    let ranges = count(&bytes, b"/ByteRange");
    let tags = count(&bytes, b"/pdfc_HandSig");
    let font_files = count(&bytes, b"/FontFile2");
    report.note(format!(
        "box {first:?}; ink before={ink_before} after={ink_after} undone={ink_undone}; requested \
         via={requested:?}; placed (via, chars, tagged, face)={placed:?}; refused={refused:?}; \
         tag after placing at {next:?}, after undo {undone_box:?}; saved {} bytes, /FontFile2 \
         x{font_files}, /ByteRange x{ranges}, /pdfc_HandSig x{tags}",
        bytes.len()
    ));

    let mut findings = Vec::new();
    if tags == 0 {
        findings.push(
            "the saved copy holds no `/pdfc_HandSig` tag: the signature is not recorded in the document, so the box reads unsigned after reopening."
                .to_owned(),
        );
    }
    if !outside.is_empty() {
        findings.push(format!(
            "the Type tab draws {} outside the window's body, or not at all.",
            outside.join(", ")
        ));
    }
    if ink_before != 0 {
        findings.push(format!(
            "{ink_before} ink-coloured pixels in the box before signing: the pixel oracle is \
             counting something other than the signature."
        ));
    }
    let chars = NAME.chars().count();
    match &placed {
        Some((Some(via), Some(n), Some(1), Some(_))) if via == "type" && *n == chars => {}
        other => findings.push(format!(
            "placing traced (via, chars, tagged, face)={other:?}, refused={refused:?}; a typed \
             name of {chars} characters, tagged with the box's field name, was expected."
        )),
    }
    if ink_after < 20 {
        findings.push(format!(
            "only {ink_after} ink pixels in or above the box after Place: the typed name did \
             not land in the box."
        ));
    }
    match next {
        Some(r) if r.min.y > first.max.y => {}
        other => findings.push(format!(
            "after placing, the first tagged box is {other:?}; the signed box at {first:?} must \
             lose its tag."
        )),
    }
    if ink_undone != 0 {
        findings.push(format!(
            "{ink_undone} ink pixels remain after Ctrl+Z: one undo did not remove the typed \
             signature."
        ));
    }
    if undone_box.is_none_or(|r| (r.min.y - first.min.y).abs() > 1.0) {
        findings.push(format!(
            "after Ctrl+Z the first tag is at {undone_box:?}; the unsigned box at {first:?} must \
             carry it again."
        ));
    }
    if font_files == 0 {
        findings.push(
            "the saved copy embeds no TrueType program (/FontFile2): the handwriting face was \
             not embedded, so another reader would show a substitute."
                .to_owned(),
        );
    }
    if ranges != 0 {
        findings.push(format!(
            "the saved copy carries {ranges} /ByteRange: a typed signature must never be \
             written as a digital signature."
        ));
    }
    Ok((!findings.is_empty()).then(|| findings.join("\n")))
}
