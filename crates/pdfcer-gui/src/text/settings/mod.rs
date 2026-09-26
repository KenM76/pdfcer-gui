//! # `text::settings` — every word the Settings window shows
//!
//! The catalog area for [`crate::dialogs::settings`]. Ported from the old
//! shell's `ui_text.rs`, where these strings occupied roughly 700 lines in the
//! middle of a 7,912-line file.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/settings/mod.md`.

pub mod bytes;
pub mod extract;
pub mod look;
/// The colour the recognised text is drawn in over a scan — O229. Its own
/// file because it is the copy for a *feature*, where this module's neighbours
/// are copy for answers to a silent standard.
pub mod ocrlayer;
pub mod overprint;
/// The two print-ready colour controls and the field wash. Its header says
/// which of the three is there for a weak reason and should move out first if
/// the module grows.
pub mod print_colour;
pub mod redaction;
pub mod shell;

pub use bytes::*;
pub use extract::*;
pub use look::*;
pub use ocrlayer::*;
pub use overprint::*;
pub use print_colour::*;
pub use redaction::*;
pub use shell::*;

use egui_shell::theme::Preset;
use pdfcer_core::settings::StoreKind;
use pdfcer_core::settings::StoreLocation;

// ===========================================================================
// Window chrome
// ===========================================================================

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Settings"
}

/// The paragraph under the title.
#[must_use]
pub const fn intro() -> &'static str {
    "The PDF standard leaves some things genuinely undefined, so different \
     programs can open the same file and be equally correct while showing you \
     different results. Where that happens, pdfcer asks you rather than deciding \
     quietly. Each choice below says what the standard does not settle, what \
     pdfcer ships as its answer and why, and what changing it affects."
}

/// Where the settings file lives, said in the operator's terms.
#[must_use]
pub fn store_location(store: &StoreLocation) -> String {
    match (store.kind, store.path.as_deref()) {
        (StoreKind::Portable, Some(path)) => format!(
            "Kept in {} — this folder is yours. When you update pdfcer by replacing \
             the program files, keep it.",
            path.display()
        ),
        (StoreKind::Portable, None) => "Your choices are kept beside the program.".to_owned(),
        (StoreKind::PlatformFallback, Some(path)) => format!(
            "Kept in {} because pdfcer's own folder is not writable. These choices \
             will NOT travel with the program folder if you move or copy it.",
            path.display()
        ),
        (StoreKind::PlatformFallback, None) => {
            "Kept in your system settings folder, because pdfcer's own folder is not writable."
                .to_owned()
        }
        _ => "No writable location was found, so anything you change here lasts only \
              until you close pdfcer."
            .to_owned(),
    }
}

// ===========================================================================
// Buttons
// ===========================================================================

/// The commit button.
#[must_use]
pub const fn save() -> &'static str {
    "Save"
}

/// Why Save is greyed.
#[must_use]
pub const fn save_disabled_tooltip() -> &'static str {
    "Nothing has changed yet."
}

/// The abort button.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

/// What Cancel promises, said plainly and unconditionally.
#[must_use]
pub const fn cancel_tooltip() -> &'static str {
    "Close without changing anything. Nothing you have clicked here has taken \
     effect yet."
}

/// The reset control.
#[must_use]
pub const fn restore_defaults() -> &'static str {
    "Restore defaults"
}

/// Why *Restore defaults* is greyed.
#[must_use]
pub const fn restore_defaults_disabled_tooltip() -> &'static str {
    "Everything is already set to pdfcer's own answer."
}

/// What *Restore defaults* actually does, on hover when it is live.
#[must_use]
pub const fn restore_defaults_tooltip() -> &'static str {
    "Sets every choice below back to pdfcer's own answer. Nothing is written \
     until you press Save, and Cancel still puts everything back."
}

/// The status-bar line after a successful save.
#[must_use]
pub fn saved(path: &str) -> String {
    format!("Settings saved to {path}.")
}

