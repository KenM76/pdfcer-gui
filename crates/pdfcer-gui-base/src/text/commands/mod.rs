//! # text::commands — the label and tooltip of every ribbon command
//!
//! One function per command, each returning a [`CommandText`]. The ribbon's
//! *structural* strings — tab labels, tab questions, group captions, mode
//! labels — live next door in [`crate::text::ribbon`].
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/commands/mod.md`.

/// The two operator-visible strings a ribbon command carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandText {
    /// What the control says. Sentence case, no trailing period; an
    /// ellipsis when activating it opens a dialog rather than acting.
    pub label: &'static str,
    /// What the control says on hover. A full sentence, with punctuation.
    pub tooltip: &'static str,
}

impl CommandText {
    /// Pair a label with its tooltip.
    #[must_use]
    pub const fn new(label: &'static str, tooltip: &'static str) -> Self {
        Self { label, tooltip }
    }
}

/// The File tab's Save As copy, a module of its own under R2 — the same seam
/// [`annotate`] and [`view`] are drawn on. Re-exported, so callers keep
/// spelling it `text::commands::file_save_as`.
mod file;
pub use file::{
    file_export_image, file_export_tables, file_export_text, file_export_word, file_import_text,
    file_new_from_clipboard, file_save_as, pages_insert_from_clipboard,
};

/// **The View tab's entries**, a module of its own under R2.
mod view;

pub use view::*;

/// The **Format tab's** command copy, a module of its own under R2 — the
/// same seam [`file`], [`view`], [`annotate`], [`markupstyle`] and [`arrange`]
/// are drawn on.
///
/// Glob-re-exported, so callers keep spelling it
/// `text::commands::format_select_form()`.
mod format;

pub use format::*;

// ===========================================================================
// FILE TAB
// ===========================================================================

/// `file.open`
#[must_use]
pub const fn file_open() -> CommandText {
    CommandText::new("Open…", "Open a PDF document (Ctrl+O).")
}

/// `file.new`
#[must_use]
pub const fn file_new() -> CommandText {
    CommandText::new(
        "New",
        "Make a new document: one blank A4 page (Ctrl+N). It replaces what is open. Use Save a \
         copy to keep it; it is asked where to write every time, so the document itself stays \
         untitled.",
    )
}

/// `file.new_from_template`
#[must_use]
pub const fn file_new_from_template() -> CommandText {
    CommandText::new(
        "New from template…",
        "Choose a page size and make a new document: A0 to A6, Letter, Legal, Tabloid, the ANSI \
         engineering sizes, or a size you type. It replaces what is open.",
    )
}

/// `file.close`
#[must_use]
pub const fn file_close() -> CommandText {
    CommandText::new(
        "Close",
        "Close this document (Ctrl+W). You are asked what to do about unsaved edits \
         first. Your other open documents stay open.",
    )
}

/// `file.recent`
#[must_use]
pub const fn file_recent() -> CommandText {
    CommandText::new(
        "Recent",
        "Open one of the last ten documents you had open. A document stored on a drive that \
         is not connected right now is hidden from the list until it comes back; it is not \
         forgotten.",
    )
}

/// `file.save`
#[must_use]
pub const fn file_save() -> CommandText {
    CommandText::new(
        "Save",
        "Save this document over the file you opened (Ctrl+S). The edits are appended as an \
         update, so the previous version stays inside the file and nothing is thrown away. Use \
         Save a copy to write somewhere else instead.",
    )
}

/// `file.save_copy`
#[must_use]
pub const fn file_save_copy() -> CommandText {
    CommandText::new(
        "Save a copy…",
        "Write the document, including unsaved edits, to a file you choose (Ctrl+Shift+S). The \
         original is never overwritten unless you pick it, and the edits are appended as an \
         update so the previous version stays intact inside the file.",
    )
}

/// `file.save_compacted`
#[must_use]
pub const fn file_save_compacted() -> CommandText {
    CommandText::new(
        "Save a compacted copy…",
        "Write the whole document fresh to a file you choose, dropping anything no longer \
         used. Unlike Save a copy, this does NOT keep the previous version inside the \
         file and CANNOT keep a digital signature — so the copy may be much smaller, \
         and your original is left untouched. Use it after removing pages, images or \
         embedded fonts.",
    )
}

/// `edit.reflow_block`
#[must_use]
pub const fn edit_reflow_block() -> CommandText {
    CommandText::new(
        "Reflow paragraph",
        "Re-wrap a paragraph so its lines fill their box again, after retyping a sentence that \
         made a line too long or too short. First choose Edit text and click inside the \
         paragraph, then press this. It needs real prose — a title-block cell or a single \
         label is not a paragraph and cannot be re-wrapped — and it works on the document as \
         you opened it, so if you have already changed this file, save it and open it again \
         first.",
    )
}

/// `file.export_dxf`
#[must_use]
pub const fn file_export_dxf() -> CommandText {
    CommandText::new(
        "Export DXF…",
        "Write this page's lines, curves and text out as a DXF file that CAD and CNC software \
         can open.",
    )
}

/// `file.stamp_collection` — `OPERATOR_REQUESTS.md` **O169**.
#[must_use]
pub const fn file_stamp_collection() -> CommandText {
    CommandText::new(
        crate::text::stamps::command_label(),
        crate::text::stamps::command_tooltip(),
    )
}

/// `file.import_form_data`
#[must_use]
pub const fn file_import_form_data() -> CommandText {
    CommandText::new(
        "Import form data…",
        "Fill this document's form from an FDF, XFDF or CSV file, replacing any values it \
         names. One Ctrl+Z takes the whole import back.",
    )
}

/// `file.export_form_data`
#[must_use]
pub const fn file_export_form_data() -> CommandText {
    CommandText::new(
        "Export form data…",
        "Write this document's filled form values out as FDF, XFDF or CSV.",
    )
}

/// `file.copy_page_text`
#[must_use]
pub const fn file_copy_page_text() -> CommandText {
    CommandText::new(
        "Copy page text",
        "Copy this page's text to the clipboard (Ctrl+Shift+C). Where a PDF does not say where \
         words and lines end, pdfcer works it out from the position of the letters, and says \
         how much of the copy that was.",
    )
}

/// `file.copy_document_text`
#[must_use]
pub const fn file_copy_document_text() -> CommandText {
    CommandText::new(
        "Copy document text",
        "Copy every page's text to the clipboard. On a long document this can take a few \
         seconds, during which the window will not respond.",
    )
}

/// `file.print`
#[must_use]
pub const fn file_print() -> CommandText {
    CommandText::new(
        "Print…",
        "Set up and print this document. Nothing prints until you press Print in the dialog.",
    )
}

/// `file.properties`
#[must_use]
pub const fn file_properties() -> CommandText {
    CommandText::new(
        "Properties",
        "The properties of whatever is selected on the page — an object, a mark you have \
         placed, or a form field.",
    )
}

/// `file.document_properties`
#[must_use]
pub const fn file_document_properties() -> CommandText {
    CommandText::new(
        "Document properties",
        "This document's own title, author, subject and keywords — stored in the file and \
         travelling with it — and the facts pdfcer read about it.",
    )
}

/// `file.fonts`
#[must_use]
pub const fn file_fonts() -> CommandText {
    CommandText::new(
        "Fonts",
        "Show every font this document declares — type, encoding, embedded size, and whether \
         its embedded program could be removed.",
    )
}

/// `file.settings`
#[must_use]
pub const fn file_settings() -> CommandText {
    CommandText::new(
        "Settings…",
        "Choose how pdfcer reads and writes documents where the PDF standard leaves the answer \
         open — colour, printing separations, text extraction. Your choices are kept in a file \
         beside the program and survive restarts.",
    )
}

/// `file.shortcuts`
#[must_use]
pub const fn file_shortcuts() -> CommandText {
    CommandText::new("Keyboard shortcuts", "Show every keyboard shortcut.")
}

/// `file.about`
#[must_use]
pub const fn file_about() -> CommandText {
    CommandText::new(
        "About pdfcer",
        "Show this build's version, pdfcer's own licence, and the third-party material included \
         in the program.",
    )
}

/// `file.ocr`
#[must_use]
pub const fn file_ocr() -> CommandText {
    CommandText::new(
        "Recognise text…",
        "Read the words in a scanned page and add them as invisible text behind the image, so \
         Find and copy work. Every word is a guess and this recogniser scores none of them, so \
         you are shown what it read before anything is saved. The page still looks the same and \
         the scan is never re-encoded. Running it again replaces the text it recognised before.",
    )
}

/// `file.remove_ocr`
///
/// Says which text goes (pdfcer's own recognition only) and that the page's
/// look is untouched; greyed only while no document is open.
#[must_use]
pub const fn file_remove_ocr() -> CommandText {
    CommandText::new(
        "Remove OCR text",
        "Take out the invisible text pdfcer recognised in this document. The pages look the \
         same; Find and copy stop seeing that text. Text recognised by other programs is left \
         alone. One Ctrl+Z puts it all back. Greyed while no document is open.",
    )
}

/// File ▸ Recognise ▸ Straighten scans…. Says what turns and what does not,
/// and that pages with text are skipped by default.
#[must_use]
pub const fn file_deskew() -> CommandText {
    CommandText::new(
        "Straighten scans…",
        "Measure how far each scanned page is tilted and turn the scan back level. Pages that already have text are skipped unless you say otherwise, because that text would not turn with the picture. One Ctrl+Z undoes the run. Greyed while no document is open.",
    )
}

// ===========================================================================
// PAGES TAB
//
// Every command here operates on THIS document's page set and respects the
// thumbnail rail's selection when there is one. That is the tab's
// organising rule and it is what distinguishes it from Tools, which
// produces new files. The tooltips say so where the distinction is easy to
// get wrong — `pages.merge_into` against `tools.merge_files` especially.
// ===========================================================================

/// `pages.insert_from_file`
#[must_use]
pub const fn pages_insert_from_file() -> CommandText {
    CommandText::new(
        "Insert from file…",
        "Insert the pages of another PDF into this document, before or after the page you have \
         selected.",
    )
}

/// `pages.delete`
#[must_use]
pub const fn pages_delete() -> CommandText {
    CommandText::new(
        "Delete pages",
        "Remove the selected pages from this document. Undo reverses it.",
    )
}

/// `pages.extract`
#[must_use]
pub const fn pages_extract() -> CommandText {
    CommandText::new(
        "Extract…",
        "Write the selected pages out as a new PDF. This document is left unchanged.",
    )
}

/// `pages.move_up`
#[must_use]
pub const fn pages_move_up() -> CommandText {
    CommandText::new(
        "Move up",
        "Move the selected pages one place earlier in the document (Alt+Up).",
    )
}

/// `pages.move_down`
#[must_use]
pub const fn pages_move_down() -> CommandText {
    CommandText::new(
        "Move down",
        "Move the selected pages one place later in the document (Alt+Down).",
    )
}

/// `pages.split`
#[must_use]
pub const fn pages_split() -> CommandText {
    CommandText::new(
        "Split…",
        "Split this document into several files at page boundaries you choose.",
    )
}

/// `pages.merge_into`
#[must_use]
pub const fn pages_merge_into() -> CommandText {
    CommandText::new(
        "Merge into this document…",
        "Add the pages of one or more other PDFs to this document. To combine files into a new \
         one instead, leaving this document alone, use Tools > Merge files.",
    )
}

/// `pages.rotate_left`
#[must_use]
pub const fn pages_rotate_left() -> CommandText {
    CommandText::new(
        "Rotate left",
        "Turn the selected pages 90° counter-clockwise ([). This changes the document, not \
         just the view, and is saved with it — use Undo to reverse it.",
    )
}

/// `pages.rotate_right`
#[must_use]
pub const fn pages_rotate_right() -> CommandText {
    CommandText::new(
        "Rotate right",
        "Turn the selected pages 90° clockwise (]). This changes the document, not just the \
         view, and is saved with it — use Undo to reverse it.",
    )
}

/// `pages.crop`
#[must_use]
pub const fn pages_crop() -> CommandText {
    CommandText::new(
        "Crop…",
        "Hide the edges of the selected pages by a margin from each side. Nothing is \
         deleted and the paper stays the same size; Show whole sheet brings every edge back.",
    )
}

/// `pages.bates`
#[must_use]
pub const fn pages_bates() -> CommandText {
    CommandText::new(
        "Bates numbering…",
        "Stamp a running number, with an optional prefix and suffix, on every page or on the pages picked in the page rail.",
    )
}

/// `pages.bates_remove`
#[must_use]
pub const fn pages_bates_remove() -> CommandText {
    CommandText::new(
        "Remove Bates numbers",
        "Take off the Bates numbers pdfcer stamped, from every page or from the pages picked in the page rail. Numbers stamped by another program are left alone.",
    )
}

/// `pages.labels`
#[must_use]
pub const fn pages_labels() -> CommandText {
    CommandText::new(
        "Number pages…",
        "Choose how pages are numbered in the page box and thumbnails — for example i, ii, iii for front matter, then 1 onwards. Nothing is printed on the pages.",
    )
}

/// `pages.resize`.
#[must_use]
pub const fn pages_resize() -> CommandText {
    CommandText::new(
        "Sheet size…",
        "Put the selected pages on a different size of paper. This changes the paper only — \
         nothing on the page moves and nothing is scaled to fit, so a smaller sheet crops the \
         drawing rather than shrinking it. The window shows what would fall off before you \
         commit.",
    )
}

// ===========================================================================
// EDIT TAB
// ===========================================================================

/// `edit.text`
#[must_use]
pub const fn edit_text() -> CommandText {
    CommandText::new(
        "Edit text",
        "Edit words already on this page — fix a typo, resize, or recolour existing text \
         (Ctrl+E). To add brand-new page text instead, use Add text.",
    )
}

/// `edit.add_text`
#[must_use]
pub const fn edit_add_text() -> CommandText {
    CommandText::new(
        "Add text",
        "Add new text to the page itself — a label, caption or note that becomes real, \
         permanent page content, exactly like the text already here (Ctrl+Shift+E). For a \
         removable comment instead, use Markup > Text box.",
    )
}

/// `edit.insert_image`
#[must_use]
pub const fn edit_insert_image() -> CommandText {
    CommandText::new("Image…", "Place an image file on this page.")
}

/// `edit.insert_3d`
#[must_use]
pub const fn edit_insert_3d() -> CommandText {
    CommandText::new(
        "3D model…",
        "Place a U3D, PRC or STEP model on this page, as Acrobat's 3D tool does. \
         Save one back out from the Attachments panel.",
    )
}

/// `edit.attachments`
#[must_use]
pub const fn edit_attachments() -> CommandText {
    CommandText::new(
        "Attachments",
        "The files this document carries inside itself — attach one, save one out, or remove one. \
         They appear on no page.",
    )
}

/// `edit.align`
#[must_use]
pub const fn edit_align() -> CommandText {
    CommandText::new(
        "Align and Distribute",
        "Line up the selected objects, or space them evenly.",
    )
}

/// `edit.align_left`
#[must_use]
pub const fn edit_align_left() -> CommandText {
    CommandText::new(
        "Align left edges",
        "Line up the selected objects' left edges, relative to the Align panel's choice. Ctrl+Alt+4.",
    )
}

/// `edit.align_right`
#[must_use]
pub const fn edit_align_right() -> CommandText {
    CommandText::new(
        "Align right edges",
        "Line up the selected objects' right edges, relative to the Align panel's choice. Ctrl+Alt+6.",
    )
}

/// `edit.align_top`
#[must_use]
pub const fn edit_align_top() -> CommandText {
    CommandText::new(
        "Align top edges",
        "Line up the selected objects' top edges, relative to the Align panel's choice. Ctrl+Alt+8.",
    )
}

/// `edit.align_bottom`
#[must_use]
pub const fn edit_align_bottom() -> CommandText {
    CommandText::new(
        "Align bottom edges",
        "Line up the selected objects' bottom edges, relative to the Align panel's choice. Ctrl+Alt+2.",
    )
}

/// `edit.align_centre_x`
#[must_use]
pub const fn edit_align_centre_x() -> CommandText {
    CommandText::new(
        "Centre on vertical axis",
        "Line up the selected objects' centres left to right, relative to the Align panel's choice. Ctrl+Alt+7.",
    )
}

/// `edit.align_centre_y`
#[must_use]
pub const fn edit_align_centre_y() -> CommandText {
    CommandText::new(
        "Centre on horizontal axis",
        "Line up the selected objects' centres top to bottom, relative to the Align panel's choice. Ctrl+Alt+1.",
    )
}

/// `edit.align_centre`
#[must_use]
pub const fn edit_align_centre() -> CommandText {
    CommandText::new(
        "Centre on both axes",
        "Put the selected objects' centres on one point, relative to the Align panel's choice. Ctrl+Alt+5.",
    )
}

/// **Text field** — the box an operator types into.
#[must_use]
pub const fn edit_form_text_field() -> CommandText {
    CommandText::new(
        "Text field",
        "A box to type into. Click where you want it, or drag out the exact size.",
    )
}

/// **Check box** — one independent on/off box.
#[must_use]
pub const fn edit_form_check_box() -> CommandText {
    CommandText::new(
        "Check box",
        "A single box that is either ticked or not. Click where you want it, or drag out the exact size.",
    )
}

/// **Radio button** — one of a group.
#[must_use]
pub const fn edit_form_radio_button() -> CommandText {
    CommandText::new(
        "Radio button",
        "One of a set, where choosing one clears the others. Give them the same group name to make them alternatives.",
    )
}

/// **Choice** — a drop-down or list.
#[must_use]
pub const fn edit_form_choice() -> CommandText {
    CommandText::new(
        "Drop-down",
        "A list of options to choose from. Click where you want it, or drag out the exact size.",
    )
}

/// **Select everything on this page, including what has slid off it.**
#[must_use]
pub const fn edit_select_all() -> CommandText {
    CommandText::new(
        "Select all",
        "Selects everything drawn on this page, including anything moved off the edge of the sheet and out of reach of the mouse.",
    )
}

/// **Push button** — authorable, inert, and greyed until pdfcer can run actions.
#[must_use]
pub const fn edit_form_push_button() -> CommandText {
    CommandText::new("Button", "A button that runs an action when pressed.")
}

/// Why the push button is greyed.
#[must_use]
pub const fn edit_form_push_button_unavailable() -> &'static str {
    "pdfcer can place a button but cannot yet run what a button does, so one placed now would do nothing when pressed."
}

pub const fn edit_form_create_field() -> CommandText {
    CommandText::new(
        "Create field",
        "Add a new form field to the page. Click where you want it, or drag out the exact size.",
    )
}

#[must_use]
/// `edit.form_manage_fields`
pub const fn edit_form_manage_fields() -> CommandText {
    CommandText::new(
        "Manage fields",
        "Open the Forms panel to list every field in this document, fill it, rename it or \
         remove it.",
    )
}

/// `edit.form_repair_fonts`
#[must_use]
pub const fn edit_form_repair_fonts() -> CommandText {
    CommandText::new(
        "Repair fonts",
        "Fix a form whose filled values Acrobat shows blank: each font the form keeps inline is moved into an object of its own. Nothing changes how the form looks here.",
    )
}

/// `edit.form_flatten`
#[must_use]
pub const fn edit_form_flatten() -> CommandText {
    CommandText::new(
        "Flatten",
        "Turn the filled values into ordinary page content, so they draw everywhere but can no \
         longer be edited as fields.",
    )
}

/// `edit.find`
#[must_use]
pub const fn edit_find() -> CommandText {
    CommandText::new(
        "Find",
        "Search the text drawn on this document's pages, and highlight every hit (Ctrl+F). Form fields, comments, bookmarks and attachments are not searched, and a word the producer split across two text runs is not found.",
    )
}

/// `edit.redact`
#[must_use]
pub const fn edit_redact() -> CommandText {
    CommandText::new(
        "Redact",
        "Mark what is to be permanently removed — a whole page, every occurrence of some text, \
         or everything matching a pattern. Marking is reversible, and so is applying until the \
         file is written; once it is written, that cannot be undone.",
    )
}

/// `edit.redact_apply`
#[must_use]
pub const fn edit_redact_apply() -> CommandText {
    CommandText::new(
        "Apply redactions",
        "Permanently remove everything the redaction marks cover. By default this happens when \
         you next save, and can be cancelled until then; once the file is written it cannot be \
         undone.",
    )
}

/// `edit.undo`
#[must_use]
pub const fn edit_undo() -> CommandText {
    CommandText::new("Undo", "Undo the last change (Ctrl+Z).")
}

/// `edit.redo`
#[must_use]
pub const fn edit_redo() -> CommandText {
    CommandText::new(
        "Redo",
        "Redo the change you just undid (Ctrl+Y or Ctrl+Shift+Z).",
    )
}

// ===========================================================================
// MARKUP AND MEASURE — in `annotate`
//
// **A module of its own under R2**, at the seam that module's header argues
// for: these two tabs are what an operator *adds on top of* the page,
// which is the line `app::modes::Capabilities` already draws between
// `edit_content` and the two authoring flags, and the line `shell::manifest`
// already draws by keeping `markup.rs` and `measure.rs` as files of their own.
//
// Re-exported by name — not by glob — so every call site still writes
// `t::markup_rectangle()` and nothing outside `text/` learns the catalog was
// split, while a function added over there still has to be named here to reach
// the crate. The catalog's discipline is that every operator-visible string is
// named somewhere a reviewer looks.
// ===========================================================================
pub mod annotate;

pub use annotate::{
    markup_add_node, markup_arrow, markup_attach_file, markup_cloud, markup_comments,
    markup_ellipse, markup_finish, markup_flatten, markup_flatten_page, markup_highlight,
    markup_ink, markup_insert_text, markup_paste_image_stamp, markup_polygon, markup_polyline,
    markup_rectangle, markup_remove_node, markup_replace_text, markup_screen, markup_sound,
    markup_squiggly, markup_stamp, markup_sticky_note, markup_strikeout, markup_text_box,
    markup_underline, measure_area, measure_finish, measure_length, measure_linear,
    measure_manage_groups, measure_perimeter, measure_radius_diameter, measure_set_scale,
    measure_two_line,
};

/// The five Format ▸ Markup controls, a module of their own under **R2**.
/// The seam is [`annotate`]'s, one step along the same line — that module
/// holds the strings of the commands that
/// **place** a mark, and this one the strings of the commands that **restyle
/// one already placed**. Re-exported by name, not by glob, so a function added
/// over there still has to be named here to reach the crate.
pub mod markupstyle;

pub use markupstyle::{
    format_arrowheads, format_colour, format_fill, format_line_style, format_line_width,
    format_opacity,
};

/// The four Markup ▸ Arrange controls, a module of their own under **R2**
/// for [`markupstyle`]'s reason and along the same seam one step further:
/// [`annotate`] holds the strings of the commands that **place** a mark,
/// `markupstyle` those that **restyle** one already placed, and this one those
/// that **re-depth** one already placed. Its header carries why all four labels
/// are borrowed verbatim from five reference applications and why not one of
/// them may say *z-order*.
pub mod arrange;

pub use arrange::{
    markup_bring_forward, markup_bring_to_front, markup_send_backward, markup_send_to_back,
};

// ===========================================================================
// TOOLS TAB
// ===========================================================================

/// `tools.merge_files`
#[must_use]
pub const fn tools_merge_files() -> CommandText {
    CommandText::new(
        "Merge files…",
        "Combine several PDFs into one new file. This document is not changed — to add pages \
         to it instead, use Pages > Merge into this document.",
    )
}

/// `tools.split_files`
#[must_use]
pub const fn tools_split_files() -> CommandText {
    CommandText::new(
        "Split files…",
        "Split one or more PDFs into separate files. The originals are not changed.",
    )
}

/// `tools.font_folders`
#[must_use]
pub const fn tools_font_folders() -> CommandText {
    CommandText::new(
        "Font folders…",
        "Point pdfcer at folders of your own font files (.ttf/.otf) so it can draw a document's \
         missing text with the real typeface instead of a bundled substitute. This changes how \
         missing fonts look, not where text sits on the page.",
    )
}

/// `tools.embed_fonts`
#[must_use]
pub const fn tools_embed_fonts() -> CommandText {
    CommandText::new(
        "Embed fonts",
        "Copy the font programs this document relies on into the file itself, so it draws the \
         same on a machine that does not have them.",
    )
}

/// `tools.unembed_fonts`
#[must_use]
pub const fn tools_unembed_fonts() -> CommandText {
    CommandText::new(
        "Unembed fonts",
        "Remove embedded font programs from the file. The document gets smaller and starts \
         depending on the reader having those fonts.",
    )
}

/// `tools.render_diagnostics`
#[must_use]
pub const fn tools_render_diagnostics() -> CommandText {
    CommandText::new(
        "Render diagnostics",
        "Show what the renderer did with the last page — how long it took, at what raster \
         size, and anything it could not draw.",
    )
}

/// `tools.ink_picker`
#[must_use]
pub const fn tools_ink_picker() -> CommandText {
    CommandText::new(
        "Ink picker",
        "Click a point on the page to read the cyan, magenta, yellow, black and spot ink a \
         press would put there.",
    )
}

// ===========================================================================
// MODES
//
// Not ribbon commands: these are the three positions of the selector at the
// far right of the tab row, reachable from the keymap. They are registered
// commands because a key binding resolves against the registry, and because
// the mode selector is a control like any other.
//
// Each tooltip states the rule that makes the feature safe — a mode changes
// what is VISIBLE and never makes a visible control silently inert. That
// distinction is the whole difference between this and the `editing_enabled`
// master toggle it replaces.
// ===========================================================================

/// `mode.read`
#[must_use]
pub const fn mode_read() -> CommandText {
    CommandText::new(
        "Read",
        "Show only what a reader needs: File and View (Ctrl+1). Nothing is hidden from the \
         document — only from the interface — and your edits are untouched.",
    )
}

/// `mode.review`
#[must_use]
pub const fn mode_review() -> CommandText {
    CommandText::new(
        "Review",
        "Add the Pages, Markup and Measure tabs (Ctrl+2) — comment on a drawing, measure it, \
         and reorganise the sheets, without the content-editing tools.",
    )
}

/// `mode.edit`
#[must_use]
pub const fn mode_edit() -> CommandText {
    CommandText::new("Edit", "Show every tab (Ctrl+3).")
}

/// The properties every command's copy must hold — one label per command,
/// no two labels alike, every tooltip a sentence. Split out under R2; see
/// that module's header.
#[cfg(test)]
mod tests;
