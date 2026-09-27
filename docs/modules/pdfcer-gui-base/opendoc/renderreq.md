# `pdfcer-gui-base/opendoc/renderreq`

## Item notes

### `fn render_key`

The staleness key the shell wants, built from the same constructor the
worker labels its output with — see
[`crate::render::worker::RenderKey::new`]. One arithmetic path, so
"what I want" and "what I have" cannot disagree about how a key is
spelled.

`pub(crate)` for one reason: a panel whose control is blocked on
something else needs to be able to assert that *its* input reaches the
key. `crate::panels::layers` does exactly that — see
`the_render_key_no_longer_blocks_a_layer_toggle` — which is how the
next person to restore that checkbox learns which of its three
preconditions is still open without re-deriving the answer.

### `fn render_key_for`

[`Self::render_key`]'s general form, and the one a continuous strip
needs: every visible page is rendered with the same scale, annotation
stance and layer override, so the only thing that varies between them
is the page index. Written as one function with the current page as a
special case, rather than two, because two would be two places for the
annotation stance to be forgotten — and a key that omitted it would
leave the strip's pages showing annotations after the operator turned
them off, while the current page obeyed.

### `fn region_for`

The page check is the whole of this method's job. Without it a
region computed for page 4 would be applied to page 5 as well, and
both rectangles are valid — so the wrong part of the neighbour would
be rasterized with nothing reporting an error.

### `fn render_request_for`

The one constructor for a [`RenderRequest`], so the current page and a
strip page cannot be rendered with different options. It exists here,
on the document, rather than in [`crate::render::settle`] because the
annotation stance and the layer override are **private** fields of this
type — and they should stay private: they are changed through
[`Self::set_annotations_visible`] and [`Self::set_hidden_layers`],
which are the methods that keep the staleness keys moving.
