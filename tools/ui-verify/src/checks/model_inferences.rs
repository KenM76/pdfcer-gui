//! `checks::model_inferences` — **the 3D viewer says when an assembly
//! coloured the model**
//!
//! Places the engine corpus's `overridden.prc` from the ribbon, as
//! `checks::model_colours` does, opens it with *View…*, and reads the
//! `model-view-opened` line: the model's parts take their colour from its
//! assembly's entity references, so `overridden=` must be at least 1, the count
//! the viewer states through `t::view_overridden`.
//!
//! `TheViewerSaysAMeshWasBestFit` does the same with `best_fit.prc`, one
//! compressed mesh only the best-fit search rebuilds: `best-fit=` must be at
//! least 1, the count the viewer states through `t::mesh_best_fit`.

use crate::checks::driving::declared_in;
use crate::checks::model_view_window::{
    VIEW_REGION, launch_with_model_file, press_insert, ui_rect,
};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// A PRC assembly whose entity references recolour its parts.
const MODEL: &str = "D:/Dev/pdfcer/fixtures/synthetic/prc/overridden.prc";

/// A PRC whose one compressed mesh only the best-fit search rebuilds.
const BEST_FIT_MODEL: &str = "D:/Dev/pdfcer/fixtures/synthetic/prc/best_fit.prc";

/// See the module documentation.
pub struct TheViewerSaysTheAssemblyColouredIt;

impl Check for TheViewerSaysTheAssemblyColouredIt {
    fn name(&self) -> &'static str {
        "the_3d_viewer_says_the_assembly_coloured_it"
    }

    fn defect(&self) -> &'static str {
        "a 3D model whose parts take their colour from its assembly is drawn with no word that \
         the colours came from the assembly"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch_with_model_file(ctx, &mut report, "overridden", MODEL).and_then(
            |(session, pointer)| drive(ctx, &mut report, &session, &pointer, "overridden"),
        );
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// See the module documentation.
pub struct TheViewerSaysAMeshWasBestFit;

impl Check for TheViewerSaysAMeshWasBestFit {
    fn name(&self) -> &'static str {
        "the_3d_viewer_says_a_mesh_was_best_fit"
    }

    fn defect(&self) -> &'static str {
        "a 3D mesh only a best-fit search could rebuild, so another shape could match the file, \
         is drawn with no word that it was a best fit"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch_with_model_file(ctx, &mut report, "best_fit", BEST_FIT_MODEL)
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer, "best-fit"));
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Places the model, opens *View…*, and requires the `model-view-opened`
/// line's `field` count to be at least 1.
fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    field: &str,
) -> Result<Option<String>> {
    pointer.gone(session)?;
    session.settle(10);
    if !press_insert(ctx, session, pointer)? {
        return Ok(Some("no 3D model button on the Edit tab.".to_owned()));
    }
    session.settle(30);
    let trace = session.trace()?;
    let Some((view, vp)) = declared_in(&trace, ui_rect(ctx)?, VIEW_REGION) else {
        return Ok(Some(format!(
            "no `{VIEW_REGION}` region beside the placed model."
        )));
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(view))?;
    session.settle(30);
    let trace = session.trace()?;
    let line = trace.events("model-view-opened").last();
    report.note(format!("viewer: {:?}", line.map(|l| l.raw.clone())));
    match line.and_then(|l| l.get(field)?.parse::<usize>().ok()) {
        None => Ok(Some(format!(
            "View… traced no `model-view-opened … {field}=`."
        ))),
        Some(0) => Ok(Some(format!(
            "the viewer traces `{field}=0`; this model needs at least 1 (`overridden`: its \
             entity references colour its parts; `best-fit`: only the best-fit search \
             rebuilds its mesh)."
        ))),
        Some(_) => Ok(None),
    }
}
