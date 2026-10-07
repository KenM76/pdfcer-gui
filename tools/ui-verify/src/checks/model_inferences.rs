//! `checks::model_inferences` — **the 3D viewer says when an assembly
//! coloured the model**
//!
//! Places the engine corpus's `overridden.prc` from the ribbon, as
//! `checks::model_colours` does, opens it with *View…*, and reads the
//! `model-view-opened` line: the model's parts take their colour from its
//! assembly's entity references, so `overridden=` must be at least 1, the count
//! the viewer states through `t::view_overridden`.
//!
//! The best-fit count (`t::mesh_best_fit`) is not driven: the engine corpus
//! holds no PRC whose mesh only a best-fit search rebuilds (`compressed.prc`
//! is not rebuilt at all).

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
        let driven = launch_with_model_file(ctx, &mut report, "overridden", MODEL)
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
    match line.and_then(|l| l.get("overridden")?.parse::<usize>().ok()) {
        None => Ok(Some(
            "View… traced no `model-view-opened … overridden=`.".to_owned(),
        )),
        Some(0) => Ok(Some(
            "the viewer counts no part coloured by the assembly; this model's entity \
             references colour its parts."
                .to_owned(),
        )),
        Some(_) => Ok(None),
    }
}
