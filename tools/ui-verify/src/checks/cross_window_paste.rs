//! `a_selection_copied_in_one_window_pastes_as_objects_in_another` — two
//! pdfcer-gui windows run at once; a selection copied in the first is pasted
//! into the second as page objects, not as a picture. Both windows are placed
//! off the desktop and driven by the scripted pointer, and the clipboard goes
//! to a capture folder, so the operator's own clipboard is never touched.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/cross_window_paste.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::security_notes::await_line;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};

const OFFSCREEN: &str = "-4200,-4200,1400,900";
pub(super) const CAPTURE_ENV: &str = "PDFCER_DIAG_CLIPBOARD_DIR"; // ui-text-exempt: an environment variable name
const MODE: &str = "edit"; // ui-text-exempt: a ribbon mode id
const TAB: &str = "edit"; // ui-text-exempt: a ribbon tab id
const SELECT_ALL: &str = "ribbon.item.edit.select_all"; // ui-text-exempt: a trace region name
const COPY: &str = "ribbon.item.edit.copy"; // ui-text-exempt: a trace region name
const PASTE: &str = "ribbon.item.edit.paste"; // ui-text-exempt: a trace region name
const COPIED: &str = "clipboard-copy"; // ui-text-exempt: a trace event name
const PUBLISHED: &str = "clipboard-objectclip"; // ui-text-exempt: a trace event name
const ADOPTED: &str = "clip-adopted"; // ui-text-exempt: a trace event name
const PASTED: &str = "clipboard-paste"; // ui-text-exempt: a trace event name
const APPLIED: &str = "paste-objects-applied"; // ui-text-exempt: a trace event name
/// The capture file the copy writes for the private format, by suffix.
const CLIP_FILE: &str = "-pdfcer_gui_ObjectClip.bin"; // ui-text-exempt: a capture file name

/// See the module documentation.
pub struct ASelectionCopiedInOneWindowPastesInAnother;

impl Check for ASelectionCopiedInOneWindowPastesInAnother {
    fn name(&self) -> &'static str {
        "a_selection_copied_in_one_window_pastes_as_objects_in_another"
    }

    fn defect(&self) -> &'static str {
        "A selection copied in one pdfcer-gui window does not paste into another, or pastes as \
         a picture rather than as the objects that were copied"
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

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let dir = ctx.out("cross-window-capture");
    let _ = std::fs::remove_dir_all(&dir);
    if dir.exists() {
        return Err(Error::new(format!("cannot clear {}.", dir.display())));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");

    let (source, source_pointer) = launch(ctx, report, &dir, "pure-k-square.pdf", "source")?;
    let (target, target_pointer) = launch(ctx, report, &dir, "four-pages.pdf", "target")?;

    to_edit_tab(&source, &source_pointer, ui_rect)?;
    click(&source, &source_pointer, ui_rect, SELECT_ALL)?;
    click(&source, &source_pointer, ui_rect, COPY)?;
    let Some(copied) = await_line(&source, COPIED)? else {
        return Ok(Some(format!(
            "the first window was told to copy its selection and traced no `{COPIED}` line. \
             Trace: {}.",
            source.trace_path().display()
        )));
    };
    report.note(format!("first window: `{}`", copied.raw));
    let Some(published) = source.trace()?.last(PUBLISHED).cloned() else {
        return Ok(Some(format!(
            "the copy traced `{}` and placed no clip for another window: no `{PUBLISHED}` line.",
            copied.raw
        )));
    };
    let on_disk = clip_file_len(&dir);
    if published.get("bytes").and_then(|b| b.parse::<u64>().ok()) != on_disk {
        return Ok(Some(format!(
            "the copy traced `{}` and the capture folder holds {on_disk:?} bytes under \
             `*{CLIP_FILE}`.",
            published.raw
        )));
    }

    to_edit_tab(&target, &target_pointer, ui_rect)?;
    click(&target, &target_pointer, ui_rect, PASTE)?;
    let judged = judge(report, &target, &copied);
    source_pointer.gone(&source)?;
    target_pointer.gone(&target)?;
    judged
}

/// The second window adopted the first's clip, pasted it as a selection of
/// the same number of objects, and the engine wrote them.
fn judge(
    report: &mut CheckReport,
    target: &Session,
    copied: &crate::trace::TraceLine,
) -> Result<Option<String>> {
    let Some(adopted) = await_line(target, ADOPTED)? else {
        return Ok(Some(format!(
            "the second window pasted without adopting the first window's clip: no `{ADOPTED}` \
             line. Trace: {}.",
            target.trace_path().display()
        )));
    };
    report.note(format!("second window: `{}`", adopted.raw));
    let Some(applied) = await_line(target, APPLIED)? else {
        return Ok(Some(format!(
            "the clip was adopted and no `{APPLIED}` line followed. Trace: {}.",
            target.trace_path().display()
        )));
    };
    let trace = target.trace()?;
    let pasted = trace
        .events(PASTED)
        .filter(|l| l.get("kind") == Some("selection"))
        .last()
        .map(|l| l.raw.clone());
    report.note(format!("paste: {pasted:?}; applied: `{}`", applied.raw));
    let objects = copied.get("objects");
    if adopted.get("objects") != objects || applied.get("pasted") != objects {
        return Ok(Some(format!(
            "the first window copied `{}`; the second adopted `{}` and wrote `{}`. The object \
             counts must agree, and a picture paste writes none.",
            copied.raw, adopted.raw, applied.raw
        )));
    }
    Ok(None)
}

/// The size of the private-format capture file, when there is one.
fn clip_file_len(dir: &std::path::Path) -> Option<u64> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(std::result::Result::ok)
        .find(|e| e.file_name().to_string_lossy().ends_with(CLIP_FILE))
        .and_then(|e| e.metadata().ok())
        .map(|m| m.len())
}

pub(super) fn to_edit_tab(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
) -> Result<()> {
    driving::click_mode_segment(session, pointer, ui_rect, MODE)?;
    crate::checks::ocr::click_tab(session, pointer, ui_rect, TAB)
}

pub(super) fn click(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    name: &str,
) -> Result<()> {
    let Some(item) = driving::declared_or_in_overflow(session, pointer, ui_rect, name)? else {
        return Err(Error::new(format!(
            "the Edit tab declares no `{name}`, on the band or in a collapsed group."
        )));
    };
    crate::input::Click::click_rect(pointer, session, item)?;
    session.settle(30);
    Ok(())
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    dir: &std::path::Path,
    fixture: &str,
    role: &str,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let viewport_env = ctx
        .profile
        .viewport_env
        .ok_or_else(|| Error::new("the profile has no viewport variable."))?;
    let source = driving::repo_fixture(fixture, "It is checked in.")?;
    let doc = ctx.out(&format!("cross-window-{role}.pdf"));
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {fixture}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("cross-window-{role}.trace.txt")));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.env
        .push((CAPTURE_ENV.to_owned(), dir.display().to_string()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(
        &mut spec,
        ctx.out(&format!("cross-window-{role}.pointer.txt")),
    )?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!("{role} window: pid {}", session.pid()));
    session.settle(40);
    Ok((session, pointer))
}
