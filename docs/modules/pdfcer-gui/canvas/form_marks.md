# `pdfcer-gui/canvas/form_marks`

## Item notes

### `fn spotlight`

**Every widget of that field**, not one. A field may be painted in
several places — a header repeated on each page, a radio group — and
spotlighting one of them would answer *"where is this field"* with a half
truth. `WidgetBox::field` is the fully-qualified name, so the filter is the
same identity the panel wrote.

Draws nothing when the panel is not pointing at anything, which includes
every frame the panel is not on screen: it clears the channel before its rows
draw, so an unhidden panel with nothing focused leaves it empty.

### `fn shade`

The whole of the feature's drawing. `crate::canvas::overlay::draw_field_shade`
owns the colour and the alpha and argues both; this owns *which* boxes and
*whether at all*.

One painter per page view rather than one for the lot, because a box's rect
is in its own page's space and `PageMapping` is per page — the same reason
[`cursor`] walks the views rather than the boxes.

### `fn authoring_boxes`

## Why this is not [`shade`] with a different argument

Three things differ, and each is the reason the other two are not enough.

1. **The set.** [`shade`] walks `Placed::boxes` — the widgets a click can
   *fill*, which excludes a drop-down, a push button and a signature by
   design. Edit mode hit-tests `Placed::targets`, every widget with a
   rectangle, and a wash that skipped the kinds the operator most often
   wants to reposition would reproduce his complaint for exactly those.
2. **The mark.** A wash alone is invisible on a field with no `/MK`
   background and no value — which is most of a form being authored. This
   draws the hairline too; see [`crate::canvas::overlay::draw_field_target`].
3. **The gate.** [`shade`] is the operator's `shade_form_fields` display
   option, about reading a form. This is Edit mode's own affordance and is
   unconditional, which is the conventional behaviour of every authoring
   surface in this product class: Acrobat's Prepare Form outlines every
   field whether or not field highlighting is switched on, because in that
   mode the boxes *are* the subject.

Drawn before the selection outline and its grips, so the one box the
operator has picked still reads as picked — a selection that had to be
distinguished from its neighbours by degree rather than by kind is the
failure [`spotlight`]'s own doc comment argues against.
