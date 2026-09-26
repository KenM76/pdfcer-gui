# `pdfcer-gui/text/export_form`

## Item notes

### `fn name_list`

It keeps the FIRST few and says how many were dropped, rather than
sampling from the middle or the end. A form's field names share a prefix —
`Revision.Row0.Date`, `Revision.Row1.Date` — so the opening names are what
identify the group, and an operator who recognises the prefix does not need
the rest.

### `const MAX_NAMED_FIELDS`

Four. Enough to recognise a group — a revision table's four columns are the
commonest case this fires on — and few enough that the sentence still fits a
status line beside the count that precedes it.

### `fn the_neutralisation_disclosure_reads_as_an_act_not_an_alarm`

The failure this guards is a rewording toward alarm. pdfcer performed a
protection the operator did not ask for and should keep; a sentence
containing "error", "failed" or "warning" would invite them to go
looking for the switch that turns it off.

### `fn a_long_field_list_is_bounded`

Asserted against a real shape rather than a token: a form whose every
field is formula-shaped is a revision table with forty rows, and that is
the case that would otherwise push the count off the line.

### `fn the_two_empty_states_are_told_apart`

They describe states with different remedies — add a form, or add fields
to the one you have — and a single sentence covering both would be
vague about the only thing the operator needs.
