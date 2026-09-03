//! # `app::settings_window` — the four things pressing a button in Settings does
//!
//! The host half of [`crate::dialogs::settings`]. That module renders and
//! returns an [`Outcome`]; this one decides what an outcome *means*, and every
//! one of the four consequences below is a decision rather than plumbing.
//!
//! Kept out of `app/dispatch.rs` because it is not a command dispatch: the
//! Settings window raises its verbs from its own buttons, not from the ribbon,
//! and folding them into the command `match` would put three arms there that no
//! command id can reach.

use egui_shell::theme::Preset;
use pdfcer_core::settings::Settings;

use super::PdfcerApp;
use crate::dialogs::settings::{self, Outcome};

impl PdfcerApp {
    /// Draw the Settings window if it is open, and act on what was pressed.
    ///
    /// # The ✕ is a Cancel, and that is a contract rather than a convenience
    ///
    /// `egui::Window::open` gives the title bar a close button that this code
    /// does not otherwise see. If closing by ✕ did anything different from
    /// closing by Cancel — kept the draft, half-applied it, saved it — the
    /// window would have two exits with two meanings and no way to tell which
    /// one an operator took. Both paths drop the draft.
    pub(super) fn settings_window(&mut self, ctx: &egui::Context) {
        let Some(draft) = self.settings_draft.as_mut() else {
            return;
        };

        let mut open = true;
        let outcome = settings::show(ctx, draft, &self.settings_store, &mut open);

        match outcome {
            Outcome::Save => self.save_settings(),
            Outcome::Cancel => {
                // Nothing else. The theme reverts by itself, because the
                // per-frame token lookup falls back to `self.settings.theme`
                // the moment the draft is gone.
                self.settings_draft = None;
            }
            Outcome::RestoreDefaults => {
                // ★ Replaces the DRAFT only, and does not save or close.
                //
                // "Restore defaults" is not the kind of button that should be
                // able to discard a configuration in one click with no way
                // back — and every other program that makes it immediate has
                // taught operators to expect that it is, which is exactly why
                // this one must not be. The operator still has to press Save,
                // and still has Cancel.
                draft.working = Settings::default();
                // Both stores, because the operator pressed one button. See
                // `Draft::working_prefs`.
                draft.working_prefs = crate::app::prefs::Prefs::default();
            }
            Outcome::Idle => {}
        }

        if !open {
            self.settings_draft = None;
        }
    }

