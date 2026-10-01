# The program: modes, ribbon, where features live

pdfcer-gui is a Windows desktop PDF viewer and editor, built for CAD drawings
but general. It has a multi-document window (one tab per PDF), a ribbon, a
left rail, dockable panels and a status bar. The engine (the same one as
`pdfcer.exe`) is built in.

## Shipped beside the exe

- `pdfcer-gui.exe`: the program. Takes one optional argument, a PDF path.
- `pdfcer-remote.exe`: live-link client (see live-link.md).
- `MANUAL-GUI.md`: the user manual. `FEATURES-GUI.md`: every capability and what is established about it.
- `README-GUI.md`, `LICENSE-GUI`, `THIRD_PARTY_LICENSES-GUI.md`, `BUILD-INFO-GUI.txt`.
- `models\`: OCR model weights. `userdata\`: the user's settings (userdata.md).
- `LLM-GUI\`: this folder.
- Maybe `pdfcer.exe` with its own README.md, LICENSE and BUILD-INFO.txt (cli.md).

## Modes: what the program lets the user do

Selector at the top right; Ctrl+1/2/3; command ids `mode.read|review|edit`.

| Mode | Ribbon tabs |
|---|---|
| Read | File, View |
| Review | File, View, Pages, Markup, Measure |
| Edit | File, View, Pages, Edit, Markup, Measure, Tools |

- A command whose tab the mode lacks is **not drawn** (remote: `hidden`).
  Greyed means temporarily unavailable (no selection, no document).
- Ids on no main tab (Format tab, rail, keymap-only, context menus) are offered
  in every mode. Clipboard ids (`edit.copy/cut/paste/paste_duplicate/duplicate`)
  are offered everywhere and decide for themselves.
- Read mode also stops canvas edits (dragging, deleting). Filling a form
  works in Read mode.
- Read **mode** is not read **view**. Ctrl+H (`view.read_mode`) hides the
  ribbon and panels; Ctrl+H again restores them. F11 is full screen.

## Ribbon tabs: the question each answers, and its groups

- **File** (whole file, or the program itself): File, Recognise, Save, Export, Security, Print, Document, pdfcer. Holds Settings, Keyboard shortcuts and About.
- **View** (what is on screen): Page display, Navigate, Zoom, Display (rulers, grid, guides, line weights, annotations, OCR text, off-page content), Panels, Window.
- **Pages** (the set of pages): Insert, Clipboard, Organise, Transform, Stamp (Bates).
- **Edit** (changing existing content): Content (edit text, add text, select all, redact), Insert (image, 3D), Arrange (align), Clipboard, Forms, Protect.
- **Markup** (adding for someone else): Shapes, Text markup, Notes, Style, Arrange (stacking), Comments.
- **Measure** (ce dimensions and scale): Dimension, Scale.
- **Tools** (across files, or configured once): Batch (merge files), Fonts, Diagnostics.
- **Format**: a contextual tab, shown while the selection is formattable. Font, Markup style, Selection (properties, select the form, unshare form, merge text runs, delete).

Quick-access toolbar: Open, Save, Undo, Redo. Trailing: Acrobat (only when
Acrobat is installed). A narrow window folds groups into a `⌄` button or
scrolls the tabs (`›`).

## Left rail and panels

Rail: the panel buttons (Pages, Bookmarks, Layers, Signatures, Comments,
Fonts), the canvas tools (Select V, Points A, Text T, Hand H), Smart select,
Text chunks, Select all, and Rotate left/right.
Panels (View ▸ Panels): Pages, Bookmarks, Layers, Signatures, Objects, Fill
form, Comments, Fonts. Panels dock, float and are saved per mode.

## Canvas behaviour worth knowing

- Click selects; Shift-click adds; drag a box. A right-to-left box takes
  everything it touches.
- Smart select: a click takes a whole group (title block, symbol). Double-click
  goes inside it; Escape steps back out.
- The grey area around the sheet is real space. Off-page objects are drawn
  there and can be selected. Ctrl+A includes them.
- Plain wheel turns pages, Ctrl+wheel zooms, the middle button pans.
- Select filter (bottom left): which object classes a click may land on.
- The canvas shows applied content exactly as it will save. Warnings and
  disclosures appear in the status bar or panels, never drawn on the page.

## ce dimension and pdf dimension: always say which

- **ce dimension**: a measurement annotation pdfcer authors (Measure tab,
  `dimension-*`/`group-*` CLI). Editable, grouped, scaled.
- **pdf dimension**: a dimension the CAD exporter drew as ordinary page
  vectors and text. Not to be altered unless the user asks.
