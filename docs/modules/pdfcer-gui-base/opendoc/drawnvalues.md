# `opendoc::drawnvalues`

Form fields with no stored drawing, drawn for the screen only.

# What the file asks, and what Acrobat does

A widget with no `/AP` `/N` — commonly a form written by a program that sets
`/AcroForm /NeedAppearances true` and leaves drawing to the reader — paints
nothing when rendered as stored. Acrobat and other readers draw the value,
border and background themselves. The engine's rasterizer does not
(request G230), so this module does it from outside the renderer.

# The display copy

`OpenDoc::display_copy` builds, once per `(edit_epoch, session pointer)`:

1. `EditSession::to_incremental_bytes` — the current revision as bytes. With
   no edits these are the opened bytes; the parse is lazy, so the cost is
   the drawing, not the document's size.
2. `Document::from_bytes` and `SettingsExt::open_session` — a second,
   private session.
3. `fill_text_field(name, "")` for each empty text field with an undrawn
   widget. `regenerate_appearances` skips a text field with no `/V`, and an
   empty fill is what draws its border and background.
4. `regenerate_appearances` — every field holding a value is drawn.

`OpenDoc::display_session(page)` hands the renderer that copy for the pages
holding an undrawn widget, and the real session for every other page. The
page view and the thumbnails both go through it.

Measured: 0 ms on `fixtures/need-appearances-fields.pdf`; 6 ms on the
5.7 MB dense site plan with that form merged in.

# Invariants

- **The open document is never written.** The copy is never saved and never
  on the undo stack; Save with no edit writes the opened bytes. The ui-verify
  check `a_form_without_drawn_values_draws_and_clicks` asserts it byte for
  byte.
- **No marking.** The copy draws what *Redraw values* would store, so the
  screen matches the document after a redraw and save (rule R8b). The
  disclosure that the file stores no drawing is off-canvas, in the Forms
  panel (`forms_canvas_screen_drawn_note`).
- **A copy that cannot be built is `None`** (a signed form, a fill guard that
  refuses): those pages render as stored, the trace line
  `drawn-values … outcome=failed reason=…` says why, and the panel says the
  fields show empty (`forms_canvas_undrawable_note`).
- Hit-testing is unaffected: `canvas::forms::boxes` places every widget from
  `/Annots` and `/Rect`, drawn or not.

# Limits

Buttons are not drawn: `regenerate_appearances` leaves check boxes and radio
buttons alone, so one with no `/AP` still shows empty. A single-line value
is drawn where the engine's generator puts it (the top of a tall box), not
centred as Acrobat does. Both are in G230; when the renderer draws undrawn
widgets itself, this module is deleted.
