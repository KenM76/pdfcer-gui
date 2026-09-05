//! # `text::settings` — every word the Settings window shows
//!
//! The catalog area for [`crate::dialogs::settings`]. Ported from the old
//! shell's `ui_text.rs`, where these strings occupied roughly 700 lines in the
//! middle of a 7,912-line file.
//!
//! ## ★ The one rule this module has that the rest of the catalog does not
//!
//! Carried across verbatim from the source, because it is the reason the copy
//! is written the way it is:
//!
//! > Every string here must be readable by someone who has never opened the
//! > PDF standard. These settings exist BECAUSE the standard is silent, so the
//! > operator is being asked to make a judgement — and a judgement cannot be
//! > made from a clause number. The clause is named for traceability; the
//! > SENTENCE has to stand on its own.
//!
//! So `§8.6.4.4` appears in exactly one place per setting, inside a sentence
//! that would still make sense with the number deleted. An operator who has
//! never heard of ISO 32000-1 must be able to choose correctly.
//!
//! ## The three obligations, and how they are enforced by shape
//!
//! `settings_panel.rs`'s header names three things a settings screen must
//! show that a conventional one omits. Two of them are enforced here by
//! **function naming**, not by review: every setting has a `*_title`, a
//! `*_silence` and a `*_radius`, and
//! [`crate::dialogs::settings::widgets::header`] takes all three as required
//! arguments. A setting cannot be added without answering all three.
//!
//! | obligation | where it lives |
//! |---|---|
//! | 1. What the default rests on | inside the chosen option's `_note`, and only where it is true |
//! | 2. That a choice was made at all | `*_silence` — what the standard leaves open |
//! | 3. Which way costs what | `*_radius` — preview, extraction, or **saved bytes** |
//!
//! ### Obligation 1 is the one the source got wrong, and it is fixed here
//!
//! The ambiguity register grades each recommended default: **(a)** observed
//! Acrobat behaviour, **(b)** corpus census, **(c)** other implementations,
//! **(d)** reasoned inference — *a guess*. Most are (d), and the source's own
//! header says a guess must say it is a guess.
//!
//! It said so for five settings and not for five others that `pdfcer-core`
//! grades (d) just as explicitly: `image_minify`, `unmappable_code`,
//! `actual_text`, `missing_as` and `trailing_eol` all read as confident
//! recommendations. **Their notes now carry the disclosure**, in the same
//! words the settings that had it already use, so the contract the window
//! states about itself is true of all thirteen rather than of eight.
//!
//! The one *positively* sourced default — CMYK JPEG polarity, tier (c) —
//! says so too, because "pdfcer matched every other engine" and "pdfcer
//! guessed" are different claims and must not read alike.
//!
//! ## Two disclosures the source documented in the engine and showed nowhere
//!
//! Both are added here, and both are facts rather than directions:
//!
//! - [`unmappable_omit_note`] now says that a run whose codes are **all**
//!   unmappable disappears entirely under *Leave it out* — not merely that
//!   characters go missing. The layout pass drops a run with no characters,
//!   so a page of `Identity-H` text with no `/ToUnicode` yields *zero runs*.
//!   That is the surprising half and the source's note omitted it.
//! - [`actual_text_bound`] is new. No length correspondence exists between
//!   `/ActualText` and the content it replaces, so character-level mapping
//!   back to glyph positions is **impossible across such a run whichever
//!   option is chosen** — which bounds search highlighting, selection and
//!   redaction-by-text to sequence granularity. `pdfcer-core` calls this *"a
//!   fact to disclose, not a direction to pick"* and the old window disclosed
//!   it nowhere.
//!
//! Both settings' radius lines also now name **redaction**, because R35 is
//! explicit that a redaction built under one value is not equivalent under
//! another, and "affects copied and extracted text" does not tell an operator
//! that.

pub mod bytes;
pub mod extract;
pub mod look;
pub mod overprint;
/// ★ The two print-ready colour controls and the field wash, split out of
/// [`look`] on 2026-09-02 under R2. Its header says which of the three is there
/// for a weak reason and should move out first if the module grows.
pub mod print_colour;

