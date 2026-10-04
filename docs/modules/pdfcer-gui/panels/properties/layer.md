# `panels::properties::layer` — the Layer row and the Move to layer window

**Layer row.** Drawn in a mode that edits content, in a document with at
least one layer, for a selection `layerassign::operand` accepts. A combo shows
what the selection is on — a layer's name, *No layer*, *Several layers*
(membership `Mixed`) or *A visibility rule* (an `/OC` that is not a registered
layer) — and offers *No layer* then every layer. Choosing a different entry
raises `LayerAction::Assign`. A selection of parts, or across pages, shows the
current label on a greyed button whose hover says why.

What the selection is on comes from `panels::layers::highlight`: `resolve`
for page content, `on_annotation` for an annotation or widget.

**Move to layer window.** `format.move_to_layer` (ribbon Format ▸ Selection,
and the object, field, markup and dimension canvas menus) calls
`open_window`; `window`, drawn each frame from `app::frame`, holds the choice
in egui temp memory under `layer-assign-window` until Move or Cancel.

## Regions

`properties.layer.combo`, `properties.layer.option.<key>`,
`layer-assign.window.combo`, `layer-assign.window.option.<key>`,
`layer-assign.window.go`. `<key>` is the shown name with every
non-alphanumeric character as `_` (`No_layer`).
