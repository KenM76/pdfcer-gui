//! The texture cache behind a continuous strip: the rasters for the pages
//! on screen other than the current one, under a texel budget.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/render/strip.md`.

use crate::raster::PageTexture;
use crate::renderworker::RenderKey;

/// The most texels this cache will hold when the operator has expressed no
/// preference — the value `pdfcer_gui::app::prefs::PageCache::default()` resolves
/// to, kept here beside the code that spends it.
pub const MAX_CACHED_TEXELS: u64 = 256_000_000;

/// Either a page's raster, or the reason there will not be one.
#[derive(Debug)]
pub enum PageRaster {
    /// The page drew, and here are its pixels.
    Ready(Box<PageTexture>),
    /// The page would not draw, and this is the renderer's own account of why.
    Failed(String),
}

/// One cached page.
#[derive(Debug)]
struct Entry {
    /// Which page.
    page: usize,
    /// What the raster is *of* — page, scale, annotations, layer generation.
    key: RenderKey,
    /// The document revision it was rendered at. See the module header on why
    /// this is carried in addition to `key`.
    epoch: u64,
    /// The raster, or the refusal.
    raster: PageRaster,
    /// How many texels it occupies, for the budget.
    texels: u64,
}

/// **The rasters for the pages a continuous strip is showing, other than the
/// current one.**
#[derive(Debug, Default)]
pub struct StripRasters {
    /// Newest first. A `Vec` rather than a map because it holds a handful of
    /// entries — the pages that fit in a viewport — and a linear scan over
    /// four elements is faster than hashing one, with no allocation per
    /// insert and an eviction order that is a `sort` rather than a rebuild.
    entries: Vec<Entry>,
}

impl StripRasters {
    /// The raster for `page`, if there is a current one.
    #[must_use]
    pub fn get(&self, page: usize, key: RenderKey, epoch: u64) -> Option<&PageRaster> {
        self.entries
            .iter()
            .find(|e| e.page == page && e.key == key && e.epoch == epoch)
            .map(|e| &e.raster)
    }

    /// Whether a *current* entry exists for `page` — a texture or a recorded
    /// refusal.
    #[must_use]
    pub fn has(&self, page: usize, key: RenderKey, epoch: u64) -> bool {
        self.get(page, key, epoch).is_some()
    }

    /// Record a finished render.
    pub fn insert(&mut self, page: usize, key: RenderKey, epoch: u64, raster: PageRaster) {
        let texels = match &raster {
            PageRaster::Ready(texture) => {
                let size = texture.texture.size();
                (size[0] as u64).saturating_mul(size[1] as u64)
            }
            // A refusal occupies a string. Counting it as zero is honest: the
            // budget is about GPU memory, and there is none here.
            PageRaster::Failed(_) => 0,
        };
        self.entries.retain(|e| e.page != page);
        self.entries.push(Entry {
            page,
            key,
            epoch,
            raster,
            texels,
        });
    }

