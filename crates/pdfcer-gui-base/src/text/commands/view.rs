//! # `text::commands::view` — every label and tooltip on the View tab
//!
//! Split out of [`super`] on 2026-08-20, when that file crossed rule R2's
//! 1,500-line ceiling. The View tab is the largest single section of the
//! catalogue by a wide margin — page display, zoom, the overlays, the nine
//! panel toggles and the window verbs — and it is also the most
//! self-contained: nothing here is read by any other tab's entry, and every
//! entry answers one question, which `RIBBON_IA.md` §3 gives as *"what am I
//! looking at, and how?"*
//!
//! ## It is re-exported, so nothing changed for a caller
//!
//! `super` carries `pub use view::*;`. Every call site still writes
//! `crate::text::commands::view_zoom_in()`, the catalogue's coverage test
//! still walks one list, and the split is a fact about where the source lives
//! rather than about the shape of the module. That is deliberate: a seam that
//! forces a rename at ninety call sites is a seam that will be argued about
//! instead of taken.
//!
//! ## The rules that apply here are `super`'s
//!
//! Sentence case, no trailing period on a label, an ellipsis when activating
//! opens a dialog, a full sentence with punctuation on a tooltip, and — the
//! one that has cost this project real defects — **never state a capability
//! the build does not have**. See that module's header, which is where those
//! are argued.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/commands/view.md`.

use super::CommandText;

// ===========================================================================
// VIEW TAB
// ===========================================================================

/// `view.page_single`
#[must_use]
pub const fn view_page_single() -> CommandText {
    CommandText::new(
        "Single page",
        "Show one page at a time. This is pdfcer's default, because paging one drawing sheet at \
         a time is the right model for reading a sheet set.",
    )
}

/// `view.page_continuous`
#[must_use]
pub const fn view_page_continuous() -> CommandText {
    CommandText::new(
        "Continuous",
        "Scroll through every page in one run, for a document you read rather than a sheet set \
         you page through. pdfcer remembers this choice for this document, so another file keeps \
         its own.",
    )
}

/// `view.page_facing`
#[must_use]
pub const fn view_page_facing() -> CommandText {
    CommandText::new(
        "Facing",
        "Show two pages side by side, as an open book. The first page sits alone, so every \
         later spread pairs the way a bound document does.",
    )
}

/// `view.page_facing_continuous`
#[must_use]
pub const fn view_page_facing_continuous() -> CommandText {
    CommandText::new(
        "Facing continuous",
        "Scroll through every spread in one run — facing pages, without stopping at each one.",
    )
}

/// `view.panel_pages`
#[must_use]
pub const fn view_panel_pages() -> CommandText {
    CommandText::new(
        "Pages",
        "Show or hide the panel of page thumbnails: click one to go there, and pick several to \
         act on them together.",
    )
}

/// `view.zoom_actual`
#[must_use]
pub const fn view_zoom_actual() -> CommandText {
    CommandText::new(
        "Actual size",
        "Show the page at actual size — one PDF point per screen point (Ctrl+0).",
    )
}

/// `view.zoom_selection`
#[must_use]
pub const fn view_zoom_selection() -> CommandText {
    CommandText::new(
        "Zoom to selection",
        "Scale and centre the view on what is selected.",
    )
}

/// `view.zoom_region`
#[must_use]
pub const fn view_zoom_region() -> CommandText {
    CommandText::new(
        "Zoom to region",
        "Drag a rectangle on the page to zoom to it. The selection is left alone.",
    )
}

/// `view.tool_hand`
#[must_use]
pub const fn edit_cut() -> CommandText {
    CommandText::new(
        "Cut",
        "Copy what is selected and remove it \u{2014} a comment, a shape on the page, or a \
         form field. One Ctrl+Z brings it back.",
    )
}

/// `edit.copy` — the object clipboard's copy.
#[must_use]
pub const fn edit_copy() -> CommandText {
    CommandText::new(
        "Copy",
        "Copy what is selected \u{2014} a comment, a shape on the page, or a form field. \
         Ctrl+C over selected TEXT copies the text instead.",
    )
}

/// `edit.paste` — the object clipboard's paste.
#[must_use]
pub const fn edit_paste() -> CommandText {
    CommandText::new(
        "Paste",
        "Put what you copied on this page. A copied form field arrives as a NEW field \
         with its own value. On the page it came from it lands slightly offset so you \
         can see it; on any other page it lands where it was.",
    )
}

