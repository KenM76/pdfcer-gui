//! The named checks — **the index of which checks exist.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/mod.md`.

/// O263: Align left edges moves every box, in one undo.
pub mod align_left;
pub mod annot_angle_typed;

pub mod annot_delete_gate;

/// **The ninth handle on an ANNOTATION** — draw a shape, grab the rotate
/// handle, and it turns.
pub mod annot_rotate;

pub mod blend_space;
pub mod export_keep_text;
pub mod field_outline;
pub mod merge_runs;
pub mod ocr_colour_setting;
pub mod ocr_layer_view;
pub mod off_page_blank_overhang;
pub mod overprint_default;
pub mod paragraph_on_drawing;
pub mod properties_restyle_word;
pub mod remove_ocr;
pub mod repair_form_fonts;
pub mod rich_copy;
pub mod run_width;
/// A selection dropped on another window is pasted there; Shift moves it.
pub mod selection_drop;
pub mod smooth_pictures;
pub mod snapshot_picture;
/// One text object split into one per line, and a split the engine refuses.
pub mod split_lines;
pub mod spots_flattened;
/// A document tab dragged off the strip moves or tears off, keeping its page.
pub mod tab_drag;
pub mod text_render_mode;
pub mod text_spacing;
pub mod undrawn_fields;

pub mod checkbox_resize;

pub mod comment_popup;

pub mod comments_census;

pub mod band_cmyk;
pub mod bilevel_pictures;
pub mod compact_classes;
pub mod compact_closeness;
pub mod compact_compare;
pub mod compact_content;
pub mod compact_fonts;
pub mod compact_pictures;
pub mod compact_reach;
pub mod compact_round;
pub mod compact_target;
pub mod compact_version;
pub mod compact_web;
pub mod filelink_prompt;
pub mod freehand_arrowheads;
pub mod freehand_detail;
pub mod link_cmyk;
pub mod objects_multiselect;
pub mod pen_cmyk;
pub mod point_click_delete;
pub mod shrink_pictures;
pub mod store_appearances;
pub mod weblink_prompt;

pub mod compact_save;

/// Doc-only: how a check is written and what makes one admissible.
pub mod conventions;

pub mod delete_key;

/// The Manage-dimension-groups window: it opens, it creates a group, and the
/// group comes back joinable.
pub mod dimension_groups;

pub mod bates_batch;
pub mod bates_scripted;
/// Double-clicking a text box edits the text (O70).
pub mod double_click_text;
pub mod double_click_text_scripted;
pub mod evidence_scripted;
pub mod export_tables_choices;
pub mod export_tables_scripted;
pub mod export_word_scripted;
pub mod extract_pages_labels;
pub mod labels_scripted;
pub mod measure_area_scripted;
pub mod measure_place_scripted;
pub mod purge_passwords_scripted;
/// Pages ▸ Split… writes the files its preview listed, by each rule.
pub mod split_pages;
/// File ▸ Export ▸ the QDF round trip, and its stale-base refusal.
pub mod structure_round_trip;

pub mod driving;

/// Helpers shared by checks that press through the scripted pointer.
pub mod scripteddrive;

pub mod drop_onto_thumbnails;

pub mod embed_bundled;

pub mod embed_fonts;
pub mod enlarge_scans;

/// Export to DXF: the file reaches disk, and its contents agree with the
/// counts the shell reported.
pub mod export_dxf;
pub mod export_dxf_options;
pub mod export_dxf_pages;
pub mod export_image_standard;
pub mod small_first_page;
pub mod small_text;
pub mod system_theme;
pub mod tab_glyphs;
pub mod tabs_look;

pub mod export_form_data;

pub mod foreign_icon_name;

pub mod import_text;

/// **Insert blank pages** — several sheets land as separate page objects, of
/// the chosen size, and one undo removes them.
pub mod insert_blank_pages;

pub mod line_weights;

pub mod load_anomalies;

pub mod picking;

pub mod reaching;

pub mod recovery_losses;

