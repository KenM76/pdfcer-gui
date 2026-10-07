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
//!
//! `TheViewerDrawsATexture` does the same with `textured.prc`, a square whose
//! texture picture is red, green, blue and white: `textured=` must be at least
//! 1, and the last `model-view-rendered` line must count at least three hues,
//! which a square drawn in one base colour cannot produce.

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
            |(session, pointer)| {
                drive(
                    ctx,
                    &mut report,
                    &session,
                    &pointer,
                    "overridden",
                    "its entity references colour its parts",
                )
            },
        );
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// A square styled by a red, green, blue and white texture picture.
const TEXTURED_MODEL: &str = "D:/Dev/pdfcer/fixtures/synthetic/prc/textured.prc";

/// Hues the textured square's picture shows; its base colour is one.
const TEXTURE_HUES: usize = 3;

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
        let driven = launch_with_model_file(ctx, &mut report, "best_fit", BEST_FIT_MODEL).and_then(
            |(session, pointer)| {
                drive(
                    ctx,
                    &mut report,
                    &session,
                    &pointer,
                    "best-fit",
                    "only the best-fit search rebuilds its mesh",
                )
            },
        );
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// See the module documentation.
pub struct TheViewerDrawsATexture;

impl Check for TheViewerDrawsATexture {
    fn name(&self) -> &'static str {
        "the_3d_viewer_draws_a_texture"
    }

    fn defect(&self) -> &'static str {
        "a 3D part styled by a texture picture is drawn in one flat colour, or the viewer does \
         not say it drew a texture"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch_with_model_file(ctx, &mut report, "textured", TEXTURED_MODEL).and_then(
            |(session, pointer)| {
                let opened = drive(
                    ctx,
                    &mut report,
                    &session,
                    &pointer,
                    "textured",
                    "its one part is styled by a texture",
                )?;
                if opened.is_some() {
                    return Ok(opened);
                }
                let trace = session.trace()?;
                let line = trace.events("model-view-rendered").last();
                report.note(format!("rendered: {:?}", line.map(|l| l.raw.clone())));
                Ok(
                    match line.and_then(|l| l.get("hues")?.parse::<usize>().ok()) {
                        None => {
                            Some("the viewer traced no `model-view-rendered … hues=`.".to_owned())
                        }
                        Some(h) if h < TEXTURE_HUES => Some(format!(
                            "the picture shows {h} hue(s); the texture has {TEXTURE_HUES}, so \
                             the square was drawn in a flat colour."
                        )),
                        Some(_) => None,
                    },
                )
            },
        );
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Places the model, opens *View…*, and requires the `model-view-opened`
/// line's `field` count to be at least 1; `why` says why this model needs it.
fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    field: &str,
    why: &str,
) -> Result<Option<String>> {
    if let Some(failure) = open_viewer(ctx, session, pointer)? {
        return Ok(Some(failure));
    }
    let trace = session.trace()?;
    let line = trace.events("model-view-opened").last();
    report.note(format!("viewer: {:?}", line.map(|l| l.raw.clone())));
    match line.and_then(|l| l.get(field)?.parse::<usize>().ok()) {
        None => Ok(Some(format!(
            "View… traced no `model-view-opened … {field}=`."
        ))),
        Some(0) => Ok(Some(format!(
            "the viewer traces `{field}=0`; this model needs at least 1: {why}."
        ))),
        Some(_) => Ok(None),
    }
}

/// Places the launched model from the ribbon and opens it with *View…*;
/// `Some` says which step found nothing to press.
pub(crate) fn open_viewer(
    ctx: &CheckContext,
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
    Ok(None)
}
