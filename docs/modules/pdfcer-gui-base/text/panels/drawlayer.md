# `text::panels::drawlayer`

The words for the current layer: the layer the operator picks in the Layers
panel for new content (`OpenDoc::draw_layer`). Design: `DESIGNS.md`, "Row 74 —
a current layer that new content goes on".

- `choose_hover` and `current_hover` are the two hovers on a layer's name. The
  current one says how to stop, because the same click both sets and clears.
- `receipt` is appended to an add's notes, so the status line says where the
  new content went. The canvas shows nothing different: applied content renders
  as saved, and layer membership is not drawn.
- The refusal for an add onto a hidden current layer is
  `LayerRefusal::DrawLayerHidden` in `text::panels::layeredit`, beside the
  other layer refusals, so it is recorded through `Declined::Layer`.
