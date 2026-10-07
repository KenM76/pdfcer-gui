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

### `fn regenerates_the_whole_group`

The module header's first table, expressed once as code so a new variant
cannot be added without picking a side. [`apply`] uses it to decide
whether to clear every cached raster, and the honest answer is derived
from *what the engine verb touches*, not from what the operator was
looking at when they asked.

[`Self::Commit`] is `false` even though it is the one that *creates* a
member: authoring places a single annotation on a single page, and the
group's other members are not redrawn by it.

Three verbs that look group-scoped answer `false`, each for its own
reason, which is why they are worth stating rather than leaving to the
`matches!`:

- [`Self::RenameGroup`] regenerates **nothing at all** — no member's
  appearance depends on what its group is called.
- [`Self::SetDimensionGroup`] regenerates **exactly one** annotation.
  Its label changes, which is startling and is still one annotation.
- [`Self::DeleteGroup`] regenerates **as many as its policy moves**,
  which is zero under `Refuse` and every member under `Reassign`. That
  is a property of the *policy* rather than of the verb, and a predicate
  taking `&self` cannot see inside the variant honestly — so [`apply`]
  decides it there, at the one place the policy is in hand.

### `fn apply`

# The two-step every arm shares

1. **Invalidate as widely as the verb reaches.** A group verb clears
   `doc.strip_rasters` wholesale, because a group's members are wherever the
   operator put them and a strip entry drawn before the edit would keep
   showing the old number with nothing to say so. This is the same
   wholesale-invalidation argument `app::pages` makes for a page
   permutation, arriving from a different direction.
2. **Mutate through [`super::apply::vector_edit`]**, so the
   cancel-mutate-bump-invalidate protocol, the undo entry, the trace line
   and the disclosure store are the ones every other edit in this
   application uses, rather than a second implementation of them here.

# Why the group arms pass page `0`

`vector_edit` takes a page for its trace line and its per-page raster drop.
A group is document-scoped and has no page, so `0` is passed with this note
rather than the signature gaining an `Option<usize>` that every other caller
would have to spell. The wholesale clear in step 1 is what actually
discharges the invalidation; the page reaches the funnel only as a label.

### `fn trace_members_shown`

Under `PDFCER_DIAG` only: after a group's scale or format changes, one
`dimension-member-shown group= dim= text="…"` line per member, read from
`model.display`. It is the label the operator now sees, which is the only
oracle for a unit conversion.
