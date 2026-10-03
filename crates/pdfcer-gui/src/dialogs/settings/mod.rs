//! # `dialogs::settings` — where a spec ambiguity becomes a choice
//!
//! ## What this surface is for
//!
//!
//! > Where standards are ambiguous those should become settings that the user
//! > can choose direction one, with the initial installed default as the best
//! > guess of what is usually followed.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/settings/mod.md`.

/// O122 — where Acrobat is. The one group in this window that is about
/// **another program on this machine**, and the only one whose whole purpose
/// is to be reachable while the control it governs is absent from the ribbon.
mod acrobat;
/// The button O173 asks for at the top of this window, and the line that
/// says what Windows actually opens PDFs with. Its header argues why it is
/// above even the presets row and why it is not a collapsible group.
pub use pdfcer_gui_base::defaultappsetting as defaultapp;
pub use pdfcer_gui_base::settingspages::appearance;
pub use pdfcer_gui_base::settingspages::colour;
use pdfcer_gui_base::settingspages::comments;
use pdfcer_gui_base::settingspages::preset;

/// The eighth group, and the only one not about the PDF standard: how pdfcer
/// draws, as distinct from what it draws. Two settings out of seven that were
/// commissioned — its header says which five had nothing behind them.
pub mod display;
/// The Fonts group — where pdfcer may take a font from when it has to embed
/// one. See its header for why the list lives here rather than on the batch
/// pane its blocker named.
mod fonts;
/// The order Tab visits a page in. Its header carries the filing rule:
/// both settings answer *"I pressed Tab and it went to the wrong field"*,
/// and neither is discoverable from anywhere else in the program.
use pdfcer_gui_base::settingspages::forms;
pub use pdfcer_gui_base::settingspages::images;
pub use pdfcer_gui_base::settingspages::measuring;
/// The page list, the search box and the page pane.
pub(crate) mod nav;
pub use pdfcer_gui_base::settingspages::pages;
pub use pdfcer_gui_base::settingspages::redaction;
use pdfcer_gui_base::settingspages::remote;
pub use pdfcer_gui_base::settingspages::saving;
/// May pdfcer read the trust list Acrobat has downloaded, and where is it.
pub mod signatures;
pub use pdfcer_gui_base::settingspages::text;
pub use pdfcer_gui_base::settingspages::widgets;

use egui::RichText;
use pdfcer_core::settings::StoreLocation;

use crate::text::settings as t;

/// The region the window publishes for its whole body.
pub const REGION_BODY: &str = "dialog:settings"; // ui-text-exempt: trace region name, never displayed

/// The Cancel button's rect, so a driven check can press the abort path itself
/// rather than pressing Escape and assuming the two agree.
pub const REGION_CANCEL: &str = "dialog:settings.cancel"; // ui-text-exempt: trace region name, never displayed

/// The Save button's rect, so a driven check can commit a change it made.
/// Every check runs in its own profile directory, so pressing it writes that
/// directory's files, never the operator's.
pub const REGION_SAVE: &str = "dialog:settings.save"; // ui-text-exempt: trace region name, never displayed

/// The region each page's entry in the page list publishes, suffixed with its key.
pub const REGION_HEADING_PREFIX: &str = "settings.heading."; // ui-text-exempt: trace region name, never displayed

pub use pdfcer_gui_base::settingspages::REGION_THEME_PREFIX;

pub use pdfcer_gui_base::settingspages::Draft;

/// What the window is asking the application to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing was pressed this frame.
    Idle,
    /// Adopt the working copy and write it.
    Save,
    /// Discard the working copy.
    Cancel,
    /// Replace the working copy with the defaults.
    ///
    /// **Does not save.** The operator still has to confirm, and still has
    /// Cancel. "Restore defaults" is not the kind of button that should be able
    /// to discard a configuration in one click with no way back.
    RestoreDefaults,
}

