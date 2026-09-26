# `pdfcer-gui/app/actions/dimensions`

Everything the ce-dimension feature asks the document to do.

# Rule 15 first, because this module is where it bites

A **ce dimension** is one *pdfcer itself authors*: a `/Line` annotation
carrying `/IT /LineDimension`, a baked `/AP`, and a record in the
document's `/PieceInfo` sidecar that says which group it belongs to and
what it measures. A **pdf dimension** is CAD-exported page content that
pdfcer reads and must not silently alter. Nothing in this file touches the
second kind. Every verb here names the first, and the bare word
"dimension" is not used on its own anywhere in it.

# Why this is its own file

**R2**, and the seam the sibling modules already draw: [`super::pages`] is
*what happens to a page*, [`super::annots`] is *what happens to an
annotation that already exists*, [`super::apply`] is *what happens to page
content*. This is **what happens to the dimensioning model** — the groups,
their scales and standards, the style cascade, and the per-ce-dimension
overrides.

It is a subject rather than a size-driven cut, and the evidence is that its
verbs share a property none of the others do: **most of them regenerate
appearance streams for annotations the operator is not looking at.** A
group's members are wherever they were placed, across any number of pages,
and a change to the group rewrites all of them. Every arm below has to
reason about that, and no arm anywhere else does.

# The two blast radii, which is what to know first

`EditSession`'s ce-dimension verbs come in two shapes, and confusing them
is the failure this module is arranged to prevent:

| shape | verbs | blast radius |
|---|---|---|
| **group** | `set_group_scale`, `set_group_standard`, `set_group_style`, `toggle_dimension_layer` | every member of the group, on every page |
| **one ce dimension** | `set_dimension_style`, `set_dimension_display`, `place_dimension`, `set_dimension_label`, the vertex verbs | exactly one annotation |

The group verbs therefore clear **every** cached raster rather than the
current page's, and they pass page `0` to [`super::apply::vector_edit`]
with a note, because a group is document-scoped and has no page. The
per-ce-dimension verbs pass the real page and invalidate normally.

# A returned count is not a count of anything visible

`set_group_style` and `set_group_standard` both return `usize`, and
`docs/core-api/03-capabilities.md` §1.6 trap (a) warns in as many words
that it is the number of members **regenerated** — which is every wired
member, including the ones that override the property being changed,
because regenerating an overrider produces byte-identical output and is
free in the diff.

The number an operator wants is how many will visibly **move**, which is a
strictly smaller set (the members whose `StyleProvenance` for the edited
property reports `follows_group() == true`) and which **must be computed
before the edit** if it is to be shown before the edit. That is the
surface's job, not this file's, and it is why no arm here discloses a
count. Disclosing the engine's number would be worse than disclosing
nothing: it is a real number, plausibly labelled, answering a different
question.

## Item notes

### `fn set_label`

# What is reported, and why the restore says something different

`DimensionLabelChange` carries `measured` and `printed` separately. When an
override goes on they differ, and the receipt names **both** — the operator
has just hidden a number and the one place that number must still be
available is the sentence about hiding it.

When the override comes **off**, `printed` becomes `measured` again and the
receipt says so plainly. That is not a formality: the whole reassurance this
feature rests on is that clearing the caption restores the *original*
measurement rather than re-measuring, and a receipt naming the number is
what lets an operator confirm it did.

`changed: false` produces no disclosure at all. The engine returns `Ok`
for a no-op — setting a caption to what it already says — and a sentence
there would evict a real disclosure to report that nothing happened.

### `fn every_group_verb_is_document_wide_and_every_other_is_not`

Written as a listing of values rather than as `matches!` a second time,
so the assertion is about the verbs themselves and not about a repeated
copy of the predicate's own pattern.

Note the limit: because `regenerates_the_whole_group` is a `matches!`
with an implicit fallback, a variant added to [`DimensionAction`] and
not added here silently answers `false`. Nothing in this test compels
the author to pick a side; only turning the predicate into an exhaustive
`match` would.