pub use bytes::*;
pub use extract::*;
pub use look::*;
pub use overprint::*;
pub use print_colour::*;

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
///
/// Load-bearing rather than decorative: it is the sentence that tells an
/// operator why this window is full of questions instead of being full of
/// answers. Without it, thirteen radio groups read as thirteen things pdfcer
/// could not decide.
#[must_use]
pub const fn intro() -> &'static str {
    "The PDF standard leaves some things genuinely undefined, so different \
     programs can open the same file and be equally correct while showing you \
     different results. Where that happens, pdfcer asks you rather than deciding \
     quietly. Each choice below says what the standard does not settle, what \
     pdfcer ships as its answer and why, and what changing it affects."
}

/// Where the settings file lives, said in the operator's terms.
///
/// # Why this line is always shown
///
/// An operator who does not know which of the two homes is live cannot follow
/// the update instructions, and those instructions are the one place a wrong
/// guess costs them their configuration: *"replace the program files, keep
/// your `userdata` folder"* means nothing if the settings are not in it.
///
/// [`StoreKind`] is `#[non_exhaustive]`, so the catch-all arm is required by
/// the compiler — and it still says something useful rather than falling
/// silent, because a variant this build does not know about is still a home
/// the operator's settings are in.
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
///
/// Greyed rather than absent, which is the one place this window departs from
/// the no-placeholders rule and is entitled to: Save is *temporarily*
/// unavailable — one radio click makes it live — and greying with a reason on
/// hover is exactly what that rule reserves greying for.
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
///
/// Not a courtesy. Four of the thirteen settings change **saved bytes**, so an
/// operator who has been clicking radio buttons for a minute needs to know
/// that none of it has taken effect — and needs to know it *before* they
/// decide whether to click Cancel, which is why it is a tooltip on an
/// always-enabled control rather than a confirmation after the fact.
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
///
/// It replaces the **draft** and does not save. Said out loud because the
/// button's name suggests otherwise: "restore defaults" in most programs is
/// immediate and irreversible, and this one is neither.
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
///
/// Loud, and deliberately not softened: the operator asked for something to be
/// remembered and it was not. The session still honours the choice — see the
/// dispatch arm — so the sentence has to carry the distinction between "this
/// did not happen" and "this will not survive a restart".
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
///
/// ★ *"Comments"*, not *"Annotations"* or *"Markup"*. Every reviewer UI the
/// operator has used calls them comments; *annotation* is the PDF's word for
/// the object and *markup* is ours for the tool. The heading is where somebody
/// looks, so it takes their word.
#[must_use]
pub const fn group_comments() -> &'static str {
    "Comments"
}

/// Group 3.
#[must_use]
pub const fn group_images() -> &'static str {
    "Images and transparency"
}

/// The Fonts group's caption.
///
/// ★ *"Fonts"*, not *"Font folders"*. A group caption names the subject and the
/// control inside it names the property — the same call `text::ribbon`'s
/// `group_format_font` makes, and it leaves room for a second font setting to
/// join without the caption becoming a list.
#[must_use]
pub const fn group_fonts() -> &'static str {
    "Fonts"
}

/// The folder list's label.
#[must_use]
pub const fn font_folders_label() -> &'static str {
    "Folders to take fonts from"
}

/// ★★ The hint states the **consequence of leaving it empty**, which is the
/// one fact an operator cannot discover from an empty list.
///
/// pdfcer does not search the system font directory and will not: embedding
/// whatever a machine happens to hold into somebody's document is a licensing
/// decision, and it is not pdfcer's to make silently. So an empty list is not a
/// default that works — it is embedding switched off, and the sentence says so
/// before the operator meets it at the far end of a failed embed.
#[must_use]
pub const fn font_folders_hint() -> &'static str {
    "When a document names a font it does not carry, pdfcer looks here to embed it. It \
     never searches your system fonts on its own."
}

/// Shown in place of an empty list.
///
/// ★★ Its wording changed on 2026-08-28 when the OS-fonts checkbox landed:
/// "no folders" stopped meaning "nothing to embed from", because the box may be
/// ticked. An empty-state sentence that contradicts a control four rows below it
/// is worse than none -- an operator who has ticked the box and reads *"nowhere
/// to take one from"* has been told their setting does not work.
#[must_use]
pub const fn font_folders_none() -> &'static str {
    "No folders of your own yet."
}