/// The status-bar line after a failed save.
#[must_use]
pub fn save_failed(reason: &str) -> String {
    format!(
        "Settings could NOT be saved: {reason} — this session is using your \
         choices, but they will be gone when pdfcer restarts."
    )
}

// ===========================================================================
// Group headings
// ===========================================================================

/// Group 1.
#[must_use]
pub const fn group_appearance() -> &'static str {
    "Appearance"
}

/// Group 2 — the one that starts expanded.
#[must_use]
pub const fn group_colour() -> &'static str {
    "Colour"
}

/// The group holding the one control about the PERSON rather than the document
/// or the program.
#[must_use]
pub const fn group_comments() -> &'static str {
    "Comments"
}

/// The Forms group's caption.
#[must_use]
pub const fn group_forms() -> &'static str {
    "Forms"
}

/// Group 3.
#[must_use]
pub const fn group_images() -> &'static str {
    "Images and transparency"
}

/// The Fonts group's caption.
#[must_use]
pub const fn group_fonts() -> &'static str {
    "Fonts"
}

/// The folder list's label.
#[must_use]
pub const fn font_folders_label() -> &'static str {
    "Folders to take fonts from"
}

/// The hint states the **consequence of leaving it empty**, which is the
/// one fact an operator cannot discover from an empty list.
#[must_use]
pub const fn font_folders_hint() -> &'static str {
    "When a document names a font it does not carry, pdfcer looks here to embed it. It \
     never searches your system fonts on its own."
}

/// Shown in place of an empty list.
#[must_use]
pub const fn font_folders_none() -> &'static str {
    "No folders of your own yet."
}

/// The same empty state when the OS-fonts box is **not** ticked either.
#[must_use]
pub const fn font_folders_none_at_all() -> &'static str {
    "No folders yet and this computer's fonts are switched off, so embedding a missing \
     font has nowhere to take one from."
}

/// The checkbox the operator asked for, in his own words.
#[must_use]
pub const fn use_os_fonts_label() -> &'static str {
    "Use the fonts installed on this computer"
}

/// What ticking it means, including the part pdfcer cannot answer for them.
#[must_use]
pub const fn use_os_fonts_hint() -> &'static str {
    "Embedding puts a font's outlines inside a document you may send to somebody else, \
     and whether you may do that depends on the font. pdfcer leaves that to you."
}

/// The heading over the folders the checkbox resolves to.
#[must_use]
pub const fn use_os_fonts_folders() -> &'static str {
    "pdfcer will also search:"
}

/// Shown when the box is ticked and the machine reports no font folder at all.
#[must_use]
pub const fn use_os_fonts_none_found() -> &'static str {
    "pdfcer could not find a font folder on this computer."
}

/// The Add button.
#[must_use]
pub const fn font_folder_add() -> &'static str {
    "Add a folder…"
}

/// See [`font_folder_add`].
#[must_use]
pub const fn font_folder_add_hover() -> &'static str {
    "Folders are searched in the order they are listed, and the first one holding the face wins."
}

/// The per-row remove button.
#[must_use]
pub const fn font_folder_remove() -> &'static str {
    "Remove"
}

/// See [`font_folder_remove`].
#[must_use]
pub const fn font_folder_remove_hover() -> &'static str {
    "Stop searching this folder. Nothing on disk is touched."
}

/// The Add button when the list is at its cap.
#[must_use]
pub fn font_folders_full(cap: usize) -> String {
    format!("{cap} folders is the most pdfcer will search. Remove one to add another.")
}

/// The folder picker's title bar.
#[must_use]
pub const fn font_folder_dialog_title() -> &'static str {
    "Choose a folder pdfcer may take fonts from"
}

/// Group 4.
#[must_use]
pub const fn group_text() -> &'static str {
    "Copying and extracting text"
}

/// Group 5.
#[must_use]
pub const fn group_measuring() -> &'static str {
    "Measuring and dimensioning"
}

/// Group 6.
#[must_use]
pub const fn group_pages() -> &'static str {
    "Pages and printing"
}

