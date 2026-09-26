//! # `dialogs::settings::fonts` — the Settings window's Fonts group
//!
//! One control: the list of folders pdfcer searches when it has to embed a font
//! a document names but does not carry.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/settings/fonts.md`.

use egui::Ui;
use pdfcer_core::settings::StylePolicy;

use crate::app::prefs::{Prefs, fonts};
use crate::text::settings as t;

use super::{Draft, widgets};

/// The Fonts group's rect, for `ui-verify`.
// ui-text-exempt: trace region name, never displayed
pub const REGION: &str = "settings.fonts";
/// The Add button.
// ui-text-exempt: trace region name, never displayed
pub const ADD_REGION: &str = "settings.fonts.add";
/// The "use this computer's fonts" checkbox.
// ui-text-exempt: trace region name, never displayed
pub const OS_REGION: &str = "settings.fonts.use_os";
/// The faking-bold-and-italic radio group.
// ui-text-exempt: trace region name, never displayed
pub const STYLE_POLICY_REGION: &str = "settings.fonts.style_policy";

/// Draw the folder list and its two controls.
pub fn folders(ui: &mut Ui, prefs: &mut Prefs) {
    ui.label(t::font_folders_label());
    ui.small(t::font_folders_hint());
    ui.add_space(4.0);

    // The list is drawn before the buttons, and each row carries its own
    // Remove. A single "remove selected" would need a selection model for a
    // list that is at most sixteen rows — and a row whose delete is on the row
    // is the arrangement every list in this shell already uses.
    let mut remove: Option<usize> = None;
    for (index, folder) in prefs.font_folders.iter().enumerate() {
        ui.horizontal(|ui| {
            if ui
                .small_button(t::font_folder_remove())
                .on_hover_text(t::font_folder_remove_hover())
                .clicked()
            {
                remove = Some(index);
            }
            // `truncate` with the full path on hover — `panels::properties`'
            // row rule, and for its reason: a path can run to any length and a
            // row that grew to three lines would push the buttons under it
            // around as the operator added folders.
            let text = folder.display().to_string();
            ui.add(egui::Label::new(&text).truncate())
                .on_hover_text(&text);
        });
    }
    if let Some(index) = remove {
        prefs.font_folders.remove(index);
    }

    if prefs.font_folders.is_empty() {
        // The empty state is a SENTENCE, not a blank. An empty list is
        // indistinguishable from a broken control, and this one has a
        // consequence worth stating before the operator meets it at the far end
        // of an embed.
        //
        // TWO sentences, because there are two states and only one is a
        // problem. "No folders" does not mean "nothing to embed from" — the
        // OS-fonts box may be ticked — and a single empty state would
        // contradict a control four rows below it, telling an operator who HAS
        // ticked it that their setting does not work.
        ui.small(if prefs.use_os_fonts {
            t::font_folders_none()
        } else {
            t::font_folders_none_at_all()
        });
    }

    ui.add_space(4.0);
    let full = prefs.font_folders.len() >= fonts::MAX_FOLDERS;
    let add = ui.add_enabled(!full, egui::Button::new(t::font_folder_add()));
    crate::diag::ui_rect_visible(ADD_REGION, add.rect, ui.clip_rect());
    let add = if full {
        // R9: greyed for a temporarily unavailable capability, explained on
        // hover. Removing a folder makes it available again, which is what the
        // sentence says.
        add.on_disabled_hover_text(t::font_folders_full(fonts::MAX_FOLDERS))
    } else {
        add.on_hover_text(t::font_folder_add_hover())
    };
    if add.clicked()
        && let crate::app::files::Picked::Path(path) = crate::app::files::pick_font_folder()
    {
        fonts::add(&mut prefs.font_folders, &path);
    }

    ui.add_space(8.0);
    ui.separator();
    // **The checkbox the operator asked for — `OPERATOR_REQUESTS.md` O50.**
    //
    // *"just a simple checkbox to include fonts from the OS installed font
    // folders."* Below the list rather than above it, and the order is the
    // argument: the folders **he** curated are the primary answer and this is
    // the fallback, which is also the search order `fonts::search_path`
    // enforces. A control drawn above a list it is subordinate to reads as the
    // main event.
    let os = ui.checkbox(&mut prefs.use_os_fonts, t::use_os_fonts_label());
    crate::diag::ui_rect_visible(OS_REGION, os.rect, ui.clip_rect());
    // The hint is DRAWN, not put on hover, and it is the one place in this
    // window that argues for itself. Every other hint here describes a control;
    // this one hands the operator a licensing decision, and a decision nobody
    // reads is a decision the program took. Hover text is for the operator who
    // went looking — this is for the one who did not.
    ui.small(t::use_os_fonts_hint());

    // The folders the tick resolves to, drawn under it.
    //
    // A checkbox whose effect is invisible is one nobody can verify. The
    // per-user folder in particular — `…\AppData\Local\Microsoft\Windows\Fonts`
    // — is somewhere most operators do not know exists, and it is where a plain
    // double-click on a `.ttf` installs by default on a modern Windows. Listing
    // it is the difference between a setting an operator trusts and one they
    // re-tick to see whether it took.
    //
    // Drawn only when ticked. An unticked box with a list of folders under it
    // states a fact about the machine and implies a promise about the program.
    if prefs.use_os_fonts {
        let found = fonts::os_font_dirs();
        if found.is_empty() {
            ui.small(t::use_os_fonts_none_found());
        } else {
            ui.small(t::use_os_fonts_folders());
            for dir in &found {
                // `weak`, and truncated with the full path on hover, for the
                // rows above's reason: these are a **consequence** of the tick
                // rather than entries the operator manages, and drawing them
                // like the list would invite a Remove button that cannot exist.
                let text = dir.display().to_string();
                ui.add(egui::Label::new(egui::RichText::new(&text).weak().small()).truncate())
                    .on_hover_text(&text);
            }
        }
    }
    crate::diag::ui_rect(REGION, ui.min_rect());
}

/// **May pdfcer fake a bold or an italic that the page has no real face for?**
pub fn style_policy(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::style_policy_title(),
        t::style_policy_silence(),
        t::style_policy_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.style_policy,
        StylePolicy::Auto,
        t::style_policy_auto_label(),
        Some(t::style_policy_auto_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.style_policy,
        StylePolicy::Warn,
        t::style_policy_warn_label(),
        Some(t::style_policy_warn_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.style_policy,
        StylePolicy::Refuse,
        t::style_policy_refuse_label(),
        Some(t::style_policy_refuse_note()),
    );
    widgets::disclosure(ui, t::style_policy_bound());
    crate::diag::ui_rect(STYLE_POLICY_REGION, ui.min_rect());
}
