//! # `app::actions::redactimg` — say at MARK time that a region covers an image
//!
//! ## What this closes
//!
//! **Ken:** *"every time I've tried the redact feature it tells me it can't
//! because there is objects that weren't redacted."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/redactimg.md`.

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
