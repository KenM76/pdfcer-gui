# `app::markupband` tests — the Format ▸ Markup band's own assertions

## Why they live in a file of their own

Module beside module, `#[cfg(test)] mod tests;` — the seam
`canvas::annotnodes` and `app::conditions` also take. It is a **subject**
seam rather than an arithmetic one: the parent draws controls, and this
file asserts what they decide.

## What these can and cannot prove, stated first

**They cannot prove an operator can restyle a mark.** Every test here calls
a function directly, and R1's whole point is that a passing unit test is not
a report of working software. Nothing below draws a pixel, opens a combo, or
reaches `set_markup_style`; `tools/ui-verify` is the instrument for that.

What they do prove:

1. **The engine's subtype list is *asked*, not restated.** Every visibility
   predicate is checked against `MarkupStyleSupport::for_subtype` rather
   than against a table kept here — see
   [`each_predicate_reads_the_engines_flag_and_nothing_else`], and
   `NO_SURFACE.md`'s *How to read a row* for why a table alone would be two
   copies of one constant that cannot disagree.
2. **A parked edit sets exactly one field**, which no operator could
   report going wrong: a width that silently reverts one frame after it is
   set reads as *the drag did not take*.
3. **`Clear` reaches the engine as `Clear`.** The fifth state is the one
   change whose whole effect is invisible on screen and visible only in the
   saved bytes, so a test is the only place it can be seen at all.

**Every negative assertion here is paired with a positive control.** A
"not X" assertion is vacuous when the thing that would produce X is absent:
it passes just as well on a build where the feature is switched off
entirely, and only a positive row beside it separates the two.

## Item notes

### `fn every_markup_kind_in_the_register_is_drawn_by_this_module`

The assertion that closes the gap `manifest::COLOUR_SWATCH`'s own doc
comment records: a custom kind the manifest declares and no renderer
matches draws as a caption over an empty band, with nothing anywhere
reporting the mismatch. The shell reserves the item's space, the
application declines to draw, and the only symptom is a gap.

It is asserted through `manifest::CUSTOM_BACKED`, which already pairs a
command id with the kind that draws it and is already tested against the
manifest in both directions. Reading it here makes the chain complete:
manifest → register → renderer → registry.

### `fn this_module_draws_exactly_the_six_markup_kinds`

A further kind added here and not to the manifest is a renderer arm
nothing can ever reach; one added to the manifest and not here is the
empty-band defect above. Only an equality catches both.

It also asserts that this module does **not** claim the Font group's
three or the Markup tab's pen swatch. `COLOUR_SWATCH` is the one that
matters: it is a colour control called `colour_swatch` that sits two tabs
away and means the opposite thing about *when* — it chooses the colour of
the mark you are about to draw, where `MARKUP_STROKE` restyles the mark
you have selected. Claiming it here would put a document-editing verb
behind a control that edits `PdfcerApp::pen`.

### `fn only_one_field_is_ever_set`

The rule `MarkupStyle`'s own doc states, asserted rather than trusted: a
Format tab whose colour picker also restated the current width would
overwrite whatever the operator had set from the other control. The
failure that prevents has no symptom the operator could report — a width
silently reverting one frame after it was set reads as *the drag did not
take*.

Counted by comparing against `MarkupStyle::default()` field by field,
which is the only way to state "exactly one" over a struct of `Option`s.

### `fn changing_which_ends_preserves_the_arrowhead_shape`

The property that makes a four-entry list honest over a nine-value
field: a mark drawn with closed arrowheads keeps them when the operator
moves the head to the other end. Without it the chooser would answer a
question nobody asked, and the rewrite would be invisible here and
visible in another viewer.

### `fn the_four_arrowhead_positions_are_named_and_ordered`

The **order** is the assertion worth making: the chooser lists them
fewest-endings-first, so an operator scanning for "both" finds it last
every time, and a reordering that put the common case first would be a
change to the control's shape rather than to its wording.

### `fn a_swatch_shows_only_colours_it_can_show_without_converting`