/// Draw one frame of the window. Returns what the operator asked for.
pub fn show(
    ctx: &egui::Context,
    draft: &mut Draft,
    store: &StoreLocation,
    open: &mut bool,
    acrobat_viewer: Option<&crate::acrobat::Viewer>,
) -> Outcome {
    // Taken, not read: it selects the page once, and the operator can move on.
    let focus = draft.focus.take();
    let screen = ctx.input(egui::InputState::content_rect);
    let width = 860.0_f32.min(screen.width() - 40.0).max(560.0);
    let height = (screen.height() * 0.82).clamp(420.0, 900.0);

    // Its own OS window. `open` is the caller's flag; `frame.closed` is
    // written into it.
    let mut outcome = Outcome::Idle;
    let (frame, ()) = crate::dialogs::host::Host::new(
        "settings", // ui-text-exempt: a viewport key, never displayed.
        t::window_title(),
        egui::vec2(width, height),
        egui::vec2(560.0, 420.0),
    )
    .show(ctx, |ui| {
        crate::diag::ui_rect(REGION_BODY, ui.max_rect());
        // Always shown, never a control. An operator who does not know
        // which of the two homes is live cannot follow the update
        // instructions, and those instructions are the one place a wrong
        // guess costs them their configuration.
        ui.label(RichText::new(t::store_location(store)).small().weak());
        ui.separator();

        // The bottom bar is a separator and one row of buttons.
        let height = (ui.available_height() - 40.0).max(180.0);
        nav::show(ui, height, draft, focus, acrobat_viewer);

        ui.separator();
        ui.horizontal(|ui| {
            let dirty = draft.is_dirty();
            let save = ui.add_enabled(dirty, egui::Button::new(t::save()));
            crate::diag::ui_rect(REGION_SAVE, save.rect);
            if save.clicked() {
                // Retire a stored choice the settings no longer express,
                // **before** it is written.
                //
                // `preset::live_choice` already declines to SHOW a chosen
                // standard once a control has been changed by hand — that is
                // `still_holds`, asked against the preset rather than
                // remembered as a flag. Without this line the window would stop
                // claiming it and the file would go on carrying it, so the next
                // session would open showing a standard the values contradict.
                //
                // The display rule and the stored value have to retire
                // together, and this is the one place a stored value is
                // committed.
                if !preset::still_chosen(draft) {
                    draft.chosen_preset = None;
                    draft.working_prefs.chosen_standard = None;
                }
                outcome = Outcome::Save;
            }
            if !dirty {
                save.on_disabled_hover_text(t::save_disabled_tooltip());
            }
            // **Cancel publishes its own rect, and the reason is a check
            // that cannot be written without it.**
            //
            // `settings_theme_takes_effect` drives a live theme change and then
            // proves it is put back — the one-line coupling whose failure is
            // silent. To press Cancel, a driven check has to know where Cancel
            // is.
            //
            // Without this it could only press **Escape**, which reaches the
            // same `settings_draft = None` and is documented as contractually
            // identical — but "identical today" is not the property under test.
            // An edit that made `Outcome::Cancel` behave differently from the
            // Escape path would leave the check green.
            //
            // And it cannot simply aim at the button beside it: that is
            // **Save**, which writes the operator's real `settings.txt`. A
            // harness that mis-aims by one control does not fail — it silently
            // rewrites his preferences.
            //
            // `ui_rect`, not `ui_rect_visible`: this row is in the dialog's
            // fixed footer, outside the body's scroll area, so it is never
            // partly clipped. The visible-fraction filter exists for content
            // that can scroll out from under the pointer and would report a
            // sliver a driven click cannot hit.
            let cancel = ui.button(t::cancel()).on_hover_text(t::cancel_tooltip());
            crate::diag::ui_rect(REGION_CANCEL, cancel.rect);
            if cancel.clicked() {
                outcome = Outcome::Cancel;
            }
            // Separated from the two commit/abort controls, because it is
            // the destructive one and a mis-click on it costs the operator
            // every choice they have ever made here.
            ui.add_space(12.0);
            let all_default = draft.is_all_default();
            let restore = ui.add_enabled(!all_default, egui::Button::new(t::restore_defaults()));
            if restore.clicked() {
                outcome = Outcome::RestoreDefaults;
            }
            if all_default {
                restore.on_disabled_hover_text(t::restore_defaults_disabled_tooltip());
            } else {
                restore.on_hover_text(t::restore_defaults_tooltip());
            }
        });
    });

    // The caller's flag, written rather than returned. See the note above the
    // host for why this dialog differs from the others.
    if frame.closed {
        *open = false;
    }
    outcome
}

