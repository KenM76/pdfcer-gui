//! # `dialogs::settings::acrobat` — where Acrobat is, when the operator has
//! had to say
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/settings/acrobat.md`.

use egui::Ui;

use crate::text::acrobat as t;

/// The region the resolved-state line publishes.
pub const REGION_RESOLVED: &str = "settings:acrobat.resolved"; // ui-text-exempt: trace region name, never displayed

/// The Browse button's region.
pub const REGION_BROWSE: &str = "settings:acrobat.browse"; // ui-text-exempt: trace region name, never displayed

/// Where Acrobat is — the field, its Browse button, and the line that says
/// what pdfcer currently resolves.
pub fn path(
    ui: &mut Ui,
    prefs: &mut crate::app::prefs::Prefs,
    resolved: Option<&crate::acrobat::Viewer>,
) {
    super::widgets::header(ui, t::path_title(), t::path_silence(), t::path_radius());
    super::widgets::text_value(
        ui,
        // ui-text-exempt: an egui control id, never displayed.
        "settings-acrobat-path",
        &mut prefs.acrobat_path,
        &t::path_label(),
        Some(&t::path_note()),
        Clone::clone,
        |typed| Some(typed.to_owned()),
    );

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let browse = ui.button(t::path_browse());
        crate::diag::ui_rect_visible(REGION_BROWSE, browse.rect, ui.clip_rect());
        let browse = browse.on_hover_text(t::path_browse_hover());
        // A picker as well as a field, because the value is a full path to a
        // program file and typing one from memory is how a letter goes
        // missing. `super::fonts`' Add-folder button is the same shape.
        if browse.clicked()
            && let crate::app::files::Picked::Path(picked) = crate::app::files::pick_acrobat()
        {
            prefs.acrobat_path = picked.display().to_string();
        }
    });

    ui.add_space(6.0);
    // The state line. `notice` rather than the body ink, because it is a
    // report about the machine rather than part of the setting — and a theme
    // role rather than a colour, per `tools/gates/check-theme-colors.sh`.
    let line = ui.label(
        egui::RichText::new(t::resolved_note(resolved))
            .color(egui_shell::theme::Theme::of(ui.ctx()).palette.notice),
    );
    crate::diag::ui_rect_visible(REGION_RESOLVED, line.rect, ui.clip_rect());
}
