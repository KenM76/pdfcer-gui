# `pdfcer-gui/text/panels/formfield`

## Item notes

### `fn the_limitation_note_never_advises_deleting_the_field`

The test it replaces asserted the opposite — it required the string
`"delete this field"` to be present, on the reasoning that *"a note that
only said 'cannot be changed' would leave the operator stuck"*. That
reasoning was sound and its premise was false: the capability existed,
so the operator was not stuck, and the test was pinning a sentence that
recommended destroying a field's name, value and tab position for
nothing.

A test can pin a sentence and cannot know whether the sentence is
true. This one is written in the negative for that reason: it does not
try to say what the note should claim, only that it must not send an
operator down the destructive route again.

### `fn a_typeless_field_is_described_as_unfillable`

A `/FT`-less field is what a bare kid that lost its `/Parent` becomes,
and no viewer can fill it. "Unknown" would read as pdfcer failing to
look; this says what is true of the document.
