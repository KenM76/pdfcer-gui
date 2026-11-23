//! A measure tool snaps to a placed image: with Linear armed, hovering beside
//! the image's top-left corner resolves a snap whose source is the image, and
//! hovering over blank paper resolves none.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/snap_image.md`.

use super::dimdrive::{Fixture, run_on};
use super::multiselect::step;
use super::snap_markup::hover;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const IMAGE: Fixture = Fixture {
    file: "image-box.pdf",
    method: "Rebuild it with `python fixtures/image-box.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 300.0,
    },
};
/// Just inside the image's top-left corner; it fills x 100..300, y 100..200.
const BESIDE_CORNER: (f64, f64) = (101.0, 199.0);
/// Paper with nothing near it.
const BLANK: (f64, f64) = (360.0, 260.0);

/// See the module documentation.
pub struct MeasureToolSnapsToAnImage;

impl Check for MeasureToolSnapsToAnImage {
    fn name(&self) -> &'static str {
        "measure_tool_snaps_to_an_image"
    }

    fn defect(&self) -> &'static str {
        "a measure tool hovering at a picture's corner finds nothing to snap to, so a placed \
         image cannot be dimensioned"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &IMAGE,
            "mode.review,measure.linear",
            "snap-image",
            drive,
        )
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let blank = step!(hover(
        ctx,
        &IMAGE,
        session,
        pointer,
        BLANK,
        "the pointer rested on blank paper with Linear armed"
    ));
    report.note(format!("blank: `{}`", blank.raw));
    if blank.get("source") != Some("none") {
        return Ok(Some(format!(
            "over blank paper the hover traced `{}`: `source=none` was wanted, so a snap \
             naming the image means something.",
            blank.raw
        )));
    }
    let corner = step!(hover(
        ctx,
        &IMAGE,
        session,
        pointer,
        BESIDE_CORNER,
        "the pointer rested beside the image's corner"
    ));
    report.note(format!("corner: `{}`", corner.raw));
    if corner.get("snap") != Some("1") || corner.get("source") != Some("image") {
        return Ok(Some(format!(
            "beside the image's corner the hover traced `{}`: `snap=1 source=image` was \
             wanted.",
            corner.raw
        )));
    }
    Ok(None)
}
