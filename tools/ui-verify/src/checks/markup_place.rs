//! `a_markup_is_nudged_duplicated_restacked_and_typed_to_size` — a selected
//! markup shape moves under the arrow keys, duplicates with `Ctrl+D`, goes to
//! the back from Markup ▸ Arrange and takes a width typed in Properties,
//! judged in the saved file's `/Rect`s and `/Annots` order.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/markup_place.md`.

use super::dimdrive::{Fixture, click_on, press, press_on_tab, run_on_with};
use super::savedannots::{SavedAnnot, listed_annots};
use super::stroke_style::{into_view, type_into};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const FIXTURE: Fixture = Fixture {
    file: "annots-with-everything.pdf",
    method: "Rebuild it with `python tools/gen-annots-with-everything-fixture.py`.",
    page: PageGeometry {
        width_pt: 595.0,
        height_pt: 842.0,
    },
};
const INVOKE: &str = "mode.edit,file.properties";
const STEM: &str = "markup-place";
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name

const MARKUP_TAB: &str = "ribbon.tab.markup"; // ui-text-exempt: a trace region name
const TO_BACK: &str = "ribbon.item.markup.send_to_back"; // ui-text-exempt: a trace region name
const FILE_TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name
const SAVE_COPY: &str = "ribbon.item.file.save_copy"; // ui-text-exempt: a trace region name
/// Mirror `panels::properties::geometry`'s annotation regions.
const WIDTH: &str = "properties.annotgeometry.width"; // ui-text-exempt: a trace region name
const APPLY: &str = "properties.annotgeometry.apply"; // ui-text-exempt: a trace region name

const SELECTED: &str = "annot-select"; // ui-text-exempt: a trace event name
const NUDGED: &str = "annot-nudge"; // ui-text-exempt: a trace event name
const MOVED: &str = "move-annotation-applied"; // ui-text-exempt: a trace event name
const DUPLICATED: &str = "annot-duplicate"; // ui-text-exempt: a trace event name
const PASTED: &str = "paste-objects-applied"; // ui-text-exempt: a trace event name
const ARRANGED: &str = "arrange-annotation-applied"; // ui-text-exempt: a trace event name
const RESIZED: &str = "resize-annotation-applied"; // ui-text-exempt: a trace event name
const SAVED: &str = "save-copy"; // ui-text-exempt: a trace event name

/// Inside the fixture's square, `[120 560 320 700]`, and inside its
/// duplicate (10 pt right and down) too, where the duplicate is on top.
const ON_SQUARE: (f64, f64) = (220.0, 630.0);
/// One arrow-key step, page points.
const STEP: f64 = 1.0;
/// `canvas::clipboard::PASTE_OFFSET_PT`: a duplicate lands this far right
/// and this far down.
const DUPLICATE_OFFSET: f64 = 10.0;
/// The width typed for the duplicate, page points.
const TYPED_WIDTH: f64 = 150.0;
const TOLERANCE: f64 = 0.05;

/// See the module documentation.
pub struct AMarkupIsNudgedDuplicatedRestackedAndTypedToSize;

impl Check for AMarkupIsNudgedDuplicatedRestackedAndTypedToSize {
    fn name(&self) -> &'static str {
        "a_markup_is_nudged_duplicated_restacked_and_typed_to_size"
    }

    fn defect(&self) -> &'static str {
        "a selected markup shape does not move under the arrow keys, Ctrl+D does not duplicate \
         it, Send to back leaves it on top, or a width typed in Properties is not what the file \
         gets"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let saved = ctx.out(&format!("{STEM}.saved.pdf"));
        let _ = std::fs::remove_file(&saved);
        let env = [(SAVE_PATH_ENV, saved.display().to_string())];
        run_on_with(self, ctx, &FIXTURE, INVOKE, STEM, &env, drive)
    }
}

