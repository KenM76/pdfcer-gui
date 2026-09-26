# `panels::comments::filter` — narrowing the reviewer's work list

One subject: **which rows the Comments panel shows, and in what order.**
The state, the pure predicate, and the control strip that sets them.

## Why this exists — the gap the operator's report exposed

The operator, 2026-09-05: *"the review features should look and act the
same as they do in Acrobat Reader."* Acrobat's Comment pane is a **work
list**, and a work list you cannot narrow is a list you scroll. Its filter
offers reviewer, type, status and checkmark; its sort offers page, type,
author, date and colour.

This panel had **none of it**: every annotation in the document, in
document order, always. On `SW41177.pdf` — thirty-six sheets — a reviewer
looking for their own three comments scrolls past everybody else's.

## What is offered, and what is NOT, and why each absence

| Acrobat offers | here | why |
|---|---|---|
| filter by **reviewer** | ✅ [`Filter::author`] | `/T` is modelled and read |
| filter by **type** | ✅ [`Filter::subtype`] | `/Subtype` is modelled and read |
| filter to **comments with text** | ✅ [`Filter::with_note_only`] | this shell's own, and it earns its place: pdfcer's own markup authoring cannot write `/Contents` on a geometric shape, so a drawing pdfcer marked up is a column of "no note" rows with the reviewer's actual remarks scattered through it |
| filter by **status** (Accepted / Rejected / …) | ✅ [`Filter::status`] | **arrived 2026-09-06.** It was ❌ here for one day, on the grounds that *"`/State` and `/StateModel` have zero occurrences in `pdfcer-core` v0.38.0 — not read, not written, not modelled"*, filed as `request_review_status_is_not_modelled_at_all.md`. `Pass 253.1` closed it. The predicate is **not** in [`Filter::keeps`] and could not be — a status lives on *other* annotations (§12.5.6.3) — so it is answered by [`super::reviewstate::narrow`]; that field's doc carries the whole split |
| filter by **checkmark** | ❌ | a per-viewer flag Acrobat keeps outside the PDF. Not a document property, so not this panel's |
| sort by **page** | ✅ [`Sort::Document`], the default | |
| sort by **type** | ✅ [`Sort::Subtype`] | |
| sort by **author** | ✅ [`Sort::Author`] | |
| sort by **date** | ❌ | **`/M` is not reliably a date.** §12.5.2 gives its type as *"date **or** text string"* and requires a reader to accept any format, so `pdfcer-core` stores it raw and its own docs say *"do not assume it parses"*. A sort would have to either parse it — rejecting legal values — or sort the strings, which orders `D:2026…` before `17 January` and calls it chronology. `crate::text::panels::comments::comment_row_byline` makes the same ruling for display and this is it holding for ordering |
| sort by **colour** | ❌ | `/C` is **not in the engine's read model** at all — `annot.rs`'s parser reads `/CA` and never `/C`. Filed as `request_an_annotations_colour_cannot_be_read.md` |

⇒ Of the four absences this table opened with, **one has since closed** —
status, on the day after it was written — and it closed because the request
was filed rather than worked around. Two remain as engine gaps with requests
filed, and the fourth is a deliberate exclusion. None of them is drawn
greyed: R9.

That is worth leaving in place rather than tidying into a table of four
ticks: the row's history is the evidence for how this shell is supposed to
meet a capability it does not have — say what is missing, name the request,
draw nothing — and a table that had been rewritten to look as though status
was always there would have thrown that away.

## The disclosure that makes filtering safe

A filtered list is a list that is **lying by omission** unless it says so.
`crate::panels::comments`' founding discipline is that *"nothing is
silently omitted"* — its exclusion line already states the arithmetic for
widgets, pop-ups and `/TrapNet` — and a filter is a fourth kind of
omission, and the only one the operator caused.

So [`Filter::is_narrowing`] exists, and the panel draws
`comments_filtered(shown, total)` above the list whenever it is true. A
reviewer who set a filter, went to lunch and came back must not conclude
from six rows that the drawing carries six comments.

## The sort is STABLE, and that is load-bearing

`sort_by_key` on a `Vec` is stable in Rust, so sorting by author leaves
each author's comments in **document order** — page order then `/Annots`
order, which is `pdfcer list-annotations`' ordering, reused by name rather
than re-decided. An unstable sort would reshuffle a reviewer's own comments
on every frame, which reads as the panel flickering.

