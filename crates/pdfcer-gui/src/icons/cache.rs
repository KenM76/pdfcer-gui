//! # icons::cache — one raster per (icon, physical size, weight), memoized
//!
//! The ribbon re-runs every frame (60/s), and rasterizing thirty icons per
//! frame would be absurd. [`IconCache`] memoizes the uploaded
//! [`egui::TextureHandle`] per [`CacheKey`], so nothing is rasterized twice
//! unless the display scale changes or a control becomes selected.
//!
//! ## Why the tint is not in the key
//!
//! Because it cannot be: the raster is a white coverage mask (see
//! [`super::svg`], "Theming") and the colour arrives at draw time. Putting
//! the tint in the key would multiply the cache by the number of theme
//! presets times the number of widget states for *exactly zero* benefit,
//! and — worse — would mean a hover produced a texture upload. The cache is
//! keyed on the three things that genuinely change the pixels: which icon,
//! how many physical pixels a side, and how heavy the stroke is.
//!
//! `super::tests::cache_serves_repeat_requests_without_re_rasterizing`
//! asserts the hit path, and
//! `super::tests::cache_re_rasterizes_for_a_different_size_or_weight`
//! asserts the miss path. Both matter: a cache that never misses is a cache
//! that shows a stale, wrongly-sized glyph after the window moves to a
//! 150% monitor, which is the exact blur this pipeline exists to prevent.
//!
//! ## Why it is a thread-local rather than application state
//!
//! The seam this module ultimately serves is `egui_shell`'s
//! `IconPainter` — a `dyn FnMut(&egui::Painter, &IconRequest)`. The painter
//! is handed a `Painter` **precisely so that it cannot allocate layout**,
//! and it is invoked from deep inside a button that is already being laid
//! out. There is nowhere in that call chain to thread a `&mut` cache from,
//! short of making every application that supplies a painter own one and
//! close over it — which would push a pure-memoization detail into the
//! shell's public contract.
//!
//! An interior-mutable per-thread cache sidesteps it with no behavioural
//! cost, because the cache is pure memoization: evicting it changes
//! performance, never pixels. eframe runs the UI on a single thread, so
//! per-thread is per-app in practice, and a second thread would simply get
//! its own (unused) cache rather than a data race.
//!
//! [`IconCache`] itself is public and independently constructible so the
//! caching contract is unit-testable without going through the
//! thread-local.

use std::cell::RefCell;
use std::collections::HashMap;

use super::svg::{IconArt, blank_image};
use super::{Icon, IconWeight};

/// Hard cap on live cache entries before the whole cache is dropped.
///
/// The cache grows only along three axes — one entry per icon, per weight,
/// per distinct physical size the display scale has taken this session —
/// so in normal use it settles well under this and never reaches it. The cap
/// exists solely so that a session that repeatedly changes display scale
/// (dragging a window between a 100% and a 150% monitor) cannot accumulate
/// stale textures without bound.
///
/// # The arithmetic this number has to satisfy, and does not
///
/// One entry per icon per weight per distinct physical size. `Icon::ALL` holds
/// **143** icons and [`IconWeight`] has two variants, so **one** display scale
/// is already 286 entries and two is 572 — past this cap. **Re-derive this
/// number whenever the icon set grows**; it is `Icon::ALL.len() * 2 * (the
/// number of display scales a session should hold without churning)`.
///
/// Exceeding it is not a crash and not a wrong pixel, which is why it is worth
/// writing down: the cache clears wholesale and re-rasterizes the entire
/// visible ribbon, repeatedly, on any machine whose window is dragged between
/// two monitors of different scale. A hitch, blamed on the renderer, caused by
/// a constant nobody re-derived when the set changed size.
///
/// Clearing wholesale rather than evicting least-recently-used is
/// deliberate: it is one line, it happens approximately never, and the
/// recovery cost is one frame of re-rasterization.
const CACHE_CAPACITY: usize = 512;

/// What uniquely identifies a raster.
///
/// The tint is deliberately absent — see this module's header.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CacheKey {
    /// Which glyph.
    pub icon: Icon,
    /// Physical pixels a side — the DPI-resolved size, not the logical one.
    pub px: u32,
    /// How heavily the outline is stroked.
    pub weight: IconWeight,
}

/// Memoized icon textures.
#[derive(Default)]
pub struct IconCache {
    textures: HashMap<CacheKey, egui::TextureHandle>,
    /// How many times an asset has actually been parsed and rasterized.
    ///
    /// Nothing in the renderer reads this. It exists because "the cache is
    /// doing its job" is otherwise only *plausible* — a test can compare two
    /// `TextureId`s and see they match without ever proving the second
    /// request skipped the work. This counter is what makes it an assertion.
    rasterizations: usize,
}

