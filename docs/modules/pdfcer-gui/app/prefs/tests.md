# `pdfcer-gui/app/prefs/tests`

## Item notes

### `fn every_preference_round_trips_through_the_file`

The property a preferences store exists for, and the one a hand-written
writer and a hand-written parser get wrong first: they are two spellings
of the same vocabulary, and this is what stops them drifting.

Every field is varied, and the three multi-valued enums are varied over
**all** their values rather than one apiece — a writer that emitted a
constant token would pass a single-value check.

### `fn each_overlay_is_written_and_read_on_its_own_key`

The failure this catches is a copy-paste in either the writer or the
parser sending two overlays to one field — which the round-trip above
would only catch for the specific combination it happens to use. Here
each is set alone and the other two are asserted to have stayed off.

### `fn the_defaults_are_the_constants_they_replaced`

`ZOOM_SETTLE` was a compiled-in 150 ms, `raster_scale` had no
multiplier at all, and `ViewState::default` was fit-page with all three
overlays off. A build that never opens the Settings window has to
behave exactly as the build before this module did — the standing rule
for a capability becoming choosable.

### `fn seeding_from_the_shipped_preferences_changes_nothing`

The strongest form of the rule above, and the one a reordering or a
typo in [`Prefs::seed_view`] would break: seeding a default `ViewState`
from default preferences must leave it **byte-identical**. Asserting
the fields one at a time would pass while a fourth field was silently
clobbered; asserting the whole struct will not.

### `fn a_preference_that_hides_guides_does_not_hide_remembered_ones`

Row two of [`Prefs::seed_view`]'s table, and the whole reason that one
field ORs. `canvas::guides::opening` turns the layer on for a document
that has guides saved against it, because *"the presence of the work is
the preference"* — and an assignment here would hide work the operator
did, on the document they did it on, because of a switch they set weeks
earlier about documents in general.

This is the failing direction: preference **off**, view already **on**.

### `fn rulers_and_grid_follow_the_preference_in_both_directions`

The counterpart, and it is what stops the OR being copied to all three
out of symmetry. Neither has any per-document memory, so a `true`
arriving in the view is not evidence of anything the operator did — it
would just be a stale value that the preference could then never turn
off.

### `fn a_bad_line_costs_only_its_own_key`

The fail-soft contract, and the reason it matters here rather than being
inherited politeness: this file is *meant* to be hand-edited, and a
parser that failed a whole document over one typo would punish the
operator for doing the thing the file invites.

### `fn a_trillion_percent_is_accepted_and_the_page_actually_draws_there`

The point of the setting is that the performance trade is his; a ceiling
exists only because `f32` must stay finite — and since both precision
ceilings were removed, the page actually draws there.

### `fn an_infinite_maximum_is_a_bad_value_rather_than_a_clamp`

`inf` would propagate into a scroll extent and blank the canvas, which is
the failure `canvas::geometry`'s guards exist for. Reporting it as
*clamped* would also imply the operator wrote something reasonable.

### `fn the_file_writes_a_readable_number_rather_than_an_exponent`

The preferences file is the operator's to read and edit; a machine-shaped
number there means he cannot tell at a glance what he set.

And it records something the operator will otherwise discover by
reading his own file: **`f32` cannot hold a trillion exactly.** It
stores `999,999,995,904` — a rounding of four thousand parts in a
trillion, four ten-millionths of one percent. At a zoom where one screen
pixel is a millionth of a point, that difference is unobservable; but a
value written back as a number he did not type is worth knowing about
rather than being mistaken for a bug.

### `fn an_out_of_range_settle_clamps_and_says_so`

Reported, not silent: the operator wrote a number and is getting a
different one, which is exactly the kind of quiet substitution the
engine's store spends a note variant on.

### `fn an_off_step_ui_scale_is_rounded_and_reported`

Rounding rather than accepting, because the file is hand-editable and
the slider is not: a value of `1.234` would sit in the control until the
operator touched it, at which point it would jump — a change they did
not make, to a setting they did. Reported for the same reason the settle
clamp is: the operator wrote a number and is getting a different one.

### `fn a_non_finite_ui_scale_is_refused_rather_than_clamped`

The one parse arm in this file that needs a guard beyond `parse()`
succeeding. `"nan"` and `"inf"` are both valid `f32` literals, and
`f32::clamp` **propagates** NaN rather than rejecting it — so without
the `is_finite` check a hand-edited `ui_scale = nan` would flow through
`normalise_ui_scale` untouched and reach `Context::set_zoom_factor`,
which is a window that draws nothing.

Reported as a bad value rather than clamped to an end, because the
operator did not name an end. `inf` is included for the same reason
even though clamping would in fact handle it: two spellings of "this is
not a size" should not get two different treatments.

### `fn an_empty_file_produces_defaults_and_no_notes`

A first run is the expected state, not a fault. Reporting it would train
the operator to ignore the channel that carries the real problems — the
engine's store makes the same distinction and states it in a table.

### `fn the_writer_emits_no_key_the_parser_rejects`

The drift this catches is the one that would be silent in both
directions: a key added to [`Prefs::write_to_string`] and not to
[`Prefs::parse`] makes pdfcer report its **own** file as containing an
unknown key, on every start, forever — and the operator would have no
way to tell that the file they never edited was written by the program
complaining about it.

The round-trip test above cannot see this: it compares the parsed struct
and would pass on a key that was written, unread and defaulted back to
the same value.

### `fn the_preferences_file_lives_beside_the_settings_file`

Asserted against `pdfcer-core`'s own answer rather than by re-deriving a
path, so the two cannot drift — which is the failure this project
already found once, when two callers in one process disagreed about
which home was live.

### `fn an_unstated_page_display_preference_writes_no_key_and_reads_back_as_unstated`

The whole reason `default_page_display` is an `Option` rather than a
`PageDisplay`. `None` has to be expressible on disk, because
`MODES_AND_PANELS.md`'s per-mode rule — Read opens continuous — must keep
applying to an operator who has never stated a preference. A writer that
emitted `default_page_display = single` for `None` would silently override
that rule for everybody, and it would do it the first time anybody's
preferences file was rewritten for an unrelated reason.

### `fn a_remembered_document_outranks_the_standing_preference_which_outranks_the_mode`

| tier | wins over |
|---|---|
| this document's own remembered arrangement | everything |
| his standing preference | the per-mode default |
| the per-mode default | nothing |

Asserted as the expression `lifecycle` actually evaluates, because the
order is the whole design and a reversed `or` would compile, run, and
silently make a global preference override a document the operator had
deliberately arranged.
