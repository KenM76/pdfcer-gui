# `ui-verify/checks/model_close_asks`

## `closing_the_document_asks_about_its_model_window`

Off-screen, scripted pointer and keys only. Places the engine corpus's
`assembly.prc` from the ribbon (`model_view_window::launch_with_model`,
`press_insert`) and opens it with *View…*.

Oracles:

1. `model-view-owner owned=0` within 40 tries ten frames apart: the
   operating system reports no owner window for the viewer.
2. Ctrl+W on the document traces `model-close-asked reason=tab`, and no
   `model-view-closed` or `close slot=` line after it.
3. Cancel traces `model-close-answered reason=tab choice=cancel`, and
   still neither line.
4. Ctrl+W again and *Close both*: `model-view-closed how=document`, then
   `unsaved-asked` (the inserted model is an unsaved edit); *Discard* there,
   then a `close slot=` line.

Buttons are pressed only after they are declared after the step's mark: the
Insert 3D model window publishes the same `dialog.buttons.*` names earlier.

Not asserted: the opening hold's keyboard behaviour, and the quit route
(`reason=quit`), which a driven close of the root window would end the
session for.
