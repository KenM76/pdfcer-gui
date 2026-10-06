//! `checks::model_saved_view` — **the 3D viewer opens on the file's own
//! saved view, and *File's view* returns to it from a named one**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/model_saved_view.md`.

use std::path::PathBuf;

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// Edit mode with the Attachments panel showing.
const INVOKE: &str = "mode.edit,edit.attachments";
const VIEW_REGION: &str = "models.view";
const ISOMETRIC_REGION: &str = "model3d.view.0";
const FILE_VIEW_REGION: &str = "model3d.view.file";
const MODEL: &str = "D:/Dev/pdfcer/fixtures/synthetic/prc/assembly.prc";
/// The saved view's camera: it looks along its z column and its y column is
/// up (ISO 32000-1 Table 304 `/C2W`). Neither is a named view's axis.
const C2W: &str = "-0.8 0 -0.6 0 1 0 0.6 0 -0.8 -6 0 8";
const LOOK: [f64; 3] = [0.6, 0.0, -0.8];
const UP: [f64; 3] = [0.0, 1.0, 0.0];
/// The cosine two directions must reach to count as the same.
const SAME: f64 = 0.999;

/// A camera's look direction and up.
type Aim = ([f64; 3], [f64; 3]);

/// See the module documentation.
pub struct TheViewerOpensOnTheFilesView;

impl Check for TheViewerOpensOnTheFilesView {
    fn name(&self) -> &'static str {
        "the_3d_viewer_opens_on_the_files_view"
    }

    fn defect(&self) -> &'static str {
        "a 3D model whose file saves an opening view opens in the viewer at a view of pdfcer's \
         choosing, or the viewer offers no way back to the file's view"
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

/// One page carrying a 3D annotation whose PRC stream saves one view, the
/// stream's default.
fn pdf_bytes(prc: &[u8]) -> Vec<u8> {
    let mut stream = format!(
        "<< /Type /3D /Subtype /PRC /VA [<< /Type /3DView /XN (Corner) /MS /M /C2W [{C2W}] >>] \
         /DV 0 /Length {} >>\nstream\n",
        prc.len()
    )
    .into_bytes();
    stream.extend_from_slice(prc);
    stream.extend_from_slice(b"\nendstream");
    crate::fixture::pdf_of(&[
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 300] /Resources << >> \
          /Annots [4 0 R] >>"
            .to_vec(),
        b"<< /Type /Annot /Subtype /3D /Rect [50 50 350 250] /P 3 0 R /3DD 5 0 R >>".to_vec(),
        stream,
    ])
}

fn fixture(ctx: &CheckContext) -> Result<PathBuf> {
    let prc = std::fs::read(MODEL).map_err(|e| {
        Error::new(format!(
            "the engine corpus's fixture is unreadable at {MODEL}: {e}"
        ))
    })?;
    let path = ctx.out("model-saved-view.pdf");
    std::fs::write(&path, pdf_bytes(&prc))
        .map_err(|e| Error::new(format!("cannot write {}: {e}", path.display())))?;
    Ok(path)
}

/// The last `model-view-rendered` line's look direction and up.
fn aimed(session: &Session) -> Result<Option<Aim>> {
    let trace = session.trace()?;
    let Some(line) = trace.events("model-view-rendered").last() else {
        return Ok(None);
    };
    let vector = |key: &str| -> Option<[f64; 3]> {
        let parts: Vec<f64> = line
            .get(key)?
            .split(',')
            .filter_map(|p| p.parse().ok())
            .collect();
        <[f64; 3]>::try_from(parts).ok()
    };
    Ok(vector("dir").zip(vector("up")))
}

fn same(a: [f64; 3], b: [f64; 3]) -> bool {
    let length = |v: [f64; 3]| v.iter().map(|c| c * c).sum::<f64>().sqrt();
    let dot: f64 = (0..3).map(|i| a[i] * b[i]).sum();
    dot / (length(a) * length(b)).max(f64::MIN_POSITIVE) > SAME
}

fn on_file(aim: Option<Aim>) -> bool {
    aim.is_some_and(|(d, u)| same(d, LOOK) && same(u, UP))
}

fn launch(ctx: &CheckContext, pdf: PathBuf) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("model-saved-view.trace.txt"));
    spec.pdf = Some(pdf);
    for (key, value) in [
        ctx.profile.diag_env,
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", INVOKE),
    ] {
        spec.env.push((key.to_owned(), value.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("model-saved-view.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer))
}

/// Click `region` and let the viewer redraw; `false` when it is not declared.
fn press(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    region: &str,
) -> Result<bool> {
    let trace = session.trace()?;
    let Some((rect, vp)) = declared_in(&trace, ui_rect, region) else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(25);
    Ok(true)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let (session, pointer) = launch(ctx, fixture(ctx)?)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let failure = steps(&session, &pointer, ui_rect, report);
    let parked = pointer.gone(&session);
    match failure? {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

fn steps(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    if !press(session, pointer, ui_rect, VIEW_REGION)? {
        return Ok(Some(format!(
            "no `{VIEW_REGION}` region: the file's 3D model is not listed with a View… button."
        )));
    }
    let opened = session
        .trace()?
        .events("model-view-opened")
        .last()
        .map(|l| l.raw.clone());
    let first = aimed(session)?;
    let iso = press(session, pointer, ui_rect, ISOMETRIC_REGION)?;
    let turned = aimed(session)?;
    let back = press(session, pointer, ui_rect, FILE_VIEW_REGION)?;
    let returned = aimed(session)?;
    report.note(format!(
        "opened: {opened:?}; first (dir, up)={first:?}; Isometric pressed={iso} -> {turned:?}; \
         File's view pressed={back} -> {returned:?}"
    ));
    let mut findings = Vec::new();
    if !opened.as_deref().is_some_and(|l| l.contains("file-view=1")) {
        findings.push(format!(
            "the viewer did not open on the file's view: `model-view-opened` was {opened:?}."
        ));
    }
    if !on_file(first) {
        findings.push(format!(
            "the first picture looked along and up {first:?}; the file's view looks along \
             {LOOK:?} with {UP:?} up."
        ));
    }
    if !iso || on_file(turned) {
        findings.push(format!(
            "the Isometric button (declared: {iso}) left the camera at {turned:?}: a return to \
             the file's view cannot be told from never leaving it."
        ));
    }
    if !back {
        findings.push(format!("no `{FILE_VIEW_REGION}` (File's view) button."));
    } else if !on_file(returned) {
        findings.push(format!(
            "File's view left the camera at {returned:?}; along {LOOK:?} with {UP:?} up was \
             expected."
        ));
    }
    Ok((!findings.is_empty()).then(|| findings.join("\n")))
}
