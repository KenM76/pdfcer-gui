//! Format ▸ Make pictures smaller driven to store a picture as black and
//! white through each 1-bit encoder: CCITT Group 4 on one picture, JBIG2 on
//! the other. Each must reach the engine as that encoder and rewrite the
//! picture with it.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/bilevel_pictures.md`.

use super::dimdrive::{Fixture, click_on, press, press_on_tab, run_on};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.edit";
const OPENED: &str = "shrink-images-opened"; // ui-text-exempt: a trace event name, never displayed
const SHRUNK: &str = "images-shrunk"; // ui-text-exempt: a trace event name, never displayed
const FORMAT_TAB: &str = "ribbon.tab.format";
const RIBBON_ITEM: &str = "ribbon.item.format.shrink_images";
const COLOUR_LIST: &str = "compact.pictures.colour";
const BLACK_AND_WHITE: &str = "compact.pictures.colour.bilevel";
const FORMAT_LIST: &str = "compact.pictures.format";
const COMMIT: &str = "shrink-images.commit";
const PHOTOS: Fixture = Fixture {
    file: "two-photos.pdf",
    method: "Rebuild it with `python fixtures/two-photos.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 200.0,
    },
};
/// Each pass: the picture's centre in page space, and the encoder chosen.
const PASSES: [((f64, f64), &str); 2] = [((100.0, 100.0), "ccitt"), ((300.0, 100.0), "jbig2")];

/// See the module documentation.
pub struct BilevelPicturesStoreAsCcittAndJbig2;

impl Check for BilevelPicturesStoreAsCcittAndJbig2 {
    fn name(&self) -> &'static str {
        "bilevel_pictures_store_as_ccitt_and_jbig2"
    }

    fn defect(&self) -> &'static str {
        "Make pictures smaller with Colours ▸ Black and white offers no CCITT Group 4 or \
         JBIG2 storage, or Apply does not rewrite the picture with the encoder chosen"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &PHOTOS, INVOKE, "bilevelpictures", drive)
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    for (at, encoder) in PASSES {
        if let Some(failure) = pass(ctx, report, session, pointer, at, encoder)? {
            return Ok(Some(failure));
        }
    }
    Ok(None)
}

/// Select the picture at `at`, open the window, choose black and white
/// stored by `encoder`, Apply, and require the edit to have used it.
fn pass(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    at: (f64, f64),
    encoder: &str,
) -> Result<Option<String>> {
    click_on(ctx, session, pointer, &PHOTOS, 0, at)?;
    let mark = session.trace()?.mark();
    press_on_tab(ctx, session, pointer, RIBBON_ITEM, FORMAT_TAB)?;
    let trace = session.trace()?;
    if trace.last_after(OPENED, mark).is_none() {
        return Ok(Some(format!(
            "the Format tab opened no picture window (no `{OPENED}`). Trace: {}.",
            session.trace_path().display()
        )));
    }
    press(ctx, session, pointer, COLOUR_LIST)?;
    press(ctx, session, pointer, BLACK_AND_WHITE)?;
    press(ctx, session, pointer, FORMAT_LIST)?;
    press(ctx, session, pointer, &format!("{FORMAT_LIST}.{encoder}"))?;
    press(ctx, session, pointer, COMMIT)?;
    session.settle(60);
    let trace = session.trace()?;
    let Some(line) = trace.last_after(SHRUNK, mark) else {
        return Ok(Some(format!(
            "Apply after choosing {encoder} reached no edit (no `{SHRUNK}`). Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("{encoder}: `{}`", line.raw));
    let used = line.get(encoder) == Some("1");
    let changed = line.get("changed") == Some("1") && line.get("skipped") == Some("0");
    if used && changed {
        return Ok(None);
    }
    Ok(Some(format!(
        "Apply with Black and white stored as {encoder} must read `changed=1 {encoder}=1 \
         skipped=0`: the picture rewritten by that encoder. It read `{}`. Trace: {}.",
        line.raw,
        session.trace_path().display()
    )))
}
