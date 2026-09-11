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

    /// ★★★ **Ask the engine whether `page` composites in ink, once per
    /// page per open document**, so [`Self::ink_at`] has an answer before the
    /// first raster rather than one raster later.
    ///
    /// # What changed, and why the old arrangement was not wrong
    ///
    /// This shell used to learn a page's blending space by **rendering it and
    /// reading the counters afterwards** — `cmyk_buffer_engaged ||
    /// cmyk_buffer_refused`, written in `crate::render::settle`. The engine has
    /// since confirmed that inference is exactly the union it computes
    /// internally, so it was correct; it simply cost a full page raster to
    /// answer a question the page's own `/Group` dictionary answers.
    ///
    /// `Pass 296.4` (`8d2f6bb`, consumed 2026-09-11) made
    /// `page_composites_in_ink` public. It is **the same computation**, not a
    /// second one: it calls the renderer's own `page_blend_space` with the
    /// policy taken out of the `RenderOptions` passed in, and the engine's test
    /// asserts the **agreement** — each fixture is asked and then rendered and
    /// the two are required to match. A pre-flight that can disagree with the
    /// render is worse than no pre-flight, because a caller acts on it.
    ///
    /// # ★★ Why it takes the render options, when [`Self::ink_at`]'s ceiling
    /// # deliberately does not
    ///
    /// These are two different values and the distinction is easy to lose.
    ///
    /// `ink_at` reads `Settings::max_cmyk_buffer_bytes` **directly** rather
    /// than through `SettingsExt::render_options`, because that is a budget and
    /// the question it serves is asked before there is a render to run.
    ///
    /// This function needs the options themselves, because
    /// `page_blend_space_source` is the setting that **changes the answer**:
    /// the same page can composite in ink under one policy and not under
    /// another, and the annotation scope can remove page content from the
    /// question entirely. Passing anything but the options a render would
    /// actually use would reintroduce exactly the disagreement the engine's
    /// agreement test exists to forbid.
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
        // cannot be, by construction. The instrument that catches that is
        // `tools/gates/check-engine-api-drift.sh`, which enumerates every
        // public item at the pinned revision and fails on one this repository
        // names nowhere. Stating which mechanism actually holds the line is the
        // point of this paragraph; a comment claiming the compiler does would
        // be a contract nothing enforces.
        let pdfcer_render::PageInk {
            composites_in_ink,
            source,
            ..
        } = pdfcer_render::page_composites_in_ink(&self.session.view(), subject, &options);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            //
            // `source.token()`, NOT the `Debug` derive, and the difference is
            // the whole reason this call exists. The derive spells
            // `PageGroup`; pdfcer's own metrics line spells `page_group`. Two
            // stable spellings of one fact across a boundary whose entire
            // purpose is that both sides agree, which means a log from the CLI
            // and a log from this shell would not compare.
            //
            // This trace DID take the derive, for about an hour on
            // 2026-09-11, because `token()` was `pub(crate)`. The alternative
            // was a `PageGroup => "page_group"` table written here - the exact
            // drift `token()` exists to prevent, and one that goes stale in
            // silence the day a fourth variant arrives (R74). So it was
            // reported under decision 058 instead, and `Pass 296.8`
            // (`f392b19`, consumed 2026-09-11) published the function. The
            // three tokens are a contract now: a variant may be added, an
            // existing spelling may not change without that being breaking.
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
