# `pdfcer-gui/app/actions/funnel`

## Item notes

### `enum EditScope`

The whole design is in which of these two is the *default*. See
[`crate::app::state::pageepoch`] for the measurements and for why getting
this wrong is worse than the slowness it fixes.

### `fn vector_edit`

`page` here is **for the trace only**: a third of the
call sites pass a literal `0` because their verb has no page (a bookmark, a
font, a metadata field). It is therefore **not** a safe narrowing signal,
which is exactly why [`EditScope::Page`] has to be passed deliberately
through [`vector_edit_on_page`] rather than inferred from this argument.
Inferring it would have narrowed `set-info-field`, `embed-fonts` and
`paste-bookmark` to page 0 and left every other page's thumbnail stale.

### `fn vector_edit_on_page`

Identical in every respect except the invalidation breadth, so the four-step
protocol still exists once. The scope argument is separate from `page`
deliberately — see [`vector_edit`]'s note on why `page` cannot be trusted
as a narrowing signal.

# What a caller is asserting by using this

Not *"it only touched that page this time"*. **Every** call of this verb, on
every document, with every operand, changes nothing a rasteriser would draw
on any other sheet. If that is a property of the operands rather than of the
verb, use [`vector_edit`] and take the extra work.
