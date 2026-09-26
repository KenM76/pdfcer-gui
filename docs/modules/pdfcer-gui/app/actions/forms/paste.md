# `pdfcer-gui/app/actions/forms/paste`

## Item notes

### `fn widget_index_after_paste`

Re-read from the document rather than derived from the outcome's
`widget_ids`, because the two address spaces differ: the outcome names
`ObjId`s and `SelectedField` wants an **index within the field**. Reading
the field back is the only thing that knows both.

Zero when the field cannot be found, which is unreachable on the success
path and is a defensible index rather than a panic if it ever is not.

### `fn paste`

# Why this is not [`author`]

Reusing [`author`] would be the right instinct — one authoring path, not two
— and it is wrong here for a reason that shows up in the file rather than on
the screen. `New*Field` is a **spec**: geometry plus a dozen booleans. So a
re-authored paste can carry only what the spec can *express*, and these are
readable on `forms::Field` and writable nowhere — `/DA` (the font, its size
and its colour), `/Q`, `/DV`, `/AA`, `/MK`'s border and background colours,
`/BS`'s styles beyond solid, the `/Ff` bits no spec names, and the baked
`/AP`. A shell that re-authored would have to disclose that loss and carry a
hand-written table of it, and such a table *"rots silently every time we add
an authoring key."*

`paste_field` does not *express* properties, it **carries** them, so there is
no table to maintain.

⇒ [`author`] is still the right verb for the **dialog**, where the operator
is choosing values and a spec is exactly what a form of controls produces.
The two are not duplicates; they are authoring-from-choices and
authoring-from-a-source.

# The disclosures are the ENGINE's and are surfaced verbatim

`FieldPasteOutcome::disclosures` is a `Vec<String>` covering a dropped
value, dropped actions, a carried calculation and its `/CO` registration, a
**renamed font resource**, an ignored rectangle size on a radio group, the
tab-order position, a dropped structure-tree link and a reused accessibility
name. Rule 4's off-canvas obligation lands there, and this function does not
re-derive a word of it — *one fact, one wording*.

The engine's own note calls it *"not optional reading"*, so `vector_edit`
carries it to the status row exactly as every other verb's disclosures.

# Selecting what landed

[`author`]'s O53 behaviour, kept: a newly placed field is left selected so
the grips are already there and the next drag is already live. Widget 0 for a
new field, because that is the one the engine placed first; for a duplicate
the operator's own widget index is unknown until the outcome comes back, so
the **last** widget of the field is the one that just arrived.
