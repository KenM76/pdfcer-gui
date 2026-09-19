//! # `canvas::chunks` — **the boxes that show what a text block is made of**
//!
//! `OPERATOR_REQUESTS.md` **O215**, ask 3, in his words: *"click on a text
//! block and it would show us boxes around all the blocks contained within it,
//! then let us use our usual mouse selection methods to move the chunks."*
//!
//! ## What a chunk is, and why the boxes are the prerequisite
//!
//! A *chunk* is what [`crate::panels::objects::provider::ObjectModelProvider`]
//! calls a text **line**: the run range `text_line_runs_of` answers for, which
//! is the Part rung's unit and the operand of `MoveTextLine`,
//! `format.select_text_line` and the run menu. One text object on a CAD sheet
//! holds hundreds of them.
//!
//! Selection already descends to that unit, and the operator cannot see it.
//! `canvas::presspick::covers` decides on **press** whether a drag moves the
//! chunk under the pointer or the whole block, and it decides by asking whether
//! the press landed inside the current selection's outline — a rectangle
//! nothing draws. A press a few points off it re-selects at the Object rung and
//! the same gesture moves the whole block instead. That is the mechanism behind
//! the non-determinism O215 ask 1 reports, so drawing the boxes is not
//! decoration: it is what makes the gesture repeatable, because it is what makes
//! the thing being aimed at visible.
//!
//! ## R8b — these are a cursor, not content marking
//!
//! *Fuzzy, never sneaky* forbids styling applied content as provisional. A chunk
//! outline styles nothing: it is a **pre-commit affordance**, in the same class
//! as a snap indicator, a rubber-band or a selection handle, and the rule admits
//! those by name. Saving and reopening the document produces the same page; only
//! the cursor differs.
//!
//! ## Where the state lives
//!
//! Two homes, exactly as [`crate::canvas::smart`] does and for its reason: the
//! live answer in `egui::Memory`, because the painter and the click path reach
//! it with a context and nothing else, and the persisted answer on
//! [`crate::app::prefs::Prefs`], because an operator who turns a mode off
//! expects it to still be off tomorrow. `crate::app::frame` mirrors the second
//! into the first once a frame, and the only writer of the persisted answer is
//! the dispatch arm an operator's press runs.

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
///
/// A backstop rather than the normal case. The largest text object measured on
/// the operator's own `SW41177.pdf` holds 144 chunks; this is an order of
/// magnitude above that, so a page that trips it is a page where the boxes would
/// be a grey wash rather than an answer. When it trips, the count is disclosed
/// off-canvas — an inference the operator cannot see still owes a report.
pub const MAX_CHUNK_BOXES: usize = 2_000;

/// The memory id for a key.
fn id(key: &str) -> egui::Id {
    egui::Id::new(key)
}

/// Are the chunk boxes switched on?
///
/// **`true` when nothing has been stored**, which is the same argument smart
/// select makes: the switch exists so the boxes can be turned OFF, and an
/// operator who has to find a checkbox before a feature he asked for appears has
/// not been given the feature.
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
///
/// The direction is strictly `Prefs` → memory, and the guard is not an
/// optimisation: an unconditional write every frame would make the value
/// impossible to change from anywhere else, which is how a mirror becomes an
/// overwrite.
pub fn sync(ctx: &egui::Context, on: bool) {
    if enabled(ctx) != on {
        ctx.data_mut(|d| d.insert_temp(id(ENABLED_KEY), on));
    }
}

/// **Whether the boxes are drawn for `object`** — the switch is on and the
/// object holds more than one chunk.
///
/// The gate on every gesture that narrows the selection to a chunk. Narrowing
/// to a unit the operator cannot see is the unpredictability O215 ask 1
/// reports, reached from the other side, so the rung is offered exactly where
/// the boxes are and nowhere else. The second term is the same one [`outlines`]
/// declines `single-chunk` on: a one-line object draws no box, so a click that
/// descended into it would change the verb set with nothing on screen to say
/// it had.
#[must_use]
pub fn boxed(ctx: &egui::Context, doc: &OpenDoc, object: TargetId) -> bool {
    enabled(ctx)
        && doc
            .page_objects()
            .is_some_and(|provider| provider.text_line_count_of(object) >= 2)
}

/// **The chunk of `object` under `point`**, in the same page space
/// [`crate::canvas::input::probe`] asks its questions in.
///
/// The first hit, exactly as `probe` takes the first of `part_hits_of` — one
/// rule, asked in two places, rather than two rules that agree today. `None`
/// means the point is inside the block's box but not on any of its lines,
/// which on a CAD note is most of the block.
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

/// Why no chunk outlines were drawn.
///
/// A fixed vocabulary rather than free text, for the reason
/// `canvas::painting`'s `draw_anchors` states: the driven checks that read this
/// trace all begin by asking whether the boxes appeared, and every one of them
/// would otherwise have to guess whether the answer means *the program is
/// broken*, *the aim is wrong* or *this is the normal case*.
///
/// | reason | what it means | what it is about |
/// |---|---|---|
/// | `switched-off` | the operator turned the boxes off | neither; normal |
/// | `nothing-selected` | the boxes are on and nothing is selected | the driver: click a text block first |
/// | `other-page` | everything selected is on a page this call is not painting | neither; continuous view |
/// | `not-text` | the selection holds no text object | the driver, or the aim |
/// | `single-chunk` | every selected text object holds one chunk | neither: a box on the whole block says nothing the selection outline does not |
/// | `no-provider` | the page decomposition is not available this frame | the program, or a load still in flight |
/// | `too-many-chunks` | past [`MAX_CHUNK_BOXES`]; the count is on the status bar | the document |
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
///
/// Canvas space, not screen space: the projection is the painter's, and doing it
/// here would bake this frame's scroll offset into a value the caller then
/// projects again.
///
/// Callers hold the returned `Vec` rather than a `Ref` into the decomposition
/// cache, deliberately. The borrow is taken and released inside this function,
/// which is the discipline `app::cache::page_objects` asks of every reader.
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
        // ★ A single-chunk object contributes nothing. Its one box would sit on
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
    ///
    /// The vocabulary earns its keep only if a harness can tell one from
    /// another; two reasons that happened to be spelled the same would collapse
    /// *click something first* into *the program is broken* with nothing to say
    /// which had happened.
    #[test]
    fn every_decline_reason_is_distinct() {
        let mut sorted = DECLINED_REASONS.to_vec();
        sorted.sort_unstable();
        let before = sorted.len();
        sorted.dedup();
        assert_eq!(sorted.len(), before, "every reason is distinct");
    }
}
