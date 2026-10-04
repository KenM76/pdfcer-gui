# `app::actions::buttonicon` — choosing a push button's picture

`pick(doc, field, widget)` is the apply half of `FieldAction::PickButtonIcon`.
The picker runs here rather than in the panel because a modal file dialog
inside a frame's drawing would block the paint that raised it.

1. `files::pick_image_source()`. Cancel traces `button-icon-cancelled` and
   changes nothing. The test seam is `PDFCER_DIAG_IMAGE_PATH`.
2. Read the file and `Picture::import` it.
3. A raster becomes `WidgetEdit::with_button_icon(image)` through
   `forms::edit_widget`, so it is one undo step with the engine's appearance
   disclosure on the status line.
4. An SVG or EMF, or a file that does not import, traces
   `button-icon-declined field= widget= reason=<svg|emf|unreadable>` and
   records the reason on the status line. The engine's icon verb takes an
   `ImportedImage` only.

**Every icon edit redraws foreign artwork.** The engine's default,
`ForeignAppearance::ReplaceOnIconEdit`, replaces another program's artwork
when an edit touches the icon; kept, it would hide the picture while the panel
says it has one. The status line says when foreign artwork was replaced.
`panels/properties/buttonicon.md`'s `push_edit` relies on the same default
for Remove and the caption position.

Without a caption position the engine picks one: picture only for a button
with no caption, caption below otherwise.
