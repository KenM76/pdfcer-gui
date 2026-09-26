//! # `canvas::chunks` — **the boxes that show what a text block is made of**
//!
//! `OPERATOR_REQUESTS.md` **O215**, ask 3, in his words: *"click on a text
//! block and it would show us boxes around all the blocks contained within it,
//! then let us use our usual mouse selection methods to move the chunks."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/chunks.md`.

use egui::Rect;

use crate::app::state::OpenDoc;
use crate::canvas::selection::SelectionState;
use crate::canvas::target::{CanvasTargetProvider, TargetId};

/// Whether the chunk boxes are switched on. Memory key.
///
/// Application-scoped rather than per document, like the armed tool and smart
/// select: it is a statement about how this operator works, not about a file.
const ENABLED_KEY: &str = "pdfcer.text-chunks.enabled"; // ui-text-exempt: a memory key, never displayed

/// The most chunk outlines drawn at once.
pub const MAX_CHUNK_BOXES: usize = 2_000;

/// The memory id for a key.
fn id(key: &str) -> egui::Id {
    egui::Id::new(key)
}

/// Are the chunk boxes switched on?
#[must_use]
pub fn enabled(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(id(ENABLED_KEY)))
        .unwrap_or(true)
}

/// Turn them on or off.
pub fn set_enabled(ctx: &egui::Context, on: bool) {
    ctx.data_mut(|d| d.insert_temp(id(ENABLED_KEY), on));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("text-chunks enabled={on}")
    });
}

/// Mirror the persisted answer into the live one, once per frame.
pub fn sync(ctx: &egui::Context, on: bool) {
    if enabled(ctx) != on {
        ctx.data_mut(|d| d.insert_temp(id(ENABLED_KEY), on));
    }
}

/// **Whether the boxes are drawn for `object`** — the switch is on and the
/// object holds more than one chunk.
#[must_use]
pub fn boxed(ctx: &egui::Context, doc: &OpenDoc, object: TargetId) -> bool {
    enabled(ctx)
        && doc
            .page_objects()
            .is_some_and(|provider| provider.text_line_count_of(object) >= 2)
}

/// **The chunk of `object` under `point`**, in the same page space
/// [`crate::canvas::input::probe`] asks its questions in.
#[must_use]
pub fn under(
    doc: &OpenDoc,
    page_index: usize,
    object: TargetId,
    point: egui::Pos2,
    tolerance: f64,
) -> Option<usize> {
    let provider = doc.page_objects()?;
    let hit = provider
        .part_hits_of(page_index, object, point, tolerance)
        .first()
        .copied();
    drop(provider);
    hit
}

/// **Which chunks of `object` a released rubber-band takes**, ascending and
/// unique, in the canvas space [`outlines`] draws in.
#[must_use]
pub fn within(doc: &OpenDoc, object: TargetId, band: Rect, crossing: bool) -> Vec<usize> {
    let Some(provider) = doc.page_objects() else {
        return Vec::new();
    };
    let count = provider.text_line_count_of(object).min(MAX_CHUNK_BOXES);
    let mut taken = Vec::with_capacity(count);
    for line in 0..count {
        // A chunk the provider will not measure cannot be reached by a band
        // either: there is no rectangle to test, and guessing one would select
        // a line the operator never saw a box around.
        let Some(rect) = provider.text_line_bounds_canvas_of(object, line) else {
            continue;
        };
        if reaches(band, rect, crossing) {
            taken.push(line);
        }
    }
    drop(provider);
    taken
}

/// Whether a band takes one chunk, under the direction rule.
fn reaches(band: Rect, chunk: Rect, crossing: bool) -> bool {
    if crossing {
        band.intersects(chunk)
    } else {
        band.contains_rect(chunk)
    }
}

/// Why no chunk outlines were drawn.
pub type Declined = &'static str;

/// Every reason [`outlines`] can decline with.
///
/// Iterated by the test that asserts they are distinct strings, which is the
/// property a harness telling them apart depends on.
pub const DECLINED_REASONS: &[Declined] = &[
    "switched-off",
    "nothing-selected",
    "other-page",
    "not-text",
    "single-chunk",
    "no-provider",
    "too-many-chunks",
];

