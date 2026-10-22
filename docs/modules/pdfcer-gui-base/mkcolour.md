# `mkcolour` — one `/MK` colour, wherever it is asked for

A labelled swatch over one of a widget's two `/MK` colour keys, `/BG` and
`/BC`. `OPERATOR_REQUESTS.md` **O202**, whose ask covers both halves of the
life of a form box: *"the forms objects have no way to edit their colour
before or after placement."*

## Why this is its own module rather than two similar rows

Before placement the answer goes into a [`crate::canvas::formfield::Draft`]
field; after placement it goes into a `WidgetEdit` and an undo entry. Those
are different destinations, but every question in front of them is the same
one — Table 189 gives each key four states, two of which are not a colour,
and each of those owes the operator a **mark** for the button face and a
**note** in the popup. A colour is shown and edited in the space the file
states — grey, RGB or CMYK — through the shared picker (O321). Written
twice, the placement dialog and the properties pane would answer these
questions differently within a month.

So this module owns the reading, and the two callers own only where the
answer goes.

## What it deliberately does not own

The **sentences**. Every string arrives through [`Row`], because what *no
colour* means is a fact about which key this is — a background stating it
paints nothing at all, a border stating it is still drawn black — and that
is knowledge `text::panels::formfield` holds, not this file.

## Rule 4

Nothing here marks the canvas, in either caller. After placement the colour
is applied and from that instant the page shows what the saved file will
show. Before placement there is no content yet to mark; the swatch is part
of the cursor, not part of the document.

## Item notes

### `fn no_colour_would_change_something`

False only when the key already holds the empty array. Named rather than
inlined so [`the_two_entries_never_offer_the_same_state`] can walk Table
189's states against both predicates at once.

### `fn removal_would_change_something`

False only when the key is already absent — including when it is absent and
the operator is looking at a *no colour* entry that is live, which is the
pair everyone reads backwards.

### `fn mk_value`

Separate from [`row`] because this is the whole of Table 189's four-state
reading and the only part a test can reach without a live `Ui`. The
file's numbers pass through unclamped: an out-of-range component is a
malformed file, not a value to silently correct, and the swatch clamps only
on the way to the screen.

### `fn a_mk_key_reaches_the_swatch_in_four_distinguishable_states`

The two that matter and would not be noticed if they collapsed: a CMYK
value reaches the picker as CMYK rather than a conversion, and *the file is silent*
carries a different face from *the file states no colour*. The second is
the whole reason the engine models the key as `Option<MkColor>` with an
`MkColor::None` inside, and a surface showing one glyph for both would
throw that distinction away where the operator reads it.

### `fn the_two_entries_never_offer_the_same_state`

The pair this guards is the one that reads backwards: an **absent** key
offers *no colour* and not *remove*, and an **empty** key offers
*remove* and not *no colour*. Swap the two predicates and every state
still lights exactly one entry, so nothing short of the full table
catches it.

### `enum Pick`

The shell's own enum rather than `pdfcer_core::edit::MkColorEdit`, which
carries the same two states. `MkColorEdit` is `#[non_exhaustive]`, so
matching it here would need a catch-all arm — and a catch-all is exactly
what [`mk_value`] refuses, for the reason written there: a state added to
the engine's enum must break this file's build rather than fall silently
into a default. The conversion is one `match` in each caller, at the point
where the answer's destination is already known.

### `fn row`

`None` on every other frame, including every frame of a drag inside the
picker — see [`super::swatch`]'s header for why that matters and what it
costs when it is got wrong.