/// The same empty state when the OS-fonts box is **not** ticked either.
///
/// ★ Two sentences for two states rather than one that hedges. This is the only
/// configuration in which embedding genuinely cannot take a font from anywhere,
/// and it is worth saying plainly at the moment it is true -- not at the far end
/// of an embed, which is where the operator would otherwise meet it.
#[must_use]
pub const fn font_folders_none_at_all() -> &'static str {
    "No folders yet and this computer's fonts are switched off, so embedding a missing \
     font has nowhere to take one from."
}

/// The checkbox the operator asked for, in his own words.
///
/// ★★★ `OPERATOR_REQUESTS.md` **O50**: *"just a simple checkbox to include fonts
/// from the OS installed font folders."* "Installed on this computer" rather
/// than "system fonts" or "OS fonts", because that is what the thing IS to the
/// person ticking it -- they installed them, or their IT did, and either way
/// "OS" is a word about implementation.
#[must_use]
pub const fn use_os_fonts_label() -> &'static str {
    "Use the fonts installed on this computer"
}

/// What ticking it means, including the part pdfcer cannot answer for them.
///
/// ★★ It states the **licensing** consequence, and that is not legal throat-
/// clearing: it is the reason this is a checkbox and not the default. The
/// operator is being handed a decision, and a control that hands somebody a
/// decision without saying what the decision is about is a control that took it
/// for them.
#[must_use]
pub const fn use_os_fonts_hint() -> &'static str {
    "Embedding puts a font's outlines inside a document you may send to somebody else, \
     and whether you may do that depends on the font. pdfcer leaves that to you."
}

/// The heading over the folders the checkbox resolves to.
///
/// ★★ The folders are DRAWN, greyed, under the tick. A checkbox whose effect is
/// invisible is one nobody can verify -- and the per-user folder in particular
/// is somewhere most operators do not know exists, so listing it is the
/// difference between a setting they trust and one they re-tick to see if it
/// took.
#[must_use]
pub const fn use_os_fonts_folders() -> &'static str {
    "pdfcer will also search:"
}

