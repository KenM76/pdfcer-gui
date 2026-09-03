//! # `app::actions::redactimg` — say at MARK time that a region covers an image
//!
//! ## What this closes
//!
//! **Ken, 2026-09-03:** *"every time I've tried the redact feature it tells me
//! it can't because there is objects that weren't redacted."*
//!
//! Reproduced with `pdfcer` alone, so the refusal was the engine's and was
//! never ours to remove:
//!
//! ```text
//! redaction refused: redaction region on page 1 intersects an image; pdfcer
//! cannot yet destroy image pixels (clipping or masking would leave them
//! recoverable, ISO 32000-1 §12.5.6.23) — apply refused rather than producing a
//! false redaction
//! ```
//!
//! The gate was in `redact-apply`, and apply was **all-or-nothing for the
//! document**. So the sequence he actually lived through was: mark twelve
//! regions across a drawing, carefully; press Apply; be told the whole thing is
//! refused, because **one** of them grazed the bounding box of a logo. Nothing
//! said which. On a title block, a value worth redacting is often inches from a
//! company logo.
//!
//! ## ★★★ THE REFUSAL IS GONE — and this module's subject changed, not its job
//!
//! `pdfcer-core` **v0.26.0** (`Pass 245.0`) and **v0.27.0** (`Pass 246.0`) both
//! landed on 2026-09-03, hours after the report. The engine now:
//!
//! * gates on the image **samples** rather than the bounding boxes, so a region
//!   that merely touches an image's rectangle destroys nothing;
//! * **destroys** the covered samples — decode, overwrite, clear the matching
//!   part of any soft mask, re-encode losslessly — and removes an image that is
//!   wholly covered;
//! * retains just the **marks** it cannot apply rather than refusing the
//!   document;
//! * and **cuts vector geometry** at the region boundary, which was the bigger
//!   residual on a drawing and one nobody had reported.
//!
//! ★★ **So the disclosure changed subject rather than becoming unnecessary, and
//! it changed to the more important of the two.** It used to say *"this will be
//! refused"*; it now says *"those pixels will be destroyed, not hidden"*. A
//! raster redaction is irreversible in a way a text one is not — the samples are
//! overwritten and the image re-encoded — and finding that out while the
//! rectangle is being drawn is the same argument, applied to the opposite
//! outcome.
//!
//! ⇒ ★ **Nothing in this repository could have caught the change.** The claim
//! lived in a UI string; it compiled and passed for as long as it was false. The
//! engine's reply asked us to re-word it, by name. Where such a claim can be
//! spelled as a test assertion instead, spell it as one — `redact::tests`'s
//! image test went red the hour the engine changed underneath it, which is
//! exactly the behaviour a paragraph cannot have.
//!
//! ## This is DISCLOSURE, not a gate — and the distinction is load bearing
//!
//! It refuses nothing and blocks nothing. The mark is authored exactly as
//! before, because a mark is reversible and costs nothing, and because pdfcer
//! must not decide on the operator's behalf that a region is not worth marking.
//! What changes is only that he is told, in the same breath as the success, and
//! can act while it is cheap — which now means *before* the pixels go, rather
//! than before a refusal arrives.
//!
//! ★★ Rule 4: nothing is drawn on the canvas. The mark renders exactly as any
//! other mark renders, because it IS any other mark — a warning tint would be
//! pdfcer styling its own uncertainty into content, which is the thing the rule
//! forbids by name. The sentence goes where every other disclosure goes.

use crate::app::state::OpenDoc;
use crate::canvas::pick::PickClass;
use crate::canvas::target::CanvasTargetProvider;
use pdfcer_core::vector::MarqueeMode;

/// How many of `targets` on `page_index` are raster images.
///
/// Split out as its own function because it is the whole factual claim this
/// module makes, and because it is the part that could be wrong in a way an
/// operator would notice: over-counting invents a warning about a page that
/// would have redacted cleanly, and under-counting is the silence this module
/// exists to end.
fn image_count(
    doc: &OpenDoc,
    page_index: usize,
    targets: &[crate::canvas::target::TargetId],
) -> usize {
    let Some(provider) = doc.page_objects() else {
        return 0;
    };
    targets
        .iter()
        .filter(|t| {
            matches!(
                provider.object_class(page_index, **t),
                Some(PickClass::Image)
            )
        })
        .count()
}

/// **Does anything the operator just selected sit on a raster image?**
///
/// Used by the selection route, where the answer needs no geometry at all: the
/// operator picked the objects, so their classes are already known and asking
/// the decomposition a second question could only produce a second answer.
#[must_use]
pub fn images_in_selection(doc: &OpenDoc) -> usize {
    let page_index = doc.view.page_index;
    let targets = doc.selection.targets_on(page_index);
    image_count(doc, page_index, &targets)
}

/// **How many raster images are anywhere on `page_index`.**
///
/// The whole-page route's question, and it is the simple one: a mark that
/// covers the page covers every image on it, so any image at all means the
/// apply will be refused.
#[must_use]
pub fn images_on_page(doc: &OpenDoc, page_index: usize) -> usize {
    let Some(provider) = doc.page_objects() else {
        return 0;
    };
    let Some(page) = doc.pages.get(page_index) else {
        return 0;
    };
    // ★ The sheet in CANVAS space, taken from the render geometry rather than
    // from the media box directly: canvas space is the page's device space at
    // scale 1.0, and `page_device_geometry` is the one place that mapping
    // lives. Building the rectangle from `/MediaBox` by hand would be a second
    // statement of it, and the two would disagree the first time a page carried
    // a `/Rotate` or a crop.
    let (w, h, _) = pdfcer_render::page_device_geometry(page, 1.0);
    let whole = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(w as f32, h as f32));
    let hits = provider.hit_test_rect(page_index, whole, MarqueeMode::Touched);
    image_count(doc, page_index, &hits)
}
