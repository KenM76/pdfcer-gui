# `app::actions::pages` — the four page verbs, and the resync a structural
edit owes

[`super::apply`] is the interpreter for every [`Action`]; this file is the
body of the four that act on **pages** rather than on the marks drawn on
them — [`Action::RotatePages`], [`Action::DeletePages`],
[`Action::ReorderPages`] and [`Action::ExtractPages`] — plus the one
function every *other* edit in the application now calls as well,
[`resync`].

## ★ Why these four are not four more arms in `apply.rs`

Rule R2's own justification decides it, exactly as it decided the
`apply.rs` split from `actions.rs`: *"the value of the limit is that the
file has to have a single subject."* `apply.rs`'s subject is **the
cancel–mutate–bump–invalidate protocol** — what happens when a request to
change the document is granted, and the ordering that makes it safe. That
protocol is the same for every verb and changes only when the protocol
changes.

This file's subject is a different one, and it is the reason page verbs
could not simply be four more `vector_edit` calls:

> **A page index is a position, not an identity**, and the application
> holds four things that are stated in page indices.

Those four are the flattened page vector, every cached raster, the canvas's
object selection, and the Pages panel's own picks. The general rule is that
a selection is an identity — page, object, subpath, node — not a position,
and `crate::canvas::interact`'s header states the measured half of it:
*"`move_*` renumbers nothing … the `delete_*` family is the one that
renumbers."* A page delete is that sentence one structure up, and a page
**reorder** is a third case neither of them names.

## ★★ The table this whole file exists to implement

| | page vector | rasters | canvas selection | panel picks | `view.page_index` |
|---|---|---|---|---|---|
| markup, move, fill *(existing verbs)* | unchanged | current page's dropped | survives — resolved against the new epoch | untouched | valid |
| **rotate** | `/Rotate` differs | **all stale** — every turned page's picture is sideways | survives; a rotation adds and removes no operator | **survives** — the same sheets are still the same sheets | valid |
| **reorder** | order differs | **all stale** — page *N* is a different sheet | **cleared** | **remapped** — the permutation says where each went | valid, may now show a different sheet |
| **delete** | shorter | **all stale** | **cleared** | **cleared** — the picked sheets are gone | **may be past the end**; clamped |
| **extract** | unchanged | unchanged | untouched | untouched | valid |

Every row of that table is asserted below or in
[`crate::panels::pages::select`], because every cell is a way for the
application to end up drawing the wrong sheet or aiming a destructive verb
at one nobody chose — and none of them fails loudly.

## ★ [`resync`] is called from `vector_edit`, not from these four arms

That placement is the one design decision in this file worth arguing: it is
the one-choke-point rule applied to a *consequence* rather than to a
dispatch.

The naive arrangement is for each page arm to do its own tidying after its
own `vector_edit` call. It is wrong for a reason that is invisible until
undo exists — and undo exists: `Action::Undo` runs the *same* engine
commands backwards, through the *same* `vector_edit`, and an undone
`DeletePages` puts four sheets back. Tidying in the four arms would leave
the undo of a page delete showing a page vector that is four sheets short,
with a page count in the status bar to match, and nothing anywhere would
error.

So the resync sits at the one place every document change already passes
through, and it is **self-describing rather than told**: it compares the
page vector it has against the one the session now reports and acts on the
difference. A verb it has never heard of gets the right treatment, which is
what makes it a choke point rather than a fifth copy of a rule.

Its cost on an edit that changed no page is one page-tree walk and one
`Vec` comparison, paid **per operator gesture** rather than per frame. That
is the same order as the `Arc::get_mut` and the epoch bump beside it.

## What [`resync`] cannot do, and who does it instead

The **Pages panel's picks** live on `PdfcerApp::panels`, not on `OpenDoc`,
and `vector_edit` takes an `&mut OpenDoc`. They are therefore handled in
`apply.rs`'s two arms that know which edit ran — which is sound because
those are the only two verbs that can move a pick, and it is *stated* here
rather than left to be discovered, because the consequence of forgetting is
a Delete aimed at sheets the operator did not choose.

The **thumbnail cache** needs nothing at all: it is keyed on
`(edit_epoch, pixels_per_point)` and `ThumbnailCache::sync` empties itself
the moment the epoch moves, which `vector_edit` has already done by the time
the next frame draws the panel. That is the key-carries-the-staleness design
`apply.rs` wishes the page texture had.
