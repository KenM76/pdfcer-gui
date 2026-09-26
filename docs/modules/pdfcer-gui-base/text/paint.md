# `pdfcer-gui-base/text/paint`

## Item notes

### `fn fill_label`

"Fill" and "Line", not "fill" and "stroke". *Stroke* is the PDF word and
the drawing-office word is *line* — the same vocabulary rule
`text::formfield`'s header states, applied one panel along.

### `fn recoloured_partly`

The operator asked for exactly this shape: *"a selection of twelve strokes
where three are in a colour space pdfcer will not rewrite needs to say 'nine
changed', not 'done'."*

### `fn subject`

The count is the whole safety of a multi-object colour control. A marquee
on a CAD sheet routinely takes hundreds of objects, and *"Fill"* over a
swatch says nothing about how many things pressing it changes. Word,
Illustrator and Inkscape all report the selection size somewhere permanent;
this panel has no status strip of its own, so the row says it.

`not_paths` is reported separately rather than folded into the count,
because they are two different facts and only one is about what will change.
A marquee over a table catches its rules **and** its labels, and an operator
who recolours it needs to know the labels were not included — otherwise the
text staying black reads as the control half-working.

### `fn mixed_named_inks`

The state O89 called out as the hard one: *"A mixed selection containing one
spot ink must not let a screen colour flatten it."* It does not.
`EditSession::set_object_paint` refuses each named-ink member by name and
reports it, so the plate is safe whatever this panel draws; what this
sentence adds is that the operator **knows before pressing**, rather than
finding out from a count afterwards.

It names the inks where the file names them, for
[`undecoded`]'s reason: *"this stroke is spot ink PANTONE 300"* tells a
drawing office what it needs. An unnamed undecodable space contributes to
the count and not to the list, because there is nothing truthful to call it.

The list is capped at three names. A selection of two hundred strokes in
nine separations would otherwise put a paragraph on a 180-point panel and
the sentence would be scrolled past — which is the failure mode
`text::panels::fonts`' two-word verdicts were shortened to avoid.

### `fn undecoded_across`

The single-object refusal, widened to say how many. It is
[`undecoded`]'s sentence for `total == 1` — deliberately word for word, so
an operator who selects one spot-inked line and then selects five reads the
same explanation rather than wondering whether the second one is a different
state.

### `fn mixed_hint`

Not an error and not a refusal: the control still applies. This is the
indeterminate state every editor in the class shows, and the sentence says
what it means — there is no one colour to open on, and picking one sets all
of them.

It names **shapes**, where the clicked-text twin
(`crate::text::panels::textobject::mixed_hint`) names **words**, and the two
exist separately for a defect that shipped for twenty minutes and could not
have been caught by a test: `panels::properties::swatch` is shared by both
rows, its first draft reached for the text sentence inline, and a selection
of paths was told *"These words are not all one colour."* Both strings
compile, both render, and the wrong one is grammatical. The widget takes the
sentence as a parameter now; see its `show`'s doc comment.