Grey is accepted **in** and never produced **out**, which is the
asymmetry `srgb_to_colour` argues: reading `Gray(v)` as an equal-channel
swatch is lossless, and writing an equal-channel pick back as `Gray`
would be pdfcer choosing a colour space the operator did not ask for.

### `fn the_width_range_matches_the_pen_that_authors`

Two ranges for one quantity would let an operator author a 2 pt mark and
then be unable to set 2 pt on it — or, worse, set a width here the pen
could not have produced, so a document would carry marks the shell
cannot make.

### `fn every_value_present`

Deliberately over-supplied: a width, an interior, a dash, an endings
pair and a `/LE` in the file, all at once, on a mark no real subtype
could be. That is the point — a test that fed each control only the
values its own subtype really carries could not tell "the engine's
answer hid it" from "there was no value to show".

### `fn the_engines_answer_is_what_hides_a_control_not_the_spec_arm`

Each predicate consults `MarkupStyleSupport::for_subtype`; none of them
derives an answer from which `MarkupSpec` arm came back, or from which
fields that arm happened to fill. A predicate that read the arm instead
would be a copy of a list `pdfcer-core` owns, and the first subtype to gain
or lose a border is the day the copy is wrong with nothing to say so.

The `Current` fed in has **every value present** for every subtype, so
the values cannot be what differs. If a control is withheld here it is
because `MarkupStyleSupport::for_subtype` said so, and if a control is
offered it is for the same reason.

Both directions: a `Highlight` row asserting three `false`s would pass
with the whole feature deleted — every predicate returning `false`
satisfies it — so the `Square`, `Line` and `Ink` rows are the positive
control that makes the negative one mean something.

The rows are not vacuous. `offers_width` reading only
`self.width.is_some()` turns the `Highlight` row red, and `offers_fill`
reading `self.interior_set` turns the `Line` and `Ink` rows red.

### `fn each_predicate_reads_the_engines_flag_and_nothing_else`

The test above is a table, so it needs this one beside it. This asserts
the **relation**: for every subtype named above, each predicate equals the
corresponding field of `MarkupStyleSupport::for_subtype`, whatever that
field happens to say. The table pins today's behaviour; this pins the
*source*, and the day the engine changes an answer the table fails and this
one does not, which is how a reader is told which of the two to edit.

`NO_SURFACE.md`'s *How to read a row* is the rule being obeyed — two
copies of one constant cannot disagree, so assert a relation and not a
magnitude.

### `fn clearing_the_arrowheads_removes_the_key_where_choosing_a_position_writes_it`

The distinction `StyleEdit` exists for, asserted at the one place this
module decides it. `Set((None, None))` and `Clear` are the pair that
matters: they draw the same line and they are different files.

**Paired, deliberately.** A "not X" assertion is vacuous when the thing
that would produce X is absent, so *"Clear does not write an array"* is
not asserted on its own — the `Set` row asserting an array **is**
written sits beside it, and a `MarkupEdit::Endings` that had stopped
producing anything at all would fail that row.

Not vacuous: re-wrapping the payload as `StyleEdit::Set(..)` in
`into_style` turns the `Clear` row red and leaves the `Set` row green.

### `fn the_removal_is_offered_only_when_the_file_carries_a_line_ending_entry`

[`fill`]'s Clear rule applied to the chooser: a Clear beside a mark that
has nothing to clear is a control whose only possible effect is an undo
entry the operator did not earn. It needs its own field because
`spec_from_dict` cannot answer it — Table 176's default means an absent
`/LE` and a written `[/None /None]` both read back as `(None, None)`,
which is right for a reader whose subject is the picture and useless to
a control whose subject is the difference.

Both directions again. "Absent when the key is absent" alone would
pass with the action deleted.

Not vacuous: `offers_endings_clear` returning `self.offers_endings()`
turns the first assertion red.

### `fn a_mark_that_could_not_be_read_offers_nothing`

Asserted against `for_subtype(b"")` rather than against three literal
`false`s, for this module's whole reason: an all-`false` literal here
would be the last surviving copy of the engine's list, three entries
long.
