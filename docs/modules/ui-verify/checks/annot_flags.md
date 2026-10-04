# `ui-verify/checks/annot_flags`

`an_annotation_can_be_hidden_and_shown_again` — the Properties panel's Print
and Show on screen switches change a selected annotation's `/F`; a mark taken
off screen is no longer selectable on the canvas; the Comments list's *Show on
screen* brings it back.

# What it drives

A copy of `fixtures/layer-assign.pdf` (ignores `--pdf`), with
`PDFCER_DIAG_INVOKE=mode.edit,markup.comments,file.properties`. The fixture's
`/Square` (`/Rect [480 350 720 520]`) carries no `/F`.

1. A scripted click at (600, 435) on page 1 → `annot-select`.
2. `properties.annot_flags.prints` → `set-annotation-flag-applied
   switch=prints on=true`, with Print (4) set in `after`.
3. `properties.annot_flags.on_screen` → `switch=on-screen on=false`, with
   NoView (32) set.
4. The same click again → no `annot-select` after it.
5. `comments.show_again` (raising `dock.tab.markup.comments` first if the
   Comments panel is not the active tab) → `switch=on-screen on=true`, with
   NoView clear.

`before=` and `after=` are the whole `/F` word as the engine held it either
side of `set_annotation_flags`, so each step proves the file changed and that
the switch moved its own bit.

# Falsified

Drawing *Show on screen* only on rows that are NOT suppressed fails step 5:
the hidden mark has no way back.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.
