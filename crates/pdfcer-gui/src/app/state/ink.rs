//! # `app::state::ink` — **what the render tier needs to know about a page's
//! # colour**
//!
//! One method, split out of [`super`] on 2026-09-01 under R2 when the
//! form-edit counter (`OPERATOR_REQUESTS.md` O70 / the decomposition cache key)
//! took that file to 1,524 lines.
//!
//! ## ★ Why this is the seam
//!
//! Everything else on `OpenDoc` is about the **document**: its session, its
//! pages, its selection, its caches, the epoch that invalidates them. This is
//! about the **renderer** — it answers a question `crate::render::strategy`
//! asks, in that module's own vocabulary, and it is the only method on the type
//! whose caller is the raster tier rather than a surface.
//!
//! ⇒ Two subjects with two rates of change, which is this project's test for a
//! seam. `state.rs` changes when the document model does; this changes when the
//! ink strategy does, and it has not changed since it was written.

use super::OpenDoc;

impl OpenDoc {
    /// ★ **What the render tier needs to know about `page`'s colour** — the one
    /// reader of [`Self::ink_pages`].
    ///
    /// Two facts in one value, because they are only ever wanted together and
    /// separating them would let a call site pair the observation with the
    /// wrong ceiling: *has this page been seen compositing in ink*, and *what
    /// ceiling has the operator set*. See [`crate::render::strategy::Ink`].
    ///
    /// The ceiling is `Settings::max_cmyk_buffer_bytes` read **through the
    /// document's own settings**, which is where the settings window's Apply
    /// lands — so raising it in the window moves the tier on the next frame,
    /// with no reopen and no second copy of the value anywhere.
    ///
    /// It is NOT taken from `SettingsExt::render_options`, which would be the
    /// tempting spelling: that builder produces the options a *render* is run
    /// with, and this question is asked before there is a render to run.
    #[must_use]
    pub fn ink_at(&self, page: usize) -> crate::render::strategy::Ink {
        if self.ink_pages.contains(&page) {
            crate::render::strategy::Ink::Subtractive(self.settings.max_cmyk_buffer_bytes)
        } else {
            crate::render::strategy::Ink::Additive
        }
    }
}