/// `edit.paste_duplicate` — the second sense of a form-field paste.
#[must_use]
pub const fn edit_paste_duplicate() -> CommandText {
    CommandText::new(
        "Paste as duplicate",
        "Paste a copied form field as ANOTHER BOX FOR THE SAME FIELD \u{2014} typing in one \
         fills both, and it keeps the original's font, colour and any calculation. For \
         anything else on the clipboard this is an ordinary paste.",
    )
}

/// `edit.duplicate` — a second copy of the selected comment, **without using
/// the clipboard**.
#[must_use]
pub const fn edit_duplicate() -> CommandText {
    CommandText::new(
        "Duplicate",
        "Put a second copy of the selected comment on the page, slightly offset so you \
         can see it \u{2014} WITHOUT using the clipboard, so whatever you had copied is \
         still there. Press it again for another.",
    )
}

/// `edit.copy_as_vector` — the clipboard's copy-OUT.
#[must_use]
pub const fn edit_copy_as_vector() -> CommandText {
    CommandText::new(
        "Copy as vector",
        "Copy the selection \u{2014} or this whole page, if nothing is selected \u{2014} as \
         EDITABLE GEOMETRY rather than as a picture of it. Paste into Word, PowerPoint or \
         Inkscape and the line-work can still be scaled, recoloured and taken apart.",
    )
}

/// `pages.copy` — copy the picked sheets.
#[must_use]
pub const fn pages_copy() -> CommandText {
    CommandText::new(
        "Copy pages",
        "Copy the sheets picked in the Pages panel, or the current sheet if none are picked. \
         What pdfcer holds is a complete PDF, so you can paste it here or into another drawing.",
    )
}

/// `pages.cut` — copy the picked sheets and remove them.
#[must_use]
pub const fn pages_cut() -> CommandText {
    CommandText::new(
        "Cut pages",
        "Copy the picked sheets and remove them from this drawing. One Ctrl+Z brings them back.",
    )
}

/// `pages.paste` — put copied sheets in after the current one.
///
/// It says WHERE, because a page paste changes the document dramatically and
/// an operator who cannot predict where the sheets land will not use it twice.
#[must_use]
pub const fn pages_paste() -> CommandText {
    CommandText::new(
        "Paste pages",
        "Put the copied sheets in after the one you are looking at. Copied form fields may \
         arrive as boxes nothing can fill \u{2014} pdfcer says so if they do.",
    )
}

/// `edit.redact_selection` — mark what is selected, without searching for it.
#[must_use]
pub const fn edit_redact_selection() -> CommandText {
    CommandText::new(
        "Redact selection",
        "Mark whatever you have selected \u{2014} a shape, an image, a piece of text \u{2014} to be \
         removed. Use this for anything the search box cannot find, like a drawn title block or \
         a scanned stamp. Nothing is removed until you apply the redactions.",
    )
}

/// `edit.offpage` — find everything drawn outside the sheet.
#[must_use]
pub const fn edit_offpage() -> CommandText {
    CommandText::new(
        "Check for content off the sheet",
        "Look for anything drawn outside the page boundary on any sheet. It does not print and \
         does not show on screen, but it is still in the file and still travels when you send it \
         \u{2014} an old revision note dragged off the edge, a cropped title block, a name moved \
         out of the frame. pdfcer lists what it finds and can mark it for removal.",
    )
}

/// See the module header.
#[must_use]
pub const fn view_tool_select() -> CommandText {
    CommandText::new(
        "Select",
        "Click a shape to select it, drag to move it, drag on empty paper to select several. \
         The tool everything returns to.",
    )
}

/// The **Node tool** — the white arrow.
#[must_use]
pub const fn view_tool_node() -> CommandText {
    CommandText::new(
        "Points",
        "Click a shape to show its points, then click one and drag to move it. Shift-click to \
         take several. A point on a curve also shows its handles.",
    )
}

/// See the module header.
#[must_use]
pub const fn view_tool_hand() -> CommandText {
    CommandText::new(
        "Hand",
        "Drag to pan the page instead of selecting. Hold Space to pan without switching tools.",
    )
}

/// `view.tool_text`
#[must_use]
pub const fn view_tool_text() -> CommandText {
    CommandText::new(
        "Text",
        "Click text to edit it, or click empty space to start new text. Drag to select text \
         for copying. Press again to return to the select tool.",
    )
}

/// `view.zoom_fit_page`
#[must_use]
pub const fn view_zoom_fit_page() -> CommandText {
    CommandText::new(
        "Fit page",
        "Scale the page so all of it is visible, and keep it fitted as the window resizes.",
    )
}

/// `view.zoom_fit_width`
#[must_use]
pub const fn view_zoom_fit_width() -> CommandText {
    CommandText::new(
        "Fit width",
        "Scale the page so its full width is visible, and keep it fitted as the window resizes.",
    )
}

