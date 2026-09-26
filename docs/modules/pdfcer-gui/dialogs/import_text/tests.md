# `pdfcer-gui/dialogs/import_text/tests`

## Item notes

### `fn the_window_opens_on_a4_whatever_order_the_engine_lists_its_sheets_in`

`PaperSize::ALL`'s order is the engine's business and it has said the table
will grow. A hard-coded index would silently open on a different sheet the
day one is inserted before A4 — and a window that opens on the wrong paper
is a defect an operator only notices *after* importing, when the pages are
already in his document.

The test asserts the **id**, not the index, for the same reason the
implementation looks it up by id: an index asserted here would go green
against a reordered table while the window opened on Letter.

### `fn the_template_keeps_every_engine_default_the_window_does_not_control`

`PageTemplate` has ten fields and this window offers four. The other six —
`leading`, `alignment`, `color`, `unmappable` and the two margins that are
not separately controlled — must arrive as `PageTemplate::new()` set them,
so that a field the engine adds tomorrow comes with the engine's default
rather than a zero this shell invented.

`unmappable` is the one that matters most: its default is `Refuse`, which
is what makes a text file full of characters the face cannot write **stop**
rather than arrive with silent gaps. A window that reconstructed the
template field by field could drop that without any test noticing.

### `fn the_four_controls_reach_the_template_and_the_margin_reaches_all_four_sides`

The other half of the test above, and it needs saying separately: a build
that returned `PageTemplate::new()` unchanged would pass every assertion
there and ignore every choice the operator made.

The margin is asserted on **all four** sides. The window offers one
spinner and the engine has four fields; a build that set only `margin_left`
would produce a page with text running off three edges, and it would look
like a rendering fault.

### `fn a_sheet_index_past_the_end_of_the_engines_list_clamps_rather_than_panicking`

`PaperSize::ALL` can **shrink** between builds as well as grow — the engine
says the table moves — and this window stores an index. Clamping rather than
indexing is what stops a window failing to open, which is a far worse
outcome than opening on the wrong sheet with the chooser right there.

Asserted at `usize::MAX` rather than `len()`, because the interesting
failure is not off-by-one — it is a stored value from a completely different
table.

### `fn the_radios_carry_the_frozen_page_into_the_engines_position`

The conversion is the reason `Where` exists as a local enum at all —
`dialogs::insert_pages` states it: two of the four need the current page
index, which the radio does not carry and the dialog does.

It asserts against page **7** rather than 0, because `Before(0)` and
`Start` are the same position and a test using the first page could not tell
a build that confused them from a correct one.

### `fn every_offered_face_has_its_own_label`

`face_name`'s `_` arm answers `"Helvetica"`, which is correct for
`Std14::Helvetica` and would be *silently wrong* for any face added to
[`FACES`] without a matching arm. This is what notices.
