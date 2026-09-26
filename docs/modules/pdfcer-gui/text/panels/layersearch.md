# `text::panels::layersearch` — the four strings the Layers search says

**Surface:** the search field at the top of the Layers panel, and the
two lines that describe what it did.
**Consumer:** [`crate::panels::layers`], and nothing else.

Its own module rather than four more functions in
[`super`] — which is at 1,299 lines — for R2's reason, and for a better
one: these four are the only strings in this program that describe **a
filter's effect on a list**, and the wording rules they follow are
particular to that job. Keeping them together is what lets those rules
be written down once and asserted below.

# The three rules this copy follows

**1. Say the number, not the adjective.** *"3 of 16 layers"* and never
*"some layers are hidden"*. An operator scanning a filtered list needs
to know whether the thing they are looking for is absent from the
document or absent from the *view*, and a count answers that in one
glance. `panels::comments`' `comments_excluded` is the precedent and it
is quoted in its own docs: *"the panel discloses the filter in numbers
instead of in doctrine."*

**2. The empty case says the query back.** Not *"No matches."* — which
is what a broken panel says too — but the text that was typed, in
quotation marks. There is exactly one way for an operator to be sure a
search ran, and it is seeing their own query repeated by the thing that
ran it.

**3. Nothing is said when nothing was filtered.** R9's shape applied to
prose: a panel showing every layer must read exactly as it did before
this feature existed. That is why [`narrowed`] returns `Option` — the
`None` is not "no text available", it is *"there is nothing to
disclose"*, and a caller that unwrapped it to an empty string would put
a blank line above every unfiltered list forever.

## Item notes

### `fn a_query_containing_quotes_is_still_shown_as_typed`

The sentence uses typographic quotes precisely so that a query
containing a straight `"` does not close the quotation early and
read as a different string from the one typed.

### `fn the_field_tooltip_says_it_matches_the_name`

Decision 1 (names only, never state) is invisible to an operator
until they type `hidden` and are surprised. This is the one place it
is said, so it is the one place a test can hold it.

### `fn field_hint`

*"Search layers"* and not *"Filter…"* or *"Type to filter"*. The
operator asked for *"a search to implement on the layers"* and that is
the word they used; a control whose label is not the word the person
asking for it used is a control they have to translate. "Filter" is also
the wrong promise in a small way — a filter usually implies a set of
fixed criteria you choose from, which is what a state filter would be
(see `panels::layers::search`'s Decision 1) and what this is not.

No ellipsis: it is a hint, not a command that opens something.

### `fn field_tooltip`

It states what is matched, because Decision 1 in
`panels::layers::search` is a decision an operator can otherwise only
discover by typing `hidden` and being surprised. One clause, at the one
moment they are looking at the control.

### `fn clear_label`

It exists at all because the field is drawn **above a list the
search may have emptied**, and an empty list is the one state in which
the operator most needs to undo the thing that caused it. Clearing a
text field by selecting and deleting is three gestures; this is one, and
it is beside the state it repairs.

A word rather than a `×` glyph: the icon set has no clear-field art, and
`icons::paint` draws a visible mark for an unknown key rather than
nothing — so an invented key would ship a placeholder rectangle, which
is precisely what R9 forbids.

### `fn narrowed`

`None` when the search removed nothing, which is the whole of rule 3: a
panel with no query in it says exactly what it said before this feature
existed.

It reports `shown of total` rather than `hidden`, and the two are not
interchangeable. The operator is looking at the list; the useful number
is the size of the thing in front of them and how much of the whole it
is. *"13 layers hidden"* makes them do the subtraction to find out
whether the one they want could still be there.

### `fn none_matched`

R9 says an unavailable capability renders nothing, and an empty list is
not a placeholder — but *"no rows"* and *"no rows because of what you
typed"* are different states, and the operator can see the layers are in
the document because they were on screen a moment ago.

The query is quoted back, per rule 2. The count of what was hidden
follows it, because the two together are the complete answer: *what you
asked for*, and *what is still there behind it*.

The query is **not** truncated. A pasted paragraph would make a long
line, and a long line in a narrow panel wraps — which is ugly and
correct. Eliding it would produce a sentence quoting something the
operator did not type, which for the one string whose job is to prove
the search ran is the one thing it must never do.