pub mod copy_as_vector;
pub mod copy_as_vector_scripted;
/// O285: an SVG and an EMF placed through Insert image, without the mouse.
pub mod insert_drawing;

// O120's fourth export format, and the one a driven check is worth most for:
// EMF is the ONLY vector route LibreOffice 24.x and Word's Paste Special have,
// so a radio that draws and does not bind hands the operator a file those
// programs open as an empty frame. ⬜ NOT RUN — see the module header, which
// says so in its own words rather than leaving an absent result to imply it.
pub mod export_image_emf;

pub mod export_remembered;

pub mod export_text;

/// The same defect one `/Subtype` along: a certified document's FORM FIELDS.
pub mod field_delete_gate;

pub mod field_menu;

pub mod find_bar;

pub mod find_options;

pub mod form_field;

pub mod form_ghost;

pub mod form_groups;

pub mod form_leaf_descend;

pub mod alignmixed;
pub mod annot_restyle;
/// Shared driving for the ce dimension checks on `dimension-scaled.pdf`.
mod dimdrive;
pub mod dimension_colour_cmyk;
pub mod dimension_text_select;
pub mod dimension_tool_options;
pub mod form_leaf_move;
pub mod form_node_move;
pub mod form_part_copy;
pub mod form_part_delete;
pub mod form_style;
pub mod form_text_reshape;
pub mod group_unit_converts;
pub mod leafplace;
pub mod leaftransform;
pub mod markup_reshape;
pub mod nodeshape;
pub mod nudgecontent;
pub mod replaceimage;
pub mod restack;
/// The annotations a saved file finally lists on its pages.
mod savedannots;
pub mod stroke_style;
/// A typed move and turn applied together are one Undo.
pub mod typedonestep;
/// A typed X over several selected objects moves them together, as one Undo.
pub mod typedseveral;
pub mod typedturn;

/// **Where to click so that a form-field selection CHANGES**, shared by the
/// three checks that author a field and then try to select it.
pub mod formaim;

pub mod graphics_pressure;

/// Insert an image: the picture reaches the page, and the resolution the
/// window promised is the one the document reports.
pub mod insert_image;

pub mod insert_image_place;

/// The left rail — O123 part 7. ⚠ WRITTEN, NOT RUN: see the module header.
pub mod left_rail;

pub mod legibility;

pub mod markup_band;

pub mod markup_move;

/// A new markup is drawn in Acrobat's red, measured off the screen.
pub mod markup_palette;

pub mod markup_rectangle;

pub mod markup_resize_preview;

pub mod markup_shapes;

/// An open dock's hide arrow sits on its inner edge, clear of every tab.
pub mod hide_arrow;
/// Edit's right dock is one tab bar holding the inspector panels.
pub mod inspector;

pub mod progressive;

pub mod read_image_copy;

pub mod signature_save;

pub mod smart_select;

pub mod revocation_sources;
pub mod revocation_verdict;
pub mod trust_store;

pub mod unembed_fonts;

pub mod widget_dash;
pub mod widget_move;

pub mod widget_rotate;

/// A bookmark subtree can be copied and pasted (O59).
pub mod bookmark_clipboard;

pub mod cut_gate;

/// A ce dimension drag previews the dimension the release writes.
pub mod dimension_circular_label_drag;
/// **A shape the operator drew can gain and lose a corner** — his own
/// report. ⬜ WRITTEN AND NOT DRIVEN; see the module header.
pub mod dimension_corner_count;
/// A circular ce dimension switches radius ↔ diameter from its right-click menu.
pub mod dimension_display_menu;
pub mod dimension_extension_grip;
pub mod dimension_label_drag;
/// A linear ce dimension's text slides along its line; the line stays.
pub mod dimension_text_slide;

pub mod field_clipboard;

pub mod form_selection;

pub mod link_follow;

pub mod point_destination;

pub mod markup_node_edit;

/// A Review shape drawn with the pen set to draw on the page.
pub mod markup_on_page;
/// Markup ▸ Style — a ribbon group whose one item the manifest declared at S2
/// and no renderer ever drew, so it shipped as a caption over an empty band.
pub mod markup_style;

