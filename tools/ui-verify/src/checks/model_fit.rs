//! `the_3d_viewer_draws_a_mesh_in_part` — a model whose one compressed mesh
//! no fit rebuilds opens drawn in part, Mesh fit's *Continued* draws the rest
//! of its decode, and *Exact only* is refused rather than leaving an empty
//! picture. See `docs/modules/ui-verify/checks/model_fit.md`.

use crate::checks::driving::declared_in;
use crate::checks::model_inferences::open_viewer;
use crate::checks::model_view_window::{launch_with_model_file, ui_rect};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// One compressed mesh of four triangles; no fit rebuilds it and three
/// decode before its arrays stop fitting.
const MODEL: &str = "D:/Dev/pdfcer/fixtures/synthetic/prc/partial.prc";

/// The Mesh fit drop-down and two of its items.
const FIT: &str = "model3d.mesh_fit";
const CONTINUED: &str = "model3d.mesh_fit.3";
const EXACT_ONLY: &str = "model3d.mesh_fit.1";

/// See the module documentation.
pub struct TheViewerDrawsAMeshInPart;

impl Check for TheViewerDrawsAMeshInPart {
    fn name(&self) -> &'static str {
        "the_3d_viewer_draws_a_mesh_in_part"
    }

    fn defect(&self) -> &'static str {
        "a 3D model whose compressed mesh no fit rebuilds shows nothing, or is drawn in part \
         with no word of it, or Mesh fit does not change what is drawn"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch_with_model_file(ctx, &mut report, "partial", MODEL)
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer));
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The last `event` line's raw text, noted, and whether it carries every
/// one of `fields`.
fn last_has(
    report: &mut CheckReport,
    session: &Session,
    event: &str,
    fields: &[(&str, &str)],
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let line = trace.events(event).last();
    report.note(format!("{event}: {:?}", line.map(|l| l.raw.clone())));
    let Some(line) = line else {
        return Ok(Some(format!("the viewer traced no `{event}`.")));
    };
    let wrong: Vec<String> = fields
        .iter()
        .filter(|(k, v)| line.get(k) != Some(*v))
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    Ok((!wrong.is_empty()).then(|| format!("`{event}` lacks {}: {}", wrong.join(" "), line.raw)))
}

/// Open the drop-down and pick `item`; `Some` names what was not drawn.
fn pick(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    item: &str,
) -> Result<Option<String>> {
    for region in [FIT, item] {
        let trace = session.trace()?;
        let Some((rect, vp)) = declared_in(&trace, ui_rect, region) else {
            return Ok(Some(format!("no `{region}` region in the viewer.")));
        };
        pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
        session.settle(20);
    }
    Ok(None)
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if let Some(failure) = open_viewer(ctx, session, pointer)? {
        return Ok(Some(failure));
    }
    let opened = [
        ("fit", "partial"),
        ("opened-in-part", "1"),
        ("partial", "1"),
        ("partial-triangles", "3/4"),
    ];
    if let Some(failure) = last_has(report, session, "model-view-opened", &opened)? {
        return Ok(Some(format!(
            "★★★ it did not open drawn in part: {failure}"
        )));
    }
    let ui_rect = ui_rect(ctx)?;
    if let Some(failure) = pick(session, pointer, ui_rect, CONTINUED)? {
        return Ok(Some(failure));
    }
    let continued = [("fit", "continue"), ("partial-triangles", "4/4")];
    if let Some(failure) = last_has(report, session, "model-view-refit", &continued)? {
        return Ok(Some(format!("★★ Continued drew no more: {failure}")));
    }
    if let Some(failure) = pick(session, pointer, ui_rect, EXACT_ONLY)? {
        return Ok(Some(failure));
    }
    last_has(
        report,
        session,
        "model-view-refit-refused",
        &[("fit", "unique")],
    )
    .map(|f| f.map(|f| format!("★★ Exact only, which draws nothing here, was not refused: {f}")))
}
