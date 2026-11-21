//! # `objectprovider::alpha` — a click passes through an image's clear pixels
//!
//! Point hits go through `hit_test_point_deep_with` and an engine
//! `ImageAlphaCache`, so a sample the image's mask leaves fully clear does not
//! pick the image and the ink beneath it is picked instead.
//!
//! The cache's resolver needs a `DocumentView` at query time, and the canvas
//! asks its point queries through `CanvasTargetProvider`, which holds no view.
//! So every masked image on the page is decoded when the provider is built,
//! from the view that builds it, and queries read the filled cache through a
//! view of an empty document that is never consulted for a filled entry. The
//! provider is rebuilt whenever the page's content changes, which is the
//! cache's validity contract.

use pdfcer_core::document::Document;
use pdfcer_core::vector::{
    HitTarget, ImageAlpha, ImageAlphaCache, ImageSource, PageObjects, Point, VectorObject,
    hit_test_point_deep_with,
};
use pdfcer_core::view::DocumentView;

/// A document with no objects: the view queries borrow. Every image the page
/// draws is in the cache, so it is read only for an image the build missed,
/// which then decodes to nothing and counts as opaque.
const EMPTY: &[u8] = b"%PDF-1.7\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
2 0 obj\n<< /Type /Pages /Kids [] /Count 0 >>\nendobj\ntrailer\n\
<< /Root 1 0 R /Size 3 >>\n%%EOF\n";

/// The page's decoded image masks.
#[derive(Default)]
pub(super) struct PageAlpha {
    cache: ImageAlphaCache,
    empty: Option<Document>,
}

impl PageAlpha {
    /// Decode the mask of every image `objects` draws, page and form leaves.
    pub(super) fn filled(view: &DocumentView<'_>, objects: &PageObjects, page: usize) -> Self {
        let started = std::time::Instant::now();
        let cache = ImageAlphaCache::new();
        let resolver = cache.with_view(view);
        let images = objects
            .objects
            .iter()
            .chain(objects.leaves.iter().map(|leaf| &leaf.object))
            .filter_map(|object| match object {
                VectorObject::Image(image)
                    if image.source != ImageSource::Form && image.xobject.is_some() =>
                {
                    Some(image)
                }
                _ => None,
            });
        let mut asked = 0usize;
        for image in images {
            let _ = resolver.is_clear_at(image, 0.5, 0.5);
            asked += 1;
        }
        drop(resolver);
        let decoded = cache.len();
        let ms = started.elapsed().as_millis();
        crate::diag::trace(move || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("image-alpha-filled page={page} images={asked} decoded={decoded} ms={ms}")
        });
        Self {
            cache,
            empty: Document::from_bytes(EMPTY.to_vec()).ok(),
        }
    }

    /// The deep point hit test, skipping fully clear image samples.
    ///
    /// A clear sample with nothing beneath it still picks the image. A scan
    /// saved with a transparent background is clear almost everywhere, and
    /// passing those clicks through to an empty page would leave the picture
    /// clickable only on its ink.
    pub(super) fn hits(
        &self,
        objects: &PageObjects,
        point: Point,
        tolerance: f64,
    ) -> Vec<HitTarget> {
        let Some(empty) = &self.empty else {
            return pdfcer_core::vector::hit_test_point_deep(objects, point, tolerance);
        };
        let view = empty.view();
        let resolver = self.cache.with_view(&view);
        let through = hit_test_point_deep_with(objects, point, tolerance, &resolver);
        if through.is_empty() {
            return pdfcer_core::vector::hit_test_point_deep(objects, point, tolerance);
        }
        through
    }
}