pub mod measure_calibrate;

/// The operator's own report (O105), driven on a fixture built to reproduce
/// it: one path object holding a small circle and forty unrelated segments.
pub mod measure_circular_points;

pub mod measure_hover;

pub mod measure_linear;

pub mod measure_perimeter;

pub mod menu_icons;

/// **Arming another tool writes the text draft** — `OPERATOR_REQUESTS.md`
/// O222.
pub mod navigate_commits_text;

pub mod new_document;

pub mod new_document_size;

pub mod ocr;
pub mod ocr_by_layout;

/// A Settings OCR folder reaching the model drop-down.
pub mod ocr_extra_folder;

/// A Tesseract program add-on: listed, run, disclosed, and refused by policy.
pub mod ocr_program_addon;

/// The text pen's CMYK and opacity reaching the saved run.
pub mod add_text_ink_scripted;
pub mod invisible_text_scripted;
pub mod ocr_edit_preview;
pub mod ocr_export_filter;
pub mod ocr_export_scripted;
pub mod ocr_fetch_models;
pub mod ocr_layer_group;
pub mod ocr_layer_kept;
pub mod ocr_layer_wysiwyg;
pub mod ocr_live_search;
pub mod ocr_models_common;
pub mod ocr_models_install;
pub mod ocr_models_manage;
pub mod ocr_models_unwritable;
pub mod ocr_progress;
pub mod ocr_reading_order;
pub mod ocr_scripted;
/// A clicked text object's opacity, read, set and saved.
pub mod text_opacity_scripted;

/// **The operator can FIND OUT a sheet has content beside it** — the off-page census.
pub mod off_page_census;

/// Off-page content never costs the operator his zoom (O218, O221).
pub mod off_page_ceiling;

/// **A band dragged into the grey margin reaches an object off the page** (O92).
pub mod off_page_marquee;

pub mod off_page_press;

pub mod off_page_toggle;

pub mod off_page_visible;

pub mod off_page_wall;

pub mod off_page_zoom;

pub mod off_sheet;

pub mod turned_region;

pub mod os_fonts_setting;

pub mod page_cache;

pub mod page_clipboard;

pub mod page_display_pref;

pub mod page_display_recentres;

pub mod page_ops;

pub mod page_prefetch;

/// A fresh profile draws page previews with no time limit.
pub mod page_previews_default;
pub mod page_previews_pref;

/// Fit onto a new sheet scales the drawing by the factor the window quoted.
pub mod page_scale;

pub mod page_size;

pub mod pagetree_guard;

pub mod password_prompt;

pub mod quit_unsaved;

pub mod save_as;

pub mod scale_ratio_units;
pub mod scale_reads_the_group;

pub mod tab_navigation;

pub mod tab_object_ring;

pub mod unshare_form;

pub mod display_two_rows;

pub mod field_shading;

pub mod canvas_choice_fill;
pub mod option_arrows;