/// `view.zoom_fit_height`
#[must_use]
pub const fn view_zoom_fit_height() -> CommandText {
    CommandText::new(
        "Fit height",
        "Scale the page so its full height is visible, and keep it fitted as the window resizes.",
    )
}

/// `view.show_annotations`
#[must_use]
pub const fn view_show_annotations() -> CommandText {
    CommandText::new(
        "Annotations",
        "Show or hide the markup, stamps and form-field appearances stored in this document, \
         so the page content can be seen alone.",
    )
}

/// `view.show_points`
#[must_use]
pub const fn view_show_points() -> CommandText {
    CommandText::new(
        "Points",
        "Show the editable points of every part of the object you are working inside, not just \
         the part you have selected. Points always appear for the selected part.",
    )
}

/// `view.smart_select`
#[must_use]
pub const fn view_smart_select() -> CommandText {
    CommandText::new(
        "Smart select",
        "Click selects a whole thing — a title block, a stamped drawing, a symbol — instead of one line inside it. Double-click goes inside, as many times as it takes to reach the part you want. Escape steps back out.",
    )
}

/// `view.text_chunks`
#[must_use]
pub const fn view_text_chunks() -> CommandText {
    CommandText::new(
        "Text chunks",
        "Draw a thin box round each chunk of text inside a selected block, so you can see which \
         piece a click will pick up before you click it. It changes nothing about the page.",
    )
}

/// `view.ocr_layer`
#[must_use]
pub const fn view_ocr_layer() -> CommandText {
    CommandText::new(
        "OCR text",
        "Draw the recognised text a scan is carrying. It is already in the file — searchable and \
         copyable, just never drawn — and this shows you where it landed and what it says. Slide \
         between the scan and the text to compare them. It changes nothing about the page.",
    )
}

/// `view.rulers`
#[must_use]
pub const fn view_rulers() -> CommandText {
    CommandText::new(
        "Rulers",
        "Show rulers along the top and left of the page, reading in points — or in this \
         document's own units if a measurement scale has been set for it. Drag out of a ruler \
         to place a guide.",
    )
}

/// `view.grid`
#[must_use]
pub const fn view_grid() -> CommandText {
    CommandText::new(
        "Grid",
        "Draw a grid over each page, spaced to match the rulers so its heavier lines fall on \
         the numbered marks. The grid belongs to the sheet and scrolls with it.",
    )
}

/// `view.guides`
#[must_use]
pub const fn view_guides() -> CommandText {
    CommandText::new(
        "Guides",
        "Show the guide lines placed on this document's pages, and let them be moved. Drag one \
         out of a ruler to place it, drag it off the page to remove it, or double-click it. \
         pdfcer remembers a document's guides for the next time you open it.",
    )
}

/// `view.line_weights`
#[must_use]
pub const fn view_line_weights() -> CommandText {
    CommandText::new(
        "Line weights",
        "Turn this off to draw every line one pixel wide, however wide the file says they are \
         — the CAD way of reading a dense drawing, so lines that sit close together stop \
         merging into one black bar when you zoom in. Filled shapes and hatching are not \
         affected. Printing, exporting and the print preview always use the real widths.",
    )
}

/// `view.off_page`
#[must_use]
pub const fn view_off_page() -> CommandText {
    CommandText::new(
        "Off-page content",
        "Some drawings carry marks outside the sheet itself — a title block dragged off the \
         page, a detail parked in the margin. Turn this on to draw them and let you click \
         them; pdfcer widens the canvas around the page to make room. Turn it off and the \
         page is shown on its own, with no extra space around it. Read starts with this \
         off; Review and Edit start with it on, and pdfcer remembers your answer for each.",
    )
}

/// `view.sidebar`
#[must_use]
pub const fn view_sidebar() -> CommandText {
    CommandText::new(
        "Sidebar",
        "Show or hide the left panel — page thumbnails and the active tool's options.",
    )
}

/// `view.panel_bookmarks`
#[must_use]
pub const fn view_panel_bookmarks() -> CommandText {
    CommandText::new(
        "Bookmarks",
        "Show the document's bookmarks. Click one to jump to its page.",
    )
}

/// `view.panel_layers`
#[must_use]
pub const fn view_panel_layers() -> CommandText {
    CommandText::new(
        "Layers",
        "Show the document's layers, and switch any of them on or off while you look at it. \
         The document is not changed.",
    )
}

/// `view.panel_signatures`
#[must_use]
pub const fn view_panel_signatures() -> CommandText {
    CommandText::new(
        "Signatures",
        "Show each digital signature: what it covers, whether the bytes it covers were \
         altered, and whether the signer is one you have chosen to trust.",
    )
}