/// The last line named `event` after `mark`, or the failure sentence.
fn after(
    session: &Session,
    event: &str,
    mark: usize,
    what: &str,
) -> Result<std::result::Result<TraceLine, String>> {
    let trace = session.trace()?;
    Ok(trace.last_after(event, mark).cloned().ok_or_else(|| {
        format!(
            "{what} and no `{event}` followed. Trace: {}.",
            session.trace_path().display()
        )
    }))
}

/// Unwrap a step's line or return its failure sentence from `drive`.
macro_rules! step {
    ($e:expr) => {
        match $e? {
            Ok(line) => line,
            Err(why) => return Ok(Some(why)),
        }
    };
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    // --- select the square, and nudge it one step right.
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, ON_SQUARE)?;
    let selected = step!(after(session, SELECTED, mark, "the square was clicked"));
    if selected.get("subtype") != Some("Square") {
        return Ok(Some(format!(
            "a click inside the square selected `{}`, not the square.",
            selected.raw
        )));
    }
    let mark = session.trace()?.mark();
    pointer.key(session, None, "ArrowRight", None)?;
    session.settle(30);
    let nudged = step!(after(session, NUDGED, mark, "ArrowRight was pressed"));
    let moved = step!(after(
        session,
        MOVED,
        mark,
        "an arrow key nudged the square"
    ));
    report.note(format!("nudged: `{}` then `{}`", nudged.raw, moved.raw));
    let Some(original) = moved.get("id").and_then(|id| id.parse::<u32>().ok()) else {
        return Ok(Some(format!("`{}` names no object number.", moved.raw)));
    };

    // --- duplicate it; the copy lands on top, so a click on the overlap picks it.
    let mark = session.trace()?.mark();
    pointer.key(session, None, "D", Some("ctrl"))?;
    session.settle(30);
    let duplicated = step!(after(session, DUPLICATED, mark, "Ctrl+D was pressed"));
    let pasted = step!(after(session, PASTED, mark, "Ctrl+D planned a duplicate"));
    report.note(format!(
        "duplicated: `{}` then `{}`",
        duplicated.raw, pasted.raw
    ));
    if pasted.get("annots") != Some("1") {
        return Ok(Some(format!(
            "`{}` must plant one annotation (annots=1).",
            pasted.raw
        )));
    }
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, ON_SQUARE)?;
    step!(after(session, SELECTED, mark, "the overlap was clicked"));

    // --- send the duplicate to the back from the ribbon.
    let mark = session.trace()?.mark();
    press_on_tab(ctx, session, pointer, TO_BACK, MARKUP_TAB)?;
    session.settle(30);
    let arranged = step!(after(session, ARRANGED, mark, "Send to back was pressed"));
    report.note(format!("arranged: `{}`", arranged.raw));

    // --- type its width and apply.
    let mark = session.trace()?.mark();
    type_into(ctx, session, pointer, WIDTH, &format!("{TYPED_WIDTH}"))?;
    into_view(ctx, session, pointer, APPLY)?;
    press(ctx, session, pointer, APPLY)?;
    session.settle(30);
    let resized = step!(after(
        session,
        RESIZED,
        mark,
        "a width was typed and Apply pressed"
    ));
    report.note(format!("resized: `{}`", resized.raw));
    if resized.get("id") == Some(original.to_string().as_str()) {
        return Ok(Some(format!(
            "the typed width reached the original square (id {original}), not the duplicate the \
             click on the overlap should have picked: `{}`.",
            resized.raw
        )));
    }

    // --- the file.
    let mark = session.trace()?.mark();
    press_on_tab(ctx, session, pointer, SAVE_COPY, FILE_TAB)?;
    session.settle(40);
    step!(after(session, SAVED, mark, "Save a copy was pressed"));
    let saved = ctx.out(&format!("{STEM}.saved.pdf"));
    report.artifact(saved.clone());
    let bytes = std::fs::read(&saved)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", saved.display())))?;
    let annots = listed_annots(&bytes);
    for a in &annots {
        report.note(format!("saved {} {}: rect={:?}", a.subtype, a.num, a.rect));
    }
    Ok(judge(&annots, original))
}