/// Group 7.
#[must_use]
pub const fn group_saving() -> &'static str {
    "Saving files"
}

/// Group 8 — the only one that is not about the PDF standard.
#[must_use]
pub const fn group_display() -> &'static str {
    "Drawing the page"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **How many settings the window offers.**
    ///
    /// Quoted by two tests that approach it from opposite ends — the copy
    /// catalog below, and [`the_window_draws_exactly_the_settings_this_catalog_describes`],
    /// which counts the controls the dialog actually builds. Neither is
    /// meaningful without the other: a catalog can describe a setting nobody
    /// draws, and a dialog can draw one nobody described.
    ///
    /// 15 answers to a silent standard, plus **6** preferences of the shell's
    /// own.
    ///
    ///
    ///
    /// It caught something larger that time. The same engine Pass changed
    /// what `format_text` DOES by default — a synthesis request that used to be
    /// refused is now applied — and that silently removed this shell's Bold
    /// button, which was built on the refusal. `cargo update` brought both in
    /// together, and the settings test and one face-by-name assertion were the
    /// only two things that noticed.
    // 28 → 29 on 2026-09-02: `spot_colorant_device_model`, new in
    // `pdfcer-core 0.20`. Ken: *"the engine I think has a couple of new options
    // for colour rendering that we might need to surface."* He was right, and
    // the coverage gate two files away fired on the same `cargo update` — the
    // pair working as designed, one demanding the control and one demanding
    // the copy.
    // 29 → 30 on 2026-09-02: `shade_form_fields`. Ken: *"in our display
    // section we should have an option to shade the form fields like acrobat
    // does."* Note this one is a SHELL preference rather than an engine
    // setting, so the sibling coverage test in `dialogs::settings` — which
    // enumerates the engine's store — could never have demanded it. This
    // catalog is the only instrument that covers both.
    // 30 → 31 on 2026-09-04: `acrobat_path` — O122, *"have a setting where
    // people can change it."* The second SHELL preference in this count and the
    // first setting in the window about **another program on this machine**, so
    // neither the engine-store coverage test nor anything else could have
    // demanded it. Its copy lives in `crate::text::acrobat` rather than here,
    // because O122's four surfaces are one conversation and were filed
    // together; this list reaches across for it, which is what keeps the count
    // honest about a group whose words live elsewhere.
    // 31 → 33 on 2026-09-05, and it is the only entry in this list
    // that moved the count by TWO. The trust-store work adds one ENGINE setting
    // (`acrobat_trust_store`, which the sibling completeness test in
    // `dialogs::settings` demanded — it was red before this control existed)
    // and one SHELL preference beside it (`acrobat_trust_store_path`, which no
    // test could have demanded, because the engine deliberately does not model
    // where the file is: *"locating the file is the shell's job"*).
    //
    // They are two headers rather than one on purpose. A permission and a
    // location have different blast radii — one governs the pdfcer command line
    // as well, the other changes only which file is read — and a single
    // `radius` line covering both would have to be vague about the one that
    // matters. `dialogs::settings::signatures`' header carries the argument.
    //
    // 33 → 34 on 2026-09-05: **one** header over the two auto-hide
    // toggles, and the singular is the decision rather than a shortcut. They
    // are one question the operator answers twice — *"how much of the window
    // do I want the drawing to have?"* — and the sentence that makes the
    // feature safe to try, that the drawing does not move when a strip comes
    // and goes, is true of both. Two headers would have to say it twice or
    // leave it off one of them, which is the trust-store pair's test applied
    // and answered the other way: those two have different blast radii, these
    // two have the same one.
    //
    // 38 → 39: the colour the recognised text is drawn in over a scan —
    // O229, and the third SHELL preference in this count that no engine-store
    // coverage test could have demanded. It is one header rather than a colour
    // row folded under the field wash, because the two settings share only the
    // word *colour*: one is an affordance over form controls and this one is
    // the appearance of a mode the operator switches on deliberately.
    //
    // The coloured-icons switch (O232) is a shell preference beside the UI
    // scale in the Appearance group.
    const SETTINGS_COUNT: usize = 40;

    /// The `(title, silence, radius)` triple for every setting in the window.
    fn triples() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            (theme_title(), theme_silence(), theme_radius()),
            // O122's triple, reached across into `crate::text::acrobat`. See
            // `SETTINGS_COUNT` on why that module holds it.
            (
                crate::text::acrobat::path_title(),
                crate::text::acrobat::path_silence(),
                crate::text::acrobat::path_radius(),
            ),
            (
                cmyk_intent_title(),
                cmyk_intent_silence(),
                cmyk_intent_radius(),
            ),
            (polarity_title(), polarity_silence(), polarity_radius()),
            (
                author_name_title(),
                author_name_silence(),
                author_name_radius(),
            ),
            (
                blend_space_title(),
                blend_space_silence(),
                blend_space_radius(),
            ),
            (zero_tint_title(), zero_tint_silence(), zero_tint_radius()),
            // Beside its sibling, because they are the same subject from two
            // sides: that one is what OVERPRINTS a spot colour, this one is
            // what a spot colour IS. New in pdfcer-core 0.20 (O100).
            (
                spot_model_title(),
                spot_model_silence(),
                spot_model_radius(),
            ),
            // A shell preference, not an engine setting — see SETTINGS_COUNT.
            (
                field_shade_title(),
                field_shade_silence(),
                field_shade_radius(),
            ),
            // A shell preference, not an engine setting — see
            // SETTINGS_COUNT. Beside the field wash because both are colours
            // pdfcer draws over a page and neither reaches the file.
            (
                ocr_colour_title(),
                ocr_colour_silence(),
                ocr_colour_radius(),
            ),
            (
                cmyk_ceiling_title(),
                cmyk_ceiling_silence(),
                cmyk_ceiling_radius(),
            ),
            (
                mesh_padding_title(),
                mesh_padding_silence(),
                mesh_padding_radius(),
            ),
            (mask_title(), mask_silence(), mask_radius()),
            (minify_title(), minify_silence(), minify_radius()),
            (word_gap_title(), word_gap_silence(), word_gap_radius()),
            // A shell preference, not an engine setting — see
            // SETTINGS_COUNT. Beside `word_gap` because both are about what
            // comes out of the page as text.
            (find_trim_title(), find_trim_silence(), find_trim_radius()),
            (parallel_title(), parallel_silence(), parallel_radius()),
            (
                unmappable_title(),
                unmappable_silence(),
                unmappable_radius(),
            ),
            (
                actual_text_title(),
                actual_text_silence(),
                actual_text_radius(),
            ),
            (
                style_policy_title(),
                style_policy_silence(),
                style_policy_radius(),
            ),
            (
                separations_title(),
                separations_silence(),
                separations_radius(),
            ),
            (
                missing_as_title(),
                missing_as_silence(),
                missing_as_radius(),
            ),
            (xref_eol_title(), xref_eol_silence(), xref_eol_radius()),
            (
                trailing_eol_title(),
                trailing_eol_silence(),
                trailing_eol_radius(),
            ),
            // The one setting in this window whose SILENCE line does not
            // describe a silence. §12.5.6.10 states a corner order and almost
            // no producer follows it, so the sentence says that instead — see
            // `dialogs::settings::saving::quad_point_order`.
            (
                quad_order_title(),
                quad_order_silence(),
                quad_order_radius(),
            ),
            // The four in the *Drawing the page* group — the shell's own
            // preferences rather than answers to a silent standard. They are
            // in this list for exactly the same reason the thirteen above are:
            // the obligation is a property of a **control in this window**, not
            // of which file its value happens to be stored in.
            (quality_title(), quality_silence(), quality_radius()),
            (settle_title(), settle_silence(), settle_radius()),
            (
                page_cache_title(),
                page_cache_silence(),
                page_cache_radius(),
            ),
            (
                opening_fit_title(),
                opening_fit_silence(),
                opening_fit_radius(),
            ),
            (
                wheel_paging_title(),
                wheel_paging_silence(),
                wheel_paging_radius(),
            ),
            // O58 — the paste-order choice. It sits beside wheel paging
            // because both are the same shape of question: what should a
            // familiar input mean in this program.
            (
                paste_chords_title(),
                paste_chords_silence(),
                paste_chords_radius(),
            ),
            (chrome_title(), chrome_silence(), chrome_radius()),
            // The two Forms settings. Both are engine settings whose radius
            // stops at a keystroke, which is why their strings sit in
            // `look` beside the shell's own preferences rather than in
            // `bytes`.
            (tab_tail_title(), tab_tail_silence(), tab_tail_radius()),
            (
                tab_tolerance_title(),
                tab_tolerance_silence(),
                tab_tolerance_radius(),
            ),
            (auto_hide_title(), auto_hide_silence(), auto_hide_radius()),
            // The two trust-store settings, reached across into
            // `crate::text::trust` for the reason `crate::text::acrobat`'s
            // triple is reached across for: the subject's copy is one
            // conversation and lives in one module, and this list reaching for
            // it is what keeps the count honest about a group whose words are
            // written elsewhere.
            //
            // The first is an ENGINE setting and the second a SHELL preference.
            // They sit adjacent here because the obligation this list checks is
            // a property of a CONTROL IN THIS WINDOW, not of which file its
            // value happens to be stored in — the same rule the four *Drawing
            // the page* entries above are here under.
            (
                crate::text::trust::use_store_title(),
                crate::text::trust::use_store_silence(),
                crate::text::trust::use_store_radius(),
            ),
            (
                crate::text::trust::store_path_title(),
                crate::text::trust::store_path_silence(),
                crate::text::trust::store_path_radius(),
            ),
            // The theme's twin in the Appearance group — the second setting
            // that changes the program rather than the document.
            (ui_scale_title(), ui_scale_silence(), ui_scale_radius()),
            (
                colour_icons_title(),
                colour_icons_silence(),
                colour_icons_radius(),
            ),
            // The Redacting group's one setting — a shell preference, here for
            // the reason every other shell preference in this list is here.
            (reach_title(), reach_silence(), reach_radius()),
        ]
    }

    /// Every setting answers all three obligations, and none of the answers
    /// is empty.
    #[test]
    fn every_setting_states_its_silence_and_its_radius() {
        let triples = triples();
        assert_eq!(
            triples.len(),
            SETTINGS_COUNT,
            "one triple per setting in the window"
        );
        for (title, silence, radius) in triples {
            assert!(!title.is_empty(), "a setting with no title");
            assert!(!silence.is_empty(), "{title:?} does not say what is open");
            assert!(!radius.is_empty(), "{title:?} does not say what it costs");
        }
    }

    /// **The window draws exactly the settings this catalog describes.**
    const GROUP_SOURCES: &[(&str, &str)] = &[
        (
            "appearance",
            include_str!("../../dialogs/settings/appearance.rs"),
        ),
        ("colour", include_str!("../../dialogs/settings/colour.rs")),
        ("acrobat", include_str!("../../dialogs/settings/acrobat.rs")),
        (
            "comments",
            include_str!("../../dialogs/settings/comments.rs"),
        ),
        ("display", include_str!("../../dialogs/settings/display.rs")),
        ("forms", include_str!("../../dialogs/settings/forms.rs")),
        ("fonts", include_str!("../../dialogs/settings/fonts.rs")),
        ("images", include_str!("../../dialogs/settings/images.rs")),
        (
            "measuring",
            include_str!("../../dialogs/settings/measuring.rs"),
        ),
        ("pages", include_str!("../../dialogs/settings/pages.rs")),
        (
            "redaction",
            include_str!("../../dialogs/settings/redaction.rs"),
        ),
        ("saving", include_str!("../../dialogs/settings/saving.rs")),
        (
            "signatures",
            include_str!("../../dialogs/settings/signatures.rs"),
        ),
        ("text", include_str!("../../dialogs/settings/text.rs")),
    ];

    #[test]
    fn the_window_draws_exactly_the_settings_this_catalog_describes() {
        // Every module under `dialogs/settings/` that draws a setting. `mod.rs`
        // draws none (it composes groups) and `widgets.rs` defines the helper
        // rather than calling it.

        /// Counts calls whose callee path ends in `header`.
        struct Counter(usize);
        impl<'ast> syn::visit::Visit<'ast> for Counter {
            fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
                if let syn::Expr::Path(path) = &*call.func
                    && path
                        .path
                        .segments
                        .last()
                        .is_some_and(|s| s.ident == "header")
                {
                    self.0 += 1;
                }
                // Recurse: a header called inside a closure or a nested block
                // is still a header drawn.
                syn::visit::visit_expr_call(self, call);
            }
        }

        let mut drawn = 0;
        for (name, src) in GROUP_SOURCES {
            let file = syn::parse_file(src)
                .unwrap_or_else(|e| panic!("dialogs/settings/{name}.rs did not parse: {e}"));
            let mut counter = Counter(0);
            syn::visit::visit_file(&mut counter, &file);
            assert!(
                counter.0 > 0,
                "dialogs/settings/{name}.rs draws no setting at all — either it \
                 stopped being a group module, or the header call is written in \
                 a shape this counter does not recognise. The second is the \
                 dangerous one: it would silently under-count."
            );
            drawn += counter.0;
        }

        assert_eq!(
            drawn, SETTINGS_COUNT,
            "the dialog draws {drawn} settings and this catalog describes \
             {SETTINGS_COUNT}. A setting drawn but not catalogued ships with \
             copy nothing checks; a setting catalogued but not drawn is copy \
             nobody can read."
        );
    }

    /// EVERY GROUP MODULE IS IN THE LIST ABOVE — checked, not remembered.
    #[test]
    fn every_settings_module_is_counted() {
        // `defaultapp` (O173) is the third entry and the least obvious. It is
        // drawn at the top of the window and it looks exactly like a group, but
        // it describes NO setting: it reads Windows, and it performs an act
        // whose effect is outside pdfcer. Its words live in
        // `crate::text::assoc` rather than in this catalog for that reason, so
        // listing it in `GROUP_SOURCES` would make the sibling check
        // `the_window_draws_exactly_the_settings_this_catalog_describes` look
        // for catalog entries that must not exist.
        const NOT_A_GROUP: &[&str] = &["widgets", "preset", "defaultapp"];
        let src = include_str!("../../dialogs/settings/mod.rs");
        let file = syn::parse_file(src).expect("dialogs/settings/mod.rs did not parse");
        let declared: Vec<String> = file
            .items
            .iter()
            .filter_map(|item| match item {
                syn::Item::Mod(m) if m.content.is_none() => Some(m.ident.to_string()),
                _ => None,
            })
            .filter(|name| !NOT_A_GROUP.contains(&name.as_str()))
            .collect();
        assert!(
            declared.len() >= 8,
            "parsed {} module declaration(s) out of dialogs/settings/mod.rs — the PARSER is \
             stale, not the list. A test that can only return one answer cannot detect the \
             thing it was added to detect.",
            declared.len()
        );
        let listed: Vec<&str> = GROUP_SOURCES.iter().map(|(n, _)| *n).collect();
        for name in declared {
            assert!(
                listed.contains(&name.as_str()),
                "`dialogs/settings/{name}.rs` is a group module and is NOT in the hand-written \
                 list in this file, so every setting it draws is invisible to the check built \
                 to find it — and the totals still add up, which is what makes it silent. Add \
                 it, or add it to NOT_A_GROUP with the reason it is not a group."
            );
        }
    }

    /// Every setting that changes SAVED BYTES says so, and no other does.
    #[test]
    fn exactly_the_byte_changing_settings_say_they_change_the_file() {
        // "the file you save" / "the bytes pdfcer writes" / "the saved file".
        let touches_bytes = |radius: &str| {
            radius.contains("the file you save")
                || radius.contains("bytes pdfcer writes")
                || radius.contains("the saved file")
        };

        for radius in [
            separations_radius(),
            xref_eol_radius(),
            trailing_eol_radius(),
            polarity_radius(),
            // The least obvious member of this list, which is why it is in
            // it. A faked weight looks like a rendering choice and is written
            // into the content stream — see `style_policy_radius`.
            style_policy_radius(),
        ] {
            assert!(touches_bytes(radius), "a byte setting hides it: {radius:?}");
        }

        // The theme is checked against BOTH its lines, and it is the only one.
        //
        // Every other setting makes its "and the file is untouched" promise in
        // its radius line. The theme makes it in its **silence** line —
        // *"nothing here is written into a PDF you save"* — because that is the
        // sentence explaining why a window-chrome setting is in a window full
        // of file-format questions at all, and repeating it one line later
        // would be padding. Its radius line has a different and more useful job:
        // saying that this one setting takes effect **before** Save, which is
        // the exception to the whole window's contract.
        //
        // So the pair is joined here rather than the theme being exempted. An
        // exemption would let a future edit delete the promise from both.
        let theme_says = format!("{} {}", theme_silence(), theme_radius());
        assert!(!touches_bytes(&theme_says), "{theme_says:?}");
        assert!(
            theme_says.contains("written into a PDF"),
            "the theme no longer promises it leaves documents alone: {theme_says:?}"
        );

        for radius in [
            cmyk_intent_radius(),
            mask_radius(),
            minify_radius(),
            word_gap_radius(),
            find_trim_radius(),
            parallel_radius(),
            unmappable_radius(),
            actual_text_radius(),
            missing_as_radius(),
            // All four of the shell's own preferences are preview-only, and
            // they are listed here rather than exempted. A preference file is
            // still a file, so "does not change the file" is a claim worth
            // pinning: it means *your PDF*, and an operator reading it needs it
            // to keep meaning that if a preference ever gains a document-facing
            // consequence.
            quality_radius(),
            settle_radius(),
            opening_fit_radius(),
            chrome_radius(),
            ui_scale_radius(),
            colour_icons_radius(),
            tab_tail_radius(),
            tab_tolerance_radius(),
        ] {
            assert!(
                !touches_bytes(radius),
                "a preview-only setting claims it changes the file: {radius:?}"
            );
            //
            // A loose match would be satisfied by a radius line saying the
            // setting *does* change the file — the exact opposite claim — so
            // the looseness would cost the assertion its meaning in the one
            // direction it exists to catch. Each entry below is a full
            // negation, and adding one is a two-second edit for whoever writes
            // a fourteenth way to say it.
            //
            // The third entry is the UI scale's, and it says more than the
            // other two rather than merely differently: *"never changes the
            // page or the file"*. That extra clause is load-bearing for that
            // setting specifically — its title contains the word "size", so
            // the thing an operator will most reasonably expect it to resize is
            // the document, and the radius line has to say it does not.
            const LEAVES_THE_FILE_ALONE: &[&str] = &[
                "does not change the file",
                "Does not change the file",
                "never changes the page or the file",
            ];
            assert!(
                LEAVES_THE_FILE_ALONE.iter().any(|p| radius.contains(p)),
                "a preview-only setting does not say it leaves the file alone: {radius:?}"
            );
        }
    }

    /// Every default that is a GUESS admits it, in its own note.
    #[test]
    fn every_guessed_default_says_it_is_a_guess() {
        let admits = |note: &str| {
            note.contains("pdfcer's own")
                || note.contains("considered guess")
                || note.contains("are guesses")
                || note.contains("has not been checked")
                || note.contains("pdfcer reading")
                || note.contains("pdfcer taking")
        };
        for (name, note) in [
            ("mask_resample", mask_nearest_note()),
            ("image_minify", minify_point_note()),
            ("word_gap_ratio", word_gap_note()),
            ("unmappable_code", unmappable_replacement_note()),
            ("actual_text", actual_text_always_note()),
            ("missing_as", missing_as_nothing_note()),
            ("trailing_eol", trailing_eol_lf_note()),
        ] {
            assert!(
                admits(note),
                "{name}'s default is a guess and its note does not say so: {note:?}"
            );
        }
    }

    /// The one SOURCED default says it is sourced, and says it differently.
    #[test]
    fn the_sourced_default_claims_its_evidence() {
        let note = polarity_never_note();
        assert!(
            note.contains("best-supported"),
            "the one sourced default no longer claims its evidence: {note:?}"
        );
        assert!(
            !note.contains("guess") && !note.contains("pdfcer's own"),
            "the sourced default hedges like a guessed one: {note:?}"
        );
    }

    /// **The colour section offers two options and neither is the deleted
    /// third.**
    #[test]
    fn the_superseded_formula_and_its_divergence_note_are_gone() {
        let section = format!(
            "{} {} {} {} {}",
            cmyk_intent_title(),
            cmyk_intent_neutral_label(),
            cmyk_intent_neutral_note(),
            cmyk_intent_calibrated_label(),
            cmyk_intent_calibrated_note()
        );
        assert!(
            !section.contains("old pdfcer formula"),
            "the superseded formula is back in the colour section: {section:?}"
        );
        assert!(
            !section.contains("deliberately differs"),
            "the divergence note is back, and the default no longer diverges: {section:?}"
        );
        // The two that remain still say what they are for, so this cannot
        // pass by the whole section having been emptied.
        assert!(cmyk_intent_calibrated_label().contains("Match other PDF viewers"));
        assert!(cmyk_intent_neutral_note().contains("CAD"));
    }

    /// The unknown-theme sentence quotes the token and promises to keep it.
    #[test]
    fn an_unknown_theme_is_named_and_kept() {
        let said = theme_unknown("midnight");
        assert!(said.contains("\"midnight\""), "{said:?}");
        assert!(said.contains("kept"), "{said:?}");
    }

    /// Every store location says something, including the one with no home.
    #[test]
    fn every_store_location_is_described() {
        use std::path::PathBuf;
        let portable = StoreLocation {
            path: Some(PathBuf::from("C:\\pdfcer\\userdata\\settings.txt")),
            kind: StoreKind::Portable,
        };
        assert!(store_location(&portable).contains("userdata"));

        let nowhere = StoreLocation {
            path: None,
            kind: StoreKind::None,
        };
        let said = store_location(&nowhere);
        assert!(!said.is_empty());
        assert!(
            said.contains("until you close"),
            "a session with no writable store must say the choices are temporary: {said:?}"
        );
    }

    /// Each theme preset has a distinct name and a distinct description.
    #[test]
    fn the_presets_are_distinguishable() {
        let labels: Vec<&str> = Preset::ALL.iter().map(|p| theme_preset_label(*p)).collect();
        let notes: Vec<&str> = Preset::ALL.iter().map(|p| theme_preset_note(*p)).collect();
        for i in 0..labels.len() {
            for j in (i + 1)..labels.len() {
                assert_ne!(labels[i], labels[j]);
                assert_ne!(notes[i], notes[j]);
            }
        }
    }

    /// The two disclosures added in this port are actually present.
    #[test]
    fn the_two_engine_facts_the_old_window_hid_are_disclosed() {
        assert!(
            unmappable_omit_note().contains("disappears altogether"),
            "the disappearing-run consequence is not disclosed: {:?}",
            unmappable_omit_note()
        );
        let bound = actual_text_bound();
        assert!(
            bound.contains("Whichever you choose"),
            "the ActualText bound reads as an argument for one option: {bound:?}"
        );
        assert!(
            bound.contains("redact"),
            "the ActualText bound does not name redaction: {bound:?}"
        );
    }
}
