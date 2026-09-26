# `dialogs::insert_pages` — which pages, and where they land

The second half of `pages.insert_from_file`. The picker asks *which file*;
this asks the two questions a picker cannot.

## Why the dialog exists at all

A picker alone would satisfy the sentence *"add insert from file"* — take
every page of the chosen file, put them after the current one — and that is
not the feature. The operator:

> *"when I ask for something, my expectation is usually that everything
> surrounding that request is also done to where it would match the
> behaviour a user would expect. Otherwise I am left typing out every little
> missing detail."*

⇒ The standing test that falls out of it is *"what would a competent user
reach for next, within this same gesture?"*, and for an insert the answers
are immediate: **how many pages am I about to add**, **do I want all of
them**, and **where do they go**. Acrobat's own Insert Pages dialog asks the
last two and shows the first.

## The four positions are the engine's own vocabulary

`pdfcer_core::pageops::InsertPosition` is `Start` / `End` / `Before(n)` /
`After(n)`, and this dialog produces one directly rather than mapping
through a local enum. A second vocabulary for the same four choices would
be a second place for "before" and "after" to drift apart — and the drift
would be silent, because both spellings compile and both insert *somewhere*.

## The range grammar is the print dialog's, deliberately

[`crate::dialogs::print::tabs::parse_page_range`] parses `3`, `1-4`,
`5,1-2`, and its own header carries the argument for why there is exactly
one of it: two range parsers eventually disagree about something like
`5,1-2` — whether it reorders, whether it deduplicates — and an operator
moving between them has no way to know which one they are talking to.

That argument was made about the GUI and the CLI. It is the same argument
between two GUI surfaces, and stronger: an operator who learns the range
syntax on Print is entitled to it working here.

**And the order-preserving, non-deduplicating behaviour is a feature
here.** `5,1-2` inserts source page 5 first, then 1 and 2 — which is a
reorder an operator can ask for in one gesture. `1,1` inserts page 1 twice,
which is also legitimate. Both fall out of treating the text as a sequence,
and both match Print and the CLI.

## Rule 4: nothing here is drawn on the page

The dialog states what it will do and does it. The one inference pdfcer makes
on the operator's behalf is the **refusal** of an unparseable range, and
that is disclosed in words with the reason, never by silently inserting a
guess — the same posture the print dialog takes with the same parser.

## Item notes

### `enum Where`

A local enum **only** for the radio state, converted to
`pdfcer_core::pageops::InsertPosition` at the point of use — because two of
the four need the current page index, which the radio does not carry and the
dialog does.

### `fn chosen`

`None` is what disables the commit button *and* draws the refusal — one
derivation feeding both, so the button cannot be live while the sentence
says the range is bad.

### `fn each_position_maps_to_the_engines_own`

The failure this catches is an off-by-one between "after page 7" as the
operator reads it and `After(6)` as the engine takes it — invisible in
any test that only checks that *a* position was produced, and visible to
an operator as pages landing one sheet away from where they asked.

### `fn a_bad_range_names_nothing_and_does_not_fall_back_to_all`

Both halves matter: a bad range must not fall back to "all" — that would
insert a document the operator did not ask for — and an empty result
must be `None` rather than `Some(vec![])`, or the button would be drawn
over a selection of nothing.

### `fn it_opens_on_every_page_after_the_current_one`

Pinned because it is the fast path: an operator who wants the whole file
after the page they are on presses Insert twice and reads nothing. Any
change to the seeded state costs that operator a dialog they were not
reading.

### `fn the_range_is_a_sequence_not_a_set`

Order is preserved and duplicates are kept, because the text is a
SEQUENCE the operator wrote. Here that is not a quirk to tolerate — it
is how an operator inserts pages in a different order, or twice, in one
gesture. Asserted so that a later "tidy-up" into a sorted set has to
argue with a test rather than with nothing.
