# `each_tab_remembers_its_mode`

**Defect it catches:** with Settings > Display > *Remember the mode of each
tab* on, a tab brought back shows the mode of the tab just left instead of
its own; or, with it off, switching tabs changes the mode (O463).

## What it drives

Two launches off-screen (scripted pointer, `--no-input` safe), one with
`remember_tab_mode = false` planted in the sandbox's preferences and one with
`true`. Each opens `fixtures/four-pages.pdf`, with `PDFCER_DIAG_OPEN_PATH`
answering the Open dialog with `fixtures/drawing-sheets.pdf`.

1. Click `ribbon.mode.edit`; the dock must follow (`mode-changed to=edit`).
2. Ctrl+O opens the second document. Click `ribbon.mode.review`.
3. Click `doc-tab.0`; a `document-activate slot=0` line must follow.

## Oracle

The last `mode-changed to=` after the switch: `review` with the setting off
(the window's mode is kept), `edit` with it on (the first tab's own mode).
Each launch is the other's control. Neither mode is the start mode, so a
restore cannot be confused with a default.

SKIP when a mode segment click did not move the mode, or the switch was not
traced. The preferences file is reset afterwards.

## Not covered

Closing a tab, tearing one off into another window, and a document opened
while the setting is on (it starts in the current mode).