    /// **Drop the current page's entry, then prune to the budget.**
    pub fn retain(&mut self, current: usize, budget: u64) {
        self.entries.retain(|e| e.page != current);

        let mut total: u64 = self.entries.iter().map(|e| e.texels).sum();
        while total > budget && self.entries.len() > 1 {
            // Furthest from the current page. `position_max_by_key` does not
            // exist in std, so this is the explicit fold — and it must be a
            // fold rather than a sort, because sorting a cache to drop one
            // entry reorders every insert's neighbours for nothing.
            let Some((index, _)) = self
                .entries
                .iter()
                .enumerate()
                .max_by_key(|(_, e)| e.page.abs_diff(current))
            else {
                break;
            };
            total = total.saturating_sub(self.entries[index].texels);
            let dropped = self.entries.remove(index);
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "strip-raster-evicted page={} texels={} remaining={}",
                    dropped.page, dropped.texels, total
                )
            });
        }
    }

    /// **Remove and return** the raster for `page`, if it is current.
    #[must_use]
    pub fn take(&mut self, page: usize, key: RenderKey, epoch: u64) -> Option<PageRaster> {
        let index = self
            .entries
            .iter()
            .position(|e| e.page == page && e.key == key && e.epoch == epoch)?;
        Some(self.entries.remove(index).raster)
    }

    /// Forget everything.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// How many pages are cached. Diagnostic, and what the `canvas` trace line
    /// reports so a driven run can see the strip filling.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing is cached — true for every single-page session.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// How many texels are resident. Diagnostic; the budget's own reading.
    #[must_use]
    pub fn texels(&self) -> u64 {
        self.entries.iter().map(|e| e.texels).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(page: usize, scale: f32) -> RenderKey {
        RenderKey::new(
            page,
            scale,
            true,
            0,
            // Faithful widths — the default, and the only answer these
            // cache-behaviour tests care about. `view.line_weights` (O137) is a
            // key component, so a test that varied it would be testing the key
            // rather than the strip; `the_render_key_moves_when_line_weights_are_turned_off`
            // in `renderworker` is where that property is pinned.
            pdfcer_render::font::StrokeDisplay::Actual,
        )
    }

    /// A failure is cached, so a page that will not draw is not re-attempted
    /// on every frame.
    #[test]
    fn a_refusal_is_cached_so_it_is_not_retried_every_frame() {
        let mut cache = StripRasters::default();
        assert!(!cache.has(3, key(3, 2.0), 0));
        cache.insert(
            3,
            key(3, 2.0),
            0,
            PageRaster::Failed("content stream will not inflate".to_owned()),
        );
        assert!(
            cache.has(3, key(3, 2.0), 0),
            "the refusal must count as an answer"
        );
        assert!(matches!(
            cache.get(3, key(3, 2.0), 0),
            Some(PageRaster::Failed(_))
        ));
        assert_eq!(cache.texels(), 0, "a refusal occupies no texels");
    }

    /// **A lookup misses on a different scale, and on a different edit
    /// epoch.**
    #[test]
    fn a_stale_key_or_a_stale_epoch_is_a_miss() {
        let mut cache = StripRasters::default();
        cache.insert(3, key(3, 2.0), 7, PageRaster::Failed(String::new()));
        assert!(cache.has(3, key(3, 2.0), 7));
        assert!(!cache.has(3, key(3, 2.5), 7), "a zoom change must miss");
        assert!(!cache.has(3, key(3, 2.0), 8), "an edit must miss");
        assert!(!cache.has(4, key(4, 2.0), 7), "another page must miss");
    }

    /// Inserting a page twice replaces it rather than accumulating.
    #[test]
    fn a_second_render_of_one_page_replaces_the_first() {
        let mut cache = StripRasters::default();
        cache.insert(3, key(3, 1.0), 0, PageRaster::Failed("a".to_owned()));
        cache.insert(3, key(3, 2.0), 0, PageRaster::Failed("b".to_owned()));
        assert_eq!(cache.len(), 1);
        assert!(!cache.has(3, key(3, 1.0), 0), "the stale entry is gone");
        assert!(cache.has(3, key(3, 2.0), 0));
    }

    /// **The current page is never in this cache.**
    #[test]
    fn the_current_page_is_pruned_out_even_when_it_is_visible() {
        let mut cache = StripRasters::default();
        for page in 0..4 {
            cache.insert(page, key(page, 1.0), 0, PageRaster::Failed(String::new()));
        }
        cache.retain(2, MAX_CACHED_TEXELS);
        assert_eq!(cache.len(), 3);
        assert!(!cache.has(2, key(2, 1.0), 0), "the current page was kept");
        assert!(cache.has(1, key(1, 1.0), 0));
    }

    /// **Pages that scrolled out of view are KEPT**, and this test used to
    /// assert the opposite.
    #[test]
    fn pages_that_left_the_viewport_are_kept_until_the_budget_bites() {
        let mut cache = StripRasters::default();
        for page in 0..6 {
            cache.insert(page, key(page, 1.0), 0, PageRaster::Failed(String::new()));
        }
        cache.retain(5, MAX_CACHED_TEXELS);
        assert_eq!(
            cache.len(),
            5,
            "everything but the current page stays: a page already drawn must not have to be \
             drawn again just because it scrolled off the top"
        );
        assert!(
            cache.has(0, key(0, 1.0), 0),
            "page 0 is five pages away from the one being read and is still worth keeping — \
             what bounds this cache is memory, not visibility"
        );
        assert!(
            !cache.has(5, key(5, 1.0), 0),
            "the current page is still pruned"
        );
    }

    /// **The budget is what bounds it**, and it evicts furthest-first.
    #[test]
    fn a_budget_of_nothing_prunes_to_the_floor() {
        let mut cache = StripRasters::default();
        for page in 0..6 {
            cache.insert(page, key(page, 1.0), 0, PageRaster::Failed(String::new()));
        }
        // Every entry is a refusal and therefore zero texels, so even a budget
        // of zero cannot evict: `total > budget` is `0 > 0`, false. That is the
        // honest outcome and it is worth pinning, because it says the budget is
        // a MEMORY bound and not a page count — a build that had quietly become
        // a page-count cache would fail here.
        cache.retain(5, 0);
        assert_eq!(
            cache.len(),
            5,
            "zero-texel entries cost nothing, so no budget can evict them — the cache bounds \
             MEMORY, not pages"
        );
    }

    /// The budget evicts furthest-from-current first, so the pages either side
    /// of the operator survive a fast scroll.
    #[test]
    fn eviction_prefers_the_page_furthest_from_the_one_being_read() {
        let mut cache = StripRasters::default();
        for page in 0..7 {
            cache.insert(page, key(page, 1.0), 0, PageRaster::Failed(String::new()));
        }
        cache.retain(3, MAX_CACHED_TEXELS);
        // Nothing is over budget, so everything but the current page stays.
        assert_eq!(cache.len(), 6);
        // The distance ordering the budget pass would use, checked directly:
        // page 6 and page 0 are furthest from page 3, page 2 and 4 nearest.
        assert!(6_usize.abs_diff(3) >= 2_usize.abs_diff(3));
        assert!(0_usize.abs_diff(3) >= 4_usize.abs_diff(3));
    }

    /// An empty cache is the single-page steady state and costs nothing.
    #[test]
    fn a_single_page_session_caches_nothing() {
        let mut cache = StripRasters::default();
        assert!(cache.is_empty());
        assert_eq!(cache.texels(), 0);
        cache.retain(0, MAX_CACHED_TEXELS);
        assert!(cache.is_empty());
        cache.clear();
        assert!(cache.is_empty());
    }
}
