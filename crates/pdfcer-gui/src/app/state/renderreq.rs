//! # `app::state::renderreq` — what this view is asking the renderer FOR
//!
//! ## The seam
//!
//! `state.rs` answers *what is open and how it is being looked at*: the
//! document, the pages, the zoom, the scroll, the selection, the epochs. This
//! file answers a narrower question that hangs off the end of it — **given all
//! of that, what exactly do we hand to `pdfcer-render`, and how do we know
//! whether the picture we already have is still a picture of it?**
//!
//! Four functions, and they are one subject:
//!
//! | function | question |
//! |---|---|
//! | [`OpenDoc::render_key`] | is the texture I am holding still true? |
//! | [`OpenDoc::render_key_for`] | …for a page that is not the current one |
//! | [`OpenDoc::region_for`] | which part of that page is worth rasterizing? |
//! | [`OpenDoc::render_request_for`] | the whole order, ready for the worker |
//!
//! They belong together because a change to one is nearly always a change to
//! the others: a new input that affects the picture has to enter the **key**
//! (or a stale texture survives an edit) *and* the **request** (or the new
//! picture is drawn without it). Splitting those two across files is how a
//! toggle ships that visibly does nothing — this crate has had exactly that
//! defect, and `crate::panels::layers`' own test still names which of its
//! preconditions remains open.
//!
//! ## Why it moved, 2026-09-10
//!
//! R2. `state.rs` reached 1,580 lines when [`OpenDoc::load_options`] landed for
//! the duplicate-key re-read, and `tools/gates/check-file-size.sh`'s header says
//! what to do about that: *"Split the module along its seams — one subject per
//! file — rather than raising the limit."*
//!
//! This is the seam that was already there. Everything else in that `impl` is
//! about the document or the operator's view of it; these four are about an
//! **order placed with another crate**, and they are the only members that name
//! `RenderKey`, `RenderRequest` or a raster scale.
//!
//! ## What it does NOT contain, deliberately
//!
//! [`OpenDoc::strip`] stayed behind. A strip is where the pages *are* — a
//! layout fact the whole frame agrees on, used by hit-testing and scrolling as
//! much as by drawing — and it names no render type. It sits directly under
//! these four in `state.rs` and the resemblance is superficial.

use std::sync::Arc;

use super::OpenDoc;
use crate::render::worker::{RenderKey, RenderRequest};

impl OpenDoc {
    /// What a render of the current view would be *of*.
    ///
    /// The staleness key the shell wants, built from the same constructor the
    /// worker labels its output with — see
    /// [`crate::render::worker::RenderKey::new`]. One arithmetic path, so
    /// "what I want" and "what I have" cannot disagree about how a key is
    /// spelled.
    ///
    /// `pub(crate)` for one reason: a panel whose control is blocked on
    /// something else needs to be able to assert that *its* input reaches the
    /// key. `crate::panels::layers` does exactly that — see
    /// `the_render_key_no_longer_blocks_a_layer_toggle` — which is how the
    /// next person to restore that checkbox learns which of its three
    /// preconditions is still open without re-deriving the answer.
    pub(crate) fn render_key(&self, raster_scale: f32) -> RenderKey {
        self.render_key_for(self.view.page_index, raster_scale)
    }

    /// What a render of **any** page in this view's current settings would be
    /// *of*.
    ///
    /// [`Self::render_key`]'s general form, and the one a continuous strip
    /// needs: every visible page is rendered with the same scale, annotation
    /// stance and layer override, so the only thing that varies between them
    /// is the page index. Written as one function with the current page as a
    /// special case, rather than two, because two would be two places for the
    /// annotation stance to be forgotten — and a key that omitted it would
    /// leave the strip's pages showing annotations after the operator turned
    /// them off, while the current page obeyed.
    pub(crate) fn render_key_for(&self, page_index: usize, raster_scale: f32) -> RenderKey {
        RenderKey::new(
            page_index,
            raster_scale,
            self.annotations,
            self.layers.generation,
            // ★★★ O137. Through `ViewState::stroke_display`, which is also what
            // `render_request_for` hands the worker — one conversion, so "what
            // I want" and "what I have" cannot disagree about whether line
            // weights were on. Omit it and the toggle looks inert: the cache
            // reports a hit and serves the picture drawn under the other
            // answer.
            self.view.stroke_display(),
        )
        .with_region(self.region_for(page_index))
    }

    /// The region to rasterize for `page_index`, if the canvas set one **for
    /// that page**.
    ///
    /// ★ The page check is the whole of this method's job. Without it a
    /// region computed for page 4 would be applied to page 5 as well, and
    /// both rectangles are valid — so the wrong part of the neighbour would
    /// be rasterized with nothing reporting an error.
    #[must_use]
    pub(crate) fn region_for(&self, page_index: usize) -> Option<pdfcer_core::page_tree::Rect> {
        self.raster_region
            .filter(|(page, _)| *page == page_index)
            .map(|(_, rect)| rect)
    }

    /// Everything a worker needs to rasterize `page_index`, or `None` if there
    /// is no such page.
    ///
    /// The one constructor for a [`RenderRequest`], so the current page and a
    /// strip page cannot be rendered with different options. It exists here,
    /// on the document, rather than in [`crate::render::settle`] because the
    /// annotation stance and the layer override are **private** fields of this
    /// type — and they should stay private: they are changed through
    /// [`Self::set_annotations_visible`] and [`Self::set_hidden_layers`],
    /// which are the methods that keep the staleness keys moving.
    pub(crate) fn render_request_for(
        &self,
        page_index: usize,
        raster_scale: f32,
    ) -> Option<RenderRequest> {
        let page = self.pages.get(page_index)?;
        Some(RenderRequest {
            // ★ O24's region tier, live since 2026-08-22. `None` below the
            // pixmap ceiling — which is every zoom that can render whole-page,
            // so panning there is unchanged — and `Some` above it, where the
            // alternative is the operator's `MAX_PIXMAP_EDGE` failure.
            region: self.region_for(page_index),
            // The `Arc` is handed over rather than a `DocumentView`, which is
            // what lets the borrow stay local to the worker thread.
            session: Arc::clone(&self.session),
            page: page.clone(),
            page_index,
            raster_scale,
            annotations: self.annotations,
            // ★★★ O137 — `view.line_weights`, canvas only. The same conversion
            // the key above uses; see `ViewState::stroke_display` for why it is
            // a function and not two `if`s. `render_on_worker` is the only
            // place this is read, and no export or print path builds a
            // `RenderRequest` at all.
            stroke_display: self.view.stroke_display(),
            layers: self.layer_visibility(),
            layers_generation: self.layers.generation,
            // ★ The SNAPSHOT, not a live read — see the field's own docs.
            //
            // The worker runs on another thread and may finish after the
            // operator has changed a setting, so what it must be given is the
            // configuration this document's caches are keyed to. Handing it a
            // live value would produce a texture drawn under settings that no
            // cached neighbour shares, and nothing would notice: the render key
            // does not carry the settings, because `adopt_settings` drops every
            // cache instead, which is the more direct mechanism and the visible
            // one.
            //
            // Cloned rather than shared: one `String` and twelve `Copy` fields,
            // paid once per render request, against a rasterization measured in
            // tens of milliseconds.
            settings: self.settings.clone(),
        })
    }
}