/// Hide, print and show an annotation again from Properties and Comments.
pub mod annot_flags;
/// Attach a file to a page as a marker from Markup.
pub mod attach_file;
/// Attach a WAV recording to a page from Markup.
pub mod attach_sound;
/// Choose a check box's mark from Properties.
pub mod check_mark;
/// Give a multi-select list several default choices from Properties.
pub mod choice_defaults;
/// A selection copied in one window pastes as objects in another.
pub mod cross_window_paste;
/// Create a self-signed digital ID in the Sign window and sign with it.
pub mod digital_id;
/// Document properties says which page boxes were not used as written.
/// One edit re-renders only its page, never blanks a picture and never blocks.
pub mod edit_never_blanks;
/// An encrypted file edits under its password, or offers its owner password.
pub mod encrypted_edit;
pub mod field_extras;
/// A text field calculated from others and given a range, from Properties.
pub mod field_scripts;
pub mod field_text_cmyk;
pub mod form_preview;
pub mod forms_spotlight;
/// Forms ▸ Style's width reaches the next placed field.
pub mod forms_style;
/// A field's alignment chosen, then handed back to what the form states.
pub mod inherited_alignment;
/// Mark an insertion with a caret from Markup.
pub mod insert_text;
/// Put a selected object or annotation on a layer.
pub mod layer_assign;
/// Create, rename and delete a layer from the Layers panel.
pub mod layer_authoring;
/// Merge a layer into another and flatten all, from the Layers panel.
pub mod layer_combine;
/// Folders and moves in the Layers panel's tree.
pub mod layer_folders;
/// A parent switch carries its sublayers; a band skips hidden layers; Objects headings.
pub mod layer_hiding;
/// Make a markup part of the page from its right-click menu.
pub mod markup_flatten;
/// Markup ▸ Media clip: a dragged region carries the clip, its type and its choices.
pub mod media_clip;
/// A 3D model opens from the page and turns about the chosen up.
pub mod model_axes;
pub mod new_radio_mark;
/// The Objects panel says where its list and the page disagree.
pub mod object_disagreements;
/// Open several documents from one Open.
pub mod open_many;
pub mod page_boxes;
pub mod page_print_boxes;
pub mod password_fill;
/// A slow default printer does not hold the Print window shut.
pub mod print_colour;
pub mod print_presets;
/// The colour render notes and View ▸ Display ▸ Skip tiny details.
pub mod print_shop;
pub mod print_slow_printer;
pub mod properties_pane;
/// Edits to an RC4 file wait for the operator, and a save says what they cost.
pub mod rc4_append;
/// A redraw says what it did to a field's text.
pub mod redraw_layout;
/// Remove metadata writes a copy without the items ticked.
pub mod remove_metadata;
/// Remove OCR text takes only the pages chosen.
pub mod remove_ocr_pages;
/// Propose replacement words for selected text from Markup.
pub mod replace_text;
/// Document properties ▸ Security notes and the open-time wrapper warning.
pub mod security_notes;
/// A password typed into an earlier saved version is said, field and version.
pub mod security_passwords;
/// The Security tab's two bands, and the rail's snapshot row.
pub mod security_tab;
/// Straighten scans turns the tilted pages and leaves the rest.
pub mod straighten_confidence;
pub mod straighten_scans;
pub mod tab_tips;
/// The page thumbnails zoom without a blank tile.
pub mod thumbnail_zoom;
pub mod widget_bg_cmyk;

pub mod layers_membership;

pub mod layers_search;

pub mod pages_drag;

pub mod dock_drop;

pub mod panel_carry;
pub mod panel_float;

pub mod preset_group_reachable;

pub mod preview_popout;
/// Operator request O238: poster printing.
pub mod print_poster;
pub mod print_preview_detail;

pub mod print_auto_paper;

pub mod print_clip_claim;
pub mod print_position;

pub mod print_dialog;

pub mod print_dismissal;

pub mod print_layout;

pub mod print_paper;

pub mod print_remembered;

/// The Properties panel's document-metadata half: a title typed into it
/// reaches the file, and an undo takes it back out of the box too.
pub mod properties_metadata;

/// `OPERATOR_REQUESTS.md` O123 part 2, driven — every control the Tool
/// panel held is on screen in Properties, its new home.
pub mod properties_tool;

/// O119, driven — Security ▸ Security reports the document's own state and refuses
/// a signed document instead of drawing a form. ⚠ Written and NOT RUN.
pub mod protect;

pub mod qat_icons;

pub mod read_mode;

pub mod redact_image_warning;

pub mod signing;

pub mod tab_order_drag;

pub mod page_tabs_chooser;

pub mod title_build_stamp;

pub mod render_diagnostics;

pub mod bezier_handle;

pub mod clipboard_annotation;
/// **The one assertion no unit test in this workspace can make** — what is
/// actually on the operating system's clipboard after Ctrl+C.
pub mod cloud_border;

pub mod clipboard_mode;

pub mod clipboard_text;

/// **O89's object route** — the colour control on the text you CLICKED,
/// where `font_group` asserts only the sentence telling you to sweep.
pub mod colour_clicked_text;

