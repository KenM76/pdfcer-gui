//! `a_snapshot_set_to_picture_copies_no_vectors` — with **Snapshot copies as**
//! set to *Picture only*, Copy over a laid snapshot box places a picture and
//! no SVG, metafile or PDF. Driven through the scripted pointer on a window
//! placed off the desktop; the clipboard is restored afterwards.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/snapshot_picture.md`.

use super::os_image_paste::ClipGuard;
use super::snapshot_box::{GROUPS, ITEM, TAB, laid_box, launch};
use super::snapshot_copy::{FROM, TO, clear};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::{Error, Result};
use crate::report::CheckReport;

const COPIED: &str = "clipboard-snapshot-copy"; // ui-text-exempt: a trace event name, never displayed
/// The preferences line that selects the picture, as the app writes it.
const PICTURE: &str = "snapshot_copy = picture\n"; // ui-text-exempt: a preferences line
const PNG: &str = "PNG"; // ui-text-exempt: a clipboard format name, never displayed
/// The vector formats a page copy places, as the app names them.
const VECTOR: [&str; 3] = ["image/svg+xml", "CF_ENHMETAFILE", "application/pdf"]; // ui-text-exempt: clipboard format names, never displayed

/// See the module documentation.
pub struct ASnapshotSetToPictureCopiesNoVectors;

impl Check for ASnapshotSetToPictureCopiesNoVectors {
    fn name(&self) -> &'static str {
        "a_snapshot_set_to_picture_copies_no_vectors"
    }

    fn defect(&self) -> &'static str {
        "with Snapshot copies as set to Picture only, Copy over a snapshot box still places SVG \
         and a metafile, so a paste into a program that prefers vectors gets the drawing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let mut guard = ClipGuard::take();
        let outcome = prefs(ctx, PICTURE).and_then(|()| drive(ctx, &mut report, &mut guard));
        report.note(guard.release());
        if let Err(why) = prefs(ctx, "") {
            report.note(format!("could not restore the preferences: {why}"));
        }
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Write `text` as the driven binary's preferences file.
fn prefs(ctx: &CheckContext, text: &str) -> Result<()> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let userdata = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata");
    crate::sandbox::write_prefs(&userdata, text)
        .map_err(|e| Error::new(format!("could not write the preferences: {e}")))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    guard: &mut ClipGuard,
) -> Result<Option<String>> {
    let (rig, page) = launch(ctx, report, "snapshot-picture")?;
    rig.click(TAB)?;
    rig.click_item(ITEM, GROUPS)?;
    let (w, h) = (page.width_pt, page.height_pt);
    let corners = ((FROM.0 * w, FROM.1 * h), (TO.0 * w, TO.1 * h));
    let mapping = CanvasMapping::from_trace(&rig.session.trace()?, &ctx.profile.vocab, page, 0)?;
    let a = mapping.doc_to_window(DocPoint::new(0, corners.0.0, corners.0.1))?;
    let b = mapping.doc_to_window(DocPoint::new(0, corners.1.0, corners.1.1))?;
    rig.pointer.drag(&rig.session, a, b, 12)?;
    rig.session.settle(20);
    if let Err(why) = laid_box(&rig, corners)? {
        return Ok(Some(why));
    }
    clear(guard)?;
    rig.pointer.copy(&rig.session, None)?;
    rig.session.settle(60);
    guard.adopt();
    let trace = rig.session.trace()?;
    let Some(line) = trace.last(COPIED) else {
        return Ok(Some(format!("★ Copy traced no `{COPIED}` line.")));
    };
    report.note(format!("copy: `{}`", line.raw));
    let formats: Vec<&str> = line.get("formats").unwrap_or_default().split(',').collect();
    let vectors = formats.iter().any(|f| VECTOR.contains(f));
    if line.get("vectors") != Some("picture-only") || vectors || !formats.contains(&PNG) {
        return Ok(Some(format!(
            "★★★ the copy placed `{}`, not a picture only: `vectors=picture-only` and PNG with \
             none of {VECTOR:?}.",
            line.raw
        )));
    }
    Ok(None)
}
