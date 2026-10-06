# `ui-verify/checks/ocr_layer_group`

`recognised_text_is_a_layers_row` — recognised text is written on a layer
named *Recognised text*, the Layers panel lists it, and Remove OCR text
deletes the layer it leaves empty, in one undo step with the words.

# What it drives

A scratch copy of `fixtures/synthetic-image-only.pdf` (ignores `--pdf`), off
the desktop under `--no-input` with a `ScriptedPointer`:

1. Through `ocr_scripted::recognise`: File ▸ Recognise text… with `ocrs`,
   then Ctrl+S. `ocr-layer-group` must say `made=true`, and
   `save-in-place outcome=ok`.
2. A second launch on the saved file, Edit mode, View ▸ Layers if the row is
   not drawn: `panel.layers.row.Recognised_text` must be declared.
3. File ▸ Remove OCR text: `remove-ocr-layers-applied` must say `removed=1`
   and `groups-deleted=1`, and the row must be gone.
4. Ctrl+Z once: the row must be back.

# Not driven

Reusing the layer on a later run. A page with text is refused by Recognise
text (`AlreadyHasText`), so a one-page fixture reaches the made branch only;
the reuse is a name lookup in `app::actions::ocrlayers::find_group`.

# Falsified

Two plants in `app::actions::ocrlayers`, each red:

- The write without `.on_layer(group)`: the words land on no layer, the
  removal empties none, and step 3 fails on `groups-deleted=0`.
- The removal folded as `coalesce_last(removed, …)`: the layer's deletion is
  its own undo entry, and step 4 fails on `undo_depth=2`. The kind alone does
  not catch this — a one-entry fold relabels the top entry — so step 4 also
  requires `undo_depth=1`.