/// **The operator's own MAX_PIXMAP_EDGE failure, driven** — zoom past the
/// ceiling and assert the page still renders.
pub mod deep_pan;

pub mod deep_zoom;

pub mod deeper_rung_delete;

/// What another program copied becomes a new PDF, or pages after this one.
pub mod clip_pages;
/// In Review a dropped picture or drawing becomes a stamp; Read places nothing.
pub mod drop_stamp;
/// A picture dropped on a page lands where it was dropped, through the
/// scripted pointer's `drop` step.
pub mod dropped_file;
/// A PDF dropped on an open document asks whether to open, insert or place it.
pub mod dropped_pdf;
/// A text file dropped on the window becomes pages after the one on screen.
pub mod dropped_text;
/// A form's pages inserted into a plain document bring their fields.
pub mod insert_fields;
/// In Review a copied picture pastes as a stamp.
pub mod paste_stamp;

/// Every window's title bar follows the Dark and Quiet presets.
pub mod caption_theme;

pub mod enter_newline;

/// **Escape on a text draft WRITES it, and `Ctrl+Z` takes it back** —
/// `OPERATOR_REQUESTS.md` O223.
pub mod escape_commits_text;

/// **A draft on existing page text is drawn in that text's own font** — O247.
pub mod text_in_own_font;

pub mod first_frame;

pub mod fit_left_by_a_pan;

pub mod fit_places_the_view;

pub mod font_group;

/// **The same two surfaces as `font_group`, on the document the operator
/// names** — his SolidWorks drawing rather than a committed fixture.
pub mod font_group_real;

/// The **typed** route to a resize — the Properties panel's X/Y/W/H fields.
pub mod geometry_fields;

/// **The zoom readout is a button now, and buttons must be proved to do
/// something** — O24.
pub mod max_zoom;

pub mod move_line_of_text;

pub mod multi_node;

pub mod object_clipboard;

pub mod pan_refresh;

pub mod raster_wall;

pub mod reach_out;

pub mod read_mode_chrome;

pub mod read_mode_exit;

pub mod redact_preview;

pub mod redact_chunk;

pub mod redact_menu;

pub mod redact_plural;

pub mod redact_selection;

/// **Redaction** — the removed text is proved gone by a byte scan of the saved file.
pub mod redaction;

pub mod redaction_reach;

pub mod reflow;

/// An open paragraph's width handle, dragged in: the paragraph gains lines.
pub mod reflow_width;

/// A four-click burst on a line takes its paragraph.
pub mod paragraph_select;

/// Paragraphs switched off: a click in a paragraph opens one line.
pub mod paragraphs_off;

/// Paragraphs switched off for one page: the next page still opens them.
pub mod paragraphs_page;

/// One paragraph sent back to lines: its neighbour still opens whole.
pub mod paragraph_here;

pub mod panel_toggles;
/// Clipboard leads Edit, Save follows File, the highlight band is yellow.
pub mod ribbon_order;
pub mod tab_faces;
/// The Content band's word gap re-extracts the page.
pub mod word_gap_band;

/// A Select click on recognised text takes its whole block.
pub mod ocr_select_block;

/// One click selects a paragraph drawn from several text objects.
pub mod paragraph_pick;

pub mod resize;

pub mod restyle_text;

pub mod ribbon_captions;

pub mod run_menu_route;

/// The zoom ladder and the closed-loop aim `scale_sweep` drives its battery
/// with. Separate from it under R2; its header carries the seam.
pub mod scale_aim;

pub mod scale_sweep;

pub mod scripted_keys;

pub mod preview_width;

/// **The chooser offers a face the document does not contain** —
/// `pdfcer-core` v0.15.0's standard-14 authoring, reached from a font list.
pub mod refused_character_face;

// The band's PROPORTIONS against `mockups/pdfcer-shell.html`, and the two
// claims about it that only a rendered screenshot can settle: a resting
// control drawn with no frame, and a control drawn with no glyph. UNRUN --
// see its header for why, and for the command that runs it.
pub mod ribbon_mockup;

