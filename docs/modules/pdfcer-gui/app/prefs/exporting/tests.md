# `pdfcer-gui/app/prefs/exporting/tests`

## Item notes

### `const IMAGE_FORMATS`

Deliberately **not** `ImageFormat::ALL`. A round-trip test that draws its
input from the same accessor the production code uses proves the two agree
with each other; it does not prove either agrees with the file format. This
list is the test's own statement of what the file must carry, so a variant
added upstream goes red here — at the `match` in [`image_format_key`] — with
a message about the file rather than silently expanding a sweep.

### `fn the_text_default_is_what_the_dialog_used_to_hard_code`

The `AllPages` line is the one worth looking at: it disagrees with the image
window's `CurrentPage` on purpose, and that disagreement is why the two
scopes are separate keys.

### `fn the_dxf_defaults_still_match_the_engine_and_a_change_upstream_lands_here`

[`ExportDxfPrefs::default`] writes literals rather than delegating to
`DxfOptions::default()`, so that an engine change to a default cannot alter
this window's behaviour on a `cargo update` with nothing in the diff. This
test is the other half of that decision: the day the engine does change one,
**this goes red** and somebody decides, in a commit, whether the window
follows.

Without it the literals would be a silent fork rather than a deliberate one.

### `fn every_token_round_trips_and_is_distinct_within_its_enum`

The second half matters as much as the first: two values spelled the same
way makes one of them unreachable from a hand-edited file, and the writer
would keep producing a file the parser reads as the *other* value with no
note raised anywhere.

### `fn typed_has_no_token_and_degrades_to_the_windows_own_default`

The failure this prevents is specific and invisible from the code: a window
restored into *Pages* with an empty range box, its Export button greyed, and
nothing on screen saying why.

### `fn the_dxf_units_synonyms_are_read_but_never_written`

The writer emits exactly one spelling — asserted here too, because a writer
that started emitting a synonym would make the file teach a vocabulary its
own comment block contradicts.

### `fn fields_of`

Returns the field names in declaration order. The shape it depends on —
every field on one line, `pub`, no trailing comment — is the same shape
[`crate::app::prefs::printing`]'s equivalent depends on, and it is stated in
that struct's doc comment for the same reason.

### `fn every_field_of_every_group_is_both_written_and_parsed`

# Why a source-text check rather than a value round trip

A round trip over [`ExportPrefs`] — write, parse, compare — is also in this
file and is the stronger check *for the fields it covers*. It cannot cover a
field nobody wrote: a new field simply keeps its default on both sides of the
comparison and the round trip stays green while the preference is inert.

This test closes that hole from the other direction. It proves only that the
field is *mentioned* in both halves, which is weak — and is exactly the
difference between a preference that is wired up and one that is not.

⚠ It is blind in one direction by construction: it cannot see a field
mentioned in the right place and used wrongly. That is what the round trip
below and the driven `ui-verify` check are for.

### `fn every_remembered_field_is_read_back_by_its_dialog`

The half of O196 that no compiler and no other test in this file can see,
and the half most likely to rot. Ported from `super::super::printing`'s
`every_remembered_field_is_read_back_by_the_print_dialog`, which found the
shape first for O166, and generalised over the three groups.

# Why the writing half is free and the reading half is not

Each window's `habits()` is a struct literal with **no
`..Default::default()`**, so *writing* a new preference is compiler-enforced:
add a field to one of these groups and that function stops building.

The *reading* side has no such property. A window's `open` is a struct
literal of the **dialog's** fields, and a field of `ExportImagePrefs` that
nothing over there mentions compiles perfectly. The window opens on its
hard-coded value while the preferences file dutifully records, writes and
reloads a number nobody ever looks at.

Every other test in this module would still pass: the round trip works, the
tokens are unique, the defaults match, `write_block` and `parse_key` both
name the field. The only symptom is the operator saying *"it still doesn't
remember the DXF units"*, months later — which is, word for word, the
complaint this module exists to answer.

# ⚠ The DXF row points somewhere else, and that is the design

[`crate::dialogs::export_dxf::ExportDxfDialog::open`] does not mention
`remembered.units` at all. It calls
[`crate::dialogs::export_dxf::seeded_options`], which is where the ordering
rule lives — the operator's habit first, the page's own calibration second —
lifted out precisely so that rule could have a unit test.

So this table names, per group, **the function that actually reads
`remembered`**. The two alternatives were both worse. Pointing every row at
`open` reports a false failure for DXF. Searching the whole file lets a
mention in a doc comment satisfy every row, and this project has been caught
by exactly that: a gate keyed on a name, discharged by prose.

# What it does not prove

⚠ It is a source-text check, so it proves the name is *mentioned* in the
right function, not that it is used correctly. That is still the whole
difference between a preference that is wired up and one that is silently
inert. The driven `ui-verify` check is what proves the value survives a
restart, and it is the only thing that can.

### `fn the_dragvalue_ranges_are_the_constants_the_file_clamps_to`

# The rule this makes structural

The four bound constants at the top of [`super`] carry an instruction in
their own doc comment: *"if a control's range changes, change it here in the
same commit — the round-trip is only honest while the two agree."* That was
a thing to remember, and a thing to remember has no instrument. Both boxes
did in fact repeat their literals — `1.0..=4800.0` and `1..=100` — while the
constants sat beside the clamp, so the two halves could drift apart in a
single edit and nothing in the toolchain would notice.

# What drifting apart would cost

The preferences file clamps a read value into these constants. If a box were
widened and the constants were not, an operator could drag the resolution to
a number the box accepts, close pdfcer, and reopen it to find a different
number — because the file clamped on the way back in. That is O196's
complaint arriving through a different door: the window forgot what it was
told, and nothing anywhere said so.

# Why the source text and not the values

There is no value to compare. The range lives inside a builder call and is
consumed by egui, which exposes it again to nobody. What *can* be measured
is whether the call names the constant, and here that is the whole of the
rule rather than a proxy for it: a literal and a constant cannot both be
written in the same position, so naming the constant is exactly the property
wanted.

### `fn parse_block`

Returns the number of keys the parser **accepted**, so a caller can assert on
it; a key that fell through as [`KeyOutcome::NotMine`] is counted separately
and is a failure in every caller here, because everything in this block is
by construction ours.

### `fn the_defaults_are_written_too_so_the_file_teaches_its_own_vocabulary`

The second half is the point: a fresh profile must still find all thirteen keys
in the file, with their comment blocks, because *a preference nobody can
discover is a preference nobody has.*

### `fn a_non_finite_resolution_is_a_bad_value_and_never_reaches_the_window`

This is the arm most likely to be written as a bare `.parse().ok()`, and a
NaN is the worse of the two: it survives `clamp` unchanged, so it would be
*stored*, handed to the window, and every comparison against it would answer
`false`.

### `fn a_fractional_resolution_survives_the_file_exactly`

`f32::to_string` is shortest-round-trip, which is why the file can hold a
float at all — but *"the standard library says so"* is a claim worth one
cheap measurement, because a change to how this module writes the number
(a format specifier, a rounding step) would break it silently and the
operator's 150.5 would come back as 150.