/// Shown when the box is ticked and the machine reports no font folder at all.
///
/// ★ A real state and not a defensive one: `%WINDIR%` and `%LOCALAPPDATA%` are
/// read from the environment rather than assumed, and a stripped or unusual
/// image can leave both unset. Saying so beats a tick with nothing under it,
/// which reads as the list still loading.
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
///
/// ★ A word rather than a `×`. This list is at most sixteen rows and every row
/// is a path an operator typed or picked; a glyph that means *delete* on a row
/// whose other content is a file path is one mis-click from removing the wrong
/// one, and the word is two characters wider.
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
///
/// ★ **New in this port.** In the old shell `parallel_epsilon_degrees` sat
/// under *Copying and extracting text* — where it has nothing to do with
/// either — purely because it happened to be a slider like the word-gap one
/// beside it. The operator symptom is *"my dimension came out as an angle"*,
/// and nobody with that symptom looks under a heading about copying.
///
/// The group headings are the whole navigation model of this window: an
/// operator arrives with a symptom and the headings are how a symptom finds
/// its setting. A setting filed under the wrong one is not untidy, it is
/// unreachable.
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
///
/// ★ **Named for what it is about, not for where its values are stored.** These
/// two settings live in `preferences.txt` rather than `settings.txt`, which is
/// an implementation fact the operator has no business meeting: they opened one
/// window, they press one Save, and one Cancel discards the lot.
///
/// *"Drawing"* rather than *"Rendering"* or *"Performance"*. "Rendering" is a
/// word from our side of the fence; "Performance" promises a tuning panel and
/// there are two controls. What both settings actually change is how the page
/// gets drawn, which is what the heading says.
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
    /// ★ Was 13 + 5 until 2026-08-19, when `quad_point_order` acquired a
    /// control. That setting had been honoured by the engine and offered
    /// nowhere, which is the gap
    /// `crate::dialogs::settings::tests::every_setting_the_store_carries_has_a_control_in_this_window`
    /// now refuses to compile past.
    ///
    /// ★★ And 23 → **24** on 2026-08-26, when `max_cmyk_buffer_bytes` acquired
    /// one. The same gap, caught by the same test, on the very first build
    /// after the engine grew the setting — which is the whole reason that test
    /// is a `#[test]` and not a note. The engine shipped it in v0.14.0 at this
    /// shell's own request; the number moved in the same session the control
    /// was written, which is the property this constant exists to guarantee.
    /// ★★ And 24 → **25** on 2026-08-28, when the engine's `Pass 143.0` added
    /// `overprint_zero_tint_scope` and this window's own completeness test —
    /// `every_setting_the_store_carries_has_a_control_in_this_window` — caught
    /// it within one `cargo update`. That is the mechanism working: a setting
    /// the engine honours and the window cannot reach is a setting an operator
    /// can only change by hand-editing a text file.
    /// ★★ And 25 → **26** on 2026-08-28 with `author_name` — the **first**
    /// entry here that is neither an answer to a silent standard nor a
    /// rendering preference. It is a fact about the *person*, and it is in
    /// this window because the alternative was a comment nobody signed.
    /// ★★ And 26 → **27** on 2026-08-29 with `paste_chords` —
    /// `OPERATOR_REQUESTS.md` O58, and the first entry here that exists
    /// because **another program disagrees with us**. It is a compatibility
    /// preference rather than a taste one: Acrobat assigns the two form-field
    /// pastes to the opposite chords, both assignments have a real argument,
    /// and the operator asked for the choice rather than a ruling.
    /// ★★ And 27 → **28** on 2026-08-30 with `style_policy` — the engine's
    /// `Pass 179.0`, and the **third** time this window's completeness test
    /// caught a setting the engine had grown and the shell could not reach.
    ///
    /// ★★★ It caught something larger that time. The same engine Pass changed
    /// what `format_text` DOES by default — a synthesis request that used to be
    /// refused is now applied — and that silently removed this shell's Bold
    /// button, which was built on the refusal. `cargo update` brought both in
    /// together, and the settings test and one face-by-name assertion were the
    /// only two things that noticed.
    // ★ 28 → 29 on 2026-09-02: `spot_colorant_device_model`, new in
    // `pdfcer-core 0.20`. Ken: *"the engine I think has a couple of new options
    // for colour rendering that we might need to surface."* He was right, and
    // the coverage gate two files away fired on the same `cargo update` — the
    // pair working as designed, one demanding the control and one demanding
    // the copy.
    // ★ 29 → 30 on 2026-09-02: `shade_form_fields`. Ken: *"in our display
    // section we should have an option to shade the form fields like acrobat
    // does."* Note this one is a SHELL preference rather than an engine
    // setting, so the sibling coverage test in `dialogs::settings` — which
    // enumerates the engine's store — could never have demanded it. This
    // catalog is the only instrument that covers both.
    // ★ 30 → 31 on 2026-09-04: `acrobat_path` — O122, *"have a setting where
    // people can change it."* The second SHELL preference in this count and the
    // first setting in the window about **another program on this machine**, so
    // neither the engine-store coverage test nor anything else could have
    // demanded it. Its copy lives in `crate::text::acrobat` rather than here,
    // because O122's four surfaces are one conversation and were filed
    // together; this list reaches across for it, which is what keeps the count
    // honest about a group whose words live elsewhere.
    // ★★★ 31 → 33 on 2026-09-05, and it is the only entry in this list
    // that moved the count by TWO. The trust-store work adds one ENGINE setting
    // (`acrobat_trust_store`, which the sibling completeness test in
    // `dialogs::settings` demanded — it was red before this control existed)
    // and one SHELL preference beside it (`acrobat_trust_store_path`, which no
    // test could have demanded, because the engine deliberately does not model
    // where the file is: *"locating the file is the shell's job"*).
    //
    // ★★ They are two headers rather than one on purpose. A permission and a
    // location have different blast radii — one governs the pdfcer command line
    // as well, the other changes only which file is read — and a single
    // `radius` line covering both would have to be vague about the one that
    // matters. `dialogs::settings::signatures`' header carries the argument.
    const SETTINGS_COUNT: usize = 33;

    /// The `(title, silence, radius)` triple for every setting in the window.
    ///
    /// ★ **Hoisted out of the test it used to live inside, on 2026-08-17, when
    /// it turned out to be four short.**
    ///
    /// The list held exactly the thirteen `pdfcer_core::settings` entries and
    /// had never grown: the *Drawing the page* group's two preferences were
    /// added on 2026-08-17 and neither reached it, so the window's own stated
    /// contract — *"a setting cannot be added without answering all three,
    /// because the code does not compile otherwise"* — was being checked over a
    /// subset of the window while reading as though it covered all of it.
    ///
    /// The `header` helper's required arguments did their job: both settings
    /// **do** answer all three. What was missing was any check that they were
    /// non-empty, and nothing would have caught a `""` passed to satisfy the
    /// signature. That is the whole failure mode the test exists for, and it
    /// had quietly stopped applying to the newest group — which is this
    /// project's most common defect shape wearing a fifth set of clothes.
    fn triples() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            (theme_title(), theme_silence(), theme_radius()),
            // ★ O122's triple, reached across into `crate::text::acrobat`. See
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
            // ★ Beside its sibling, because they are the same subject from two
            // sides: that one is what OVERPRINTS a spot colour, this one is
            // what a spot colour IS. New in pdfcer-core 0.20 (O100).
            (
                spot_model_title(),
                spot_model_silence(),
                spot_model_radius(),
            ),
            // ★ A shell preference, not an engine setting — see SETTINGS_COUNT.
            (
                field_shade_title(),
                field_shade_silence(),
                field_shade_radius(),
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
            // ★ The one setting in this window whose SILENCE line does not
            // describe a silence. §12.5.6.10 states a corner order and almost
            // no producer follows it, so the sentence says that instead — see
            // `dialogs::settings::saving::quad_point_order`.
            (
                quad_order_title(),
                quad_order_silence(),
                quad_order_radius(),
            ),
            // ★ The four in the *Drawing the page* group — the shell's own
            // preferences rather than answers to a silent standard. They are
            // in this list for exactly the same reason the thirteen above are:
            // the obligation is a property of a **control in this window**, not
            // of which file its value happens to be stored in.
            (quality_title(), quality_silence(), quality_radius()),
            (settle_title(), settle_silence(), settle_radius()),
            // ★ How much memory pdfcer may spend so a page it has already drawn
            // does not have to be drawn again — 2026-08-19. Its radius line is
            // the one in this window that names a way to make the program
            // FAIL rather than merely behave differently, which is why it says
            // so out loud.
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
            // ★ O58 — the paste-order choice. It sits beside wheel paging
            // because both are the same shape of question: what should a
            // familiar input mean in this program.
            (
                paste_chords_title(),
                paste_chords_silence(),
                paste_chords_radius(),
            ),
            (chrome_title(), chrome_silence(), chrome_radius()),
            // ★★★ The two trust-store settings, reached across into
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
        ]
    }

    /// ★ Every setting answers all three obligations, and none of the answers
    /// is empty.
    ///
    /// The mechanical half of the window's stated contract. A setting added
    /// with a title and no silence line would compile — the helper takes
    /// `&str` — and would ship a control that says what it is and never says
    /// why the operator is being asked.
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

    /// ★ **The window draws exactly the settings this catalog describes.**
    ///
    /// Written 2026-08-17, and it is the guard that would have caught the
    /// omission [`triples`] documents. Everything else about the window's
    /// contract is enforced from the copy side: `header`'s signature forces
    /// three arguments, and the test above forces three non-empty answers. Both
    /// are blind to the failure that actually happened, which is a control
    /// drawn in the dialog and never entered in the catalog — because a catalog
    /// cannot notice something that is not in it.
    ///
    /// So this counts from the **other** end: it parses the dialog's own source
    /// and counts the [`crate::dialogs::settings::widgets::header`] calls the
    /// application actually makes.
    ///
    /// # Why `syn` and not a grep
    ///
    /// The same reason `shell::commands::reach` uses it: a substring search
    /// would count the word in a doc comment, in a string, or in
    /// `widgets.rs`'s own `header(ui, title, silence, radius)` sketch — which
    /// is inside a fenced block and is not code. The syntax tree contains no
    /// comments, so a header discussed is not a header called.
    ///
    /// # Why the file list is written out
    ///
    /// `include_str!` needs a literal path, and that is the useful half rather
    /// than the awkward half: a **moved or deleted module is a compile error**,
    /// so "scanned nothing" cannot pass as "found nothing" — the trap
    /// `reach.rs` names in its own header. A *new* group module is the one case
    /// this cannot see by construction, and it fails in the right direction:
    /// its settings will be in neither list, the counts still agree, and the
    /// catalog test then fails on the missing triples. The cost is that the
    /// author has to add one line here; the alternative is a directory walk at
    /// test time, which is the runtime file read `reach.rs` refused.
    /// The group modules the window is built from, paired with their source.
    ///
    /// ★ Hoisted out of `the_window_draws_exactly_the_settings_this_catalog_describes`
    /// on 2026-08-30 so that `every_settings_module_is_counted` can check the
    /// list is complete. A hand-written list that only one test can see is a
    /// hand-written list nothing can audit.
    const GROUP_SOURCES: &[(&str, &str)] = &[
        (
            "appearance",
            include_str!("../../dialogs/settings/appearance.rs"),
        ),
        ("colour", include_str!("../../dialogs/settings/colour.rs")),
        // ★★★ Added 2026-09-04 with the Acrobat-path control — O122 — and
        // added BEFORE the control was written rather than ten minutes after,
        // which is the whole point of the note on `comments` below and of
        // `every_settings_module_is_counted`. That test caught this file's
        // absence on the first `cargo test` after the module was created,
        // exactly as designed.
        ("acrobat", include_str!("../../dialogs/settings/acrobat.rs")),
        // ★★★ Added 2026-08-28 with the author-name control, and its
        // absence for the first ten minutes is the finding: this list is
        // HAND-WRITTEN, so a new settings module is invisible to the very
        // test whose job is to prove the window and the catalog agree.
        // A new file draws a header nobody counts and describes a setting
        // nobody checks — the count still adds up and both halves are
        // wrong. If a third module is ever added, this line is the one to
        // remember before the control is written.
        (
            "comments",
            include_str!("../../dialogs/settings/comments.rs"),
        ),
        ("display", include_str!("../../dialogs/settings/display.rs")),
        // ★★★ Added 2026-08-30 with the faking-bold-and-italic control,
        // and it is `comments`' lesson recurring exactly as that comment
        // predicted. `fonts.rs` had existed for two days and drew no
        // `header` at all while it held only the folder list, so its
        // absence here cost nothing and announced nothing. The moment it
        // gained a setting, this list was short by one — which is why
        // `every_settings_module_is_counted` below now derives the
        // membership instead of asking the next session to remember.
        ("fonts", include_str!("../../dialogs/settings/fonts.rs")),
        ("images", include_str!("../../dialogs/settings/images.rs")),
        (
            "measuring",
            include_str!("../../dialogs/settings/measuring.rs"),
        ),
        ("pages", include_str!("../../dialogs/settings/pages.rs")),
        ("saving", include_str!("../../dialogs/settings/saving.rs")),
        // ★ Added 2026-09-05 WITH the trust-store controls rather than after
        // them — `comments`' note above asks for exactly that, and
        // `every_settings_module_is_counted` is what stops it being a request.
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
        ///
        /// Matching on the **last segment** rather than the full path, because
        /// the call is written `widgets::header` today and `super::widgets::header`
        /// or a plain `header` after an import would be the same call. Nothing
        /// else in these modules is named `header`, so the loose match costs
        /// nothing and survives an import style change.
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

    /// ★★★ EVERY GROUP MODULE IS IN THE LIST ABOVE — checked, not remembered.
    ///
    /// # The gap this closes, which had already been found once and left open
    ///
    /// The list above is hand-written, and the comment beside it says so:
    /// *"a new settings module is invisible to the very test whose job is to
    /// prove the window and the catalog agree."* That was written on
    /// 2026-08-28 after `comments.rs` was missed for ten minutes, and it ends
    /// *"if a third module is ever added, this line is the one to remember"*.
    ///
    /// ★★ A note asking a future session to remember something is not a
    /// mechanism, and on 2026-08-30 it failed exactly as written: `fonts.rs`
    /// had existed since 2026-08-28, drew no `header` while it held only the
    /// folder list, and the moment it gained one the count was silently short
    /// by one in **both** directions at once.
    ///
    /// ⇒ So this test reads `dialogs/settings/mod.rs` and requires every module
    /// it declares to appear above. Two modules are excluded **by name and with
    /// a reason**, which is the part that keeps the exclusion honest:
    ///
    /// | module | why it is not a group |
    /// |---|---|
    /// | `widgets` | the helpers the groups are built from; it draws no setting of its own |
    /// | `preset` | the preset row at the top of the window. It DOES call `header`, and that is precisely why it must be excluded rather than forgotten: a preset is not a setting, and counting its header would inflate the total by one forever |
    #[test]
    fn every_settings_module_is_counted() {
        const NOT_A_GROUP: &[&str] = &["widgets", "preset"];
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

    /// ★ Every setting that changes SAVED BYTES says so, and no other does.
    ///
    /// The distinction the window exists to make legible: a setting whose blast
    /// radius is the file on disk is a different kind of decision from one that
    /// only changes the preview, and an operator must be able to tell them
    /// apart from the words. Four of thirteen touch bytes.
    ///
    /// Asserted in both directions. A preview setting whose radius grew the
    /// word "file" would be quietly claiming a consequence it does not have —
    /// which trains the operator to stop reading these lines, and the four that
    /// matter are the ones that then get skipped.
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
            // ★ The least obvious member of this list, which is why it is in
            // it. A faked weight looks like a rendering choice and is written
            // into the content stream — see `style_policy_radius`.
            style_policy_radius(),
        ] {
            assert!(touches_bytes(radius), "a byte setting hides it: {radius:?}");
        }

        // ★ The theme is checked against BOTH its lines, and it is the only one.
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
            parallel_radius(),
            unmappable_radius(),
            actual_text_radius(),
            missing_as_radius(),
            // ★ All four of the shell's own preferences are preview-only, and
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
        ] {
            assert!(
                !touches_bytes(radius),
                "a preview-only setting claims it changes the file: {radius:?}"
            );
            // ★ An EXPLICIT list of accepted phrasings, widened 2026-08-17 and
            // deliberately not loosened to "contains the word file".
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

    /// ★ Every default that is a GUESS admits it, in its own note.
    ///
    /// Obligation 1, mechanised — and the test that would have failed on the
    /// old shell for five of these. `pdfcer-core` grades `image_minify`,
    /// `unmappable_code`, `actual_text`, `missing_as` and `trailing_eol` tier
    /// (d), reasoned inference, exactly as explicitly as it grades the two that
    /// already said so, and all five read as confident recommendations.
    ///
    /// The predicate is deliberately loose about wording and strict about
    /// presence: what matters is that the sentence disclaims external
    /// authority, not that it uses one phrasing.
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

    /// ★ The one SOURCED default says it is sourced, and says it differently.
    ///
    /// The counterpart to the test above, and the reason that one is not
    /// enough. If every note hedged, the operator would have no way to tell
    /// which of thirteen defaults rests on evidence. CMYK JPEG polarity is
    /// tier (c) — every reference engine agrees — and it must not read like a
    /// guess.
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

    /// ★★★ **The colour section offers two options and neither is the deleted
    /// third.**
    ///
    /// This replaces `the_acrobat_divergence_names_the_option_that_matches`,
    /// which asserted that a note naming the divergence from Acrobat also named
    /// the option that restores parity. That note is **gone** — the default now
    /// matches Acrobat (`OPERATOR_REQUESTS.md` O52), so a sentence saying pdfcer
    /// deliberately differs is backwards rather than redundant.
    ///
    /// ★★ The replacement asserts the **absence**, which is the harder half. A
    /// deleted string leaves no test behind it, so nothing would notice a
    /// future edit reinstating either one — and *"the old pdfcer formula"* is
    /// exactly the kind of option somebody restores while looking for
    /// something else.
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
        // ★ The two that remain still say what they are for, so this cannot
        // pass by the whole section having been emptied.
        assert!(cmyk_intent_calibrated_label().contains("Match other PDF viewers"));
        assert!(cmyk_intent_neutral_note().contains("CAD"));
    }

    /// The unknown-theme sentence quotes the token and promises to keep it.
    ///
    /// Both halves matter. Quoting is what makes the cause legible; the promise
    /// is what stops the operator "fixing" it by picking one of the three,
    /// which would discard a newer version's setting.
    #[test]
    fn an_unknown_theme_is_named_and_kept() {
        let said = theme_unknown("midnight");
        assert!(said.contains("\"midnight\""), "{said:?}");
        assert!(said.contains("kept"), "{said:?}");
    }

    /// Every store location says something, including the one with no home.
    ///
    /// The `None` case is the one worth pinning: an operator whose settings
    /// cannot be written anywhere must be told before they spend a minute
    /// choosing, not after they press Save.
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
    ///
    /// Both were documented in `pdfcer-core` and shown nowhere in the old
    /// window. A test rather than a comment, because "we should surface that"
    /// is the kind of intention that survives one session.
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
