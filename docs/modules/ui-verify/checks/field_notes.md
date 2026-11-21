# `ui-verify/checks/field_notes`

`a_field_drawn_otherwise_says_why_in_the_forms_panel` — the Forms panel
shows, under a field's row, what pdfcer's drawing of that field showed in
place of, or ignored from, its stored state, in the engine's words
(`SynthesizedAppearance::notes`).

# What it drives

A copy of `fixtures/field-display-notes.pdf` (built by
`fixtures/field-display-notes.PROVENANCE.py`), opened off-screen under
`--no-input`, with `PDFCER_DIAG_INVOKE=mode.edit,view.panel_forms` opening
the Forms panel. No clicks are needed.

1. `Amount`, whose `/AA /F` is a script pdfcer does not reproduce, must have
   a `forms-field-note row=0 kind=unformatted` line whose `reason` carries
   the engine's `not a recognised AF*_Format call`, and its row must declare
   `forms.fill.note.0`.
2. `Pick`, a list box with `/TI 7` over two options, must have
   `kind=ignored-top-index` naming 7, and declare `forms.fill.note.1`.
3. `Plain`, drawn as stored, must have neither a note line nor a note region.

# What would make it fail

The panel not reading `WidgetAppearances`, a note kind dropped from
`text::forms::field_display_notes`, or `FieldNotes::show` not drawing the
lines. Removing `show`'s region declaration turns step 1 red.

# What it does not prove

The rich-text and signature notes go through the same reader and sentences,
but this fixture holds no such field; `text::forms::notes` unit tests cover
their wording only.
