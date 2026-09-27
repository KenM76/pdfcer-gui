//! # `app::state::ink` — what the render tier needs to know about a page's colour
//!
//! The seam: everything else on `OpenDoc` is about the **document** — its
//! session, pages, selection, caches, and the epoch that invalidates them.
//! These methods are about the **renderer**, answering a question
//! `crate::rasterstrategy` asks in that module's own vocabulary. Two subjects
//! with two rates of change, which is this project's test for a seam.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/opendoc/ink.md`.

use super::OpenDoc;

impl OpenDoc {
    /// **What the render tier needs to know about `page`'s colour** — the one
    /// reader of [`Self::ink_pages`].
    #[must_use]
    pub fn ink_at(&self, page: usize) -> crate::rasterstrategy::Ink {
        if self.ink_pages.contains(&page) {
            crate::rasterstrategy::Ink::Subtractive(self.settings.max_cmyk_buffer_bytes)
        } else {
            crate::rasterstrategy::Ink::Additive
        }
    }

    /// **Ask the engine whether `page` composites in ink, once per page per
    /// open document**, so [`Self::ink_at`] has an answer before the first
    /// raster rather than one raster later.
    pub fn learn_ink(&mut self, page: usize) {
        if !self.ink_asked.insert(page) {
            return;
        }
        let Some(subject) = self.pages.get(page) else {
            return;
        };
        let options = crate::settings::SettingsExt::render_options(&self.settings);
        // DESTRUCTURED rather than bound whole, and it is not a style choice.
        // `PageInk` is two facts with two owners - the flag belongs to the
        // render tier, the source belongs to the operator's report - and a
        // `let ink = ...` that reads one field and drops the other is how the
        // second one stays unwired while every gate in this repository stays
        // green. Naming both at the one call site is what makes the pair
        // legible here rather than at two unrelated reader sites.
        //
        // The `..` is REQUIRED: `PageInk` is `#[non_exhaustive]`, so this
        // pattern is NOT a tripwire on the engine growing a third field - it
        // cannot be, by construction. `tools/gates/check-engine-api-drift.sh`
        // is the instrument that catches that, by enumerating every public item
        // at the pinned revision and failing on one this repository names
        // nowhere.
        let pdfcer_render::PageInk {
            composites_in_ink,
            source,
            ..
        } = pdfcer_render::page_composites_in_ink(&self.session.view(), subject, &options);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            //
            // `source.token()`, NEVER the `Debug` derive. The derive spells
            // `PageGroup`; pdfcer's own metrics line spells `page_group`, and
            // two spellings of one fact across a boundary whose whole purpose
            // is that both sides agree means a CLI log and a shell log do not
            // compare. Do not write a local `PageGroup => "page_group"` table
            // either - that is the drift `token()` exists to prevent, and it
            // goes stale in silence the day a further variant arrives.
            //
            // The tokens are IDENTIFIERS, not sentences. Nothing here is shown
            // to an operator - the sentences he reads are this shell's, in
            // `crate::text::diagnostics`, and they are deliberately different
            // words.
            format!(
                "ink-asked page={page} ink={} source={}",
                u8::from(composites_in_ink),
                source.token()
            )
        });
        self.ink_source.insert(page, source);
        if composites_in_ink {
            self.ink_pages.insert(page);
        }
    }
}