    /// Adopt the working copy, write it, and make the change visible.
    ///
    /// Four steps, in this order, and the order carries two of the decisions.
    ///
    /// # 1. Adopt FIRST, then persist
    ///
    /// `self.settings = draft.working` happens before the write is attempted,
    /// so a disk that refuses does not cost the operator the choice they just
    /// made. The session behaves as they asked **while telling them it will not
    /// survive a restart**.
    ///
    /// Adopting only on a successful write is the obvious alternative and is
    /// worse: it silently ignores a deliberate choice, and the operator's only
    /// evidence would be that nothing changed — which reads as the setting not
    /// working rather than as the file not being writable.
    ///
    /// # 2. Save closes the window
    ///
    /// `take()` rather than a borrow. There is no *Apply*: a window with Apply
    /// has a third state — saved, unsaved, and saved-but-still-open-with-more-
    /// edits — and this one is short enough that Save-and-close costs nothing.
    ///
    /// # 3. ★ Every cached raster is invalidated
    ///
    /// This is the step whose absence would be a defect, and a confusing one.
    /// Five of the thirteen settings change how a page **renders** —
    /// `cmyk_intent`, `mask_resample`, `image_minify`, `cmyk_jpeg_polarity`,
    /// `missing_as` — and the canvas texture and the thumbnail rail were both
    /// produced under the old values. Without an invalidation the operator
    /// changes how black is drawn, presses Save, and **nothing on screen
    /// moves** until something else happens to dirty the cache.
    ///
    /// That reads as the setting not working. It is the exact failure mode the
    /// old shell's dispatcher guarded against with the same call, and its note
    /// on the point is worth keeping: use the established funnel rather than
    /// reaching into the two caches by hand, because a third cache added later
    /// joins the funnel and does not join a pair of hand-written clears.
    ///
    /// # 4. A failure is reported in the status bar, never in a dialog
    ///
    /// A save that **failed** says so, because the operator asked for something
    /// to be remembered and is owed the truth if it was not. Not modally: a
    /// configuration problem may not interrupt the document they are working
    /// on, and it may certainly not stop them opening a file.
    ///
    /// A save that **worked** says nothing, which is this shell's standing
    /// convention rather than an omission here. The operator pressed a button
    /// in a window they were looking at and the window closed; narrating that
    /// back to them trains them to stop reading a bar whose only other job is
    /// to carry the failures.
    fn save_settings(&mut self) {
        let Some(draft) = self.settings_draft.take() else {
            return;
        };

        // 1 + 2. BOTH stores adopted, and both before either write is
        // attempted — the window edits two files and the operator pressed one
        // button, so a half-adopted state would be a state they cannot see and
        // cannot have asked for.
        self.settings = draft.working;
        self.prefs = draft.working_prefs;

        // ★★★ AND THE KEYMAP FOLLOWS THE PASTE-ORDER PREFERENCE — O58.
        //
        // `Prefs` is data; the shell's keymap is what a keystroke actually
        // consults. Adopting the preference without rewriting the binding would
        // give the operator a radio button that saves correctly, reloads
        // correctly, reads correctly in the pane — and changes nothing when they
        // press the key. That is precisely the silently-inert control this
        // project has shipped before, and the reason `wheel_paging` grew its own
        // live-apply branch three screens away in `frame.rs`.
        //
        // ★ Unconditional rather than guarded on a change. `apply_paste_chords`
        // clears both chords and rewrites both, so its result depends only on
        // the preference — running it when nothing changed costs two map
        // operations and removes the need for a before/after snapshot that could
        // itself go stale.
        if let Some(shell) = self.shell.as_mut() {
            crate::shell::manifest::apply_paste_chords(shell, self.prefs.paste_chords);
        }

        // ★ Trace before the write, so a harness can see the adopted values
        // even if the write is what fails. `theme` is named separately because
        // it is the one setting whose effect is already on screen by now.
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "settings-save theme={:?} preset={:?} cmyk={:?} mask={:?} minify={:?} \
                 polarity={:?} unmappable={:?} actual_text={:?} missing_as={:?} \
                 separations={:?} xref_eol={:?} trailing_eol={:?} word_gap={} \
                 parallel_deg={} store={:?}",
                self.settings.theme,
                Preset::from_key(&self.settings.theme),
                self.settings.cmyk_intent,
                self.settings.mask_resample,
                self.settings.image_minify,
                self.settings.cmyk_jpeg_polarity,
                self.settings.unmappable_code,
                self.settings.actual_text,
                self.settings.missing_as,
                self.settings.separations,
                self.settings.xref_entry_eol,
                self.settings.trailing_eol,
                self.settings.word_gap_ratio,
                self.settings.parallel_epsilon_degrees,
                self.settings_store.kind,
            )
        });

        // 4 — the write, and its report.
        //
        // ★ Success is SILENT, and that is this shell's convention rather than
        // an omission. `crate::app::status::decline`'s own note on the
        // save-a-copy path states it: there is deliberately no matching "it
        // worked" call, because the operator pressed a button in a window they
        // were looking at and the window closed. A sentence would narrate what
        // they just did — and a status bar that speaks on every success is one
        // an operator stops reading before the failure arrives.
        //
        // The store's `Display` is a developer's sentence, so it goes to the
        // trace beside the store kind rather than to the bar. The operator's
        // actionable half is *which folder*, and the settings window states
        // that on a line it draws every time it opens.
        match self.settings.save(&self.settings_store) {
            Ok(()) => {
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "settings-saved path={:?}",
                        self.settings_store.path
                    )
                });
            }
            Err(error) => {
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "settings-save-failed store={:?} path={:?} reason={error}",
                        self.settings_store.kind, self.settings_store.path,
                    )
                });
                // Recorded AFTER the adoption above, which is what makes the
                // sentence true. See the variant's own documentation.
                crate::app::status::decline::record_settings_not_saved();
            }
        }

        // ★ The shell's own preferences, written to their own file.
        //
        // Separate from the engine's store and reported separately, because
        // they can fail separately — a settings write can succeed while a
        // preferences write fails, and telling the operator "settings were not
        // saved" when twelve of the fourteen were would be worse than useless.
        //
        // The failure sentence is deliberately the SAME one, though: from the
        // operator's side both are "the choices I made in that window", they
        // pressed one button, and what they need to know is identical — this is
        // in force now and will be gone when pdfcer restarts. Two sentences
        // would be pdfcer explaining its own file layout.
        if let Err(error) = self.prefs.save() {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "prefs-save-failed reason={error}"
                )
            });
            crate::app::status::decline::record_settings_not_saved();
        } else {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "prefs-saved quality={:?} settle_ms={}",
                    self.prefs.render_quality, self.prefs.zoom_settle_ms,
                )
            });
        }

        // 3 — hand the new configuration to the open document and drop
        // everything derived under the old one, LAST, so a failed write still
        // leaves the session rendering under the settings it adopted.
        self.adopt_settings();
    }

    /// ★ **Give the open document the current settings, and drop everything
    /// that was derived under the previous ones.**
    ///
    /// One function, two acts, and they are together on purpose. This is the
    /// only place in the application that writes `OpenDoc::settings`, and it is
    /// also the only place that clears the caches keyed to it — so there is no
    /// state in which a page texture, a strip entry or a page-text cache
    /// disagrees with the snapshot beside them.
    ///
    /// Splitting them is the obvious mistake and would be invisible: update the
    /// snapshot without clearing, and the operator changes how black is drawn,
    /// presses Save, and **nothing on screen moves** until something else
    /// happens to dirty the cache — which reads as the setting not working.
    /// Clear without updating, and every cache immediately refills under the
    /// old configuration, which reads the same way and is harder to find.
    ///
    /// # Called from three places, and the third is the one that is easy to forget
    ///
    /// A settings Save, an open, and a create. The first is obvious; the other
    /// two exist because `OpenDoc::assemble` starts every document on the
    /// *shipped defaults* — it cannot reach `PdfcerApp` — so a document opened
    /// by an operator who has configured anything would otherwise render under
    /// pdfcer's answers rather than theirs.
    ///
    /// `opening_a_document_adopts_the_operators_settings` is the test that
    /// stops a fourth open path being added without this call.
    ///
    /// # Why the caches are cleared wholesale
    ///
    /// Working out which cached sheets a given setting change affects would be
    /// a second statement of what each setting does, in a function that has no
    /// business knowing. They refill from the visible set on the next frame —
    /// the same argument `app::actions::pages` makes when a page permutation
    /// invalidates the strip.
    pub(crate) fn adopt_settings(&mut self) {
        let settings = self.settings.clone();
        let prefs = self.prefs.clone();
        // ★ **Every open document, not only the one on screen** — 2026-08-19,
        // with the document tabs.
        //
        // The snapshot-plus-caches argument above is a property of an
        // `OpenDoc`, not of the active one: a parked document keeps its page
        // texture and its strip cache (`crate::app::documents` §4 says why),
        // so a parked document left un-adopted would be a cached picture drawn
        // under the operator's *previous* colour answers, revealed the moment
        // they clicked its tab. That is the same "a control that reads back
        // what you set and does not do it" failure this function exists to
        // prevent, delayed by one click and therefore harder to attribute.
        //
        // Written as one closure over `status` and every parked slot rather
        // than as two copies of the body, so a fourth thing to invalidate
        // cannot be added to one and missed on the other.
        let adopt_one = |status: &mut crate::app::state::Status| {
            let crate::app::state::Status::Open(doc) = status else {
                return;
            };
            doc.settings = settings.clone();
            // The shell's own preferences ride along, because `render_quality`
            // is baked into a cached texture exactly as the engine's five
            // rendering settings are. Two stores, one snapshot point, one
            // invalidation — see `OpenDoc::prefs`.
            doc.prefs = prefs.clone();
            // Rasters: the current page and every strip entry.
            doc.page_texture = None;
            doc.strip_rasters.clear();
            // Derived text: the page-text cache and everything computed from
            // it. Three settings change what an extraction produces, and one of
            // them can make a whole run vanish — so a stale extraction is not
            // merely differently-spaced, it can be missing content that a
            // find or a redaction-by-pattern would then fail to see.
            doc.invalidate_derived_text();
        };
        adopt_one(&mut self.status);
        for parked in &mut self.parked {
            adopt_one(parked);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::PdfcerApp;
    use crate::app::state::Status;
    use crate::panels::objects::test_support::engine_fixture;
    use pdfcer_core::settings::{CmykIntent, UnmappableCode};

    /// A four-page fixture from the engine's own corpus, as the other
    /// lifecycle tests use.
    const FIXTURE: &str = "pageops/four-pages.pdf";

    /// ★ **Opening a document gives it the operator's settings.**
    ///
    /// The regression test for the one hole this design leaves open.
    /// `OpenDoc::assemble` starts every document on the *shipped defaults* —
    /// it cannot reach `PdfcerApp` — so the snapshot is only ever correct
    /// because `adopt` calls `adopt_settings`. A fourth open path added later
    /// that forgot to would produce a document rendered under pdfcer's answers
    /// while the settings window showed the operator's, correctly, which is a
    /// control that reads back what you set and does not do it.
    ///
    /// The assertion is on a **non-default** value, deliberately. Asserting
    /// that a defaulted app produces a defaulted document would pass with the
    /// call deleted.
    #[test]
    fn opening_a_document_adopts_the_operators_settings() {
        let mut app = PdfcerApp::new();
        app.settings.cmyk_intent = CmykIntent::Calibrated;
        app.settings.unmappable_code = UnmappableCode::Omit;

        app.open_path(engine_fixture(FIXTURE));

        let Status::Open(doc) = &app.status else {
            panic!("the fixture did not open");
        };
        assert_eq!(
            doc.settings.cmyk_intent,
            CmykIntent::Calibrated,
            "the document was opened under pdfcer's own colour answer rather than the operator's"
        );
        assert_eq!(doc.settings.unmappable_code, UnmappableCode::Omit);
    }

    /// ★ **Adopting settings drops everything derived under the old ones.**
    ///
    /// The other half of `adopt_settings`, and the half whose absence would be
    /// invisible: the snapshot updates, the caches do not, and the operator
    /// changes how black is drawn and sees nothing move.
    ///
    /// Asserted on the page-text cache rather than on a raster, because it is
    /// the one this test can fill and observe without a GPU context — and
    /// because it is the more dangerous of the two. Three settings change what
    /// an extraction produces and one of them can make a whole run vanish, so
    /// a stale extraction is not differently spaced, it can be missing content
    /// that a find or a redaction-by-pattern would then fail to see.
    #[test]
    fn adopting_settings_drops_the_text_derived_under_the_old_ones() {
        let mut app = PdfcerApp::new();
        app.open_path(engine_fixture(FIXTURE));

        // Fill the cache, and prove it filled — a test that asserted a cache
        // was empty afterwards would pass on a cache that had never worked.
        let filled = match &app.status {
            Status::Open(doc) => {
                let _ = doc.page_text();
                doc.page_text.built_for.get().is_some()
            }
            _ => panic!("the fixture did not open"),
        };
        assert!(
            filled,
            "the page-text cache never filled, so this proves nothing"
        );

        app.settings.unmappable_code = UnmappableCode::QuestionMark;
        app.adopt_settings();

        match &app.status {
            Status::Open(doc) => {
                assert!(
                    doc.page_text.built_for.get().is_none(),
                    "the extraction cached under the previous settings survived the change"
                );
                assert_eq!(doc.settings.unmappable_code, UnmappableCode::QuestionMark);
            }
            _ => panic!("the document closed itself"),
        }
    }
}
