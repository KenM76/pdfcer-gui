//! `the_3d_viewer_lists_the_model_tree` — the 3D viewer lists the model's
//! parts.
//!
//! Places the engine corpus's `assembly.prc` from the ribbon and opens it with
//! *View…*. The `model-view-parts` line must list the engine's tree for it
//! (`PrcFile::model_tree`): [`EXPECTED_NODES`] nodes at [`EXPECTED_DEPTHS`],
//! all unnamed, and the `model3d.parts` region must be drawn inside the
//! window. No corpus PRC names a part or stores one hidden (G135), so names
//! and visibility are not asserted.

use crate::checks::driving::declared_in;
use crate::checks::model_inferences::open_viewer;
use crate::checks::model_view_window::{launch_with_model_file, ui_rect};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// A PRC assembly: a root placing one part twice.
const MODEL: &str = "D:/Dev/pdfcer/fixtures/synthetic/prc/assembly.prc";

/// The part list's region (`dialogs::model3d::parts::REGION_PARTS`).
const PARTS_REGION: &str = "model3d.parts";

/// Nodes the engine lists for [`MODEL`].
const EXPECTED_NODES: usize = 3;

/// Their depths in list order, as the trace joins them: the root, then its
/// two children.
const EXPECTED_DEPTHS: &str = "0|1|1";

/// Their names, as the trace joins them: none is named.
const EXPECTED_NAMES: &str = "-|-|-";

/// See the module documentation.
pub struct TheViewerListsTheModelTree;

impl Check for TheViewerListsTheModelTree {
    fn name(&self) -> &'static str {
        "the_3d_viewer_lists_the_model_tree"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer shows no list of the model's parts, or lists other parts than the file holds"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch_with_model_file(ctx, &mut report, "assembly", MODEL)
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer));
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
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
    let trace = session.trace()?;
    let line = trace.events("model-view-parts").last();
    report.note(format!("parts: {:?}", line.map(|l| l.raw.clone())));
    let Some(line) = line else {
        return Ok(Some("View… traced no `model-view-parts` line.".to_owned()));
    };
    let nodes = line.get("nodes").and_then(|n| n.parse::<usize>().ok());
    let depths = line.get("depths");
    let names = line.get("names");
    if nodes != Some(EXPECTED_NODES)
        || depths != Some(EXPECTED_DEPTHS)
        || names != Some(EXPECTED_NAMES)
    {
        return Ok(Some(format!(
            "the viewer lists nodes={nodes:?} depths={depths:?} names={names:?}; the engine's \
             tree for this model is {EXPECTED_NODES} nodes at depths `{EXPECTED_DEPTHS}` named \
             `{EXPECTED_NAMES}`."
        )));
    }
    if declared_in(&trace, ui_rect(ctx)?, PARTS_REGION).is_none() {
        return Ok(Some(format!(
            "no `{PARTS_REGION}` region is drawn inside the viewer."
        )));
    }
    Ok(None)
}
