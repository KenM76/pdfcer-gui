//! `copy_as_vector_without_the_mouse` — Edit ▸ Clipboard ▸ Copy as vector,
//! driven through the scripted pointer on a window off the desktop, with the
//! payload sent to a folder so the operator's clipboard is untouched.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/copy_as_vector_scripted.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};

/// Off the desktop, so the check runs while the operator uses the machine.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const CAPTURE_ENV: &str = "PDFCER_DIAG_CLIPBOARD_DIR"; // ui-text-exempt: an environment variable name
const MODE: &str = "edit"; // ui-text-exempt: a ribbon mode id
const TAB: &str = "edit"; // ui-text-exempt: a ribbon tab id
const COMMAND: &str = "ribbon.item.edit.copy_as_vector"; // ui-text-exempt: a trace region name
const PLACED: &str = "clipboard-copy-out"; // ui-text-exempt: a trace event name
const REFUSED: &str = "clipboard-copy-out-refused"; // ui-text-exempt: a trace event name
const CAPTURED: &str = "clipboard-captured"; // ui-text-exempt: a trace event name

/// The formats in placement order, as `ClipFormat::name` spells them, each
/// with the file stem the capture gives it.
const EXPECTED: [(&str, &str); 4] = [
    ("image/svg+xml", "1-image_svg_xml.bin"), // ui-text-exempt: a clipboard format name and file name
    ("CF_ENHMETAFILE", "2-CF_ENHMETAFILE.bin"), // ui-text-exempt: a clipboard format name and file name
    ("PNG", "3-PNG.bin"), // ui-text-exempt: a clipboard format name and file name
    ("CF_DIBV5", "4-CF_DIBV5.bin"), // ui-text-exempt: a clipboard format name and file name
];

/// See the module documentation.
pub struct CopyAsVectorWithoutTheMouse;

impl Check for CopyAsVectorWithoutTheMouse {
    fn name(&self) -> &'static str {
        "copy_as_vector_without_the_mouse"
    }

    fn defect(&self) -> &'static str {
        "Copy as vector places the formats out of order, places fewer than four, or places \
         bytes that are not the format they are named as"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The capture folder, emptied so an earlier run cannot pass for this one.
