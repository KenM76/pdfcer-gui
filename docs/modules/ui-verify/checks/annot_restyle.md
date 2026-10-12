# `ui-verify/checks/annot_restyle`

Three checks of the Properties panel restyling annotations whose look another
program drew:

- `a_text_box_another_program_drew_is_restyled_on_request`
- `a_stamps_opacity_and_words_change`
- `a_markers_icon_is_recoloured_on_request`

# What they drive

A copy of `fixtures/restyle-annots.pdf` with
`PDFCER_DIAG_INVOKE=mode.edit,file.properties`. One 400 x 400 page carries a
`/FreeText`, a `/Stamp` and a `/Caret`; the text box and the caret have
hand-drawn appearances, so the engine refuses to restyle them until the
operator allows a redraw. Each check selects its annotation with a scripted
click inside its `/Rect` and requires an `annot-select` line.

**Text box.**

1. The rows `properties.markup.textbox` and its `.width`, `.size`, `.border`,
   `.fill` and `.ink` regions are all declared.
2. `3` typed into the border width leaves no `set-text-annot-style-applied`,
   and leaves a `redraw-offer why=text-box-foreign`.
3. `properties.markup.any.redraw` → `set-text-annot-style-applied` with
   `was_foreign=1 redraw=true frame=true`: the width asked for was applied,
   replacing the foreign look.
4. `16` typed into the text size applies directly, `was_foreign=0 size=true`.
5. A colour picked in the background swatch applies with `frame=true`.

**Stamp.**

1. `40` typed into `properties.markup.any.opacity` →
   `set-annot-opacity-applied current=0.4`.
2. `CHECKED` typed into `properties.markup.textannot.words` →
   `set-text-annot-style-applied label=true`.

**Caret.**

1. A colour picked in `properties.markup.any.marker` leaves no
   `set-marker-colour-applied` and leaves `redraw-offer why=marker-foreign`.
2. `properties.markup.any.redraw` → `set-marker-colour-applied was_foreign=1
   redraw=true`.

A swatch commits when its picker closes. `dimdrive::pick_colour` clicks the
lower part of `<region>.picker` (the saturation/value square) and presses
Escape.

A `redraw-offer` line with no `properties.markup.any.redraw` button in the panel
is a FAIL, not a SKIP. A refusal the operator has no way to take is the defect
these checks exist to catch.

# Driven off-screen

Scripted pointer, window off every monitor, `spec.place = false`; they run
under `--no-input`.
