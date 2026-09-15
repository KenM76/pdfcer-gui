//! # `app::actions::redactimg` — say at MARK time that a region covers an image
//!
//! ## What this closes
//!
//! **Ken:** *"every time I've tried the redact feature it tells me it can't
//! because there is objects that weren't redacted."*
//!
//! A redaction that covers a raster is not a redaction that fails. The engine
//! gates on the image **samples** rather than on bounding boxes, so a region
//! that merely touches an image's rectangle destroys nothing; where the region
//! does cover samples it **destroys** them — decode, overwrite, clear the
//! matching part of any soft mask, re-encode losslessly — and removes an image
//! that is wholly covered. Only a mark over an image pdfcer cannot decode is
//! **retained** (`RedactionReport::marks_retained`), and only a document where
//! *every* mark would be retained is refused (`RedactError::ImageUndestroyable`).
//!
//! **So the sentence this module supplies is about destruction, not refusal:**
//! *those pixels will be destroyed, not hidden*. A raster redaction is
//! irreversible in a way a text one is not — the samples are overwritten and
//! the image re-encoded — and the moment to learn that is while the rectangle
//! is being drawn.
//!
//! ⇒ **A claim about the engine that lives only in a UI string compiles and
//! passes for as long as it is false.** Where such a claim can be spelled as a
//! test assertion instead, spell it as one: `redact::tests`' image test goes
//! red the hour the engine changes underneath it, which is exactly the
//! behaviour a paragraph cannot have.
//!
//! ## This is DISCLOSURE, not a gate — and the distinction is load bearing
//!
//! It refuses nothing and blocks nothing. The mark is authored exactly as
//! before, because a mark is reversible and costs nothing, and because pdfcer
//! must not decide on the operator's behalf that a region is not worth marking.
//! What changes is only that he is told, in the same breath as the success, and
//! can act while it is cheap — before the pixels go.
//!
//! Rule 4: nothing is drawn on the canvas. The mark renders exactly as any
//! other mark renders, because it IS any other mark — a warning tint would be
//! pdfcer styling its own uncertainty into content, which is the thing the rule
//! forbids by name. The sentence goes where every other disclosure goes.

use crate::app::state::OpenDoc;
use crate::canvas::pick::PickClass;
use crate::canvas::target::CanvasTargetProvider;
use pdfcer_core::vector::{FormMarquee, MarqueeMode};

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
/// covers the page covers every image on it, so any image at all means an apply
/// will destroy raster samples somewhere on that sheet.
#[must_use]
pub fn images_on_page(doc: &OpenDoc, page_index: usize) -> usize {
    let Some(provider) = doc.page_objects() else {
        return 0;
    };
    let Some(page) = doc.pages.get(page_index) else {
        return 0;
    };
    // The sheet in CANVAS space, taken from the render geometry rather than
    // from the media box directly: canvas space is the page's device space at
    // scale 1.0, and `page_device_geometry` is the one place that mapping
    // lives. Building the rectangle from `/MediaBox` by hand would be a second
    // statement of it, and the two would disagree the first time a page carried
    // a `/Rotate` or a crop.
    let (w, h, _) = pdfcer_render::page_device_geometry(page, 1.0);
    let whole = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(w as f32, h as f32));
    // `Exclude`, and the choice is about MEANING rather than about the
    // number. This question is *"is there ink here that redaction cannot
    // destroy?"*, and a form's `/BBox` is a clipping extent (§8.10.1), not
    // ink — so a container has no business in the answer. The count is
    // unchanged either way today, because `object_class` reports a form as
    // `PickClass::FormXObject` and never as `PickClass::Image`; stating
    // `Exclude` is what keeps that from being the reason, because the day a
    // form starts classing as an image is the day this would start warning
    // about a page with no images on it.
    //
    // Images INSIDE a form are still counted — leaves are candidates under
    // both policies. That is the half that matters: a logo in a title block
    // is the commonest raster on a CAD sheet.
    let hits = provider.hit_test_rect(
        page_index,
        whole,
        MarqueeMode::Touched,
        FormMarquee::Exclude,
    );
    image_count(doc, page_index, &hits)
}
