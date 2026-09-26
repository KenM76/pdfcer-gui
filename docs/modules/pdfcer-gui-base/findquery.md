# `findquery` — preparing a typed query for the engine

One subject, one file: the single decision about what the operator typed
versus what is handed to `EditSession::search_text`. It exists because that
decision now has a preference attached to it and a disclosure that has to
agree with both, and three things that must agree are three things that
drift when they live in three files.


Ken: *"trailing spaces/tabs/etc stops a search from finding text on the page
that doesn't have these symbols … copy pasting from excel seems to give a
trailing space that I have to remove to search."*

He is describing the engine behaving exactly as specified. `search_text`
matches the needle it is given; a needle with a trailing space asks for a
trailing space, and a drawing's title block does not have one. The defect is
not in the matching — it is that **nothing between his clipboard and the
engine ever looked at the query**, so an invisible character decided the
answer and no surface said so.

## Why trimming is the default and is still a preference

Defaulting to trim is defensible on its own: a leading or trailing space is
almost never what a person searching a drawing means, and the one reader who
*does* mean it — looking for `" … "` as a delimiter, or checking whether a
field was padded — is a reader who knows precisely what they are doing and
will find a switch.

But trimming silently would be the same defect wearing the other coat:
the operator would type a space, get hits, and have no way to learn that the
space was discarded. So the bar discloses it whenever the raw query has
whitespace at either end — see [`has_edge_whitespace`] and
`bar::whitespace_note` — off-canvas, in the bar's own second row, beside the
unsearchable-fonts note that already lives there. Rule 4: render normally,
report separately, and never mark the page.

## What is NOT trimmed, deliberately

**Interior whitespace.** `"PART  NUMBER"` with two spaces stays as typed.
Collapsing runs inside the needle would change which text matches in a way
the operator cannot predict from looking at their own query, and a PDF
genuinely can contain two spaces. The report is about the ends; the fix is
about the ends.

**The stored query.** [`crate::find::FindState::query`] keeps exactly what
was typed, so the text box still shows it, the caret still behaves, and
deleting the space by hand is still possible. Only the value handed to the
engine is prepared.

## Item notes

### `fn has_edge_whitespace`

The predicate behind the bar's disclosure row, and it is deliberately about
the RAW query rather than about whether trimming changed anything: the
operator is owed the sentence whether the setting is on (*"the space was
ignored"*) or off (*"the space is part of what you asked for"*). Those are
two different sentences about one observable fact, which is why the
predicate answers the fact and the caller chooses the sentence.

`str::trim` rather than a space test, because he wrote *"spaces/tabs/etc"*
and a clipboard hands over a tab, a non-breaking space or a zero-width
anything just as readily as a space — and all of them are invisible in a
one-line text box, which is the entire problem.

### `fn for_search`

Borrowing rather than allocating: with the setting off, or with a query that
has no edge whitespace — which is nearly every search — this returns the
caller's own string and costs nothing.

A query that is ENTIRELY whitespace trims to the empty string, and that is
correct rather than a hole: `find::search` treats an empty needle as *"you
have not typed anything"* rather than as *"there is nothing here"*, which is
exactly the right reading of a box containing one space. Without the trim
the operator would get a confident *"no results"* for a document that is
full of spaces.
