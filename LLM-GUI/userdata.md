# userdata: where settings live

The store is `<exe folder>\userdata\` when it is writable, otherwise
`%APPDATA%\pdfcer\`. It is resolved once per process. A copied exe therefore
gets its own store. Keep `userdata\` when updating; replace everything else.
No document password is ever written here.

| File | Holds |
|---|---|
| `settings.txt` | Engine settings, shared with `pdfcer.exe` (rendering, colour, text, writing) |
| `preferences.txt` | GUI behaviour: cache, zoom ceiling, print and export habits, remote control... |
| `layout.ron` | Dock layout and per-mode workspaces |
| `select-filter.txt` | Which object classes a click may select |
| `recent.txt` | Recent files: one absolute path per line, newest first, at most 10 |
| `guides.txt` | Guides placed per document |
| `page-display.txt` | Page display mode per document |

All but `layout.ron` are plain `key = value` text with `#` comments. An unknown
key or a bad value is reported, and only that setting falls back. Deleting a file
resets only what it held. Do not edit the user's files. Read them if needed,
and edit only a private copy's.

## preferences.txt keys

- General: `page_cache`, `render_quality`, `max_zoom_percent`, `zoom_settle_ms`,
  `ui_scale`, `colour_icons`, `wheel_paging`, `opening_fit`, `default_page_display`,
  `page_previews`, `page_preview_budget_ms`, `ribbon_auto_hide`, `rail_auto_hide`.
- Opening state: `show_rulers`, `show_grid`, `show_guides`, `smart_select`,
  `text_chunks`, `off_page.<mode>` (true/false; default off in read, on in review/edit).
- Editing: `paste_chords` (`new_field_first` | `acrobat`), `redaction_reach`,
  `chosen_standard`, `author_name`, `shade_form_fields`.
- Find: `find_zoom_on_jump`, `find_trim_query`.
- Fonts: `font_folder`, `use_os_fonts`.
- OCR: `ocr_engine`, `ocr_layer_colour`.
- Integration: `acrobat_path`, `acrobat_trust_store_path`, `ask_default_app`,
  `sign_timestamp_server`, `remote_control` (`ask` | `always` | `never`).
- Print: `print_collate`, `print_copies`, `print_custom_percent`, `print_duplex`,
  `print_line_auto`, `print_line_fixed`, `print_line_width_mm`, `print_markup`,
  `print_max_dpi`, `print_orientation`, `print_paper`, `print_poster`,
  `print_poster_cut_marks`, `print_poster_labels`, `print_poster_large_only`,
  `print_poster_overlap_mm`, `print_poster_percent`, `print_printer`,
  `print_reverse`, `print_scale`, `print_subset`, `print_tray_by_page_size`.
- Export: `export_dxf_{fit_arcs,text,units,version}`,
  `export_image_{dpi,format,keep_text,pages,quality,transparent}`,
  `export_tables_{format,pages}`,
  `export_text_{byte_order_mark,line_endings,order,pages,separator}`.

## settings.txt keys (engine)

`acrobat_trust_store`, `actual_text`, `all_process_spaces`,
`alternate_space_substitution`, `cmyk_intent`, `cmyk_jpeg_polarity`,
`device_cmyk_only`, `edited_stream_compression`, `grey_as_k_only`,
`image_minify`, `mask_resample`, `max_cmyk_buffer_bytes`, `mesh_patch_padding`,
`missing_as`, `overprint_zero_tint_scope`, `page_blend_space_source`,
`parallel_epsilon_degrees`, `quad_point_order`, `separations`,
`simulate_separations`, `spot_colorant_device_model`, `style_policy`,
`tab_row_tolerance`, `theme`, `trailing_eol`, `unmappable_code`,
`widget_tab_tail`, `word_gap_ratio`, `xref_entry_eol`.

Values and meanings: the Settings window (File ▸ Settings) explains each one.
