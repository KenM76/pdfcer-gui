# `ui-verify/checks/attach_file`

`a_file_attaches_as_a_marker` — Markup ▸ Attach file puts a `/FileAttachment`
marker on the page that carries the chosen file.

# What it drives

Its own fixture, `fixtures/layer-assign.pdf` (ignores `--pdf`; an 800 × 600
page), and a payload it writes to its output folder, named to the picker
through `PDFCER_DIAG_ATTACH_PATH`.

1. Review mode, `ribbon.tab.markup`, `ribbon.item.markup.attach_file`:
   `markup-tool tool=TextAnnot(Attachment)`.
2. A click at (150, 450), clear of the fixture's box and annotation:
   `attach-picked source=env`, then `attach-annot-open … bytes=<payload size>`.
3. `text-annot.attach-icon.Paperclip`, then `text-annot.accept`:
   `attach-annot-read … bytes=<payload size> icon=Paperclip`, then
   `attach-annot-placed … id=<non-zero>`.

The payload's byte count is the oracle that the file itself, not its name,
reached the engine; the icon is the oracle that the window's choice survives
to the action.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.