pub mod rotate;

pub mod scale_switch;

pub mod shape_preview;

pub mod std14_face;

/// **Font file…** — restyle swept text into a face from a file on disk,
/// embedding the subset it needs; asserted on the trace and the pixels.
pub mod font_file_face;

pub mod autosize_overflow;

pub mod rotated_text;

pub mod save_copy;

pub mod save_in_place;

/// **Does scrolling a long way cost the canvas its pointer input?**
pub mod scroll_input;

/// **The selection filter is load-bearing, not decorative** — switching a
/// class off changes what the next click on the same pixel selects.
pub mod select_filter;

pub mod settings_headings;

pub mod shift_constrains;

pub mod tool_row;

/// O232: the coloured-icons option is off by default and reaches the icons when on.
pub mod colour_icons;

pub mod wheel_flips_pages;

/// A letter a subset lacks is added from its installed face.
pub mod augment_subset;
/// Arrow keys in a text draft move neither focus nor the page.
pub mod draft_arrows;
/// Undo, redo, word delete, Tab, paste, select-all and save inside a draft.
pub mod draft_keys;
/// A paragraph of two looks opens whole and keeps both through an edit.
pub mod draft_mixed;
pub mod draft_mixed_ink;
/// A paragraph re-opens as one draft and keeps its hand breaks.
pub mod draft_paragraph;
/// Enter in a paragraph on the page breaks it.
pub mod enter_paragraph;
/// The Select filter's Form fields and Characters rows change what a click reaches.
pub mod filter_rows;
/// A paragraph's hand breaks are read from pdfcer's marks, or the guess is disclosed.
pub mod hand_breaks;
pub mod ime;
/// A letter no page font has is typed in a face from the font folders.
pub mod installed_letter;
/// A key typed mid-line is previewed where the commit puts it.
pub mod mid_line_preview;
/// Open in Acrobat asks, cancels cleanly, and hands the file to the configured viewer.
pub mod open_in_acrobat;
/// Edge keys and multi-line paste open a line's paragraph; recognised Enter declines.
pub mod paragraph_joins;
/// A stand-in preview is the text's size, and the status bar says why.
pub mod preview_fallback;
pub mod recognised_paragraph;
pub mod reface_commit;
/// Every refused key is named, with one click to a face that has them.
pub mod refused_keys;
/// Ctrl+B on the second copy of a repeated word bolds that copy.
pub mod repeated_word;
/// The Find bar's Replace and Replace all, and Replace all as one undo.
pub mod replace_all;
/// A comment stored as formatted text names its formatting in the panel.
pub mod rich_comment;
/// Double, triple and Shift+arrow selection, a drag over text, rotated carets.
pub mod selection_gestures;
/// A strikethrough says where its line's height came from.
pub mod strike_source;
/// Ctrl+B and Ctrl+I pressed again take their style off.
pub mod style_off;
/// A letter an embedded subset outlines but never showed can be typed.
pub mod subset_glyph;
/// Text made bold by a stroke shows Bold pressed, and Ctrl+B takes it off.
pub mod synthetic_bold;
/// Part of a tagged paragraph underlined: the structure tree's gap is said.
pub mod tagged_underline;
/// Text-tool clicks reach a note, a field or a scan's remedy.
pub mod text_click_routes;
pub mod text_tool_click_types;
/// Ctrl+U marks the characters themselves, reads back pressed, and toggles off.
pub mod tied_underline;
/// Text holding a letter its font draws two ways still edits around it.
pub mod two_codes;
/// A document moves to another window with its unsaved edits and password.
pub mod window_carry;
/// A document moves to another window, or tears off into its own.
pub mod window_move;
pub mod word_line_edit;
pub mod word_styles;
/// A refused text edit offers the way the engine can make it.
pub mod workaround_offer;

pub mod zoom_burst;
pub mod zoom_gallery;
pub mod zoom_notch_walk;

pub mod zoom_keeps_place;

pub mod zoom_out_keeps_place;

