# `pdfcer-gui/text/settings/overprint`

## Item notes

### `fn exactly_one_scope_is_marked_as_the_engines_default`

Asserting **exactly one** rather than "the right one carries it"
catches the other half: a suffix added to a second label by hand, which
would leave two options both claiming to be what pdfcer does.

### `fn no_scope_label_spells_out_the_default_itself`

The suffix is derived; a label that spells it out would be a second,
unsynchronised claim about the same fact — which is exactly how the
original defect happened.
