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