/// **Opening a second PDF adds a tab, and the tab switches to it** — the
/// registration half of multi-document, which no unit test can observe.
pub mod document_tabs;

/// **A page dragged out of one open document and into another**, through a
/// spring-loaded tab — the operator's request, end to end.
pub mod page_drag_between_documents;

pub mod tab_reorder;

pub mod panel_tab_reorder;

pub mod about;

pub mod add_text;

/// Insert a form's pages to make orphaned widgets, then register one back.
pub mod adopt_widget;

pub mod attachment_clip;

pub mod attachments;

/// A 3D model placed from the ribbon, listed, and saved back byte for byte.
pub mod models;

/// A part-coloured 3D model draws in its colours, in the viewer and on the page.
pub mod model_colours;
/// The 3D viewer draws a compressed mesh in part, under a chosen fit.
pub mod model_fit;
/// The 3D viewer says when an assembly coloured the model.
pub mod model_inferences;
pub mod model_tree;
/// The 3D viewer saves the view on screen as a PNG picture.
pub mod model_view_picture;
/// The view chosen in the 3D viewer, or a picture file, becomes the page picture of the model.
pub mod model_view_poster;

/// A push button takes a picture, moves its caption beside it, and loses it.
pub mod button_icon;
/// A picture pasted while a push button is selected becomes its picture.
pub mod button_icon_paste;
/// A placed 3D model shows the engine's picture of it on the page.
pub mod model_poster;
/// The 3D viewer opens on the file's own saved view, and *File's view* returns
/// to it.
pub mod model_saved_view;
/// While Play 3D content is off, View… on a model asks first.
pub mod model_trust;
/// A placed 3D model opened in the viewer turns, pans and zooms under the
/// scripted pointer.
pub mod model_view;
/// The 3D viewer zooms about the pointer, fills the screen and comes back.
pub mod model_view_window;
/// The 3D viewer saves its named views into the file.
pub mod model_views_saved;
/// A picture another program copied pastes at the pointer; the newer copy wins.
pub mod os_image_paste;
/// Ctrl+V pastes another program's picture when the clipboard holds no text.
pub mod paste_chord;
/// Text another program copied pastes as a text box at the pointer.
pub mod paste_text;
/// Security ▸ Trust's switches flip and are saved; Web sites… forgets a site.
pub mod trust;
/// Every kind of form field loses its border through Properties' No border.
pub mod widget_no_border;

pub mod band_scroll;

pub mod block_nav;

/// A bookmark can be written into a document that has no outline.
pub mod bookmark_add;

pub mod bookmark_dest;

pub mod bookmark_edit;

pub mod settings_nav;
/// A shortcut changed in Settings reaches the keys and survives a restart.
pub mod shortcut_change;
/// A snapshot box's outline marches and the page inside it holds still.
pub mod snapshot_ants;
/// A snapshot box stays on its page area through a zoom.
pub mod snapshot_box;
/// Copy with a snapshot box laid puts the box on the clipboard, cut to it.
pub mod snapshot_copy;
/// The snapshot resolution typed in Settings survives a restart.
pub mod snapshot_dpi;
/// A snapshot box moves, resizes and clears with its tool.
pub mod snapshot_grips;
/// A snapshot copy pastes back into pdfcer as a drawing, not a picture.
pub mod snapshot_paste;
pub mod snapshot_save;

pub mod bookmark_move;

pub mod button_action;

pub mod chords;

pub mod comment_note;

pub mod custom_stamp;

/// **O173's ask-once offer**, and the only check in the suite that has to
/// UNDO the sandbox's own seed before it can measure anything — see its header.
pub mod default_app_offer;

pub mod dialog_windows;

pub mod draft_selection;

pub mod marquee_table;

/// Recognise text's editable output takes the words out of the scan.
pub mod ocr_editable;
pub mod ocr_text_select;
/// A recognised line on a turned page previews upright while edited.
pub mod ocr_turned_edit;
pub mod ocr_turned_page;
pub mod ocr_upscale;
pub mod ocr_word_lists;