impl IconCache {
    /// Fetch (or build) the texture for one icon at one physical size and
    /// weight.
    ///
    /// # What happens when the asset is broken
    ///
    /// It degrades to a 1×1 transparent texture and a one-line stderr
    /// complaint rather than panicking. The assets are compiled-in
    /// constants, so a failure here means the *build* shipped a broken one —
    /// a condition `super::tests::every_icon_parses` is designed to catch
    /// first — and taking down an editor holding the operator's unsaved
    /// edits over a missing 16 px glyph would be a far worse outcome than a
    /// blank slot with an intact tooltip and accessible name.
    ///
    /// Note the asymmetry with an **unknown key**, which is a different
    /// failure with a different answer: see [`super::paint_ribbon_icon`].
    /// A broken asset is a build defect that the test gate catches before an
    /// operator ever sees it, and the stderr line is addressed to the
    /// developer who broke it. An unknown key can reach a real operator
    /// (a command naming an icon the set does not have), so it is drawn
    /// visibly instead of silently.
    pub fn texture(
        &mut self,
        ctx: &egui::Context,
        icon: Icon,
        px: u32,
        weight: IconWeight,
    ) -> egui::TextureHandle {
        let key = CacheKey { icon, px, weight };
        if let Some(handle) = self.textures.get(&key) {
            return handle.clone();
        }
        if self.textures.len() >= CACHE_CAPACITY {
            self.textures.clear();
        }

        let image = match IconArt::parse(icon.source()) {
            Ok(art) => art.rasterize(px, weight),
            Err(err) => {
                eprintln!(
                    // ui-text-exempt: stderr diagnostic, never rendered in the GUI. An icon is a
                    // compiled-in constant, so this fires only when a DEVELOPER has committed a
                    // malformed asset — it is a build-time defect report addressed to whoever
                    // broke it, not operator copy. The operator-visible consequence is the blank
                    // image below, which is deliberately silent: pdfcer never invents a look for
                    // something it could not draw.
                    "pdfcer: icon asset '{}' failed to parse ({err}); drawing nothing",
                    icon.name()
                );
                blank_image()
            }
        };
        self.rasterizations += 1;

        // ui-text-exempt: an egui texture debug name, visible only in a
        // texture inspector. See `Icon::name`, job 2.
        let name = format!("icon:{}@{px}:{weight:?}", icon.name());
        // LINEAR filtering: the raster is produced at the exact physical size
        // it will be drawn at, so filtering is a no-op in the normal case —
        // but if egui ever draws it at a fractional offset, linear is the
        // difference between a soft edge and a shimmering one.
        // Recorded before the upload is ordered — see
        // `crate::render::pressure`. The icon sheet can never be the cause of
        // a zoom-dependent failure, and that is precisely why it is counted:
        // an unrecorded upload makes a two-upload frame look like a one-upload
        // frame, and the page raster left standing gets blamed for this.
        crate::render::pressure::record_other(
            ctx,
            crate::render::pressure::Surface::Icon,
            u32::try_from(image.width()).unwrap_or(u32::MAX),
            u32::try_from(image.height()).unwrap_or(u32::MAX),
        );
        let handle = ctx.load_texture(name, image, egui::TextureOptions::LINEAR);
        self.textures.insert(key, handle.clone());
        handle
    }

    /// How many rasterizations have happened — the cache's testable
    /// observable (see the `rasterizations` field docs).
    #[must_use]
    pub fn rasterizations(&self) -> usize {
        self.rasterizations
    }

    /// How many distinct textures are currently held.
    #[must_use]
    pub fn len(&self) -> usize {
        self.textures.len()
    }

    /// Whether nothing has been cached yet.
    ///
    /// Present because a public `len()` without an `is_empty()` is an API
    /// guidelines violation, not because anything needs it.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.textures.is_empty()
    }
}

thread_local! {
    /// The process-wide (in practice, UI-thread-wide) icon cache.
    ///
    /// See this module's header for why this is a thread-local rather than
    /// a field on the application struct.
    static CACHE: RefCell<IconCache> = RefCell::new(IconCache::default());
}

