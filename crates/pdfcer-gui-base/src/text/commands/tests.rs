//! `text::commands::tests` — the properties every command's copy must hold.
//!
//!
//! Nothing moved but the module wrapper. Every test below is byte-identical
//! to what it was, de-indented by one level, so a failure here reads exactly
//! as it did before the split.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/commands/tests.md`.

use super::*;

/// Every command in this catalog, so the rules below are checked
/// against all of them rather than against whichever ones somebody
/// remembered to list.
fn all() -> Vec<CommandText> {
    vec![
        file_new(),
        file_open(),
        file_close(),
        file_recent(),
        file_save_copy(),
        edit_reflow_block(),
        file_save_compacted(),
        file_export_dxf(),
        file_export_image(),
        file_stamp_collection(),
        file_export_form_data(),
        file_export_word(),
        file_copy_page_text(),
        file_copy_document_text(),
        file_print(),
        file_properties(),
        file_fonts(),
        file_settings(),
        file_shortcuts(),
        file_about(),
        file_ocr(),
        file_remove_ocr(),
        file_deskew(),
        view_page_single(),
        view_page_continuous(),
        view_page_facing(),
        view_page_facing_continuous(),
        view_zoom_actual(),
        view_zoom_fit_page(),
        view_zoom_fit_width(),
        view_zoom_fit_height(),
        view_show_annotations(),
        view_show_points(),
        view_rulers(),
        view_grid(),
        view_guides(),
        view_line_weights(),
        view_skip_tiny_details(),
        view_off_page(),
        view_sidebar(),
        view_panel_pages(),
        view_panel_bookmarks(),
        view_panel_layers(),
        view_panel_signatures(),
        view_panel_objects(),
        view_panel_forms(),
        view_read_mode(),
        view_fullscreen(),
        view_next_document(),
        view_previous_document(),
        view_close_other_documents(),
        view_reset_layout(),
        view_panel_float(),
        view_panel_dock(),
        view_panel_close(),
        view_dock_all_panels(),
        pages_insert_from_file(),
        pages_insert_from_clipboard(),
        file_new_from_clipboard(),
        pages_delete(),
        pages_extract(),
        pages_move_up(),
        pages_move_down(),
        pages_split(),
        pages_merge_into(),
        pages_rotate_left(),
        pages_rotate_right(),
        edit_text(),
        edit_add_text(),
        edit_insert_image(),
        edit_insert_3d(),
        edit_attachments(),
        edit_form_create_field(),
        edit_form_manage_fields(),
        edit_form_flatten(),
        edit_form_repair_fonts(),
        edit_redact(),
        edit_redact_apply(),
        edit_undo(),
        edit_redo(),
        markup_rectangle(),
        markup_ellipse(),
        markup_arrow(),
        markup_polyline(),
        markup_polygon(),
        markup_cloud(),
        markup_ink(),
        markup_finish(),
        markup_highlight(),
        markup_text_box(),
        markup_sticky_note(),
        markup_stamp(),
        markup_attach_file(),
        markup_sound(),
        markup_screen(),
        markup_insert_text(),
        markup_replace_text(),
        markup_paste_image_stamp(),
        markup_comments(),
        measure_linear(),
        measure_length(),
        measure_perimeter(),
        measure_area(),
        measure_radius_diameter(),
        measure_two_line(),
        measure_finish(),
        measure_set_scale(),
        measure_manage_groups(),
        tools_merge_files(),
        tools_split_files(),
        tools_font_folders(),
        tools_embed_fonts(),
        tools_unembed_fonts(),
        tools_render_diagnostics(),
        tools_ink_picker(),
        format_delete(),
        format_merge_text_runs(),
        format_split_text_lines(),
        format_dimension_diameter(),
        format_dimension_radius(),
        format_dimension_area(),
        format_dimension_perimeter(),
        markup_bring_to_front(),
        edit_bring_to_front(),
        edit_bring_forward(),
        edit_send_backward(),
        edit_send_to_back(),
        markup_bring_forward(),
        markup_send_backward(),
        markup_send_to_back(),
        mode_read(),
        mode_review(),
        mode_edit(),
    ]
}

/// **Every command has a non-empty label and a non-empty tooltip.**
#[test]
fn every_command_has_a_label_and_a_tooltip() {
    for t in all() {
        assert!(!t.label.trim().is_empty(), "empty label: {t:?}");
        assert!(!t.tooltip.trim().is_empty(), "empty tooltip: {t:?}");
    }
}

/// The Edit tab's Arrange commands, which take the Markup tab's words by
/// `RIBBON_IA.md` §5.4: the same act on a different kind of thing, never on
/// one surface together.
fn edit_arrange() -> [CommandText; 4] {
    [
        edit_bring_to_front(),
        edit_bring_forward(),
        edit_send_backward(),
        edit_send_to_back(),
    ]
}

/// **The shared Arrange labels are exactly the Markup tab's.**
#[test]
fn the_edit_arrange_labels_are_the_markup_ones() {
    let markup = [
        markup_bring_to_front(),
        markup_bring_forward(),
        markup_send_backward(),
        markup_send_to_back(),
    ];
    for (edit, markup) in edit_arrange().iter().zip(markup.iter()) {
        assert_eq!(edit.label, markup.label);
        assert_ne!(
            edit.tooltip, markup.tooltip,
            "the tooltip says which kind of thing"
        );
    }
}

/// **No two commands share a label**, apart from the Arrange pairs above.
#[test]
fn no_two_commands_share_a_label() {
    let shared = edit_arrange();
    let mut labels: Vec<&str> = all()
        .iter()
        .filter(|t| !shared.iter().any(|s| s.tooltip == t.tooltip))
        .map(|t| t.label)
        .collect();
    let total = labels.len();
    labels.sort_unstable();
    labels.dedup();
    assert_eq!(
        labels.len(),
        total,
        "two commands share a label — an operator cannot tell them apart"
    );
}

/// A tooltip is a sentence: it ends in punctuation.
#[test]
fn tooltips_are_sentences_and_labels_are_not() {
    for t in all() {
        assert!(
            t.tooltip.ends_with('.'),
            "a tooltip is prose and ends in a full stop: {:?}",
            t.tooltip
        );
        assert!(
            !t.label.ends_with('.'),
            "a label is a name and takes no trailing period: {:?}",
            t.label
        );
    }
}

/// **The three illegible labels are gone.**
#[test]
fn the_content_tools_have_real_labels() {
    assert_eq!(edit_text().label, "Edit text");
    assert_eq!(edit_add_text().label, "Add text");
}