fn capture_dir(ctx: &CheckContext) -> Result<std::path::PathBuf> {
    let dir = ctx.out("copy-as-vector-capture");
    let _ = std::fs::remove_dir_all(&dir);
    if dir.exists() {
        return Err(Error::new(format!(
            "cannot clear {} before the run.",
            dir.display()
        )));
    }
    Ok(dir)
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    dir: &std::path::Path,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let viewport_env = ctx
        .profile
        .viewport_env
        .ok_or_else(|| Error::new("the profile has no viewport variable."))?;
    let pdf = driving::repo_fixture(
        "pure-k-square.pdf",
        "A one-page vector fixture every writer accepts, so --pdf is ignored.",
    )?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("copy_as_vector_scripted.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((CAPTURE_ENV.to_owned(), dir.display().to_string()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer =
        ScriptedPointer::attach(&mut spec, ctx.out("copy_as_vector_scripted.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(pointer.path().to_path_buf());
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched on {} as pid {}",
        pdf.display(),
        session.pid()
    ));
    session.settle(30);
    Ok((session, pointer))
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let dir = capture_dir(ctx)?;
    let (session, pointer) = launch(ctx, report, &dir)?;
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");
    driving::click_mode_segment(&session, &pointer, ui_rect, MODE)?;
    crate::checks::ocr::click_tab(&session, &pointer, ui_rect, TAB)?;
    let Some(item) = driving::declared_or_in_overflow(&session, &pointer, ui_rect, COMMAND)? else {
        return Err(Error::new(format!(
            "the Edit tab declares no `{COMMAND}`, on the band or in a collapsed group."
        )));
    };
    crate::input::Click::click_rect(&pointer, &session, item)?;
    session.settle(60);
    pointer.gone(&session)?;

    let trace = session.trace()?;
    if let Some(refused) = trace.last(REFUSED) {
        return Ok(Some(format!(
            "★ Copy as vector refused: `{}`.",
            refused.raw
        )));
    }
    let Some(placed) = trace.last(PLACED) else {
        return Ok(Some(format!(
            "★ `{COMMAND}` was clicked and no `{PLACED}` line followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("`{}`", placed.raw));
    let want = EXPECTED.map(|(name, _)| name).join(",");
    if placed.get("selection") != Some("false") || placed.get("formats") != Some(want.as_str()) {
        return Ok(Some(format!(
            "★★ the page copy should place `selection=false formats={want}`; it traced `{}`.",
            placed.raw
        )));
    }
    if trace.last(CAPTURED).and_then(|l| l.get("formats")) != Some("4") {
        return Ok(Some(format!(
            "the copy did not go to the capture folder (no `{CAPTURED} formats=4`), so it \
             may have replaced the operator's clipboard. Trace: {}.",
            session.trace_path().display()
        )));
    }
    judge(report, &dir)
}

/// The folder holds exactly the four files, each well formed for its name.
fn judge(report: &mut CheckReport, dir: &std::path::Path) -> Result<Option<String>> {
    let mut found: Vec<String> = std::fs::read_dir(dir)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", dir.display())))?
        .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().into_owned()))
        .collect();
    found.sort();
    let want: Vec<&str> = EXPECTED.iter().map(|(_, file)| *file).collect();
    if found != want {
        return Ok(Some(format!(
            "★★ the capture folder holds {found:?}; it should hold {want:?}."
        )));
    }
    for (name, file) in EXPECTED {
        let path = dir.join(file);
        let bytes = std::fs::read(&path)
            .map_err(|e| Error::new(format!("cannot read {}: {e}", path.display())))?;
        if let Some(why) = malformed(name, &bytes) {
            return Ok(Some(format!(
                "★★★ the `{name}` entry ({} bytes) {why}.",
                bytes.len()
            )));
        }
        report.artifact(path);
    }
    Ok(None)
}

/// Why `bytes` are not a well-formed `name` entry, or `None` when they are.
fn malformed(name: &str, bytes: &[u8]) -> Option<&'static str> {
    let u32_at = |at: usize| {
        bytes
            .get(at..at + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    match name {
        "image/svg+xml" => (!bytes.windows(4).any(|w| w == b"<svg")).then_some("holds no `<svg`"),
        // EMR_HEADER is record type 1, and its dSignature at offset 40 is " EMF".
        "CF_ENHMETAFILE" => (u32_at(0) != Some(1) || bytes.get(40..44) != Some(b" EMF"))
            .then_some("is not an EMF: no EMR_HEADER with the ` EMF` signature"),
        "PNG" => (!bytes.starts_with(b"\x89PNG\r\n\x1a\n")).then_some("has no PNG signature"),
        // BITMAPV5HEADER's bV5Size.
        "CF_DIBV5" => (u32_at(0) != Some(124)).then_some("does not open with a BITMAPV5HEADER"),
        _ => Some("is a format this check does not know"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_validator_accepts_its_own_shape_and_refuses_anothers() {
        let mut emf = vec![0u8; 44];
        emf[0] = 1;
        emf[40..44].copy_from_slice(b" EMF");
        let png = b"\x89PNG\r\n\x1a\nrest".to_vec();
        let mut dib = vec![0u8; 8];
        dib[0] = 124;
        assert_eq!(malformed("image/svg+xml", b"<?xml?><svg/>"), None);
        assert_eq!(malformed("CF_ENHMETAFILE", &emf), None);
        assert_eq!(malformed("PNG", &png), None);
        assert_eq!(malformed("CF_DIBV5", &dib), None);
        assert!(malformed("CF_ENHMETAFILE", &png).is_some());
        assert!(malformed("PNG", &emf).is_some());
        assert!(malformed("CF_DIBV5", &png).is_some());
        assert!(malformed("image/svg+xml", &dib).is_some());
    }
}
