//! # `opendoc::renderreq` — what this view is asking the renderer FOR
//!
//! `state.rs` answers *what is open and how it is being looked at*. This file
//! answers the narrower question hanging off the end of it: **given all of
//! that, what exactly do we hand to `pdfcer-render`, and how do we know whether
//! the picture we already have is still a picture of it?** These are the only
//! members of `impl OpenDoc` that name `RenderKey`, `RenderRequest` or a raster
//! scale.
//!
//! **The key and the request must change together.** A new input that
//! affects the picture has to enter the **key** — or a stale texture survives an
//! edit — *and* the **request** — or the new picture is drawn without it.
//! Splitting the two across files is how a toggle ships that visibly does
//! nothing; `crate::panels::layers`' own test names which of its preconditions
//! remains open.
//!
//! [`OpenDoc::strip`] deliberately stays in `state.rs`: a strip is where the
//! pages *are*, a layout fact the whole frame agrees on and used by hit-testing
//! and scrolling as much as by drawing, and it names no render type.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/opendoc/renderreq.md`.

use std::sync::Arc;

use super::OpenDoc;
use crate::renderworker::{RenderKey, RenderRequest};

impl OpenDoc {
    /// What a render of the current view would be *of*.
    pub fn render_key(&self, raster_scale: f32) -> RenderKey {
        self.render_key_for(self.view.page_index, raster_scale)
    }

    /// What a render of **any** page in this view's current settings would be
    /// *of*.
    pub fn render_key_for(&self, page_index: usize, raster_scale: f32) -> RenderKey {
        RenderKey::new(
            page_index,
            raster_scale,
            self.annotations,
            self.layers.generation,
            // O137. Through `ViewState::stroke_display`, which is also what
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
    #[must_use]
    pub fn region_for(&self, page_index: usize) -> Option<pdfcer_core::page_tree::Rect> {
        self.raster_region
            .filter(|(page, _)| *page == page_index)
            .map(|(_, rect)| rect)
    }

    /// Everything a worker needs to rasterize `page_index`, or `None` if there
    /// is no such page.
    pub fn render_request_for(
        &self,
        page_index: usize,
        raster_scale: f32,
    ) -> Option<RenderRequest> {
        let page = self.pages.get(page_index)?;
        Some(RenderRequest {
            // O24's region tier. `None` below the pixmap ceiling — which is
            // every zoom that can render whole-page, so panning there is
            // unchanged — and `Some` above it, where the alternative is the
            // operator's `MAX_PIXMAP_EDGE` failure.
            region: self.region_for(page_index),
            // The `Arc` is handed over rather than a `DocumentView`, which is
            // what lets the borrow stay local to the worker thread.
            session: Arc::clone(&self.session),
            page: page.clone(),
            page_index,
            raster_scale,
            annotations: self.annotations,
            // O137 — `view.line_weights`, canvas only. The same conversion
            // the key above uses; see `ViewState::stroke_display` for why it is
            // a function and not two `if`s. `render_on_worker` is the only
            // place this is read, and no export or print path builds a
            // `RenderRequest` at all.
            stroke_display: self.view.stroke_display(),
            layers: self.layer_visibility(),
            layers_generation: self.layers.generation,
            // The SNAPSHOT, not a live read — see the field's own docs.
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
            // Cloned rather than shared: a flat record of scalars and one
            // `String`, paid once per render request, against a rasterization
            // measured in tens of milliseconds.
            settings: self.settings.clone(),
        })
    }
}