pub mod save_after_edit;

pub mod settings_theme;

/// The keyboard reference lists every chord, and every chord names a command.
pub mod shortcuts;

pub mod stamp_collection;

pub mod stamp_dialog_reopen;

pub mod stamp_size;

/// **O309** — CAD text set `1 Tf` under a scaled matrix reads and takes its
/// drawn size.
pub mod scaled_text_size;

/// **O310** — a restyle of a many-run text object keeps the window painting
/// and is one undo entry.
pub mod restyle_keeps_painting;

/// **O326** — Tools > Compare lists an older version's changes, jumps to one
/// and marks them as comments.
pub mod compare_documents;

pub mod stamp_size_properties;

pub mod text_annot;

pub mod text_annot_focus;

pub mod text_box;

pub mod text_chunks;

pub mod chunk_click;

pub mod chunk_multi_move;

pub mod chunk_band;

pub mod chunk_ghost;
pub mod chunk_ghost_pixels;

pub mod chunk_empty;

/// **The text-editing round trip** — `DEFECTS.md` D4b: the edit reaches the
/// bytes and the text the operator did not touch does not move.
pub mod text_edit;

/// The operator's own report, driven: Edit text on a real CAD sheet, aimed
/// where the engine says text is.
pub mod text_edit_real;

/// Marking a text selection — underline, strikeout, squiggly — and the
/// control correctly disabled without one.
pub mod text_markup;

pub mod text_selection;

pub mod text_tool;

pub mod theme_page;

/// **A refusal an operator can read** — `OPERATOR_REQUESTS.md` O140, driven
/// on the file he reported it on.
pub mod typo_refusal;

pub mod ui_scale;

/// Several annotations, one locked, deleted by one Delete and restored by one Undo.
pub mod annot_delete_many;
/// Several annotations, one locked, dragged together and restored by one Undo.
pub mod annot_group_drag;
/// Two markups resized and turned by one grip drag each, one undo step each.
pub mod annot_group_scale;
/// Several markups, a link and a ce dimension selected and aligned as one undo step.
pub mod annot_multiselect;
/// An Align over more annotations than one undo step holds, refused with a reason.
pub mod annot_too_many;
/// Several form fields duplicated, cut and pasted, each as one undo step.
pub mod field_clip_many;
/// Several form fields deleted by one Delete as one undo step.
pub mod field_delete_many;
pub mod field_delete_panel;
/// Several form fields dragged together by any member, as one undo step.
pub mod field_group_drag;
/// Several form fields selected by Shift-click and marquee, aligned as one undo step.
pub mod field_multiselect;
/// A Shift-marquee adds form fields to the selection; a Ctrl-marquee removes them.
pub mod fieldbandmerge;
/// Arrow keys over several annotations, several form fields, or a ce dimension.
pub mod groupnudge;
/// Moving, resizing, copying and deleting a link, its quads following its box.
pub mod link_quads;
/// Drawing a link and repointing a selected one.
pub mod links;
/// Nudging, duplicating, re-stacking and typing the size of a markup shape.
pub mod markup_place;
/// Shift-click, marquee, save and saved-edge steps shared by the multi-select checks.
pub mod multiselect;
/// A radio button joining a group by paste, by its group row and by rename.
pub mod radio_group;
/// Copy, Cut and Duplicate over several annotations, a page object, a drawing part.
pub mod selection_clip;
pub mod undo_redo;
pub mod widget_turn;

// `crate::checks::CheckReport` — most check modules spell the report
// type as a sibling of the trait they implement, which reads correctly and
// resolves because a private `use` is in scope for the whole module subtree
// beneath it. It stays HERE rather than travelling with the trait: a name
// imported into `harness` is in scope for `harness`, not for `checks::*`.
use crate::report::CheckReport;

/// **The contract** — the [`Check`] trait and the [`CheckContext`] every check
/// is handed.
mod harness;

pub use harness::{Check, CheckContext};

/// **The roster** — every check, in the order the suite runs them.
mod roster;

pub use roster::all;