/// `view.panel_objects`
#[must_use]
pub const fn view_panel_objects() -> CommandText {
    CommandText::new(
        "Objects",
        "Show or hide the right-hand panel listing everything on the page, nested into parts \
         and points.",
    )
}

/// `view.panel_forms`
#[must_use]
pub const fn view_panel_forms() -> CommandText {
    CommandText::new(
        "Fill form",
        "List this document's fillable fields and type into them. Nothing is written to disk \
         until you save.",
    )
}

/// `view.read_mode`
#[must_use]
pub const fn view_read_mode() -> CommandText {
    CommandText::new(
        "Read mode",
        "Hide the ribbon and the panels and give the whole window to the page (Ctrl+H).",
    )
}

/// `view.fullscreen`
#[must_use]
pub const fn view_fullscreen() -> CommandText {
    CommandText::new("Full screen", "Fill the whole display with pdfcer (F11).")
}

/// `view.next_document`
#[must_use]
pub const fn view_next_document() -> CommandText {
    CommandText::new(
        "Next document",
        "Show the next open document (Ctrl+Tab). Wraps round at the end.",
    )
}

/// `view.previous_document`
#[must_use]
pub const fn view_previous_document() -> CommandText {
    CommandText::new(
        "Previous document",
        "Show the previous open document (Ctrl+Shift+Tab). Wraps round at the start.",
    )
}

/// `view.close_other_documents`
#[must_use]
pub const fn view_close_other_documents() -> CommandText {
    CommandText::new(
        "Close others",
        "Close every open document except the one you opened this on. Any with unsaved \
         edits are asked about one at a time (Ctrl+W closes just the one you are \
         looking at).",
    )
}

/// `view.reset_layout`
#[must_use]
pub const fn view_panel_float() -> CommandText {
    CommandText::new(
        // "Float panel" and not "Float". The label has to be unique across
        // the whole registry (`no_two_commands_share_a_label`), and the noun
        // earns its place beyond that test: this row sits in a menu beside
        // "Reset layout", which acts on the whole dock, so saying which
        // subject each row has is what stops the two reading as a pair of
        // options on one thing.
        "Float panel",
        "Move this panel into a window of its own, which you can put anywhere - including on \
         another monitor. Dock puts it back where it came from.",
    )
}

/// **Dock this panel** — put a floating panel back where it came from.
#[must_use]
pub const fn view_panel_dock() -> CommandText {
    CommandText::new(
        "Dock panel",
        "Put this panel back in the dock, in the same place it was when you floated it.",
    )
}

/// **Close this panel** — take it off screen entirely.
#[must_use]
pub const fn view_panel_close() -> CommandText {
    CommandText::new(
        // "Close panel", not "Close" — and here the noun is load-bearing
        // rather than merely tidy. `file.close` is already labelled "Close"
        // and closes the DOCUMENT. Two rows reading "Close", one of which
        // discards a panel and the other of which can discard unsaved work,
        // is a collision the operator pays for and not one the registry does.
        "Close panel",
        "Take this panel off screen. You can bring it back from the View tab.",
    )
}

/// **Dock all floating panels** — the way back to a window you cannot
/// reach.
#[must_use]
pub const fn view_dock_all_panels() -> CommandText {
    CommandText::new(
        "Dock all panels",
        "Bring every floating panel back into the dock. Use this if a panel window has ended up \
         on a monitor you no longer have.",
    )
}

/// **Auto-hide the ribbon** — his instruction of 2026-09-05.
#[must_use]
pub const fn view_ribbon_auto_hide() -> CommandText {
    CommandText::new(
        "Auto-hide ribbon",
        "Keep the row of tab names and hide the buttons under it until you move the pointer \
         onto that row. The buttons then appear OVER the drawing, so nothing you were about to \
         click moves. Press this again to keep them showing.",
    )
}

/// **Auto-hide the left strip** — the same instruction, the other surface.
#[must_use]
pub const fn view_rail_auto_hide() -> CommandText {
    CommandText::new(
        "Auto-hide left strip",
        "Shrink the strip of panel and tool buttons down the left edge to a narrow marked band, \
         and bring it back when you move the pointer onto that band. It appears OVER the panel \
         beside it, so the panel does not change width. Press this again to keep it showing.",
    )
}

#[must_use]
pub const fn view_reset_layout() -> CommandText {
    CommandText::new(
        "Reset layout",
        "Put both panel docks back where this mode started them. Your other modes keep the \
         arrangements you gave them.",
    )
}