/// `/Annots` as `[5 Square, 6 Text, ...]`, for a failure sentence.
fn listing(annots: &[SavedAnnot]) -> String {
    let items: Vec<String> = annots
        .iter()
        .map(|a| format!("{} {}", a.num, a.subtype))
        .collect();
    format!("[{}]", items.join(", "))
}

/// The failure sentence when the saved annotations are not the nudged
/// original (object `original`) and, first on the page, its duplicate at the
/// typed width.
fn judge(annots: &[SavedAnnot], original: u32) -> Option<String> {
    let squares: Vec<&SavedAnnot> = annots.iter().filter(|a| a.subtype == "Square").collect();
    let (Some(kept), Some(duplicate), 2) = (
        squares.iter().find(|a| a.num == original),
        squares.iter().find(|a| a.num != original),
        squares.len(),
    ) else {
        return Some(format!(
            "the saved page should list the original square {original} and one duplicate: {}.",
            listing(annots)
        ));
    };
    if annots.first().map(|a| a.num) != Some(duplicate.num) {
        return Some(format!(
            "the duplicate {} was sent to the back and is not first in the page's /Annots: {}.",
            duplicate.num,
            listing(annots)
        ));
    }
    let want_original = [120.0 + STEP, 560.0, 320.0 + STEP, 700.0];
    let (dx, dy) = (STEP + DUPLICATE_OFFSET, -DUPLICATE_OFFSET);
    let want_duplicate = [120.0 + dx, 560.0 + dy, 120.0 + dx + TYPED_WIDTH, 700.0 + dy];
    [
        (*kept, want_original, "nudged original"),
        (*duplicate, want_duplicate, "duplicate"),
    ]
    .into_iter()
    .find(|(a, want, _)| {
        a.rect
            .iter()
            .zip(want)
            .any(|(g, w)| (g - w).abs() > TOLERANCE)
    })
    .map(|(a, want, what)| {
        format!(
            "the {what} square {} has /Rect {:?}; it should be {want:?}.",
            a.num, a.rect
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(num: u32, rect: [f64; 4]) -> SavedAnnot {
        SavedAnnot {
            num,
            subtype: "Square".to_owned(),
            rect,
            body: String::new(),
        }
    }

    #[test]
    fn the_expected_end_state_passes() {
        let annots = [
            square(9, [131.0, 550.0, 281.0, 690.0]),
            square(5, [121.0, 560.0, 321.0, 700.0]),
        ];
        assert_eq!(judge(&annots, 5), None);
    }

    #[test]
    fn a_duplicate_left_on_top_is_named() {
        let annots = [
            square(5, [121.0, 560.0, 321.0, 700.0]),
            square(9, [131.0, 550.0, 281.0, 690.0]),
        ];
        assert!(judge(&annots, 5).unwrap().contains("not first"));
    }

    #[test]
    fn an_original_the_arrow_did_not_move_is_named() {
        let annots = [
            square(9, [131.0, 550.0, 281.0, 690.0]),
            square(5, [120.0, 560.0, 320.0, 700.0]),
        ];
        assert!(judge(&annots, 5).unwrap().contains("nudged original"));
    }

    #[test]
    fn the_fixture_holds_one_square_where_this_check_clicks() {
        let bytes = std::fs::read(
            crate::fixture::workspace_root()
                .join("fixtures")
                .join(FIXTURE.file),
        )
        .expect("the fixture is committed");
        let squares: Vec<SavedAnnot> = listed_annots(&bytes)
            .into_iter()
            .filter(|a| a.subtype == "Square")
            .collect();
        assert_eq!(squares.len(), 1);
        assert_eq!(squares[0].rect, [120.0, 560.0, 320.0, 700.0]);
    }
}
