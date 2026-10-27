//! `store_appearances_writes_a_picture_into_the_file` — Markup ▸ Comments ▸
//! *Store appearances* on a markup with no `/AP` (the shape Microsoft Edge
//! saves) writes one, so other viewers draw it. Scripted pointer, window off
//! the desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/store_appearances.md`.

use super::dimdrive::{Fixture, reveal, ui_rect_event};
use super::multiselect::{Saves, run_saving, save};
use super::savedannots::{listed_annots, read};
use crate::checks::driving::declared_or_in_overflow;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::Click;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.review";
const FIXTURE: Fixture = Fixture {
    file: "square-no-appearance.pdf",
    method: "Committed with the repository; restore it from git.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const STEM: &str = "store-appearances";
const MARKUP_TAB: &str = "ribbon.tab.markup";
const STORE: &str = "ribbon.item.markup.store_appearances";
const STORED: &str = "appearances-stored"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct StoreAppearancesWritesAPictureIntoTheFile;

impl Check for StoreAppearancesWritesAPictureIntoTheFile {
    fn name(&self) -> &'static str {
        "store_appearances_writes_a_picture_into_the_file"
    }

    fn defect(&self) -> &'static str {
        "Store appearances is missing, or leaves a markup without an /AP, so other viewers \
         still show nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_saving(self, ctx, &FIXTURE, INVOKE, STEM, steps)
    }
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    saves: &Saves,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    reveal(ctx, session, pointer, STORE, MARKUP_TAB)?;
    session.settle(15);
    // An absent command is the defect, not an unrunnable check.
    let Some(found) = declared_or_in_overflow(session, pointer, ui_rect_event(ctx)?, STORE)? else {
        return Ok(Some(format!(
            "★★★ Markup offers no `{STORE}`, on the band or in its overflow. Trace: {path}."
        )));
    };
    let mark = session.trace()?.mark();
    pointer.click_rect(session, found)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(line) = trace.last_after(STORED, mark) else {
        return Ok(Some(format!(
            "★★★ pressing `{STORE}` traced no `{STORED}`. Trace: {path}."
        )));
    };
    let baked = line.get("baked").unwrap_or_default().to_owned();
    report.note(format!("{STORED} baked={baked}"));
    if baked != "1" {
        return Ok(Some(format!(
            "★★★ the fixture's one appearance-less /Square traced `baked={baked}`, not 1. \
             Trace: {path}."
        )));
    }
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let annots = listed_annots(&read(&saves.moved)?);
    let squares: Vec<&str> = annots
        .iter()
        .filter(|a| a.subtype == "Square")
        .map(|a| a.body.as_str())
        .collect();
    report.note(format!("saved Squares: {}", squares.len()));
    Ok(
        (squares.is_empty() || !squares.iter().all(|b| b.contains("/AP"))).then(|| {
            format!(
                "★★★ {} lists {} /Square, not every one with an /AP: the picture did not reach \
             the file.",
                saves.moved.display(),
                squares.len()
            )
        }),
    )
}
