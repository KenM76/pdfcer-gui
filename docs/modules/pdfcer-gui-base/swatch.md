# `swatch` — a form box's `/MK` colour control

The swatch behind a form box's background and border colour (`/MK` `/BG` and
`/BC`). Every other document colour in the shell — marks, the pen, link
borders, text and paths — goes through `colourpick`; this widget exists
because one `/MK` key has states a colour picker has no place for: *absent*,
the empty array that states *no colour*, and a key pdfcer will not overwrite.

The sentence for a key it cannot show is the caller's job on purpose: what
to say depends on what the key holds, and a widget that worded it would word
some case wrongly. This widget refuses to draw a control; the sentence that
stands in its place belongs to whoever knows what the key is.

## ONE UNDO STEP PER GESTURE, and why this widget could not be
## `ui.color_edit_button_srgb`

`egui`'s own colour button marks its response **changed on every frame of a
drag inside the picker** (`color_edit_button_hsva` calls
`button_response.mark_changed()` from inside the popup body). A caller that
acts on `.changed()` therefore authors **one document edit per frame** while
the operator drags across the saturation square — sixty content-stream
rewrites a second, sixty undo entries.

A colour popup has no `drag_stopped`, so the commit event has to be *the
popup closing* — which means owning the popup id, which means owning the
button. Hence this module. Open-and-close-without-touching returns `None`, so
idly inspecting a colour never writes to the document.

## Rule 4 — nothing here reaches the canvas

This widget draws in a dock panel. It marks no page and renders no preview:
the document changes only when the caller's action is applied. The swatch's
own fill is not a theme colour — it is the document's content — and says so
on the line.

## Item notes

### `const ASPECT`

Wider than tall, which is what a colour *swatch* looks like everywhere
this operator works — Word's font-colour button, Illustrator's fill chip,
SolidWorks' line-colour control. A square reads as a button with a coloured
glyph; a bar reads as a sample of the colour itself.

### `fn disc`

Hand-drawn rather than a `Button` with a rounded corner radius, because a
rounded rectangle at this size reads as a button with a tint and the point
of the shape is that the operator sees a **disc** — the thing a radio
button's background will actually be. It keeps the button's own interaction
(a click opens the popup) by allocating an interactive rect of the same size
and painting into it.

### `enum MkValue`

One widget's `/MK` `/BG` is one key on one dictionary, and Table 189 gives
it four states: absent, the empty array that states *no colour*, a DeviceGray or DeviceRGB value, and a DeviceCMYK
value. Both colour states are carried as a `colourpick::Ink` in the space the
file states, and the popup is `colourpick::body` with every space and no
opacity (`/MK` has none), so a CMYK background opens on its own numbers and
is written back in CMYK.

The two states that are not a colour are deferred to the caller with their
mark and note: which of them a widget is in is a fact about `/MK`, and `/MK`
is not a thing this file knows about.

### `fn show_mk`

Returns `Some` **only** on the frame a choice was made: the picker closed
after the operator moved it, or the *no colour* entry was pressed. The
commit rule is the one argued at the top of this page. Its state,
`MkEditing`, lives in `egui`'s temp data rather than on a draft struct: a draft
holds a *reading of the document* and is invalidated when the document moves,
while this holds *where the operator's finger is*, which must never outlive the
popup.

The *no colour* entry commits **immediately** rather than on close, and
that is not an inconsistency: it is a discrete press, not a drag, so there
is no run of intermediate values for a close-edge to collapse. It closes the
popup itself, because `PopupCloseBehavior::CloseOnClickOutside` would
otherwise leave a picker open over a widget that no longer has a colour.