/// **One canvas-space rectangle per chunk of every selected text object on
/// `page_index`** — or the reason there are none.
pub fn outlines(
    doc: &OpenDoc,
    selection: &SelectionState,
    page_index: usize,
) -> Result<Vec<Rect>, Declined> {
    let entries = selection.entries();
    if entries.is_empty() {
        return Err("nothing-selected");
    }
    let here: Vec<_> = entries.iter().filter(|s| s.page == page_index).collect();
    if here.is_empty() {
        return Err("other-page");
    }
    let Some(provider) = doc.page_objects() else {
        return Err("no-provider");
    };
    let mut text_objects = 0usize;
    let mut chunks = 0usize;
    let mut boxes: Vec<Rect> = Vec::new();
    for entry in here {
        let count = provider.text_line_count_of(entry.object);
        if count == 0 {
            continue;
        }
        text_objects += 1;
        chunks += count;
        // A single-chunk object contributes nothing. Its one box would sit on
        // top of the selection outline already drawn there, and two rectangles
        // at the same place saying two different things is worse than one.
        if count < 2 {
            continue;
        }
        for line in 0..count {
            if boxes.len() >= MAX_CHUNK_BOXES {
                break;
            }
            if let Some(rect) = provider.text_line_bounds_canvas_of(entry.object, line) {
                boxes.push(rect);
            }
        }
    }
    drop(provider);
    if text_objects == 0 {
        return Err("not-text");
    }
    if chunks > MAX_CHUNK_BOXES {
        crate::app::actions::record_note(
            doc.edit_epoch,
            crate::text::status::too_many_text_chunks(chunks, MAX_CHUNK_BOXES),
        );
        return Err("too-many-chunks");
    }
    if boxes.is_empty() {
        return Err("single-chunk");
    }
    Ok(boxes)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A band that clips a chunk takes it only when it is a crossing one.**
    #[test]
    fn only_a_crossing_band_takes_a_chunk_it_merely_clips() {
        let chunk = Rect::from_min_max(egui::pos2(100.0, 100.0), egui::pos2(300.0, 112.0));
        let clipping = Rect::from_min_max(egui::pos2(200.0, 90.0), egui::pos2(400.0, 130.0));
        assert!(reaches(clipping, chunk, true), "a crossing band takes it");
        assert!(
            !reaches(clipping, chunk, false),
            "an enclosing band does not, because it does not surround it"
        );

        let around = Rect::from_min_max(egui::pos2(90.0, 90.0), egui::pos2(400.0, 130.0));
        assert!(reaches(around, chunk, false), "surrounded is taken");
        assert!(reaches(around, chunk, true), "and touched, by both rules");

        let elsewhere = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(50.0, 50.0));
        assert!(!reaches(elsewhere, chunk, true), "a miss is a miss");
        assert!(!reaches(elsewhere, chunk, false));
    }

    /// The default is ON, and it is the default a *fresh* context reports —
    /// which is the value an operator who has never touched the switch gets.
    #[test]
    fn a_context_that_has_never_been_told_says_the_boxes_are_on() {
        let ctx = egui::Context::default();
        assert!(enabled(&ctx));
    }

    /// Off survives, and is not re-defaulted by the next read.
    #[test]
    fn switching_them_off_is_remembered() {
        let ctx = egui::Context::default();
        set_enabled(&ctx, false);
        assert!(!enabled(&ctx));
        assert!(!enabled(&ctx), "a read must not re-default the answer");
        set_enabled(&ctx, true);
        assert!(enabled(&ctx));
    }

    /// The mirror carries the persisted answer in both directions.
    #[test]
    fn the_mirror_carries_the_persisted_answer_both_ways() {
        let ctx = egui::Context::default();
        sync(&ctx, false);
        assert!(!enabled(&ctx));
        sync(&ctx, true);
        assert!(enabled(&ctx));
    }

    /// Every decline reason is a distinct string.
    #[test]
    fn every_decline_reason_is_distinct() {
        let mut sorted = DECLINED_REASONS.to_vec();
        sorted.sort_unstable();
        let before = sorted.len();
        sorted.dedup();
        assert_eq!(sorted.len(), before, "every reason is distinct");
    }
}
