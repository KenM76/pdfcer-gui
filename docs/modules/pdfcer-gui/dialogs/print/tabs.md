# `pdfcer-gui/dialogs/print/tabs`

## Item notes

### `fn publish_scale_region`

`ui_rect_visible` and not `ui_rect`, because the options column scrolls: a
radio scrolled out of view must stop being published rather than hand a
driver a rectangle it would click through to whatever is on top.

The percentage inside `ScaleMode::Custom` is ignored — all four modes are
one radio each, and `Custom(0.35)` and `Custom(1.0)` are the same control.

### `fn resolution`

The limit field is drawn in EVERY state. Drawn only while the cap bound, it
vanished the moment the operator raised the cap to the printer's own
resolution — the next job had nothing to cap, so the control that would
lower it again was gone.

### `fn the_typed_order_is_preserved_and_duplicates_are_kept`

Both are *behaviours*, not accidents, and both are shared with the
CLI — which is the whole reason there is one parser. A future "tidy"
that sorted or de-duplicated here would make the same text mean two
different jobs depending on which surface the operator typed it into,
and neither surface would say which.

### `fn malformed_input_refuses_rather_than_recovering`

The property the whole "one parser" argument rests on: a range that
cannot be read must not become a job. Each of these would be a
plausible thing to "recover" from, and recovering would print pages
nobody asked for.

### `fn a_stale_current_page_selects_nothing`

Reachable rather than theoretical: the dialog holds the page index it
opened on, and a document can be closed and a shorter one opened while
it is up. Selecting *something* here would print a page that is not
the one the radio names.

### `fn the_three_tabs_are_distinguishable`

The tabs earn their keep only if their names distinguish them; three
tabs sharing a tooltip would be the drawer this design replaced,
wearing a strip of buttons.

### `const ALL`

An array rather than three literal calls at the draw site, so adding a
tab is one edit and cannot leave the strip and the content branch
disagreeing about how many there are.

### `fn region_word`

Deliberately not [`Self::label`] lowercased: a published region name is
a contract with `tools/ui-verify`, and deriving it from user-visible
copy would break every driven check the next time a label is reworded.

### `fn indices`

The subset filter, the reversal and the copy multiplication are **not**
applied here: they are `pdfcer-print`'s, they have a defined order of
operations (subset → reverse → copies) that is *"the only place a
defect can hide"*, and restating any of it in the shell would be a
second implementation of exactly the kind [`parse_page_range`]'s own
docs argue against.

An unparseable custom range yields an **empty** vector rather than a
guess, which is what lets the dialog say so and withhold the commit
button instead of printing a range nobody asked for.

### `fn parse_page_range`

# Deliberately the same syntax the CLI accepts

Carried across verbatim, with its reasoning:

> Two range parsers would eventually disagree about something like
> `5,1-2` — whether it reorders, whether it deduplicates — and an operator
> moving between the GUI and a script would have no way to know which one
> they were talking to. The syntax is kept identical and the behaviour on
> malformed input is the same: an unparseable range yields NOTHING rather
> than a guess, so the Print button disables and says why instead of
> printing a range nobody asked for.

Note what "the same" *includes*, because two of these are surprising and
both are deliberate: the result **preserves the order typed** (so `5,1-2`
prints 5 first) and **does not deduplicate** (so `1,1` prints page 1
twice). Both fall out of treating the text as a *sequence* the operator
wrote rather than as a set, and both match the CLI.


The argument above was made about the GUI and the CLI. It is the same
argument between two GUI surfaces and stronger: an operator who learned this
syntax on Print is entitled to it working on Insert, and a second parser
here would be the drift that paragraph exists to prevent, one layer in.

### `fn pages_layout`

Takes the planned job rather than the sheet extracted from it, so that every
sentence in the column names the rectangle the job was actually laid out
against — the TURNED sheet — rather than the device's un-rotated default.
Pulling one value out here and another out at the call site is how a tab
comes to describe two different jobs in one column.
