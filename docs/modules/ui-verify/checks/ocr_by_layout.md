# `reading_by_layout_layers_each_region_apart`

**With PaddleOCR-VL and *Read by layout* ticked, words read from titles,
tables and other regions land on a layer per region, each a sublayer of
*Recognised text*, in the same undo step; Remove OCR text deletes every layer
it empties. The window draws neither word-list choice for this model.**

# Sequence

A scratch copy of `fixtures/synthetic-image-only.pdf` (ignores `--pdf`), off
the desktop under `--no-input` with a `ScriptedPointer`. Needs the dev build's
`models/paddle-vl/layout.onnx`; without it the checkbox is not drawn and step 1
fails on the click.

1. `ocr_scripted::recognise_choosing` with `paddle-vl`, clicking
   `ocr-by-layout` before Run, then Ctrl+S.
2. Neither `ocr-no-word-lists` nor `ocr-add-words` is declared (★), and
   `ocr-started` says `by-layout=true` (★).
3. `ocr-layer-regions groups=` names n ≥ 1 region layers (★★), and
   `ocr-layer-nested` says `n=n of=n` (★★).
4. `ocr-layer-group` says `made=true groups-made=n+1`, and the save succeeded.
5. Relaunch on the saved file; `ocr_layer_group::remove_all`.
   `remove-ocr-layers-applied` must say `groups-deleted=n+1` (★★★).

On the fixture the layout model finds one title, so n = 1.

# Oracle

Nesting is read from the trace the move itself writes, not from the Layers
panel's indentation: the move's own result is what is in question, and a row
drawn indented can come from a tree the panel built some other way.

# Falsified

- `nest` stubbed to move nothing: FAIL at ★★ (`n=0 of=1`).
- `delete_empty_regions` stubbed out: FAIL at ★★★ (`groups-deleted=1`).

# Not covered

A region layer that still holds words when the removal runs (it is kept by
the trial delete's undo); tables, figures and formulas, which this fixture
does not contain.
