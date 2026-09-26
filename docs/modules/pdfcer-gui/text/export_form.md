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

### `fn save_dialog_title`

It names all three formats, because the dialog is where the format is
**chosen** — by the extension — and a title saying only "Export form data"
would leave an operator who wants CSV with no way to know they may ask for
it. The one place this can be said is the one window they are looking at.

### `fn imported`

The two numbers are not decoration and the second is the important one: a
data file may legitimately name a **superset** of this document's fields —
that is the ordinary case when one FDF fills a family of related forms — and
`import_form_data` counts those and skips them rather than failing.

So an operator who imports forty values into a thirty-field form gets
thirty filled and ten skipped, and **nothing anywhere else would tell
them**. A sentence saying only "imported" would be true and would hide the
ten fields they thought they were setting.

`skipped` is mentioned only when it is non-zero. The overwhelming case is
a file that matches, and a bar that narrated "0 skipped" would be adding a
number to be ignored.

### `fn import_unparseable`

Distinct from [`import_unreadable`], and the distinction is the operator's
next move: an unreadable file is a permissions or a path problem, and an
unparseable one means they picked the wrong file or the format is one pdfcer
does not read. The remedies share nothing.

### `fn import_refused`

Its own sentence rather than folding into [`import_unparseable`], because
this is a refusal about the **document** — no form, a certification that
forbids filling, an encrypted file — rather than about the data file. An
operator told their data file was bad when their document is certified would
go and re-export it, twice.

### `fn no_form`

Distinct from [`no_fields`], and the two are not pedantry: a document with
no form has nothing to export and never will until fields are added, while a
document with an empty form is one somebody has already started. The remedy
differs, so the sentence does.

### `fn wrote_fdf`

The format is named in the operator's terms — *"the format Acrobat
uses"* — because `FDF` is an acronym that tells somebody who does not
already know it precisely nothing, and the reason to pick it over the other
two is exactly that other software reads it.

### `fn neutralised`

See the module header. The three things this sentence has to carry:

**How many**, because one is a curiosity and forty is a form somebody has
been putting expressions into on purpose.

**Which**, because the operator may need to check the value survived
intelligibly — a part number `-40C` is a legitimate value that a spreadsheet
would otherwise read as arithmetic, and its owner should know it now reads
with a leading quote.

**What was done**, in the passive voice of a thing pdfcer did rather than a
thing that went wrong. It is a protection, and an operator who reads it as
an error will go looking for a way to switch it off.

The field list is **elided in the middle** past a few names. A status line
is one line; naming four hundred fields would push everything else off it,
and the first and last names are what an operator scans to recognise the
group.

### `fn written_to`

Its own sentence rather than a clause on the format line, because the two
answer different questions and an operator scanning for *"where is it?"*
should not have to read past *"what is it?"*.

### `fn export_failed`

The OS string is passed through rather than re-worded, for
`export_dxf::export_failed`'s reason: *"access is denied"* and *"the device
is not ready"* are different problems with different remedies, and a
generic *"could not write the file"* throws away the only part an operator
can act on.