/// Draw one page's settings. `nav::PAGES` names the pages; a key it does not
/// name draws nothing.
fn page_body(
    ui: &mut egui::Ui,
    key: &str,
    draft: &mut Draft,
    acrobat_viewer: Option<&crate::acrobat::Viewer>,
) {
    // Within a page the order is an argument: what the operator adjusts while
    // looking comes before what applies to the next document, and a setting
    // comes after the one it qualifies.
    match key {
        "general" => {
            ui.label(t::intro());
            ui.add_space(10.0);
            defaultapp::group(ui, &mut draft.default_app);
        }
        "presets" => preset::row(ui, draft),
        "appearance" => {
            // The only settings that change the program's appearance rather
            // than the document's, and the only ones applied before Save.
            appearance::theme(ui, draft);
            ui.add_space(10.0);
            appearance::ui_scale(ui, &mut draft.working_prefs);
            ui.add_space(10.0);
            appearance::colour_icons(ui, &mut draft.working_prefs);
        }
        "colour" => {
            colour::intent(ui, draft);
            ui.add_space(10.0);
            colour::polarity(ui, draft);
            ui.add_space(10.0);
            colour::page_blend_space(ui, draft);
            ui.add_space(10.0);
            // After `page_blend_space`: that decides whether a page is
            // composited in ink, these decide what the ink rules then reach.
            colour::zero_tint(ui, draft);
            colour::spot_model(ui, draft);
            ui.add_space(10.0);
            colour::cmyk_ceiling(ui, draft);
            ui.add_space(10.0);
            colour::mesh_patch_padding(ui, draft);
        }
        "fonts" => {
            // Where pdfcer looks for a face, then what it does when looking
            // has failed.
            fonts::folders(ui, &mut draft.working_prefs);
            ui.add_space(10.0);
            ui.separator();
            fonts::style_policy(ui, draft);
        }
        "images" => {
            images::mask_resample(ui, draft);
            ui.add_space(10.0);
            images::minify(ui, draft);
        }
        "text" => {
            text::word_gap(ui, draft);
            ui.add_space(10.0);
            text::unmappable(ui, draft);
            ui.add_space(10.0);
            text::actual_text(ui, draft);
            ui.add_space(10.0);
            text::find_trim(ui, &mut draft.working_prefs);
        }
        "measuring" => measuring::parallel(ui, draft),
        "comments" => comments::author_name(ui, &mut draft.working_prefs),
        "forms" => {
            forms::tab_tail(ui, draft);
            ui.add_space(10.0);
            forms::row_tolerance(ui, draft);
        }
        "pages" => {
            pages::separations(ui, draft);
            ui.add_space(10.0);
            pages::missing_as(ui, draft);
        }
        "signatures" => {
            // The permission before the location: a path typed with the
            // permission off would look broken.
            signatures::use_store(ui, draft);
            ui.add_space(10.0);
            signatures::store_path(ui, draft);
        }
        "display" => {
            let prefs = &mut draft.working_prefs;
            display::render_quality(ui, prefs);
            ui.add_space(10.0);
            display::zoom_settle(ui, prefs);
            ui.add_space(10.0);
            display::page_cache(ui, prefs);
            ui.add_space(10.0);
            display::opening_fit(ui, prefs);
            display::wheel_paging(ui, prefs);
            display::paste_chords(ui, prefs);
            ui.add_space(10.0);
            display::field_shade(ui, prefs);
            ui.add_space(10.0);
            display::ocr_colour(ui, prefs);
            display::auto_hide(ui, prefs);
            display::page_chrome(ui, prefs);
        }
        // Drawn whether or not an Acrobat was found: the ribbon control is
        // absent when discovery failed, so this page is where the operator
        // learns the feature exists.
        "acrobat" => acrobat::path(ui, &mut draft.working_prefs, acrobat_viewer),
        "saving" => {
            saving::xref_entry_eol(ui, draft);
            ui.add_space(10.0);
            saving::trailing_eol(ui, draft);
            ui.add_space(10.0);
            saving::edited_stream_compression(ui, draft);
            ui.add_space(10.0);
            saving::quad_point_order(ui, draft);
        }
        "redaction" => redaction::residual_reach(ui, &mut draft.working_prefs),
        "shortcuts" => pdfcer_gui_base::settingspages::keys::page(ui, draft),
        "remote" => remote::remote_control(ui, &mut draft.working_prefs),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::pageops::SeparationPolicy;
    use pdfcer_core::settings::{CmykIntent, Settings};

    /// Every settings-page source, here and in the base crate's
    /// `settingspages`, paired with its module name.
    const SOURCES: &[(&str, &str)] = &[
        ("mod", include_str!("mod.rs")),
        // The base crate's half of this window: `Draft` and the pages that
        // need no document state.
        (
            "settingspages",
            include_str!("../../../../pdfcer-gui-base/src/settingspages/mod.rs"),
        ),
        (
            "appearance",
            include_str!("../../../../pdfcer-gui-base/src/settingspages/appearance.rs"),
        ),
        (
            "colour",
            include_str!("../../../../pdfcer-gui-base/src/settingspages/colour.rs"),
        ),
        ("acrobat", include_str!("acrobat.rs")),
        (
            "comments",
            include_str!("../../../../pdfcer-gui-base/src/settingspages/comments.rs"),
        ),
        // Binds no `Settings` field at all - its one stored value is a
        // `Prefs` field, and the act it performs is outside pdfcer entirely -
        // so listing it changes no verdict today. It is listed anyway: the
        // check is about the DIRECTORY being fully scanned, and a module
        // exempted because it happens to bind nothing is a module nobody
        // re-checks when it starts binding something.
        (
            "defaultapp",
            include_str!("../../../../pdfcer-gui-base/src/defaultappsetting.rs"),
        ),
        ("display", include_str!("display.rs")),
        ("fonts", include_str!("fonts.rs")),
        (
            "forms",
            include_str!("../../../../pdfcer-gui-base/src/settingspages/forms.rs"),
        ),
        (
            "images",
            include_str!("../../../../pdfcer-gui-base/src/settingspages/images.rs"),
        ),
        (
            "measuring",
            include_str!("../../../../pdfcer-gui-base/src/settingspages/measuring.rs"),
        ),
        ("nav", include_str!("nav.rs")),
        (
            "pages",
            include_str!("../../../../pdfcer-gui-base/src/settingspages/pages.rs"),
        ),
        (
            "preset",
            include_str!("../../../../pdfcer-gui-base/src/settingspages/preset.rs"),
        ),
        // Binds no `Settings` field either — its one stored value is a `Prefs`
        // field — and is listed for `defaultapp`'s reason above.
        (
            "redaction",
            include_str!("../../../../pdfcer-gui-base/src/settingspages/redaction.rs"),
        ),
        (
            "remote",
            include_str!("../../../../pdfcer-gui-base/src/settingspages/remote.rs"),
        ),
        (
            "saving",
            include_str!("../../../../pdfcer-gui-base/src/settingspages/saving.rs"),
        ),
        // Its absence would make the completeness test report
        // `Settings::acrobat_trust_store` as uncontrolled with the control
        // written and working — the failure that
        // `every_source_in_this_directory_is_listed` exists to catch, and the
        // reason a module is listed in the edit that creates it rather than
        // afterwards.
        ("signatures", include_str!("signatures.rs")),
        (
            "keys",
            include_str!("../../../../pdfcer-gui-base/src/settingspages/keys.rs"),
        ),
        (
            "keyscatalog",
            include_str!("../../../../pdfcer-gui-base/src/settingspages/keyscatalog.rs"),
        ),
        (
            "text",
            include_str!("../../../../pdfcer-gui-base/src/settingspages/text.rs"),
        ),
        (
            "widgets",
            include_str!("../../../../pdfcer-gui-base/src/settingspages/widgets.rs"),
        ),
    ];

    /// EVERY MODULE THIS DIRECTORY OR `settingspages` DECLARES IS IN [`SOURCES`].
    #[test]
    fn every_source_in_this_directory_is_listed() {
        let mut declared: Vec<String> = Vec::new();
        for src in [
            include_str!("mod.rs"),
            include_str!("../../../../pdfcer-gui-base/src/settingspages/mod.rs"),
        ] {
            let file = syn::parse_file(src).expect("a settings mod.rs did not parse");
            declared.extend(file.items.iter().filter_map(|item| match item {
                syn::Item::Mod(m) if m.content.is_none() => Some(m.ident.to_string()),
                _ => None,
            }));
        }
        assert!(
            declared.len() >= 8,
            "parsed {} module declaration(s) — the PARSER is stale, not the list.",
            declared.len()
        );
        let listed: Vec<&str> = SOURCES.iter().map(|(name, _)| *name).collect();
        for name in declared {
            assert!(
                listed.contains(&name.as_str()),
                "`{name}.rs` is not in SOURCES, so any control it binds is invisible to \
                 `every_setting_the_store_carries_has_a_control_in_this_window` — which will \
                 then report that setting as having no control at all, whether or not it has \
                 one. Add it."
            );
        }
    }

    /// **Every setting `pdfcer-core` carries has a control in this window.**
    #[test]
    fn every_setting_the_store_carries_has_a_control_in_this_window() {
        let file = Settings::default().write_to_string();
        let keys: Vec<&str> = file
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with('#') && !line.is_empty())
            .filter_map(|line| line.split_once('='))
            .map(|(key, _)| key.trim())
            .collect();

        // A floor, so a change to the file format that stopped this parser
        // finding anything reports ITSELF rather than reporting that every
        // setting is covered. An instrument that can only return one answer
        // cannot detect the thing it was added to detect.
        assert!(
            keys.len() >= 10,
            "parsed {} key(s) out of the settings file — the PARSER is stale, not the window. \
             `Settings::write_to_string` no longer emits `key = value` lines this test can read.",
            keys.len()
        );

        // Collected rather than asserted one at a time: an engine pin bump
        // commonly adds a related PAIR of settings, and a test that stops at
        // the first makes the operator discover the second on the next run.
        let missing: Vec<&str> = keys
            .into_iter()
            .filter(|key| {
                let needle = format!("working.{key}");
                !SOURCES.iter().any(|(_, src)| src.contains(&needle))
            })
            .collect();
        assert!(
            missing.is_empty(),
            "{} setting(s) honoured by the engine have NO control in this window, so an \
             operator can only change them by hand-editing settings.txt — which is not a user \
             interface: {}. Add each to whichever group matches the SYMPTOM that would send \
             somebody looking for it (see the module header), rather than to whichever group \
             is shortest.",
            missing.len(),
            missing.join(", ")
        );
    }

    /// The shell's own preferences are **not** covered by the sweep above, and
    /// this records why rather than leaving the gap implied.
    #[test]
    fn the_sweep_is_scoped_to_the_engines_store_on_purpose() {
        let draft = Draft::new(&Settings::default(), &crate::app::prefs::Prefs::default());
        // Both halves exist and are distinct fields; the assertion is that this
        // test's premise is still true, so its doc comment is not describing a
        // structure that has since changed.
        let _ = &draft.working;
        let _ = &draft.working_prefs;
    }

    /// Opening the window is not an edit.
    #[test]
    fn a_fresh_draft_is_not_dirty() {
        let draft = Draft::new(&Settings::default(), &crate::app::prefs::Prefs::default());
        assert!(!draft.is_dirty());
        assert!(draft.is_all_default());
    }

    /// Changing a value makes Save live and Restore live.
    #[test]
    fn changing_a_value_makes_the_draft_dirty() {
        let mut draft = Draft::new(&Settings::default(), &crate::app::prefs::Prefs::default());
        // ⇒ **A test that names a specific value to prove "something changed"
        // is coupled to what the default is.** Assign the value that is
        // currently the default and the draft does not move, and the test fails
        // on a build where the dirty flag works perfectly. The assertion below
        // states the premise, so a move in the engine's defaults fails with
        // *"the test needs a CHANGE"* rather than with *"the dirty flag is
        // broken"*.
        draft.working.cmyk_intent = CmykIntent::NeutralBlack;
        assert_ne!(
            draft.working.cmyk_intent,
            Settings::default().cmyk_intent,
            "the test needs a CHANGE"
        );
        assert!(draft.is_dirty());
        assert!(!draft.is_all_default());
    }

    /// The dirty flag does not latch.
    #[test]
    fn changing_a_value_back_makes_it_clean_again() {
        let mut draft = Draft::new(&Settings::default(), &crate::app::prefs::Prefs::default());
        let was = draft.working.cmyk_intent;
        // Any value other than the default works here — the test is about the
        // flag not latching, not about colour — and the one requirement is that
        // it differs from `was`, which the assertion two lines down enforces.
        draft.working.cmyk_intent = CmykIntent::NeutralBlack;
        assert_ne!(draft.working.cmyk_intent, was, "the test needs a CHANGE");
        assert!(draft.is_dirty());
        draft.working.cmyk_intent = was;
        assert!(!draft.is_dirty(), "the dirty flag latched");
    }

    /// The two predicates must not collapse into one.
    #[test]
    fn a_draft_started_from_non_default_settings_is_clean_but_not_all_default() {
        // `Settings` is `#[non_exhaustive]`, so a struct expression is illegal
        // out of crate: the only shape available is start-from-default and
        // assign, which is what this seeds through.
        let mut seed = Settings::default();
        seed.separations = SeparationPolicy::Refuse;
        let draft = Draft::new(&seed, &crate::app::prefs::Prefs::default());
        assert!(!draft.is_dirty(), "loading is not editing");
        assert!(
            !draft.is_all_default(),
            "Restore defaults must stay available"
        );
    }

    /// Restore defaults reaches all-default without going through Save.
    #[test]
    fn restoring_defaults_is_a_draft_edit_and_nothing_else() {
        let mut seed = Settings::default();
        seed.separations = SeparationPolicy::Refuse;
        let mut draft = Draft::new(&seed, &crate::app::prefs::Prefs::default());
        draft.working = Settings::default();
        assert!(draft.is_all_default());
        // …and it is an *edit*, so Save is offered. The operator must confirm.
        assert!(
            draft.is_dirty(),
            "restoring defaults must leave something to save"
        );
    }
}
