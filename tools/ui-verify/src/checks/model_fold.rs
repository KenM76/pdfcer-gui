//! `the_3d_viewers_part_tree_folds_on_its_boxes` — the boxed + and − on the
//! part tree's guide lines fold and unfold a branch.
//!
//! Places the engine corpus's `assembly.prc` (a root placing one part twice)
//! and opens it with *View…*. Presses the root's fold box twice. Oracles:
//! `model-view-parts-fold why=box folded=0` after the first press, `folded=-`
//! after the second. Design: `DESIGNS.md` § O415.

use crate::checks::driving::declared_in;
use crate::checks::model_inferences::open_viewer;
use crate::checks::model_view_window::{launch_with_model_file, ui_rect};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const MODEL: &str = "D:/Dev/pdfcer/fixtures/synthetic/prc/assembly.prc";
/// `dialogs::model3d::partstree::REGION_FOLD_PREFIX` and the root's index.
const ROOT_FOLD: &str = "model3d.part.fold.0";
const FOLD: &str = "model-view-parts-fold"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct ThePartTreeFolds;

impl Check for ThePartTreeFolds {
    fn name(&self) -> &'static str {
        "the_3d_viewers_part_tree_folds_on_its_boxes"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer's part tree has no + or − box on its guide lines, or pressing one \
         does not fold and unfold the branch"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch_with_model_file(ctx, &mut report, "model-fold", MODEL)
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
    for want in ["0", "-"] {
        let Some((rect, vp)) = declared_in(&session.trace()?, ui_rect(ctx)?, ROOT_FOLD) else {
            return Ok(Some(format!(
                "no `{ROOT_FOLD}` box is drawn on the root's row."
            )));
        };
        let mark = session.trace()?.mark();
        pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
        pointer.gone(session)?;
        session.settle(40);
        let trace = session.trace()?;
        let line = trace.last_after(FOLD, mark);
        report.note(format!(
            "fold box: `{}`",
            line.map_or("nothing", |l| l.raw.as_str())
        ));
        if line.and_then(|l| l.get("folded")) != Some(want)
            || line.and_then(|l| l.get("why")) != Some("box")
        {
            return Ok(Some(format!(
                "pressing the root's fold box must trace `{FOLD} why=box folded={want}`; it \
                 traced `{}`.",
                line.map_or("nothing", |l| l.raw.as_str())
            )));
        }
    }
    Ok(None)
}