/// Run `f` with the shared cache.
///
/// Kept private so no caller can hold the `RefMut` across a re-entrant call
/// (which would panic); every entry point in [`super`] borrows, does one
/// lookup, and releases before it draws anything.
pub(super) fn with_cache<R>(f: impl FnOnce(&mut IconCache) -> R) -> R {
    CACHE.with(|c| f(&mut c.borrow_mut()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The load-bearing property: the ribbon asks for the same icon every
    /// frame, and only the FIRST ask may rasterize.
    ///
    /// Tint is not part of the key by design (mask + tint), so re-asking
    /// while the theme, the hover state or the enabled state has changed is
    /// still a cache hit — which is why this loop does not vary anything.
    /// There is nothing to vary: none of it reaches this layer.
    #[test]
    fn cache_serves_repeat_requests_without_re_rasterizing() {
        let ctx = egui::Context::default();
        let mut cache = IconCache::default();
        assert!(cache.is_empty());

        let first = cache.texture(&ctx, Icon::Undo, 16, IconWeight::Regular);
        assert_eq!(cache.rasterizations(), 1);
        assert_eq!(cache.len(), 1);

        // Same icon+size+weight, 60 more times — one simulated second of
        // ribbon frames.
        for _ in 0..60 {
            let again = cache.texture(&ctx, Icon::Undo, 16, IconWeight::Regular);
            assert_eq!(again.id(), first.id());
        }
        assert_eq!(cache.rasterizations(), 1, "cache re-rasterized on a hit");
        assert_eq!(cache.len(), 1);
    }

    /// A display-scale change changes the physical size, and that MUST
    /// produce a new raster — reusing the old one is exactly the
    /// blurry-on-HiDPI bug this pipeline exists to avoid.
    #[test]
    fn cache_re_rasterizes_for_a_different_size_or_weight() {
        let ctx = egui::Context::default();
        let mut cache = IconCache::default();

        let small = cache.texture(&ctx, Icon::Undo, 16, IconWeight::Regular);
        let large = cache.texture(&ctx, Icon::Undo, 32, IconWeight::Regular);
        assert_ne!(small.id(), large.id(), "different size reused a texture");
        assert_eq!(cache.rasterizations(), 2);

        let bold = cache.texture(&ctx, Icon::Undo, 16, IconWeight::Bold);
        assert_ne!(small.id(), bold.id(), "different weight reused a texture");
        assert_eq!(cache.rasterizations(), 3);
        assert_eq!(cache.len(), 3);

        // A different icon at an already-cached size is still a miss.
        let _ = cache.texture(&ctx, Icon::Redo, 16, IconWeight::Regular);
        assert_eq!(cache.rasterizations(), 4);
    }

    /// Every icon must survive a full round trip through the real cache,
    /// which is what the ribbon actually calls.
    #[test]
    fn cache_handles_the_whole_catalogue() {
        let ctx = egui::Context::default();
        let mut cache = IconCache::default();
        for &icon in Icon::ALL {
            let _ = cache.texture(&ctx, icon, 16, IconWeight::Regular);
        }
        // Open and FontFolders share one asset but are distinct cache entries
        // (distinct keys), which is intended and cheap.
        assert_eq!(cache.len(), Icon::ALL.len());
        assert_eq!(cache.rasterizations(), Icon::ALL.len());
    }

    /// The capacity guard drops everything rather than growing without
    /// bound, and the cache keeps working afterwards.
    ///
    /// Reached by asking for more distinct *sizes* than the cap, which is
    /// the only axis an operator can actually drive without bound (drag a
    /// window between monitors of different scale, repeatedly).
    #[test]
    fn the_capacity_guard_clears_rather_than_growing_without_bound() {
        let ctx = egui::Context::default();
        let mut cache = IconCache::default();
        for px in 1..=(CACHE_CAPACITY as u32 + 4) {
            let _ = cache.texture(&ctx, Icon::Undo, px, IconWeight::Regular);
        }
        assert!(
            cache.len() <= CACHE_CAPACITY,
            "cache grew past its cap: {}",
            cache.len()
        );
        // And it still serves: the recovery cost is one re-rasterization,
        // not a broken cache.
        let before = cache.rasterizations();
        let a = cache.texture(&ctx, Icon::Undo, 999, IconWeight::Regular);
        let b = cache.texture(&ctx, Icon::Undo, 999, IconWeight::Regular);
        assert_eq!(a.id(), b.id());
        assert_eq!(cache.rasterizations(), before + 1);
    }

    /// The shared thread-local is reachable and behaves like the standalone
    /// one — the standalone `IconCache` exists for testability, and a test
    /// that only ever exercised it would prove nothing about the cache the
    /// application actually uses.
    #[test]
    fn the_shared_cache_memoizes_too() {
        let ctx = egui::Context::default();
        let first = with_cache(|c| c.texture(&ctx, Icon::Stamp, 21, IconWeight::Regular));
        let before = with_cache(|c| c.rasterizations());
        let again = with_cache(|c| c.texture(&ctx, Icon::Stamp, 21, IconWeight::Regular));
        assert_eq!(first.id(), again.id());
        assert_eq!(with_cache(|c| c.rasterizations()), before);
    }
}
