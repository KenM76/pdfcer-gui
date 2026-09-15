//! # `app::state::ink` — what the render tier needs to know about a page's colour
//!
//! The seam: everything else on `OpenDoc` is about the **document** — its
//! session, pages, selection, caches, and the epoch that invalidates them.
//! These methods are about the **renderer**, answering a question
//! `crate::render::strategy` asks in that module's own vocabulary. Two subjects
//! with two rates of change, which is this project's test for a seam.

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

    /// **Ask the engine whether `page` composites in ink, once per page per
    /// open document**, so [`Self::ink_at`] has an answer before the first
    /// raster rather than one raster later.
    ///
    /// `pdfcer_render::page_composites_in_ink` is the **same computation** the
    /// renderer performs, not a second one — the engine's own test asks each
    /// fixture and then renders it and requires the two to agree. A pre-flight
    /// that can disagree with the render is worse than no pre-flight, because a
    /// caller acts on it.
    ///
    /// # ★★ Why this takes the render options when [`Self::ink_at`] does not
    ///
    /// `ink_at` reads `Settings::max_cmyk_buffer_bytes` directly: that is a
    /// **budget**, and its question is asked before there is a render to run.
    /// This function needs the options themselves, because
    /// `page_blend_space_source` is the setting that **changes the answer** —
    /// the same page can composite in ink under one policy and not under
    /// another, and the annotation scope can remove page content from the
    /// question entirely. Passing anything but the options a render would
    /// actually use reintroduces exactly the disagreement the engine's
    /// agreement test forbids.
    ///
    /// # It is the SPACE question, not the BUDGET question
    ///
    /// `composites_in_ink` is *the page asked for ink*, and it survives a
    /// change of zoom. Whether the colorant buffer is actually engaged is the
    /// second half, `will_composite_in_cmyk(w, h, budget)`, and that lives in
    /// [`crate::render::strategy::for_page`] where the pixel dimensions are.
    /// This function deliberately takes no dimensions.
    ///
    /// # It records BOTH halves of the answer
    ///
    /// `composites_in_ink` goes to [`Self::ink_pages`] and is what the render
    /// tier reads. `source` goes to [`Self::ink_source`] and is what the
    /// *operator* reads, in the render report - it is the difference between
    /// *this sheet declares a subtractive page group* and *this sheet declares
    /// nothing and the file's output intent decided for it*, and the second is
    /// the case Settings - Colour's `page_blend_space_source` exists to govern.
    /// A shell that stored only the flag would be throwing away the fact that
    /// makes that control legible.
    ///
    /// # Cost
    ///
    /// One page dictionary read, once per page, for the life of the open
    /// document — [`Self::ink_asked`] is why "no" is remembered as firmly as
    /// "yes". Called from the tier decision, which runs every frame, so the
    /// set is load-bearing rather than an optimisation.
    pub fn learn_ink(&mut self, page: usize) {
        if !self.ink_asked.insert(page) {
            return;
        }
        let Some(subject) = self.pages.get(page) else {
            return;
        };
        let options = crate::app::settings::SettingsExt::render_options(&self.settings);
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
            // ★ `source.token()`, NEVER the `Debug` derive. The derive spells
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
