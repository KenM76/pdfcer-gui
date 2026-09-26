# `pdfcer-gui/canvas/overlay/tests`

## Item notes

### `fn an_annotations_ghost_box_is_its_own_rect_and_grip_box_is_left_alone`

> *"the Markup Items don't have a live preview — the bounding box stays
> the same size when I drag the handles."*

[`grip_box`] reads `SelectionState::outlines`, which holds **page
content** entries; a markup annotation's box lives on `AnnotSelection`.
So `grip_box` answered `None` for every annotation, the `if let` in
`canvas::painting` never ran, and the resize ghost was unreachable for
exactly the selections he was dragging.

⚠ **Both halves are asserted, and the second is the one that keeps this
honest.** A `ghost_box` that simply answered `Some` for everything
would satisfy the first; the second pins that it is the *annotation's*
rectangle and not some other box that happens to exist.

And `grip_box`'s own behaviour is asserted **unchanged**, because the
tempting fix was to widen it instead — which would have altered what
`pressing::grabbable` measures and what the published `canvas-grip-box`
rect means to a driven check. Two callers wanting the content box is
why this is a second function rather than an edit to the first.

### `fn nothing_selected_has_no_ghost_box`

A ghost box for an empty selection would put a preview rectangle on
screen for a gesture aimed at nothing — and, worse, it would make the
assertion above pass on a build that had learned nothing about
annotations.

### `fn the_marquee_wash_is_translucent_and_keeps_the_themes_hue`

Asserted through `to_srgba_unmultiplied` rather than through `.r()`,
and **approximately**. Both halves of that are the point:

- [`Color32`] stores **premultiplied** components, so a translucent
  blue reads back as `(11, 23, 38)` from the plain accessors and looks
  as though the hue was lost. It was not, and "fixing" that by dropping
  the alpha would be the wrong repair.
- Premultiplying at alpha 48 and dividing back out is lossy — 60
  returns as 58 — so exact equality would be asserting the precision of
  egui's colour storage rather than the property this function has.

### `fn the_move_ghost_is_translucent_and_keeps_the_themes_hue`

Asserted through `to_srgba_unmultiplied` for the reason [`ghost`]'s own
docs give, and approximately because premultiplying and dividing back
out is lossy.

### `fn the_current_find_hit_differs_by_emphasis_and_keeps_the_themes_hue`

Both halves are asserted because both are the design:

- the two alphas differ by enough to read at a glance, so a page of
  hits shows *which one* the readout is counting;
- the hue is the theme's, unchanged, in both — a find highlight that
  borrowed `warn_fg_color` would say *warning* about something that is
  not a warning, and would break the first time somebody restyled the
  warning colour for warnings. `tools/gates/check-theme-colors.sh`
  enforces the general rule; this asserts the specific consequence.

The second signal — the stroke on the current hit — is structural
rather than a colour and is asserted by reading [`draw_find_hits`],
which strokes if and only if `current`.

### `fn the_text_selection_wash_is_readable_through_and_keeps_the_themes_hue`

A find highlight marks one of several candidate answers; a text
selection marks *the characters that are about to be copied*, and the
operator's only way to check them is to read them. So the ceiling is
asserted at compile time against the same value
[`CURRENT_ALPHA`]'s own screenshot-derived bound uses, and the hue is
asserted to be the theme's — a selection wash that borrowed a named
palette entry would break the first time somebody restyled it for its
real purpose. `tools/gates/check-theme-colors.sh` enforces the general
rule; this asserts the specific consequence.

### `fn a_travelling_copy_samples_the_painted_rect_and_never_stretches`

Every number below is worked out from the definition of the projection
rather than read off [`blit_of`], because an expectation produced by the
function under test agrees with it however wrong both are.

⚠ The middle case is the decisive one. A chunk half off the painted
raster must lose the same amount from BOTH rectangles: cropping the UV and
clamping the destination to the full width would stretch the lettering to
fill it, and a picture travelling at the wrong size looks like a rendering
quirk rather than like clipping.

### `fn a_preview_with_geometry_in_it`

Its subpath list is empty, because every predicate under test reads
`ShapePreview::is_empty`, which asks whether there is a SHAPE — one shape
is what makes the operator see anchors travel.

### `fn the_ghost_is_withheld_only_for_a_preview_that_has_something_in_it`

The gate [`ghost_is_owed`] replaced read *is this an inner rung*, which is
not the same question and had the same answer until a text chunk became
selectable. Then a chunk drag previewed **nothing at all**: the ghost was
withheld, `draw_selection` is gated on the same flag so no outline was
drawn, and the chunk boxes are painted where the text still is.

⚠ **The middle row is the one that matters.** `shapes::transformed` returns
a preview with an EMPTY `shapes` vec for a text object, having no path to
transform. A gate that asked `is_some()` would read that as *the real
geometry is travelling* and withhold, which is the defect restated in a
different word.

### `fn the_travelling_copy_is_withheld_only_when_the_geometry_itself_moves`

[`raster_ghost_is_owed`] decides whether a translucent copy of the page's
own pixels travels with the pointer. It withholds on exactly one ground:
the real geometry is already moving on screen.

⚠ The empty preview is the trap. `shapes::transformed` returns a preview
that EXISTS and is EMPTY for a text object, so a predicate spelled
`already_travelling.is_none()` withholds the lettering while the outline
still draws — a box travelling with none of the operator's words in it,
which is the O215 defect in a spelling that reads as a fix.
