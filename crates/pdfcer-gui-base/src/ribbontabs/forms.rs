//! The **Forms** tab — *what will someone filling this in be asked?*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/ribbontabs/forms.md`.

use super::{command, group};
use crate::text::ribbon;
use egui_shell::manifest::{Item, Tab};

/// The Forms tab.
pub fn tab() -> Tab {
    Tab::new("forms", ribbon::tab_forms())
        .with_icon("forms")
        .with_question(ribbon::question_forms())
        .with_groups([
            // The five field types, most-used first: text boxes outnumber
            // everything else on a real form. Each arms a placement tool.
            group(
                "fields",
                ribbon::group_forms_fields(),
                [
                    command("edit.form_text_field"),
                    command("edit.form_check_box"),
                    command("edit.form_radio_button"),
                    command("edit.form_choice"),
                    command("edit.form_push_button"),
                ],
            ),
            // How the NEXT field looks, never the selected one — the Markup ▸
            // Style rule. A selected field is restyled in Properties.
            group(
                "style",
                ribbon::group_forms_style(),
                [Item::custom(super::FORM_LOOK)],
            ),
            group(
                "manage",
                ribbon::group_forms_manage(),
                [
                    command("edit.form_manage_fields"),
                    command("edit.form_use_as_default"),
                    command("edit.form_flatten"),
                    command("edit.form_repair_fonts"),
                ],
            ),
        ])
}