## Item notes

### `fn the_default_shows_everything`

The assertion that protects every operator who never opens the filter
strip: this panel's contract has always been *every annotation in the
document*, and a filter whose `Default` narrowed would silently change
what a surface means for everybody.

### `fn filtering_by_author_keeps_only_that_author`

Both halves asserted — the count *and* that no other author survives —
because a predicate that returned `true` for everything passes a bare
"Ken's comments are still here" check.

### `fn an_author_filter_does_not_match_a_prefix`

*Ken* and *Ken Mantle* are two reviewers. A substring match would fold
one into the other, and the operator would read one person's comments
under another's name — the worst available failure on a surface whose
whole subject is attribution.

### `fn with_text_only_drops_the_noteless_rows`

The filter this shell added that Acrobat does not have, and the reason
it earns its place: `MarkupSpec` has no contents field on any variant,
so every shape pdfcer draws arrives with no `/Contents`. On a drawing
marked up here, this is the switch that turns forty rows into the three
somebody actually wrote on.

### `fn a_description_counts_as_text`

§12.5.2 makes `/Contents` dual-purpose and the row already says which
meaning it carries. The filter asks *"is there anything to read"*, and
dropping a description would hide a string somebody wrote on the
grounds that it is not a *comment* — a distinction the operator did not
ask this switch to make.

### `fn sorting_by_author_is_stable_within_a_name`

The property that stops the list flickering. An unstable sort would
reshuffle one reviewer's own comments between frames — the panel
redraws sixty times a second — which reads as the surface being broken
rather than as an ordering choice.

### `fn an_unsigned_comment_sorts_last`

A reviewer ordering by author is looking for a person. `None` sorting
first — which is the derived order on `Option` and therefore what an
implementation gets for free — buries the name they asked for under
every anonymous row in the document.

### `fn sorting_alone_raises_no_disclosure`

Stated as a test because the obvious implementation of `is_narrowing`
is *"the filter is not `Default`"*, which would put a "some comments
are hidden" notice above a list that is hiding nothing.

### `enum Sort`

An enum rather than a pair of booleans for the reason
`crate::canvas::selection::annot::AnnotKind` is one: exactly one ordering
is in force, and a type that could say *by author* and *by type* at once is
a type whose illegal states are prevented by discipline instead of by
construction.

### `struct Filter`

`Default` is **everything**, which is the state the panel has always been
in — so a fresh profile behaves exactly as this panel did before the filter
existed, and nothing is hidden from an operator who never touches it.

### `fn is_narrowing`

The predicate the disclosure hangs off — see the module header. Note
that [`Self::sort`] is **not** part of it: reordering a list omits
nothing, and a "you have sorted this" notice would be noise attached to
a change the operator can see.
[`Self::status`] counts, even though [`Self::keeps`] cannot evaluate
it — see that field. This predicate answers *"is the operator being
shown less than the document holds"*, and where the answer is computed
has no bearing on it.

### `fn keeps`

Split out from [`apply`] so the rule can be asserted against a single
row without building a list — and so the three clauses are visible
together rather than spread through an iterator chain.

⚠ **It does not evaluate [`Self::status`]**, and cannot — a review
status is on other annotations, not on this row. See that field;
[`super::reviewstate::narrow`] is the second half of the pipeline.

### `fn apply`

Takes and returns owned rows rather than borrowing, because the caller
draws from the result and a borrow would keep the whole `Listing` alive
across the draw for no benefit — the rows are a handful of `String`s each
and a document with enough comments for that to matter has a layout cost
two orders of magnitude larger (see `crate::panels::comments`' cost note).

**Stable**, so an ordering by author or by kind preserves document order
within each group. See the module header.

### `fn authors`

# Sorted and de-duplicated, and blank names dropped

A chooser is a list of *people*, so it is alphabetical rather than in
document order — the operator is looking up a name, not walking the sheet.
A `/T` of `"  "` is a byline nobody wrote (the commonest way for one to
exist is a producer writing an empty string) and offering it would put a
blank row in the menu that filters to comments credited to a space. The
same trimming rule `crate::panels::comments::keeps_author_name` applies,
which is what keeps *"has an author"* meaning one thing across the surface.

### `fn subtypes`

The **file's own spelling**, never a friendly relabelling: `Square` is the
word every other surface in this shell uses for that annotation, and a
chooser offering "Rectangle" would be a fourth name for one thing.
