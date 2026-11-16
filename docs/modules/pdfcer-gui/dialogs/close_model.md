# `pdfcer-gui/dialogs/close_model`

The question asked before a document with an open 3D viewer closes, or the
program closes with a viewer open. Strings: `text::model3dclose`.

| Closing | Asks | Choices |
|---|---|---|
| the document's tab | *The 3D model window for {name} is open. Closing the document closes it too.* | **Close both**, **Cancel** |
| the program | *A 3D model window is open. Closing pdfcer closes it too.* | **Close**, **Cancel** |

Both add *The 3D window reads its saved views from the document and writes
views, measurements and pictures into it, so it cannot stay open without
it.* That is why no choice keeps the viewer open without its document.
Escape and the window's close button count as Cancel; Enter presses the
close button.

The question's state lives in `ModelView` (`asking`, `answered`), so it
closes with the viewer. If the viewer is closed while the question is up,
the pending close is dropped and the operator closes again.

`Closing::Tab` carries the document's path, not a slot: slots renumber when
a tab closes. The answer is acted on by `app::quitting::step_model_close`.

Region: `dialog:close_model` (body), and the host's `dialog.buttons.accept`
and `dialog.buttons.cancel`.
