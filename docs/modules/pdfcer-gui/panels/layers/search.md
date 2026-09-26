# `pdfcer-gui/panels/layers/search`

`panels::layers::search` — narrowing the Layers list as you type.

# Why this is its own file


The split is worth it for one reason beyond R2: a predicate in a file of
its own can be **swept** — every layer shape this project knows about,
against every query shape, with no window open. A predicate written
inline in a `for` loop inside a `ScrollArea` closure can only be tested
by looking at it.

# Decision 1: it matches THE NAME THE ROW SHOWS, and nothing else

The operator's ask was *"there is a search to implement on the layers"*.
The question that leaves open is whether a query also matches a layer's
**state** — so that typing `hidden` returns the hidden ones.

**It does not, and the reason is not that state matching is hard.**

> **A search result has to be explicable from the row it returned.**

Every row in this panel shows a name, a visible/hidden marker, and up to
six caveats hung off the name as tooltips. If a query matched state as
well as name, then a document with a layer genuinely called
*"Hidden Detail"* and eleven layers that are switched off would answer
`hidden` with twelve rows, one of which matched for a completely
different reason from the other eleven — and **nothing on screen would
say which**. The operator's next move is to conclude the search is
broken, and they would be right to.

⇒ Names only. The state is already legible on every row as a word (R84:
never colour or a glyph alone), so an operator who wants the hidden ones
can see them without asking. If a *state filter* is ever wanted it is a
different control — a pair of tick boxes, like `app::status::filter`'s
eleven-class pick filter — and it composes with this rather than hiding
inside it.

**"The name the ROW shows"**, not `Layer::name`. A layer whose
`/Name` is absent — a real malformation, since Table 98 makes it
Required — is drawn as [`crate::text::panels::layer_unnamed`]'s
placeholder rather than as an invented "Layer 3". A search that matched
the *underlying* field would leave that row unmatchable by anything the
operator can read, which is the same defect as matching state: the
result would not be explicable from the row.

# Decision 2: case-insensitive, substring, literal

Taken wholesale from [`crate::find::FindOptions`]'s default rather than
decided again, because this shell has already argued it and a second
answer in a second place is how two searches in one program come to
behave differently:

> *"an operator who types `total` and is not shown `TOTAL` on the next
> line reads that as a search that did not work."*

The one thing NOT carried across is the **Match case** control. `find`
offers it because it searches a document, where a case-sensitive search
is a real technique for a real problem. This searches at most
[`pdfcer_core::layers::MAX_LAYERS`] short labels in a panel narrow
enough that the whole list is usually visible, so a control to make the
search stricter would be a control for a problem the list's size
prevents.

ASCII case folding rather than Unicode, and that is a limitation stated
rather than hidden: `str::to_lowercase` is Unicode-correct and allocates
per row per frame, `str::eq_ignore_ascii_case` does neither. Layer names
in the wild are overwhelmingly CAD layer names — `HIDDEN`, `DIM`,
`A-WALL-FULL` — and a Turkish dotted İ in one would match on its bytes
rather than on its case-folded form. Worth the trade; worth saying.

# Decision 3: the query is TRIMMED, and an all-whitespace query is no
query

`redact`'s search field does the same and for the same reason: a
trailing space from a paste is invisible, and a filter that answered
"nothing matches" because of one would be a search that failed for a
reason the operator cannot see.

# What the empty result says, and why it says anything at all

R9 forbids a placeholder, and an empty list is not one — but it owes a
sentence, because *"no rows"* and *"no rows **because of what you
typed**"* are different states and the operator has to be able to tell
them apart. `panels::comments` is the precedent: its empty case still
discloses the filter, because *"a drawing whose every annotation is a
form field is a real and common shape, and 'no notes or markup' alone
would leave an operator who can see annotations on the page believing
the panel had failed."*

Here the equivalent is worse, because the operator can see the layers in
the document — they were on screen a moment ago. So the empty case says
the query back to them and how many rows it is hiding. See
[`crate::text::panels::layers_search_none`].

## Item notes

### `fn contains_ignore_ascii_case`

`str::contains` with a closure cannot express "case-insensitively", and
`to_lowercase().contains(&q.to_lowercase())` allocates two `String`s per
row per frame. This walks the byte windows instead: at most
`MAX_LAYERS` rows of a few dozen bytes, once per frame, with nothing on
the heap.

Byte windows are safe here despite UTF-8 being multi-byte, and the
reason is worth stating because it looks like a bug: `eq_ignore_ascii_case`
on two byte slices is `true` only when they are equal after folding
*ASCII* letters, and every non-ASCII byte must therefore match exactly.
A window that starts mid-character cannot match a needle that starts on
a character boundary unless the bytes are genuinely equal — in which
case it is a real match on the same bytes. So no false positive can be
produced by the slicing, and none can be lost either.

### `fn a_query_is_literal_and_a_star_is_a_character`

`find` offers wildcards behind a control; this does not, and a query
containing one must therefore match the character rather than
silently matching everything.

### `fn the_unnamed_placeholder_is_searchable_by_what_it_says`

Asserted through the placeholder the panel actually draws, so that a
change to that wording is caught here rather than leaving one row in
the list unmatchable by anything on screen.

### `fn an_empty_result_knows_it_was_the_query_that_emptied_it`

The whole reason `Filtered` counts rather than discards. Without
this distinction the panel's only available sentence is "nothing
here", which is also what a panel that failed to read the document
says — and R9's rule that an absent capability renders nothing is
exactly the rule that makes those two indistinguishable if the
count is thrown away.
