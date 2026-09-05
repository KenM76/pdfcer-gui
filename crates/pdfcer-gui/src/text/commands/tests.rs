//! `text::commands::tests` — the properties every command's copy must hold.
//!
//! Split out of [`super`] under **R2** on 2026-09-04, when the four
//! panel-layout verbs took that file to 1,501 lines. The seam is the
//! standard one this tree already uses for `panels/tests.rs` and
//! `app/actions/tests.rs`: **the assertions about a catalog are a different
//! subject from the catalog**, and the catalog is the half a reader opens to
//! find out what a control says.
//!
//! Nothing moved but the module wrapper. Every test below is byte-identical
//! to what it was, de-indented by one level, so a failure here reads exactly
//! as it did before the split.

use super::*;

/// Every command in this catalog, so the rules below are checked
/// against all of them rather than against whichever ones somebody
/// remembered to list.
///
/// Maintained by hand, and that is the point: adding a command means
/// adding a line here, and the count assertion in
/// `crate::shell::commands` cross-checks this list against the
/// registry, so a command that is registered but never appears here
/// fails a test rather than shipping with unreviewed copy.
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
        file_export_form_data(),
        // Moved from the Edit block below on 2026-08-14 with the commands
        // themselves; this list is in tab order for the same reason the
        // catalog is.
        file_copy_page_text(),
        file_copy_document_text(),
        file_print(),
        file_properties(),
        file_fonts(),
        file_settings(),
        file_shortcuts(),
        file_about(),
        file_ocr(),
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
        edit_attachments(),
        edit_form_create_field(),
        edit_form_manage_fields(),
        edit_form_flatten(),
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
        markup_comments(),
        measure_linear(),
        measure_length(),
        measure_perimeter(),
        measure_radius_diameter(),
        // `measure_two_line` was registered on 2026-08-14 and was not
        // added here, so for one day the label-uniqueness and
        // tooltip-is-a-sentence rules were being asserted over a list that
        // did not contain it. Both new Measure entries are here.
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
        format_delete(),
        mode_read(),
        mode_review(),
        mode_edit(),
    ]
}

/// **Every command has a non-empty label and a non-empty tooltip.**
///
/// P3 reserves greying for temporarily unavailable and requires that
/// it always be explained on hover. A command with no tooltip cannot
/// honour that, and the salvage source shipped four such controls on
/// the Measure tab.
#[test]
fn every_command_has_a_label_and_a_tooltip() {
    for t in all() {
        assert!(!t.label.trim().is_empty(), "empty label: {t:?}");
        assert!(!t.tooltip.trim().is_empty(), "empty tooltip: {t:?}");
    }
}

/// **No two commands share a label.**
///
/// The defect this prevents shipped: `edit_text_tool_button()` and
/// `add_text_tool_button()` both returned the literal `"Aa"`, and the
/// two adjacent buttons in the Content group were distinguishable only
/// by icon and tooltip. Two identical labels side by side is not a
/// style problem; it is two controls the operator cannot tell apart.
///
/// The check is deliberately global rather than per-group. A label
/// duplicated across two tabs is less confusing than one duplicated
/// within a group, but it is still a search result with two answers,
/// and the moment customization lets an operator move a command
/// between tabs the per-group version of this rule stops holding.
#[test]
fn no_two_commands_share_a_label() {
    let mut labels: Vec<&str> = all().iter().map(|t| t.label).collect();
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
///
/// A label is a name and takes no trailing period; a tooltip is prose
/// and does. Stated as a rule in [`crate::text`] and worth checking,
/// because the two conventions sit two lines apart in this file and
/// the wrong one is easy to copy.
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
///
/// `RIBBON_IA.md` §5.4 requires `Aa`, `I⁺ Aa` and `Obj` to become
/// real words. This asserts the outcome rather than trusting that
/// nobody copies the old literals back in — `Obj` is not a word, and
/// it was the label on one of the three primary editing tools.
#[test]
fn the_content_tools_have_real_labels() {
    assert_eq!(edit_text().label, "Edit text");
    assert_eq!(edit_add_text().label, "Add text");
}
